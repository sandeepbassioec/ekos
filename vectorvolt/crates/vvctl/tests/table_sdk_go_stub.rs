//! Tests for Layer 376.
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
fn gosdk_create_ok() {
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
fn gosdk_insert_ok() {
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
fn gosdk_get_ok() {
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
fn gosdk_scan_ok() {
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
fn gosdk_delete_ok() {
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
fn gosdk_update_ok() {
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
fn gosdk_count_ok() {
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
fn gosdk_schema_ok() {
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
fn gosdk_export_ok() {
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
fn gosdk_batch_get_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![1, 2, 3],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn gosdk_batch_delete_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![1, 2]
        })
        .is_ok()
    );
}
#[test]
fn gosdk_check_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn gosdk_stats_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
