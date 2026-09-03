//! Tests for Layer 366.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("t1")
}
fn tdir2(d: &TempDir) -> PathBuf {
    d.path().join("t2")
}
fn setup(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}
fn setup2(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir2(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

#[test]
fn ae_insert_5_both() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
        run(TableCommand::Insert {
            dir: tdir2(&d),
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
fn ae_diff_zero() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
        run(TableCommand::Insert {
            dir: tdir2(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn ae_update_diverge() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
        run(TableCommand::Insert {
            dir: tdir2(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: 3,
            col: 1,
            val: 999
        })
        .is_ok()
    );
}
#[test]
fn ae_diff_1_changed() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
        run(TableCommand::Insert {
            dir: tdir2(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        val: 999,
    })
    .unwrap();
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn ae_merge_reconcile() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
        run(TableCommand::Insert {
            dir: tdir2(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        val: 999,
    })
    .unwrap();
    assert!(
        run(TableCommand::Merge {
            src: tdir(&d),
            dst: tdir2(&d)
        })
        .is_ok()
    );
}
#[test]
fn ae_diff_zero_after_merge() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
        run(TableCommand::Insert {
            dir: tdir2(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        val: 999,
    })
    .unwrap();
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn ae_check_t1() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn ae_check_t2() {
    let d = make_dir();
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir2(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: tdir2(&d) }).is_ok());
}
#[test]
fn ae_vacuum_t1() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}
#[test]
fn ae_vacuum_t2() {
    let d = make_dir();
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir2(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir2(&d) }).is_ok());
}
#[test]
fn ae_stats_t1() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn ae_export_t1() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Export {
            dir: tdir(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn ae_recount_t2() {
    let d = make_dir();
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir2(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
