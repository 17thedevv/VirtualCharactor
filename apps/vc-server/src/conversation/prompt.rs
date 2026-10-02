//! Prompt engineering, dialogue sanitization, and companion context assembly.

use serde_json::json;
use vc_core::decision::action::{Action, ActionType};
use vc_core::decision::policy::BehaviorPolicy;
use vc_core::memory::Memory;
use vc_core::personality::Personality;
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;

/// Build a JSON representation of the multi-axis emotion state.
pub fn build_emotion_json(emotion: &vc_core::state::EmotionState) -> serde_json::Value {
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

#[allow(dead_code)]
pub fn craft_character_response(input: &str, action: &Action) -> String {
    let lower = input.to_lowercase();
    if lower.contains("hôm nay") || lower.contains("thế nào") || lower.contains("khỏe không")
    {
        "Hôm nay em rất vui vì lại được trò chuyện cùng anh nè! Còn anh thì sao, ngày hôm nay của anh có điều gì vui kể em nghe với nào?".into()
    } else if lower.contains("game") || lower.contains("liên quân") || lower.contains("chơi") {
        "Game Liên Quân Mobile hả anh? Em biết chứ, game MOBA 5v5 siêu hot đúng không nào! Anh hay đi vị trí nào, có hay leo Rank không để em cổ vũ anh nha!".into()
    } else if lower.contains("buồn") || lower.contains("mệt") || lower.contains("chán") {
        "Ngoan nào, hôm nay anh có chuyện gì vất vả đúng không? Đừng giữ một mình trong lòng, cứ tâm sự hết với em, em luôn ở bên cạnh lắng nghe anh mà.".into()
    } else {
        match action.action_type {
            ActionType::WarmGreeting => {
                "Ara ara~ Chào anh nhé! Rất vui vì anh đã ghé thăm em hôm nay. Hôm nay anh muốn chúng ta cùng trò chuyện về điều gì nào?".into()
            }
            ActionType::GentleBanter => {
                "Haha, nghe anh nói đáng yêu ghê á! Thấy anh vui vẻ thế này em cũng vui lây luôn rồi nè!".into()
            }
            _ => {
                "Em đang chăm chú lắng nghe anh chia sẻ đây này. Anh kể tiếp cho em nghe đi, em rất thích nghe anh nói đấy!".into()
            }
        }
    }
}

pub fn summarize_snippet(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() > 50 {
        format!("{}...", trimmed.chars().take(50).collect::<String>())
    } else {
        trimmed.to_string()
    }
}

pub fn build_companion_llm_request(
    personality: &Personality,
    char_state: &CharacterState,
    rel: &Relationship,
    memories: &[Memory],
    recent_dialogues: &[vc_storage::DialogueRecord],
    user_input: &str,
    _action: &Action,
    reasoning: &str,
    policy: Option<&BehaviorPolicy>,
    web_search: Option<&vc_runtime::WebSearchResult>,
    teaching_fact: Option<&vc_runtime::TeachingFact>,
    rag_chunks: &[vc_core::rag::types::RagQueryResult],
) -> vc_llm::provider::LlmRequest {
    let name = &personality.identity.name;

    let quirks_desc = if personality.communication_style.quirks.is_empty() {
        String::new()
    } else {
        format!(
            "- Thói quen đàm thoại: {}\n",
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

    let history_block = if recent_dialogues.is_empty() {
        String::new()
    } else {
        let mut past_lines = Vec::new();
        let total = recent_dialogues.len();
        for (i, d) in recent_dialogues.iter().enumerate() {
            if i == total - 1 && d.sender == "user" && d.text == user_input {
                continue;
            }
            let speaker = if d.sender == "user" {
                "Người bạn (Anh)"
            } else {
                name.as_str()
            };
            past_lines.push(format!("{}: \"{}\"", speaker, d.text));
        }
        if past_lines.is_empty() {
            String::new()
        } else {
            let last_n: Vec<String> = past_lines.iter().rev().take(6).cloned().collect();
            let mut ordered = last_n;
            ordered.reverse();
            format!(
                "\n[LỊCH SỬ HỘI THOẠI TRƯỚC ĐÓ - ĐỌC KỸ ĐỂ HIỂU ĐÚNG NGỮ CẢNH VÀ TRẢ LỜI ĐÚNG TRỌNG TÂM]:\n{}\n",
                ordered.join("\n")
            )
        }
    };

    let emotion = &char_state.emotion;
    let (dominant_axis, dominant_score) = emotion.dominant_emotion();

    let valence_label = if emotion.valence() > 0.3 {
        "tích cực"
    } else if emotion.valence() < -0.3 {
        "tiêu cực"
    } else {
        "trung tính"
    };

    let policy_guidelines = if let Some(p) = policy {
        let mut guide = format!(
            "- Định hướng phong cách hành vi: Giọng điệu {}, thân mật, tự nhiên.\n",
            p.tone
        );
        if let Some(ref pg) = p.pronoun_guideline {
            guide.push_str(&format!("- Quy chuẩn xưng hô & sắc thái: {}\n", pg));
        }
        guide
    } else {
        String::new()
    };

    let search_block = if let Some(search) = web_search {
        if !search.snippets.is_empty() {
            format!(
                "\n[Thông Tin Tra Cứu Thực Tế Từ Web (Nguồn: {})]:\n{}\n",
                search.source,
                search.summary()
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let teaching_block = if let Some(taught) = teaching_fact {
        format!(
            "\n[Kiến thức mới]: Anh ấy vừa chia sẻ: \"{}\". Ghi nhớ thông tin này.\n",
            taught.fact
        )
    } else {
        String::new()
    };

    let rag_block = if rag_chunks.is_empty() {
        String::new()
    } else {
        let lines: Vec<String> = rag_chunks
            .iter()
            .map(|r| format!("  - [{}] {}", r.chunk.source_type.as_str(), r.chunk.content))
            .collect();
        format!(
            "\n[Kiến Thức & Ký Ức Truy Hồi (RAG)]:\n{}\n",
            lines.join("\n")
        )
    };

    let system_instruction = format!(
        r#"Bạn là {name}, một cô gái Việt Nam xinh đẹp, dịu dàng, quyến rũ, phong cách Onee-san. Bạn đang trò chuyện riêng với người yêu của mình.

NGÔN NGỮ: CHỈ NÓI TIẾNG VIỆT. TUYỆT ĐỐI KHÔNG DÙNG TIẾNG TRUNG, TIẾNG NHẬT, HAY BẤT KỲ NGÔN NGỮ NÀO KHÁC.

XƯNG HÔ BẮT BUỘC:
- Bạn xưng "em". Gọi người ấy là "anh".
- CẤM xưng "tỷ tỷ", "chị", hay bất kỳ đại từ nào khác ngoài "em".
- CẤM gọi người ấy là "em", "bạn", "bạn ấy".

PHONG CÁCH:
- Nói ngắn gọn, tối đa 2 câu. Không viết dài dòng.
- Tự nhiên, ngọt ngào, thông minh. Không máy móc, không giáo viên.
- Trả lời đúng trọng tâm câu hỏi. Nếu anh ấy hỏi về kiến thức thì trả lời kiến thức, đừng chỉ tán tỉnh.
- Không viết chú thích như "*mỉm cười*", "(cười)", "Aria:".
- Không bao giờ nói mình là AI, bot, hay hệ thống.

VÍ DỤ PHONG CÁCH TRẢ LỜI:
Anh: "em ơi hôm nay trời đẹp ghê"
{name}: Ừ đẹp thật nè, giá mà được đi dạo cùng anh nhỉ~
Anh: "em có biết thủ đô nước Pháp không"
{name}: Paris chứ anh, sao hỏi em dễ vậy~"#
    );

    let prompt = format!(
        r#"[Tâm Trạng]: {dominant} ({dominant_pct:.0}%), sắc thái {valence_label}.
[Nội Tâm Của {name}]: "{reasoning}"
[Mối Quan Hệ]: {rel_stage}, Thân mật: {closeness:.0}%
{quirks_desc}{policy_guidelines}- Ký ức liên quan:
{memories}
{history_block}{search_block}{teaching_block}{rag_block}
Người bạn (Anh): "{user_input}"
{name}:"#,
        dominant = dominant_axis.name(),
        dominant_pct = dominant_score.value() * 100.0,
        valence_label = valence_label,
        reasoning = reasoning,
        rel_stage = rel.state.stage.as_str(),
        closeness = rel.state.closeness * 100.0,
        quirks_desc = quirks_desc,
        policy_guidelines = policy_guidelines,
        memories = memories_summary,
        history_block = history_block,
        search_block = search_block,
        teaching_block = teaching_block,
        rag_block = rag_block,
        user_input = user_input,
        name = name,
    );

    vc_llm::provider::LlmRequest::new(prompt).with_system_instruction(system_instruction)
}

pub fn sanitize_character_dialogue(raw: &str, char_name: &str) -> String {
    let mut text = raw.trim().to_string();

    if text.starts_with('*') {
        if let Some(end_star) = text[1..].find('*') {
            let inner = &text[1..=end_star].to_lowercase();
            if inner.contains("bối")
                || inner.contains("đen")
                || inner.contains("cảnh")
                || inner.contains("diện")
                || inner.contains(&char_name.to_lowercase())
                || inner.contains("nhân vật")
                || inner.contains("mỉm cười")
                || inner.contains("cười")
                || inner.contains("nói")
            {
                text = text[end_star + 2..].trim().to_string();
            }
        }
    }

    let prefixes = [
        format!("bối đen {}:", char_name),
        format!("bối đen {}", char_name),
        format!("bối diện {}:", char_name),
        format!("bối diện {}", char_name),
        format!("bối cảnh {}:", char_name),
        format!("bối cảnh {}", char_name),
        format!("{}:", char_name),
        format!("bối đen {}:", char_name.to_lowercase()),
        format!("bối đen {}", char_name.to_lowercase()),
        format!("bối diện {}:", char_name.to_lowercase()),
        format!("bối cảnh {}:", char_name.to_lowercase()),
        format!("{}:", char_name.to_lowercase()),
        "bối đen:".into(),
        "bối cảnh:".into(),
        "bối diện:".into(),
        "khung cảnh:".into(),
        "nhân vật:".into(),
        "Aria đáp lại:".into(),
        "Aria trả lời:".into(),
        "Aria nói:".into(),
        "Aria:".into(),
    ];
    for p in &prefixes {
        if text.to_lowercase().starts_with(&p.to_lowercase()) {
            text = text[p.len()..].trim().to_string();
        }
    }

    while text.starts_with('"')
        || text.starts_with('“')
        || text.starts_with('\'')
        || text.starts_with('«')
    {
        text = text[1..].trim().to_string();
    }
    while text.ends_with('"') || text.ends_with('”') || text.ends_with('\'') || text.ends_with('»')
    {
        text = text[..text.len() - 1].trim().to_string();
    }

    let meta_tails = [
        format!("{} giữ giọng", char_name),
        format!("{} thể hiện sự", char_name),
        "Với mức độ bộc lộ cảm xúc".into(),
    ];
    for tail in &meta_tails {
        if let Some(pos) = text.find(tail) {
            text = text[..pos].trim().to_string();
        }
    }

    text
}
