use crate::provider::{LlmProvider, LlmRequest, LlmResponse};

pub struct MockLlmProvider {
    pub default_response: String,
}

impl LlmProvider for MockLlmProvider {
    fn generate_text(&self, _request: LlmRequest) -> vc_core::Result<LlmResponse> {
        Ok(LlmResponse {
            text: self.default_response.clone(),
        })
    }
}
