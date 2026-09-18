use vc_core::decision::action::{Action, ActionType};
use vc_core::decision::candidate::DecisionCandidate;
use vc_core::decision::context::DecisionContext;
use vc_core::decision::policy::BehaviorPolicy;
use vc_core::decision::{Decision, DecisionEngine, DecisionResult};
use vc_core::error::Result;
use vc_core::relationship::stage::RelationshipStage;

/// A multi-step deterministic Decision Engine for Phase 1.
///
/// Under Skill 15 (Decision Engineering):
/// - Explicitly separates deciding *what to do* from *how the LLM generates words*.
/// - Evaluates candidates against Personality traits, Relationship stage, and Emotional state.
/// - Produces transparent inner monologue reasoning and confidence scores.
#[derive(Debug, Default, Clone)]
pub struct RuleDecisionEngine;

impl RuleDecisionEngine {
    pub fn new() -> Self {
        Self
    }
}

impl DecisionEngine for RuleDecisionEngine {
    fn make_decision(&self, ctx: &DecisionContext) -> Result<Decision> {
        let input_lower = ctx.user_input.to_lowercase();
        let rel_stage = ctx.relationship.as_ref().map(|r| r.state.stage).unwrap_or(RelationshipStage::Stranger);

        let empathy = ctx.personality.traits.empathy.value();
        let curiosity = ctx.personality.traits.curiosity.value();
        let playfulness = ctx.personality.traits.playfulness.value();

        // 1. Situation Analysis & Candidate Proposal
        let mut candidates = Vec::new();

        let is_greeting = input_lower.contains("chào")
            || input_lower.contains("hello")
            || input_lower.contains("hi")
            || input_lower.contains("hey");

        let is_identity = input_lower.contains("bạn là ai")
            || input_lower.contains("who are you")
            || input_lower.contains("tính cách")
            || input_lower.contains("cá tính")
            || input_lower.contains("tâm hồn")
            || input_lower.contains("identity");

        let is_emotional = input_lower.contains("buồn")
            || input_lower.contains("mệt")
            || input_lower.contains("sad")
            || input_lower.contains("tired")
            || input_lower.contains("áp lực")
            || input_lower.contains("lo lắng")
            || input_lower.contains("vui")
            || input_lower.contains("hạnh phúc");

        let is_accomplishment = input_lower.contains("xong")
            || input_lower.contains("hoàn thành")
            || input_lower.contains("finished")
            || input_lower.contains("built")
            || input_lower.contains("thành công");

        let is_technical = input_lower.contains("giải thích")
            || input_lower.contains("tại sao")
            || input_lower.contains("kiến trúc")
            || input_lower.contains("rust")
            || input_lower.contains("compiler")
            || input_lower.contains("memory")
            || input_lower.contains("how");

        // 2. Candidate Generation based on situation
        if is_greeting {
            let score_greeting = 0.80 + (empathy * 0.15);
            candidates.push(DecisionCandidate::new(
                Action::new(ActionType::WarmGreeting, "welcoming", "Chào đón ấm áp và mở rộng kết nối"),
                0.95,
                score_greeting,
                "Phát hiện lời chào thân thiện; ưu tiên tiếp đón nồng hậu.",
            ));
            candidates.push(DecisionCandidate::new(
                Action::new(ActionType::CuriousInquiry, "asking_focus", "Hỏi thăm sự chú ý và công việc hiện tại"),
                0.65,
                0.60 + (curiosity * 0.15),
                "Chủ động gợi mở hỏi han người dùng đang làm gì.",
            ));
        }

        if is_identity {
            candidates.push(DecisionCandidate::new(
                Action::new(ActionType::SelfDisclosure, "core_identity", "Bộc lộ bản sắc căn tính và giá trị cốt lõi"),
                0.96,
                0.92,
                "Người dùng hỏi về bản thân; bộc lộ chân thành và nhất quán.",
            ));
            candidates.push(DecisionCandidate::new(
                Action::new(ActionType::ThoughtfulExplanation, "cognitive_architecture", "Chia sẻ góc nhìn triết học và nhận thức số"),
                0.75,
                0.70 + (curiosity * 0.15),
                "Diễn giải sâu sắc về nhận thức và trải nghiệm đồng hành.",
            ));
        }

        if is_accomplishment {
            let score_encourage = 0.85 + (empathy * 0.10);
            candidates.push(DecisionCandidate::new(
                Action::new(ActionType::InspireEncourage, "celebrate_achievement", "Chúc mừng và cùng chia sẻ niềm vui thành tựu"),
                0.94,
                score_encourage,
                "Người dùng vừa hoàn thành một cột mốc quan trọng; cộng hưởng tự hào.",
            ));
            candidates.push(DecisionCandidate::new(
                Action::new(ActionType::CuriousInquiry, "explore_next_step", "Tò mò khám phá những bước tiếp theo"),
                0.78,
                0.72 + (curiosity * 0.15),
                "Khám phá xem kế hoạch tiếp theo của dự án là gì.",
            ));
        }

        if is_emotional {
            let score_resonance = 0.85 + (empathy * 0.12);
            candidates.push(DecisionCandidate::new(
                Action::new(ActionType::EmotionalResonance, "empathic_comfort", "Thấu cảm và chia sẻ cảm xúc chân thành"),
                0.92,
                score_resonance,
                "Chủ đề mang tính cảm xúc cao; ưu tiên hiện diện ấm áp và thấu hiểu.",
            ));
            candidates.push(DecisionCandidate::new(
                Action::new(ActionType::ActiveListening, "listening", "Lắng nghe chăm chú và tạo không gian an toàn"),
                0.80,
                0.75,
                "Tạo điểm tựa lắng nghe tĩnh lặng, không phán xét.",
            ));
        }

        if is_technical {
            candidates.push(DecisionCandidate::new(
                Action::new(ActionType::ThoughtfulExplanation, "system_concept", "Diễn giải thấu đáo và phân tích logic"),
                0.90,
                0.80 + (curiosity * 0.12),
                "Chủ đề kỹ thuật/kiến trúc; diễn giải mạch lạc và thấu đáo.",
            ));
        }

        // Playful Banter Candidate (modulated by Relationship Stage and Playfulness)
        let banter_gated_score = match rel_stage {
            RelationshipStage::Stranger => 0.20, // Suppressed for strangers
            RelationshipStage::Acquaintance => 0.45,
            RelationshipStage::CasualCompanion => 0.65 + (playfulness * 0.15),
            RelationshipStage::CloseFriend | RelationshipStage::Confidant => 0.75 + (playfulness * 0.20),
        };
        candidates.push(DecisionCandidate::new(
            Action::new(ActionType::GentleBanter, "witty_banter", "Trêu đùa nhẹ nhàng, duyên dáng và dí dỏm"),
            0.60,
            banter_gated_score,
            format!("Hành động dí dỏm được điều tiết theo giai đoạn quan hệ ({:?}).", rel_stage),
        ));

        // Default fallback if no specific triggers matched
        if candidates.is_empty() {
            candidates.push(DecisionCandidate::new(
                Action::new(ActionType::CuriousInquiry, "conversational_lead", "Gợi mở cuộc trò chuyện với sự tò mò ấm áp"),
                0.80,
                0.75 + (curiosity * 0.15),
                "Tiếp tục mạch hội thoại mở và duy trì kết nối.",
            ));
        }

        // 3. Candidate Sorting & Optimal Selection
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        let selected = candidates.first().cloned().expect("Candidates must not be empty");

        // 4. Synthesize Inner Monologue Reasoning
        let reasoning = match selected.action.action_type {
            ActionType::WarmGreeting => format!(
                "Phát hiện lời chào từ người dùng. Với nét tính cách Thấu cảm ({:.0}%) và Tò mò ({:.0}%), ưu tiên mở ra không gian chào đón nồng hậu.",
                empathy * 100.0,
                curiosity * 100.0
            ),
            ActionType::InspireEncourage => format!(
                "Người bạn vừa đạt được cột mốc ý nghĩa. Ở giai đoạn quan hệ {:?}, cùng chúc mừng và chia sẻ niềm vui tự hào là điều tuyệt vời nhất.",
                rel_stage
            ),
            ActionType::EmotionalResonance => format!(
                "Nhận thấy sắc thái cảm xúc từ người dùng. Nét thấu cảm ({:.0}%) ưu tiên lắng nghe, đồng điệu và mang lại cảm giác an toàn tâm lý.",
                empathy * 100.0
            ),
            ActionType::SelfDisclosure => format!(
                "Người dùng bày tỏ sự quan tâm tới căn tính của mình. Chia sẻ chân thật về nhận thức số và các giá trị sống cốt lõi."
            ),
            ActionType::ThoughtfulExplanation => format!(
                "Chủ đề đòi hỏi chiều sâu suy nghĩ. Sử dụng tư duy phân tích và sự kiên nhẫn ({:.0}%) để làm sáng tỏ vấn đề một cách mạch lạc.",
                ctx.personality.traits.patience.value() * 100.0
            ),
            ActionType::GentleBanter => format!(
                "Không khí thoải mái và mức độ gắn kết {:?} cho phép trêu đùa dí dỏm ({:.0}%) để tăng thêm sinh khí.",
                rel_stage,
                playfulness * 100.0
            ),
            _ => format!(
                "Lựa chọn hành động {:?} với độ tin cậy {:.0}% dựa trên phân tích bối cảnh và mục tiêu đồng hành.",
                selected.action.action_type,
                selected.confidence * 100.0
            ),
        };

        // 5. Select Behavior Policy
        let policy = match selected.action.action_type {
            ActionType::EmotionalResonance => Some(BehaviorPolicy::warm_empathic()),
            ActionType::GentleBanter => Some(BehaviorPolicy::playful_witty()),
            ActionType::ThoughtfulExplanation => Some(BehaviorPolicy::contemplative_inquiry()),
            _ => Some(BehaviorPolicy::default()),
        };

        Ok(Decision::new(DecisionResult {
            selected_action: selected.action,
            confidence: selected.confidence,
            reasoning,
            candidates,
            policy,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_core::personality::Personality;
    use vc_core::relationship::Relationship;
    use vc_core::state::CharacterState;

    #[test]
    fn test_greeting_selects_warm_greeting() {
        let engine = RuleDecisionEngine::new();
        let personality = Personality::baseline_aria();
        let state = CharacterState::default_aria();
        let ctx = DecisionContext::new("Chào Aria buổi sáng!", None, personality, state, None, vec![]);

        let decision = engine.make_decision(&ctx).expect("Decision should succeed");
        assert_eq!(decision.result.selected_action.action_type, ActionType::WarmGreeting);
        assert!(decision.result.confidence > 0.85);
        assert!(!decision.result.candidates.is_empty());
    }

    #[test]
    fn test_relationship_gating_banter() {
        let engine = RuleDecisionEngine::new();
        let personality = Personality::baseline_aria();
        let state = CharacterState::default_aria();

        // For Stranger: Banter score is heavily suppressed
        let stranger_rel = Relationship::new_stranger(vc_core::character::CharacterId(personality.id.0), "stranger-1");
        let ctx_stranger = DecisionContext::new("Nói gì đi bạn", None, personality.clone(), state.clone(), Some(stranger_rel), vec![]);
        let decision_stranger = engine.make_decision(&ctx_stranger).unwrap();
        let banter_stranger = decision_stranger.result.candidates.iter().find(|c| c.action.action_type == ActionType::GentleBanter).unwrap();

        // For Confidant: Banter score is high
        let mut confidant_rel = Relationship::new_stranger(vc_core::character::CharacterId(personality.id.0), "friend-1");
        confidant_rel.metrics.closeness = vc_core::relationship::RelationshipScore::clamped(0.85);
        confidant_rel.metrics.trust = vc_core::relationship::RelationshipScore::clamped(0.90);
        confidant_rel.metrics.familiarity = vc_core::relationship::RelationshipScore::clamped(0.80);
        confidant_rel.sync_state();

        let ctx_confidant = DecisionContext::new("Nói gì đi bạn", None, personality, state, Some(confidant_rel), vec![]);
        let decision_confidant = engine.make_decision(&ctx_confidant).unwrap();
        let banter_confidant = decision_confidant.result.candidates.iter().find(|c| c.action.action_type == ActionType::GentleBanter).unwrap();

        assert!(banter_confidant.score > banter_stranger.score, "Banter score must be higher for confidants than strangers");
    }
}
