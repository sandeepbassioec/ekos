//! Tests for Layer 269: vvctl table JSON output correctness (VV-CTL-269).

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
        values: vec![100],
    })
    .unwrap();
}

#[test]
fn json_get_ok() {
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
fn json_scan_ok() {
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
fn json_stats_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn json_count_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn json_head_ok() {
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
fn json_keys_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Keys {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn json_pk_range_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::PkRange {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn json_col_stats_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::ColStats {
        dir: tdir(&d),
        col: "val".into(),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn json_schema_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Schema {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn json_info_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Info {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn json_recount_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Recount {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn json_pitr_list_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::PitrList {
        dir: tdir(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn json_histogram_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Histogram {
        dir: tdir(&d),
        col: 1,
        bucket_size: 10,
        output: OutputFormat::Json,
    })
    .unwrap();
}
