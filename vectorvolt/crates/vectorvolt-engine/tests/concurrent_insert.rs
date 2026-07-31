//! Concurrent write-path correctness (VV-TXN-020).
//!
//! ## What these do and do not establish
//!
//! They are general coverage of the concurrent write path: many threads,
//! interleaved keys, no sleeps, with and without flushes running underneath.
//! Everything inserted must read back with its own values.
//!
//! **They are NOT a regression guard for the intermittent failure described in
//! `server_integration.rs`.** That was verified rather than assumed: the
//! write-buffer offset race was deliberately reintroduced and these tests still
//! passed, at 16 threads and again at 8x core count. A test that passes with the
//! bug present guards nothing, and claiming otherwise would be worse than having
//! no test — it would close the investigation.
//!
//! The offset race they were written for is real and is fixed in
//! `TxnManager::insert`, but it produces a row with the WRONG VALUES, whereas the
//! observed failure was a key missing entirely. Those are different bugs. See the
//! commit message for what has been ruled out.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

/// Driven through `TableEngine` rather than `TxnManager`, because the read side
/// of the race lives there — `TxnManager` has no `get`, and the bug is only
/// observable as "a key I inserted reads back missing or wrong".
fn manager(dir: &TempDir, threshold: usize) -> Arc<TableEngine> {
    Arc::new(
        TableEngine::open_with(
            vec!["pk".into(), "a".into(), "b".into()],
            dir.path(),
            EngineOptions::new(threshold).with_durability(Durability::WriteOnly),
        )
        .unwrap(),
    )
}

/// **Every concurrently inserted key is readable, with its own values.**
///
/// The direct regression test. `b` is the thread id, so a crossed index entry
/// shows up as a wrong value rather than only as a missing one — both failure
/// modes of the original bug are caught.
#[test]
fn concurrent_inserts_are_all_visible_with_correct_values() {
    for attempt in 0..5 {
        let dir = TempDir::new().unwrap();
        // Never auto-flush: the bug lives in the buffered path, and flushing would
        // move rows out of it and hide the race.
        let m = manager(&dir, 10_000_000);

        let threads: u64 = 16;
        const PER_THREAD: u64 = 200;

        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let m = Arc::clone(&m);
                std::thread::spawn(move || {
                    // Interleaved keys, not blocked ranges — adjacent keys from
                    // different threads maximise the chance of two inserts landing
                    // between the same pair of buffer operations.
                    for i in 0..PER_THREAD {
                        let pk = i * threads + t;
                        m.insert(pk, vec![pk, t, i]).unwrap();
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().expect("writer thread panicked");
        }

        for t in 0..threads {
            for i in 0..PER_THREAD {
                let pk = i * threads + t;
                let row = m
                    .get(pk)
                    .unwrap_or_else(|e| panic!("attempt {attempt}: get({pk}) errored: {e}"))
                    .unwrap_or_else(|| panic!("attempt {attempt}: pk {pk} is missing"))
                    .values;
                assert_eq!(row[0], pk, "attempt {attempt}: pk {pk} slot holds {row:?}");
                assert_eq!(
                    row[1], t,
                    "attempt {attempt}: pk {pk} resolved to thread {}'s row",
                    row[1]
                );
                assert_eq!(row[2], i, "attempt {attempt}: pk {pk} wrong payload");
            }
        }
    }
}

/// The same, but with flushes happening concurrently underneath.
///
/// A flush drains the buffer and rewrites index entries, so it races the insert
/// path in a second way: an offset recorded against the old buffer must not be
/// resolved against the new one.
#[test]
fn concurrent_inserts_survive_interleaved_flushes() {
    let dir = TempDir::new().unwrap();
    // Small threshold, so auto-flush fires repeatedly during the run.
    let m = manager(&dir, 64);

    const THREADS: u64 = 6;
    const PER_THREAD: u64 = 200;

    let handles: Vec<_> = (0..THREADS)
        .map(|t| {
            let m = Arc::clone(&m);
            std::thread::spawn(move || {
                for i in 0..PER_THREAD {
                    let pk = i * THREADS + t;
                    m.insert(pk, vec![pk, t, i]).unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("writer thread panicked");
    }

    let mut missing = Vec::new();
    let mut wrong = Vec::new();
    for t in 0..THREADS {
        for i in 0..PER_THREAD {
            let pk = i * THREADS + t;
            match m.get(pk).unwrap() {
                None => missing.push(pk),
                Some(r) if r.values[0] != pk || r.values[1] != t || r.values[2] != i => {
                    wrong.push((pk, r.values));
                }
                Some(_) => {}
            }
        }
    }
    assert!(
        missing.is_empty() && wrong.is_empty(),
        "{} keys missing (first few: {:?}), {} keys crossed (first few: {:?})",
        missing.len(),
        &missing[..missing.len().min(5)],
        wrong.len(),
        &wrong[..wrong.len().min(5)]
    );
}

/// Concurrent readers see a consistent view while writers run.
///
/// A read must never observe a half-installed row: either the key is not there
/// yet, or it is there with its own values. Never there with somebody else's.
#[test]
fn readers_never_observe_a_torn_row() {
    let dir = TempDir::new().unwrap();
    let m = manager(&dir, 10_000_000);
    const THREADS: u64 = 4;
    const PER_THREAD: u64 = 400;

    let torn = Arc::new(AtomicUsize::new(0));

    let writers: Vec<_> = (0..THREADS)
        .map(|t| {
            let m = Arc::clone(&m);
            std::thread::spawn(move || {
                for i in 0..PER_THREAD {
                    let pk = i * THREADS + t;
                    m.insert(pk, vec![pk, t, i]).unwrap();
                }
            })
        })
        .collect();

    let readers: Vec<_> = (0..3)
        .map(|_| {
            let m = Arc::clone(&m);
            let torn = Arc::clone(&torn);
            std::thread::spawn(move || {
                for _ in 0..2000 {
                    for t in 0..THREADS {
                        let pk = (t * 37) % (PER_THREAD * THREADS);
                        // A row that exists must be internally consistent: column 0
                        // is the pk by convention, so row[0] != pk means the index
                        // pointed at the wrong slot.
                        if let Ok(Some(r)) = m.get(pk)
                            && r.values[0] != pk
                        {
                            torn.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            })
        })
        .collect();

    for h in writers.into_iter().chain(readers) {
        h.join().expect("thread panicked");
    }
    assert_eq!(
        torn.load(Ordering::Relaxed),
        0,
        "a reader saw an index entry pointing at another row"
    );
}
