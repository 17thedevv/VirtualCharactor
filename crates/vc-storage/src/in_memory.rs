use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use vc_core::character::{Character, CharacterId};
use vc_core::error::{CoreError, Result};
use vc_core::memory::{Memory, MemoryId, MemoryQuery};
use vc_core::personality::Personality;
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;

use crate::repository::{
    CharacterRepository, MemoryRepository, RelationshipRepository, StateRepository,
};

/// A thread-safe in-memory storage implementation for testing and development.
#[derive(Debug, Clone, Default)]
pub struct InMemoryStorage {
    characters: Arc<RwLock<HashMap<CharacterId, Character>>>,
    personalities: Arc<RwLock<HashMap<CharacterId, Personality>>>,
    states: Arc<RwLock<HashMap<CharacterId, CharacterState>>>,
    relationships: Arc<RwLock<HashMap<(CharacterId, String), Relationship>>>,
    memories: Arc<RwLock<HashMap<CharacterId, Vec<Memory>>>>,
    embeddings: Arc<RwLock<HashMap<MemoryId, Vec<f32>>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CharacterRepository for InMemoryStorage {
    fn get_character(&self, id: CharacterId) -> Result<Character> {
        self.characters
            .read()
            .get(&id)
            .cloned()
            .ok_or_else(|| CoreError::CharacterNotFound(format!("Character {} not found", id.0)))
    }

    fn save_character(&self, character: &Character) -> Result<()> {
        self.characters
            .write()
            .insert(character.id, character.clone());
        Ok(())
    }

    fn get_personality(&self, character_id: CharacterId) -> Result<Option<Personality>> {
        Ok(self.personalities.read().get(&character_id).cloned())
    }

    fn save_personality(&self, character_id: CharacterId, personality: &Personality) -> Result<()> {
        self.personalities
            .write()
            .insert(character_id, personality.clone());
        Ok(())
    }

    fn list_characters(&self) -> Result<Vec<Character>> {
        Ok(self.characters.read().values().cloned().collect())
    }
}

impl StateRepository for InMemoryStorage {
    fn get_state(&self, character_id: CharacterId) -> Result<Option<CharacterState>> {
        Ok(self.states.read().get(&character_id).cloned())
    }

    fn save_state(&self, character_id: CharacterId, state: &CharacterState) -> Result<()> {
        self.states.write().insert(character_id, state.clone());
        Ok(())
    }
}

impl RelationshipRepository for InMemoryStorage {
    fn get_relationship(
        &self,
        character_id: CharacterId,
        actor_id: &str,
    ) -> Result<Option<Relationship>> {
        let key = (character_id, actor_id.to_string());
        Ok(self.relationships.read().get(&key).cloned())
    }

    fn save_relationship(&self, relationship: &Relationship) -> Result<()> {
        let key = (relationship.character_id, relationship.target_id.clone());
        self.relationships.write().insert(key, relationship.clone());
        Ok(())
    }

    fn list_relationships(&self, character_id: CharacterId) -> Result<Vec<Relationship>> {
        let rels = self.relationships.read();
        Ok(rels
            .iter()
            .filter(|((cid, _), _)| *cid == character_id)
            .map(|(_, rel)| rel.clone())
            .collect())
    }
}

impl MemoryRepository for InMemoryStorage {
    fn get_memory(&self, id: MemoryId) -> Result<Memory> {
        let guard = self.memories.read();
        for mems in guard.values() {
            if let Some(m) = mems.iter().find(|m| m.id == id) {
                return Ok(m.clone());
            }
        }
        Err(CoreError::MemoryNotFound(format!(
            "Memory {} not found",
            id.0
        )))
    }

    fn save_memory(&self, memory: &Memory) -> Result<()> {
        let mut guard = self.memories.write();
        let target_id = memory
            .character_id
            .unwrap_or_else(|| CharacterId(uuid::Uuid::nil()));
        let list = guard.entry(target_id).or_default();
        if let Some(idx) = list.iter().position(|m| m.id == memory.id) {
            list[idx] = memory.clone();
        } else {
            list.push(memory.clone());
        }
        Ok(())
    }

    fn list_memories(&self, character_id: CharacterId) -> Result<Vec<Memory>> {
        let guard = self.memories.read();
        Ok(guard.get(&character_id).cloned().unwrap_or_default())
    }

    fn query_memories(
        &self,
        character_id: CharacterId,
        query: &MemoryQuery,
        now: u64,
    ) -> Result<Vec<Memory>> {
        let mut guard = self.memories.write();
        let list = guard.entry(character_id).or_default();

        let mut scored: Vec<(f32, usize)> = Vec::new();

        for (idx, mem) in list.iter().enumerate() {
            // 1. Strict Actor Privacy Isolation (Skill 12 Rule)
            if !mem.can_be_retrieved_by(query.actor_id.as_deref()) {
                continue;
            }

            // 2. Filter by Memory Type if specified
            if let Some(ref required_type) = query.memory_type {
                if &mem.metadata.memory_type != required_type {
                    continue;
                }
            }

            // 3. Filter by minimum importance threshold
            let eff_imp = mem.effective_importance();
            if let Some(min_imp) = query.min_importance {
                if eff_imp < min_imp {
                    continue;
                }
            }

            // 4. Keyword / Semantic match score [0.0, 1.0]
            let (sim_score, has_text_filter) = match &query.query_text {
                Some(text) if !text.trim().is_empty() => {
                    let query_tokens: Vec<&str> = text.split_whitespace().collect();
                    let content_lower = mem.content.to_lowercase();
                    let mut matched = 0;
                    for token in &query_tokens {
                        let tok_lower = token.to_lowercase();
                        if content_lower.contains(&tok_lower)
                            || mem.metadata.tags.iter().any(|t| t.contains(&tok_lower))
                        {
                            matched += 1;
                        }
                    }
                    if query_tokens.is_empty() {
                        (0.5, false)
                    } else {
                        ((matched as f32) / (query_tokens.len() as f32), true)
                    }
                }
                _ => (0.5, false),
            };

            // If a specific query was provided, skip non-pinned memories with zero keyword relevance
            if !mem.lifecycle.is_pinned && has_text_filter && sim_score == 0.0 {
                continue;
            }

            // 5. Recency score [0.0, 1.0]
            let age_secs = now.saturating_sub(mem.metadata.timestamp);
            let age_days = (age_secs as f32) / 86400.0;
            let recency_score = (-0.05 * age_days).exp();

            // 6. Multidimensional composite score
            let composite_score = (sim_score * query.semantic_weight)
                + (recency_score * query.recency_weight)
                + (eff_imp * query.importance_weight);

            scored.push((composite_score, idx));
        }

        // Sort descending by composite score
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let limit = query.limit.max(1);
        let selected_indices: Vec<usize> =
            scored.into_iter().take(limit).map(|(_, idx)| idx).collect();

        let mut results = Vec::with_capacity(selected_indices.len());
        for idx in selected_indices {
            list[idx].touch(now);
            results.push(list[idx].clone());
        }

        Ok(results)
    }

    fn delete_memory(&self, id: MemoryId) -> Result<()> {
        self.embeddings.write().remove(&id);
        let mut guard = self.memories.write();
        for list in guard.values_mut() {
            if let Some(pos) = list.iter().position(|m| m.id == id) {
                list.remove(pos);
                return Ok(());
            }
        }
        Ok(())
    }

    fn clear_memories(&self, character_id: CharacterId) -> Result<()> {
        let mut guard = self.memories.write();
        if let Some(list) = guard.remove(&character_id) {
            let mut emb_guard = self.embeddings.write();
            for m in list {
                emb_guard.remove(&m.id);
            }
        }
        Ok(())
    }

    fn save_memory_with_embedding(&self, memory: &Memory, embedding: &[f32]) -> Result<()> {
        self.save_memory(memory)?;
        self.embeddings
            .write()
            .insert(memory.id, embedding.to_vec());
        Ok(())
    }

    fn get_memory_embedding(&self, id: MemoryId) -> Result<Option<Vec<f32>>> {
        Ok(self.embeddings.read().get(&id).cloned())
    }

    fn search_similar_memories(
        &self,
        character_id: CharacterId,
        query: &crate::embedding::VectorMemoryQuery,
    ) -> Result<Vec<(Memory, f32)>> {
        let memories = self.list_memories(character_id)?;
        let emb_guard = self.embeddings.read();
        let mut candidates = Vec::new();

        for mem in memories {
            // Strict Actor Privacy Isolation (Skill 12 / 25)
            if !mem.can_be_retrieved_by(query.actor_id.as_deref()) {
                continue;
            }

            if let Some(emb) = emb_guard.get(&mem.id) {
                let sim = crate::embedding::cosine_similarity(&query.query_vector, emb);
                if sim >= query.min_similarity {
                    candidates.push((mem, sim));
                }
            }
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_k = query.top_k.max(1);
        candidates.truncate(top_k);

        Ok(candidates)
    }
}
