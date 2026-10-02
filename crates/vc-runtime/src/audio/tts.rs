use crate::error::RuntimeError;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// Dynamic voice modulation parameters derived from character emotional state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceModulation {
    /// Pitch scale multiplier (e.g., 1.0 = baseline, 1.15 = joyful, 0.88 = sad).
    pub pitch_modifier: f32,
    /// Speech tempo scale multiplier (e.g., 1.0 = normal, 1.2 = excited/hurried, 0.9 = solemn).
    pub speed_modifier: f32,
    /// Energy / volume intensity (1.0 = normal, 1.3 = shouting/energetic, 0.7 = intimate).
    pub energy_level: f32,
    /// Subtle breathy whisper style for intimate or late-night conversations.
    pub whisper_effect: bool,
}

impl Default for VoiceModulation {
    fn default() -> Self {
        Self {
            pitch_modifier: 1.0,
            speed_modifier: 1.0,
            energy_level: 1.0,
            whisper_effect: false,
        }
    }
}

/// Timed viseme marker used for driving Live2D / 3D VRM facial mouth lip-sync.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisemeCue {
    /// Viseme phonetic category: "sil", "a", "i", "u", "e", "o", "f_v", "l", etc.
    pub viseme: String,
    /// Offset in milliseconds from audio start.
    pub start_ms: u32,
    /// Duration of the viseme shape in milliseconds.
    pub duration_ms: u32,
    /// Normalized mouth open ratio [0.0, 1.0].
    pub mouth_open: f32,
}

/// Complete synthesized audio artifact containing waveform data and lip-sync metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOutput {
    /// Raw PCM or WAV encoded audio bytes.
    pub audio_bytes: Vec<u8>,
    /// Sample rate in Hz (e.g. 22050, 24000, 48000).
    pub sample_rate: u32,
    /// Estimated total playback duration in milliseconds.
    pub duration_ms: u32,
    /// Synchronized viseme timeline for avatar lip-sync rendering.
    pub visemes: Vec<VisemeCue>,
}

/// Abstract contract for Text-to-Speech (TTS) voice generation.
pub trait TtsProvider: Send + Sync {
    /// Synthesize speech audio with emotional modulation.
    fn synthesize(
        &self,
        text: &str,
        modulation: &VoiceModulation,
    ) -> Result<AudioOutput, RuntimeError>;

    /// Native audio sampling rate supported by the underlying voice model.
    fn sample_rate(&self) -> u32;

    /// Provider identifier (e.g., "piper-tts-cpu", "kokoro-onnx", "mock-tts").
    fn provider_name(&self) -> &str;
}

/// Deterministic in-memory mock TTS provider for testing audio pipelines and lip-sync cues.
pub struct MockTtsProvider {
    sample_rate: u32,
    last_synthesized_text: RwLock<Option<String>>,
    last_modulation: RwLock<Option<VoiceModulation>>,
}

impl MockTtsProvider {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            last_synthesized_text: RwLock::new(None),
            last_modulation: RwLock::new(None),
        }
    }

    /// Retrieve the last text sent to `synthesize()`.
    pub fn last_text(&self) -> Option<String> {
        self.last_synthesized_text.read().unwrap().clone()
    }

    /// Retrieve the last modulation parameters used.
    pub fn last_modulation(&self) -> Option<VoiceModulation> {
        self.last_modulation.read().unwrap().clone()
    }
}

impl Default for MockTtsProvider {
    fn default() -> Self {
        Self::new(24000)
    }
}

impl TtsProvider for MockTtsProvider {
    fn synthesize(
        &self,
        text: &str,
        modulation: &VoiceModulation,
    ) -> Result<AudioOutput, RuntimeError> {
        if text.trim().is_empty() {
            return Err(RuntimeError::TtsError(
                "Cannot synthesize empty text".into(),
            ));
        }

        *self.last_synthesized_text.write().unwrap() = Some(text.to_string());
        *self.last_modulation.write().unwrap() = Some(modulation.clone());

        // Approximate duration: ~60ms per character scaled inversely by speech speed
        let base_duration_ms = (text.chars().count() as f32 * 60.0) as u32;
        let effective_speed = modulation.speed_modifier.clamp(0.5, 2.0);
        let duration_ms = ((base_duration_ms as f32) / effective_speed).max(100.0) as u32;

        // Generate synthetic Viseme timeline based on vowels in the text
        let mut visemes = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_time_ms = duration_ms / (words.len() as u32).max(1);

        for (i, word) in words.iter().enumerate() {
            let start = i as u32 * word_time_ms;
            let duration = (word_time_ms as f32 * 0.8) as u32;

            // Pick dominant vowel or default to "a"
            let viseme_name = if word.contains('o') || word.contains('ô') || word.contains('ơ') {
                "o"
            } else if word.contains('u') || word.contains('ư') {
                "u"
            } else if word.contains('i') || word.contains('y') {
                "i"
            } else if word.contains('e') || word.contains('ê') {
                "e"
            } else {
                "a"
            };

            let mouth_open = match viseme_name {
                "a" => 0.9,
                "o" => 0.7,
                "e" => 0.6,
                "u" => 0.5,
                "i" => 0.4,
                _ => 0.3,
            };

            visemes.push(VisemeCue {
                viseme: viseme_name.to_string(),
                start_ms: start,
                duration_ms: duration,
                mouth_open,
            });
        }

        // Generate dummy audio bytes representing synthetic PCM samples
        let num_samples = (self.sample_rate as f32 * (duration_ms as f32 / 1000.0)) as usize;
        let audio_bytes = vec![0u8; num_samples * 2]; // 16-bit mono

        Ok(AudioOutput {
            audio_bytes,
            sample_rate: self.sample_rate,
            duration_ms,
            visemes,
        })
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn provider_name(&self) -> &str {
        "mock-tts"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_tts_synthesis_and_viseme_generation() {
        let provider = MockTtsProvider::default();
        let modulation = VoiceModulation {
            pitch_modifier: 1.1,
            speed_modifier: 1.0,
            energy_level: 1.2,
            whisper_effect: false,
        };

        let output = provider
            .synthesize("Aria thích chơi game", &modulation)
            .unwrap();

        assert_eq!(provider.last_text(), Some("Aria thích chơi game".into()));
        assert_eq!(provider.last_modulation(), Some(modulation));

        assert!(output.duration_ms > 0);
        assert!(!output.audio_bytes.is_empty());
        assert_eq!(output.sample_rate, 24000);

        // 4 words -> 4 viseme cues generated for lip-sync
        assert_eq!(output.visemes.len(), 4);
        assert!(output.visemes[0].mouth_open > 0.0);
    }

    #[test]
    fn test_empty_text_error() {
        let provider = MockTtsProvider::default();
        let res = provider.synthesize("   ", &VoiceModulation::default());
        assert!(res.is_err());
    }
}
