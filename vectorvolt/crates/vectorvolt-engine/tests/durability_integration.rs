//! Durability tests for [`TableEngine`].
//!
//! These exist because `TableEngine::open` previously hardcoded `with_fsync(false)`
//! with **no way to change it**, so every write landed in the OS page cache and
//! never on disk. Nothing tested durability, so nothing noticed — and every INSERT
//! latency figure measured against that engine was a non-durable write.
//!
//! ## What can and cannot be tested in-process
//!
//! Being honest about the limit up front, because it shapes what these tests are
//! worth:
//!
//! | Failure | Testable here? |
//! |---|---|
//! | Process crash / abrupt exit | **Yes** — drop the engine, reopen, assert the data is there. Passes under either durability setting, since the page cache survives the process |
//! | **Power loss / kernel panic** | **No.** Only `fsync` protects against it, and proving that needs the machine to actually lose power. No unit test can assert it |
//!
//! So these tests verify the *mechanism* is wired and the data survives a
//! process boundary. They cannot verify the `fsync` syscall's effect against real
//! power loss — that claim rests on the OS contract, and the honest statement is
//! that we request the guarantee rather than that we have demonstrated it.

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};

fn schema() -> Vec<String> {
    vec!["id".into(), "amount".into()]
}

// ── defaults ──────────────────────────────────────────────────────────────────

#[test]
fn open_defaults_to_durable() {
    // The correctness fix: the safe setting is the one you get without asking.
    assert_eq!(EngineOptions::new(1000).durability, Durability::Fsync);
}

#[test]
fn durability_can_be_traded_away_explicitly() {
    let o = EngineOptions::new(1000).with_durability(Durability::WriteOnly);
    assert_eq!(o.durability, Durability::WriteOnly);
    assert_eq!(o.flush_threshold, 1000);
}

// ── data survives a process boundary ──────────────────────────────────────────

/// Reopening after dropping the engine is the strongest in-process proxy for a
/// crash: it releases every mmap and file handle and rebuilds all state from disk.
fn survives_reopen(durability: Durability) {
    let dir = TempDir::new().unwrap();
    let opts = EngineOptions::new(1_000_000).with_durability(durability);

    {
        let engine = TableEngine::open_with(schema(), dir.path(), opts).unwrap();
        for i in 1..=50u64 {
            engine.insert(i, vec![i, i * 100]).unwrap();
        }
        engine.flush().unwrap();
    } // dropped — as close to a crash as an in-process test gets

    let reopened = TableEngine::open_with(schema(), dir.path(), opts).unwrap();
    for i in 1..=50u64 {
        let row = reopened
            .get(i)
            .unwrap()
            .unwrap_or_else(|| panic!("pk {i} lost across reopen with {durability:?}"));
        assert_eq!(row.values[1], i * 100, "value corrupted for pk {i}");
    }
}

#[test]
fn durable_writes_survive_reopen() {
    survives_reopen(Durability::Fsync);
}

#[test]
fn write_only_also_survives_reopen_which_is_why_the_bug_hid() {
    // Deliberately asserts the *weaker* mode also passes. That is exactly why the
    // missing fsync went unnoticed: a process-boundary test cannot distinguish
    // "durable" from "in the page cache". Only power loss can, and no test here
    // can produce it. Documenting that in an assertion is more useful than
    // pretending this suite proves durability.
    survives_reopen(Durability::WriteOnly);
}

// ── the WAL is genuinely written in both modes ─────────────────────────────────

#[test]
fn fsync_puts_bytes_on_disk_while_buffered_does_not() {
    // The finding this change came from: `wal.append()` copies into a BufWriter and
    // nothing on the write path flushed it, so bytes stayed in *this process's*
    // memory and a plain crash lost recent writes.
    //
    // PLATFORM CAVEAT, learned by running this: file *size* cannot distinguish
    // WriteOnly from Buffered on Windows. A `write()` hands bytes to the kernel,
    // but the directory entry's size is not refreshed until something syncs, so an
    // un-synced-but-written file still reports its old length. Only the Fsync vs
    // Buffered gap is observable this way, so that is all this asserts.
    fn wal_bytes_after_inserts(durability: Durability) -> u64 {
        let dir = TempDir::new().unwrap();
        let engine = TableEngine::open_with(
            schema(),
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(durability),
        )
        .unwrap();
        for i in 1..=20u64 {
            engine.insert(i, vec![i, i]).unwrap();
        }
        // Deliberately NOT dropping the engine — drop flushes the BufWriter, which
        // would erase the distinction this test exists to show.
        std::fs::read_dir(dir.path().join("wal"))
            .map(|rd| {
                rd.flatten()
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum()
            })
            .unwrap_or(0)
    }

    let fsync = wal_bytes_after_inserts(Durability::Fsync);
    let buffered = wal_bytes_after_inserts(Durability::Buffered);

    println!("  WAL bytes visible on disk after 20 inserts, engine still alive:");
    println!("    Fsync={fsync}  Buffered={buffered}");

    assert!(
        fsync > buffered,
        "Fsync ({fsync}) did not put more on disk than Buffered ({buffered}) —          the WAL is not being synced, so this mode provides no durability"
    );
    // 20 frames of ~97 bytes overhead plus payload is well over a kilobyte; a
    // bare segment header is 64. Guards against "it synced an empty file".
    assert!(
        fsync > 1_000,
        "Fsync wrote only {fsync} bytes for 20 frames — suspiciously close to just          the segment header"
    );
}

#[test]
fn deletes_survive_reopen_when_durable() {
    let dir = TempDir::new().unwrap();
    let opts = EngineOptions::new(1_000_000); // durable by default

    {
        let engine = TableEngine::open_with(schema(), dir.path(), opts).unwrap();
        for i in 1..=10u64 {
            engine.insert(i, vec![i, i * 7]).unwrap();
        }
        engine.flush().unwrap();
        engine.delete(4).unwrap();
    }

    let reopened = TableEngine::open_with(schema(), dir.path(), opts).unwrap();
    assert!(
        reopened.get(4).unwrap().is_none(),
        "deleted pk 4 came back after reopen — WAL delete replay is broken"
    );
    for i in [1u64, 2, 3, 5, 10] {
        assert!(reopened.get(i).unwrap().is_some(), "pk {i} lost");
    }
}

// ── cost ──────────────────────────────────────────────────────────────────────

#[test]
fn each_durability_level_costs_what_it_should() {
    // Pins the reason group commit matters, and produces the honest per-insert
    // numbers. Only the ORDERING is asserted — absolute fsync cost varies by an
    // order of magnitude across storage hardware, so a fixed threshold would be
    // flaky rather than meaningful.
    use std::time::Instant;

    fn time_inserts(durability: Durability, n: u64) -> std::time::Duration {
        let dir = TempDir::new().unwrap();
        let engine = TableEngine::open_with(
            schema(),
            dir.path(),
            EngineOptions::new(1_000_000).with_durability(durability),
        )
        .unwrap();
        engine.insert(0, vec![0, 0]).unwrap(); // warm up: creates the segment
        let t0 = Instant::now();
        for i in 1..=n {
            engine.insert(i, vec![i, i]).unwrap();
        }
        t0.elapsed()
    }

    const N: u64 = 200;
    let buffered = time_inserts(Durability::Buffered, N);
    let write_only = time_inserts(Durability::WriteOnly, N);
    let fsync = time_inserts(Durability::Fsync, N);

    let per = |d: std::time::Duration| d.as_nanos() as f64 / N as f64 / 1000.0;
    println!("  per-insert cost over {N} inserts:");
    println!(
        "    Buffered   {:>8.1} us   survives nothing",
        per(buffered)
    );
    println!(
        "    WriteOnly  {:>8.1} us   survives process crash",
        per(write_only)
    );
    println!(
        "    Fsync      {:>8.1} us   survives power loss",
        per(fsync)
    );
    println!(
        "    Fsync is {:.1}x Buffered — the gap group commit (VV-WAL-011) exists to amortise",
        fsync.as_secs_f64() / buffered.as_secs_f64().max(f64::MIN_POSITIVE)
    );

    assert!(
        fsync >= buffered,
        "Fsync ({fsync:?}) was not slower than Buffered ({buffered:?}) — fsync is          probably not being issued at all"
    );
}

// ── group commit (VV-WAL-011) ─────────────────────────────────────────────────

#[test]
fn concurrent_durable_writers_share_fsyncs() {
    // The payoff of wiring SyncCommitter through TxnManager: with fsync at 67x the
    // cost of the actual work, the thing that matters is that N writers do not cost
    // N fsyncs.
    use std::sync::Arc;

    const THREADS: usize = 8;
    const PER_THREAD: u64 = 30;

    let dir = TempDir::new().unwrap();
    let engine = Arc::new(
        TableEngine::open_with(schema(), dir.path(), EngineOptions::new(1_000_000)).unwrap(),
    );

    let handles: Vec<_> = (0..THREADS)
        .map(|t| {
            let engine = Arc::clone(&engine);
            std::thread::spawn(move || {
                let base = t as u64 * 1_000;
                for i in 1..=PER_THREAD {
                    engine.insert(base + i, vec![base + i, i]).unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("writer thread panicked");
    }

    let s = engine.commit_stats();
    println!(
        "  {THREADS} threads x {PER_THREAD} durable inserts: {} commits, {} fsyncs, \
         {} free rides ({:.2} commits/sync)",
        s.commits,
        s.syncs,
        s.free_rides,
        s.commits_per_sync()
    );

    assert!(
        s.syncs < s.commits,
        "no batching through the engine: {} fsyncs for {} commits",
        s.syncs,
        s.commits
    );
    assert!(
        s.free_rides > 0,
        "no commit was made durable by another thread's fsync"
    );

    // Correctness is not negotiable for a throughput win: every row must be there.
    for t in 0..THREADS as u64 {
        for i in 1..=PER_THREAD {
            let pk = t * 1_000 + i;
            assert!(
                engine.get(pk).unwrap().is_some(),
                "pk {pk} lost under concurrent durable writes"
            );
        }
    }
}

#[test]
fn single_writer_gets_no_batching_and_that_is_expected() {
    // Documents the limit in executable form, so nobody reads "group commit" as
    // "durable writes are now fast".
    let dir = TempDir::new().unwrap();
    let engine =
        TableEngine::open_with(schema(), dir.path(), EngineOptions::new(1_000_000)).unwrap();
    for i in 1..=10u64 {
        engine.insert(i, vec![i, i]).unwrap();
    }
    let s = engine.commit_stats();
    assert_eq!(
        s.syncs, s.commits,
        "a lone writer should pay one fsync per commit; got {} syncs for {} commits",
        s.syncs, s.commits
    );
    assert_eq!(s.free_rides, 0);
}
