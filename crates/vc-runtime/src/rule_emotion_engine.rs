use vc_core::personality::Personality;
use vc_core::state::emotion::EmotionState;
use vc_core::state::engine::EmotionEngine;
use vc_core::state::transition::EmotionDelta;

/// A rule-based implementation of the EmotionEngine trait.
///
/// Uses keyword matching on Vietnamese + English text to compute
/// emotion deltas, modulated by the character's personality traits.
///
/// This is the first (Phase 0) engine. Future engines can use
/// learned models or hybrid approaches while implementing the
/// same `EmotionEngine` trait.
pub struct RuleBasedEmotionEngine;

/// A keyword rule mapping input patterns to emotion deltas.
struct KeywordRule {
    /// Keywords that trigger this rule (Vietnamese + English).
    keywords: Vec<&'static str>,
    /// Base delta to apply when the rule matches.
    base_delta: EmotionDelta,
}

impl RuleBasedEmotionEngine {
    pub fn new() -> Self {
        Self
    }

    fn keyword_rules() -> Vec<KeywordRule> {
        vec![
            // Joy / Happiness
            KeywordRule {
                keywords: vec![
                    "vui", "tuyệt", "hay", "thích", "yêu", "tốt", "giỏi", "xuất sắc",
                    "happy", "great", "awesome", "love", "wonderful", "amazing", "excited",
                    "haha", "hihi", "😊", "😄", "❤️", "🎉",
                ],
                base_delta: EmotionDelta {
                    joy: 0.20,
                    sadness: -0.05,
                    anger: -0.03,
                    affection: 0.10,
                    curiosity: 0.05,
                    ..EmotionDelta::zero()
                },
            },
            // Sadness / Tiredness
            KeywordRule {
                keywords: vec![
                    "buồn", "mệt", "chán", "cô đơn", "khóc", "thất vọng", "đau",
                    "sad", "tired", "lonely", "cry", "disappointed", "pain", "hurt",
                    "😢", "😭", "💔",
                ],
                base_delta: EmotionDelta {
                    joy: -0.10,
                    sadness: 0.25,
                    affection: 0.08,  // Empathic response
                    embarrassment: 0.02,
                    ..EmotionDelta::zero()
                },
            },
            // Anger / Frustration
            KeywordRule {
                keywords: vec![
                    "tức", "giận", "bực", "ghét", "khó chịu", "ức",
                    "angry", "hate", "annoyed", "frustrated", "furious",
                    "😡", "🤬",
                ],
                base_delta: EmotionDelta {
                    anger: 0.20,
                    joy: -0.08,
                    fear: 0.05,
                    sadness: 0.05,
                    ..EmotionDelta::zero()
                },
            },
            // Fear / Worry
            KeywordRule {
                keywords: vec![
                    "sợ", "lo", "bất an", "hoang mang", "kinh", "hãi",
                    "afraid", "scared", "worried", "anxious", "fear",
                    "😨", "😰",
                ],
                base_delta: EmotionDelta {
                    fear: 0.20,
                    sadness: 0.05,
                    anger: 0.03,
                    curiosity: -0.05,
                    ..EmotionDelta::zero()
                },
            },
            // Surprise
            KeywordRule {
                keywords: vec![
                    "bất ngờ", "ngạc nhiên", "wow", "whoa", "sao lại", "thật sao",
                    "surprised", "unexpected", "unbelievable", "no way",
                    "😱", "😲", "🤯",
                ],
                base_delta: EmotionDelta {
                    surprise: 0.30,
                    curiosity: 0.15,
                    ..EmotionDelta::zero()
                },
            },
            // Curiosity / Questioning
            KeywordRule {
                keywords: vec![
                    "tại sao", "như thế nào", "là gì", "thế nào", "giải thích",
                    "tò mò", "muốn biết", "hỏi",
                    "why", "how", "what", "explain", "curious", "wonder", "tell me",
                    "🤔",
                ],
                base_delta: EmotionDelta {
                    curiosity: 0.20,
                    joy: 0.05,
                    surprise: 0.05,
                    ..EmotionDelta::zero()
                },
            },
            // Affection / Closeness
            KeywordRule {
                keywords: vec![
                    "nhớ", "quan tâm", "ôm", "thương", "gần gũi", "bên cạnh",
                    "miss", "care", "hug", "close", "together", "appreciate",
                    "🤗", "💕", "💗",
                ],
                base_delta: EmotionDelta {
                    affection: 0.25,
                    joy: 0.10,
                    sadness: -0.05,
                    ..EmotionDelta::zero()
                },
            },
            // Embarrassment
            KeywordRule {
                keywords: vec![
                    "xấu hổ", "ngại", "ngượng", "mắc cỡ", "quê",
                    "embarrassed", "shy", "awkward", "blush",
                    "😳", "🙈",
                ],
                base_delta: EmotionDelta {
                    embarrassment: 0.25,
                    joy: 0.03,
                    affection: 0.05,
                    ..EmotionDelta::zero()
                },
            },
            // Greeting (mild positive)
            KeywordRule {
                keywords: vec![
                    "chào", "hello", "hi", "xin chào", "hey",
                    "👋",
                ],
                base_delta: EmotionDelta {
                    joy: 0.10,
                    curiosity: 0.15,
                    affection: 0.08,
                    surprise: 0.05,
                    ..EmotionDelta::zero()
                },
            },
        ]
    }

    /// Modulate a base delta by personality traits.
    ///
    /// For example, a highly empathetic character will have amplified
    /// affection and sadness responses.
    fn modulate_by_personality(delta: &EmotionDelta, personality: &Personality) -> EmotionDelta {
        let empathy = personality.traits.empathy.value();
        let playfulness = personality.traits.playfulness.value();
        let curiosity = personality.traits.curiosity.value();

        EmotionDelta {
            joy: delta.joy * (0.7 + playfulness * 0.6),
            sadness: delta.sadness * (0.6 + empathy * 0.8),
            anger: delta.anger * (0.5 + (1.0 - personality.traits.patience.value()) * 0.5),
            fear: delta.fear * 0.8,
            surprise: delta.surprise * 0.9,
            affection: delta.affection * (0.6 + empathy * 0.8),
            embarrassment: delta.embarrassment * (0.5 + (1.0 - personality.traits.assertiveness.value()) * 0.5),
            curiosity: delta.curiosity * (0.6 + curiosity * 0.8),
        }
    }

    /// Apply smoothing based on current state to avoid jarring jumps.
    ///
    /// When current emotion is already high on an axis, additional increases
    /// have diminishing returns (soft saturation).
    fn smooth(delta: &EmotionDelta, current: &EmotionState) -> EmotionDelta {
        fn dampen(delta_val: f32, current_val: f32) -> f32 {
            if delta_val > 0.0 {
                // Diminishing returns: harder to push already-high emotions higher
                delta_val * (1.0 - current_val * 0.4)
            } else {
                // Easier to reduce already-low emotions? Keep as-is
                delta_val
            }
        }

        EmotionDelta {
            joy: dampen(delta.joy, current.joy.value()),
            sadness: dampen(delta.sadness, current.sadness.value()),
            anger: dampen(delta.anger, current.anger.value()),
            fear: dampen(delta.fear, current.fear.value()),
            surprise: dampen(delta.surprise, current.surprise.value()),
            affection: dampen(delta.affection, current.affection.value()),
            embarrassment: dampen(delta.embarrassment, current.embarrassment.value()),
            curiosity: dampen(delta.curiosity, current.curiosity.value()),
        }
    }
}

impl Default for RuleBasedEmotionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionEngine for RuleBasedEmotionEngine {
    fn evaluate(
        &self,
        current_state: &EmotionState,
        user_input: &str,
        personality: &Personality,
    ) -> EmotionDelta {
        let lower = user_input.to_lowercase();
        let rules = Self::keyword_rules();

        // Accumulate deltas from all matching rules
        let mut accumulated = EmotionDelta::zero();
        let mut match_count = 0u32;

        for rule in &rules {
            let matched = rule.keywords.iter().any(|kw| lower.contains(kw));
            if matched {
                accumulated = accumulated.combine(&rule.base_delta);
                match_count += 1;
            }
        }

        // If no rules matched, apply a small neutral curiosity bump
        if match_count == 0 {
            accumulated = EmotionDelta {
                curiosity: 0.08,
                joy: 0.02,
                ..EmotionDelta::zero()
            };
        } else if match_count > 1 {
            // If multiple rules matched, scale down to avoid overreaction
            let damping = 1.0 / (match_count as f32).sqrt();
            accumulated = accumulated.scale(damping);
        }

        // Modulate by personality
        let modulated = Self::modulate_by_personality(&accumulated, personality);

        // Smooth based on current state
        Self::smooth(&modulated, current_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_core::state::emotion::EmotionScore;

    fn test_engine() -> RuleBasedEmotionEngine {
        RuleBasedEmotionEngine::new()
    }

    fn test_state() -> EmotionState {
        EmotionState::neutral()
    }

    fn test_personality() -> Personality {
        Personality::baseline_aria()
    }

    #[test]
    fn test_happy_input_increases_joy() {
        let engine = test_engine();
        let state = test_state();
        let personality = test_personality();

        let delta = engine.evaluate(&state, "Hôm nay mình rất vui!", &personality);
        assert!(delta.joy > 0.0, "Joy delta should be positive for happy input");
    }

    #[test]
    fn test_sad_input_increases_sadness() {
        let engine = test_engine();
        let state = test_state();
        let personality = test_personality();

        let delta = engine.evaluate(&state, "Mình đang buồn quá", &personality);
        assert!(delta.sadness > 0.0, "Sadness delta should be positive");
        assert!(delta.affection > 0.0, "Empathic affection should increase");
    }

    #[test]
    fn test_angry_input() {
        let engine = test_engine();
        let state = test_state();
        let personality = test_personality();

        let delta = engine.evaluate(&state, "Mình rất tức giận!", &personality);
        assert!(delta.anger > 0.0, "Anger should increase");
    }

    #[test]
    fn test_curious_input() {
        let engine = test_engine();
        let state = test_state();
        let personality = test_personality();

        let delta = engine.evaluate(&state, "Tại sao bầu trời lại xanh?", &personality);
        assert!(delta.curiosity > 0.0, "Curiosity should increase for questions");
    }

    #[test]
    fn test_greeting_input() {
        let engine = test_engine();
        let state = test_state();
        let personality = test_personality();

        let delta = engine.evaluate(&state, "Chào Aria!", &personality);
        assert!(delta.joy > 0.0, "Joy should increase for greetings");
        assert!(delta.curiosity > 0.0, "Curiosity should increase for greetings");
    }

    #[test]
    fn test_neutral_input_still_produces_delta() {
        let engine = test_engine();
        let state = test_state();
        let personality = test_personality();

        let delta = engine.evaluate(&state, "abc xyz 123", &personality);
        assert!(!delta.is_zero(), "Even unrecognized input should produce some delta");
        assert!(delta.curiosity > 0.0, "Default should bump curiosity");
    }

    #[test]
    fn test_high_empathy_amplifies_affection() {
        let engine = test_engine();
        let state = test_state();
        let personality = test_personality();

        let delta = engine.evaluate(&state, "Mình nhớ bạn quá", &personality);
        // Aria has high empathy (0.88), so affection should be amplified
        assert!(delta.affection > 0.15, "High empathy should amplify affection delta");
    }

    #[test]
    fn test_smoothing_diminishes_already_high_emotions() {
        let engine = test_engine();
        let personality = test_personality();

        // State with already high joy
        let mut high_joy_state = test_state();
        high_joy_state.joy = EmotionScore::clamped(0.95);

        // State with low joy
        let low_joy_state = test_state();

        let delta_high = engine.evaluate(&high_joy_state, "Tuyệt vời!", &personality);
        let delta_low = engine.evaluate(&low_joy_state, "Tuyệt vời!", &personality);

        assert!(
            delta_high.joy < delta_low.joy,
            "Already-high joy should have diminished increase: high={} low={}",
            delta_high.joy,
            delta_low.joy
        );
    }
}
