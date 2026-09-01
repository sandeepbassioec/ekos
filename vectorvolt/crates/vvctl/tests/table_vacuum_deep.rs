//! Tests for Layer 273: vvctl table vacuum deep (VV-CTL-273).

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
fn vacuum_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn vacuum_after_insert_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn vacuum_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 2,
        values: vec![20],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn vacuum_partition_count_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 3,
        values: vec![30],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn vacuum_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 4,
        values: vec![40],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn vacuum_rows_readable_after() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![50],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn vacuum_missing_dir_errors() {
    let d = make_dir();
    let res = run(TableCommand::Vacuum {
        dir: d.path().join("nope"),
    });
    assert!(res.is_err());
}

#[test]
fn vacuum_large_table_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn vacuum_check_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
}

#[test]
fn vacuum_stats_after_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 6,
        values: vec![60],
    })
    .unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn vacuum_twice_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
}

#[test]
fn vacuum_json_stats_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn vacuum_table_stats_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}
