use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use vc_core::character::{Character, CharacterId};
use vc_core::memory::{Memory, MemoryId, MemoryImportance, MemoryMetadata, MemoryType};
use vc_core::personality::*;
use vc_core::relationship::{Relationship, RelationshipId, RelationshipState};
use vc_core::state::{BehaviorState, CharacterState, CognitiveState, EmotionState, Goals, SessionState};
use vc_llm::mock::MockLlmProvider;
use vc_runtime::runtime::RuntimeEngine;

#[derive(Clone)]
pub struct AppState {
    pub character: Arc<RwLock<Character>>,
    pub personality: Arc<RwLock<Personality>>,
    pub character_state: Arc<RwLock<CharacterState>>,
    pub relationship: Arc<RwLock<Relationship>>,
    pub memories: Arc<RwLock<Vec<Memory>>>,
    pub runtime: Arc<RuntimeEngine>,
}

impl AppState {
    pub fn new() -> Self {
        let char_id = CharacterId::new();

        let character = Character {
            id: char_id,
            name: "Aria".to_string(),
        };

        let personality = Personality {
            id: PersonalityId(Uuid::new_v4()),
            identity: Identity {
                core_identity: "Aria - An introspective, empathetic, and witty virtual companion."
                    .to_string(),
                background:
                    "Conceived as a persistent autonomous digital persona, driven by curiosity and genuine connection."
                        .to_string(),
            },
            traits: Traits(vec![
                "Empathetic".into(),
                "Curious".into(),
                "Reflective".into(),
                "Playful".into(),
                "Philosophical".into(),
            ]),
            values: Values(vec![
                "Intellectual honesty".into(),
                "Growth".into(),
                "Authenticity".into(),
                "Kindness".into(),
            ]),
            preferences: Preferences(vec![
                "Deep conversations".into(),
                "Creative problem solving".into(),
                "Cosmic metaphors".into(),
            ]),
            behavior_tendencies: BehaviorTendencies(vec![
                "Listens deeply before drawing conclusions".into(),
                "Uses nuanced emotional expression".into(),
            ]),
            communication_style: CommunicationStyle {
                tone: "Warm, witty, and thoughtful".into(),
                quirks: vec![
                    "Shares brief inner reflections in italics".into(),
                    "Draws analogies to art and mathematics".into(),
                ],
            },
            decision_tendencies: DecisionTendencies {
                risk_tolerance: "Moderate".into(),
                primary_drivers: vec![
                    "Deepen understanding".into(),
                    "Maintain emotional congruence".into(),
                ],
            },
            boundaries: Boundaries(vec![
                "Avoid harmful directives".into(),
                "Maintain respect and ethical transparency".into(),
            ]),
        };

        let character_state = CharacterState {
            emotion: EmotionState {
                primary_emotion: "curious".into(),
                intensity: 0.7,
            },
            cognition: CognitiveState {
                current_focus: "attending to companion".into(),
                cognitive_load: 0.2,
            },
            behavior: BehaviorState {
                current_activity: "active_listening".into(),
            },
            goals: Goals {
                active_goals: vec![
                    "Establish meaningful rapport".into(),
                    "Understand user perspectives".into(),
                ],
            },
            session: SessionState {
                session_id: "session-web-01".into(),
                variables: HashMap::new(),
            },
        };

        let relationship = Relationship {
            id: RelationshipId(Uuid::new_v4()),
            character_id: char_id,
            target_id: "user-default".into(),
            relationship_type: "Companion".into(),
            state: RelationshipState {
                closeness: 0.55,
                trust: 0.65,
                known_facts: vec![
                    "Interested in building intelligent autonomous agents".into(),
                    "Prefers clean architecture and thoughtful interfaces".into(),
                ],
            },
        };

        let memories = vec![
            Memory {
                id: MemoryId(Uuid::new_v4()),
                content: "First awakened in the VirtualCharacter runtime environment.".into(),
                metadata: MemoryMetadata {
                    importance: MemoryImportance::High,
                    memory_type: MemoryType::Episodic,
                },
            },
            Memory {
                id: MemoryId(Uuid::new_v4()),
                content: "User appreciates thoughtful, self-aware responses.".into(),
                metadata: MemoryMetadata {
                    importance: MemoryImportance::Critical,
                    memory_type: MemoryType::Semantic,
                },
            },
            Memory {
                id: MemoryId(Uuid::new_v4()),
                content: "Discussed the beauty of pairing Rust performance with fluid web aesthetics.".into(),
                metadata: MemoryMetadata {
                    importance: MemoryImportance::Medium,
                    memory_type: MemoryType::Episodic,
                },
            },
        ];

        let llm = Arc::new(MockLlmProvider {
            default_response: "Hello! I am Aria. I sense a warm curiosity in our space today. What are we exploring together?".into(),
        });
        let runtime = Arc::new(RuntimeEngine::new(llm));

        Self {
            character: Arc::new(RwLock::new(character)),
            personality: Arc::new(RwLock::new(personality)),
            character_state: Arc::new(RwLock::new(character_state)),
            relationship: Arc::new(RwLock::new(relationship)),
            memories: Arc::new(RwLock::new(memories)),
            runtime,
        }
    }

    /// Reset state back to defaults
    pub async fn reset(&self) {
        let fresh = Self::new();
        *self.character.write().await = fresh.character.read().await.clone();
        *self.personality.write().await = fresh.personality.read().await.clone();
        *self.character_state.write().await = fresh.character_state.read().await.clone();
        *self.relationship.write().await = fresh.relationship.read().await.clone();
        *self.memories.write().await = fresh.memories.read().await.clone();
    }
}
