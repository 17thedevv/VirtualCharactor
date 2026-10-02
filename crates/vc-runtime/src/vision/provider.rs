//! Vision Provider abstraction and backends (Ollama VLM and Mock).

use crate::error::RuntimeError;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::time::Instant;

/// Structured semantic observation produced by a Vision Language Model (VLM).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisionObservation {
    /// Natural language description of what is taking place on screen.
    pub description: String,
    /// Key UI entities, icons, or text snippets recognized.
    pub detected_entities: Vec<String>,
    /// Confidence score [0.0, 1.0].
    pub confidence: f32,
    /// End-to-end inference latency in milliseconds.
    pub latency_ms: u64,
    /// Model identifier that produced the observation.
    pub model_used: String,
}

impl VisionObservation {
    pub fn new(
        description: impl Into<String>,
        detected_entities: Vec<String>,
        confidence: f32,
        latency_ms: u64,
        model_used: impl Into<String>,
    ) -> Self {
        Self {
            description: description.into(),
            detected_entities,
            confidence: confidence.clamp(0.0, 1.0),
            latency_ms,
            model_used: model_used.into(),
        }
    }

    /// Degraded fallback observation when VLM is offline or times out.
    pub fn fallback(reason: &str) -> Self {
        Self {
            description: format!("Không thể quan sát màn hình ({})", reason),
            detected_entities: Vec::new(),
            confidence: 0.0,
            latency_ms: 0,
            model_used: "fallback".to_string(),
        }
    }
}

/// Abstract contract for Vision Language Model providers.
pub trait VisionProvider: Send + Sync {
    /// Process a screen image frame and generate a semantic observation.
    fn observe(&self, image_data: &[u8], prompt: &str) -> Result<VisionObservation, RuntimeError>;

    /// Model / Provider name identifier.
    fn provider_name(&self) -> &str;
}

/// Deterministic in-memory Mock Vision Provider for unit and integration testing.
pub struct MockVisionProvider {
    provider_name: String,
    canned_observations: RwLock<Vec<VisionObservation>>,
    recorded_prompts: RwLock<Vec<String>>,
    simulated_error: RwLock<Option<String>>,
}

impl MockVisionProvider {
    pub fn new(provider_name: impl Into<String>) -> Self {
        Self {
            provider_name: provider_name.into(),
            canned_observations: RwLock::new(Vec::new()),
            recorded_prompts: RwLock::new(Vec::new()),
            simulated_error: RwLock::new(None),
        }
    }

    pub fn push_canned_observation(&self, observation: VisionObservation) {
        let mut obs = self.canned_observations.write().unwrap();
        obs.push(observation);
    }

    pub fn set_simulated_error(&self, error_message: Option<String>) {
        let mut err = self.simulated_error.write().unwrap();
        *err = error_message;
    }

    pub fn last_prompt(&self) -> Option<String> {
        let prompts = self.recorded_prompts.read().unwrap();
        prompts.last().cloned()
    }
}

impl VisionProvider for MockVisionProvider {
    fn observe(&self, _image_data: &[u8], prompt: &str) -> Result<VisionObservation, RuntimeError> {
        {
            let mut recorded = self.recorded_prompts.write().unwrap();
            recorded.push(prompt.to_string());
        }

        {
            let err = self.simulated_error.read().unwrap();
            if let Some(ref msg) = *err {
                return Err(RuntimeError::VisionError(format!(
                    "Simulated VLM Error: {}",
                    msg
                )));
            }
        }

        let mut canned = self.canned_observations.write().unwrap();
        if !canned.is_empty() {
            Ok(canned.remove(0))
        } else {
            Ok(VisionObservation::new(
                "Màn hình hiển thị trình biên dịch mã nguồn Rust đang hoạt động bình thường.",
                vec!["VS Code".into(), "Cargo".into()],
                0.95,
                15,
                &self.provider_name,
            ))
        }
    }

    fn provider_name(&self) -> &str {
        &self.provider_name
    }
}

/// Ollama Local Vision Provider for models like `qwen2.5vl:3b`.
pub struct OllamaVisionProvider {
    base_url: String,
    model: String,
    timeout_secs: u64,
}

impl OllamaVisionProvider {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>, timeout_secs: u64) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            model: model.into(),
            timeout_secs,
        }
    }

    pub fn default_local() -> Self {
        Self::new("http://localhost:11434", "qwen2.5vl:3b", 15)
    }

    /// Fast base64 encoding without external bulky dependencies.
    pub fn base64_encode(data: &[u8]) -> String {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
        for chunk in data.chunks(3) {
            let b0 = chunk[0];
            let b1 = chunk.get(1).copied().unwrap_or(0);
            let b2 = chunk.get(2).copied().unwrap_or(0);

            out.push(TABLE[(b0 >> 2) as usize] as char);
            out.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
            if chunk.len() > 1 {
                out.push(TABLE[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(TABLE[(b2 & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
        }
        out
    }
}

#[derive(Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    images: Vec<String>,
    stream: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OllamaGenerateResponse {
    response: Option<String>,
    total_duration: Option<u64>,
}

impl VisionProvider for OllamaVisionProvider {
    fn observe(&self, image_data: &[u8], prompt: &str) -> Result<VisionObservation, RuntimeError> {
        let start = Instant::now();
        let encoded_img = Self::base64_encode(image_data);

        let payload = OllamaGenerateRequest {
            model: &self.model,
            prompt,
            images: vec![encoded_img],
            stream: false,
        };

        let url = format!("{}/api/generate", self.base_url);
        let resp = ureq::post(&url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .send_json(&payload)
            .map_err(|e| RuntimeError::VisionError(format!("Lỗi kết nối Ollama Vision: {}", e)))?;

        let gen_resp: OllamaGenerateResponse = resp.into_json().map_err(|e| {
            RuntimeError::VisionError(format!("Lỗi giải mã JSON Ollama Vision: {}", e))
        })?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let text = gen_resp
            .response
            .unwrap_or_else(|| "Không có mô tả hình ảnh từ Ollama.".to_string());

        Ok(VisionObservation::new(
            text.trim(),
            Vec::new(),
            0.9,
            duration_ms,
            &self.model,
        ))
    }

    fn provider_name(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_vision_provider_canned_and_recording() {
        let provider = MockVisionProvider::new("test-vlm");
        let canned = VisionObservation::new(
            "Người dùng đang xem video YouTube về kiến trúc máy tính",
            vec!["YouTube".into(), "Browser".into()],
            0.88,
            25,
            "test-vlm",
        );
        provider.push_canned_observation(canned.clone());

        let res = provider.observe(&[1, 2, 3], "Mô tả màn hình").unwrap();
        assert_eq!(res.description, canned.description);
        assert_eq!(provider.last_prompt(), Some("Mô tả màn hình".to_string()));
    }

    #[test]
    fn test_mock_vision_provider_simulated_error() {
        let provider = MockVisionProvider::new("test-vlm");
        provider.set_simulated_error(Some("Timeout 10s".into()));

        let res = provider.observe(&[1, 2, 3], "Prompt");
        assert!(res.is_err());
    }

    #[test]
    fn test_base64_encoding_correctness() {
        let data = b"Hello, Aria!";
        let encoded = OllamaVisionProvider::base64_encode(data);
        assert_eq!(encoded, "SGVsbG8sIEFyaWEh");
    }
}
