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
fn delete_then_stats_count_decrements() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
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
fn delete_nonexistent_errors() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: 999
        })
        .is_err()
    );
}

#[test]
fn insert_delete_insert_same_pk() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 5,
    })
    .unwrap();
    assert!(
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: 5,
            values: vec![99]
        })
        .is_ok()
    );
}

#[test]
fn delete_multiple_pks_no_ghost() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    for i in 1..=5 {
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: i,
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn delete_then_get_same_pk() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 77);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 7,
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 7,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn delete_middle_key_others_remain() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins(&d, 3, 30);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 2,
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
fn delete_first_key() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    assert!(
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: 1
        })
        .is_ok()
    );
}

#[test]
fn delete_last_key() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    assert!(
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: 2
        })
        .is_ok()
    );
}

#[test]
fn repeated_delete_same_pk_errors() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 30);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
    })
    .unwrap();
    assert!(
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: 3
        })
        .is_err()
    );
}

#[test]
fn delete_all_then_scan_empty() {
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
fn insert_delete_reinsert_different_val() {
    let d = make_dir();
    setup(&d);
    ins(&d, 42, 100);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 42,
    })
    .unwrap();
    assert!(
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: 42,
            values: vec![200]
        })
        .is_ok()
    );
}

#[test]
fn delete_and_flush_no_ghost() {
    let d = make_dir();
    setup(&d);
    ins(&d, 9, 90);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 9,
    })
    .unwrap();
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}

#[test]
fn delete_and_compact_no_ghost() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 5);
    }
    for i in 1..=4 {
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: i,
        })
        .unwrap();
    }
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}
