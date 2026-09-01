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
fn increment_basic() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
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
fn decrement_basic() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
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
fn increment_then_get() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 20);
    run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 2,
        col: 1,
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
fn decrement_then_get() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 30);
    run(TableCommand::DecrementField {
        dir: tdir(&d),
        pk: 3,
        col: 1,
    })
    .unwrap();
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
fn increment_multiple_times() {
    let d = make_dir();
    setup(&d);
    ins(&d, 4, 0);
    for _ in 0..5 {
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 4,
            col: 1,
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 4,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn decrement_multiple_times() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 100);
    for _ in 0..5 {
        run(TableCommand::DecrementField {
            dir: tdir(&d),
            pk: 5,
            col: 1,
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn increment_decrement_cancel() {
    let d = make_dir();
    setup(&d);
    ins(&d, 6, 50);
    run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 6,
        col: 1,
    })
    .unwrap();
    run(TableCommand::DecrementField {
        dir: tdir(&d),
        pk: 6,
        col: 1,
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 6,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn increment_nonexistent_pk() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 999,
            col: 1
        })
        .is_err()
    );
}

#[test]
fn decrement_nonexistent_pk() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::DecrementField {
            dir: tdir(&d),
            pk: 999,
            col: 1
        })
        .is_err()
    );
}

#[test]
fn increment_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 70);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::IncrementField {
            dir: tdir(&d),
            pk: 7,
            col: 1
        })
        .is_ok()
    );
}

#[test]
fn decrement_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 8, 80);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::DecrementField {
            dir: tdir(&d),
            pk: 8,
            col: 1
        })
        .is_ok()
    );
}

#[test]
fn increment_many_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    for i in 1..=5 {
        assert!(
            run(TableCommand::IncrementField {
                dir: tdir(&d),
                pk: i,
                col: 1
            })
            .is_ok()
        );
    }
}

#[test]
fn decrement_then_scan() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 5);
    }
    run(TableCommand::DecrementField {
        dir: tdir(&d),
        pk: 2,
        col: 1,
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
