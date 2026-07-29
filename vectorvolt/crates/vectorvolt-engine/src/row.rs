/// A fully-decoded row returned from [`TableEngine`] queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineRow {
    /// Primary key value.
    pub pk: u64,
    /// All column values in schema order (including the PK column at index 0).
    pub values: Vec<u64>,
}
