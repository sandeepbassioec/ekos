//! Scan primitives — range requests and result types used by ProjectionDispatcher.

use std::ops::Bound;
use bytes::Bytes;

/// A key range for a scan.  Uses `std::ops::Bound` semantics.
#[derive(Debug, Clone)]
pub struct KeyRange {
    pub start: Bound<Bytes>,
    pub end:   Bound<Bytes>,
}

impl KeyRange {
    /// Unbounded on both ends — full table scan.
    pub fn full() -> Self {
        KeyRange { start: Bound::Unbounded, end: Bound::Unbounded }
    }

    /// Keys with a given prefix (inclusive start, exclusive end computed by
    /// incrementing the last byte of the prefix).
    pub fn prefix(prefix: impl Into<Bytes>) -> Self {
        let p: Bytes = prefix.into();
        let end = next_prefix(&p);
        KeyRange {
            start: Bound::Included(p),
            end:   match end {
                Some(e) => Bound::Excluded(e),
                None    => Bound::Unbounded,
            },
        }
    }

    /// Inclusive on both ends.
    pub fn inclusive(start: impl Into<Bytes>, end: impl Into<Bytes>) -> Self {
        KeyRange { start: Bound::Included(start.into()), end: Bound::Included(end.into()) }
    }

    /// Check whether a key falls within this range.
    pub fn contains(&self, key: &Bytes) -> bool {
        let after_start = match &self.start {
            Bound::Unbounded        => true,
            Bound::Included(lo)     => key >= lo,
            Bound::Excluded(lo)     => key > lo,
        };
        let before_end = match &self.end {
            Bound::Unbounded        => true,
            Bound::Included(hi)     => key <= hi,
            Bound::Excluded(hi)     => key < hi,
        };
        after_start && before_end
    }

    /// True if a key is already past the end of this range (no point scanning further).
    pub fn past_end(&self, key: &Bytes) -> bool {
        match &self.end {
            Bound::Unbounded    => false,
            Bound::Included(hi) => key > hi,
            Bound::Excluded(hi) => key >= hi,
        }
    }
}

/// Increment a byte-string prefix by 1 to compute the exclusive upper bound
/// for a prefix scan.  Returns `None` if the prefix is all 0xFF bytes.
fn next_prefix(prefix: &Bytes) -> Option<Bytes> {
    let mut p = prefix.to_vec();
    for byte in p.iter_mut().rev() {
        if *byte < 0xFF {
            *byte += 1;
            return Some(Bytes::from(p));
        }
        *byte = 0;
    }
    None // all 0xFF — no upper bound
}

/// A single result entry from a scan.  Tombstones are never returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanEntry {
    pub key:   Bytes,
    pub value: Bytes,
}

/// Parameters for a scan operation.
#[derive(Debug, Clone)]
pub struct ScanRequest {
    pub range: KeyRange,
    /// Maximum number of entries to return.
    pub limit: Option<usize>,
}

impl ScanRequest {
    pub fn full() -> Self {
        ScanRequest { range: KeyRange::full(), limit: None }
    }

    pub fn prefix(prefix: impl Into<Bytes>) -> Self {
        ScanRequest { range: KeyRange::prefix(prefix), limit: None }
    }

    pub fn with_limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_range() {
        let r = KeyRange::prefix("user:");
        assert!(r.contains(&Bytes::from("user:alice")));
        assert!(r.contains(&Bytes::from("user:bob")));
        assert!(!r.contains(&Bytes::from("order:1")));
    }

    #[test]
    fn inclusive_range() {
        let r = KeyRange::inclusive("b", "d");
        assert!(!r.contains(&Bytes::from("a")));
        assert!(r.contains(&Bytes::from("b")));
        assert!(r.contains(&Bytes::from("c")));
        assert!(r.contains(&Bytes::from("d")));
        assert!(!r.contains(&Bytes::from("e")));
    }

    #[test]
    fn past_end() {
        let r = KeyRange::inclusive("a", "c");
        assert!(!r.past_end(&Bytes::from("b")));
        assert!(r.past_end(&Bytes::from("d")));
    }
}
