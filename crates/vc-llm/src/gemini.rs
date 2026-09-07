use crate::provider::{LlmProvider, LlmRequest, LlmResponse};

pub struct GeminiProvider {
    pub api_key: String,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl LlmProvider for GeminiProvider {
    fn generate_text(&self, _request: LlmRequest) -> vc_core::Result<LlmResponse> {
        // This is a stub implementation
        Err(vc_core::CoreError::ProviderError(
            "Gemini provider not yet fully implemented".into(),
        ))
    }
}
