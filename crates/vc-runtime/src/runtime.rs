use std::sync::{Arc, RwLock};

use vc_core::character::CharacterId;
use vc_core::decision::DecisionEngine;
use vc_core::memory::{Memory, MemoryImportance};
use vc_core::personality::Personality;
use vc_core::relationship::transition::RelationshipTransition;
use vc_core::relationship::Relationship;
use vc_core::state::{CharacterState, EmotionEngine};
use vc_llm::provider::LlmProvider;

use crate::interaction::{InteractionId, InteractionOutcome};
use crate::session::{SessionId, SessionManager};

/// Central Orchestrator executing the VirtualCharacter interaction lifecycle.
///
/// Under Skill 22 (Runtime Engineering):
/// - Orchestration only: Runtime connects Inputs -> Context -> Decision -> LLM -> Outputs -> State/Memory feedback.
/// - No business logic: Delegates perception, decisions, and emotions to `vc-core`.
/// - Thread-safe session management.
pub struct RuntimeEngine {
    pub llm_provider: Arc<dyn LlmProvider>,
    pub decision_engine: Arc<dyn DecisionEngine>,
    pub emotion_engine: Arc<dyn EmotionEngine>,
    pub session_manager: Arc<RwLock<SessionManager>>,
}

impl RuntimeEngine {
    /// Create a RuntimeEngine with standard rule engines and session manager.
    pub fn new(llm_provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            llm_provider,
            decision_engine: Arc::new(crate::rule_decision_engine::RuleDecisionEngine::new()),
            emotion_engine: Arc::new(crate::rule_emotion_engine::RuleBasedEmotionEngine::new()),
            session_manager: Arc::new(RwLock::new(SessionManager::new())),
        }
    }

    /// Create a fully customized RuntimeEngine with injected domain engines.
    pub fn with_engines(
        llm_provider: Arc<dyn LlmProvider>,
        decision_engine: Arc<dyn DecisionEngine>,
        emotion_engine: Arc<dyn EmotionEngine>,
    ) -> Self {
        Self {
            llm_provider,
            decision_engine,
            emotion_engine,
            session_manager: Arc::new(RwLock::new(SessionManager::new())),
        }
    }

    /// Periodic background tick (decay, background reconciliation).
    pub fn tick(&self, _character_id: CharacterId) -> vc_core::Result<()> {
        Ok(())
    }

    /// Orchestrate a complete interaction turn through the 9-stage lifecycle.
    pub fn process_interaction(
        &self,
        character_id: CharacterId,
        actor_id: &str,
        user_input: &str,
        personality: &Personality,
        state: &mut CharacterState,
        relationship: &mut Relationship,
        memories: &mut Vec<Memory>,
        now: u64,
    ) -> vc_core::Result<InteractionOutcome> {
        // 1. Session Management (docs/design/session.md)
        let session_id: SessionId = {
            let mut mgr = self.session_manager.write().expect("RwLock poisoned");
            let session = mgr.get_or_create(character_id, actor_id, now);
            session.touch(now);
            session.id
        };

        // 2. Memory Retrieval with strict actor isolation (Skill 12)
        let query = vc_core::memory::MemoryQuery::new(3)
            .with_actor(actor_id)
            .with_text(user_input);
        let retrieved_memories = crate::in_memory_store::InMemoryMemoryStore::retrieve_from_slice(
            memories,
            &query,
            now,
        );

        // 3. Context Construction & Budget Governance (Skill 16)
        let context = vc_core::context::ContextBuilder::new()
            .with_system_directive("You are Aria, an independent living virtual companion.")
            .with_personality(personality.clone())
            .with_state(state.clone())
            .with_relationship(relationship.clone())
            .with_memories(retrieved_memories.clone())
            .with_user_input(user_input)
            .build(vc_core::context::ContextBudget::standard_4k());

        // 4. Decision Evaluation (Skill 15)
        let decision_ctx = vc_core::decision::context::DecisionContext::new(
            user_input,
            Some(actor_id.to_string()),
            personality.clone(),
            state.clone(),
            Some(relationship.clone()),
            retrieved_memories.clone(),
        );
        let decision = self.decision_engine.make_decision(&decision_ctx)?;

        // 5. Response Generation via LLM
        let policy_str = if let Some(ref p) = decision.result.policy {
            format!("Giọng điệu: {}, Độ súc tích: {:.1}, Tính chủ động: {:.1}", p.tone, p.verbosity, p.initiative)
        } else {
            "Tự nhiên, ấm áp".to_string()
        };

        let prompt = format!(
            "{}\n\n[Quyết Định Nội Tâm]:\n- Hành động: {}\n- Ý đồ: {}\n- Độc thoại nội tâm: \"{}\"\n- Phong cách: {}\n\n[Lời Nhắn Người Dùng]: \"{}\"\n\nHãy phản hồi hoàn toàn tự nhiên và phản ánh đúng quyết định nội tâm trên:",
            context.render_for_llm(),
            decision.result.selected_action.action_type,
            decision.result.selected_action.description,
            decision.result.reasoning,
            policy_str,
            user_input
        );

        let llm_request = vc_llm::provider::LlmRequest {
            prompt,
            system_instruction: Some("You are a persistent, warm, and authentic AI character.".into()),
        };

        let response = self.llm_provider.generate_text(llm_request)?;
        let response_text = response.text;

        // 6. Emotion / State Feedback Update (EmotionEngine)
        let emotion_delta = self.emotion_engine.evaluate(&state.emotion, user_input, personality);
        state.emotion.apply_delta(&emotion_delta);
        state.sync_behavior(personality);

        // 7. Relationship Evolution
        let rel_transition = if state.emotion.valence() > 0.15 {
            RelationshipTransition::positive_interaction()
        } else if state.emotion.valence() < -0.15 {
            RelationshipTransition::conflicted_interaction()
        } else {
            RelationshipTransition::zero()
        };
        relationship.record_interaction(&rel_transition, now);

        // 8. Memory Consolidation (Forming new episodic memory)
        let memory_content = format!(
            "User discussed: \"{}\" | Aria responded: \"{}\"",
            summarize_snippet(user_input),
            summarize_snippet(&response_text)
        );
        let new_memory = Memory::new_episodic(
            memory_content,
            MemoryImportance::Medium,
            Some(actor_id.to_string()),
            now,
        );
        memories.push(new_memory.clone());

        // 9. Assemble and return complete Outcome
        Ok(InteractionOutcome {
            interaction_id: InteractionId::new(),
            session_id,
            response_text,
            decision,
            context_breakdown: context.breakdown,
            emotion_delta,
            relationship_transition: Some(rel_transition),
            formed_memory: Some(new_memory),
        })
    }
}

fn summarize_snippet(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() > 60 {
        format!("{}...", trimmed.chars().take(60).collect::<String>())
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_llm::mock::MockLlmProvider;

    #[test]
    fn test_runtime_engine_creation_and_tick() {
        let mock_llm = Arc::new(MockLlmProvider {
            default_response: "Hello test".into(),
        });
        let runtime = RuntimeEngine::new(mock_llm);
        assert!(runtime.tick(CharacterId::new()).is_ok());
    }
}
