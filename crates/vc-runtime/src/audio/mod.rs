pub mod chunker;
pub mod cpal_input;
pub mod input;
pub mod modulator;
pub mod sapi_tts;
pub mod streaming_pipeline;
pub mod tts;
pub mod whisper_transcriber;

pub use chunker::{calculate_rms_volume_slices, clean_narrative_text, TextStreamChunker};
pub use cpal_input::CpalAudioInputProvider;
pub use input::{AudioInputProvider, InputAudioMode, MockAudioInputProvider, VadConfig, VadStatus};
pub use modulator::EmotionAwareVoiceModulator;
pub use sapi_tts::WindowsSapiTtsProvider;
pub use streaming_pipeline::{OrderedTtsQueue, SynthesizedSentenceAudio};
pub use tts::{AudioOutput, MockTtsProvider, TtsProvider, VisemeCue, VoiceModulation};
pub use whisper_transcriber::{WhisperCpuTranscriber, WhisperResult};
