//! Real Windows WASAPI / Cross-platform Microphone Audio Input Provider.
//!
//! Uses `cpal` to capture live microphone PCM audio from the host OS,
//! running real-time Voice Activity Detection (VAD) on CPU without GPU overhead.
//! Audio capture streams are isolated in a dedicated background worker thread
//! to guarantee COM threading safety (WASAPI) and satisfy `Send + Sync` bounds.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex, RwLock};

use crate::audio::input::{AudioInputProvider, InputAudioMode, VadConfig, VadStatus};
use crate::error::RuntimeError;

enum AudioCommand {
    Start,
    Stop,
    Quit,
}

/// Real audio input capture provider backed by system sound drivers (WASAPI on Windows).
pub struct CpalAudioInputProvider {
    config: VadConfig,
    mode: RwLock<InputAudioMode>,
    is_listening: Arc<RwLock<bool>>,
    transcription_queue: Arc<Mutex<VecDeque<String>>>,
    cmd_tx: mpsc::Sender<AudioCommand>,
    accumulated_pcm: Arc<Mutex<Vec<f32>>>,
    speech_active: Arc<Mutex<bool>>,
    silence_samples_count: Arc<Mutex<usize>>,
}

impl CpalAudioInputProvider {
    /// Create a new native audio input provider with the given VAD parameters.
    pub fn new(config: VadConfig) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let is_listening = Arc::new(RwLock::new(false));
        let accumulated_pcm = Arc::new(Mutex::new(Vec::new()));
        let speech_active = Arc::new(Mutex::new(false));
        let silence_samples_count = Arc::new(Mutex::new(0));

        let is_listening_worker = Arc::clone(&is_listening);
        let pcm_worker = Arc::clone(&accumulated_pcm);
        let speech_worker = Arc::clone(&speech_active);
        let silence_worker = Arc::clone(&silence_samples_count);
        let cfg_clone = config.clone();

        std::thread::Builder::new()
            .name("vc-cpal-audio-worker".into())
            .spawn(move || {
                let mut current_stream: Option<cpal::Stream> = None;
                while let Ok(cmd) = cmd_rx.recv() {
                    match cmd {
                        AudioCommand::Start => {
                            if current_stream.is_none() {
                                let host = cpal::default_host();
                                if let Some(device) = host.default_input_device() {
                                    if let Ok(default_config) = device.default_input_config() {
                                        let channels = default_config.channels();
                                        let sample_rate = default_config.sample_rate().0;
                                        let stream_config: StreamConfig =
                                            default_config.clone().into();
                                        let energy_thresh = cfg_clone.energy_threshold;
                                        let silence_timeout_samples =
                                            (cfg_clone.silence_timeout_ms as f32 / 1000.0
                                                * sample_rate as f32)
                                                as usize;

                                        let pcm_cb = Arc::clone(&pcm_worker);
                                        let sp_cb = Arc::clone(&speech_worker);
                                        let sil_cb = Arc::clone(&silence_worker);

                                        let stream_res = match default_config.sample_format() {
                                            SampleFormat::F32 => device.build_input_stream(
                                                &stream_config,
                                                move |data: &[f32], _| {
                                                    let mono_samples: Vec<f32> = if channels > 1 {
                                                        data.chunks(channels as usize)
                                                            .map(|frame| {
                                                                frame.iter().sum::<f32>()
                                                                    / channels as f32
                                                            })
                                                            .collect()
                                                    } else {
                                                        data.to_vec()
                                                    };
                                                    let sum_sq: f32 =
                                                        mono_samples.iter().map(|&s| s * s).sum();
                                                    let rms = (sum_sq
                                                        / mono_samples.len().max(1) as f32)
                                                        .sqrt();

                                                    let mut active_guard = sp_cb.lock().unwrap();
                                                    let mut pcm_guard = pcm_cb.lock().unwrap();
                                                    let mut silence_guard = sil_cb.lock().unwrap();

                                                    if rms >= energy_thresh {
                                                        *active_guard = true;
                                                        *silence_guard = 0;
                                                        pcm_guard.extend_from_slice(&mono_samples);
                                                    } else if *active_guard {
                                                        *silence_guard += mono_samples.len();
                                                        pcm_guard.extend_from_slice(&mono_samples);
                                                        if *silence_guard >= silence_timeout_samples
                                                        {
                                                            *active_guard = false;
                                                            *silence_guard = 0;
                                                        }
                                                    }
                                                },
                                                |err| eprintln!("Audio stream error: {}", err),
                                                None,
                                            ),
                                            SampleFormat::I16 => device.build_input_stream(
                                                &stream_config,
                                                move |data: &[i16], _| {
                                                    let mono_samples: Vec<f32> = if channels > 1 {
                                                        data.chunks(channels as usize)
                                                            .map(|frame| {
                                                                let sum: f32 = frame
                                                                    .iter()
                                                                    .map(|&s| s as f32 / 32768.0)
                                                                    .sum();
                                                                sum / channels as f32
                                                            })
                                                            .collect()
                                                    } else {
                                                        data.iter()
                                                            .map(|&s| s as f32 / 32768.0)
                                                            .collect()
                                                    };
                                                    let sum_sq: f32 =
                                                        mono_samples.iter().map(|&s| s * s).sum();
                                                    let rms = (sum_sq
                                                        / mono_samples.len().max(1) as f32)
                                                        .sqrt();

                                                    let mut active_guard = sp_cb.lock().unwrap();
                                                    let mut pcm_guard = pcm_cb.lock().unwrap();
                                                    let mut silence_guard = sil_cb.lock().unwrap();

                                                    if rms >= energy_thresh {
                                                        *active_guard = true;
                                                        *silence_guard = 0;
                                                        pcm_guard.extend_from_slice(&mono_samples);
                                                    } else if *active_guard {
                                                        *silence_guard += mono_samples.len();
                                                        pcm_guard.extend_from_slice(&mono_samples);
                                                        if *silence_guard >= silence_timeout_samples
                                                        {
                                                            *active_guard = false;
                                                            *silence_guard = 0;
                                                        }
                                                    }
                                                },
                                                |err| eprintln!("Audio stream error: {}", err),
                                                None,
                                            ),
                                            _ => Err(cpal::BuildStreamError::DeviceNotAvailable),
                                        };

                                        if let Ok(s) = stream_res {
                                            if s.play().is_ok() {
                                                current_stream = Some(s);
                                                *is_listening_worker.write().unwrap() = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        AudioCommand::Stop => {
                            current_stream = None;
                            *is_listening_worker.write().unwrap() = false;
                        }
                        AudioCommand::Quit => {
                            *is_listening_worker.write().unwrap() = false;
                            break;
                        }
                    }
                }
            })
            .expect("Failed to spawn audio worker thread");

        Self {
            config,
            mode: RwLock::new(InputAudioMode::VoiceActivity),
            is_listening,
            transcription_queue: Arc::new(Mutex::new(VecDeque::new())),
            cmd_tx,
            accumulated_pcm,
            speech_active,
            silence_samples_count,
        }
    }

    /// List all physical audio input devices available on the host system.
    pub fn list_input_devices() -> Result<Vec<String>, RuntimeError> {
        let host = cpal::default_host();
        let devices = host.input_devices().map_err(|e| {
            RuntimeError::Internal(format!("Failed to query audio input devices: {}", e))
        })?;

        let mut device_names = Vec::new();
        for dev in devices {
            if let Ok(name) = dev.name() {
                device_names.push(name);
            }
        }
        Ok(device_names)
    }

    /// Enqueue a transcription result manually (or from an offline Whisper worker).
    pub fn push_transcription(&self, text: impl Into<String>) {
        let mut queue = self.transcription_queue.lock().unwrap();
        queue.push_back(text.into());
    }

    /// Take the accumulated PCM audio buffer if an utterance just completed.
    pub fn drain_accumulated_pcm(&self) -> Vec<f32> {
        let mut buf = self.accumulated_pcm.lock().unwrap();
        std::mem::take(&mut *buf)
    }
}

impl Drop for CpalAudioInputProvider {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(AudioCommand::Quit);
    }
}

impl AudioInputProvider for CpalAudioInputProvider {
    fn start_listening(&self) -> Result<(), RuntimeError> {
        self.cmd_tx.send(AudioCommand::Start).map_err(|e| {
            RuntimeError::Internal(format!(
                "Failed to send Start command to audio worker: {}",
                e
            ))
        })
    }

    fn stop_listening(&self) -> Result<(), RuntimeError> {
        self.cmd_tx.send(AudioCommand::Stop).map_err(|e| {
            RuntimeError::Internal(format!(
                "Failed to send Stop command to audio worker: {}",
                e
            ))
        })
    }

    fn is_listening(&self) -> bool {
        *self.is_listening.read().unwrap()
    }

    fn poll_transcription(&self) -> Option<String> {
        let mut queue = self.transcription_queue.lock().unwrap();
        queue.pop_front()
    }

    fn process_audio_chunk(&self, samples: &[f32]) -> Result<VadStatus, RuntimeError> {
        if samples.is_empty() {
            return Ok(VadStatus::Silence);
        }

        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();

        let mut active_guard = self.speech_active.lock().unwrap();
        let mut pcm_guard = self.accumulated_pcm.lock().unwrap();
        let mut silence_guard = self.silence_samples_count.lock().unwrap();

        if rms >= self.config.energy_threshold {
            *active_guard = true;
            *silence_guard = 0;
            pcm_guard.extend_from_slice(samples);
            Ok(VadStatus::SpeechActive)
        } else if *active_guard {
            *silence_guard += samples.len();
            pcm_guard.extend_from_slice(samples);

            let silence_timeout_samples = (self.config.silence_timeout_ms as f32 / 1000.0
                * self.config.sample_rate as f32)
                as usize;
            if *silence_guard >= silence_timeout_samples {
                *active_guard = false;
                *silence_guard = 0;
                let finished_buf = std::mem::take(&mut *pcm_guard);
                Ok(VadStatus::SpeechFinished(finished_buf))
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
    fn test_list_input_devices_does_not_panic() {
        let devices = CpalAudioInputProvider::list_input_devices();
        println!("Detected Input Devices: {:?}", devices);
        assert!(devices.is_ok());
    }

    #[test]
    fn test_manual_audio_chunk_vad_detection() {
        let config = VadConfig {
            energy_threshold: 0.1,
            min_speech_duration_ms: 10,
            silence_timeout_ms: 20,
            sample_rate: 1000,
        };
        let provider = CpalAudioInputProvider::new(config);

        // 1. Silent chunk
        let silent = vec![0.0f32; 100];
        let status = provider.process_audio_chunk(&silent).unwrap();
        assert_eq!(status, VadStatus::Silence);

        // 2. Speech chunk (high amplitude)
        let speech = vec![0.5f32; 100];
        let status = provider.process_audio_chunk(&speech).unwrap();
        assert_eq!(status, VadStatus::SpeechActive);

        // 3. Silence chunk after speech, triggering SpeechFinished after timeout
        let silence_after = vec![0.0f32; 30]; // 30 samples > 20 sample timeout
        let status = provider.process_audio_chunk(&silence_after).unwrap();
        assert!(matches!(status, VadStatus::SpeechFinished(_)));
    }
}
