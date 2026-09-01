//! Tests for Layer 267: vvctl table atomicity across multiple ops (VV-CTL-267).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn tdir2(d: &TempDir) -> PathBuf {
    d.path().join("tbl2")
}
fn setup(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

#[test]
fn atomicity_insert_flush_compact_readable() {
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
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn atomicity_insert_delete_flush_count0() {
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
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn atomicity_insert_update_get_correct() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 2,
        values: vec![20],
    })
    .unwrap();
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 2,
        col: 1,
        val: 200,
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
fn atomicity_batch_insert_batch_delete_count0() {
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
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![1, 2, 3, 4, 5],
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn atomicity_upsert_existing_then_get_new() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 3,
        values: vec![30],
    })
    .unwrap();
    run(TableCommand::UpsertRow {
        dir: tdir(&d),
        pk: 3,
        values: vec![999],
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 3,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn atomicity_cas_then_get_replacement() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 4,
        values: vec![40],
    })
    .unwrap();
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 4,
        col: 1,
        expected: 40,
        replacement: 400,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 4,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn atomicity_incr_decr_round_trip() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 5,
        col: 1,
    })
    .unwrap();
    run(TableCommand::DecrementField {
        dir: tdir(&d),
        pk: 5,
        col: 1,
    })
    .unwrap();
}

#[test]
fn atomicity_set_add_then_set_mul_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 6,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::SetFieldAdd {
        dir: tdir(&d),
        pk: 6,
        col: 1,
        addend: 5,
    })
    .unwrap();
    run(TableCommand::SetFieldMul {
        dir: tdir(&d),
        pk: 6,
        col: 1,
        factor: 2,
    })
    .unwrap();
}

#[test]
fn atomicity_clamp_then_get_in_range() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 7,
        values: vec![150],
    })
    .unwrap();
    run(TableCommand::ClampField {
        dir: tdir(&d),
        pk: 7,
        col: 1,
        lo: 0,
        hi: 100,
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 7,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn atomicity_snapshot_then_pitr_list_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    run(TableCommand::PitrList {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn atomicity_truncate_then_insert_count1() {
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
        pk: 2,
        values: vec![20],
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn atomicity_copy_then_diff_empty() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Diff {
        a: tdir(&d),
        b: tdir2(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn atomicity_batch_get_correct_pks() {
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
    run(TableCommand::BatchGet {
        dir: tdir(&d),
        pks: vec![1, 3, 5],
        output: OutputFormat::Table,
    })
    .unwrap();
}
