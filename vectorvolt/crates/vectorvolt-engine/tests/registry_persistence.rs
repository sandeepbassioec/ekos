//! Phase 6: TableRegistry catalog persistence tests.
//!
//! Each test exercises the write-schema-on-create / scan-on-open / remove-schema-on-drop
//! invariants without going through the network layer.

use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, SCHEMA_FILE, TableRegistry};

fn opts() -> EngineOptions {
    EngineOptions::default()
}

fn schema(cols: &[&str]) -> Vec<String> {
    cols.iter().map(|s| s.to_string()).collect()
}

// ── schema.txt written on create ─────────────────────────────────────────────

/// create_table writes schema.txt into the table's subdirectory.
#[test]
fn create_table_writes_schema_file() {
    let tmp = TempDir::new().unwrap();
    let reg = TableRegistry::new(tmp.path(), opts());
    reg.create_table("users", schema(&["id", "age"])).unwrap();

    let schema_path = tmp.path().join("users").join(SCHEMA_FILE);
    assert!(schema_path.exists(), "schema.txt not written");
    let content = std::fs::read_to_string(&schema_path).unwrap();
    let cols: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(cols, vec!["id:u64", "age:u64"]);
}

// ── open() restores tables ────────────────────────────────────────────────────

/// open() picks up a table created by create_table in a previous registry instance.
#[test]
fn open_restores_created_table() {
    let tmp = TempDir::new().unwrap();

    {
        let reg = TableRegistry::new(tmp.path(), opts());
        reg.create_table("orders", schema(&["id", "amount"]))
            .unwrap();
    }

    let reg2 = TableRegistry::open(tmp.path(), opts()).unwrap();
    assert!(reg2.get("orders").is_ok(), "orders not restored");
}

/// open() restores the correct column schema so the engine can be used.
#[test]
fn open_restores_correct_schema() {
    let tmp = TempDir::new().unwrap();

    {
        let reg = TableRegistry::new(tmp.path(), opts());
        let e = reg.create_table("t", schema(&["pk", "x", "y"])).unwrap();
        e.insert(1, vec![1, 10, 20]).unwrap();
        e.flush().unwrap();
    }

    let reg2 = TableRegistry::open(tmp.path(), opts()).unwrap();
    let e2 = reg2.get("t").unwrap();
    let row = e2.get(1).unwrap().unwrap();
    assert_eq!(row.values, vec![1, 10, 20]);
}

/// open() restores multiple tables at once.
#[test]
fn open_restores_multiple_tables() {
    let tmp = TempDir::new().unwrap();

    {
        let reg = TableRegistry::new(tmp.path(), opts());
        reg.create_table("alpha", schema(&["id", "v"])).unwrap();
        reg.create_table("beta", schema(&["id", "w"])).unwrap();
    }

    let reg2 = TableRegistry::open(tmp.path(), opts()).unwrap();
    let mut names = reg2.list_tables();
    names.sort();
    assert_eq!(names, vec!["alpha", "beta"]);
}

/// open() on an empty directory returns an empty registry without error.
#[test]
fn open_empty_dir_succeeds() {
    let tmp = TempDir::new().unwrap();
    let reg = TableRegistry::open(tmp.path(), opts()).unwrap();
    assert!(reg.list_tables().is_empty());
}

/// open() on a non-existent path also succeeds (creates no dir, just empty registry).
#[test]
fn open_nonexistent_path_succeeds() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("does_not_exist");
    let reg = TableRegistry::open(&path, opts()).unwrap();
    assert!(reg.list_tables().is_empty());
}

/// open() ignores subdirectories that have no schema.txt (orphaned data dirs).
#[test]
fn open_skips_dirs_without_schema() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join("orphan")).unwrap();

    let reg = TableRegistry::open(tmp.path(), opts()).unwrap();
    assert!(reg.list_tables().is_empty());
}

/// open() ignores files (non-directories) at the top level.
#[test]
fn open_skips_top_level_files() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("README.txt"), "not a table").unwrap();

    let reg = TableRegistry::open(tmp.path(), opts()).unwrap();
    assert!(reg.list_tables().is_empty());
}

// ── drop_table removes schema.txt ────────────────────────────────────────────

/// drop_table removes schema.txt so open() won't restore the table.
#[test]
fn drop_table_removes_schema_file() {
    let tmp = TempDir::new().unwrap();

    let reg = TableRegistry::new(tmp.path(), opts());
    reg.create_table("temp", schema(&["id"])).unwrap();
    reg.drop_table("temp").unwrap();

    let schema_path = tmp.path().join("temp").join(SCHEMA_FILE);
    assert!(!schema_path.exists(), "schema.txt still present after drop");
}

/// After drop_table, open() does NOT restore the dropped table.
#[test]
fn dropped_table_not_restored_on_open() {
    let tmp = TempDir::new().unwrap();

    {
        let reg = TableRegistry::new(tmp.path(), opts());
        reg.create_table("gone", schema(&["id"])).unwrap();
        reg.drop_table("gone").unwrap();
    }

    let reg2 = TableRegistry::open(tmp.path(), opts()).unwrap();
    assert!(reg2.get("gone").is_err(), "dropped table was restored");
}

/// drop_table leaves the data directory (only schema.txt is removed).
#[test]
fn drop_table_leaves_data_directory() {
    let tmp = TempDir::new().unwrap();

    let reg = TableRegistry::new(tmp.path(), opts());
    reg.create_table("keep_data", schema(&["id"])).unwrap();
    reg.drop_table("keep_data").unwrap();

    assert!(
        tmp.path().join("keep_data").exists(),
        "data dir was deleted"
    );
}

// ── round-trip: create, insert, restart, read ────────────────────────────────

/// Full round-trip: insert rows, drop the registry, reopen, read rows back.
#[test]
fn roundtrip_insert_restart_read() {
    let tmp = TempDir::new().unwrap();

    {
        let reg = TableRegistry::new(tmp.path(), opts());
        let e = reg.create_table("rt", schema(&["id", "val"])).unwrap();
        e.insert(1, vec![1, 100]).unwrap();
        e.insert(2, vec![2, 200]).unwrap();
        e.flush().unwrap();
    }

    let reg2 = TableRegistry::open(tmp.path(), opts()).unwrap();
    let e2 = reg2.get("rt").unwrap();
    assert_eq!(e2.get(1).unwrap().unwrap().values, vec![1, 100]);
    assert_eq!(e2.get(2).unwrap().unwrap().values, vec![2, 200]);
}

/// create_table on a registry opened with open() works and persists correctly.
#[test]
fn create_table_on_reopened_registry() {
    let tmp = TempDir::new().unwrap();

    {
        let reg = TableRegistry::open(tmp.path(), opts()).unwrap();
        reg.create_table("first", schema(&["id"])).unwrap();
    }

    {
        let reg2 = TableRegistry::open(tmp.path(), opts()).unwrap();
        reg2.create_table("second", schema(&["id", "v"])).unwrap();
    }

    let reg3 = TableRegistry::open(tmp.path(), opts()).unwrap();
    let mut names = reg3.list_tables();
    names.sort();
    assert_eq!(names, vec!["first", "second"]);
}

/// If schema.txt cannot be removed, drop_table fails and the table stays
/// registered, instead of dropping it in memory and resurrecting it on restart.
#[test]
fn drop_table_failure_keeps_table_registered() {
    let tmp = TempDir::new().unwrap();
    let reg = TableRegistry::new(tmp.path(), opts());
    reg.create_table("users", schema(&["id"])).unwrap();

    // Replace schema.txt with a non-empty directory so remove_file fails.
    let schema_path = tmp.path().join("users").join(SCHEMA_FILE);
    std::fs::remove_file(&schema_path).unwrap();
    std::fs::create_dir(&schema_path).unwrap();
    std::fs::write(schema_path.join("blocker"), b"x").unwrap();

    assert!(reg.drop_table("users").is_err());
    assert!(reg.get("users").is_ok(), "table must stay registered");
}
