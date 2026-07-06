use thiserror::Error;

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("store error: {0}")]
    Store(String),

    #[error("serialization error: {0}")]
    Serialize(String),

    #[error("knowledge object not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("knowledge object already deleted: {0}")]
    Deleted(String),
}

pub type KernelResult<T> = Result<T, KernelError>;

impl From<vectorvolt_store::StoreError> for KernelError {
    fn from(e: vectorvolt_store::StoreError) -> Self {
        KernelError::Store(e.to_string())
    }
}

impl From<rmp_serde::encode::Error> for KernelError {
    fn from(e: rmp_serde::encode::Error) -> Self {
        KernelError::Serialize(e.to_string())
    }
}

impl From<rmp_serde::decode::Error> for KernelError {
    fn from(e: rmp_serde::decode::Error) -> Self {
        KernelError::Serialize(e.to_string())
    }
}
