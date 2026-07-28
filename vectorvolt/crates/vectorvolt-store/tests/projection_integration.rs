//! ProjectionDispatcher integration tests.

use bytes::Bytes;
use tempfile::TempDir;
use vectorvolt_store::{
    FilterProjection, IdentityProjection, KeyOnlyProjection, KvStore, PipelineProjection,
    ProjectionDispatcher, ScanEntry, ScanRequest, StoreConfig,
};

fn cfg(dir: &std::path::Path) -> StoreConfig {
    StoreConfig::new(dir)
        .with_flush_threshold(256)
        .with_compaction_trigger(4)
        .with_fsync(false)
}

fn open(dir: &std::path::Path) -> ProjectionDispatcher {
    ProjectionDispatcher::new(KvStore::open(cfg(dir)).unwrap())
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn seed(dir: &std::path::Path, pairs: &[(&str, &str)]) {
    let store = KvStore::open(cfg(dir)).unwrap();
    for (k, v) in pairs {
        store
            .put(Bytes::from(k.to_string()), Bytes::from(v.to_string()))
            .unwrap();
    }
    store.flush().unwrap();
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn full_scan_returns_all_live_entries() {
    let dir = TempDir::new().unwrap();
    seed(dir.path(), &[("a", "1"), ("b", "2"), ("c", "3")]);

    let pd = open(dir.path());
    let results = pd.scan(ScanRequest::full()).unwrap();

    let keys: Vec<_> = results.iter().map(|e| e.key.clone()).collect();
    assert_eq!(keys, vec!["a", "b", "c"]);
}

#[test]
fn prefix_scan() {
    let dir = TempDir::new().unwrap();
    seed(
        dir.path(),
        &[
            ("user:alice", "30"),
            ("user:bob", "25"),
            ("order:1", "100"),
            ("order:2", "200"),
        ],
    );

    let pd = open(dir.path());
    let results = pd.scan_prefix("user:").unwrap();

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.key.starts_with(b"user:")));
}

#[test]
fn prefix_scan_with_limit() {
    let dir = TempDir::new().unwrap();
    seed(
        dir.path(),
        &[
            ("tag:a", "1"),
            ("tag:b", "2"),
            ("tag:c", "3"),
            ("tag:d", "4"),
        ],
    );

    let pd = open(dir.path());
    let results = pd.scan_prefix_limit("tag:", 2).unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn tombstone_excluded_from_scan() {
    let dir = TempDir::new().unwrap();
    let store = KvStore::open(cfg(dir.path())).unwrap();
    store.put("alive", "yes").unwrap();
    store.put("dead", "no").unwrap();
    store.delete("dead").unwrap();
    store.flush().unwrap();

    let pd = ProjectionDispatcher::new(store);
    let results = pd.scan(ScanRequest::full()).unwrap();

    let keys: Vec<_> = results.iter().map(|e| e.key.clone()).collect();
    assert!(keys.contains(&Bytes::from("alive")));
    assert!(!keys.contains(&Bytes::from("dead")));
}

#[test]
fn filter_projection() {
    let dir = TempDir::new().unwrap();
    seed(
        dir.path(),
        &[
            ("score:100", "alice"),
            ("score:050", "bob"),
            ("score:200", "charlie"),
        ],
    );

    let pd = open(dir.path());
    // Keep only entries whose value is "alice" or "charlie"
    let p = FilterProjection(|e: &ScanEntry| {
        e.value == "alice".as_bytes() || e.value == "charlie".as_bytes()
    });
    let results = pd.scan_with(ScanRequest::full(), &p).unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn key_only_projection() {
    let dir = TempDir::new().unwrap();
    seed(dir.path(), &[("k1", "big-value-1"), ("k2", "big-value-2")]);

    let pd = open(dir.path());
    let results = pd
        .scan_with(ScanRequest::full(), &KeyOnlyProjection)
        .unwrap();
    assert!(results.iter().all(|e| e.value.is_empty()));
    assert_eq!(results.len(), 2);
}

#[test]
fn count() {
    let dir = TempDir::new().unwrap();
    seed(
        dir.path(),
        &[("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")],
    );

    let pd = open(dir.path());
    assert_eq!(pd.count(ScanRequest::full()).unwrap(), 4);
    assert_eq!(pd.count(ScanRequest::prefix("a")).unwrap(), 1);
}

#[test]
fn keys_only() {
    let dir = TempDir::new().unwrap();
    seed(dir.path(), &[("x", "1"), ("y", "2"), ("z", "3")]);

    let pd = open(dir.path());
    let keys = pd.keys(ScanRequest::full()).unwrap();
    assert_eq!(keys, vec!["x", "y", "z"]);
}

#[test]
fn pipeline_projection() {
    let dir = TempDir::new().unwrap();
    seed(dir.path(), &[("keep:a", "value"), ("skip:b", "value")]);

    let pd = open(dir.path());
    let p = PipelineProjection(vec![
        Box::new(FilterProjection(|e: &ScanEntry| {
            e.key.starts_with(b"keep:")
        })),
        Box::new(IdentityProjection),
    ]);
    let results = pd.scan_with(ScanRequest::full(), &p).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, Bytes::from("keep:a"));
}

#[test]
fn scan_across_memtable_and_sstables() {
    let dir = TempDir::new().unwrap();
    let store = KvStore::open(cfg(dir.path())).unwrap();

    // First batch → flush to SSTable
    for i in 0u32..5 {
        store.put(format!("k{i:02}"), format!("v{i}")).unwrap();
    }
    store.flush().unwrap();

    // Second batch → stays in memtable
    for i in 5u32..10 {
        store.put(format!("k{i:02}"), format!("v{i}")).unwrap();
    }

    let pd = ProjectionDispatcher::new(store);
    let results = pd.scan(ScanRequest::full()).unwrap();

    // All 10 entries should be visible
    assert_eq!(results.len(), 10);
    // Keys should be in sorted order
    let keys: Vec<_> = results.iter().map(|e| e.key.clone()).collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
}

#[test]
fn memtable_overwrite_beats_sstable() {
    let dir = TempDir::new().unwrap();
    let store = KvStore::open(cfg(dir.path())).unwrap();

    store.put("key", "old").unwrap();
    store.flush().unwrap(); // "old" goes to SSTable

    store.put("key", "new").unwrap(); // "new" stays in memtable

    let pd = ProjectionDispatcher::new(store);
    let results = pd.scan(ScanRequest::full()).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, Bytes::from("new"));
}
