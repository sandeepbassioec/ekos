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
fn upsert_insert_new_row() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![10]
        })
        .is_ok()
    );
}

#[test]
fn upsert_update_existing_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 1,
            values: vec![99]
        })
        .is_ok()
    );
}

#[test]
fn upsert_then_get() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::UpsertRow {
        dir: tdir(&d),
        pk: 5,
        values: vec![50],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn upsert_idempotent_same_values() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::UpsertRow {
        dir: tdir(&d),
        pk: 2,
        values: vec![20],
    })
    .unwrap();
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 2,
            values: vec![20]
        })
        .is_ok()
    );
}

#[test]
fn upsert_multiple_different_pks() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        assert!(
            run(TableCommand::UpsertRow {
                dir: tdir(&d),
                pk: i,
                values: vec![i * 10]
            })
            .is_ok()
        );
    }
}

#[test]
fn upsert_overwrites_on_conflict() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 30);
    run(TableCommand::UpsertRow {
        dir: tdir(&d),
        pk: 3,
        values: vec![300],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn upsert_then_count() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn upsert_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 7, 70);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 7,
            values: vec![777]
        })
        .is_ok()
    );
}

#[test]
fn upsert_after_delete() {
    let d = make_dir();
    setup(&d);
    ins(&d, 9, 90);
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 9,
    })
    .unwrap();
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 9,
            values: vec![99]
        })
        .is_ok()
    );
}

#[test]
fn upsert_zero_value() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 10,
            values: vec![0]
        })
        .is_ok()
    );
}

#[test]
fn upsert_max_value() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: 11,
            values: vec![u64::MAX]
        })
        .is_ok()
    );
}

#[test]
fn upsert_then_scan() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 3],
        })
        .unwrap();
    }
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
fn upsert_then_stats() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        run(TableCommand::UpsertRow {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 4],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
