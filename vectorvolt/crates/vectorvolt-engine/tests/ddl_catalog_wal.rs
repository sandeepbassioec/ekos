//! Layer 89 — DDL catalog WAL integration tests (VV-ENG-019).
//!
//! Verifies that `TableRegistry::enable_catalog_wal()` causes CREATE TABLE and
//! DROP TABLE events to be appended as `DdlCreate`/`DdlDrop` mutations to a
//! dedicated WAL, while leaving the normal table data path unaffected.

use tempfile::TempDir;
use vectorvolt_engine::{Durability, EngineOptions, TableRegistry};
use vectorvolt_txn::ops::Mutation;
use vectorvolt_wal::{WalConfig, WalReader};

fn opts() -> EngineOptions {
    EngineOptions::default().with_durability(Durability::WriteOnly)
}

fn schema(cols: &[&str]) -> Vec<String> {
    cols.iter().map(|s| s.to_string()).collect()
}

fn read_ddl_mutations(wal_dir: &std::path::Path) -> Vec<Mutation> {
    let cfg = WalConfig::default().with_dir(wal_dir).with_fsync(false);
    let reader = WalReader::open(cfg);
    let frames = reader.read_all().unwrap();
    frames
        .into_iter()
        .filter_map(|f| Mutation::decode(&f.payload).ok())
        .collect()
}

// ── enable_catalog_wal / has_catalog_wal ──────────────────────────────────────

#[test]
fn catalog_wal_disabled_by_default() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    assert!(!reg.has_catalog_wal());
}

#[test]
fn enable_catalog_wal_sets_flag() {
    let dir = TempDir::new().unwrap();
    let wal_dir = dir.path().join("_catalog_wal");
    let mut reg = TableRegistry::new(dir.path(), opts());
    reg.enable_catalog_wal(&wal_dir).unwrap();
    assert!(reg.has_catalog_wal());
}

// ── CREATE TABLE logs DdlCreate ───────────────────────────────────────────────

#[test]
fn create_table_appends_ddl_create_mutation() {
    let dir = TempDir::new().unwrap();
    let wal_dir = dir.path().join("_catalog_wal");
    let mut reg = TableRegistry::new(dir.path(), opts());
    reg.enable_catalog_wal(&wal_dir).unwrap();

    reg.create_table("orders", schema(&["id", "amount"]))
        .unwrap();

    let mutations = read_ddl_mutations(&wal_dir);
    assert_eq!(mutations.len(), 1);
    assert_eq!(
        mutations[0],
        Mutation::DdlCreate {
            table_name: "orders".into()
        }
    );
}

#[test]
fn multiple_creates_each_logged() {
    let dir = TempDir::new().unwrap();
    let wal_dir = dir.path().join("_catalog_wal");
    let mut reg = TableRegistry::new(dir.path(), opts());
    reg.enable_catalog_wal(&wal_dir).unwrap();

    reg.create_table("a", schema(&["id"])).unwrap();
    reg.create_table("b", schema(&["id"])).unwrap();
    reg.create_table("c", schema(&["id"])).unwrap();

    let mutations = read_ddl_mutations(&wal_dir);
    assert_eq!(mutations.len(), 3);
    assert!(mutations.iter().any(|m| m
        == &Mutation::DdlCreate {
            table_name: "a".into()
        }));
    assert!(mutations.iter().any(|m| m
        == &Mutation::DdlCreate {
            table_name: "c".into()
        }));
}

// ── DROP TABLE logs DdlDrop ───────────────────────────────────────────────────

#[test]
fn drop_table_appends_ddl_drop_mutation() {
    let dir = TempDir::new().unwrap();
    let wal_dir = dir.path().join("_catalog_wal");
    let mut reg = TableRegistry::new(dir.path(), opts());
    reg.enable_catalog_wal(&wal_dir).unwrap();

    reg.create_table("tmp", schema(&["id"])).unwrap();
    reg.drop_table("tmp").unwrap();

    let mutations = read_ddl_mutations(&wal_dir);
    assert_eq!(mutations.len(), 2);
    assert_eq!(
        mutations[1],
        Mutation::DdlDrop {
            table_name: "tmp".into()
        }
    );
}

#[test]
fn create_drop_create_sequence_logged_in_order() {
    let dir = TempDir::new().unwrap();
    let wal_dir = dir.path().join("_catalog_wal");
    let mut reg = TableRegistry::new(dir.path(), opts());
    reg.enable_catalog_wal(&wal_dir).unwrap();

    reg.create_table("tbl", schema(&["id"])).unwrap();
    reg.drop_table("tbl").unwrap();
    reg.create_table("tbl", schema(&["id"])).unwrap();

    let mutations = read_ddl_mutations(&wal_dir);
    assert_eq!(mutations.len(), 3);
    assert_eq!(
        mutations[0],
        Mutation::DdlCreate {
            table_name: "tbl".into()
        }
    );
    assert_eq!(
        mutations[1],
        Mutation::DdlDrop {
            table_name: "tbl".into()
        }
    );
    assert_eq!(
        mutations[2],
        Mutation::DdlCreate {
            table_name: "tbl".into()
        }
    );
}

// ── no DDL logged when WAL disabled ──────────────────────────────────────────

#[test]
fn create_without_catalog_wal_writes_nothing_to_wal_dir() {
    let dir = TempDir::new().unwrap();
    let reg = TableRegistry::new(dir.path(), opts());
    reg.create_table("no_wal", schema(&["id"])).unwrap();
    // No WAL dir was created → no WAL segments exist.
    let wal_dir = dir.path().join("_catalog_wal");
    assert!(!wal_dir.exists());
}

// ── mutation codec (ops.rs) round-trips ──────────────────────────────────────

#[test]
fn ddl_create_mutation_encodes_and_decodes() {
    let m = Mutation::DdlCreate {
        table_name: "hello_world".into(),
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn ddl_drop_mutation_encodes_and_decodes() {
    let m = Mutation::DdlDrop {
        table_name: "goodbye".into(),
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn ddl_mutation_with_empty_name_round_trips() {
    let m = Mutation::DdlCreate {
        table_name: String::new(),
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn ddl_mutation_with_unicode_name_round_trips() {
    let m = Mutation::DdlCreate {
        table_name: "héros".into(),
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn insert_decode_still_works_alongside_ddl_opcodes() {
    let m = Mutation::Insert {
        pk: 7,
        values: vec![1, 2, 3],
    };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

#[test]
fn delete_decode_still_works_alongside_ddl_opcodes() {
    let m = Mutation::Delete { pk: 42 };
    assert_eq!(Mutation::decode(&m.encode()).unwrap(), m);
}

// ── normal table operations unaffected ───────────────────────────────────────

#[test]
fn normal_insert_unaffected_by_catalog_wal() {
    let dir = TempDir::new().unwrap();
    let wal_dir = dir.path().join("_catalog_wal");
    let mut reg = TableRegistry::new(dir.path(), opts());
    reg.enable_catalog_wal(&wal_dir).unwrap();

    let engine = reg.create_table("data", schema(&["id", "val"])).unwrap();
    engine.insert(1, vec![1, 99]).unwrap();
    engine.insert(2, vec![2, 88]).unwrap();
    engine.flush().unwrap();

    let rows = engine.get(1).unwrap();
    assert!(rows.is_some());

    // Catalog WAL has one entry (CREATE), not the data inserts.
    let mutations = read_ddl_mutations(&wal_dir);
    assert_eq!(mutations.len(), 1);
    assert!(matches!(mutations[0], Mutation::DdlCreate { .. }));
}
