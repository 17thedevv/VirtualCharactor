use crate::error::RuntimeError;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::RwLock;

/// Operating mode for microphone and auditory sensing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputAudioMode {
    /// Microphone only captures audio while user presses a designated key or hotkey.
    PushToTalk,
    /// Automated voice segmentation using Voice Activity Detection (VAD).
    VoiceActivity,
    /// Passive low-power listening for wake-word (e.g., "Aria ơi!").
    WakeWord,
}

/// Real-time detection state returned by Voice Activity Detection.
#[derive(Debug, Clone, PartialEq)]
pub enum VadStatus {
    /// Background silence or ambient noise below threshold.
    Silence,
    /// Human speech energy actively detected in current chunk.
    SpeechActive,
    /// User finished speaking (silence gap detected); returns accumulated PCM audio buffer.
    SpeechFinished(Vec<f32>),
}

/// Configuration parameters for Voice Activity Detection and speech segmentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    /// Minimum RMS energy required to consider a frame as speech (0.0 to 1.0).
    pub energy_threshold: f32,
    /// Minimum speech duration in milliseconds before triggering turn.
    pub min_speech_duration_ms: u32,
    /// Duration of silence in milliseconds that marks the end of an utterance.
    pub silence_timeout_ms: u32,
    /// Sampling rate in Hz (default: 16000 Hz standard for Whisper STT).
    pub sample_rate: u32,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            energy_threshold: 0.02,
            min_speech_duration_ms: 250,
            silence_timeout_ms: 600,
            sample_rate: 16000,
        }
    }
}

/// Abstract contract for auditory sensing and Speech-to-Text (STT) ingestion.
pub trait AudioInputProvider: Send + Sync {
    /// Start capturing audio from active input device.
    fn start_listening(&self) -> Result<(), RuntimeError>;

    /// Stop capturing audio.
    fn stop_listening(&self) -> Result<(), RuntimeError>;

    /// Check if audio capture is currently active.
    fn is_listening(&self) -> bool;

    /// Retrieve the next recognized transcript from the speech queue, if ready.
    fn poll_transcription(&self) -> Option<String>;

    /// Stream raw PCM 32-bit float audio samples into the VAD / STT pipeline.
    fn process_audio_chunk(&self, samples: &[f32]) -> Result<VadStatus, RuntimeError>;

    /// Return the active operating mode.
    fn mode(&self) -> InputAudioMode;

    /// Change the active operating mode.
    fn set_mode(&self, mode: InputAudioMode) -> Result<(), RuntimeError>;
}

/// In-memory mock audio provider for deterministic offline testing of speech loops.
pub struct MockAudioInputProvider {
    is_listening: RwLock<bool>,
    mode: RwLock<InputAudioMode>,
    config: VadConfig,
    transcription_queue: RwLock<VecDeque<String>>,
    accumulated_pcm: RwLock<Vec<f32>>,
    speech_active: RwLock<bool>,
    silence_frames: RwLock<u32>,
}

impl MockAudioInputProvider {
    pub fn new(config: VadConfig) -> Self {
        Self {
            is_listening: RwLock::new(false),
            mode: RwLock::new(InputAudioMode::VoiceActivity),
            config,
            transcription_queue: RwLock::new(VecDeque::new()),
            accumulated_pcm: RwLock::new(Vec::new()),
            speech_active: RwLock::new(false),
            silence_frames: RwLock::new(0),
        }
    }

    /// Enqueue a synthetic transcribed user utterance.
    pub fn queue_transcription(&self, text: impl Into<String>) {
        self.transcription_queue
            .write()
            .unwrap()
            .push_back(text.into());
    }

    /// Helper to compute Root Mean Square (RMS) energy of PCM float samples.
    pub fn compute_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_sq / (samples.len() as f32)).sqrt()
    }
}

impl Default for MockAudioInputProvider {
    fn default() -> Self {
        Self::new(VadConfig::default())
    }
}

impl AudioInputProvider for MockAudioInputProvider {
    fn start_listening(&self) -> Result<(), RuntimeError> {
        *self.is_listening.write().unwrap() = true;
        Ok(())
    }

    fn stop_listening(&self) -> Result<(), RuntimeError> {
        *self.is_listening.write().unwrap() = false;
        *self.speech_active.write().unwrap() = false;
        self.accumulated_pcm.write().unwrap().clear();
        Ok(())
    }

    fn is_listening(&self) -> bool {
        *self.is_listening.read().unwrap()
    }

    fn poll_transcription(&self) -> Option<String> {
        self.transcription_queue.write().unwrap().pop_front()
    }

    fn process_audio_chunk(&self, samples: &[f32]) -> Result<VadStatus, RuntimeError> {
        if !self.is_listening() {
            return Ok(VadStatus::Silence);
        }

        let rms = Self::compute_rms(samples);
        let is_speech_energy = rms >= self.config.energy_threshold;

        let mut active = self.speech_active.write().unwrap();
        let mut silence = self.silence_frames.write().unwrap();
        let mut pcm = self.accumulated_pcm.write().unwrap();

        if is_speech_energy {
            *active = true;
            *silence = 0;
            pcm.extend_from_slice(samples);
            Ok(VadStatus::SpeechActive)
        } else if *active {
            // Previously active speech, now silent frame
            *silence += 1;
            pcm.extend_from_slice(samples);

            // Simple frame-based silence check: 3 consecutive silent frames (~600ms) ends speech
            if *silence >= 3 {
                *active = false;
                *silence = 0;
                let finished_buffer = std::mem::take(&mut *pcm);
                Ok(VadStatus::SpeechFinished(finished_buffer))
            } else {
                Ok(VadStatus::SpeechActive)
            }
        } else {
            Ok(VadStatus::Silence)
        }
    }

    fn mode(&self) -> InputAudioMode {
        *self.mode.read().unwrap()
    }

    fn set_mode(&self, mode: InputAudioMode) -> Result<(), RuntimeError> {
        *self.mode.write().unwrap() = mode;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_audio_provider_transcription_queue() {
        let provider = MockAudioInputProvider::default();
        assert!(!provider.is_listening());

        provider.start_listening().unwrap();
        assert!(provider.is_listening());

        assert_eq!(provider.poll_transcription(), None);

        provider.queue_transcription("Xin chào Aria");
        provider.queue_transcription("Hôm nay bạn thế nào?");

        assert_eq!(provider.poll_transcription(), Some("Xin chào Aria".into()));
        assert_eq!(
            provider.poll_transcription(),
            Some("Hôm nay bạn thế nào?".into())
        );
        assert_eq!(provider.poll_transcription(), None);

        provider.stop_listening().unwrap();
        assert!(!provider.is_listening());
    }

    #[test]
    fn test_vad_audio_energy_detection() {
        let provider = MockAudioInputProvider::default();
        provider.start_listening().unwrap();

        // 1. Send silent chunk (RMS = 0.0)
        let silence = vec![0.0f32; 1600]; // 100ms at 16kHz
        assert_eq!(
            provider.process_audio_chunk(&silence).unwrap(),
            VadStatus::Silence
        );

        // 2. Send active speech chunk (RMS = 0.1 > threshold 0.02)
        let speech = vec![0.1f32; 1600];
        assert_eq!(
            provider.process_audio_chunk(&speech).unwrap(),
            VadStatus::SpeechActive
        );

        // 3. Send 3 silent frames to trigger end of speech
        assert_eq!(
            provider.process_audio_chunk(&silence).unwrap(),
            VadStatus::SpeechActive
        ); // Frame 1
        assert_eq!(
            provider.process_audio_chunk(&silence).unwrap(),
            VadStatus::SpeechActive
        ); // Frame 2
        let finished = provider.process_audio_chunk(&silence).unwrap(); // Frame 3

        match finished {
            VadStatus::SpeechFinished(pcm) => {
                // Must have collected speech + 3 silent frames: 1600 * 4 = 6400 samples
                assert_eq!(pcm.len(), 6400);
            }
            other => panic!("Expected SpeechFinished, got {:?}", other),
        }
    }
}
