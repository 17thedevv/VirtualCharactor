pub mod state_machine;

pub use state_machine::{
    ConversationEvent, ConversationState, ConversationStateMachine, InterruptionResult,
    TransitionError,
};
