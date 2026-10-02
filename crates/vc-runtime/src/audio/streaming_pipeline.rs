//! Ordered Streaming TTS Pipeline (P0-D).
//!
//! Inspired by the Ordered Parallel TTS Queue architecture from Open-LLM-VTuber:
//! Synthesizes sentence chunks in parallel to achieve sub-500ms Time-to-First-Audio (TTFA),
//! while using a reordering buffer to guarantee that audio chunks are emitted to speakers
//! in strictly sequential sentence order.
//!
//! In the event of a Barge-In interruption, the queue can be cancelled and purged
//! in sub-millisecond time.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::audio::chunker::calculate_rms_volume_slices;
use crate::audio::TtsProvider;
use serde::{Deserialize, Serialize};

/// Audio payload of a synthesized sentence chunk ready for playback and lip-sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedSentenceAudio {
    /// Monotonically increasing sequence number (0, 1, 2...).
    pub sequence: usize,

    /// Text content of this sentence.
    pub text: String,

    /// Raw WAV audio bytes.
    pub wav_bytes: Vec<u8>,

    /// Audio sample rate in Hz (e.g., 22050 or 24000).
    pub sample_rate: u32,

    /// Normalized [0.0, 1.0] 20ms RMS volume slices for real-time VRM lip-sync.
    pub rms_slices: Vec<f32>,

    /// Estimated audio duration in milliseconds.
    pub duration_ms: u64,
}

/// An ordered delivery queue ensuring sentence audio packets are emitted in strict sequence.
#[derive(Debug)]
pub struct OrderedTtsQueue {
    buffered_payloads: HashMap<usize, SynthesizedSentenceAudio>,
    next_sequence_to_dispatch: usize,
    total_enqueued: usize,
    is_cancelled: Arc<AtomicBool>,
}

impl Default for OrderedTtsQueue {
    fn default() -> Self {
        Self {
            buffered_payloads: HashMap::new(),
            next_sequence_to_dispatch: 0,
            total_enqueued: 0,
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl OrderedTtsQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Access the shared cancellation flag (can be cloned into worker threads/tasks).
    pub fn cancellation_flag(&self) -> Arc<AtomicBool> {
        self.is_cancelled.clone()
    }

    /// Check whether the pipeline has been cancelled by an interruption.
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::SeqCst)
    }

    /// Register a newly synthesized sentence chunk into the reordering buffer.
    pub fn insert_synthesized_chunk(&mut self, chunk: SynthesizedSentenceAudio) {
        if self.is_cancelled() {
            return;
        }
        self.total_enqueued = self.total_enqueued.max(chunk.sequence + 1);
        self.buffered_payloads.insert(chunk.sequence, chunk);
    }

    /// Retrieve the next sequential audio chunk ready for playback, if available.
    ///
    /// Even if sentence 1 finishes synthesizing before sentence 0, this will wait
    /// until sentence 0 is inserted before yielding sentence 0, and then sentence 1.
    pub fn pop_next_ready(&mut self) -> Option<SynthesizedSentenceAudio> {
        if self.is_cancelled() {
            return None;
        }

        if let Some(payload) = self
            .buffered_payloads
            .remove(&self.next_sequence_to_dispatch)
        {
            self.next_sequence_to_dispatch += 1;
            Some(payload)
        } else {
            None
        }
    }

    /// Drain all currently ready-in-sequence audio chunks.
    pub fn drain_ready(&mut self) -> Vec<SynthesizedSentenceAudio> {
        let mut ready = Vec::new();
        while let Some(chunk) = self.pop_next_ready() {
            ready.push(chunk);
        }
        ready
    }

    /// Next expected sequence number to be dispatched.
    pub fn next_sequence(&self) -> usize {
        self.next_sequence_to_dispatch
    }

    /// Count of items currently buffered waiting for earlier sequences.
    pub fn pending_count(&self) -> usize {
        self.buffered_payloads.len()
    }

    /// Purge and cancel the queue immediately (Barge-In).
    pub fn cancel_and_clear(&mut self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
        self.buffered_payloads.clear();
    }

    /// Reset queue state for a fresh conversation turn.
    pub fn reset(&mut self) {
        self.buffered_payloads.clear();
        self.next_sequence_to_dispatch = 0;
        self.total_enqueued = 0;
        self.is_cancelled.store(false, Ordering::SeqCst);
    }

    /// Synthesize a single sentence synchronously using a `TtsProvider` and package with RMS slices.
    pub fn synthesize_sentence<T: TtsProvider>(
        tts: &T,
        sequence: usize,
        text: &str,
        modulation: Option<&crate::audio::VoiceModulation>,
    ) -> Result<SynthesizedSentenceAudio, crate::error::RuntimeError> {
        let default_mod = crate::audio::VoiceModulation::default();
        let mod_ref = modulation.unwrap_or(&default_mod);
        let audio_out = tts.synthesize(text, mod_ref)?;
        let sample_rate = audio_out.sample_rate;

        // Skip standard 44-byte WAV header if present to get PCM data for RMS
        let pcm_data =
            if audio_out.audio_bytes.len() > 44 && &audio_out.audio_bytes[0..4] == b"RIFF" {
                &audio_out.audio_bytes[44..]
            } else {
                &audio_out.audio_bytes[..]
            };

        let rms_slices = calculate_rms_volume_slices(pcm_data, sample_rate);

        let duration_ms = if audio_out.duration_ms > 0 {
            audio_out.duration_ms as u64
        } else if sample_rate > 0 {
            // 16-bit mono PCM = 2 bytes per sample
            (pcm_data.len() as u64 * 1000) / (sample_rate as u64 * 2)
        } else {
            0
        };

        Ok(SynthesizedSentenceAudio {
            sequence,
            text: text.to_string(),
            wav_bytes: audio_out.audio_bytes,
            sample_rate,
            rms_slices,
            duration_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::MockTtsProvider;

    #[test]
    fn test_ordered_reordering_buffer() {
        let mut queue = OrderedTtsQueue::new();

        let chunk1 = SynthesizedSentenceAudio {
            sequence: 1,
            text: "Sentence 1".into(),
            wav_bytes: vec![1, 2, 3],
            sample_rate: 16000,
            rms_slices: vec![0.5],
            duration_ms: 100,
        };

        let chunk0 = SynthesizedSentenceAudio {
            sequence: 0,
            text: "Sentence 0".into(),
            wav_bytes: vec![4, 5, 6],
            sample_rate: 16000,
            rms_slices: vec![0.3],
            duration_ms: 100,
        };

        // Insert chunk 1 first (simulating parallel synthesis where sentence 1 finished first)
        queue.insert_synthesized_chunk(chunk1);

        // Sequence 0 is not ready yet, so pop_next_ready must return None!
        assert!(queue.pop_next_ready().is_none());
        assert_eq!(queue.pending_count(), 1);

        // Now chunk 0 arrives
        queue.insert_synthesized_chunk(chunk0);

        // Pop must yield sequence 0 first!
        let pop0 = queue.pop_next_ready().expect("chunk 0 ready");
        assert_eq!(pop0.sequence, 0);
        assert_eq!(pop0.text, "Sentence 0");

        // Then pop must yield sequence 1!
        let pop1 = queue.pop_next_ready().expect("chunk 1 ready");
        assert_eq!(pop1.sequence, 1);
        assert_eq!(pop1.text, "Sentence 1");

        // Nothing left
        assert!(queue.pop_next_ready().is_none());
    }

    #[test]
    fn test_cancel_and_clear_barge_in() {
        let mut queue = OrderedTtsQueue::new();
        queue.insert_synthesized_chunk(SynthesizedSentenceAudio {
            sequence: 0,
            text: "Sentence 0".into(),
            wav_bytes: vec![],
            sample_rate: 16000,
            rms_slices: vec![],
            duration_ms: 50,
        });

        assert_eq!(queue.pending_count(), 1);

        // Barge-In occurs!
        queue.cancel_and_clear();

        assert!(queue.is_cancelled());
        assert_eq!(queue.pending_count(), 0);
        assert!(queue.pop_next_ready().is_none());

        // Subsequent insertions are ignored while cancelled
        queue.insert_synthesized_chunk(SynthesizedSentenceAudio {
            sequence: 1,
            text: "Sentence 1".into(),
            wav_bytes: vec![],
            sample_rate: 16000,
            rms_slices: vec![],
            duration_ms: 50,
        });
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_synthesize_sentence_with_mock_tts() {
        let mock_tts = MockTtsProvider::new(22050);
        let chunk = OrderedTtsQueue::synthesize_sentence(&mock_tts, 0, "Xin chào bạn", None)
            .expect("synthesize success");

        assert_eq!(chunk.sequence, 0);
        assert_eq!(chunk.text, "Xin chào bạn");
        assert!(!chunk.wav_bytes.is_empty());
        assert_eq!(chunk.sample_rate, 22050);
        assert!(!chunk.rms_slices.is_empty());
    }
}
