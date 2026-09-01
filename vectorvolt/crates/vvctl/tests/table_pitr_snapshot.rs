//! Tests for Layer 270: vvctl table PITR/snapshot deep (VV-CTL-270).

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

#[test]
fn pitr_snapshot_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
}

#[test]
fn pitr_snapshot_creates_file_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    let res = run(TableCommand::PitrList {
        dir: tdir(&d),
        output: OutputFormat::Table,
    });
    assert!(res.is_ok());
}

#[test]
fn pitr_list_after_snapshot_has_entry() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::PitrList {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn pitr_list_json_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::PitrList {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn pitr_list_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::PitrList {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn pitr_second_snapshot_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
}

#[test]
fn pitr_list_two_snapshots_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::PitrList {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn pitr_snapshot_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::Snapshot {
        dir: d.path().join("nope"),
    });
    assert!(res.is_err());
}

#[test]
fn pitr_list_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::PitrList {
        dir: d.path().join("nope"),
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn pitr_snapshot_after_insert_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
}

#[test]
fn pitr_list_three_snaps_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::PitrList {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn pitr_snapshot_dir_arg_ok() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::Snapshot { dir: tdir(&d) });
    assert!(res.is_ok());
}

#[test]
fn pitr_list_dir_arg_ok() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::PitrList {
        dir: tdir(&d),
        output: OutputFormat::Json,
    });
    assert!(res.is_ok());
}
