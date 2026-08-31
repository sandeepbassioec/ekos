//! Integration tests for L111: `vvctl catalog diff` (VV-CTL-007).

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

fn export_to(dir: &TempDir, out: PathBuf) -> CatalogArgs {
    args(dir, CatalogCommand::Export { output: Some(out) })
}

fn diff_files(dir: &TempDir, left: PathBuf, right: PathBuf) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::Diff {
            left: Some(left),
            live: false,
            right,
        },
    )
}

fn diff_live(dir: &TempDir, right: PathBuf) -> CatalogArgs {
    args(
        dir,
        CatalogCommand::Diff {
            left: None,
            live: true,
            right,
        },
    )
}

// â”€â”€ identical catalogs â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn diff_identical_exports_succeeds() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    let out = dir.path().join("snap.json");
    run(export_to(&dir, out.clone())).unwrap();
    // diff same file against itself
    run(diff_files(&dir, out.clone(), out)).unwrap();
}

#[test]
fn diff_both_empty_exports_succeeds() {
    let src = make_dir();
    let out = src.path().join("empty.json");
    run(export_to(&src, out.clone())).unwrap();

    let dir = make_dir();
    run(diff_files(&dir, out.clone(), out)).unwrap();
}

// â”€â”€ added tables â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn diff_detects_added_table() {
    use vectorvolt_engine::{CatalogExport, diff_catalogs};

    let left = CatalogExport {
        version: 1,
        tables: vec![],
    };
    let right = CatalogExport {
        version: 1,
        tables: vec![vectorvolt_engine::ExportedTable {
            name: "new_table".into(),
            columns: vec!["pk:u64".into()],
        }],
    };
    let diff = diff_catalogs(&left, &right);
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.added[0].name, "new_table");
    assert!(diff.removed.is_empty());
    assert!(diff.changed.is_empty());
}

// â”€â”€ removed tables â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn diff_detects_removed_table() {
    use vectorvolt_engine::{CatalogExport, ExportedTable, diff_catalogs};

    let left = CatalogExport {
        version: 1,
        tables: vec![ExportedTable {
            name: "gone".into(),
            columns: vec!["pk:u64".into()],
        }],
    };
    let right = CatalogExport {
        version: 1,
        tables: vec![],
    };
    let diff = diff_catalogs(&left, &right);
    assert_eq!(diff.removed.len(), 1);
    assert_eq!(diff.removed[0].name, "gone");
}

// â”€â”€ changed tables â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn diff_detects_column_type_change() {
    use vectorvolt_engine::{CatalogExport, ColumnChangeKind, ExportedTable, diff_catalogs};

    let make = |col_type: &str| CatalogExport {
        version: 1,
        tables: vec![ExportedTable {
            name: "metrics".into(),
            columns: vec!["pk:u64".into(), format!("val:{col_type}")],
        }],
    };

    let diff = diff_catalogs(&make("u64"), &make("f64"));
    assert_eq!(diff.changed.len(), 1);
    assert!(matches!(
        &diff.changed[0].changes[0].kind,
        ColumnChangeKind::TypeChanged { from, to, .. } if from == "u64" && to == "f64"
    ));
}

#[test]
fn diff_detects_column_added_to_table() {
    use vectorvolt_engine::{CatalogExport, ColumnChangeKind, ExportedTable, diff_catalogs};

    let left = CatalogExport {
        version: 1,
        tables: vec![ExportedTable {
            name: "t".into(),
            columns: vec!["pk:u64".into()],
        }],
    };
    let right = CatalogExport {
        version: 1,
        tables: vec![ExportedTable {
            name: "t".into(),
            columns: vec!["pk:u64".into(), "extra:f64".into()],
        }],
    };
    let diff = diff_catalogs(&left, &right);
    assert!(matches!(
        &diff.changed[0].changes[0].kind,
        ColumnChangeKind::Added { name, .. } if name == "extra"
    ));
}

#[test]
fn diff_detects_column_removed_from_table() {
    use vectorvolt_engine::{CatalogExport, ColumnChangeKind, ExportedTable, diff_catalogs};

    let left = CatalogExport {
        version: 1,
        tables: vec![ExportedTable {
            name: "t".into(),
            columns: vec!["pk:u64".into(), "extra:f64".into()],
        }],
    };
    let right = CatalogExport {
        version: 1,
        tables: vec![ExportedTable {
            name: "t".into(),
            columns: vec!["pk:u64".into()],
        }],
    };
    let diff = diff_catalogs(&left, &right);
    assert!(matches!(
        &diff.changed[0].changes[0].kind,
        ColumnChangeKind::Removed { name, .. } if name == "extra"
    ));
}

#[test]
fn diff_detects_column_rename() {
    use vectorvolt_engine::{CatalogExport, ColumnChangeKind, ExportedTable, diff_catalogs};

    let left = CatalogExport {
        version: 1,
        tables: vec![ExportedTable {
            name: "t".into(),
            columns: vec!["pk:u64".into(), "old:f64".into()],
        }],
    };
    let right = CatalogExport {
        version: 1,
        tables: vec![ExportedTable {
            name: "t".into(),
            columns: vec!["pk:u64".into(), "new:f64".into()],
        }],
    };
    let diff = diff_catalogs(&left, &right);
    assert!(matches!(
        &diff.changed[0].changes[0].kind,
        ColumnChangeKind::Renamed { from, to, .. } if from == "old" && to == "new"
    ));
}

// â”€â”€ CLI subcommand â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[test]
fn diff_file_vs_file_cli_succeeds() {
    let src = make_dir();
    run(create_args(&src, "a")).unwrap();
    let snap1 = src.path().join("v1.json");
    run(export_to(&src, snap1.clone())).unwrap();

    run(create_args(&src, "b")).unwrap();
    let snap2 = src.path().join("v2.json");
    run(export_to(&src, snap2.clone())).unwrap();

    let dir = make_dir();
    run(diff_files(&dir, snap1, snap2)).unwrap();
}

#[test]
fn diff_live_vs_file_cli_succeeds() {
    let dir = make_dir();
    run(create_args(&dir, "orders")).unwrap();
    let snap = dir.path().join("snap.json");
    run(export_to(&dir, snap.clone())).unwrap();

    // Add another table so live != snap
    run(create_args(&dir, "shipments")).unwrap();

    run(diff_live(&dir, snap)).unwrap();
}

#[test]
fn diff_missing_right_file_returns_error() {
    let dir = make_dir();
    let out = dir.path().join("snap.json");
    run(export_to(&dir, out.clone())).unwrap();

    let result = run(diff_files(&dir, out, dir.path().join("nonexistent.json")));
    assert!(result.is_err());
}

#[test]
fn diff_malformed_right_file_returns_error() {
    let dir = make_dir();
    let good = dir.path().join("good.json");
    run(export_to(&dir, good.clone())).unwrap();
    let bad = dir.path().join("bad.json");
    std::fs::write(&bad, b"{{not json}}").unwrap();
    let result = run(diff_files(&dir, good, bad));
    assert!(result.is_err());
}

#[test]
fn diff_is_empty_produces_identical_message() {
    // engine-level check: is_empty() on no-diff
    use vectorvolt_engine::{CatalogExport, diff_catalogs};
    let e = CatalogExport {
        version: 1,
        tables: vec![],
    };
    let d = diff_catalogs(&e, &e);
    assert!(d.is_empty());
    assert_eq!(d.change_count(), 0);
}
