//! Integration tests for L116: `vvctl catalog stats` (VV-CTL-012).

use tempfile::TempDir;
use vectorvolt_engine::{ColumnDef, ShardCatalog, TableSchema, catalog_stats};
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

fn stats(dir: &TempDir) -> CatalogArgs {
    args(dir, CatalogCommand::Stats { verbose: false })
}

fn stats_verbose(dir: &TempDir) -> CatalogArgs {
    args(dir, CatalogCommand::Stats { verbose: true })
}

fn engine_paths(dir: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    (dir.path().join("catalog"), dir.path().join("registry"))
}

fn make_table(dir: &TempDir, name: &str) {
    let (cat, reg) = engine_paths(dir);
    let catalog = ShardCatalog::with_registry(&cat, &reg).unwrap();
    let schema = TableSchema::new(vec![ColumnDef::u64("pk"), ColumnDef::u64("val")]).unwrap();
    catalog.create_table(name, schema, 1024, 1).unwrap();
}

// ── CLI integration ───────────────────────────────────────────────────────────

#[test]
fn stats_empty_catalog_exits_ok() {
    let dir = make_dir();
    run(stats(&dir)).unwrap();
}

#[test]
fn stats_single_table_exits_ok() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    run(stats(&dir)).unwrap();
}

#[test]
fn stats_multiple_tables_exits_ok() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    run(create_args(&dir, "shipments")).unwrap();
    run(stats(&dir)).unwrap();
}

#[test]
fn stats_verbose_exits_ok() {
    let dir = make_dir();
    run(create_args(&dir, "t")).unwrap();
    run(stats_verbose(&dir)).unwrap();
}

// ── engine API ────────────────────────────────────────────────────────────────

#[test]
fn engine_empty_catalog_returns_empty_report() {
    let dir = make_dir();
    let (cat, reg) = engine_paths(&dir);
    std::fs::create_dir_all(&cat).unwrap();
    let report = catalog_stats(&cat, &reg).unwrap();
    assert!(report.tables.is_empty());
}

#[test]
fn engine_single_table_in_report() {
    let dir = make_dir();
    make_table(&dir, "orders");
    let (cat, reg) = engine_paths(&dir);
    let report = catalog_stats(&cat, &reg).unwrap();
    assert_eq!(report.tables.len(), 1);
    assert_eq!(report.tables[0].name, "orders");
}

#[test]
fn engine_column_count_matches_schema() {
    let dir = make_dir();
    make_table(&dir, "orders");
    let (cat, reg) = engine_paths(&dir);
    let report = catalog_stats(&cat, &reg).unwrap();
    assert_eq!(report.tables[0].column_count, 2);
}

#[test]
fn engine_columns_list_format() {
    let dir = make_dir();
    make_table(&dir, "orders");
    let (cat, reg) = engine_paths(&dir);
    let report = catalog_stats(&cat, &reg).unwrap();
    for col in &report.tables[0].columns {
        assert!(col.contains(':'), "column must be 'name:type'");
    }
}

#[test]
fn engine_tables_sorted_alphabetically() {
    let dir = make_dir();
    make_table(&dir, "zzz");
    make_table(&dir, "aaa");
    let (cat, reg) = engine_paths(&dir);
    let report = catalog_stats(&cat, &reg).unwrap();
    assert_eq!(report.tables[0].name, "aaa");
    assert_eq!(report.tables[1].name, "zzz");
}

#[test]
fn engine_disk_bytes_nonzero() {
    let dir = make_dir();
    make_table(&dir, "t");
    let (cat, reg) = engine_paths(&dir);
    let report = catalog_stats(&cat, &reg).unwrap();
    assert!(report.tables[0].disk_bytes > 0);
}

#[test]
fn engine_total_disk_bytes_sums_tables() {
    let dir = make_dir();
    make_table(&dir, "a");
    make_table(&dir, "b");
    let (cat, reg) = engine_paths(&dir);
    let report = catalog_stats(&cat, &reg).unwrap();
    let manual_sum: u64 = report.tables.iter().map(|t| t.disk_bytes).sum();
    assert_eq!(report.total_disk_bytes(), manual_sum);
}

#[test]
fn engine_total_column_count_sums_tables() {
    let dir = make_dir();
    make_table(&dir, "a");
    make_table(&dir, "b");
    let (cat, reg) = engine_paths(&dir);
    let report = catalog_stats(&cat, &reg).unwrap();
    assert_eq!(report.total_column_count(), 4); // 2 cols × 2 tables
}

#[test]
fn engine_file_count_nonzero_for_existing_table() {
    let dir = make_dir();
    make_table(&dir, "t");
    let (cat, reg) = engine_paths(&dir);
    let report = catalog_stats(&cat, &reg).unwrap();
    assert!(report.tables[0].file_count > 0);
}
