//! Tests for Layer 400.
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
fn dst_insert_13() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn dst_scan_13() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i * 10);
    }
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
fn dst_update_odd_pks() {
    let d = make_dir();
    setup(&d);
    for i in 1..=9u64 {
        ins(&d, i, i * 10);
    }
    for pk in [1u64, 3, 5, 7, 9] {
        run(TableCommand::Update {
            dir: tdir(&d),
            pk,
            col: 1,
            val: pk * 100,
        })
        .unwrap();
    }
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
fn dst_batch_delete_10_13() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![10, 11, 12, 13]
        })
        .is_ok()
    );
}
#[test]
fn dst_count_9() {
    let d = make_dir();
    setup(&d);
    for i in 1..=13u64 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![10, 11, 12, 13],
    })
    .unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn dst_upsert_pk1() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![999]
        })
        .is_ok()
    );
}
#[test]
fn dst_get_pk1_999() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::UpsertRow {
        dir: tdir(&d),
        pk: 1,
        values: vec![999],
    })
    .unwrap();
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
fn dst_cas_pk3_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 30);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 3,
            col: 1,
            expected: 30,
            replacement: 60
        })
        .is_ok()
    );
}
#[test]
fn dst_check_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn dst_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
#[test]
fn dst_recount_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn dst_cas_col0_errors() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 0,
            expected: 1,
            replacement: 2
        })
        .is_err()
    );
}
#[test]
fn dst_update_col0_errors() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 1,
            col: 0,
            val: 99
        })
        .is_err()
    );
}
