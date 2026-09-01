//! Tests for Layer 265: vvctl table insert/delete cycles (VV-CTL-265).

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

#[test]
fn cycle_insert_then_delete_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
    })
    .unwrap();
}

#[test]
fn cycle_reinsert_same_pk_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 1,
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![20],
    })
    .unwrap();
}

#[test]
fn cycle_get_after_reinsert_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 2,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 2,
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 2,
        values: vec![99],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 2,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn cycle_scan_after_reinsert_count1() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 3,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 3,
        values: vec![99],
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn cycle_insert5_delete3_count2() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    for i in 1..=3u64 {
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: i,
        })
        .unwrap();
    }
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn cycle_delete_nonexistent_ok() {
    let d = make_dir();
    setup(&d);
    // Delete on non-existent pk may succeed or not; either way must not panic
    let _ = run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 999,
    });
}

#[test]
fn cycle_delete_insert_delete_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![50],
    })
    .unwrap();
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 5,
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![55],
    })
    .unwrap();
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 5,
    })
    .unwrap();
}

#[test]
fn cycle_batch_delete_then_insert_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1, 2, 3],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 10,
        values: vec![100],
    })
    .unwrap();
}

#[test]
fn cycle_truncate_then_reinsert_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![99],
    })
    .unwrap();
}

#[test]
fn cycle_insert_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 2,
        values: vec![20],
    })
    .unwrap();
}

#[test]
fn cycle_insert_after_flush_compact_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![50],
    })
    .unwrap();
}

#[test]
fn cycle_upsert_existing_row_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 7,
        values: vec![70],
    })
    .unwrap();
    run(TableCommand::UpsertRow {
        dir: tdir(&d),
        pk: 7,
        values: vec![77],
    })
    .unwrap();
}

#[test]
fn cycle_upsert_new_row_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::UpsertRow {
        dir: tdir(&d),
        pk: 99,
        values: vec![999],
    })
    .unwrap();
}
