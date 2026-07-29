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
}

pub type EngineResult<T> = Result<T, EngineError>;
