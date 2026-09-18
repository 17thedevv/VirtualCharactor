use serde::{Deserialize, Serialize};
use super::types::MemoryImportance;

/// Temporal tracking, access statistics, and decay/reinforcement lifecycle of a memory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryLifecycle {
    /// Epoch timestamp when the memory was initially registered
    pub created_at: u64,
    /// Epoch timestamp when this memory was last retrieved or accessed
    pub last_accessed_at: u64,
    /// Number of times this memory has been retrieved and used in context
    pub access_count: u32,
    /// Rate at which this memory decays per day (0.0 means no natural decay)
    pub decay_rate: f32,
    /// Whether this memory is pinned (protected completely from decay/forgetting)
    pub is_pinned: bool,
    /// Dynamic retention strength [0.0, 1.0]. Starts at 1.0, decays over time, reinforced on touch.
    pub current_strength: f32,
}

impl MemoryLifecycle {
    pub fn new(now: u64, is_pinned: bool) -> Self {
        Self {
            created_at: now,
            last_accessed_at: now,
            access_count: 0,
            decay_rate: if is_pinned { 0.0 } else { 0.05 },
            is_pinned,
            current_strength: 1.0,
        }
    }

    /// Touch / access this memory during interaction, reinforcing its retention strength.
    pub fn touch(&mut self, now: u64) {
        self.access_count += 1;
        self.last_accessed_at = now;
        // Reinforce strength: Each retrieval boosts retention back towards 1.0
        self.current_strength = (self.current_strength + 0.20).min(1.0);
    }

    /// Apply temporal decay based on elapsed time in seconds.
    /// Pinned or Critical memories are protected from decay.
    pub fn apply_decay(&mut self, elapsed_secs: u64, is_critical: bool) {
        if self.is_pinned || is_critical || self.decay_rate <= 0.0 {
            return;
        }

        // Convert elapsed seconds to fractional days
        let elapsed_days = (elapsed_secs as f32) / 86400.0;
        // Exponential decay model: S(t) = S0 * e^(-λ * t)
        let decay_factor = (-self.decay_rate * elapsed_days).exp();
        self.current_strength = (self.current_strength * decay_factor).clamp(0.05, 1.0);
    }

    /// Calculate dynamic effective importance: base weight modulated by current retention strength.
    pub fn effective_importance(&self, base_importance: &MemoryImportance) -> f32 {
        base_importance.weight() * self.current_strength
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_reinforcement_on_touch() {
        let mut lc = MemoryLifecycle::new(1000, false);
        lc.current_strength = 0.5;

        lc.touch(1500);
        assert_eq!(lc.access_count, 1);
        assert_eq!(lc.last_accessed_at, 1500);
        assert!((lc.current_strength - 0.70).abs() < f32::EPSILON);
    }

    #[test]
    fn test_memory_decay() {
        let mut lc = MemoryLifecycle::new(1000, false);
        lc.decay_rate = 0.10;

        // Simulate 10 days of non-access (864,000 secs)
        lc.apply_decay(864000, false);
        assert!(lc.current_strength < 0.5);

        // Pinned memory should never decay
        let mut pinned = MemoryLifecycle::new(1000, true);
        pinned.apply_decay(864000, false);
        assert_eq!(pinned.current_strength, 1.0);

        // Critical memory should never decay
        let mut crit = MemoryLifecycle::new(1000, false);
        crit.apply_decay(864000, true);
        assert_eq!(crit.current_strength, 1.0);
    }
}
