//! Native Windows SAPI / OneCore Speech Synthesizer Provider.
//!
//! Provides 100% offline, local, zero-VRAM Text-to-Speech synthesis on Windows,
//! utilizing installed Microsoft Desktop voices (e.g. Microsoft Zira Desktop / Microsoft David).
//! Produces standard 22050 Hz 16-bit mono WAV audio with synchronized mouth viseme cues.

use crate::audio::chunker::calculate_rms_volume_slices;
use crate::audio::tts::{AudioOutput, TtsProvider, VisemeCue, VoiceModulation};
use crate::error::RuntimeError;
use std::fs;
use std::process::Command;

/// Real Windows SAPI Text-to-Speech synthesis provider.
pub struct WindowsSapiTtsProvider {
    voice_name: String,
    sample_rate: u32,
}

impl WindowsSapiTtsProvider {
    /// Create a new SAPI TTS provider with a specific voice name (e.g. "Microsoft Zira Desktop").
    pub fn new(voice_name: impl Into<String>) -> Self {
        Self {
            voice_name: voice_name.into(),
            sample_rate: 22050,
        }
    }

    /// Create default female voice tuned for virtual character personas.
    pub fn default_female() -> Self {
        Self::new("Microsoft Zira Desktop")
    }

    /// Create default male voice.
    pub fn default_male() -> Self {
        Self::new("Microsoft David Desktop")
    }

    /// List all installed SAPI voices on the host Windows machine.
    pub fn list_installed_voices() -> Result<Vec<String>, RuntimeError> {
        let script = r#"
Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
$synth.GetInstalledVoices() | ForEach-Object { $_.VoiceInfo.Name }
"#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .output()
            .map_err(|e| RuntimeError::Internal(format!("Failed to execute PowerShell: {}", e)))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(RuntimeError::Internal(format!(
                "SAPI query failed: {}",
                err
            )));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let voices: Vec<String> = text
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        Ok(voices)
    }
}

impl Default for WindowsSapiTtsProvider {
    fn default() -> Self {
        Self::default_female()
    }
}

impl TtsProvider for WindowsSapiTtsProvider {
    fn synthesize(
        &self,
        text: &str,
        modulation: &VoiceModulation,
    ) -> Result<AudioOutput, RuntimeError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(RuntimeError::TtsError(
                "Text for TTS synthesis cannot be empty".into(),
            ));
        }

        // Map speed modifier (0.5 to 2.0) to SAPI Rate (-10 to 10)
        // 1.0 -> 0; 1.5 -> +4; 0.8 -> -2
        let rate_val: i32 =
            (((modulation.speed_modifier - 1.0) * 8.0).round() as i32).clamp(-10, 10);

        // Map energy/volume (0.0 to 1.5) to SAPI Volume (0 to 100)
        let vol_val: i32 = ((modulation.energy_level * 100.0).round() as i32).clamp(10, 100);

        // Temporary unique output file in system temp dir
        let temp_wav = std::env::temp_dir().join(format!("vc_tts_{}.wav", uuid::Uuid::new_v4()));
        let temp_wav_str = temp_wav.to_string_lossy().replace('\\', "/");

        // Escape single quotes for PowerShell literal string
        let escaped_text = trimmed.replace('\'', "''");
        let escaped_voice = self.voice_name.replace('\'', "''");

        let script = format!(
            r#"
Add-Type -AssemblyName System.Speech
$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
try {{
    $synth.SelectVoice('{}')
}} catch {{}}
$synth.Rate = {}
$synth.Volume = {}
$synth.SetOutputToWaveFile('{}')
$synth.Speak('{}')
$synth.Dispose()
"#,
            escaped_voice, rate_val, vol_val, temp_wav_str, escaped_text
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
            .map_err(|e| {
                RuntimeError::Internal(format!("Failed to execute TTS synthesis process: {}", e))
            })?;

        if !output.status.success() {
            let _ = fs::remove_file(&temp_wav);
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(RuntimeError::Internal(format!(
                "TTS synthesis failed: {}",
                err
            )));
        }

        if !temp_wav.exists() {
            return Err(RuntimeError::Internal(
                "TTS output file was not generated".into(),
            ));
        }

        let audio_bytes = fs::read(&temp_wav).map_err(|e| {
            let _ = fs::remove_file(&temp_wav);
            RuntimeError::Internal(format!("Failed to read synthesized audio WAV file: {}", e))
        })?;

        // Clean up temporary file
        let _ = fs::remove_file(&temp_wav);

        if audio_bytes.len() < 44 {
            return Err(RuntimeError::Internal(
                "Synthesized WAV file is corrupt or truncated".into(),
            ));
        }

        // Parse WAV sample rate from header bytes [24..28]
        let sample_rate = u32::from_le_bytes([
            audio_bytes[24],
            audio_bytes[25],
            audio_bytes[26],
            audio_bytes[27],
        ]);

        // Calculate PCM duration from data chunk size
        let pcm_data = &audio_bytes[44..];
        let duration_ms = if sample_rate > 0 {
            // 16-bit mono = 2 bytes per sample
            let num_samples = pcm_data.len() / 2;
            ((num_samples as f64 / sample_rate as f64) * 1000.0) as u32
        } else {
            0
        };

        // Open-LLM-VTuber pattern: compute 20ms RMS volume slices for real-time lip movement
        let rms_slices = calculate_rms_volume_slices(pcm_data, sample_rate);
        let mut visemes = Vec::with_capacity(rms_slices.len());

        for (i, &rms) in rms_slices.iter().enumerate() {
            let start_ms = (i as u32) * 20;
            // Map RMS energy to mouth open ratio
            let mouth_open = (rms * 2.5).min(1.0);
            let viseme_name = if mouth_open > 0.4 {
                "a".to_string()
            } else if mouth_open > 0.15 {
                "o".to_string()
            } else {
                "sil".to_string()
            };

            visemes.push(VisemeCue {
                viseme: viseme_name,
                start_ms,
                duration_ms: 20,
                mouth_open,
            });
        }

        Ok(AudioOutput {
            audio_bytes,
            sample_rate,
            duration_ms,
            visemes,
        })
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn provider_name(&self) -> &str {
        "windows-sapi"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_installed_voices_finds_windows_voices() {
        let voices =
            WindowsSapiTtsProvider::list_installed_voices().expect("Query installed SAPI voices");
        println!("Installed SAPI Voices: {:?}", voices);
        assert!(
            !voices.is_empty(),
            "Windows system should have at least one desktop voice"
        );
        assert!(voices
            .iter()
            .any(|v| v.contains("David") || v.contains("Zira")));
    }

    #[test]
    fn test_real_windows_sapi_synthesis() {
        let provider = WindowsSapiTtsProvider::default_female();
        let modulation = VoiceModulation::default();

        let output = provider
            .synthesize("Xin chào! Tôi là Aria.", &modulation)
            .expect("Real SAPI speech synthesis");

        assert!(!output.audio_bytes.is_empty());
        assert!(output.audio_bytes.starts_with(b"RIFF"));
        assert_eq!(output.sample_rate, 22050);
        assert!(
            output.duration_ms > 500,
            "Audio should have valid duration (>500ms)"
        );
        assert!(
            !output.visemes.is_empty(),
            "Should generate synchronized visemes"
        );
        println!(
            "Synthesized: {} bytes, {} ms duration, {} viseme cues",
            output.audio_bytes.len(),
            output.duration_ms,
            output.visemes.len()
        );
    }
}
