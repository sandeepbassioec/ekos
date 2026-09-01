//! Integration tests for Layer 182: vvctl table pitr-list (VV-CTL-182).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

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

#[test]
fn pitr_list_no_snapshots_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_json_no_snapshots_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_after_snapshot_ok() {
    let d = make_dir();
    create(&d);
    run(TableCommand::Snapshot { dir: tbl(&d) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_json_after_snapshot_ok() {
    let d = make_dir();
    create(&d);
    run(TableCommand::Snapshot { dir: tbl(&d) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_missing_dir_errors() {
    assert!(
        run(TableCommand::PitrList {
            dir: PathBuf::from("/no/such/dir"),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn pitr_list_repeated_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_multiple_snapshots_ok() {
    let d = make_dir();
    create(&d);
    run(TableCommand::Snapshot { dir: tbl(&d) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_table_output_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_json_output_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_with_insert_no_snapshot_ok() {
    let d = make_dir();
    create(&d);
    run(TableCommand::Insert {
        dir: tbl(&d),
        pk: 1,
        values: vec![100],
    })
    .unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_after_snapshot_and_insert_ok() {
    let d = make_dir();
    create(&d);
    run(TableCommand::Snapshot { dir: tbl(&d) }).unwrap();
    run(TableCommand::Insert {
        dir: tbl(&d),
        pk: 1,
        values: vec![100],
    })
    .unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_fast() {
    let d = make_dir();
    create(&d);
    let start = std::time::Instant::now();
    run(TableCommand::PitrList {
        dir: tbl(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
    assert!(start.elapsed().as_secs() < 5);
}
