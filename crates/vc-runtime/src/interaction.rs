use vc_core::context::Context;
use vc_core::decision::Decision;
use vc_core::character::CharacterId;

pub trait InteractionLifecycle {
    fn process_input(&self, character_id: CharacterId, input: String) -> vc_core::Result<Context>;
    fn execute_decision(&self, decision: Decision) -> vc_core::Result<()>;
}
