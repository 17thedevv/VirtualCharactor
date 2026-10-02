use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use vc_core::character::{Character, CharacterId};
use vc_core::memory::Memory;
use vc_core::personality::{Personality, TraitScore};
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;

use crate::error::{Result, RuntimeError};
use crate::interaction::InteractionOutcome;
use crate::runtime::RuntimeEngine;

/// Complete in-memory profile bundle for a living character.
#[derive(Clone, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub character: Character,
    pub personality: Personality,
    pub state: CharacterState,
    pub memories: Vec<Memory>,
    pub relationships: HashMap<String, Relationship>,
}

impl CharacterProfile {
    pub fn new(character: Character, personality: Personality, state: CharacterState) -> Self {
        Self {
            character,
            personality,
            state,
            memories: Vec::new(),
            relationships: HashMap::new(),
        }
    }

    /// Helper to create Hikari (a lively, energetic gaming VTuber companion).
    pub fn preset_hikari() -> Self {
        let char_id = CharacterId::new();
        let character = Character {
            id: char_id,
            name: "Hikari".to_string(),
        };
        let mut personality = Personality::baseline_aria();
        personality.identity.name = "Hikari".to_string();
        personality.identity.role = "Tsundere Gamer VTuber".to_string();
        personality.identity.core_identity =
            "Năng động, cá tính, hay dỗi nhưng cực kỳ quan tâm bạn bè!".to_string();
        personality.traits.playfulness = TraitScore::clamped(0.9);
        personality.traits.assertiveness = TraitScore::clamped(0.8);

        let state = CharacterState::default_aria();
        Self::new(character, personality, state)
    }

    /// Retrieve or initialize relationship for an actor.
    pub fn get_or_create_relationship(&mut self, actor_id: &str) -> &mut Relationship {
        let char_id = self.character.id;
        self.relationships
            .entry(actor_id.to_string())
            .or_insert_with(|| Relationship::new_companion(char_id, actor_id))
    }
}

/// Lightweight summary of a registered character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSummary {
    pub id: CharacterId,
    pub name: String,
    pub archetype: String,
    pub is_active: bool,
}

/// Registry managing multiple character entities within a single runtime.
///
/// Under Skill 10 (Character Domain): Ensures absolute isolation between character personalities and states.
pub struct MultiCharacterRegistry {
    profiles: HashMap<CharacterId, CharacterProfile>,
    active_id: Option<CharacterId>,
}

impl MultiCharacterRegistry {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            active_id: None,
        }
    }

    /// Register a character profile into the registry.
    pub fn register(&mut self, profile: CharacterProfile) {
        let id = profile.character.id;
        if self.active_id.is_none() {
            self.active_id = Some(id);
        }
        self.profiles.insert(id, profile);
    }

    /// Switch active character in the runtime.
    pub fn switch_active(&mut self, char_id: CharacterId) -> Result<()> {
        if !self.profiles.contains_key(&char_id) {
            return Err(RuntimeError::Internal(format!(
                "Character {} not found in registry",
                char_id.0
            )));
        }
        self.active_id = Some(char_id);
        Ok(())
    }

    pub fn active_id(&self) -> Option<CharacterId> {
        self.active_id
    }

    pub fn get_active(&self) -> Option<&CharacterProfile> {
        self.active_id.and_then(|id| self.profiles.get(&id))
    }

    pub fn get_active_mut(&mut self) -> Option<&mut CharacterProfile> {
        let active = self.active_id?;
        self.profiles.get_mut(&active)
    }

    pub fn get_profile(&self, char_id: CharacterId) -> Option<&CharacterProfile> {
        self.profiles.get(&char_id)
    }

    pub fn get_profile_mut(&mut self, char_id: CharacterId) -> Option<&mut CharacterProfile> {
        self.profiles.get_mut(&char_id)
    }

    pub fn list_summaries(&self) -> Vec<CharacterSummary> {
        self.profiles
            .values()
            .map(|p| CharacterSummary {
                id: p.character.id,
                name: p.character.name.clone(),
                archetype: p.personality.identity.role.clone(),
                is_active: self.active_id == Some(p.character.id),
            })
            .collect()
    }
}

impl Default for MultiCharacterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A round in a multi-character dialogue session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTurn {
    pub speaker_id: CharacterId,
    pub speaker_name: String,
    pub listener_id: CharacterId,
    pub utterance: String,
    pub emotion: String,
    pub timestamp: u64,
}

/// Orchestrator for Character-to-Character (C2C) interactions.
///
/// Allows 2 virtual characters to converse, debate, and develop interpersonal relationships.
pub struct CharacterToCharacterDialogue {
    max_rounds: usize,
}

impl CharacterToCharacterDialogue {
    pub fn new(max_rounds: usize) -> Self {
        Self { max_rounds }
    }

    /// Execute a simulated multi-turn conversation between two characters.
    pub fn run_dialogue(
        &self,
        char_a: &mut CharacterProfile,
        char_b: &mut CharacterProfile,
        initial_topic: &str,
        runtime_a: &RuntimeEngine,
        runtime_b: &RuntimeEngine,
        start_ts: u64,
    ) -> Result<Vec<DialogueTurn>> {
        let mut history: Vec<DialogueTurn> = Vec::new();
        let mut current_input = initial_topic.to_string();

        let actor_for_a = format!("character:{}", char_b.character.id.0);
        let actor_for_b = format!("character:{}", char_a.character.id.0);

        for round in 0..self.max_rounds {
            let turn_ts = start_ts + (round as u64 * 10);

            // Step 1: Character A speaks
            let (turn_a, speech_a) = {
                let CharacterProfile {
                    ref character,
                    ref personality,
                    ref mut state,
                    ref mut memories,
                    ref mut relationships,
                } = *char_a;

                let rel_a = relationships
                    .entry(actor_for_a.clone())
                    .or_insert_with(|| Relationship::new_companion(character.id, &actor_for_a));

                let outcome_a: InteractionOutcome = runtime_a
                    .process_interaction(
                        character.id,
                        &actor_for_a,
                        &current_input,
                        personality,
                        state,
                        rel_a,
                        memories,
                        turn_ts,
                    )
                    .map_err(|e| RuntimeError::Internal(e.to_string()))?;

                let turn = DialogueTurn {
                    speaker_id: character.id,
                    speaker_name: character.name.clone(),
                    listener_id: char_b.character.id,
                    utterance: outcome_a.response_text.clone(),
                    emotion: format!("{:?}", state.emotion.dominant_emotion().0),
                    timestamp: turn_ts,
                };
                (turn, outcome_a.response_text)
            };
            history.push(turn_a);

            // Step 2: Character B hears Character A and responds
            let turn_b_ts = turn_ts + 5;
            let (turn_b, speech_b) = {
                let CharacterProfile {
                    ref character,
                    ref personality,
                    ref mut state,
                    ref mut memories,
                    ref mut relationships,
                } = *char_b;

                let rel_b = relationships
                    .entry(actor_for_b.clone())
                    .or_insert_with(|| Relationship::new_companion(character.id, &actor_for_b));

                let outcome_b: InteractionOutcome = runtime_b
                    .process_interaction(
                        character.id,
                        &actor_for_b,
                        &speech_a,
                        personality,
                        state,
                        rel_b,
                        memories,
                        turn_b_ts,
                    )
                    .map_err(|e| RuntimeError::Internal(e.to_string()))?;

                let turn = DialogueTurn {
                    speaker_id: character.id,
                    speaker_name: character.name.clone(),
                    listener_id: char_a.character.id,
                    utterance: outcome_b.response_text.clone(),
                    emotion: format!("{:?}", state.emotion.dominant_emotion().0),
                    timestamp: turn_b_ts,
                };
                (turn, outcome_b.response_text)
            };
            history.push(turn_b);

            // Set current_input for next round
            current_input = speech_b;
        }

        Ok(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use vc_llm::mock::MockLlmProvider;

    #[test]
    fn test_multi_character_registry_and_switching() {
        let mut registry = MultiCharacterRegistry::new();

        let char_a = Character {
            id: CharacterId::new(),
            name: "Aria".into(),
        };
        let prof_a = CharacterProfile::new(
            char_a,
            Personality::baseline_aria(),
            CharacterState::default_aria(),
        );
        let a_id = prof_a.character.id;

        let prof_b = CharacterProfile::preset_hikari();
        let b_id = prof_b.character.id;

        registry.register(prof_a);
        registry.register(prof_b);

        assert_eq!(registry.active_id(), Some(a_id));

        assert!(registry.switch_active(b_id).is_ok());
        assert_eq!(registry.active_id(), Some(b_id));
        assert_eq!(registry.get_active().unwrap().character.name, "Hikari");

        let summaries = registry.list_summaries();
        assert_eq!(summaries.len(), 2);
    }

    #[test]
    fn test_character_to_character_dialogue_and_relationship() {
        let mut char_a = CharacterProfile::new(
            Character {
                id: CharacterId::new(),
                name: "Aria".into(),
            },
            Personality::baseline_aria(),
            CharacterState::default_aria(),
        );
        let mut char_b = CharacterProfile::preset_hikari();

        let llm_a = Arc::new(MockLlmProvider::new(
            "Chào Hikari! Hôm nay cậu stream vui chứ?",
        ));
        let llm_b = Arc::new(MockLlmProvider::new(
            "Hừm, vui lắm chứ, nhưng mà cũng mệt xỉu luôn á Aria!",
        ));

        let runtime_a = RuntimeEngine::new(llm_a);
        let runtime_b = RuntimeEngine::new(llm_b);

        let dialogue = CharacterToCharacterDialogue::new(1); // 1 round = A speaks, B replies
        let history = dialogue
            .run_dialogue(
                &mut char_a,
                &mut char_b,
                "Chào buổi sáng",
                &runtime_a,
                &runtime_b,
                1000,
            )
            .expect("dialogue should succeed");

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].speaker_name, "Aria");
        assert_eq!(history[1].speaker_name, "Hikari");

        // Verify mutual relationships formed
        let rel_a_to_b = char_a
            .relationships
            .get(&format!("character:{}", char_b.character.id.0));
        assert!(rel_a_to_b.is_some());
        assert_eq!(rel_a_to_b.unwrap().interaction_count, 1);

        let rel_b_to_a = char_b
            .relationships
            .get(&format!("character:{}", char_a.character.id.0));
        assert!(rel_b_to_a.is_some());
        assert_eq!(rel_b_to_a.unwrap().interaction_count, 1);
    }
}
