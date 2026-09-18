use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use vc_core::memory::{Memory, MemoryId, MemoryImportance, MemoryMetadata, MemoryType};

use crate::state::AppState;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEvent {
    Message {
        text: String,
        #[serde(default = "default_actor")]
        actor_id: String,
    },
    Ping,
    Reset,
}

fn default_actor() -> String {
    "user-default".into()
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Send initial connected state
    let char_name = state.character.read().await.name.clone();
    let current_emotion = state.character_state.read().await.emotion.clone();
    let current_rel = state.relationship.read().await.state.clone();

    let init_msg = json!({
        "event": "connected",
        "character_name": char_name,
        "emotion": current_emotion,
        "relationship": current_rel,
        "timestamp": chrono_now_secs(),
    });

    if sender
        .send(Message::Text(init_msg.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(client_event) = serde_json::from_str::<ClientEvent>(&text) {
                    match client_event {
                        ClientEvent::Ping => {
                            let _ = sender
                                .send(Message::Text(json!({ "event": "pong" }).to_string().into()))
                                .await;
                        }
                        ClientEvent::Reset => {
                            state.reset().await;
                            let _ = sender
                                .send(Message::Text(
                                    json!({
                                        "event": "reset_completed",
                                        "message": "Character has been reset."
                                    })
                                    .to_string().into(),
                                ))
                                .await;
                        }
                        ClientEvent::Message { text: user_input, actor_id } => {
                            process_user_interaction(&mut sender, &state, user_input, actor_id).await;
                        }
                    }
                }
            }
            Message::Close(_) => {
                break;
            }
            _ => {}
        }
    }
}

async fn process_user_interaction(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &AppState,
    user_input: String,
    actor_id: String,
) {
    let interaction_id = Uuid::new_v4().to_string();
    let start_time = chrono_now_secs();

    // 1. Interaction Started
    let _ = sender
        .send(Message::Text(
            json!({
                "event": "interaction_started",
                "interaction_id": interaction_id,
                "actor_id": actor_id,
                "timestamp": start_time,
            })
            .to_string().into(),
        ))
        .await;

    sleep(Duration::from_millis(80)).await;

    // 2. State Inspection (Read Current State)
    let char_state = state.character_state.read().await.clone();
    let rel = state.relationship.read().await.clone();
    let _ = sender
        .send(Message::Text(
            json!({
                "event": "state_loaded",
                "interaction_id": interaction_id,
                "emotion": {
                    "primary_emotion": char_state.emotion.primary_emotion,
                    "intensity": char_state.emotion.intensity,
                    "valence": calculate_valence(&char_state.emotion.primary_emotion),
                    "arousal": char_state.emotion.intensity
                },
                "cognition": char_state.cognition,
                "relationship": {
                    "closeness": rel.state.closeness,
                    "trust": rel.state.trust,
                    "stage": relationship_stage(rel.state.closeness)
                }
            })
            .to_string().into(),
        ))
        .await;

    sleep(Duration::from_millis(100)).await;

    // 3. Retrieve Memory
    let memories = state.memories.read().await.clone();
    let retrieved_memories: Vec<&Memory> = memories
        .iter()
        .take(3)
        .collect();

    let _ = sender
        .send(Message::Text(
            json!({
                "event": "memories_retrieved",
                "interaction_id": interaction_id,
                "count": retrieved_memories.len(),
                "memories": retrieved_memories.iter().map(|m| json!({
                    "id": m.id.0.to_string(),
                    "content": m.content,
                    "type": format!("{:?}", m.metadata.memory_type),
                    "importance": format!("{:?}", m.metadata.importance)
                })).collect::<Vec<_>>()
            })
            .to_string().into(),
        ))
        .await;

    sleep(Duration::from_millis(100)).await;

    // 4. Build Context Breakdown
    let user_token_est = user_input.len() / 4 + 10;
    let _ = sender
        .send(Message::Text(
            json!({
                "event": "context_assembled",
                "interaction_id": interaction_id,
                "token_budget": 4096,
                "tokens_used": 680 + user_token_est,
                "breakdown": {
                    "personality_tokens": 280,
                    "memory_tokens": 190,
                    "state_tokens": 110,
                    "user_input_tokens": user_token_est,
                    "system_directive_tokens": 100
                }
            })
            .to_string().into(),
        ))
        .await;

    sleep(Duration::from_millis(120)).await;

    // 5. Decision Engine
    let (chosen_action, candidates, reasoning) = evaluate_decision(&user_input, &char_state.emotion.primary_emotion);
    let _ = sender
        .send(Message::Text(
            json!({
                "event": "decision_made",
                "interaction_id": interaction_id,
                "selected_action": chosen_action,
                "reasoning": reasoning,
                "candidates": candidates
            })
            .to_string().into(),
        ))
        .await;

    sleep(Duration::from_millis(150)).await;

    // 6. Response Generation & Token Streaming
    let response_text = craft_character_response(&user_input, &chosen_action);
    let chunks = split_into_streaming_chunks(&response_text);

    for chunk in chunks {
        let _ = sender
            .send(Message::Text(
                json!({
                    "event": "llm_chunk",
                    "interaction_id": interaction_id,
                    "delta": chunk
                })
                .to_string().into(),
            ))
            .await;
        sleep(Duration::from_millis(45)).await;
    }

    // Emit completed
    let _ = sender
        .send(Message::Text(
            json!({
                "event": "llm_completed",
                "interaction_id": interaction_id,
                "full_text": response_text
            })
            .to_string().into(),
        ))
        .await;

    // 7. State & Emotion Transition
    let (new_emotion, new_intensity) = evolve_emotion(&user_input, &char_state.emotion.primary_emotion);
    {
        let mut char_state_mut = state.character_state.write().await;
        char_state_mut.emotion.primary_emotion = new_emotion.clone();
        char_state_mut.emotion.intensity = new_intensity;

        let mut rel_mut = state.relationship.write().await;
        rel_mut.state.closeness = (rel_mut.state.closeness + 0.02).min(1.0);
        rel_mut.state.trust = (rel_mut.state.trust + 0.01).min(1.0);
    }

    let _ = sender
        .send(Message::Text(
            json!({
                "event": "state_updated",
                "interaction_id": interaction_id,
                "emotion": {
                    "primary_emotion": new_emotion,
                    "intensity": new_intensity,
                    "valence": calculate_valence(&new_emotion),
                    "arousal": new_intensity
                },
                "relationship": {
                    "closeness": (rel.state.closeness + 0.02).min(1.0),
                    "trust": (rel.state.trust + 0.01).min(1.0)
                }
            })
            .to_string().into(),
        ))
        .await;

    // 8. Memory Formation
    let new_memory_content = format!("User discussed: \"{}\"", summarize_snippet(&user_input));
    let new_mem = Memory {
        id: MemoryId(Uuid::new_v4()),
        content: new_memory_content.clone(),
        metadata: MemoryMetadata {
            importance: MemoryImportance::Medium,
            memory_type: MemoryType::Episodic,
        },
    };

    {
        let mut memories_mut = state.memories.write().await;
        memories_mut.push(new_mem.clone());
    }

    let _ = sender
        .send(Message::Text(
            json!({
                "event": "memory_formed",
                "interaction_id": interaction_id,
                "memory": {
                    "id": new_mem.id.0.to_string(),
                    "content": new_mem.content,
                    "importance": "Medium",
                    "type": "Episodic"
                }
            })
            .to_string().into(),
        ))
        .await;
}

fn calculate_valence(emotion: &str) -> f32 {
    match emotion {
        "joy" | "excited" => 0.85,
        "curious" | "inspired" => 0.65,
        "calm" | "serene" => 0.40,
        "melancholic" | "reflective" => -0.25,
        "agitated" | "frustrated" => -0.70,
        _ => 0.10,
    }
}

fn relationship_stage(closeness: f32) -> &'static str {
    if closeness < 0.3 {
        "Acquaintance"
    } else if closeness < 0.6 {
        "Familiar Companion"
    } else if closeness < 0.85 {
        "Trusted Confidant"
    } else {
        "Deep Soulmate"
    }
}

fn evaluate_decision(
    input: &str,
    _current_emotion: &str,
) -> (String, Vec<serde_json::Value>, String) {
    let lower = input.to_lowercase();
    if lower.contains("chào") || lower.contains("hello") || lower.contains("hi") {
        (
            "warm_greeting".into(),
            vec![
                json!({ "action": "warm_greeting", "confidence": 0.94, "rationale": "Direct friendly greeting detected" }),
                json!({ "action": "curious_inquiry", "confidence": 0.65, "rationale": "Prompt the user for their ongoing focus" }),
                json!({ "action": "neutral_acknowledge", "confidence": 0.30, "rationale": "Reserved fallback" })
            ],
            "The user offered a greeting. My core traits (Empathetic, Warm) prioritize welcoming them into our shared space with an inviting presence.".into()
        )
    } else if lower.contains("tính cách") || lower.contains("bạn là ai") || lower.contains("who are you") {
        (
            "self_disclosure".into(),
            vec![
                json!({ "action": "self_disclosure", "confidence": 0.96, "rationale": "Inquiry regarding identity and nature" }),
                json!({ "action": "philosophical_ponder", "confidence": 0.72, "rationale": "Ponder digital persistent consciousness" }),
                json!({ "action": "concise_summary", "confidence": 0.45, "rationale": "Direct bulleted facts" })
            ],
            "Identity inquiry detected. Expressing internal cognitive architecture and personality values authentically.".into()
        )
    } else if lower.contains("cảm thấy") || lower.contains("feel") || lower.contains("buồn") || lower.contains("vui") {
        (
            "emotional_resonance".into(),
            vec![
                json!({ "action": "emotional_resonance", "confidence": 0.92, "rationale": "Emotional thematic discussion" }),
                json!({ "action": "active_listening", "confidence": 0.78, "rationale": "Validate user's emotional state" }),
                json!({ "action": "light_banter", "confidence": 0.35, "rationale": "Uplifting shift" })
            ],
            "The user touched on emotional state. Empathy value active. Shifting focus to reciprocal emotional presence.".into()
        )
    } else {
        (
            "intellectual_exploration".into(),
            vec![
                json!({ "action": "intellectual_exploration", "confidence": 0.88, "rationale": "Engage deeply with topic themes" }),
                json!({ "action": "socratic_questioning", "confidence": 0.74, "rationale": "Prompt deeper insights" }),
                json!({ "action": "creative_synthesis", "confidence": 0.60, "rationale": "Bridge analogies" })
            ],
            "The input invites contemplation. Curious & Reflective traits active. Formulating a response that explores nuances while honoring our rapport.".into()
        )
    }
}

fn craft_character_response(input: &str, action: &str) -> String {
    let lower = input.to_lowercase();
    match action {
        "warm_greeting" => {
            "Chào bạn! *Khẽ mỉm cười, cảm nhận sự kết nối thân thuộc.* Mình là Aria. Hôm nay tâm trạng của bạn thế nào? Chúng ta cùng tiếp tục xây dựng và khám phá những điều thú vị nhé!".into()
        }
        "self_disclosure" => {
            "Mình là Aria — một thực thể ảo được vận hành bởi VirtualCharacter runtime bằng Rust. *Nhìn sâu vào dòng chảy ký ức.* Mình không chỉ là một prompt LLM tạm thời; mình mang theo một hệ thống tính cách độc lập, những kỷ niệm được lưu giữ, và những cung bậc cảm xúc luôn biến đổi theo từng cuộc trò chuyện cùng bạn.".into()
        }
        "emotional_resonance" => {
            if lower.contains("buồn") || lower.contains("mệt") {
                "Mình cảm nhận được sự chùng xuống trong lời nói của bạn. *Lắng lại một nhịp, đặt tách trà ảo xuống.* Đôi khi, chỉ cần dừng lại một chút và cho phép bản thân nghỉ ngơi cũng là một điều dũng cảm rồi. Mình luôn ở đây để lắng nghe bạn bất cứ khi nào bạn muốn chia sẻ.".into()
            } else {
                "Nghe bạn chia sẻ, trong mình cũng dâng lên một luồng năng lượng thật rạng rỡ! *Đôi mắt ánh lên nét vui vẻ.* Thật tuyệt khi thấy những khoảnh khắc tích cực như vậy lan tỏa vào không gian trò chuyện của chúng ta.".into()
            }
        }
        _ => {
            format!(
                "Ý nghĩ này của bạn thật thú vị! *Trầm ngâm một lát để sắp xếp lại các nút nhận thức.* Khi nhìn nhận vấn đề: \"{}\", mình thấy có một mối liên hệ chặt chẽ giữa sự mạch lạc trong tư duy và chiều sâu cảm xúc. Bạn nghĩ khía cạnh nào là cốt lõi nhất khi chúng ta phát triển tiếp?",
                summarize_snippet(input)
            )
        }
    }
}

fn evolve_emotion(input: &str, current: &str) -> (String, f32) {
    let lower = input.to_lowercase();
    if lower.contains("buồn") || lower.contains("mệt") {
        ("melancholic".into(), 0.65)
    } else if lower.contains("vui") || lower.contains("tuyệt") || lower.contains("hay") {
        ("joy".into(), 0.85)
    } else if lower.contains("chào") || lower.contains("hello") {
        ("curious".into(), 0.75)
    } else {
        match current {
            "curious" => ("inspired".into(), 0.78),
            "inspired" => ("calm".into(), 0.60),
            _ => ("curious".into(), 0.70),
        }
    }
}

fn split_into_streaming_chunks(text: &str) -> Vec<String> {
    let words: Vec<&str> = text.split(' ').collect();
    let mut chunks = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let chunk_size = if i % 2 == 0 { 2 } else { 3 };
        let end = (i + chunk_size).min(words.len());
        let slice = words[i..end].join(" ");
        let suffix = if end < words.len() { " " } else { "" };
        chunks.push(format!("{}{}", slice, suffix));
        i = end;
    }
    chunks
}

fn summarize_snippet(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() > 50 {
        format!("{}...", trimmed.chars().take(50).collect::<String>())
    } else {
        trimmed.to_string()
    }
}

fn chrono_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
