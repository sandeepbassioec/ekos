//! Tests for Layer 294: Exists command correctness (VV-CTL-294).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{TableCommand, run};

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
fn exists_valid_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::Exists {
            dir: d.path().join("nope")
        })
        .is_err()
    );
}

#[test]
fn exists_after_insert_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_after_truncate_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_after_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 20);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_after_update_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 99,
    })
    .unwrap();
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_multi_col_table_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_after_delete_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
    })
    .unwrap();
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_5_inserts_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_after_rename_col_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::RenameCol {
        dir: tdir(&d),
        from: "val".into(),
        to: "renamed".into(),
    })
    .unwrap();
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}

#[test]
fn exists_after_alter_add_col_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::AlterAddCol {
        dir: tdir(&d),
        column: "extra:u64".into(),
    })
    .unwrap();
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}
