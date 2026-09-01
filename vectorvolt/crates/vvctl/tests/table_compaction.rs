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
fn compact_empty_table() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_single_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_many_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=20 {
        ins(&d, i, i * 5);
    }
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_twice_idempotent() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=8 {
        ins(&d, i, i * 2);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_then_get() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 77);
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 7,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn compact_then_scan() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i * 4);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
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

#[test]
fn compact_after_deletes() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i * 3);
    }
    for i in 1..=5 {
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: i,
        })
        .unwrap();
    }
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_then_count() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn compact_then_stats() {
    let d = make_dir();
    setup(&d);
    for i in 1..=7 {
        ins(&d, i, i * 6);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn compact_interleaved_inserts() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    for i in 6..=10 {
        ins(&d, i, i);
    }
    assert!(run(TableCommand::Compact { dir: tdir(&d) }).is_ok());
}

#[test]
fn compact_then_head() {
    let d = make_dir();
    setup(&d);
    for i in 1..=8 {
        ins(&d, i, i * 2);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn compact_then_keys() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 7);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Keys {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
