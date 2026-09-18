use std::sync::{Arc, RwLock};
use vc_core::error::{CoreError, Result};
use vc_core::memory::query::MemoryQuery;
use vc_core::memory::traits::MemoryRetriever;
use vc_core::memory::Memory;

/// Deterministic in-memory implementation of the memory repository and retriever for Phase 1.
///
/// Complies with Skill 12 (Memory Engineering):
/// - Enforces strict actor-level isolation (no cross-user leakage).
/// - Combines keyword semantic relevance, base importance, and temporal recency.
/// - Limits memory results to respect context token budget.
#[derive(Debug, Clone)]
pub struct InMemoryMemoryStore {
    memories: Arc<RwLock<Vec<Memory>>>,
}

impl InMemoryMemoryStore {
    pub fn new() -> Self {
        Self {
            memories: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_initial_memories(initial: Vec<Memory>) -> Self {
        Self {
            memories: Arc::new(RwLock::new(initial)),
        }
    }

    /// Add a new memory to the store.
    pub fn add(&self, memory: Memory) -> Result<()> {
        let mut store = self.memories.write().map_err(|e| {
            CoreError::Internal(format!("RwLock poisoned: {}", e))
        })?;
        store.push(memory);
        Ok(())
    }

    /// Get total count of memories in the store.
    pub fn len(&self) -> usize {
        self.memories.read().map(|m| m.len()).unwrap_or(0)
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Synchronous retrieval implementation against a slice of memories.
    pub fn retrieve_from_slice(memories: &mut [Memory], query: &MemoryQuery, now: u64) -> Vec<Memory> {
        let mut scored: Vec<(f32, usize)> = Vec::new();

        for (idx, mem) in memories.iter().enumerate() {
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
            let sim_score = match &query.query_text {
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
                        0.5
                    } else {
                        (matched as f32) / (query_tokens.len() as f32)
                    }
                }
                _ => 0.5, // Neutral baseline when no text filter is provided
            };

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

        // Take top items limited by query budget constraint
        let limit = query.limit.max(1);
        let selected_indices: Vec<usize> = scored.into_iter().take(limit).map(|(_, idx)| idx).collect();

        let mut results = Vec::with_capacity(selected_indices.len());
        for idx in selected_indices {
            // Touch the retrieved memory to reinforce retention (access reinforcement)
            memories[idx].touch(now);
            results.push(memories[idx].clone());
        }

        results
    }
}

impl Default for InMemoryMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryRetriever for InMemoryMemoryStore {
    fn retrieve(&self, query: &MemoryQuery) -> Result<Vec<Memory>> {
        let mut store = self.memories.write().map_err(|e| {
            CoreError::Internal(format!("RwLock poisoned: {}", e))
        })?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Ok(Self::retrieve_from_slice(&mut store, query, now))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use vc_core::memory::MemoryImportance;

    #[test]
    fn test_in_memory_retrieval_and_actor_isolation() {
        let now = 10000;
        let mut memories = vec![
            Memory::new_core("VirtualCharacter core awakened in Rust", now - 5000),
            Memory::new_episodic("Alice shared her dream project about compilers", MemoryImportance::High, Some("alice".into()), now - 1000),
            Memory::new_episodic("Bob enjoys hiking in the mountains", MemoryImportance::Medium, Some("bob".into()), now - 2000),
        ];

        // Query by Alice: Should get Core + Alice memory, NEVER Bob memory
        let query_alice = MemoryQuery::new(5)
            .with_actor("alice")
            .with_text("compiler");

        let results_alice = InMemoryMemoryStore::retrieve_from_slice(&mut memories, &query_alice, now);
        assert_eq!(results_alice.len(), 2);
        assert!(results_alice.iter().any(|m| m.content.contains("Alice")));
        assert!(results_alice.iter().any(|m| m.content.contains("core")));
        assert!(!results_alice.iter().any(|m| m.content.contains("Bob")));

        // Query by Bob: Should NEVER see Alice's compiler memory
        let query_bob = MemoryQuery::new(5)
            .with_actor("bob")
            .with_text("compiler");

        let results_bob = InMemoryMemoryStore::retrieve_from_slice(&mut memories, &query_bob, now);
        assert!(!results_bob.iter().any(|m| m.content.contains("Alice")));
    }

    #[test]
    fn test_importance_and_budget_limiting() {
        let now = 10000;
        let mut memories = vec![
            Memory::new_episodic("Low priority chat detail 1", MemoryImportance::Low, Some("alice".into()), now),
            Memory::new_episodic("Critical life revelation", MemoryImportance::Critical, Some("alice".into()), now),
            Memory::new_episodic("Low priority chat detail 2", MemoryImportance::Low, Some("alice".into()), now),
        ];

        let query = MemoryQuery::new(1) // strict limit 1
            .with_actor("alice");

        let results = InMemoryMemoryStore::retrieve_from_slice(&mut memories, &query, now);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].metadata.importance, MemoryImportance::Critical);
    }
}
