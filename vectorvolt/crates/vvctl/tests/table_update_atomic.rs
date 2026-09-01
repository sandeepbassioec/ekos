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
fn update_existing_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            val: 99
        })
        .is_ok()
    );
}

#[test]
fn update_then_get() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 5,
        col: 1,
        val: 500,
    })
    .unwrap();
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
fn update_nonexistent_pk() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 999,
            col: 1,
            val: 1
        })
        .is_err()
    );
}

#[test]
fn update_multiple_times_same_pk() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 30);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        val: 300,
    })
    .unwrap();
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        val: 3000,
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
fn update_to_zero() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 77);
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 7,
            col: 1,
            val: 0
        })
        .is_ok()
    );
}

#[test]
fn update_to_max_val() {
    let d = make_dir();
    setup(&d);
    ins(&d, 2, 20);
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 2,
            col: 1,
            val: u64::MAX
        })
        .is_ok()
    );
}

#[test]
fn update_does_not_affect_other_rows() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 999,
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
fn update_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 4, 40);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 4,
            col: 1,
            val: 400
        })
        .is_ok()
    );
}

#[test]
fn update_pk_column() {
    let d = make_dir();
    setup(&d);
    ins(&d, 6, 60);
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 6,
            col: 0,
            val: 6
        })
        .is_err()
    );
}

#[test]
fn update_then_scan() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        val: 999,
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

#[test]
fn update_then_stats() {
    let d = make_dir();
    setup(&d);
    ins(&d, 8, 80);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 8,
        col: 1,
        val: 888,
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
fn update_same_value_idempotent() {
    let d = make_dir();
    setup(&d);
    ins(&d, 9, 90);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 9,
        col: 1,
        val: 90,
    })
    .unwrap();
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 9,
            col: 1,
            val: 90
        })
        .is_ok()
    );
}

#[test]
fn update_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i * 5);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 4,
            col: 1,
            val: 44
        })
        .is_ok()
    );
}
