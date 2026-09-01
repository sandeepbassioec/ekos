//! [`PrefetchAdvisor`] — stride-detection prefetch intelligence (VV-PRE-001).
//!
//! Tracks per-table access patterns in a fixed-size ring and predicts the next
//! set of primary keys to warm into cache based on detected strides.

use std::collections::{HashMap, VecDeque};

const HISTORY_CAP: usize = 64;
const STRIDE_WINDOW: usize = 4;

/// Ring-buffer of recent primary-key accesses for one table.
pub struct AccessPattern {
    history: VecDeque<u64>,
}

impl AccessPattern {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(HISTORY_CAP),
        }
    }

    /// Record a primary-key access.
    pub fn record(&mut self, pk: u64) {
        if self.history.len() == HISTORY_CAP {
            self.history.pop_front();
        }
        self.history.push_back(pk);
    }

    /// Detect the dominant stride from the last `STRIDE_WINDOW` accesses.
    /// Returns `Some(stride)` when the last four deltas are all equal (non-zero).
    pub fn detected_stride(&self) -> Option<i64> {
        let n = self.history.len();
        if n < STRIDE_WINDOW + 1 {
            return None;
        }
        let tail: Vec<u64> = self
            .history
            .iter()
            .rev()
            .take(STRIDE_WINDOW + 1)
            .copied()
            .collect();
        // tail[0] is the most recent
        let deltas: Vec<i64> = tail.windows(2).map(|w| w[0] as i64 - w[1] as i64).collect();
        let first = deltas[0];
        if first != 0 && deltas.iter().all(|&d| d == first) {
            Some(first)
        } else {
            None
        }
    }

    /// Return up to `n` predicted next primary keys based on detected stride.
    pub fn prefetch_candidates(&self, n: usize) -> Vec<u64> {
        let stride = match self.detected_stride() {
            Some(s) => s,
            None => return vec![],
        };
        let last = match self.history.back() {
            Some(&pk) => pk,
            None => return vec![],
        };
        (1..=n as i64)
            .filter_map(|i| {
                let next = last as i64 + stride * i;
                if next >= 0 { Some(next as u64) } else { None }
            })
            .collect()
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

impl Default for AccessPattern {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-table prefetch advisor.
pub struct PrefetchAdvisor {
    tables: HashMap<String, AccessPattern>,
}

impl PrefetchAdvisor {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Record an access for `table` at `pk`.
    pub fn record(&mut self, table: &str, pk: u64) {
        self.tables.entry(table.to_string()).or_default().record(pk);
    }

    /// Return prefetch candidates for `table`.
    pub fn candidates(&self, table: &str, n: usize) -> Vec<u64> {
        self.tables
            .get(table)
            .map(|ap| ap.prefetch_candidates(n))
            .unwrap_or_default()
    }

    /// Detected stride for `table`.
    pub fn stride(&self, table: &str) -> Option<i64> {
        self.tables.get(table).and_then(|ap| ap.detected_stride())
    }

    pub fn table_count(&self) -> usize {
        self.tables.len()
    }
}

impl Default for PrefetchAdvisor {
    fn default() -> Self {
        Self::new()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pattern_no_stride() {
        let ap = AccessPattern::new();
        assert!(ap.detected_stride().is_none());
    }

    #[test]
    fn record_fills_history() {
        let mut ap = AccessPattern::new();
        for i in 0..10u64 {
            ap.record(i * 2);
        }
        assert_eq!(ap.history_len(), 10);
    }

    #[test]
    fn stride_detected_on_sequential() {
        let mut ap = AccessPattern::new();
        for i in 0..=STRIDE_WINDOW as u64 {
            ap.record(i * 3);
        }
        assert_eq!(ap.detected_stride(), Some(3));
    }

    #[test]
    fn no_stride_on_random() {
        let mut ap = AccessPattern::new();
        for pk in [1u64, 7, 2, 9, 4] {
            ap.record(pk);
        }
        assert!(ap.detected_stride().is_none());
    }

    #[test]
    fn prefetch_candidates_returns_n() {
        let mut ap = AccessPattern::new();
        for i in 0..=STRIDE_WINDOW as u64 {
            ap.record(i);
        }
        let c = ap.prefetch_candidates(3);
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn prefetch_candidates_empty_no_stride() {
        let ap = AccessPattern::new();
        assert!(ap.prefetch_candidates(4).is_empty());
    }

    #[test]
    fn history_cap_enforced() {
        let mut ap = AccessPattern::new();
        for i in 0..HISTORY_CAP + 10 {
            ap.record(i as u64);
        }
        assert_eq!(ap.history_len(), HISTORY_CAP);
    }

    #[test]
    fn advisor_records_and_returns() {
        let mut adv = PrefetchAdvisor::new();
        for i in 0..=STRIDE_WINDOW as u64 {
            adv.record("orders", i * 5);
        }
        assert_eq!(adv.stride("orders"), Some(5));
        assert!(!adv.candidates("orders", 2).is_empty());
    }

    #[test]
    fn advisor_unknown_table_empty() {
        let adv = PrefetchAdvisor::new();
        assert!(adv.candidates("missing", 4).is_empty());
        assert!(adv.stride("missing").is_none());
    }

    #[test]
    fn advisor_multiple_tables() {
        let mut adv = PrefetchAdvisor::new();
        for i in 0..=STRIDE_WINDOW as u64 {
            adv.record("a", i * 2);
            adv.record("b", i * 10);
        }
        assert_eq!(adv.stride("a"), Some(2));
        assert_eq!(adv.stride("b"), Some(10));
        assert_eq!(adv.table_count(), 2);
    }

    #[test]
    fn negative_stride_no_underflow_candidates() {
        let mut ap = AccessPattern::new();
        // descending sequence: stride = -1
        for i in (0..=STRIDE_WINDOW as u64).rev() {
            ap.record(i);
        }
        let c = ap.prefetch_candidates(3);
        // next predicted values would be negative — all filtered out
        assert!(c.is_empty());
    }

    #[test]
    fn stride_needs_enough_history() {
        let mut ap = AccessPattern::new();
        ap.record(0);
        ap.record(1);
        // only 2 entries — not enough for STRIDE_WINDOW + 1
        assert!(ap.detected_stride().is_none());
    }
}
