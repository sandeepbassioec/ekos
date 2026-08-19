//! Layer 84 — LMAX ring buffer (VV-TXN-008).
//!
//! Tests the RingBuffer data structure directly: push/drain semantics,
//! capacity bounds, row_for lookup, and the flush-threshold signal.

use vectorvolt_txn::ring_buffer::RingBuffer;

fn buf(threshold: usize, width: usize) -> RingBuffer {
    RingBuffer::new(threshold, width)
}

// ── basic state ───────────────────────────────────────────────────────────────

#[test]
fn new_buffer_is_empty() {
    let rb = buf(8, 2);
    assert!(rb.is_empty());
    assert_eq!(rb.len(), 0);
}

#[test]
fn push_one_increments_len() {
    let rb = buf(8, 2);
    rb.push(1, vec![1, 10]).unwrap();
    assert_eq!(rb.len(), 1);
    assert!(!rb.is_empty());
}

#[test]
fn push_returns_false_before_threshold() {
    let rb = buf(4, 2);
    assert!(!rb.push(1, vec![1, 10]).unwrap());
    assert!(!rb.push(2, vec![2, 20]).unwrap());
    assert!(!rb.push(3, vec![3, 30]).unwrap());
}

#[test]
fn push_returns_true_at_threshold() {
    let rb = buf(4, 2);
    rb.push(1, vec![1, 10]).unwrap();
    rb.push(2, vec![2, 20]).unwrap();
    rb.push(3, vec![3, 30]).unwrap();
    let full = rb.push(4, vec![4, 40]).unwrap();
    assert!(full);
    assert!(rb.is_full());
}

#[test]
fn push_to_ring_capacity_returns_err() {
    // threshold = 2, capacity rounds up to 2 (already power of two)
    let rb = buf(2, 1);
    rb.push(1, vec![10]).unwrap();
    rb.push(2, vec![20]).unwrap();
    // Ring is physically full — next push should fail
    let result = rb.push(3, vec![30]);
    assert!(result.is_err());
}

// ── drain ─────────────────────────────────────────────────────────────────────

#[test]
fn drain_empty_returns_empty_vec() {
    let rb = buf(4, 2);
    assert!(rb.drain().is_empty());
}

#[test]
fn drain_returns_all_rows_in_order() {
    let rb = buf(8, 2);
    rb.push(10, vec![10, 100]).unwrap();
    rb.push(20, vec![20, 200]).unwrap();
    rb.push(30, vec![30, 300]).unwrap();
    let rows = rb.drain();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0], (10, vec![10, 100]));
    assert_eq!(rows[1], (20, vec![20, 200]));
    assert_eq!(rows[2], (30, vec![30, 300]));
}

#[test]
fn drain_empties_the_buffer() {
    let rb = buf(4, 1);
    rb.push(1, vec![99]).unwrap();
    rb.push(2, vec![88]).unwrap();
    rb.drain();
    assert!(rb.is_empty());
    assert_eq!(rb.len(), 0);
}

#[test]
fn drain_twice_second_is_empty() {
    let rb = buf(4, 1);
    rb.push(1, vec![1]).unwrap();
    rb.drain();
    assert!(rb.drain().is_empty());
}

#[test]
fn push_after_drain_reuses_slots() {
    let rb = buf(2, 1);
    rb.push(1, vec![10]).unwrap();
    rb.push(2, vec![20]).unwrap();
    rb.drain();
    // Ring is now empty; reuse slots
    rb.push(3, vec![30]).unwrap();
    let rows = rb.drain();
    assert_eq!(rows, vec![(3, vec![30])]);
}

// ── row_for ───────────────────────────────────────────────────────────────────

#[test]
fn row_for_finds_buffered_row() {
    let rb = buf(8, 2);
    let seq = rb.len(); // 0 before push
    rb.push(42, vec![42, 999]).unwrap();
    let got = rb.row_for(42, seq).unwrap();
    assert_eq!(got, vec![42, 999]);
}

#[test]
fn row_for_wrong_pk_returns_none() {
    let rb = buf(8, 2);
    let seq = rb.len();
    rb.push(1, vec![1, 10]).unwrap();
    assert!(rb.row_for(999, seq).is_none());
}

#[test]
fn row_for_after_drain_returns_none() {
    let rb = buf(8, 2);
    let seq = rb.len();
    rb.push(5, vec![5, 50]).unwrap();
    rb.drain();
    assert!(rb.row_for(5, seq).is_none());
}

// ── flush_threshold field ─────────────────────────────────────────────────────

#[test]
fn flush_threshold_is_stored() {
    let rb = buf(512, 3);
    assert_eq!(rb.flush_threshold, 512);
}

// ── is_full / is_empty ────────────────────────────────────────────────────────

#[test]
fn is_full_triggers_at_threshold_not_capacity() {
    // threshold=3, capacity rounds to 4
    let rb = buf(3, 1);
    rb.push(1, vec![1]).unwrap();
    rb.push(2, vec![2]).unwrap();
    assert!(!rb.is_full());
    rb.push(3, vec![3]).unwrap();
    assert!(rb.is_full());
    // One more slot still physically available (capacity=4)
    assert!(rb.push(4, vec![4]).is_ok());
}
