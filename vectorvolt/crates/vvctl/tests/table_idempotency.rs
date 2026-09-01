//! Tests for Layer 299: Idempotency correctness (VV-CTL-299).

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
fn idempotent_flush_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Flush { dir: tdir(&d) }).is_ok());
}

#[test]
fn idempotent_compact_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn idempotent_vacuum_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn idempotent_schema_twice_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Schema { dir: tdir(&d), output: OutputFormat::Table }).unwrap();
    assert!(run(TableCommand::Schema { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn idempotent_count_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).unwrap();
    assert!(run(TableCommand::Count { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn idempotent_stats_twice_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).unwrap();
    assert!(run(TableCommand::Stats { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn idempotent_check_twice_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn idempotent_recount_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).unwrap();
    assert!(run(TableCommand::Recount { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn idempotent_alter_drop_nonexistent_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    }).unwrap();
    run(TableCommand::AlterDropCol { dir: tdir(&d), column: "b".into() }).unwrap();
    assert!(run(TableCommand::AlterDropCol { dir: tdir(&d), column: "b".into() }).is_ok());
}

#[test]
fn idempotent_truncate_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Truncate { dir: tdir(&d), force: true }).unwrap();
    assert!(run(TableCommand::Truncate { dir: tdir(&d), force: true }).is_ok());
}

#[test]
fn idempotent_get_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Get { dir: tdir(&d), pk: 5, output: OutputFormat::Table }).unwrap();
    assert!(run(TableCommand::Get { dir: tdir(&d), pk: 5, output: OutputFormat::Table }).is_ok());
}

#[test]
fn idempotent_keys_twice_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Keys { dir: tdir(&d), output: OutputFormat::Table }).unwrap();
    assert!(run(TableCommand::Keys { dir: tdir(&d), output: OutputFormat::Table }).is_ok());
}

#[test]
fn idempotent_exists_twice_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Exists { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Exists { dir: tdir(&d) }).is_ok());
}
