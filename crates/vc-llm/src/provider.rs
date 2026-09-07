pub struct LlmRequest {
    pub prompt: String,
    pub system_instruction: Option<String>,
}

pub struct LlmResponse {
    pub text: String,
}

pub trait LlmProvider {
    fn generate_text(&self, request: LlmRequest) -> vc_core::Result<LlmResponse>;
}
