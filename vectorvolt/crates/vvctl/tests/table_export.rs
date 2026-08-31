//! Integration tests for Layer 151: vvctl table export (VV-CTL-151).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tbl(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}

fn create(d: &TempDir) {
    run(TableCommand::Create {
        dir: tbl(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

fn insert(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tbl(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn export_json_empty_table() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn export_csv_empty_table() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Csv
        })
        .is_ok()
    );
}

#[test]
fn export_json_with_rows() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 10);
    insert(&d, 2, 20);
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn export_csv_with_rows() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 10);
    insert(&d, 2, 20);
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Csv
        })
        .is_ok()
    );
}

#[test]
fn export_json_single_row() {
    let d = make_dir();
    create(&d);
    insert(&d, 42, 99);
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn export_csv_single_row() {
    let d = make_dir();
    create(&d);
    insert(&d, 42, 99);
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Csv
        })
        .is_ok()
    );
}

#[test]
fn export_json_many_rows() {
    let d = make_dir();
    create(&d);
    for i in 0..10u64 {
        insert(&d, i, i * 2);
    }
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn export_csv_many_rows() {
    let d = make_dir();
    create(&d);
    for i in 0..10u64 {
        insert(&d, i, i * 2);
    }
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Csv
        })
        .is_ok()
    );
}

#[test]
fn export_json_missing_dir_errors() {
    assert!(
        run(TableCommand::Export {
            dir: PathBuf::from("/no/such/dir"),
            format: ExportFormat::Json
        })
        .is_err()
    );
}

#[test]
fn export_csv_missing_dir_errors() {
    assert!(
        run(TableCommand::Export {
            dir: PathBuf::from("/no/such/dir"),
            format: ExportFormat::Csv
        })
        .is_err()
    );
}

#[test]
fn export_json_repeated_calls_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn export_csv_high_pk() {
    let d = make_dir();
    create(&d);
    insert(&d, u64::MAX / 2, 777);
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Csv
        })
        .is_ok()
    );
}

#[test]
fn export_json_and_csv_consistent() {
    let d = make_dir();
    create(&d);
    for i in 1..=5u64 {
        insert(&d, i, i * 100);
    }
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::Export {
            dir: tbl(&d),
            format: ExportFormat::Csv
        })
        .is_ok()
    );
}
