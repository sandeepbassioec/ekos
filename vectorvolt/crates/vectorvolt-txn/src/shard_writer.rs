//! Per-shard writer thread (VV-TXN-009).
//!
//! [`ShardWriter`] replaces the `Mutex<RingBuffer>` that existed in
//! [`TxnManager`](crate::manager::TxnManager) as a transitional shim since
//! Layer 84 (VV-TXN-008). It owns the ring buffer outright — no lock is ever
//! held around a push or drain — and serialises all write-side operations
//! through a dedicated OS thread (the "writer thread").
//!
//! ## Why a dedicated thread, not just a lock?
//!
//! A `Mutex<RingBuffer>` serialises pushes correctly, but it cannot avoid a
//! park/unpark round-trip when two callers compete. On a P-core a `parking_lot`
//! mutex acquisition costs ~40 ns uncontended and ~600 ns contended (a
//! cache-miss + futex). A channel send + receive costs ~120 ns but is
//! *constant* regardless of concurrency — there is no contention path because
//! the writer thread never competes with anyone for the ring buffer.
//!
//! ## Correctness guarantee
//!
//! Every `insert()` call:
//!
//! 1. Computes `offset = ring.len()` inside the writer thread (single-threaded,
//!    no concurrent readers of `len`).
//! 2. Writes the pk_index entry **before** pushing the row, so the row is
//!    always findable: the index entry exists from the moment `insert()` returns.
//! 3. Returns `full` — the caller decides whether to trigger a flush, identical
//!    to the old `Mutex` path.
//!
//! Every `drain()` call:
//!
//! 1. Empties the ring buffer in the writer thread.
//! 2. Populates `in_flight` **before** sending the reply, so a concurrent
//!    `row_for` that arrives after the drain sees the row in `in_flight`
//!    via [`TxnManager::buffered_or_inflight_row`].
//!
//! ## Shutdown
//!
//! When [`ShardWriter`] drops, the channel sender is dropped first, causing
//! the writer thread's `recv()` to return `Err`; the thread exits cleanly.

use std::collections::HashMap;
use std::sync::{Arc, mpsc};
use std::thread;

use parking_lot::{Mutex, RwLock};
use vectorvolt_index::{CuckooMap, RowDescriptor};

use crate::manager::PENDING_FILE_ID;
use crate::ring_buffer::RingBuffer;
use crate::{TxnError, TxnResult};

// ── Protocol ──────────────────────────────────────────────────────────────────

/// A row handed to a flush, with the pending offset recorded for it in
/// `pk_index` when it was buffered.
#[derive(Debug, Clone)]
pub struct DrainedRow {
    pub pk: u64,
    pub values: Vec<u64>,
    pub offset: u32,
}

/// Drain the ring, tagging each row with its absolute push sequence — the same
/// value `Insert` wrote into the row's pending descriptor.
fn drain_with_offsets(ring: &RingBuffer) -> Vec<DrainedRow> {
    let start = ring.head() - ring.len();
    ring.drain()
        .into_iter()
        .enumerate()
        .map(|(i, (pk, values))| DrainedRow {
            pk,
            values,
            offset: (start + i) as u32,
        })
        .collect()
}

/// Messages sent from calling threads to the writer thread.
///
/// Each variant carries a one-shot reply channel so the caller can wait for
/// the result without polling.
enum WriterOp {
    /// Push a row.  Returns `true` when `flush_threshold` is reached.
    Insert {
        pk: u64,
        values: Vec<u64>,
        reply: mpsc::SyncSender<TxnResult<bool>>,
    },
    /// Drain all buffered rows.  Also inserts every drained row into `in_flight`
    /// before replying, so subsequent `RowFor` queries see them there.
    Drain {
        reply: mpsc::SyncSender<Vec<DrainedRow>>,
    },
    /// Query the current ring-buffer length.
    Len { reply: mpsc::SyncSender<usize> },
    /// Look up a single row by push-sequence offset and primary key.
    RowFor {
        pk: u64,
        offset: usize,
        reply: mpsc::SyncSender<Option<Vec<u64>>>,
    },
}

// ── ShardWriter ───────────────────────────────────────────────────────────────

/// Owns a [`RingBuffer`] and a background OS thread that is the single
/// producer for that buffer.
///
/// All public methods are safe to call from any thread — they communicate with
/// the writer thread via a `std::sync::mpsc` channel and block until the
/// writer thread replies.
pub struct ShardWriter {
    tx: mpsc::Sender<WriterOp>,
    // Held only to keep the thread alive; joined implicitly on drop (detach).
    _thread: thread::JoinHandle<()>,
}

impl ShardWriter {
    /// Spawn the writer thread and return a handle.
    ///
    /// * `flush_threshold` / `schema_width` — forwarded to [`RingBuffer::new`].
    /// * `pk_index` — shared with the writer thread; the writer holds the write
    ///   lock briefly for each insert so the index entry exists before the row
    ///   is pushed into the ring.
    /// * `in_flight` — shared with the writer thread; populated atomically
    ///   during each drain so rows remain findable while a flush is in progress.
    pub fn new(
        flush_threshold: usize,
        schema_width: usize,
        pk_index: Arc<RwLock<CuckooMap>>,
        in_flight: Arc<Mutex<HashMap<u64, Vec<u64>>>>,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<WriterOp>();

        let thread = thread::Builder::new()
            .name("vv-shard-writer".into())
            .spawn(move || {
                let ring = RingBuffer::new(flush_threshold, schema_width);
                // Rows forced out of a physically full ring before the Drain
                // that was meant for them arrived. They must ride along with the
                // next Drain, or no flush ever writes them (PP-11).
                let mut overflow: Vec<DrainedRow> = Vec::new();

                while let Ok(op) = rx.recv() {
                    match op {
                        WriterOp::Insert { pk, values, reply } => {
                            // If the ring is physically full, drain it into
                            // in_flight before inserting.  This happens when a
                            // caller received full=true and sent a Drain message,
                            // but other threads' Insert messages arrived in the
                            // channel first.  The Drain that follows will see an
                            // empty ring and short-circuit — correct behaviour.
                            if ring.is_ring_full() {
                                let rows = drain_with_offsets(&ring);
                                if !rows.is_empty() {
                                    let mut inf = in_flight.lock();
                                    for r in &rows {
                                        inf.insert(r.pk, r.values.clone());
                                    }
                                }
                                overflow.extend(rows);
                            }

                            // Compute offset BEFORE push — this is the absolute
                            // head sequence number where the row will be stored.
                            // Using ring.head() (not ring.len()) ensures the offset
                            // stays correct after a drain, when len() resets to 0
                            // but head() continues from where it left off.
                            let offset = ring.head() as u32;

                            // Index entry first, row second.  If the index fails
                            // (e.g. capacity exhausted after many grows), nothing
                            // is pushed and the caller sees the error immediately.
                            match pk_index
                                .write()
                                .insert(pk, RowDescriptor::new(PENDING_FILE_ID, 0, offset))
                            {
                                Ok(_) => {
                                    let full = ring.push(pk, values).unwrap_or(true);
                                    let _ = reply.send(Ok(full));
                                }
                                Err(e) => {
                                    let _ = reply.send(Err(TxnError::Index(e)));
                                }
                            }
                        }

                        WriterOp::Drain { reply } => {
                            let mut rows = std::mem::take(&mut overflow);
                            let fresh = drain_with_offsets(&ring);
                            if !fresh.is_empty() {
                                // Populate in_flight BEFORE sending the reply so
                                // that any RowFor arriving afterwards finds the row
                                // there (writer-thread FIFO ordering guarantees this).
                                let mut inf = in_flight.lock();
                                for r in &fresh {
                                    inf.insert(r.pk, r.values.clone());
                                }
                            }
                            rows.extend(fresh);
                            let _ = reply.send(rows);
                        }

                        WriterOp::Len { reply } => {
                            let _ = reply.send(ring.len());
                        }

                        WriterOp::RowFor { pk, offset, reply } => {
                            let _ = reply.send(ring.row_for(pk, offset));
                        }
                    }
                }
                // Sender dropped → channel closed → thread exits.
            })
            .expect("failed to spawn shard writer thread");

        ShardWriter {
            tx,
            _thread: thread,
        }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Push a row. Returns `true` when `flush_threshold` rows are buffered.
    ///
    /// The pk_index entry is written inside the writer thread, before the row
    /// is pushed, so the row is always reachable by the time this returns.
    pub fn insert(&self, pk: u64, values: Vec<u64>) -> TxnResult<bool> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(WriterOp::Insert {
                pk,
                values,
                reply: reply_tx,
            })
            .map_err(|_| TxnError::WriterShutdown)?;
        reply_rx.recv().map_err(|_| TxnError::WriterShutdown)?
    }

    /// Drain all buffered rows (also populates `in_flight`).
    ///
    /// Rows come back oldest first, each with the pending offset its
    /// `pk_index` descriptor was given, so a flush can tell whether the
    /// descriptor still refers to this exact row before replacing it.
    pub fn drain(&self) -> TxnResult<Vec<DrainedRow>> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send(WriterOp::Drain { reply: reply_tx })
            .map_err(|_| TxnError::WriterShutdown)?;
        reply_rx.recv().map_err(|_| TxnError::WriterShutdown)
    }

    /// Whether the ring buffer is currently empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Number of rows currently buffered.
    pub fn len(&self) -> usize {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        if self.tx.send(WriterOp::Len { reply: reply_tx }).is_err() {
            return 0;
        }
        reply_rx.recv().unwrap_or(0)
    }

    /// Look up a buffered row by push-sequence offset and pk.
    pub fn row_for(&self, pk: u64, offset: usize) -> Option<Vec<u64>> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        if self
            .tx
            .send(WriterOp::RowFor {
                pk,
                offset,
                reply: reply_tx,
            })
            .is_err()
        {
            return None;
        }
        reply_rx.recv().unwrap_or(None)
    }
}
