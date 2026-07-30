use std::path::PathBuf;

/// Returned by [`TableEngine::compact`] on a successful compaction run.
#[derive(Debug, Clone)]
pub struct CompactResult {
    /// Number of live rows written to the merged partition.
    pub rows_kept: usize,
    /// Path of the new merged `.vvsst` file.
    pub path: PathBuf,
}
