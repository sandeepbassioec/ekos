//! Tests for Layer 324: Bloom filter stub correctness (VV-CTL-324).

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
fn insert_10_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10 {
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
fn get_5_known_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10 {
        ins(&d, i, i * 10);
    }
    for pk in 0..5 {
        assert!(
            run(TableCommand::Get {
                dir: tdir(&d),
                pk,
                output: OutputFormat::Table
            })
            .is_ok()
        );
    }
}

#[test]
fn get_5_unknown_ok_or_err() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    for pk in 100..105 {
        let _ = run(TableCommand::Get {
            dir: tdir(&d),
            pk,
            output: OutputFormat::Table,
        });
    }
}

#[test]
fn scan_all_10() {
    let d = make_dir();
    setup(&d);
    for i in 0..10 {
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
fn count_10() {
    let d = make_dir();
    setup(&d);
    for i in 0..10 {
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
fn after_compact_get_known_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn after_vacuum_get_known_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_get_5_known_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![0, 1, 2, 3, 4],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn batch_get_5_unknown_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::BatchGet {
            dir: tdir(&d),
            pks: vec![100, 101, 102, 103, 104],
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn flush_compact_scan_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
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
fn check_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn recount_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_after_insert_10() {
    let d = make_dir();
    setup(&d);
    for i in 0..10 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 0,
            lo: 0,
            hi: 9,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
