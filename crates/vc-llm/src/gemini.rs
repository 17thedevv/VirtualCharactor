use crate::provider::{LlmProvider, LlmRequest, LlmResponse};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct GeminiProvider {
    pub api_key: String,
    pub model: String,
    agent: ureq::Agent,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self::with_model(api_key, "gemini-3.5-flash-lite".to_string())
    }

    pub fn with_model(api_key: String, model: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(30))
            .timeout_write(Duration::from_secs(30))
            .build();

        Self {
            api_key,
            model,
            agent,
        }
    }
}

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
struct GeminiGenerateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    contents: Vec<GeminiContent>,
}

#[derive(Deserialize)]
struct GeminiResponseCandidate {
    content: Option<GeminiResponseContent>,
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
struct GeminiGenerateResponse {
    candidates: Option<Vec<GeminiResponseCandidate>>,
    error: Option<GeminiApiError>,
}

#[derive(Deserialize)]
struct GeminiApiError {
    message: String,
}

impl LlmProvider for GeminiProvider {
    fn generate_text(&self, request: LlmRequest) -> vc_core::Result<LlmResponse> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
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

        let req_body = GeminiGenerateRequest {
            system_instruction,
            contents,
        };

        let res = self
            .agent
            .post(&url)
            .send_json(&req_body)
            .map_err(|e| vc_core::CoreError::ProviderError(format!("Network error calling Gemini: {e}")))?;

        let gemini_resp: GeminiGenerateResponse = res
            .into_json()
            .map_err(|e| vc_core::CoreError::ProviderError(format!("Failed to parse Gemini response: {e}")))?;

        if let Some(err) = gemini_resp.error {
            return Err(vc_core::CoreError::ProviderError(format!(
                "Gemini returned error: {}",
                err.message
            )));
        }

        let text = gemini_resp
            .candidates
            .and_then(|cands| cands.into_iter().next())
            .and_then(|cand| cand.content)
            .map(|content| {
                content
                    .parts
                    .into_iter()
                    .filter_map(|p| p.text)
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        Ok(LlmResponse { text })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_provider_instantiation() {
        let provider = GeminiProvider::new("test-key".into());
        assert_eq!(provider.api_key, "test-key");
        assert_eq!(provider.model, "gemini-3.5-flash-lite");
    }

    #[test]
    fn test_gemini_provider_with_custom_model() {
        let provider = GeminiProvider::with_model("test-key".into(), "gemini-3.5-flash".into());
        assert_eq!(provider.model, "gemini-3.5-flash");
    }
}
