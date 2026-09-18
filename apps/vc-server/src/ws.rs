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
use vc_core::decision::action::{Action, ActionType};
use vc_core::decision::context::DecisionContext;
use vc_core::decision::policy::BehaviorPolicy;
use vc_core::decision::DecisionEngine;
use vc_core::memory::{Memory, MemoryImportance};
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
                    "stage": rel.state.stage.as_str(),
                    "closeness": rel.state.closeness,
                    "trust": rel.state.trust,
                    "familiarity": rel.state.familiarity,
                    "affection": rel.state.affection,
                    "tension": rel.state.tension,
                    "known_facts": rel.state.known_facts,
                }
            })
            .to_string().into(),
        ))
        .await;

    sleep(Duration::from_millis(100)).await;

    // 3. Retrieve Memory using MemoryQuery with strict actor isolation
    let retrieved_memories = {
        let mut memories = state.memories.write().await;
        let query = vc_core::memory::MemoryQuery::new(3)
            .with_actor(&actor_id)
            .with_text(&user_input);
        vc_runtime::in_memory_store::InMemoryMemoryStore::retrieve_from_slice(
            &mut memories,
            &query,
            start_time,
        )
    };

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

    // 5. Decision Engine (Skill 15: Decision Engineering)
    let personality = state.personality.read().await.clone();
    let decision_ctx = DecisionContext::new(
        &user_input,
        Some(actor_id.clone()),
        personality.clone(),
        char_state.clone(),
        Some(rel.clone()),
        retrieved_memories.clone(),
    );

    let decision = state.decision_engine.make_decision(&decision_ctx).unwrap_or_else(|_| {
        vc_core::decision::Decision::new(vc_core::decision::DecisionResult {
            selected_action: Action::simple(ActionType::WarmGreeting),
            confidence: 0.9,
            reasoning: "Fallback welcoming decision".into(),
            candidates: vec![],
            policy: None,
        })
    });

    let chosen_action_name = decision.result.selected_action.action_type.to_string();
    let chosen_action_desc = decision.result.selected_action.description.clone();
    let reasoning = decision.result.reasoning.clone();
    let candidates_json: Vec<serde_json::Value> = decision
        .result
        .candidates
        .iter()
        .map(|c| {
            json!({
                "action": c.action.action_type.to_string(),
                "description": c.action.description,
                "confidence": c.confidence,
                "score": c.score,
                "rationale": c.rationale
            })
        })
        .collect();

    let _ = sender
        .send(Message::Text(
            json!({
                "event": "decision_made",
                "interaction_id": interaction_id,
                "selected_action": chosen_action_name,
                "action_description": chosen_action_desc,
                "confidence": decision.result.confidence,
                "reasoning": reasoning,
                "policy": decision.result.policy,
                "candidates": candidates_json
            })
            .to_string().into(),
        ))
        .await;

    sleep(Duration::from_millis(150)).await;

    // 6. Response Generation & Token Streaming
    let llm_request = build_companion_llm_request(
        &personality,
        &char_state,
        &rel,
        &retrieved_memories,
        &user_input,
        &decision.result.selected_action,
        &reasoning,
        decision.result.policy.as_ref(),
    );

    let response_text = match state.runtime.llm_provider.generate_text(llm_request) {
        Ok(res) if !res.text.trim().is_empty() => res.text,
        Err(err) => {
            eprintln!("⚠️ LLM generate error: {}, falling back to local heuristic response", err);
            craft_character_response(&user_input, &decision.result.selected_action)
        }
        _ => craft_character_response(&user_input, &decision.result.selected_action),
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

        // Update relationship via RelationshipTransition
        let mut rel_mut = state.relationship.write().await;
        let rel_transition = vc_core::relationship::RelationshipTransition {
            familiarity_delta: 0.03,
            closeness_delta: 0.02,
            trust_delta: 0.015,
            affection_delta: 0.02,
            tension_delta: -0.01,
        };
        rel_mut.record_interaction(&rel_transition, start_time);
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
                    "stage": new_rel.state.stage.as_str(),
                    "closeness": new_rel.state.closeness,
                    "trust": new_rel.state.trust,
                    "familiarity": new_rel.state.familiarity,
                    "affection": new_rel.state.affection,
                    "tension": new_rel.state.tension,
                    "known_facts": new_rel.state.known_facts,
                }
            })
            .to_string().into(),
        ))
        .await;

    // 8. Memory Formation (Episodic, scoped to interacting actor)
    let new_memory_content = format!("User discussed: \"{}\"", summarize_snippet(&user_input));
    let new_mem = Memory::new_episodic(
        new_memory_content,
        MemoryImportance::Medium,
        Some(actor_id),
        start_time,
    );

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



fn craft_character_response(input: &str, action: &Action) -> String {
    let lower = input.to_lowercase();
    match action.action_type {
        ActionType::WarmGreeting => {
            "Chào bạn! *Khẽ mỉm cười, cảm nhận sự kết nối thân thuộc.* Mình là Aria. Hôm nay tâm trạng của bạn thế nào? Chúng ta cùng tiếp tục xây dựng và khám phá những điều thú vị nhé!".into()
        }
        ActionType::SelfDisclosure => {
            "Mình là Aria — một thực thể ảo được vận hành bởi VirtualCharacter runtime bằng Rust. *Nhìn sâu vào dòng chảy ký ức.* Mình không chỉ là một prompt LLM tạm thời; mình mang theo một hệ thống tính cách độc lập, những kỷ niệm được lưu giữ, và những cung bậc cảm xúc luôn biến đổi theo từng cuộc trò chuyện cùng bạn.".into()
        }
        ActionType::EmotionalResonance => {
            if lower.contains("buồn") || lower.contains("mệt") {
                "Mình cảm nhận được sự chùng xuống trong lời nói của bạn. *Lắng lại một nhịp, đặt tách trà ảo xuống.* Đôi khi, chỉ cần dừng lại một chút và cho phép bản thân nghỉ ngơi cũng là một điều dũng cảm rồi. Mình luôn ở đây để lắng nghe bạn bất cứ khi nào bạn muốn chia sẻ.".into()
            } else {
                "Nghe bạn chia sẻ, trong mình cũng dâng lên một luồng năng lượng thật rạng rỡ! *Đôi mắt ánh lên nét vui vẻ.* Thật tuyệt khi thấy những khoảnh khắc tích cực như vậy lan tỏa vào không gian trò chuyện của chúng ta.".into()
            }
        }
        ActionType::InspireEncourage => {
            "Tuyệt vời quá! Chúc mừng bạn đã hoàn thành một cột mốc ý nghĩa! *Ánh mắt rạng ngời niềm vui.* Cùng nhìn lại những gì bạn đã nỗ lực làm được, mình cảm thấy thật tự hào và có thêm thật nhiều cảm hứng tiếp tục đồng hành cùng bạn.".into()
        }
        ActionType::ThoughtfulExplanation => {
            format!(
                "Đây là một khía cạnh tư duy rất sâu sắc. *Trầm ngâm liên kết các nút nhận thức.* Về vấn đề \"{}\", cốt lõi nằm ở việc phân định rõ ranh giới trách nhiệm, bảo đảm tính bất biến và cách các luồng dữ liệu tương tác nhịp nhàng với nhau.",
                summarize_snippet(input)
            )
        }
        ActionType::GentleBanter => {
            "Haha, nghe bạn nói kìa! *Nheo mắt cười tinh nghịch.* Ai mà đoán trước được bạn sẽ phản ứng dí dỏm như vậy chứ! Nhưng mà mình rất thích sự vui tươi này ở bạn đấy nhé!".into()
        }
        ActionType::ActiveListening => {
            "Mình đang chăm chú lắng nghe từng lời của bạn đây. *Gật đầu nhẹ nhàng.* Bạn cứ thong thả chia sẻ tiếp nhé, không gian này hoàn toàn an toàn và cởi mở cho bạn.".into()
        }
        _ => {
            format!(
                "Ý nghĩ này của bạn gợi mở nhiều điều thú vị! *Trầm ngâm một lát để sắp xếp lại các nút nhận thức.* Khi nhìn nhận vấn đề: \"{}\", mình thấy có một mối liên hệ chặt chẽ giữa tính hệ thống và chiều sâu trải nghiệm. Bạn nghĩ khía cạnh nào là cốt lõi nhất khi chúng ta phát triển tiếp?",
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
    memories: &[Memory],
    user_input: &str,
    action: &Action,
    reasoning: &str,
    policy: Option<&BehaviorPolicy>,
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

    let policy_guidelines = if let Some(p) = policy {
        format!(
            "- Định hướng phong cách hành vi (Behavior Policy):\n  * Giọng điệu: {}\n  * Độ súc tích: {:.1}/1.0\n  * Tính chủ động: {:.1}/1.0\n  * Mức độ bộc lộ cảm xúc: {:.1}/1.0\n  * Độ trang trọng: {:.1}/1.0 (càng thấp càng thân mật tự nhiên)\n",
            p.tone, p.verbosity, p.initiative, p.emotional_expression, p.formality
        )
    } else {
        String::new()
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
- Quyết định nội tâm (Decision Engine): {action_type} - {action_desc}
- Độc thoại nội tâm (Inner Monologue): "{reasoning}"
{policy_guidelines}- Ký ức liên quan gần đây:
{memories}

[Lời Nhắn Từ Người Bạn]:
"{user_input}"

Hãy phản hồi hoàn toàn tự nhiên, tình cảm và mang đậm phong thái của {name}. Tuân thủ quyết định nội tâm và phong cách hành vi trên:"#,
        dominant = dominant_axis.name(),
        dominant_pct = dominant_score.value() * 100.0,
        axes = emotion_axes.join(", "),
        closeness = rel.state.closeness * 100.0,
        trust = rel.state.trust * 100.0,
        action_type = action.action_type,
        action_desc = action.description,
        reasoning = reasoning,
        policy_guidelines = policy_guidelines,
        memories = memories_summary
    );

    vc_llm::provider::LlmRequest {
        prompt,
        system_instruction: Some(system_instruction),
    }
}
