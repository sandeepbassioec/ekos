use vectorvolt_storage::StorageError;
use vectorvolt_txn::TxnError;
use vectorvolt_wal::WalError;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("wal: {0}")]
    Wal(#[from] WalError),

    #[error("txn: {0}")]
    Txn(#[from] TxnError),

    #[error("storage: {0}")]
    Storage(#[from] StorageError),

    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("partition ({file_id},{partition_id}) not registered in catalog")]
    PartitionMissing { file_id: u16, partition_id: u16 },

    #[error("column index {col_idx} out of range (schema has {col_count} columns)")]
    ColOutOfRange { col_idx: usize, col_count: usize },

    #[error("table '{0}' not found")]
    TableNotFound(String),

    #[error("table '{0}' already exists")]
    TableAlreadyExists(String),

    #[error("invalid table name: {0}")]
    InvalidTableName(String),

    #[error("invalid engine config: {0}")]
    InvalidConfig(String),

    #[error("no table selected — send UseTable first")]
    NoTableSelected,
    #[error("column {col_idx} has type {col_type} but value {value} is not valid for that type")]
    SchemaMismatch {
        col_idx: usize,
        col_type: &'static str,
        value: u64,
    },

    #[error("transaction {0} not found (already committed, rolled back, or never started)")]
    TxnNotFound(u64),
}

pub type EngineResult<T> = Result<T, EngineError>;
