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
