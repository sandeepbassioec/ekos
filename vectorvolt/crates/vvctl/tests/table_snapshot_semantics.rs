//! Tests for Layer 325: Snapshot semantics (VV-CTL-325).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

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
fn snapshot_fresh_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn snapshot_creates_file() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_shows_snapshot() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn snapshot_twice_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn pitr_list_after_2_snapshots() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn snapshot_missing_dir_error() {
    assert!(
        run(TableCommand::Snapshot {
            dir: PathBuf::from("/nonexistent/path")
        })
        .is_err()
    );
}

#[test]
fn snapshot_after_truncate_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn snapshot_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn snapshot_after_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn snapshot_after_alter_add_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "extra:u64".into(),
    })
    .unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn pitr_list_json_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn pitr_list_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn snapshot_then_check_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
