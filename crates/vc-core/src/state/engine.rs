use crate::personality::Personality;
use super::emotion::EmotionState;
use super::transition::EmotionDelta;

/// Domain trait for computing emotion transitions.
///
/// Implementations live OUTSIDE `vc-core` (e.g., `vc-runtime` for rule-based,
/// or a future learned/hybrid engine crate). This keeps the core domain
/// free from infrastructure concerns.
///
/// # Design
///
/// The engine takes the current emotional state, the user's input,
/// and the character's personality to compute an `EmotionDelta`.
/// The runtime is responsible for applying this delta to the state.
pub trait EmotionEngine: Send + Sync {
    /// Evaluate the emotional impact of a user message.
    ///
    /// Returns an `EmotionDelta` that the runtime will apply to the
    /// current `EmotionState`.
    fn evaluate(
        &self,
        current_state: &EmotionState,
        user_input: &str,
        personality: &Personality,
    ) -> EmotionDelta;
}
