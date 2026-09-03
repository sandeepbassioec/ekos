//! Tests for Layer 377.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn javasdk_create_ok() {
    let d = make_dir();
    assert!(
        run(TableCommand::Create {
            dir: tdir(&d),
            columns: vec!["pk:u64".into(), "val:u64".into()]
        })
        .is_ok()
    );
}
#[test]
fn javasdk_insert_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: 1,
            values: vec![10]
        })
        .is_ok()
    );
}
#[test]
fn javasdk_get_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn javasdk_scan_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn javasdk_delete_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: 1
        })
        .is_ok()
    );
}
#[test]
fn javasdk_update_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            val: 99
        })
        .is_ok()
    );
}
#[test]
fn javasdk_count_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn javasdk_schema_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Schema {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn javasdk_export_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn javasdk_upsert_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![99]
        })
        .is_ok()
    );
}
#[test]
fn javasdk_cas_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            expected: 10,
            replacement: 20
        })
        .is_ok()
    );
}
#[test]
fn javasdk_incr_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 1,
            col: 1
        })
        .is_ok()
    );
}
#[test]
fn javasdk_decr_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::DecrementField {
            dir: tdir(&d),
            pk: 1,
            col: 1
        })
        .is_ok()
    );
}
