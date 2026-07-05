use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("WAL error: {0}")]
    Wal(String),

    #[error("corrupt data: {0}")]
    Corrupt(String),

    #[error("key not found")]
    NotFound,
}

pub type StoreResult<T> = Result<T, StoreError>;
