use crate::resource_manager::ResourceError;
use thiserror::Error;
use vc_core::error::CoreError;

/// Comprehensive runtime error type for VirtualCharacter orchestration, audio, and sensing.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Audio input error: {0}")]
    AudioInputError(String),

    #[error("TTS synthesis error: {0}")]
    TtsError(String),

    #[error("Resource budget constraint: {0}")]
    ResourceConstraint(#[from] ResourceError),

    #[error("Core domain error: {0}")]
    Core(#[from] CoreError),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Interaction timeout: {0}")]
    Timeout(String),

    #[error("Vision error: {0}")]
    VisionError(String),

    #[error("Internal runtime error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
