use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vc_core::character::CharacterId;
use vc_core::error::Result;
use vc_core::memory::{Memory, MemoryImportance, MemoryType};
use vc_storage::MemoryRepository;

/// Report summarizing actions executed during a memory consolidation or sleep cycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsolidationReport {
    /// Total number of memories inspected during the cycle.
    pub memories_scanned: usize,
    /// Number of non-critical memories decayed.
    pub memories_decayed: usize,
    /// Number of ephemeral or decayed memories pruned.
    pub memories_pruned: usize,
    /// Number of high-level semantic insights consolidated from episodic logs.
    pub semantic_formed: usize,
}

/// Background cognitive worker responsible for offline memory pruning and episodic-to-semantic consolidation.
///
/// Implemented according to Skill 12 (Memory Engineering) and Phase 2.4 Roadmap:
/// When the character is idle or resting, this worker applies temporal decay to retention
/// strengths, removes forgotten trivial memories, and groups episodic interactions into
/// durable semantic facts.
pub struct MemoryConsolidator {
    storage: Arc<dyn MemoryRepository>,
    prune_threshold: f32,
}

impl MemoryConsolidator {
    /// Create a new consolidator with default pruning threshold (effective importance < 0.10).
    pub fn new(storage: Arc<dyn MemoryRepository>) -> Self {
        Self {
            storage,
            prune_threshold: 0.10,
        }
    }

    /// Set a custom pruning threshold.
    pub fn with_prune_threshold(mut self, threshold: f32) -> Self {
        self.prune_threshold = threshold.clamp(0.01, 0.99);
        self
    }

    /// Execute a complete consolidation pass for a character.
    pub fn run_cycle(
        &self,
        character_id: CharacterId,
        now: u64,
        elapsed_since_last_cycle: u64,
    ) -> Result<ConsolidationReport> {
        let memories = self.storage.list_memories(character_id)?;
        let mut report = ConsolidationReport {
            memories_scanned: memories.len(),
            memories_decayed: 0,
            memories_pruned: 0,
            semantic_formed: 0,
        };

        let mut surviving_episodic = Vec::new();

        for mut mem in memories {
            // 1. Core and pinned memories never decay or get pruned
            if mem.lifecycle.is_pinned || mem.metadata.importance == MemoryImportance::Critical {
                continue;
            }

            // 2. Apply decay proportional to elapsed duration
            mem.apply_decay(elapsed_since_last_cycle);
            report.memories_decayed += 1;

            // 3. Prune forgotten memories whose retention strength has fallen below threshold
            if mem.effective_importance() < self.prune_threshold {
                self.storage.delete_memory(mem.id)?;
                report.memories_pruned += 1;
            } else {
                // Save updated decay retention state
                self.storage.save_memory(&mem)?;

                if mem.metadata.memory_type == MemoryType::Episodic {
                    surviving_episodic.push(mem);
                }
            }
        }

        // 4. Episodic to Semantic Consolidation:
        // When there are 3 or more episodic memories from the same actor, consolidate them into a semantic fact
        let mut actor_groups: std::collections::HashMap<Option<String>, Vec<Memory>> =
            std::collections::HashMap::new();
        for mem in surviving_episodic {
            actor_groups
                .entry(mem.metadata.source_actor_id.clone())
                .or_default()
                .push(mem);
        }

        for (actor_opt, group) in actor_groups {
            if group.len() >= 3 {
                // Form a synthesized semantic memory
                let topics: Vec<&str> = group.iter().map(|m| m.content.as_str()).collect();
                let consolidated_content = format!(
                    "Consolidated knowledge from {} interactions: {}",
                    group.len(),
                    topics.join(" | ")
                );

                let mut semantic_mem = Memory::new_semantic(
                    consolidated_content,
                    MemoryImportance::Medium,
                    actor_opt,
                    now,
                );
                semantic_mem.character_id = Some(character_id);

                self.storage.save_memory(&semantic_mem)?;
                report.semantic_formed += 1;
            }
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_storage::InMemoryStorage;

    #[test]
    fn test_memory_decay_and_pruning_cycle() {
        let storage = Arc::new(InMemoryStorage::new());
        let consolidator = MemoryConsolidator::new(storage.clone()).with_prune_threshold(0.10);
        let char_id = CharacterId::new();

        // 1. Create a critical core memory (must never be pruned)
        let mut core = Memory::new_core("Core identity: Aria", 1000);
        core.character_id = Some(char_id);
        storage.save_memory(&core).unwrap();

        // 2. Create a low importance episodic memory (should be pruned after 20 days)
        let mut trivial = Memory::new_episodic(
            "User typed 'ok'",
            MemoryImportance::Low,
            Some("user-1".into()),
            1000,
        );
        trivial.character_id = Some(char_id);
        storage.save_memory(&trivial).unwrap();

        // 3. Create a high importance episodic memory (should decay but survive)
        let mut high = Memory::new_episodic(
            "User shared lifelong dream",
            MemoryImportance::High,
            Some("user-1".into()),
            1000,
        );
        high.character_id = Some(char_id);
        storage.save_memory(&high).unwrap();

        // Run cycle with 20 days elapsed (decay_factor = e^-1 ≈ 0.368; Low: 0.25 * 0.368 = 0.092 < 0.10; High: 0.8 * 0.368 = 0.294 > 0.10)
        let report = consolidator
            .run_cycle(char_id, 1000 + 86400 * 20, 86400 * 20)
            .unwrap();

        assert_eq!(report.memories_scanned, 3);
        assert_eq!(report.memories_pruned, 1); // trivial was pruned!

        // Verify remaining memories
        let remaining = storage.list_memories(char_id).unwrap();
        let remaining_ids: Vec<_> = remaining.iter().map(|m| m.id).collect();

        assert!(remaining_ids.contains(&core.id));
        assert!(remaining_ids.contains(&high.id));
        assert!(!remaining_ids.contains(&trivial.id));
    }

    #[test]
    fn test_episodic_to_semantic_consolidation() {
        let storage = Arc::new(InMemoryStorage::new());
        let consolidator = MemoryConsolidator::new(storage.clone());
        let char_id = CharacterId::new();

        // Add 3 episodic interactions from alice
        for i in 1..=3 {
            let mut mem = Memory::new_episodic(
                format!("Alice discussed topic {}", i),
                MemoryImportance::High,
                Some("alice".into()),
                1000 + i * 100,
            );
            mem.character_id = Some(char_id);
            storage.save_memory(&mem).unwrap();
        }

        // Run cycle
        let report = consolidator.run_cycle(char_id, 2000, 100).unwrap();
        assert_eq!(report.semantic_formed, 1);

        // Check that new semantic memory exists
        let all_memories = storage.list_memories(char_id).unwrap();
        let semantic = all_memories
            .iter()
            .find(|m| m.metadata.memory_type == MemoryType::Semantic);
        assert!(semantic.is_some());
        let s = semantic.unwrap();
        assert!(s
            .content
            .contains("Consolidated knowledge from 3 interactions"));
        assert_eq!(s.metadata.source_actor_id.as_deref(), Some("alice"));
    }
}
