//! Tests for Layer 370.
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
        columns: vec!["pk:u64".into(), "epoch:u64".into()],
    })
    .unwrap();
}

#[test]
fn epoch_insert_zero() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: 1,
            values: vec![0]
        })
        .is_ok()
    );
}
#[test]
fn epoch_cas_0_to_1() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![0],
    })
    .unwrap();
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            expected: 0,
            replacement: 1
        })
        .is_ok()
    );
}
#[test]
fn epoch_cas_wrong_expected_err() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            expected: 0,
            replacement: 2
        })
        .is_err()
    );
}
#[test]
fn epoch_cas_1_to_2() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            expected: 1,
            replacement: 2
        })
        .is_ok()
    );
}
#[test]
fn epoch_get_val_2() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![2],
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
fn epoch_incr_to_3() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![2],
    })
    .unwrap();
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
fn epoch_incr_to_4() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![3],
    })
    .unwrap();
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
fn epoch_decr_to_3() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![4],
    })
    .unwrap();
    assert!(
        run(TableCommand::DecrementField {
            dir: tdir(&d),
            pk: 1,
            col: 1
        })
        .is_ok()
    );
}
#[test]
fn epoch_set_mul_2() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![3],
    })
    .unwrap();
    assert!(
        run(TableCommand::SetFieldMul {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            factor: 2
        })
        .is_ok()
    );
}
#[test]
fn epoch_clamp_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![6],
    })
    .unwrap();
    assert!(
        run(TableCommand::ClampField {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            lo: 0,
            hi: 100
        })
        .is_ok()
    );
}
#[test]
fn epoch_check_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn epoch_stats_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn epoch_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
