//! Tests for Layer 327: Mmap cold stub correctness (VV-CTL-327).

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
fn flush_partition_count_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn scan_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
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
fn size_increases_after_insert_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Size {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn size_after_truncate_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Size {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn vacuum_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn check_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn info_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Info {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn stats_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn head_n5_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_n5_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tdir(&d),
            n: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn keys_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Keys {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn export_ok() {
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
