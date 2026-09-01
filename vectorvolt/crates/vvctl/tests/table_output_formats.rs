//! Tests for Layer 260: vvctl table output format coverage (VV-CTL-260).

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
    run(TableCommand::Insert {
        dir: tdir(d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
}

#[test]
fn output_get_table() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn output_get_json() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 1,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn output_scan_table() {
    let d = make_dir();
    setup(&d);
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
fn output_scan_json() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Scan {
        dir: tdir(&d),
        col: 0,
        lo: 0,
        hi: u64::MAX,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn output_stats_table() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn output_stats_json() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn output_count_table() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn output_count_json() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn output_head_table() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Head {
        dir: tdir(&d),
        n: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn output_head_json() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Head {
        dir: tdir(&d),
        n: 5,
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn output_keys_table() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Keys {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn output_keys_json() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Keys {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn output_schema_table() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}
