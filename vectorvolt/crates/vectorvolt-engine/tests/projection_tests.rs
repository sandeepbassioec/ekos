//! Projection integration tests (VV-PRJ-004).

use std::collections::HashMap;
use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableEngine};
use vectorvolt_projection::Reducer;
use vectorvolt_txn::Mutation;

struct LastValueReducer {
    name: String,
    rows: HashMap<u64, Vec<u64>>,
}

impl LastValueReducer {
    fn new(name: &str) -> Box<dyn Reducer> {
        Box::new(Self {
            name: name.to_string(),
            rows: HashMap::new(),
        })
    }
}

impl Reducer for LastValueReducer {
    fn name(&self) -> &str {
        &self.name
    }

    fn apply(&mut self, mutation: &Mutation) {
        match mutation {
            Mutation::Insert { pk, values } | Mutation::Upsert { pk, values } => {
                self.rows.insert(*pk, values.clone());
            }
            Mutation::Delete { pk } => {
                self.rows.remove(pk);
            }
            _ => {}
        }
    }

    fn snapshot(&self) -> Vec<(u64, Vec<u64>)> {
        let mut v: Vec<_> = self.rows.iter().map(|(&pk, v)| (pk, v.clone())).collect();
        v.sort_by_key(|(pk, _)| *pk);
        v
    }
}

fn open(dir: &TempDir) -> TableEngine {
    TableEngine::open_with(
        vec!["pk".into(), "val".into()],
        dir.path(),
        EngineOptions::new(1000).with_durability(Durability::Buffered),
    )
    .unwrap()
}

#[test]
fn no_projections_by_default() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    assert_eq!(eng.projection_count(), 0);
}

#[test]
fn attach_projection_registers() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(LastValueReducer::new("lv"));
    assert_eq!(eng.projection_count(), 1);
}

#[test]
fn detach_projection_removes() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(LastValueReducer::new("lv"));
    assert!(eng.detach_projection("lv"));
    assert_eq!(eng.projection_count(), 0);
}

#[test]
fn detach_missing_returns_false() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    assert!(!eng.detach_projection("nope"));
}

#[test]
fn insert_fans_to_projection() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(LastValueReducer::new("lv"));
    eng.insert(1, vec![1, 42]).unwrap();
    let snap = eng.query_projection("lv").unwrap();
    assert_eq!(snap.len(), 1);
    assert_eq!(snap[0], (1, vec![1, 42]));
}

#[test]
fn delete_fans_to_projection() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(LastValueReducer::new("lv"));
    eng.insert(1, vec![1, 10]).unwrap();
    eng.delete(1).unwrap();
    let snap = eng.query_projection("lv").unwrap();
    assert!(snap.is_empty());
}

#[test]
fn query_projection_unknown_name_is_none() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    assert!(eng.query_projection("missing").is_none());
}

#[test]
fn multiple_projections_all_receive_mutations() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(LastValueReducer::new("p1"));
    eng.attach_projection(LastValueReducer::new("p2"));
    eng.insert(7, vec![7, 77]).unwrap();
    assert_eq!(eng.query_projection("p1").unwrap().len(), 1);
    assert_eq!(eng.query_projection("p2").unwrap().len(), 1);
}

#[test]
fn projection_names_sorted() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(LastValueReducer::new("z"));
    eng.attach_projection(LastValueReducer::new("a"));
    let names = eng.projection_names();
    assert_eq!(names, vec!["a", "z"]);
}

#[test]
fn multiple_inserts_all_visible() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(LastValueReducer::new("lv"));
    for i in 0..5u64 {
        eng.insert(i, vec![i, i * 2]).unwrap();
    }
    let snap = eng.query_projection("lv").unwrap();
    assert_eq!(snap.len(), 5);
}

#[test]
fn projection_survives_flush() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(LastValueReducer::new("lv"));
    for i in 0..3u64 {
        eng.insert(i, vec![i, i]).unwrap();
    }
    eng.flush().unwrap();
    eng.insert(99, vec![99, 99]).unwrap();
    let snap = eng.query_projection("lv").unwrap();
    assert_eq!(snap.len(), 4);
}

#[test]
fn detach_then_insert_not_seen() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.attach_projection(LastValueReducer::new("lv"));
    eng.insert(1, vec![1, 1]).unwrap();
    eng.detach_projection("lv");
    eng.attach_projection(LastValueReducer::new("lv"));
    // after re-attach: fresh reducer, previous insert not visible
    let snap = eng.query_projection("lv").unwrap();
    assert!(snap.is_empty());
}

#[test]
fn insert_before_attach_not_seen() {
    let dir = TempDir::new().unwrap();
    let eng = open(&dir);
    eng.insert(1, vec![1, 1]).unwrap();
    eng.attach_projection(LastValueReducer::new("lv"));
    let snap = eng.query_projection("lv").unwrap();
    assert!(
        snap.is_empty(),
        "projection only sees mutations after attach"
    );
}
