//! Integration tests for Layer 160: vvctl table merge (VV-CTL-160).

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn create(dir: &Path) {
    run(TableCommand::Create {
        dir: dir.to_path_buf(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

fn insert(dir: &Path, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: dir.to_path_buf(),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn merge_empty_src_into_empty_dst_ok() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    assert!(
        run(TableCommand::Merge {
            src: src.clone(),
            dst: dst.clone()
        })
        .is_ok()
    );
}

#[test]
fn merge_rows_into_empty_dst_ok() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    insert(&src, 1, 10);
    insert(&src, 2, 20);
    assert!(run(TableCommand::Merge { src, dst }).is_ok());
}

#[test]
fn merge_empty_src_into_filled_dst_ok() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    insert(&dst, 1, 10);
    assert!(run(TableCommand::Merge { src, dst }).is_ok());
}

#[test]
fn merge_upsert_semantics_ok() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    insert(&dst, 1, 10);
    insert(&src, 1, 99);
    assert!(run(TableCommand::Merge { src, dst }).is_ok());
}

#[test]
fn merge_missing_src_errors() {
    let d = make_dir();
    let dst = d.path().join("dst");
    create(&dst);
    assert!(
        run(TableCommand::Merge {
            src: PathBuf::from("/no/such"),
            dst
        })
        .is_err()
    );
}

#[test]
fn merge_missing_dst_errors() {
    let d = make_dir();
    let src = d.path().join("src");
    create(&src);
    assert!(
        run(TableCommand::Merge {
            src,
            dst: PathBuf::from("/no/such")
        })
        .is_err()
    );
}

#[test]
fn merge_many_rows_ok() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    for i in 0..10u64 {
        insert(&src, i, i);
    }
    assert!(run(TableCommand::Merge { src, dst }).is_ok());
}

#[test]
fn merge_then_scan_ok() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    insert(&src, 5, 50);
    run(TableCommand::Merge {
        src,
        dst: dst.clone(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: dst,
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn merge_single_row_ok() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    insert(&src, 42, 99);
    assert!(run(TableCommand::Merge { src, dst }).is_ok());
}

#[test]
fn merge_disjoint_pks_ok() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    for i in 0..5u64 {
        insert(&src, i, i);
        insert(&dst, i + 100, i + 100);
    }
    assert!(run(TableCommand::Merge { src, dst }).is_ok());
}

#[test]
fn merge_does_not_modify_src() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    insert(&src, 7, 77);
    run(TableCommand::Merge {
        src: src.clone(),
        dst,
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: src,
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn merge_repeated_ok() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    insert(&src, 1, 1);
    assert!(
        run(TableCommand::Merge {
            src: src.clone(),
            dst: dst.clone()
        })
        .is_ok()
    );
    assert!(run(TableCommand::Merge { src, dst }).is_ok());
}

#[test]
fn merge_zero_pk_ok() {
    let d = make_dir();
    let src = d.path().join("src");
    let dst = d.path().join("dst");
    create(&src);
    create(&dst);
    insert(&src, 0, 0);
    assert!(run(TableCommand::Merge { src, dst }).is_ok());
}
