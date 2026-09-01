//! Tests for Layer 263: vvctl table PK boundary values (VV-CTL-263).

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

const BIG_PK: u64 = u64::MAX - 1;

#[test]
fn pk_boundary_insert_pk0() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 0,
        values: vec![100],
    })
    .unwrap();
}

#[test]
fn pk_boundary_insert_pk1() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![200],
    })
    .unwrap();
}

#[test]
fn pk_boundary_insert_pk_max_minus_2() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: u64::MAX - 2,
        values: vec![300],
    })
    .unwrap();
}

#[test]
fn pk_boundary_insert_pk_big() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: BIG_PK,
        values: vec![400],
    })
    .unwrap();
}

#[test]
fn pk_boundary_get_pk0_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 0,
        values: vec![100],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 0,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn pk_boundary_get_pk_big_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: BIG_PK,
        values: vec![999],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: BIG_PK,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn pk_boundary_delete_pk0_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 0,
        values: vec![100],
    })
    .unwrap();
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 0,
    })
    .unwrap();
}

#[test]
fn pk_boundary_delete_pk_big_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: BIG_PK,
        values: vec![999],
    })
    .unwrap();
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: BIG_PK,
    })
    .unwrap();
}

#[test]
fn pk_boundary_scan_includes_all() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 0,
        values: vec![1],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: BIG_PK,
        values: vec![2],
    })
    .unwrap();
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 0,
        lo: 0,
        hi: BIG_PK,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn pk_boundary_pk_range_min_zero() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 0,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::PkRange {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn pk_boundary_pk_range_big_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: BIG_PK,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::PkRange {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn pk_boundary_update_pk_big_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: BIG_PK,
        values: vec![1],
    })
    .unwrap();
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: BIG_PK,
        col: 1,
        val: 999,
    })
    .unwrap();
}

#[test]
fn pk_boundary_batch_get_boundaries_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 0,
        values: vec![1],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: BIG_PK,
        values: vec![2],
    })
    .unwrap();
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![0, BIG_PK],
        output: OutputFormat::Table,
    })
    .unwrap();
}
