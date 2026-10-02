use super::tts::VoiceModulation;
use vc_core::state::emotion::{EmotionAxis, EmotionState};

/// Modulates vocal synthesis parameters based on character real-time emotional state.
///
/// Implemented according to Skill 11 (Personality), Skill 13 (State Engineering),
/// and Skill 22 (Runtime Engineering):
/// Voice pitch, tempo, and acoustic energy dynamically reflect the character's internal
/// emotional landscape (Russell's Circumplex Model: Valence and Arousal).
pub struct EmotionAwareVoiceModulator {
    /// Baseline base pitch modifier (default: 1.0).
    pub base_pitch: f32,
    /// Baseline speech rate modifier (default: 1.0).
    pub base_speed: f32,
    /// Baseline acoustic energy modifier (default: 1.0).
    pub base_energy: f32,
}

impl EmotionAwareVoiceModulator {
    pub fn new() -> Self {
        Self {
            base_pitch: 1.0,
            base_speed: 1.0,
            base_energy: 1.0,
        }
    }

    /// Compute dynamic `VoiceModulation` parameters from an `EmotionState`.
    pub fn modulate(&self, emotion: &EmotionState) -> VoiceModulation {
        let (dominant_axis, dominant_score) = emotion.dominant_emotion();
        let valence = emotion.valence(); // [-1.0, 1.0]
        let arousal = emotion.arousal(); // [0.0, 1.0]

        let joy = emotion.joy.value();
        let sadness = emotion.sadness.value();
        let anger = emotion.anger.value();
        let surprise = emotion.surprise.value();
        let affection = emotion.affection.value();
        let fear = emotion.fear.value();

        // 1. Calculate Pitch:
        // Joy, surprise, high valence raise pitch; sadness lowers pitch.
        let pitch = self.base_pitch + (joy * 0.15) + (surprise * 0.20) + (valence * 0.05)
            - (sadness * 0.18);

        // 2. Calculate Speed / Tempo:
        // High arousal, anger, and panic accelerate speech; sadness and contemplation slow it down.
        let mut speed = self.base_speed + (arousal * 0.12) + (anger * 0.15) + (joy * 0.08)
            - (sadness * 0.16)
            - (emotion.embarrassment.value() * 0.06);

        // 3. Calculate Energy:
        // Arousal and dominant intensity elevate energy; sorrow and tenderness soften it.
        let mut energy = self.base_energy + (arousal * 0.20) - (sadness * 0.22);
        if dominant_axis == EmotionAxis::Anger {
            energy += dominant_score.value() * 0.15;
        }

        // 4. Determine Whisper Style:
        // Soft whisper when intimate/calm (high affection, low arousal) or cautious/fearful.
        let is_intimate = affection > 0.70 && arousal < 0.35;
        let is_hushed = fear > 0.75;
        let whisper_effect = is_intimate || is_hushed;

        if whisper_effect {
            energy *= 0.80;
            speed *= 0.95;
        }

        VoiceModulation {
            pitch_modifier: pitch.clamp(0.75, 1.40),
            speed_modifier: speed.clamp(0.75, 1.35),
            energy_level: energy.clamp(0.65, 1.40),
            whisper_effect,
        }
    }
}

impl Default for EmotionAwareVoiceModulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_core::state::emotion::EmotionScore;

    fn make_emotion(
        joy: f32,
        sadness: f32,
        anger: f32,
        fear: f32,
        surprise: f32,
        affection: f32,
        embarrassment: f32,
        curiosity: f32,
    ) -> EmotionState {
        EmotionState {
            joy: EmotionScore::clamped(joy),
            sadness: EmotionScore::clamped(sadness),
            anger: EmotionScore::clamped(anger),
            fear: EmotionScore::clamped(fear),
            surprise: EmotionScore::clamped(surprise),
            affection: EmotionScore::clamped(affection),
            embarrassment: EmotionScore::clamped(embarrassment),
            curiosity: EmotionScore::clamped(curiosity),
            last_updated: 1000,
        }
    }

    #[test]
    fn test_neutral_modulation() {
        let modulator = EmotionAwareVoiceModulator::default();
        let neutral = EmotionState::neutral();
        let modulation = modulator.modulate(&neutral);

        // Neutral emotion produces baseline modulation
        assert!((modulation.pitch_modifier - 1.0).abs() < 0.10);
        assert!((modulation.speed_modifier - 1.0).abs() < 0.10);
        assert!(!modulation.whisper_effect);
    }

    #[test]
    fn test_joyful_excited_modulation() {
        let modulator = EmotionAwareVoiceModulator::default();
        let joyful = make_emotion(0.9, 0.0, 0.0, 0.0, 0.6, 0.5, 0.0, 0.8);
        let mod_joy = modulator.modulate(&joyful);

        // Joyful state must have elevated pitch and faster pace
        assert!(mod_joy.pitch_modifier > 1.10);
        assert!(mod_joy.speed_modifier > 1.05);
        assert!(mod_joy.energy_level > 1.05);
    }

    #[test]
    fn test_sad_melancholic_modulation() {
        let modulator = EmotionAwareVoiceModulator::default();
        let sad = make_emotion(0.0, 0.9, 0.0, 0.0, 0.0, 0.2, 0.0, 0.1);
        let mod_sad = modulator.modulate(&sad);

        // Sad state must have reduced pitch, slower tempo, and lower energy
        assert!(mod_sad.pitch_modifier < 0.95);
        assert!(mod_sad.speed_modifier < 0.95);
        assert!(mod_sad.energy_level < 0.90);
    }

    #[test]
    fn test_intimate_whisper_modulation() {
        let modulator = EmotionAwareVoiceModulator::default();
        let intimate = make_emotion(0.3, 0.0, 0.0, 0.0, 0.0, 0.85, 0.1, 0.2);
        let mod_intimate = modulator.modulate(&intimate);

        assert!(mod_intimate.whisper_effect);
        assert!(mod_intimate.energy_level < 1.0);
    }
}
