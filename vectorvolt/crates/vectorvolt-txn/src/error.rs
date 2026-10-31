use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxnError {
    #[error("WAL error: {0}")]
    Wal(#[from] vectorvolt_wal::WalError),

    #[error("storage error: {0}")]
    Storage(#[from] vectorvolt_storage::StorageError),

    #[error("index error: {0}")]
    Index(#[from] vectorvolt_index::IndexError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("schema mismatch: expected {expected} values, got {got}")]
    SchemaMismatch { expected: usize, got: usize },

    #[error("corrupt WAL payload: {0}")]
    CorruptPayload(String),

    #[error("pk {0} not found")]
    PkNotFound(u64),

    #[error("writer thread shut down unexpectedly")]
    WriterShutdown,

    #[error("WAL encryption error: {0}")]
    Encryption(String),

    #[error("every partition id is in use; compact the table to free some")]
    PartitionIdsExhausted,
}

pub type TxnResult<T> = Result<T, TxnError>;
