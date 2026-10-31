//! End-to-end KvStore integration tests.

use bytes::Bytes;
use tempfile::TempDir;
use vectorvolt_store::{KvStore, StoreConfig};

fn cfg(dir: &std::path::Path) -> StoreConfig {
    StoreConfig::new(dir)
        .with_flush_threshold(512) // tiny threshold to force flushes quickly
        .with_compaction_trigger(4)
        .with_fsync(false)
}

#[test]
fn put_get_basic() {
    let dir = TempDir::new().unwrap();
    let store = KvStore::open(cfg(dir.path())).unwrap();

    store.put("hello", "world").unwrap();
    store.put("foo", "bar").unwrap();

    assert_eq!(store.get("hello").unwrap().unwrap(), Bytes::from("world"));
    assert_eq!(store.get("foo").unwrap().unwrap(), Bytes::from("bar"));
    assert!(store.get("missing").unwrap().is_none());
}

#[test]
fn overwrite() {
    let dir = TempDir::new().unwrap();
    let store = KvStore::open(cfg(dir.path())).unwrap();

    store.put("k", "v1").unwrap();
    store.put("k", "v2").unwrap();

    assert_eq!(store.get("k").unwrap().unwrap(), Bytes::from("v2"));
}

#[test]
fn delete_removes_key() {
    let dir = TempDir::new().unwrap();
    let store = KvStore::open(cfg(dir.path())).unwrap();

    store.put("del_me", "value").unwrap();
    assert!(store.get("del_me").unwrap().is_some());

    store.delete("del_me").unwrap();
    assert!(store.get("del_me").unwrap().is_none());
}

#[test]
fn flush_to_sstable_and_read_back() {
    let dir = TempDir::new().unwrap();
    let store = KvStore::open(cfg(dir.path())).unwrap();

    // Write enough to exceed the 512-byte flush threshold
    for i in 0u32..20 {
        store
            .put(
                format!("key-{i:04}"),
                format!("value-{i:04}-padding-to-exceed-threshold"),
            )
            .unwrap();
    }

    // Verify all keys readable
    for i in 0u32..20 {
        let got = store.get(format!("key-{i:04}")).unwrap();
        assert!(got.is_some(), "key-{i:04} should exist");
    }
}

#[test]
fn crash_recovery() {
    let dir = TempDir::new().unwrap();

    // Session 1: write and flush
    {
        let store = KvStore::open(cfg(dir.path())).unwrap();
        store.put("persistent", "yes").unwrap();
        store.put("ephemeral", "maybe").unwrap();
        store.flush().unwrap();
        // drop = simulated crash after flush
    }

    // Session 2: reopen — should find both keys
    {
        let store = KvStore::open(cfg(dir.path())).unwrap();
        assert_eq!(
            store.get("persistent").unwrap().unwrap(),
            Bytes::from("yes")
        );
        // ephemeral was flushed to SSTable so it must also survive
        assert_eq!(
            store.get("ephemeral").unwrap().unwrap(),
            Bytes::from("maybe")
        );
    }
}

#[test]
fn wal_replay_without_flush() {
    let dir = TempDir::new().unwrap();

    // Session 1: write but do NOT flush (simulates crash before memtable flush)
    {
        let store = KvStore::open(cfg(dir.path())).unwrap();
        store.put("wal_only", "recover_me").unwrap();
        // No flush — only in WAL
    }

    // Session 2: should recover from WAL replay
    {
        let store = KvStore::open(cfg(dir.path())).unwrap();
        assert_eq!(
            store.get("wal_only").unwrap().unwrap(),
            Bytes::from("recover_me")
        );
    }
}

#[test]
fn compaction_reduces_sstable_count() {
    let dir = TempDir::new().unwrap();
    let store = KvStore::open(cfg(dir.path())).unwrap();

    // Write enough to create ≥4 SSTables and trigger compaction
    for batch in 0u32..8 {
        for i in 0u32..5 {
            store
                .put(
                    format!("b{batch}-k{i}"),
                    "v".repeat(50), // 50 bytes × 5 = 250 bytes, exceeds 512 threshold per batch
                )
                .unwrap();
        }
    }
    store.flush().unwrap();

    let stats = store.stats();
    // After compaction, SSTable count should be ≤ compaction_trigger
    assert!(
        stats.sstable_count <= cfg(dir.path()).compaction_trigger,
        "expected ≤{} SSTables after compaction, got {}",
        cfg(dir.path()).compaction_trigger,
        stats.sstable_count
    );
}

// ── Durability (PP-14) ─────────────────────────────────────────────────────

fn count_files(dir: &std::path::Path, suffix: &str) -> usize {
    std::fs::read_dir(dir)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(suffix)
        })
        .count()
}

#[test]
fn put_and_delete_survive_crash_without_clean_close() {
    let dir = TempDir::new().unwrap();
    let cfg = || cfg(dir.path()).with_flush_threshold(1 << 20);

    let store = KvStore::open(cfg()).unwrap();
    store.put("kept", "v1").unwrap();
    store.put("gone", "v2").unwrap();
    store.delete("gone").unwrap();
    // Crash: no Drop runs, so nothing buffered in user space is written.
    std::mem::forget(store);

    let store = KvStore::open(cfg()).unwrap();
    assert_eq!(store.get("kept").unwrap().unwrap(), Bytes::from("v1"));
    assert!(store.get("gone").unwrap().is_none());
}

#[test]
fn flush_truncates_sealed_wal_segments() {
    let dir = TempDir::new().unwrap();
    let cfg = || {
        cfg(dir.path())
            .with_flush_threshold(1 << 20)
            .with_compaction_trigger(100)
            .with_wal_segment_max_bytes(1024)
    };

    let store = KvStore::open(cfg()).unwrap();
    for i in 0u32..200 {
        store
            .put(format!("k{:03}", i % 50), format!("v{i}"))
            .unwrap();
    }
    store.delete("k007").unwrap();
    assert!(count_files(dir.path(), ".vwal") > 1, "test needs rotation");
    store.flush().unwrap();
    assert_eq!(count_files(dir.path(), ".vwal"), 1);
    drop(store);

    // The kept (active) segment replays over the SSTable without
    // resurrecting older values.
    let store = KvStore::open(cfg()).unwrap();
    for i in 0u32..50 {
        let got = store.get(format!("k{i:03}")).unwrap();
        if i == 7 {
            assert!(got.is_none());
        } else {
            assert_eq!(got.unwrap(), Bytes::from(format!("v{}", 150 + i)));
        }
    }
}

// ── Failure-atomic flush / compaction ──────────────────────────────────────

#[test]
fn torn_sstable_from_crashed_flush_does_not_block_open() {
    use vectorvolt_store::memtable::MemEntry;
    use vectorvolt_store::sstable::{SsTableWriter, sstable_path};

    let dir = TempDir::new().unwrap();
    {
        let store = KvStore::open(cfg(dir.path())).unwrap();
        store.put("a", "1").unwrap();
        store.flush().unwrap();
    }
    // Crash mid-write: the writer never reaches `finish`.
    let mut w = SsTableWriter::create(sstable_path(dir.path(), 2)).unwrap();
    w.append(&MemEntry {
        key: Bytes::from("b"),
        value: Some(Bytes::from("2")),
    })
    .unwrap();
    drop(w);

    let store = KvStore::open(cfg(dir.path())).unwrap();
    assert_eq!(store.get("a").unwrap().unwrap(), Bytes::from("1"));
    assert_eq!(count_files(dir.path(), ".tmp"), 0);
}

#[test]
fn failed_compaction_leaves_store_readable() {
    let dir = TempDir::new().unwrap();
    let cfg = || {
        cfg(dir.path())
            .with_flush_threshold(1 << 20)
            .with_compaction_trigger(2)
    };

    let store = KvStore::open(cfg()).unwrap();
    store.put("a", "1").unwrap();
    store.put("doomed", "x").unwrap();
    store.flush().unwrap(); // SST 1
    store.put("b", "2").unwrap();
    store.delete("doomed").unwrap();

    // SST 2 is the flush; compaction output would be SST 3. Occupy its path
    // with a directory so the compaction fails.
    let blocker = dir.path().join("00000000000000000003.vvsst");
    std::fs::create_dir(&blocker).unwrap();
    assert!(store.flush().is_err());

    assert_eq!(store.stats().sstable_count, 2);
    assert_eq!(store.get("a").unwrap().unwrap(), Bytes::from("1"));
    assert_eq!(store.get("b").unwrap().unwrap(), Bytes::from("2"));
    assert!(store.get("doomed").unwrap().is_none());

    // Once the obstruction is gone the next compaction succeeds.
    std::fs::remove_dir(&blocker).unwrap();
    store.put("c", "3").unwrap();
    store.flush().unwrap();
    assert_eq!(store.stats().sstable_count, 1);
    drop(store);

    let store = KvStore::open(cfg()).unwrap();
    assert_eq!(store.get("a").unwrap().unwrap(), Bytes::from("1"));
    assert_eq!(store.get("b").unwrap().unwrap(), Bytes::from("2"));
    assert_eq!(store.get("c").unwrap().unwrap(), Bytes::from("3"));
    assert!(store.get("doomed").unwrap().is_none());
}
