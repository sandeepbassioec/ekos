use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("storage error: {0}")]
    Storage(#[from] vectorvolt_storage::StorageError),

    #[error("partition (file_id={file_id}, partition_id={partition_id}) not found in catalog")]
    PartitionNotFound { file_id: u16, partition_id: u16 },

    #[error("row offset {row_offset} out of bounds for partition with {row_count} rows")]
    RowOffsetOutOfBounds { row_offset: u32, row_count: u32 },

    #[error("column index {col_idx} out of range for schema of width {col_count}")]
    ColIdxOutOfRange { col_idx: usize, col_count: usize },
}

pub type QueryResult<T> = Result<T, QueryError>;
