use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("workflow not found: {0}")]
    NotFound(u64),
    #[error("step not found: {0}")]
    StepNotFound(u64),
    #[error("invalid state transition")]
    InvalidTransition,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;
