/// Snapshot of a [`TableEngine`]'s current state.
#[derive(Debug, Clone)]
pub struct EngineStats {
    /// Rows accumulated in the write buffer, not yet flushed to disk.
    pub rows_buffered: usize,
    /// Total rows flushed to `.vvsst` partition files since engine open.
    pub rows_flushed: u64,
    /// Number of `.vvsst` partition files currently registered.
    pub partition_count: usize,
    /// Column names in schema order.
    pub schema: Vec<String>,
    /// Auto-compact threshold if configured, otherwise `None`.
    pub auto_compact_threshold: Option<usize>,
}
