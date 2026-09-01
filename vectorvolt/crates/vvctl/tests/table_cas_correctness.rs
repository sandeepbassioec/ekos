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
fn cas_success_correct_expected() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            expected: 10,
            replacement: 99
        })
        .is_ok()
    );
}

#[test]
fn cas_fail_wrong_expected() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            expected: 99,
            replacement: 200
        })
        .is_err()
    );
}

#[test]
fn cas_then_get() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 20);
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 2,
        col: 1,
        expected: 20,
        replacement: 200,
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn cas_nonexistent_pk() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 999,
            col: 1,
            expected: 0,
            replacement: 1
        })
        .is_err()
    );
}

#[test]
fn cas_chained_updates() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 10);
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        expected: 10,
        replacement: 20,
    })
    .unwrap();
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 3,
            col: 1,
            expected: 20,
            replacement: 30
        })
        .is_ok()
    );
}

#[test]
fn cas_same_expected_and_replacement() {
    let d = make_dir();
    setup(&d);
    ins(&d, 4, 40);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 4,
            col: 1,
            expected: 40,
            replacement: 40
        })
        .is_ok()
    );
}

#[test]
fn cas_zero_expected() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 0);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 5,
            col: 1,
            expected: 0,
            replacement: 100
        })
        .is_ok()
    );
}

#[test]
fn cas_max_value() {
    let d = make_dir();
    setup(&d);
    ins(&d, 6, u64::MAX);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 6,
            col: 1,
            expected: u64::MAX,
            replacement: 0
        })
        .is_ok()
    );
}

#[test]
fn cas_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 70);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 7,
            col: 1,
            expected: 70,
            replacement: 700
        })
        .is_ok()
    );
}

#[test]
fn cas_multiple_rows_independent() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins(&d, 3, 30);
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 2,
        col: 1,
        expected: 20,
        replacement: 200,
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
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn cas_after_update() {
    let d = make_dir();
    setup(&d);
    ins(&d, 8, 80);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 8,
        col: 1,
        val: 800,
    })
    .unwrap();
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 8,
            col: 1,
            expected: 800,
            replacement: 8000
        })
        .is_ok()
    );
}

#[test]
fn cas_pk_column() {
    let d = make_dir();
    setup(&d);
    ins(&d, 9, 90);
    assert!(
        run(TableCommand::CompareAndSwap {
            dir: tdir(&d),
            pk: 9,
            col: 0,
            expected: 9,
            replacement: 9
        })
        .is_err()
    );
}

#[test]
fn cas_then_scan() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        expected: 30,
        replacement: 300,
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: 1000,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
