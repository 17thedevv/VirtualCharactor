//! Offline CPU Speech-to-Text Transcriber using faster-whisper.
//!
//! Executes on host CPU using CTranslate2 int8 quantization.
//! Maintains zero GPU VRAM allocation so LLM / VLM execution is uninterrupted.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::RuntimeError;

/// Structured transcription response from Whisper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperResult {
    pub text: String,
    pub language: String,
    pub probability: f32,
}

#[derive(Deserialize)]
struct RawWhisperResponse {
    success: bool,
    text: Option<String>,
    language: Option<String>,
    probability: Option<f32>,
    error: Option<String>,
}

/// Offline CPU Whisper transcriber.
pub struct WhisperCpuTranscriber {
    model_size: String,
    language: Option<String>,
    script_path: PathBuf,
}

impl WhisperCpuTranscriber {
    /// Create a new CPU transcriber with model size (e.g. "tiny", "base").
    pub fn new(model_size: impl Into<String>) -> Self {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // Root workspace is 2 levels up from crates/vc-runtime
        let script_path = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("scripts").join("whisper_transcribe.py"))
            .unwrap_or_else(|| PathBuf::from("scripts/whisper_transcribe.py"));

        Self {
            model_size: model_size.into(),
            language: None,
            script_path,
        }
    }

    /// Set explicit language hint (e.g. "en", "vi", "ja").
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }

    /// Transcribe an existing audio file on disk (WAV/MP3).
    pub fn transcribe_file(&self, audio_path: &Path) -> Result<WhisperResult, RuntimeError> {
        if !audio_path.exists() {
            return Err(RuntimeError::AudioInputError(format!(
                "Audio file does not exist: {:?}",
                audio_path
            )));
        }

        let mut cmd = Command::new("python");
        cmd.arg(&self.script_path)
            .arg(audio_path)
            .arg("--model")
            .arg(&self.model_size);

        if let Some(ref lang) = self.language {
            cmd.arg("--language").arg(lang);
        }

        let output = cmd.output().map_err(|e| {
            RuntimeError::AudioInputError(format!("Failed to execute python whisper script: {}", e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let resp: RawWhisperResponse = serde_json::from_str(stdout.trim()).map_err(|e| {
            let stderr = String::from_utf8_lossy(&output.stderr);
            RuntimeError::AudioInputError(format!(
                "Failed to parse Whisper JSON output (stderr: {}): {}",
                stderr, e
            ))
        })?;

        if !resp.success {
            let err = resp.error.unwrap_or_else(|| "Unknown Whisper error".into());
            return Err(RuntimeError::AudioInputError(err));
        }

        Ok(WhisperResult {
            text: resp.text.unwrap_or_default(),
            language: resp.language.unwrap_or_default(),
            probability: resp.probability.unwrap_or(0.0),
        })
    }

    /// Transcribe an in-memory PCM 32-bit float buffer by writing a temporary WAV file.
    pub fn transcribe_pcm(
        &self,
        samples: &[f32],
        sample_rate: u32,
    ) -> Result<WhisperResult, RuntimeError> {
        if samples.is_empty() {
            return Ok(WhisperResult {
                text: String::new(),
                language: "unknown".into(),
                probability: 0.0,
            });
        }

        let temp_wav = std::env::temp_dir().join(format!("vc_stt_{}.wav", uuid::Uuid::new_v4()));
        write_pcm_to_wav(&temp_wav, samples, sample_rate).map_err(|e| {
            RuntimeError::AudioInputError(format!("Failed to write temporary WAV: {}", e))
        })?;

        let result = self.transcribe_file(&temp_wav);
        let _ = std::fs::remove_file(&temp_wav);
        result
    }
}

impl Default for WhisperCpuTranscriber {
    fn default() -> Self {
        Self::new("tiny")
    }
}

/// Helper function to write raw 32-bit float mono samples to standard 16-bit PCM WAV.
fn write_pcm_to_wav(path: &Path, samples: &[f32], sample_rate: u32) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    let num_samples = samples.len() as u32;
    let byte_rate = sample_rate * 2; // 1 channel * 16-bit (2 bytes)
    let block_align: u16 = 2;
    let bits_per_sample: u16 = 16;
    let data_chunk_size = num_samples * 2;
    let total_file_size = 36 + data_chunk_size;

    // RIFF Header
    file.write_all(b"RIFF")?;
    file.write_all(&total_file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // fmt subchunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // Subchunk1Size (16 for PCM)
    file.write_all(&1u16.to_le_bytes())?; // AudioFormat (1 = PCM)
    file.write_all(&1u16.to_le_bytes())?; // NumChannels (1 = Mono)
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;

    // data subchunk
    file.write_all(b"data")?;
    file.write_all(&data_chunk_size.to_le_bytes())?;

    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let sample_i16 = (clamped * 32767.0) as i16;
        file.write_all(&sample_i16.to_le_bytes())?;
    }

    file.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whisper_transcription_from_test_wav() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().unwrap().parent().unwrap();
        let test_wav = root.join("scratch").join("test_tts.wav");

        if test_wav.exists() {
            let transcriber = WhisperCpuTranscriber::default();
            let res = transcriber
                .transcribe_file(&test_wav)
                .expect("Transcription should succeed");
            println!("Transcription Result: {:?}", res);
            assert!(
                res.text.to_lowercase().contains("aria")
                    || res.text.to_lowercase().contains("hello")
            );
            assert_eq!(res.language, "en");
            assert!(res.probability > 0.8);
        }
    }
}
