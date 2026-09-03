//! Tests for Layer 372.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn t1(d: &TempDir) -> PathBuf {
    d.path().join("d1").join("tenant1")
}
fn t2(d: &TempDir) -> PathBuf {
    d.path().join("d2").join("tenant2")
}

#[test]
fn tenant_insert_t1() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t1(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: t1(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: t1(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn tenant_insert_t2() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t2(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: t2(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: t2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn tenant_count_t1_3() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t1(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: t1(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: t1(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn tenant_count_t2_3() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t2(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: t2(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: t2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn tenant_get_pk1_t1() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t1(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: t1(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: t1(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn tenant_get_pk1_t2() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t2(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: t2(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: t2(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn tenant_diff_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t1(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Create {
        dir: t2(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: t1(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
        run(TableCommand::Insert {
            dir: t2(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Diff {
            a: t1(&d),
            b: t2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn tenant_update_t1_isolate() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t1(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: t1(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Update {
            dir: t1(&d),
            pk: 1,
            col: 1,
            val: 99
        })
        .is_ok()
    );
}
#[test]
fn tenant_check_t1() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t1(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: t1(&d) }).is_ok());
}
#[test]
fn tenant_check_t2() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t2(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: t2(&d) }).is_ok());
}
#[test]
fn tenant_vacuum_t1() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t1(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    assert!(run(TableCommand::Vacuum { dir: t1(&d) }).is_ok());
}
#[test]
fn tenant_stats_t2() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t2(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: t2(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: t2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn tenant_recount_t1_export_t2() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: t1(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Create {
        dir: t2(&d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: t1(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: t2(&d),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    run(TableCommand::Recount {
        dir: t1(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
    assert!(
        run(TableCommand::Export {
            dir: t2(&d),
            format: ExportFormat::Json
        })
        .is_ok()
    );
}
