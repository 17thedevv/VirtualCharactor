use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::provider::{LlmError, LlmProvider, LlmRequest, LlmResponse, LlmUsage};

/// Configuration options for Google Gemini API provider.
#[derive(Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_output_tokens: Option<u32>,
    pub timeout: Duration,
    pub max_retries: usize,
}

impl GeminiConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "gemini-3.5-flash-lite".to_string(),
            temperature: None,
            max_output_tokens: None,
            timeout: Duration::from_secs(30),
            max_retries: 2,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_output_tokens(mut self, tokens: u32) -> Self {
        self.max_output_tokens = Some(tokens);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }
}

impl std::fmt::Debug for GeminiConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiConfig")
            .field("api_key", &mask_key(&self.api_key))
            .field("model", &self.model)
            .field("temperature", &self.temperature)
            .field("max_output_tokens", &self.max_output_tokens)
            .field("timeout", &self.timeout)
            .field("max_retries", &self.max_retries)
            .finish()
    }
}

/// Helper function to redact secret keys in log outputs.
fn mask_key(key: &str) -> String {
    if key.len() <= 6 {
        "[REDACTED]".to_string()
    } else {
        format!("{}...[REDACTED]", &key[..4])
    }
}

/// Production implementation of `LlmProvider` backed by Google Gemini API.
pub struct GeminiProvider {
    pub config: GeminiConfig,
    agent: ureq::Agent,
}

impl GeminiProvider {
    /// Create a GeminiProvider with default settings for a given API key.
    pub fn new(api_key: String) -> Self {
        Self::from_config(GeminiConfig::new(api_key))
    }

    /// Create a GeminiProvider with a custom model.
    pub fn with_model(api_key: String, model: String) -> Self {
        Self::from_config(GeminiConfig::new(api_key).with_model(model))
    }

    /// Create a GeminiProvider from a custom `GeminiConfig`.
    pub fn from_config(config: GeminiConfig) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(config.timeout)
            .timeout_write(config.timeout)
            .build();

        Self { config, agent }
    }

    pub fn api_key(&self) -> &str {
        &self.config.api_key
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }
}

impl std::fmt::Debug for GeminiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiProvider")
            .field("config", &self.config)
            .finish()
    }
}

// --- Gemini Specific Internal Request / Response Types (Strictly Private) ---

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxOutputTokens")]
    max_output_tokens: Option<u32>,
}

#[derive(Serialize)]
struct GeminiGenerateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "generationConfig")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Deserialize)]
struct GeminiResponseCandidate {
    content: Option<GeminiResponseContent>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    #[serde(default)]
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: Option<String>,
}

#[derive(Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u32>,
}

#[derive(Deserialize)]
struct GeminiGenerateResponse {
    candidates: Option<Vec<GeminiResponseCandidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
    error: Option<GeminiApiError>,
}

#[derive(Deserialize)]
struct GeminiApiError {
    message: String,
    code: Option<u16>,
}

impl LlmProvider for GeminiProvider {
    fn generate_text(&self, request: LlmRequest) -> vc_core::Result<LlmResponse> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.config.model, self.config.api_key
        );

        let mut contents = Vec::new();
        contents.push(GeminiContent {
            role: Some("user".into()),
            parts: vec![GeminiPart {
                text: request.prompt,
            }],
        });

        let system_instruction = request.system_instruction.map(|sys| GeminiContent {
            role: None,
            parts: vec![GeminiPart { text: sys }],
        });

        let temp = request.temperature.or(self.config.temperature);
        let max_tokens = request.max_tokens.or(self.config.max_output_tokens);
        let generation_config = if temp.is_some() || max_tokens.is_some() {
            Some(GeminiGenerationConfig {
                temperature: temp,
                max_output_tokens: max_tokens,
            })
        } else {
            None
        };

        let req_body = GeminiGenerateRequest {
            system_instruction,
            contents,
            generation_config,
        };

        let mut attempts = 0;
        let res = loop {
            attempts += 1;
            match self.agent.post(&url).send_json(&req_body) {
                Ok(response) => break response,
                Err(ureq::Error::Status(code, _resp)) if (code == 429 || code == 503) && attempts <= self.config.max_retries => {
                    let backoff = Duration::from_millis(500 * (1 << (attempts - 1)));
                    std::thread::sleep(backoff);
                    continue;
                }
                Err(ureq::Error::Status(code, resp)) => {
                    let error_msg = resp
                        .into_string()
                        .unwrap_or_else(|_| format!("HTTP error status {}", code));

                    let llm_err = match code {
                        429 => LlmError::RateLimited {
                            message: error_msg,
                            retry_after_secs: Some(10),
                        },
                        400 | 403 => LlmError::AuthenticationFailed(error_msg),
                        500 | 503 => LlmError::ModelUnavailable(error_msg),
                        _ => LlmError::Other(error_msg),
                    };
                    return Err(llm_err.into());
                }
                Err(ureq::Error::Transport(transport)) => {
                    let msg = transport.to_string();
                    let llm_err = if msg.contains("timed out") || msg.contains("Timeout") {
                        LlmError::Timeout(msg)
                    } else {
                        LlmError::NetworkError(msg)
                    };
                    return Err(llm_err.into());
                }
            }
        };

        let gemini_resp: GeminiGenerateResponse = res
            .into_json()
            .map_err(|e| vc_core::CoreError::ProviderError(format!("Failed to parse Gemini response: {e}")))?;

        if let Some(err) = gemini_resp.error {
            let msg = err.message;
            let llm_err = match err.code {
                Some(429) => LlmError::RateLimited {
                    message: msg,
                    retry_after_secs: Some(10),
                },
                Some(400) | Some(403) => LlmError::AuthenticationFailed(msg),
                _ => LlmError::Other(msg),
            };
            return Err(llm_err.into());
        }

        let first_candidate = gemini_resp
            .candidates
            .as_ref()
            .and_then(|cands| cands.first());

        let finish_reason = first_candidate.and_then(|c| c.finish_reason.clone());

        let text = first_candidate
            .and_then(|c| c.content.as_ref())
            .map(|content| {
                content
                    .parts
                    .iter()
                    .filter_map(|p| p.text.as_deref())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        let usage = gemini_resp.usage_metadata.map(|u| LlmUsage {
            prompt_tokens: u.prompt_token_count,
            completion_tokens: u.candidates_token_count,
            total_tokens: u.total_token_count,
        });

        Ok(LlmResponse {
            text,
            usage,
            finish_reason,
        })
    }

    fn name(&self) -> &'static str {
        "GoogleGeminiProvider"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_provider_instantiation() {
        let provider = GeminiProvider::new("test-key-12345".into());
        assert_eq!(provider.api_key(), "test-key-12345");
        assert_eq!(provider.model(), "gemini-3.5-flash-lite");
        assert_eq!(provider.name(), "GoogleGeminiProvider");
    }

    #[test]
    fn test_gemini_provider_with_custom_model() {
        let provider = GeminiProvider::with_model("test-key-12345".into(), "gemini-3.5-flash".into());
        assert_eq!(provider.model(), "gemini-3.5-flash");
    }

    #[test]
    fn test_gemini_debug_redacts_api_key() {
        let provider = GeminiProvider::new("AIzaSySecretApiKey999".into());
        let debug_str = format!("{:?}", provider);

        assert!(!debug_str.contains("AIzaSySecretApiKey999"));
        assert!(debug_str.contains("[REDACTED]"));
    }

    #[test]
    fn test_gemini_config_builder() {
        let config = GeminiConfig::new("my-api-key")
            .with_model("gemini-1.5-pro")
            .with_temperature(0.3)
            .with_max_output_tokens(2048)
            .with_max_retries(3);

        assert_eq!(config.model, "gemini-1.5-pro");
        assert_eq!(config.temperature, Some(0.3));
        assert_eq!(config.max_output_tokens, Some(2048));
        assert_eq!(config.max_retries, 3);
    }
}
