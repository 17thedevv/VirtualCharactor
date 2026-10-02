//! Real-Time Streaming Brain to Voice Pipeline (P0-A, P0-C & P0-D).
//!
//! Orchestrates the live end-to-end token flow:
//! Ollama Token Stream -> TextStreamChunker -> OrderedTtsQueue -> Base64 WAV + RMS 20ms -> WebSocket.
//!
//! Replaces fake streaming (generate_text + sleep) with zero-latency real token streaming.
//! Immediately halts and flushes when a Barge-In cancellation is triggered.

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use futures::SinkExt;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use vc_llm::provider::{LlmProvider, LlmRequest};
use vc_runtime::audio::{clean_narrative_text, OrderedTtsQueue, TextStreamChunker};
use vc_runtime::conversation::ConversationStateMachine;
use vc_runtime::vision::OllamaVisionProvider;

/// Outcome of the streaming execution.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum StreamingOutcome {
    Completed {
        full_text: String,
        sentence_count: usize,
    },
    Interrupted {
        heard_text: String,
        unspoken_text: String,
    },
    Failed {
        error: String,
    },
}

/// Execute real token streaming from LLM and parallel audio synthesis to WebSocket.
pub async fn stream_brain_to_voice(
    llm_provider: Arc<dyn LlmProvider>,
    llm_request: LlmRequest,
    sender: &mut SplitSink<WebSocket, Message>,
    conversation_fsm: Arc<RwLock<ConversationStateMachine>>,
    tts_queue: Arc<Mutex<OrderedTtsQueue>>,
    interaction_id: &str,
    char_name: &str,
    cancellation_flag: Arc<AtomicBool>,
) -> StreamingOutcome {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // 1. Reset TTS queue state
    {
        let mut queue = tts_queue.lock().await;
        queue.reset();
    }

    // 2. Open LLM token stream
    let token_stream = match llm_provider.stream_text(llm_request.clone()) {
        Ok(stream) => stream,
        Err(err) => {
            eprintln!(
                "⚠️ [STREAMING] LLM stream_text error: {}, falling back to generate_text",
                err
            );
            match llm_provider.generate_text(llm_request) {
                Ok(resp) => Box::new(std::iter::once(Ok(resp.text))),
                Err(e) => {
                    return StreamingOutcome::Failed {
                        error: format!("LLM generation failed: {}", e),
                    };
                }
            }
        }
    };

    let mut chunker = TextStreamChunker::default_tts();
    let mut full_raw_text = String::new();
    let mut current_sentence_seq: usize = 0;
    let mut is_first_sentence = true;

    // Check installed TTS provider (Windows SAPI or Mock)
    #[cfg(target_os = "windows")]
    let sapi_engine = vc_runtime::audio::WindowsSapiTtsProvider::default_female();

    #[cfg(not(target_os = "windows"))]
    let mock_tts = vc_runtime::audio::MockTtsProvider::new(22050);

    // 3. Process tokens as they arrive from Ollama
    for token_res in token_stream {
        // Check cancellation (Barge-In)
        if cancellation_flag.load(Ordering::SeqCst) {
            println!("🛑 [STREAMING] Cancellation triggered mid-stream!");
            let mut queue = tts_queue.lock().await;
            queue.cancel_and_clear();
            return StreamingOutcome::Interrupted {
                heard_text: String::new(),
                unspoken_text: full_raw_text,
            };
        }

        let token = match token_res {
            Ok(tok) => tok,
            Err(e) => {
                eprintln!("⚠️ [STREAMING] Error reading token: {}", e);
                break;
            }
        };

        if token.is_empty() {
            continue;
        }

        full_raw_text.push_str(&token);

        // Emit real-time LLM token chunk immediately to WebSocket (NO sleep!)
        let chunk_msg = json!({
            "event": "llm_chunk",
            "interaction_id": interaction_id,
            "delta": token
        });
        let _ = sender
            .send(Message::Text(chunk_msg.to_string().into()))
            .await;

        // Feed to sentence boundary chunker
        let sentences = chunker.feed(&token);
        for sentence_raw in sentences {
            if cancellation_flag.load(Ordering::SeqCst) {
                break;
            }

            let cleaned = clean_narrative_text(&sentence_raw);
            if cleaned.trim().is_empty() {
                continue;
            }

            // Synthesize sentence chunk
            #[cfg(target_os = "windows")]
            let synth_res = OrderedTtsQueue::synthesize_sentence(
                &sapi_engine,
                current_sentence_seq,
                &cleaned,
                None,
            );

            #[cfg(not(target_os = "windows"))]
            let synth_res = OrderedTtsQueue::synthesize_sentence(
                &mock_tts,
                current_sentence_seq,
                &cleaned,
                None,
            );

            if let Ok(audio_chunk) = synth_res {
                let mut queue = tts_queue.lock().await;
                queue.insert_synthesized_chunk(audio_chunk);
            }
            current_sentence_seq += 1;

            // Drain any ready audio chunks and send to WebSocket
            let ready_chunks = {
                let mut queue = tts_queue.lock().await;
                queue.drain_ready()
            };

            for audio_chunk in ready_chunks {
                if is_first_sentence {
                    is_first_sentence = false;
                    let _ = conversation_fsm.write().await.on_first_audio_ready(
                        now,
                        interaction_id,
                        &audio_chunk.text,
                    );
                    let _ = sender
                        .send(Message::Text(
                            json!({
                                "event": "speaking_started",
                                "interaction_id": interaction_id,
                                "first_sentence": audio_chunk.text,
                                "timestamp": now
                            })
                            .to_string()
                            .into(),
                        ))
                        .await;
                }

                conversation_fsm
                    .write()
                    .await
                    .update_heard_progress(&audio_chunk.text);

                let audio_b64 = OllamaVisionProvider::base64_encode(&audio_chunk.wav_bytes);
                let audio_packet = json!({
                    "event": "audio_sentence_chunk",
                    "interaction_id": interaction_id,
                    "sequence": audio_chunk.sequence,
                    "text": audio_chunk.text,
                    "audio_base64": audio_b64,
                    "sample_rate": audio_chunk.sample_rate,
                    "rms_slices": audio_chunk.rms_slices,
                    "duration_ms": audio_chunk.duration_ms
                });
                let _ = sender
                    .send(Message::Text(audio_packet.to_string().into()))
                    .await;
            }
        }
    }

    // 4. Flush remaining sentence
    if let Some(trailing_sentence) = chunker.flush() {
        if !cancellation_flag.load(Ordering::SeqCst) {
            let cleaned = clean_narrative_text(&trailing_sentence);
            if !cleaned.trim().is_empty() {
                #[cfg(target_os = "windows")]
                let synth_res = OrderedTtsQueue::synthesize_sentence(
                    &sapi_engine,
                    current_sentence_seq,
                    &cleaned,
                    None,
                );

                #[cfg(not(target_os = "windows"))]
                let synth_res = OrderedTtsQueue::synthesize_sentence(
                    &mock_tts,
                    current_sentence_seq,
                    &cleaned,
                    None,
                );

                if let Ok(audio_chunk) = synth_res {
                    let mut queue = tts_queue.lock().await;
                    queue.insert_synthesized_chunk(audio_chunk);
                }
                current_sentence_seq += 1;
            }
        }
    }

    // 5. Drain all remaining buffered audio chunks
    let remaining_chunks = {
        let mut queue = tts_queue.lock().await;
        queue.drain_ready()
    };

    for audio_chunk in remaining_chunks {
        if is_first_sentence {
            is_first_sentence = false;
            let _ = conversation_fsm.write().await.on_first_audio_ready(
                now,
                interaction_id,
                &audio_chunk.text,
            );
        }

        conversation_fsm
            .write()
            .await
            .update_heard_progress(&audio_chunk.text);

        let audio_b64 = OllamaVisionProvider::base64_encode(&audio_chunk.wav_bytes);
        let audio_packet = json!({
            "event": "audio_sentence_chunk",
            "interaction_id": interaction_id,
            "sequence": audio_chunk.sequence,
            "text": audio_chunk.text,
            "audio_base64": audio_b64,
            "sample_rate": audio_chunk.sample_rate,
            "rms_slices": audio_chunk.rms_slices,
            "duration_ms": audio_chunk.duration_ms
        });
        let _ = sender
            .send(Message::Text(audio_packet.to_string().into()))
            .await;
    }

    // 6. Complete normal speech
    let sanitized_full =
        crate::conversation::sanitize_character_dialogue(&full_raw_text, char_name);
    let _ = sender
        .send(Message::Text(
            json!({
                "event": "llm_completed",
                "interaction_id": interaction_id,
                "full_text": sanitized_full
            })
            .to_string()
            .into(),
        ))
        .await;

    let _ = conversation_fsm.write().await.complete_speech(now);
    let _ = sender
        .send(Message::Text(
            json!({
                "event": "speech_completed",
                "interaction_id": interaction_id,
                "timestamp": now
            })
            .to_string()
            .into(),
        ))
        .await;

    StreamingOutcome::Completed {
        full_text: sanitized_full,
        sentence_count: current_sentence_seq,
    }
}
