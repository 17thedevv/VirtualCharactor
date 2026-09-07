use vc_core::character::CharacterId;

pub struct ChatRequest {
    pub character_id: CharacterId,
    pub user_message: String,
}

pub struct ChatResponse {
    pub reply: String,
}

pub trait ChatUseCase {
    fn handle_chat(&self, request: ChatRequest) -> vc_core::Result<ChatResponse>;
}
