use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Character not found: {0}")]
    CharacterNotFound(String),
    #[error("Memory not found: {0}")]
    MemoryNotFound(String),
    #[error("Invalid state transition")]
    InvalidStateTransition,
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
