//! Stream Token Chunker and Text Cleaner for Real-Time TTS Pipeline.
//!
//! Inspired by the AIRI streaming pipeline architecture:
//! Converts high-speed streaming LLM tokens into natural speech sentence chunks,
//! stripping out narrative stage directions (*smiles*, [laughs], (softly)) so TTS engines
//! do not pronounce bracketed emotions.

/// Cleans narrative actions, emotion descriptors, and stage directions from text.
///
/// VTuber LLM outputs frequently include expressive actions such as:
/// "*smiles brightly*", "[laughs softly]", "(whispering)", "*vẫy tay chào*".
/// These should be stripped before speech synthesis so TTS speaks only verbal dialogue.
pub fn clean_narrative_text(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_asterisk = false;
    let mut bracket_depth: u32 = 0;
    let mut paren_depth: u32 = 0;

    for ch in raw.chars() {
        match ch {
            '*' => {
                in_asterisk = !in_asterisk;
            }
            '[' => {
                bracket_depth += 1;
            }
            ']' => {
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                }
            }
            '(' => {
                paren_depth += 1;
            }
            ')' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                }
            }
            _ => {
                if !in_asterisk && bracket_depth == 0 && paren_depth == 0 {
                    out.push(ch);
                }
            }
        }
    }

    let tokens = out.split_whitespace().collect::<Vec<_>>();
    let mut result = String::with_capacity(out.len());
    for (i, tok) in tokens.iter().enumerate() {
        if i > 0 {
            let is_leading_punc = tok
                .chars()
                .next()
                .map(|c| [',', '.', '!', '?', ';', ':', '。', '！', '？', '…'].contains(&c))
                .unwrap_or(false)
                && tok.len() == 1;
            if !is_leading_punc {
                result.push(' ');
            }
        }
        result.push_str(tok);
    }
    result
}

/// Chunker that accumulates streaming tokens and yields ready-to-synthesize sentence chunks.
#[derive(Debug, Clone)]
pub struct TextStreamChunker {
    buffer: String,
    min_chunk_chars: usize,
}

impl TextStreamChunker {
    /// Create a new chunker with a specified minimum character threshold before punctuation split.
    pub fn new(min_chunk_chars: usize) -> Self {
        Self {
            buffer: String::new(),
            min_chunk_chars,
        }
    }

    /// Default chunker tuned for low-latency (<500ms TTFA) natural speech cadence.
    pub fn default_tts() -> Self {
        Self::new(12)
    }

    /// Feed a token into the chunker and return any complete sentence chunks.
    pub fn feed(&mut self, token: &str) -> Vec<String> {
        self.buffer.push_str(token);

        let mut ready_chunks = Vec::new();

        // Punctuation delimiters for sentence boundaries (Latin, CJK, Vietnamese)
        let delimiters = ['.', '!', '?', '\n', ';', ':', '。', '！', '？', '…'];

        loop {
            // Find first delimiter at or beyond min_chunk_chars
            let mut split_idx = None;

            for (byte_idx, ch) in self.buffer.char_indices() {
                if delimiters.contains(&ch) && byte_idx >= self.min_chunk_chars {
                    split_idx = Some(byte_idx + ch.len_utf8());
                    break;
                }
            }

            if let Some(idx) = split_idx {
                let chunk_raw: String = self.buffer[..idx].to_string();
                self.buffer = self.buffer[idx..].trim_start().to_string();

                let cleaned = clean_narrative_text(&chunk_raw);
                if !cleaned.is_empty() {
                    ready_chunks.push(cleaned);
                }
            } else {
                break;
            }
        }

        ready_chunks
    }

    /// Flush any remaining buffered characters at the end of the LLM stream.
    pub fn flush(&mut self) -> Option<String> {
        if self.buffer.trim().is_empty() {
            self.buffer.clear();
            None
        } else {
            let remaining = std::mem::take(&mut self.buffer);
            let cleaned = clean_narrative_text(&remaining);
            if cleaned.is_empty() {
                None
            } else {
                Some(cleaned)
            }
        }
    }

    /// Clear internal buffer.
    pub fn reset(&mut self) {
        self.buffer.clear();
    }
}

/// Computes normalized 20ms RMS (Root Mean Square) volume slices from raw 16-bit PCM audio.
///
/// Adopted from Open-LLM-VTuber architecture:
/// Allows the frontend avatar (Live2D / VRM) to calculate real-time lip movement directly
/// from audio stream packets without needing GPU neural phoneme alignment.
pub fn calculate_rms_volume_slices(pcm_16bit_le: &[u8], sample_rate: u32) -> Vec<f32> {
    if pcm_16bit_le.len() < 2 || sample_rate == 0 {
        return Vec::new();
    }

    // 20ms window size in samples
    let window_samples = (sample_rate as f32 * 0.020) as usize;
    if window_samples == 0 {
        return Vec::new();
    }

    let samples_count = pcm_16bit_le.len() / 2;
    let mut samples = Vec::with_capacity(samples_count);

    for chunk in pcm_16bit_le.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(sample as f32 / 32768.0);
    }

    let mut rms_slices = Vec::new();

    for window in samples.chunks(window_samples) {
        let sum_sq: f32 = window.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / window.len() as f32).sqrt();
        // Clamp and normalize to [0.0, 1.0]
        rms_slices.push(rms.min(1.0).max(0.0));
    }

    rms_slices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_narrative_text_removes_brackets_and_asterisks() {
        let raw =
            "*mỉm cười vui vẻ* Xin chào bạn! [laughs] Tôi rất vui được gặp bạn hôm nay (vẫy tay).";
        let cleaned = clean_narrative_text(raw);
        assert_eq!(cleaned, "Xin chào bạn! Tôi rất vui được gặp bạn hôm nay.");
    }

    #[test]
    fn test_stream_chunker_punctuation_boundaries() {
        let mut chunker = TextStreamChunker::new(10);

        let tokens = vec![
            "Xin ",
            "chào ",
            "bạn! ",
            "*cười nhẹ* ",
            "Hôm ",
            "nay ",
            "thời ",
            "tiết ",
            "rất ",
            "đẹp. ",
            "Bạn ",
            "có ",
            "khỏe ",
            "không?",
        ];

        let mut emitted = Vec::new();
        for t in tokens {
            emitted.extend(chunker.feed(t));
        }
        if let Some(rest) = chunker.flush() {
            emitted.push(rest);
        }

        assert_eq!(emitted.len(), 3);
        assert_eq!(emitted[0], "Xin chào bạn!");
        assert_eq!(emitted[1], "Hôm nay thời tiết rất đẹp.");
        assert_eq!(emitted[2], "Bạn có khỏe không?");
    }

    #[test]
    fn test_stream_chunker_respects_min_chars_before_splitting() {
        let mut chunker = TextStreamChunker::new(15);

        // "A. " is shorter than 15 chars, so it should not split immediately
        let c1 = chunker.feed("A. ");
        assert!(c1.is_empty());

        // Once buffer exceeds 15 chars and hits punctuation, it splits
        let c2 = chunker.feed("Đây là một câu hoàn chỉnh khá dài.");
        assert_eq!(c2.len(), 1);
        assert_eq!(c2[0], "A. Đây là một câu hoàn chỉnh khá dài.");
    }

    #[test]
    fn test_calculate_rms_volume_slices() {
        // Generate 100ms of 1000Hz sine wave PCM at 16000Hz (5 slices of 20ms)
        let sample_rate = 16000;
        let num_samples = 1600; // 100ms
        let mut pcm_bytes = Vec::with_capacity(num_samples * 2);

        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let val = (t * 1000.0 * 2.0 * std::f32::consts::PI).sin();
            let sample_i16 = (val * 16384.0) as i16;
            pcm_bytes.extend_from_slice(&sample_i16.to_le_bytes());
        }

        let slices = calculate_rms_volume_slices(&pcm_bytes, sample_rate);
        assert_eq!(slices.len(), 5);
        for &s in &slices {
            assert!(s > 0.25 && s < 0.6); // Sine wave RMS ~ 0.5 * 0.707 = 0.353
        }
    }
}
