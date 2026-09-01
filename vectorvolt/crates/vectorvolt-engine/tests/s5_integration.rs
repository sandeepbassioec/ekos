//! Sprint 5 end-to-end integration tests: tiered storage + PITR + projections + sinks (VV-STO-020).

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_projection::Reducer;
use vectorvolt_sink::{Sink, SinkEvent, SinkRegistry, SinkResult};
use vectorvolt_txn::Mutation;

// ── shared helpers ────────────────────────────────────────────────────────────

fn open(dir: &TempDir) -> TableEngine {
    TableEngine::open_with(
        vec!["pk".into(), "val".into()],
        dir.path(),
        EngineOptions::new(1000).with_durability(Durability::Buffered),
    )
    .unwrap()
}

// ── projection helper ─────────────────────────────────────────────────────────

struct SumReducer {
    name: String,
    totals: HashMap<u64, u64>,
}

impl SumReducer {
    fn new(name: &str) -> Box<dyn Reducer> {
        Box::new(Self {
            name: name.to_string(),
            totals: HashMap::new(),
        })
    }
}

impl Reducer for SumReducer {
    fn name(&self) -> &str {
        &self.name
    }

    fn apply(&mut self, mutation: &Mutation) {
        match mutation {
            Mutation::Insert { pk, values } | Mutation::Upsert { pk, values } => {
                let sum: u64 = values.iter().sum();
                self.totals.insert(*pk, sum);
            }
            Mutation::Delete { pk } => {
                self.totals.remove(pk);
            }
            _ => {}
        }
    }

    fn snapshot(&self) -> Vec<(u64, Vec<u64>)> {
        self.totals.iter().map(|(&pk, &v)| (pk, vec![v])).collect()
    }
}

// ── sink helper ───────────────────────────────────────────────────────────────

struct CountSink {
    name: String,
    count: Arc<AtomicU64>,
}

#[async_trait]
impl Sink for CountSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, _event: SinkEvent) -> SinkResult<()> {
        self.count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn flush(&self) -> SinkResult<()> {
        Ok(())
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn insert_then_snapshot_has_files() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    for i in 0..5u64 {
        eng.insert(i, vec![i, i + 1]).unwrap();
    }
    let marker = eng.create_snapshot().unwrap();
    // snapshot_id is nonzero
    assert!(marker.snapshot_id > 0);
}

#[test]
fn snapshot_list_grows_with_each_snapshot() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.insert(1, vec![1, 2]).unwrap();
    eng.create_snapshot().unwrap();
    eng.insert(2, vec![2, 3]).unwrap();
    eng.create_snapshot().unwrap();
    let markers = eng.list_snapshots().unwrap();
    assert!(markers.len() >= 2);
}

#[test]
fn restore_from_snapshot_zero_opens_engine() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.insert(10, vec![10, 20]).unwrap();
    // restore with snapshot_id=0 is the "open from scratch" path
    let restored =
        TableEngine::restore_from_snapshot(vec!["pk".into(), "val".into()], dir.path(), 0).unwrap();
    // Engine should be usable after restore
    restored.insert(99, vec![99, 0]).unwrap();
}

#[test]
fn projection_attached_sees_inserts() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(SumReducer::new("sums"));
    eng.insert(1, vec![10, 20]).unwrap();
    eng.insert(2, vec![5, 5]).unwrap();
    let snap = eng.query_projection("sums").unwrap();
    assert!(snap.len() >= 2);
}

#[test]
fn projection_detach_stops_updates() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(SumReducer::new("sums"));
    eng.insert(1, vec![1, 2]).unwrap();
    eng.detach_projection("sums");
    eng.insert(2, vec![5, 5]).unwrap();
    // after detach, projection is gone
    assert!(eng.query_projection("sums").is_none());
}

#[test]
fn projection_names_returns_registered_names() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(SumReducer::new("p1"));
    eng.attach_projection(SumReducer::new("p2"));
    let names = eng.projection_names();
    assert!(names.contains(&"p1".to_string()));
    assert!(names.contains(&"p2".to_string()));
}

#[test]
fn delete_removes_projection_row() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(SumReducer::new("sums"));
    eng.insert(7, vec![7, 7]).unwrap();
    eng.delete(7).unwrap();
    let snap = eng.query_projection("sums").unwrap();
    assert!(!snap.iter().any(|(pk, _)| *pk == 7));
}

#[tokio::test]
async fn sink_registry_fan_out_insert() {
    let count = Arc::new(AtomicU64::new(0));
    let sink = Arc::new(CountSink {
        name: "c".into(),
        count: count.clone(),
    });
    let reg = SinkRegistry::new();
    reg.register(sink);
    reg.fan_out(SinkEvent::insert("orders", 1, vec![10]))
        .await
        .unwrap();
    assert_eq!(count.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn sink_registry_fan_out_delete() {
    let count = Arc::new(AtomicU64::new(0));
    let sink = Arc::new(CountSink {
        name: "c".into(),
        count: count.clone(),
    });
    let reg = SinkRegistry::new();
    reg.register(sink);
    reg.fan_out(SinkEvent::delete("orders", 1)).await.unwrap();
    assert_eq!(count.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn sink_multiple_sinks_each_receive_event() {
    let c1 = Arc::new(AtomicU64::new(0));
    let c2 = Arc::new(AtomicU64::new(0));
    let reg = SinkRegistry::new();
    reg.register(Arc::new(CountSink {
        name: "a".into(),
        count: c1.clone(),
    }));
    reg.register(Arc::new(CountSink {
        name: "b".into(),
        count: c2.clone(),
    }));
    reg.fan_out(SinkEvent::insert("t", 1, vec![1]))
        .await
        .unwrap();
    assert_eq!(c1.load(Ordering::Relaxed), 1);
    assert_eq!(c2.load(Ordering::Relaxed), 1);
}

#[test]
fn pitr_snapshot_id_unique_per_call() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.insert(1, vec![1, 1]).unwrap();
    let m1 = eng.create_snapshot().unwrap();
    eng.insert(2, vec![2, 2]).unwrap();
    let m2 = eng.create_snapshot().unwrap();
    // Each snapshot gets a different ID
    assert_ne!(m1.snapshot_id, m2.snapshot_id);
}

#[test]
fn projection_count_matches_attached() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    assert_eq!(eng.projection_count(), 0);
    eng.attach_projection(SumReducer::new("x"));
    assert_eq!(eng.projection_count(), 1);
}

#[test]
fn snapshot_then_insert_new_data_accessible() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.insert(1, vec![1, 10]).unwrap();
    eng.create_snapshot().unwrap();
    eng.insert(2, vec![2, 20]).unwrap();
    let row = eng.get(2).unwrap();
    assert!(row.is_some());
}
