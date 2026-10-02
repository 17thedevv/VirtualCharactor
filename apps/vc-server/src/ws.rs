//! WebSocket connection routing, event multiplexing, and heartbeat loop.
//!
//! Under clean architecture guidelines:
//! - ws.rs is purely an I/O gateway (handling client connections, ping/pong, resets, interrupts).
//! - All conversational logic, context engineering, and decision making are delegated to `conversation::handler`.
//! - Real-time token streaming and audio chunk synthesis are delegated to `streaming::pipeline`.

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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::conversation::build_emotion_json;
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEvent {
    Message {
        text: String,
        #[serde(default = "default_actor")]
        actor_id: String,
    },
    Interrupt {
        heard_text: Option<String>,
        playback_ms: Option<u64>,
    },
    HeardProgress {
        delta: String,
    },
    Ping,
    Reset,
}

fn default_actor() -> String {
    "user-default".into()
}

fn chrono_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // 1. Send initial connected state
    let char_name = state.character.read().await.name.clone();
    let current_emotion = state.character_state.read().await.emotion.clone();
    let current_rel = state.relationship.read().await.state.clone();

    let init_msg = json!({
        "event": "connected",
        "character_name": char_name,
        "emotion": build_emotion_json(&current_emotion),
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

    // Cancellation token shared across in-flight turns for this connection
    let cancellation_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    // 2. Background Circadian Heartbeat Task (2.5s tick)
    let (hb_tx, mut hb_rx) = tokio::sync::mpsc::channel::<serde_json::Value>(16);
    let auto_engine = state.autonomous_engine.clone();
    let hb_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(2500));
        loop {
            interval.tick().await;
            let now_f64 = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();

            let reflex = {
                let mut auto = auto_engine.lock().await;
                if auto.is_shutdown() {
                    break;
                }
                auto.tick_subconscious_reflex(now_f64, None, None, 0.2)
            };

            let ev = json!({
                "event": "heartbeat_tick",
                "reflex": {
                    "should_blink": reflex.should_blink,
                    "breathing_rate": reflex.breathing_rate,
                    "posture_sway": reflex.posture_sway
                }
            });

            if hb_tx.send(ev).await.is_err() {
                break;
            }
        }
    });

    // 3. Event Loop multiplexing Client Messages and Heartbeat Ticks
    loop {
        tokio::select! {
            maybe_hb = hb_rx.recv() => {
                if let Some(hb_ev) = maybe_hb {
                    if sender.send(Message::Text(hb_ev.to_string().into())).await.is_err() {
                        break;
                    }
                }
            }
            client_msg = receiver.next() => {
                match client_msg {
                    Some(Ok(Message::Text(text))) => {
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
                                ClientEvent::Interrupt { heard_text, playback_ms } => {
                                    println!("🛑 [WS] Received Barge-In interrupt signal (playback_ms: {:?})", playback_ms);
                                    cancellation_flag.store(true, Ordering::SeqCst);
                                    let now = chrono_now_secs();
                                    let res = state.conversation_fsm.write().await.interrupt(now, heard_text);
                                    state.tts_queue.lock().await.cancel_and_clear();

                                    let final_heard = if let Ok(int_res) = res {
                                        int_res.heard_text
                                    } else {
                                        String::new()
                                    };

                                    let _ = sender
                                        .send(Message::Text(
                                            json!({
                                                "event": "speech_interrupted",
                                                "heard_text": final_heard,
                                                "timestamp": now,
                                            })
                                            .to_string().into(),
                                        ))
                                        .await;
                                }
                                ClientEvent::HeardProgress { delta } => {
                                    state.conversation_fsm.write().await.update_heard_progress(&delta);
                                }
                                ClientEvent::Message { text: user_input, actor_id } => {
                                    cancellation_flag.store(false, Ordering::SeqCst);
                                    crate::conversation::process_user_interaction(
                                        &mut sender,
                                        &state,
                                        user_input,
                                        actor_id,
                                        cancellation_flag.clone(),
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    hb_handle.abort();
}
