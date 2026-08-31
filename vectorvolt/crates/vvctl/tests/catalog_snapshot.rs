//! Integration tests for L112: `vvctl catalog snapshot-*` (VV-CTL-008).

use std::path::PathBuf;

use tempfile::TempDir;
use vvctl::commands::catalog::{CatalogArgs, CatalogCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn args(dir: &TempDir, cmd: CatalogCommand) -> CatalogArgs {
    CatalogArgs {
        dir: dir.path().to_path_buf(),
        command: cmd,
    }
}

fn create_args(dir: &TempDir, name: &str) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::CreateTable {
            name: name.to_string(),
            columns: vec!["pk".to_string(), "val".to_string()],
            flush_threshold: 1024,
            partitions: 1,
        },
    )
}

fn snapshot_create(dir: &TempDir) -> CatalogArgs {
    args(dir, CatalogCommand::SnapshotCreate { output_dir: None })
}

fn snapshot_create_to(dir: &TempDir, out: PathBuf) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::SnapshotCreate {
            output_dir: Some(out),
        },
    )
}

fn snapshot_list(dir: &TempDir) -> CatalogArgs {
    args(dir, CatalogCommand::SnapshotList)
}

fn snapshot_restore(dir: &TempDir, file: PathBuf) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::SnapshotRestore {
            file,
            flush_threshold: 1024,
            partitions: 1,
            skip_existing: false,
        },
    )
}

fn snapshot_restore_skip(dir: &TempDir, file: PathBuf) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::SnapshotRestore {
            file,
            flush_threshold: 1024,
            partitions: 1,
            skip_existing: true,
        },
    )
}

// â”€â”€ snapshot-create â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn snapshot_create_empty_catalog_succeeds() {
    let dir = make_dir();
    run(snapshot_create(&dir)).unwrap();
}

#[test]
fn snapshot_create_single_table_writes_file() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();

    let out_dir = dir.path().join("my_snaps");
    run(snapshot_create_to(&dir, out_dir.clone())).unwrap();

    let files: Vec<_> = std::fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(files.len(), 1);
    assert!(
        files[0]
            .file_name()
            .to_string_lossy()
            .ends_with(".snapshot.json")
    );
}

#[test]
fn snapshot_create_file_is_valid_snapshot_json() {
    use vectorvolt_engine::CatalogSnapshot;

    let dir = make_dir();
    run(create_args(&dir, "events")).unwrap();
    let out_dir = dir.path().join("snaps");
    run(snapshot_create_to(&dir, out_dir.clone())).unwrap();

    let file = std::fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap();
    let content = std::fs::read_to_string(file.path()).unwrap();
    let snap = CatalogSnapshot::from_json(&content).unwrap();
    assert_eq!(snap.meta.version, 1);
    assert_eq!(snap.meta.table_count, 1);
    assert_eq!(snap.export.tables[0].name, "events");
}

#[test]
fn snapshot_create_multiple_tables_all_captured() {
    use vectorvolt_engine::CatalogSnapshot;

    let dir = make_dir();
    run(create_args(&dir, "a")).unwrap();
    run(create_args(&dir, "b")).unwrap();
    run(create_args(&dir, "c")).unwrap();
    let out_dir = dir.path().join("snaps");
    run(snapshot_create_to(&dir, out_dir.clone())).unwrap();

    let file = std::fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap();
    let content = std::fs::read_to_string(file.path()).unwrap();
    let snap = CatalogSnapshot::from_json(&content).unwrap();
    assert_eq!(snap.meta.table_count, 3);
    assert_eq!(snap.export.tables.len(), 3);
}

// â”€â”€ snapshot-list â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn snapshot_list_empty_dir_succeeds() {
    let dir = make_dir();
    run(snapshot_list(&dir)).unwrap();
}

#[test]
fn snapshot_list_shows_created_snapshot() {
    let dir = make_dir();
    run(create_args(&dir, "t")).unwrap();
    run(snapshot_create(&dir)).unwrap();

    let list = vectorvolt_engine::list_snapshots(dir.path().join("snapshots")).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].table_count, 1);
}

// â”€â”€ snapshot-restore â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn snapshot_restore_recreates_table_in_fresh_catalog() {
    let src = make_dir();
    run(create_args(&src, "orders")).unwrap();
    let out_dir = src.path().join("snaps");
    run(snapshot_create_to(&src, out_dir.clone())).unwrap();

    let snap_file = std::fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap()
        .path();

    let dst = make_dir();
    run(snapshot_restore(&dst, snap_file)).unwrap();

    let schema = vectorvolt_engine::SchemaRegistry::new(dst.path().join("registry"))
        .load("orders")
        .unwrap();
    assert_eq!(schema.col_count(), 2);
}

#[test]
fn snapshot_restore_multiple_tables_roundtrip() {
    let src = make_dir();
    run(create_args(&src, "x")).unwrap();
    run(create_args(&src, "y")).unwrap();
    let out_dir = src.path().join("snaps");
    run(snapshot_create_to(&src, out_dir.clone())).unwrap();

    let snap_file = std::fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap()
        .path();

    let dst = make_dir();
    run(snapshot_restore(&dst, snap_file)).unwrap();

    let reg = vectorvolt_engine::SchemaRegistry::new(dst.path().join("registry"));
    assert!(reg.load("x").is_ok());
    assert!(reg.load("y").is_ok());
}

#[test]
fn snapshot_restore_skip_existing_does_not_error() {
    let src = make_dir();
    run(create_args(&src, "users")).unwrap();
    let out_dir = src.path().join("snaps");
    run(snapshot_create_to(&src, out_dir.clone())).unwrap();

    let snap_file = std::fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap()
        .path();

    let dst = make_dir();
    run(snapshot_restore(&dst, snap_file.clone())).unwrap();
    run(snapshot_restore_skip(&dst, snap_file)).unwrap();
}

#[test]
fn snapshot_restore_conflict_without_skip_errors() {
    let src = make_dir();
    run(create_args(&src, "t")).unwrap();
    let out_dir = src.path().join("snaps");
    run(snapshot_create_to(&src, out_dir.clone())).unwrap();

    let snap_file = std::fs::read_dir(&out_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .next()
        .unwrap()
        .path();

    let dst = make_dir();
    run(snapshot_restore(&dst, snap_file.clone())).unwrap();
    let result = run(snapshot_restore(&dst, snap_file));
    assert!(
        result.is_err(),
        "second restore without skip_existing must error"
    );
}

#[test]
fn snapshot_restore_missing_file_errors() {
    let dst = make_dir();
    let result = run(snapshot_restore(
        &dst,
        dst.path().join("nonexistent.snapshot.json"),
    ));
    assert!(result.is_err());
}

#[test]
fn snapshot_restore_malformed_file_errors() {
    let dst = make_dir();
    let bad = dst.path().join("bad.snapshot.json");
    std::fs::write(&bad, b"not json {{{").unwrap();
    let result = run(snapshot_restore(&dst, bad));
    assert!(result.is_err());
}

#[test]
fn snapshot_create_default_dir_is_under_catalog_dir() {
    let dir = make_dir();
    run(create_args(&dir, "t")).unwrap();
    run(snapshot_create(&dir)).unwrap();
    // default snapshots dir is <dir>/snapshots/
    let snaps_dir = dir.path().join("snapshots");
    assert!(snaps_dir.exists(), "snapshots directory should be created");
    let count = std::fs::read_dir(&snaps_dir).unwrap().count();
    assert_eq!(count, 1);
}
