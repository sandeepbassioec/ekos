use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("account not found: {0}")]
    AccountNotFound(u64),
    #[error("account already exists: {0}")]
    AccountExists(u64),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type LedgerResult<T> = Result<T, LedgerError>;
