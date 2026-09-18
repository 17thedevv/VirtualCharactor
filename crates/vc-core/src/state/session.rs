use serde::{Deserialize, Serialize};

/// Session-scoped state that resets when a conversation ends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionState {
    /// Unique identifier for this session.
    pub session_id: String,
    /// Current topic of conversation, if any.
    pub current_topic: Option<String>,
    /// Overall mood of the conversation so far.
    pub conversation_mood: Option<String>,
    /// Number of turns in this session.
    pub turn_count: u32,
    /// Arbitrary key-value pairs for extensibility.
    pub variables: std::collections::HashMap<String, String>,
}

impl SessionState {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            current_topic: None,
            conversation_mood: None,
            turn_count: 0,
            variables: std::collections::HashMap::new(),
        }
    }

    /// Increment the turn counter.
    pub fn increment_turn(&mut self) {
        self.turn_count += 1;
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new("default-session")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session() {
        let session = SessionState::new("test-001");
        assert_eq!(session.session_id, "test-001");
        assert_eq!(session.turn_count, 0);
        assert!(session.current_topic.is_none());
    }

    #[test]
    fn test_increment_turn() {
        let mut session = SessionState::new("test");
        session.increment_turn();
        session.increment_turn();
        assert_eq!(session.turn_count, 2);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut session = SessionState::new("s-123");
        session.current_topic = Some("Rust".into());
        session.conversation_mood = Some("excited".into());
        session.turn_count = 5;
        let json = serde_json::to_string(&session).expect("serialize");
        let deser: SessionState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.turn_count, 5);
        assert_eq!(deser.current_topic, Some("Rust".into()));
    }
}
