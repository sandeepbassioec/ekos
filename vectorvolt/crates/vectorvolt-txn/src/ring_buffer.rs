//! LMAX Disruptor-style single-producer / single-consumer ring buffer (VV-TXN-008).
//!
//! ## Design
//!
//! The ring buffer replaces `WriteBuffer` as the in-memory accumulator for
//! INSERT rows before they are flushed to a MicroPartition.
//!
//! Key properties vs. the old `Vec`-based `WriteBuffer`:
//!
//! * **Pre-allocated** — all slots are allocated upfront.  There is no `Vec`
//!   growth on the hot write path.
//! * **Lock-free** — head (producer cursor) and tail (consumer cursor) are
//!   `AtomicUsize`; no mutex is needed when only one producer and one consumer
//!   are active at the same time.
//! * **Capacity-bounded** — the producer detects a full buffer (head − tail ≥
//!   capacity) and signals the caller to trigger a flush before retrying.
//!
//! ## Current usage in TxnManager
//!
//! `TxnManager` currently wraps `RingBuffer` in a `Mutex` because concurrent
//! inserts must atomically (a) query the buffer length for a `RowDescriptor`
//! offset and (b) push the row. Once per-shard writer threads are introduced
//! (VV-TXN-009), that external mutex will be removed and the buffer will run
//! in a true single-writer configuration.
//!
//! ## Ordering
//!
//! The producer uses a `Release` store on `head` after writing data; the
//! consumer uses an `Acquire` load on `head` before reading.  This forms a
//! happens-before edge that makes the slot data visible to the consumer
//! without additional fences.

use std::cell::UnsafeCell;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

// ── Error ─────────────────────────────────────────────────────────────────────

/// Error returned by [`RingBuffer::push`] when the ring is physically full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RingFull;

impl fmt::Display for RingFull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ring buffer is full")
    }
}

// ── Ring slot ─────────────────────────────────────────────────────────────────

struct Slot {
    /// Pre-allocated storage: written by producer, consumed by consumer.
    ///
    /// # Safety
    ///
    /// Access is guarded by the head/tail protocol: the producer only writes to
    /// `slots[head & mask]` and the consumer only reads from `slots[tail & mask]`,
    /// and `head − tail ≤ capacity` is always maintained.  The two sides never
    /// touch the same physical slot simultaneously.
    data: UnsafeCell<(u64, Vec<u64>)>,
}

impl Slot {
    fn new(schema_width: usize) -> Self {
        Slot {
            data: UnsafeCell::new((0, Vec::with_capacity(schema_width))),
        }
    }
}

// Safety: the head/tail protocol guarantees exclusive access per slot.
unsafe impl Send for Slot {}
unsafe impl Sync for Slot {}

// ── RingBuffer ────────────────────────────────────────────────────────────────

/// Pre-allocated, capacity-bounded, SPSC ring buffer for INSERT rows.
pub struct RingBuffer {
    slots: Box<[Slot]>,
    /// Producer cursor: index of the next slot the producer will write into.
    head: AtomicUsize,
    /// Consumer cursor: index of the next slot the consumer will read from.
    tail: AtomicUsize,
    /// Total number of slots. Always a power of two.
    capacity: usize,
    /// Bit-mask equivalent to `capacity − 1` for fast modulo.
    mask: usize,
    /// Row count at which the caller should trigger a flush.
    pub flush_threshold: usize,
    /// Number of `u64` columns per row (for slot pre-allocation).
    pub schema_width: usize,
}

impl RingBuffer {
    /// Create a new ring buffer.
    ///
    /// `flush_threshold` is the number of rows that triggers an automatic flush
    /// (analogous to `WriteBuffer::capacity`). The actual ring capacity is
    /// rounded up to the next power of two so that the index mask trick works.
    ///
    /// # Panics
    ///
    /// Panics if `flush_threshold == 0`.
    pub fn new(flush_threshold: usize, schema_width: usize) -> Self {
        assert!(flush_threshold > 0, "flush_threshold must be > 0");
        // Round up to the next power of two so `& mask` equals `% capacity`.
        let capacity = flush_threshold.next_power_of_two();
        let mask = capacity - 1;
        let slots: Box<[Slot]> = (0..capacity)
            .map(|_| Slot::new(schema_width))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        RingBuffer {
            slots,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            capacity,
            mask,
            flush_threshold,
            schema_width,
        }
    }

    // ── State queries ─────────────────────────────────────────────────────────

    /// Number of rows currently buffered (not yet drained).
    #[inline]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head - tail
    }

    /// True when no rows are buffered.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// True when `len() >= flush_threshold`.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() >= self.flush_threshold
    }

    /// True when the ring is physically at capacity (producer must wait).
    #[inline]
    pub fn is_ring_full(&self) -> bool {
        self.len() >= self.capacity
    }

    // ── Producer ──────────────────────────────────────────────────────────────

    /// Append a row. Returns `true` if `len() >= flush_threshold` after the push.
    ///
    /// Returns `Err(())` if the ring is physically full; the caller must drain
    /// (flush) before retrying.  In normal operation, callers flush as soon as
    /// `is_full()` returns true, so the ring should never be physically full.
    /// Returns `Ok(true)` when `len() >= flush_threshold` after the push,
    /// signalling the caller to trigger a flush.  Returns `Err(RingFull)` if
    /// the ring is physically full and no slot is available.
    pub fn push(&self, pk: u64, values: Vec<u64>) -> Result<bool, RingFull> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head - tail >= self.capacity {
            return Err(RingFull);
        }
        let idx = head & self.mask;
        // SAFETY: only the producer (single writer) writes to `slots[idx]` when
        // `head - tail < capacity`, guaranteeing the consumer is not reading idx.
        unsafe {
            let slot = &mut *self.slots[idx].data.get();
            slot.0 = pk;
            slot.1.clear();
            slot.1.extend_from_slice(&values);
        }
        // Release: makes the slot write visible to the consumer that Acquire-loads head.
        self.head.store(head + 1, Ordering::Release);
        Ok(self.len() >= self.flush_threshold)
    }

    // ── Consumer ─────────────────────────────────────────────────────────────

    /// Look up a buffered row by its push-sequence offset and primary key.
    ///
    /// `offset` is the head value at the time the row was pushed (the index into
    /// the logical sequence, stored in `RowDescriptor`). The pk check guards
    /// against stale offsets: if the slot has been consumed and reused, the pk
    /// will not match and `None` is returned.
    pub fn row_for(&self, pk: u64, offset: usize) -> Option<Vec<u64>> {
        let idx = offset & self.mask;
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Acquire);
        if offset < tail || offset >= head {
            return None;
        }
        // SAFETY: offset is within [tail, head) so the slot belongs to this
        // producer-published range and the consumer has not yet advanced past it.
        let (stored_pk, values) = unsafe { &*self.slots[idx].data.get() };
        if *stored_pk == pk {
            Some(values.clone())
        } else {
            None
        }
    }

    /// Drain all buffered rows, leaving the ring empty.
    pub fn drain(&self) -> Vec<(u64, Vec<u64>)> {
        let head = self.head.load(Ordering::Acquire);
        let mut tail = self.tail.load(Ordering::Relaxed);
        let count = head - tail;
        let mut out = Vec::with_capacity(count);
        while tail < head {
            let idx = tail & self.mask;
            // SAFETY: tail < head means this slot was published by the producer
            // and the consumer (this call) is the only reader in [tail, head).
            let (pk, values) = unsafe { &*self.slots[idx].data.get() };
            out.push((*pk, values.clone()));
            tail += 1;
        }
        // Release: makes the freed slots visible to the producer.
        self.tail.store(tail, Ordering::Release);
        out
    }
}
