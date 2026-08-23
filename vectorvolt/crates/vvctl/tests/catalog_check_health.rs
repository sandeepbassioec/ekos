//! Integration tests for L113: `vvctl catalog check-health` (VV-CTL-009).

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

fn check_health(dir: &TempDir) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::CheckHealth {
            max_snapshot_age: None,
            baseline: None,
            strict: false,
        },
    )
}

fn check_health_with_max_age(dir: &TempDir, secs: u64) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::CheckHealth {
            max_snapshot_age: Some(secs),
            baseline: None,
            strict: false,
        },
    )
}

fn check_health_with_baseline(dir: &TempDir, baseline: PathBuf) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::CheckHealth {
            max_snapshot_age: None,
            baseline: Some(baseline),
            strict: false,
        },
    )
}

fn check_health_strict(dir: &TempDir) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::CheckHealth {
            max_snapshot_age: None,
            baseline: None,
            strict: true,
        },
    )
}

// ── basic ─────────────────────────────────────────────────────────────────────

#[test]
fn check_health_empty_catalog_passes() {
    let dir = make_dir();
    run(check_health(&dir)).unwrap();
}

#[test]
fn check_health_valid_table_passes() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    run(check_health(&dir)).unwrap();
}

#[test]
fn check_health_multiple_valid_tables_passes() {
    let dir = make_dir();
    run(create_args(&dir, "a")).unwrap();
    run(create_args(&dir, "b")).unwrap();
    run(check_health(&dir)).unwrap();
}

// ── orphan / ghost ────────────────────────────────────────────────────────────

#[test]
fn check_health_orphan_dir_fails() {
    let dir = make_dir();
    // orphan: catalog dir without registry entry
    std::fs::create_dir_all(dir.path().join("catalog").join("orphan")).unwrap();
    let result = run(check_health(&dir));
    assert!(
        result.is_err(),
        "orphan dir should cause health check failure"
    );
}

#[test]
fn check_health_ghost_entry_fails() {
    use vectorvolt_engine::{ColumnDef, SchemaRegistry, TableSchema};
    let dir = make_dir();
    let reg = SchemaRegistry::new(dir.path().join("registry"));
    let schema = TableSchema::new(vec![ColumnDef::u64("pk")]).unwrap();
    reg.save("ghost", &schema).unwrap();
    let result = run(check_health(&dir));
    assert!(
        result.is_err(),
        "ghost entry should cause health check failure"
    );
}

// ── snapshot freshness ────────────────────────────────────────────────────────

#[test]
fn check_health_no_max_age_passes_without_snapshot() {
    let dir = make_dir();
    run(create_args(&dir, "t")).unwrap();
    run(check_health(&dir)).unwrap();
}

#[test]
fn check_health_missing_snapshot_with_max_age_fails() {
    let dir = make_dir();
    run(create_args(&dir, "t")).unwrap();
    let result = run(check_health_with_max_age(&dir, 3600));
    assert!(
        result.is_err(),
        "missing snapshot should fail when max_age is set"
    );
}

#[test]
fn check_health_fresh_snapshot_within_max_age_passes() {
    let dir = make_dir();
    run(create_args(&dir, "t")).unwrap();
    run(args(
        &dir,
        CatalogCommand::SnapshotCreate { output_dir: None },
    ))
    .unwrap();
    run(check_health_with_max_age(&dir, 3600)).unwrap();
}

// ── baseline diff ─────────────────────────────────────────────────────────────

#[test]
fn check_health_baseline_match_passes() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();

    let snap = dir.path().join("baseline.json");
    run(args(
        &dir,
        CatalogCommand::Export {
            output: Some(snap.clone()),
        },
    ))
    .unwrap();

    run(check_health_with_baseline(&dir, snap)).unwrap();
}

#[test]
fn check_health_baseline_mismatch_fails() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();

    let snap = dir.path().join("baseline.json");
    run(args(
        &dir,
        CatalogCommand::Export {
            output: Some(snap.clone()),
        },
    ))
    .unwrap();

    // Add a table after capturing the baseline — live now differs.
    run(create_args(&dir, "shipments")).unwrap();

    let result = run(check_health_with_baseline(&dir, snap));
    assert!(
        result.is_err(),
        "catalog drift vs baseline should fail health check"
    );
}

// ── strict mode ───────────────────────────────────────────────────────────────

#[test]
fn check_health_strict_clean_catalog_passes() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    run(check_health_strict(&dir)).unwrap();
}

#[test]
fn check_health_strict_warning_fails() {
    use vectorvolt_engine::{ColumnDef, ShardCatalog, TableSchema};
    let dir = make_dir();
    // Non-u64 primary key triggers a warning in validate.
    let cat = dir.path().join("catalog");
    let reg_dir = dir.path().join("registry");
    let catalog = ShardCatalog::with_registry(&cat, &reg_dir).unwrap();
    let schema = TableSchema::new(vec![
        ColumnDef::from_line("ts:f64"), // non-u64 pk — warning
        ColumnDef::u64("val"),
    ])
    .unwrap();
    catalog.create_table("metrics", schema, 1024, 1).unwrap();

    // non-strict passes (warning only)
    run(check_health(&dir)).unwrap();
    // strict should fail
    let result = run(check_health_strict(&dir));
    assert!(result.is_err(), "strict mode should fail on warning");
}

// ── engine API ────────────────────────────────────────────────────────────────

#[test]
fn engine_health_report_snapshot_count() {
    use vectorvolt_engine::{
        ColumnDef, ShardCatalog, TableSchema, check_health as eng_check, save_snapshot,
    };

    let dir = TempDir::new().unwrap();
    let cat = dir.path().join("catalog");
    let reg = dir.path().join("registry");
    let snaps = dir.path().join("snapshots");

    let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
    let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("v")]).unwrap();
    catalog.create_table("t", schema, 1024, 1).unwrap();
    save_snapshot(&reg, &snaps).unwrap();

    let report = eng_check(&cat, &reg, &snaps, None, None).unwrap();
    assert_eq!(report.snapshot_count, 1);
    assert!(report.is_healthy());
}
