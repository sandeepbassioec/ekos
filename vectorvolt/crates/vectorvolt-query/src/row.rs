/// A single decoded row returned from a query.
///
/// `pk` is the primary key as stored in the CuckooMap.
/// `values` is one decoded `u64` per column, in schema order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRow {
    pub pk: u64,
    pub values: Vec<u64>,
}
