//! Tests for Layer 335: Sink checkpoint (VV-CTL-335).

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
fn snapshot_as_checkpoint_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn checkpoint_pitr_list_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn checkpoint_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn checkpoint_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn check_after_checkpoint_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}

#[test]
fn stats_after_checkpoint_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scan_after_checkpoint_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
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
fn info_checkpoint_ok() {
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
fn vacuum_after_checkpoint_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn recount_after_checkpoint_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn two_checkpoints_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    ins(&d, 1, 10);
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn keys_after_checkpoint_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Keys {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn count_after_checkpoint_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
