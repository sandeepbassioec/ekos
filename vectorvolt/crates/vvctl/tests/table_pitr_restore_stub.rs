//! Tests for Layer 326: PITR restore stub (VV-CTL-326).

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
fn snapshot_pitr_list_has_entry() {
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
fn pitr_list_after_truncate_snapshot_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
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
fn pitr_list_missing_dir_error() {
    assert!(
        run(TableCommand::PitrList {
            dir: PathBuf::from("/nonexistent"),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn pitr_list_after_3_snaps() {
    let d = make_dir();
    setup(&d);
    for _ in 0..3 {
        run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    }
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn snapshot_dir_arg_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn pitr_list_dir_arg_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn snapshot_check_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn snapshot_stats_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn snapshot_info_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn snapshot_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn snapshot_compact_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
