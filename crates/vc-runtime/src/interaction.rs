use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vc_core::character::CharacterId;
use vc_core::context::ContextBreakdown;
use vc_core::decision::Decision;
use vc_core::memory::Memory;
use vc_core::relationship::transition::RelationshipTransition;
use vc_core::state::EmotionDelta;

use crate::session::SessionId;

/// Unique identifier for an interaction unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InteractionId(pub Uuid);

impl InteractionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for InteractionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for InteractionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status of an interaction cycle under docs/design/interaction.md.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionStatus {
    Started,
    Processing,
    Completed,
    Failed(String),
}

/// Represents an individual interaction turn between an Actor and the Character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: InteractionId,
    pub session_id: SessionId,
    pub character_id: CharacterId,
    pub actor_id: String,
    pub user_input: String,
    pub status: InteractionStatus,
    pub started_at: u64,
    pub completed_at: Option<u64>,
}

impl Interaction {
    pub fn new(
        session_id: SessionId,
        character_id: CharacterId,
        actor_id: impl Into<String>,
        user_input: impl Into<String>,
        now: u64,
    ) -> Self {
        Self {
            id: InteractionId::new(),
            session_id,
            character_id,
            actor_id: actor_id.into(),
            user_input: user_input.into(),
            status: InteractionStatus::Started,
            started_at: now,
            completed_at: None,
        }
    }

    pub fn complete(&mut self, now: u64) {
        self.status = InteractionStatus::Completed;
        self.completed_at = Some(now);
    }

    pub fn fail(&mut self, reason: impl Into<String>, now: u64) {
        self.status = InteractionStatus::Failed(reason.into());
        self.completed_at = Some(now);
    }
}

/// Complete aggregated outcome of an orchestrated interaction turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionOutcome {
    pub interaction_id: InteractionId,
    pub session_id: SessionId,
    pub response_text: String,
    pub decision: Decision,
    pub context_breakdown: ContextBreakdown,
    pub emotion_delta: EmotionDelta,
    pub relationship_transition: Option<RelationshipTransition>,
    pub formed_memory: Option<Memory>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_lifecycle_states() {
        let sess_id = SessionId::new();
        let char_id = CharacterId::new();
        let mut interaction = Interaction::new(sess_id, char_id, "user-1", "Hello", 1000);

        assert_eq!(interaction.status, InteractionStatus::Started);
        assert_eq!(interaction.completed_at, None);

        interaction.complete(1050);
        assert_eq!(interaction.status, InteractionStatus::Completed);
        assert_eq!(interaction.completed_at, Some(1050));
    }
}
