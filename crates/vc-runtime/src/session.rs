use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vc_core::character::CharacterId;

/// Unique identifier for a runtime session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lifecycle state machine of a session under docs/design/session.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session created, awaiting first active exchange
    Created,
    /// Actively receiving interactions
    Active,
    /// Inactive past a small idle duration (candidate for memory consolidation)
    Idle,
    /// Terminated intentionally by user or system
    Completed,
    /// Terminated due to extended timeout
    Expired,
    /// Terminated due to unrecoverable system anomaly
    Cancelled,
}

impl SessionStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Created | Self::Active | Self::Idle)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Expired | Self::Cancelled)
    }
}

/// Ephemeral runtime availability container for a Character and an Actor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSession {
    pub id: SessionId,
    pub character_id: CharacterId,
    pub actor_id: String,
    pub status: SessionStatus,
    pub created_at: u64,
    pub last_active_at: u64,
    pub interaction_count: u64,
}

impl CharacterSession {
    pub fn new(character_id: CharacterId, actor_id: impl Into<String>, now: u64) -> Self {
        Self {
            id: SessionId::new(),
            character_id,
            actor_id: actor_id.into(),
            status: SessionStatus::Created,
            created_at: now,
            last_active_at: now,
            interaction_count: 0,
        }
    }

    /// Record interaction activity, updating status to Active and bumping counter.
    pub fn touch(&mut self, now: u64) {
        self.status = SessionStatus::Active;
        self.last_active_at = now;
        self.interaction_count += 1;
    }

    pub fn mark_idle(&mut self) {
        if self.status == SessionStatus::Active {
            self.status = SessionStatus::Idle;
        }
    }

    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
    }

    pub fn expire(&mut self) {
        self.status = SessionStatus::Expired;
    }

    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }
}

/// Manages active runtime sessions in memory.
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: HashMap<(CharacterId, String), CharacterSession>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve active session or create a new one if none exists or previous expired.
    pub fn get_or_create(&mut self, character_id: CharacterId, actor_id: &str, now: u64) -> &mut CharacterSession {
        let key = (character_id, actor_id.to_string());
        
        let needs_new = match self.sessions.get(&key) {
            Some(sess) => !sess.is_active(),
            None => true,
        };

        if needs_new {
            let new_session = CharacterSession::new(character_id, actor_id, now);
            self.sessions.insert(key.clone(), new_session);
        }

        self.sessions.get_mut(&key).expect("Session must exist after insert")
    }

    pub fn get_session(&self, character_id: CharacterId, actor_id: &str) -> Option<&CharacterSession> {
        self.sessions.get(&(character_id, actor_id.to_string()))
    }

    pub fn complete_session(&mut self, character_id: CharacterId, actor_id: &str) -> bool {
        if let Some(session) = self.sessions.get_mut(&(character_id, actor_id.to_string())) {
            session.complete();
            true
        } else {
            false
        }
    }

    /// Check and transition sessions that have exceeded idle timeout.
    pub fn check_idle_timeouts(&mut self, now: u64, idle_seconds: u64) {
        for session in self.sessions.values_mut() {
            if session.status == SessionStatus::Active && now.saturating_sub(session.last_active_at) >= idle_seconds {
                session.mark_idle();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle_and_transitions() {
        let char_id = CharacterId::new();
        let mut session = CharacterSession::new(char_id, "user-1", 1000);
        assert_eq!(session.status, SessionStatus::Created);
        assert!(session.is_active());

        // Touch turns it to Active
        session.touch(1010);
        assert_eq!(session.status, SessionStatus::Active);
        assert_eq!(session.interaction_count, 1);
        assert_eq!(session.last_active_at, 1010);

        // Mark idle
        session.mark_idle();
        assert_eq!(session.status, SessionStatus::Idle);
        assert!(session.is_active());

        // Complete
        session.complete();
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(!session.is_active());
        assert!(session.status.is_terminal());
    }

    #[test]
    fn test_session_manager_recreates_terminal_sessions() {
        let mut manager = SessionManager::new();
        let char_id = CharacterId::new();

        // 1. First get_or_create
        let sess_id1 = {
            let s = manager.get_or_create(char_id, "user-1", 1000);
            s.touch(1005);
            s.id
        };

        // 2. Complete session
        manager.complete_session(char_id, "user-1");

        // 3. Next get_or_create spawns a fresh session
        let sess_id2 = {
            let s = manager.get_or_create(char_id, "user-1", 1100);
            s.id
        };

        assert_ne!(sess_id1, sess_id2, "New active session must be created after completion");
    }
}
