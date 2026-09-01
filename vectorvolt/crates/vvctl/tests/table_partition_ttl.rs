//! Tests for Layer 322: Partition TTL / delete lifecycle (VV-CTL-322).

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
fn insert_flush_compact_ok() {
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
fn delete_3_pks_count_correct() {
    let d = make_dir();
    setup(&d);
    for i in 0..6 {
        ins(&d, i, i * 10);
    }
    for pk in [0u64, 1, 2] {
        run(TableCommand::Delete { dir: tdir(&d), pk }).unwrap();
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
fn batch_delete_2_pks_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![3, 4],
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
fn scan_after_delete_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 0,
    })
    .unwrap();
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
fn stats_after_delete_ok() {
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
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn vacuum_after_delete_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
    })
    .unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn check_after_delete_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn insert_new_after_delete_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
    })
    .unwrap();
    assert!(
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: 99,
            values: vec![990]
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
fn delete_nonexistent_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: 9999
        })
        .is_ok()
    );
}

#[test]
fn count_after_all_deleted() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
    })
    .unwrap();
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 2,
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
fn full_lifecycle_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    for pk in 0..3 {
        run(TableCommand::Delete { dir: tdir(&d), pk }).unwrap();
    }
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn recount_after_delete_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..4 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 0,
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
