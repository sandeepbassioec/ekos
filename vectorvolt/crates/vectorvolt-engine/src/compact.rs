use std::path::PathBuf;

/// Returned by [`TableEngine::compact`] on a successful compaction run.
#[derive(Debug, Clone)]
pub struct CompactResult {
    /// Number of live rows written to the merged partition.
    pub rows_kept: usize,
    /// Path of the new merged `.vvsst` file.
    pub path: PathBuf,
    /// Input files that could not be deleted after the manifest stopped naming
    /// them.
    ///
    /// Not an error — they are unreferenced and invisible to reads — but they
    /// occupy disk until something removes them, and a leak nobody is told about
    /// is one nobody cleans up. Nonzero is routine on Windows if any
    /// `PartitionReader` was still mapped when compaction ran.
    pub leaked_files: usize,
}
