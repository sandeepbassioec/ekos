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
fn scan_full_range() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
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
fn scan_empty_range() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 50);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 200,
            hi: 300,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_single_match() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins(&d, 3, 30);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 20,
            hi: 20,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_no_rows() {
    let d = make_dir();
    setup(&d);
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
fn scan_boundary_lo() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    ins(&d, 2, 200);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 100,
            hi: 150,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_boundary_hi() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    ins(&d, 2, 200);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 150,
            hi: 200,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_json_output() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
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
fn scan_after_delete() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
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
fn scan_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
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
fn scan_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 5);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
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
fn scan_pk_column() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 100);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 0,
            lo: 2,
            hi: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_zero_range() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 50);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 50,
            hi: 50,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_large_dataset() {
    let d = make_dir();
    setup(&d);
    for i in 1..=20 {
        ins(&d, i, i * 3);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 10,
            hi: 40,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
