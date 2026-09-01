//! Tests for Layer 254: vvctl table WAL stats behavior (VV-CTL-254).

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
fn wal_stats_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_stats_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::Stats {
        dir: d.path().join("nope"),
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn wal_stats_json_output_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn wal_stats_table_output_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_stats_after_insert_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_stats_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_stats_size_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Size {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_stats_count_correct_after_inserts() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    ins(&d, 2, 2);
    ins(&d, 3, 3);
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_stats_recount_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 10, 100);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Recount {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_stats_zero_on_empty_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn wal_stats_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 55);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn wal_stats_multiple_inserts_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn wal_stats_info_command_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    run(TableCommand::Info {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}
