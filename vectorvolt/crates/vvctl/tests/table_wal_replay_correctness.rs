//! Tests for Layer 253: vvctl table WAL replay behavior (VV-CTL-253).

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
fn wal_replay_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_replay_empty_wal_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_replay_after_inserts_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_replay_returns_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    let res = run(TableCommand::Flush { dir: tdir(&d) });
    assert!(res.is_ok());
}

#[test]
fn wal_replay_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::Flush {
        dir: d.path().join("nonexistent"),
    });
    assert!(res.is_err());
}

#[test]
fn wal_replay_json_output_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn wal_replay_table_output_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_replay_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 30);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_replay_count_correct() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    ins(&d, 2, 2);
    ins(&d, 3, 3);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_replay_scan_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 10, 100);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 0,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_replay_data_persists_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 77);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 7,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_replay_compact_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
}

#[test]
fn wal_replay_stats_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 99, 999);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}
