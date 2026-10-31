//! MVCC Phase 1 — monotonic commit clock + in-memory per-pk version chains.
//!
//! This is deliberately additive and parallel to `pk_index`: it does not touch
//! the on-disk `.vvsst` format, `CuckooMap`, or any existing read/flush/compaction
//! path. Every mutation appends a [`VersionEntry`] holding the row's committed
//! values (or a tombstone) alongside the commit timestamp handed out by
//! [`CommitClock`]. Historical reads (`get_at`, `scan_at`) walk this chain instead
//! of resolving through `pk_index`, so they stay correct even after a pk's
//! current-value descriptor has been overwritten, relocated by a flush, or
//! removed by a delete.
//!
//! ## Timestamps are WAL sequence numbers (PP-23 / PP-24)
//!
//! A live mutation's `commit_ts` is the sequence number of the WAL frame that
//! carries it, assigned while the writer holds the per-pk stripe lock that also
//! covers the in-memory apply. That makes version order equal WAL order for
//! every pk, and makes timestamps durable: a snapshot taken before a restart
//! means the same thing after it.
//!
//! History survives a restart by being rebuilt from the WAL on open (see
//! `TxnManager::rebuild_pk_index_from_partitions`), not from the `.vvsst`
//! files, which still hold one version per row. That is correct for as long as
//! the WAL is retained in full, which it is today; WAL truncation, when it is
//! added, must first persist the versions and tombstones it would discard.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;

/// Monotonically increasing commit-timestamp source.
///
/// `next()` is the only way to observe a fresh timestamp; every writer calls it
/// exactly once per committed mutation, so two commits never share a timestamp
/// even when they race across threads (`fetch_add` is atomic).
#[derive(Debug, Default)]
pub struct CommitClock {
    counter: AtomicU64,
}

impl CommitClock {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }

    /// Advance the clock and return the newly allocated timestamp.
    ///
    /// Starts at 1 (0 is reserved as "before anything was ever committed", so
    /// `begin_snapshot()` called before any write sees an empty table rather
    /// than colliding with a real commit's timestamp).
    pub fn next(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// The most recently allocated timestamp (0 if nothing has committed yet).
    pub fn current(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }

    /// Move the clock forward to `ts` if it is behind; never moves it back.
    ///
    /// Used when the timestamp is assigned elsewhere — a WAL sequence number —
    /// so concurrent committers can publish out of order without the clock
    /// ever regressing.
    pub fn advance_to(&self, ts: u64) {
        self.counter.fetch_max(ts, Ordering::SeqCst);
    }
}

/// One version of a row as of a given commit timestamp.
#[derive(Debug, Clone)]
pub struct VersionEntry {
    pub commit_ts: u64,
    /// `true` marks this entry as a tombstone: the row did not exist as of
    /// `commit_ts` (a delete landed here). `values` is `None` in that case.
    pub deleted: bool,
    pub values: Option<Vec<u64>>,
}

impl VersionEntry {
    pub fn live(commit_ts: u64, values: Vec<u64>) -> Self {
        VersionEntry {
            commit_ts,
            deleted: false,
            values: Some(values),
        }
    }

    pub fn tombstone(commit_ts: u64) -> Self {
        VersionEntry {
            commit_ts,
            deleted: true,
            values: None,
        }
    }
}

/// Per-pk append-only version chains, kept in commit_ts ascending order.
#[derive(Debug, Default)]
pub struct VersionIndex {
    chains: RwLock<HashMap<u64, Vec<VersionEntry>>>,
}

impl VersionIndex {
    pub fn new() -> Self {
        Self {
            chains: RwLock::new(HashMap::new()),
        }
    }

    /// Append a live version for `pk`. Callers must ensure `commit_ts` is
    /// greater than any previously appended for this pk (true as long as the
    /// caller always draws `commit_ts` from the same [`CommitClock`] and holds
    /// no assumption of ordering across different pks).
    pub fn push_live(&self, pk: u64, commit_ts: u64, values: Vec<u64>) {
        self.chains
            .write()
            .entry(pk)
            .or_default()
            .push(VersionEntry::live(commit_ts, values));
    }

    /// Append a tombstone for `pk` — the row is deleted as of `commit_ts`.
    pub fn push_tombstone(&self, pk: u64, commit_ts: u64) {
        self.chains
            .write()
            .entry(pk)
            .or_default()
            .push(VersionEntry::tombstone(commit_ts));
    }

    /// Bulk-append versions in the given order (used to rebuild history at
    /// open). Same ordering contract as [`push_live`](Self::push_live).
    pub fn extend(&self, versions: impl IntoIterator<Item = (u64, VersionEntry)>) {
        let mut chains = self.chains.write();
        for (pk, v) in versions {
            chains.entry(pk).or_default().push(v);
        }
    }

    /// Whether `pk` has any recorded version.
    pub fn contains(&self, pk: u64) -> bool {
        self.chains.read().contains_key(&pk)
    }

    /// The latest version of `pk` with `commit_ts <= read_ts`, or `None` if no
    /// such version exists or it is a tombstone.
    pub fn get_at(&self, pk: u64, read_ts: u64) -> Option<Vec<u64>> {
        let chains = self.chains.read();
        let chain = chains.get(&pk)?;
        // Versions are appended in ascending commit_ts order, so the last entry
        // with commit_ts <= read_ts is the visible one.
        chain
            .iter()
            .rev()
            .find(|v| v.commit_ts <= read_ts)
            .and_then(|v| if v.deleted { None } else { v.values.clone() })
    }

    /// All pks with a version visible (live, non-tombstone) as of `read_ts`.
    pub fn visible_pks_at(&self, read_ts: u64) -> Vec<u64> {
        let chains = self.chains.read();
        chains
            .iter()
            .filter_map(|(pk, chain)| {
                let visible = chain.iter().rev().find(|v| v.commit_ts <= read_ts)?;
                if visible.deleted { None } else { Some(*pk) }
            })
            .collect()
    }

    /// Drop every version of `pk` strictly older than `watermark_ts`, except the
    /// newest version with `commit_ts <= watermark_ts` (kept so reads exactly at
    /// the watermark still resolve). Versions newer than the watermark are never
    /// touched.
    pub fn gc_before(&self, pk: u64, watermark_ts: u64) {
        let mut chains = self.chains.write();
        let Some(chain) = chains.get_mut(&pk) else {
            return;
        };

        // Index of the newest entry with commit_ts <= watermark_ts (the one we
        // must keep so `get_at(pk, watermark_ts)` still resolves).
        let keep_from = chain
            .iter()
            .rposition(|v| v.commit_ts <= watermark_ts)
            .unwrap_or(0);

        if keep_from > 0 {
            chain.drain(0..keep_from);
        }
    }

    /// Number of retained versions for `pk` (test/introspection helper).
    #[cfg(test)]
    pub fn version_count(&self, pk: u64) -> usize {
        self.chains.read().get(&pk).map(Vec::len).unwrap_or(0)
    }
}
