//! Tests for Layer 298: Stress mixed ops (VV-CTL-298).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir { TempDir::new().unwrap() }
fn tdir(d: &TempDir) -> PathBuf { d.path().join("tbl") }
fn setup(d: &TempDir) {
    run(TableCommand::Create { dir: tdir(d), columns: vec!["pk:u64".into(), "val:u64".into()] }).unwrap();
}
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert { dir: tdir(d), pk, values: vec![val] }).unwrap();
}

#[test]
fn stress_insert_delete_cycle_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 { ins(&d, i, i * 10); }
    for i in 1..=5 { run(TableCommand::Delete { dir: tdir(&d), pk: i }).unwrap(); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_insert_update_flush_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    for i in 1..=5 { run(TableCommand::Update { dir: tdir(&d), pk: i, col: 1, val: i * 100 }).unwrap(); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_flush_compact_insert_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    for i in 6..=10 { ins(&d, i, i * 10); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_increment_decrement_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 50);
    for _ in 0..5 { run(TableCommand::IncrementField { dir: tdir(&d), pk: 1, col: 1 }).unwrap(); }
    for _ in 0..3 { run(TableCommand::DecrementField { dir: tdir(&d), pk: 1, col: 1 }).unwrap(); }
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_set_field_mixed_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::SetFieldAdd { dir: tdir(&d), pk: 1, col: 1, addend: 100 }).unwrap();
    run(TableCommand::SetFieldMul { dir: tdir(&d), pk: 1, col: 1, factor: 2 }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 1, output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_truncate_refill_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    run(TableCommand::Truncate { dir: tdir(&d), force: true }).unwrap();
    for i in 1..=5 { ins(&d, i, i * 20); }
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_mixed_get_scan_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=8 { ins(&d, i, i * 10); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 5, output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_schema_stable_after_ops_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Schema { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_stats_after_heavy_ops_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 { ins(&d, i, i * 10); }
    for i in 1..=5 { run(TableCommand::Update { dir: tdir(&d), pk: i, col: 1, val: 999 }).unwrap(); }
    for i in 6..=8 { run(TableCommand::Delete { dir: tdir(&d), pk: i }).unwrap(); }
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_check_after_ops_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn stress_recount_after_ops_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 { ins(&d, i, i * 10); }
    run(TableCommand::Delete { dir: tdir(&d), pk: 2 }).unwrap();
    run(TableCommand::Delete { dir: tdir(&d), pk: 4 }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_cas_batch_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    run(TableCommand::CompareAndSwap { dir: tdir(&d), pk: 1, col: 1, expected: 10, replacement: 99 }).unwrap();
    run(TableCommand::CompareAndSwap { dir: tdir(&d), pk: 2, col: 1, expected: 20, replacement: 88 }).unwrap();
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn stress_alter_add_ops_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 { ins(&d, i, i * 10); }
    run(TableCommand::AlterAddCol { dir: tdir(&d), column: "extra:u64".into() }).unwrap();
    assert!(run(TableCommand::Schema { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}
