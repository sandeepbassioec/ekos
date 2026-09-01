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
fn vacuum_empty_table() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn vacuum_with_data() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i * 5);
    }
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn vacuum_after_many_deletes() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i * 3);
    }
    for i in 1..=8 {
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: i,
        })
        .unwrap();
    }
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn vacuum_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn vacuum_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn vacuum_twice_idempotent() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn vacuum_then_get() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 77);
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
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
fn vacuum_then_count() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 2);
    }
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn vacuum_then_scan() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i * 4);
    }
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
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
fn vacuum_then_stats() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 6);
    }
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn vacuum_then_insert() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 {
        ins(&d, i, i);
    }
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: 99,
            values: vec![99]
        })
        .is_ok()
    );
}

#[test]
fn vacuum_after_updates() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    for i in 1..=5 {
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: i,
            col: 1,
            val: i * 100,
        })
        .unwrap();
    }
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn vacuum_large_dataset() {
    let d = make_dir();
    setup(&d);
    for i in 1..=40 {
        ins(&d, i, i * 7);
    }
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
