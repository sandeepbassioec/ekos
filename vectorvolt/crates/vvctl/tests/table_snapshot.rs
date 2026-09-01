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
fn snapshot_empty_table() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn snapshot_with_data() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn pitr_list_empty() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn snapshot_then_pitr_list() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
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
fn multiple_snapshots() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    ins(&d, 2, 20);
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
fn pitr_list_json_output() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn snapshot_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 3);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn snapshot_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 3);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn restore_lsn_zero() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Restore {
            dir: tdir(&d),
            lsn: 0
        })
        .is_ok()
    );
}

#[test]
fn snapshot_large_table() {
    let d = make_dir();
    setup(&d);
    for i in 1..=20 {
        ins(&d, i, i * 7);
    }
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn snapshot_after_delete() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
    })
    .unwrap();
    assert!(run(TableCommand::Snapshot { dir: tdir(&d) }).is_ok());
}

#[test]
fn pitr_list_after_multiple_snapshots_json() {
    let d = make_dir();
    setup(&d);
    for _ in 0..3 {
        ins(&d, 1, 1);
        run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    }
    assert!(
        run(TableCommand::PitrList {
            dir: tdir(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn snapshot_then_scan_unchanged() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 4);
    }
    run(TableCommand::Snapshot { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
