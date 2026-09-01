//! Tests for Layer 272: vvctl table check integrity (VV-CTL-272).

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
fn check_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn check_after_insert_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn check_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn check_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn check_after_truncate_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn check_after_alter_add_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "extra:u64".into(),
    })
    .unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn check_after_alter_drop_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::AlterDropCol {
        dir: tdir(&d),
        column: "b".into(),
    })
    .unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn check_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::Check {
        dir: d.path().join("nope"),
    });
    assert!(res.is_err());
}

#[test]
fn check_returns_ok() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::Check { dir: tdir(&d) });
    assert!(res.is_ok());
}

#[test]
fn check_two_tables_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Create {
        dir: d.path().join("tbl2"),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check {
        dir: d.path().join("tbl2"),
    })
    .unwrap();
}

#[test]
fn check_large_table_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..20u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn check_after_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn check_after_snapshot_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}
