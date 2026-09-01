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
fn scan_inclusive_lo() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins(&d, 3, 30);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 10,
            hi: 25,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_inclusive_hi() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins(&d, 3, 30);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 15,
            hi: 30,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_entire_range() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_narrow_range_single() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    ins(&d, 2, 200);
    ins(&d, 3, 300);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 200,
            hi: 200,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_below_all_values() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 100);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: 50,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_above_all_values() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 1000,
            hi: 9999,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_all_same_val() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, 42);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 42,
            hi: 42,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_json_output() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: 100,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn scan_after_update_changes_result() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 999,
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 900,
            hi: 1000,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_interleaved_pks() {
    let d = make_dir();
    setup(&d);
    ins(&d, 10, 100);
    ins(&d, 1, 10);
    ins(&d, 5, 50);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 10,
            hi: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_pk_column_range() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i * 7);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 0,
            lo: 3,
            hi: 7,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_after_delete_shrinks() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
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

#[test]
fn scan_large_lo_hi() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, u64::MAX - 1);
    ins(&d, 2, u64::MAX);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: u64::MAX - 1,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
