use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("order not found: {0}")]
    NotFound(u64),
    #[error("invalid status transition from {0:?}")]
    InvalidTransition(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type OrderResult<T> = Result<T, OrderError>;
