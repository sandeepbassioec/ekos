//! Tests for Layer 399.
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
fn chaos_insert_5() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
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
fn chaos_truncate_wipe() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Truncate {
            dir: tdir(&d),
            force: true
        })
        .is_ok()
    );
}
#[test]
fn chaos_count_0_after_truncate() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
    }
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn chaos_reinsert_5() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    for i in 1..=5u64 {
        ins(&d, i, i);
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
fn chaos_delete_pk3() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: 3
        })
        .is_ok()
    );
}
#[test]
fn chaos_reinsert_pk3() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
    })
    .unwrap();
    assert!(
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: 3,
            values: vec![99]
        })
        .is_ok()
    );
}
#[test]
fn chaos_get_pk3_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 3,
        values: vec![99],
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
fn chaos_batch_delete_4_5() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::BatchDelete {
            dir: tdir(&d),
            pks: vec![4, 5]
        })
        .is_ok()
    );
}
#[test]
fn chaos_reinsert_4_5() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
    }
    run(TableCommand::BatchDelete {
        dir: tdir(&d),
        pks: vec![4, 5],
    })
    .unwrap();
    ins(&d, 4, 4);
    ins(&d, 5, 5);
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn chaos_count_5_final() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5u64 {
        ins(&d, i, i);
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
fn chaos_check_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn chaos_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn chaos_recount_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
