use vc_core::character::CharacterId;
use vc_llm::provider::LlmProvider;
use std::sync::Arc;

pub struct RuntimeEngine {
    pub llm_provider: Arc<dyn LlmProvider>,
}

impl RuntimeEngine {
    pub fn new(llm_provider: Arc<dyn LlmProvider>) -> Self {
        Self { llm_provider }
    }
    
    pub fn tick(&self, _character_id: CharacterId) -> vc_core::Result<()> {
        Ok(())
    }
}
