use vectorvolt_engine::EngineError;
use vectorvolt_storage::StorageError;

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("syntax error: {0}")]
    Syntax(String),

    #[error("unexpected end of input")]
    UnexpectedEof,

    #[error("engine: {0}")]
    Engine(#[from] EngineError),

    #[error("storage: {0}")]
    Storage(#[from] StorageError),

    #[error("column index {col_idx} out of range (col_count={col_count})")]
    ColIdxOutOfRange { col_idx: usize, col_count: usize },

    #[error("partition not found: file_id={file_id} partition_id={partition_id}")]
    PartitionNotFound { file_id: u16, partition_id: u16 },

    #[error("row offset {row_offset} out of bounds (row_count={row_count})")]
    RowOffsetOutOfBounds { row_offset: u32, row_count: u32 },
}

pub type QueryResult<T> = Result<T, QueryError>;
