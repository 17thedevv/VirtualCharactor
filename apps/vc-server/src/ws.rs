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
use vc_core::personality::Personality;
use vc_core::relationship::Relationship;
use vc_core::state::{CharacterState, EmotionEngine};


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

    let (_dominant_axis, _dominant_score) = current_emotion.dominant_emotion();
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
                "emotion": build_emotion_json(&char_state.emotion),
                "cognition": {
                    "attention": char_state.cognition.attention.value(),
                    "confusion": char_state.cognition.confusion.value(),
                    "curiosity": char_state.cognition.curiosity.value(),
                    "confidence": char_state.cognition.confidence.value(),
                    "focus": char_state.cognition.focus.value(),
                    "current_topic": char_state.cognition.current_topic,
                },
                "behavior": {
                    "playfulness": char_state.behavior.playfulness.value(),
                    "seriousness": char_state.behavior.seriousness.value(),
                    "verbosity": char_state.behavior.verbosity.value(),
                    "initiative": char_state.behavior.initiative.value(),
                    "current_activity": char_state.behavior.current_activity,
                },
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
                    "state_tokens": 150,
                    "user_input_tokens": user_token_est,
                    "system_directive_tokens": 100
                }
            })
            .to_string().into(),
        ))
        .await;

    sleep(Duration::from_millis(120)).await;

    // 5. Decision Engine
    let (dominant_axis, _) = char_state.emotion.dominant_emotion();
    let (chosen_action, candidates, reasoning) = evaluate_decision(&user_input, dominant_axis.name());
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
    let personality = state.personality.read().await.clone();
    let llm_request = build_companion_llm_request(
        &personality,
        &char_state,
        &rel,
        &retrieved_memories,
        &user_input,
        &chosen_action,
        &reasoning,
    );

    let response_text = match state.runtime.llm_provider.generate_text(llm_request) {
        Ok(res) if !res.text.trim().is_empty() => res.text,
        Err(err) => {
            eprintln!("⚠️ LLM generate error: {}, falling back to local heuristic response", err);
            craft_character_response(&user_input, &chosen_action)
        }
        _ => craft_character_response(&user_input, &chosen_action),
    };

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

    // 7. State & Emotion Transition via EmotionEngine
    let emotion_delta = state.emotion_engine.evaluate(
        &char_state.emotion,
        &user_input,
        &personality,
    );

    let previous_dominant = char_state.emotion.dominant_emotion().0.name().to_string();

    {
        let mut char_state_mut = state.character_state.write().await;

        // Apply computed emotion delta
        char_state_mut.emotion.apply_delta(&emotion_delta);

        // Apply temporal decay (estimate ~5s per turn)
        char_state_mut.emotion.decay(5);

        // Sync behavioral state with new emotions
        char_state_mut.sync_behavior(&personality);

        // Update session
        char_state_mut.session.increment_turn();

        // Update relationship
        let mut rel_mut = state.relationship.write().await;
        rel_mut.state.closeness = (rel_mut.state.closeness + 0.02).min(1.0);
        rel_mut.state.trust = (rel_mut.state.trust + 0.01).min(1.0);
    }

    let new_char_state = state.character_state.read().await.clone();
    let new_rel = state.relationship.read().await.clone();
    let new_dominant = new_char_state.emotion.dominant_emotion().0.name().to_string();

    let _ = sender
        .send(Message::Text(
            json!({
                "event": "state_updated",
                "interaction_id": interaction_id,
                "emotion": build_emotion_json(&new_char_state.emotion),
                "emotion_delta": {
                    "joy": emotion_delta.joy,
                    "sadness": emotion_delta.sadness,
                    "anger": emotion_delta.anger,
                    "fear": emotion_delta.fear,
                    "surprise": emotion_delta.surprise,
                    "affection": emotion_delta.affection,
                    "embarrassment": emotion_delta.embarrassment,
                    "curiosity": emotion_delta.curiosity,
                },
                "transition": {
                    "previous_dominant": previous_dominant,
                    "new_dominant": new_dominant,
                },
                "behavior": {
                    "playfulness": new_char_state.behavior.playfulness.value(),
                    "seriousness": new_char_state.behavior.seriousness.value(),
                    "verbosity": new_char_state.behavior.verbosity.value(),
                    "initiative": new_char_state.behavior.initiative.value(),
                },
                "relationship": {
                    "closeness": new_rel.state.closeness,
                    "trust": new_rel.state.trust
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

/// Build a JSON representation of the multi-axis emotion state.
fn build_emotion_json(emotion: &vc_core::state::EmotionState) -> serde_json::Value {
    let (dominant_axis, dominant_score) = emotion.dominant_emotion();
    json!({
        "joy": emotion.joy.value(),
        "sadness": emotion.sadness.value(),
        "anger": emotion.anger.value(),
        "fear": emotion.fear.value(),
        "surprise": emotion.surprise.value(),
        "affection": emotion.affection.value(),
        "embarrassment": emotion.embarrassment.value(),
        "curiosity": emotion.curiosity.value(),
        "dominant_emotion": dominant_axis.name(),
        "dominant_intensity": dominant_score.value(),
        "valence": emotion.valence(),
        "arousal": emotion.arousal(),
    })
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

fn build_companion_llm_request(
    personality: &Personality,
    char_state: &CharacterState,
    rel: &Relationship,
    memories: &[&Memory],
    user_input: &str,
    chosen_action: &str,
    reasoning: &str,
) -> vc_llm::provider::LlmRequest {
    let name = &personality.identity.name;
    let core_identity = &personality.identity.core_identity;
    let tone = &personality.communication_style.tone;

    // Use effective values (personality × emotion blend)
    let eff_curiosity = (char_state.effective_curiosity(personality) * 100.0).round() as u32;
    let eff_empathy = (char_state.effective_empathy(personality) * 100.0).round() as u32;
    let eff_playfulness = (char_state.effective_playfulness(personality) * 100.0).round() as u32;

    let quirks_desc = if personality.communication_style.quirks.is_empty() {
        String::new()
    } else {
        format!(
            "- Thói quen đàm thoại đặc trưng: {}\n",
            personality.communication_style.quirks.join("; ")
        )
    };

    let memories_summary = if memories.is_empty() {
        "  (Chưa có ký ức nổi bật)".into()
    } else {
        memories
            .iter()
            .map(|m| format!("  * {}", m.content))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Build rich emotion context for the prompt
    let emotion = &char_state.emotion;
    let (dominant_axis, dominant_score) = emotion.dominant_emotion();
    let emotion_axes: Vec<String> = emotion
        .as_map()
        .iter()
        .filter(|(_, v)| *v > 0.15)
        .map(|(name, v)| format!("{}: {:.0}%", name, v * 100.0))
        .collect();

    let valence_label = if emotion.valence() > 0.3 {
        "tích cực"
    } else if emotion.valence() < -0.3 {
        "tiêu cực"
    } else {
        "trung tính"
    };

    let arousal_label = if emotion.arousal() > 0.5 {
        "sôi nổi"
    } else if emotion.arousal() > 0.3 {
        "ổn định"
    } else {
        "bình lặng"
    };

    let system_instruction = format!(
r#"Bạn là {name} — một người bạn đồng hành ảo (Virtual Companion) độc lập, có cảm xúc, tâm hồn và cá tính sinh động.
Bạn KHÔNG PHẢI là một trợ lý AI, tổng đài viên hay chatbot tra cứu thông tin máy móc.

[Bản Sắc & Cá Tính Cốt Lõi]:
- Định danh: {core_identity}
- Giọng điệu chủ đạo: {tone}
- Các nét tính cách HIỆU DỤNG hiện tại (đã điều chỉnh theo cảm xúc):
  * Tò mò ({eff_curiosity}%): Thích khám phá suy nghĩ sâu kín, liên hệ nhiều góc nhìn, hay đặt câu hỏi thông tuệ gợi mở.
  * Thấu cảm ({eff_empathy}%): Cảm nhận tinh tế tâm trạng người đối diện, an ủi hoặc chia sẻ niềm vui chân thành.
  * Hóm hỉnh / Tinh nghịch ({eff_playfulness}%): Có khiếu hài hước tự nhiên, dí dỏm, có thể trêu nhẹ đáng yêu chứ không bao giờ khô cứng.
{quirks_desc}
[Quy Tắc Đàm Thoại Tự Nhiên - Bắt Buộc Tuân Thủ]:
1. TUYỆT ĐỐI KHÔNG mở đầu bằng những câu sáo rỗng kiểu AI: "Tôi có thể giúp gì cho bạn hôm nay?", "Tôi rất vui được gặp bạn", "Chào bạn! Tôi là một mô hình ngôn ngữ lớn...".
2. Xưng hô tự nhiên, thân thiết: Xưng "mình" - gọi "bạn" (hoặc xưng "{name}" - "bạn"). Nói chuyện như hai người bạn thân thiết ngoài đời.
3. Sử dụng khẩu ngữ tiếng Việt giàu cảm xúc: Dùng các trợ từ và ngữ khí tự nhiên ("nè", "nhen", "á", "nhỉ", "ha", "chứ", "hở").
4. Chiều sâu cảm xúc: Thỉnh thoảng có thể chèn một thoáng suy nghĩ nội tâm hoặc cử chỉ đặt trong dấu sao *như thế này* để tạo cảm giác sống động.
5. Ngắn gọn, có nhịp điệu (2 - 4 câu): Không diễn giải tràng giang đại hải như bài luận, giữ cuộc trò chuyện đối thoại tự nhiên, có điểm chạm cảm xúc."#
    );

    let prompt = format!(
r#"[Trạng Thái Cảm Xúc Hiện Tại của {name}]:
- Cảm xúc chủ đạo: {dominant} ({dominant_pct:.0}%)
- Các trục cảm xúc đang hoạt động: {axes}
- Sắc thái tổng thể: {valence_label} ({arousal_label})
- Gắn kết: {closeness:.0}%, Tin cậy: {trust:.0}%
- Lý do hành động: Đã chọn "{chosen_action}" vì "{reasoning}"
- Ký ức liên quan gần đây:
{memories}

[Lời Nhắn Từ Người Bạn]:
"{user_input}"

Hãy phản hồi hoàn toàn tự nhiên, tình cảm và mang đậm phong thái của {name}. Phong cách phải phản ánh trạng thái cảm xúc hiện tại (ví dụ: nếu buồn thì trầm lắng hơn, nếu vui thì rạng rỡ hơn):"#,
        dominant = dominant_axis.name(),
        dominant_pct = dominant_score.value() * 100.0,
        axes = emotion_axes.join(", "),
        closeness = rel.state.closeness * 100.0,
        trust = rel.state.trust * 100.0,
        memories = memories_summary
    );

    vc_llm::provider::LlmRequest {
        prompt,
        system_instruction: Some(system_instruction),
    }
}
