//! Tests for Layer 390.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

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
fn pgwire_insert_3() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins(&d, 3, 30);
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn pgwire_export_json() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn pgwire_scan_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn pgwire_get_pk1() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn pgwire_count_3() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins(&d, 3, 30);
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn pgwire_sum_field_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    assert!(
        run(TableCommand::SumField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn pgwire_keys_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Keys {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn pgwire_pk_range_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 3, 30);
    assert!(
        run(TableCommand::PkRange {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn pgwire_stats_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn pgwire_info_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn pgwire_check_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn pgwire_recount_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn pgwire_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
