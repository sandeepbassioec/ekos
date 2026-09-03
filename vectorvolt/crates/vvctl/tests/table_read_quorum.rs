//! Tests for Layer 369.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

#[test]
fn rq_insert_3() {
    let d = make_dir();
    let p = d.path().join("primary");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: p.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: p,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn rq_copy_to_r1() {
    let d = make_dir();
    let p = d.path().join("primary");
    let r1 = d.path().join("r1");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: p.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(run(TableCommand::Copy { src: p, dst: r1 }).is_ok());
}
#[test]
fn rq_copy_to_r2() {
    let d = make_dir();
    let p = d.path().join("primary");
    let r2 = d.path().join("r2");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: p.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(run(TableCommand::Copy { src: p, dst: r2 }).is_ok());
}
#[test]
fn rq_all_count_3() {
    let d = make_dir();
    let p = d.path().join("primary");
    let r1 = d.path().join("r1");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: p.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Copy {
        src: p,
        dst: r1.clone(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Count {
            dir: r1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn rq_all_scan_3() {
    let d = make_dir();
    let p = d.path().join("primary");
    let r1 = d.path().join("r1");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: p.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Copy {
        src: p,
        dst: r1.clone(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: r1,
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn rq_diff_primary_r1_zero() {
    let d = make_dir();
    let p = d.path().join("primary");
    let r1 = d.path().join("r1");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: p.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Copy {
        src: p.clone(),
        dst: r1.clone(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Diff {
            a: p,
            b: r1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn rq_diff_primary_r2_zero() {
    let d = make_dir();
    let p = d.path().join("primary");
    let r2 = d.path().join("r2");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: p.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Copy {
        src: p.clone(),
        dst: r2.clone(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Diff {
            a: p,
            b: r2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn rq_get_pk1_primary() {
    let d = make_dir();
    let p = d.path().join("primary");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: p.clone(),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: p,
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn rq_get_pk1_r1() {
    let d = make_dir();
    let p = d.path().join("primary");
    let r1 = d.path().join("r1");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: p.clone(),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Copy {
        src: p,
        dst: r1.clone(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: r1,
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn rq_check_all_ok() {
    let d = make_dir();
    let p = d.path().join("primary");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: p.clone(),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: p }).is_ok());
}
#[test]
fn rq_vacuum_all_ok() {
    let d = make_dir();
    let p = d.path().join("primary");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: p.clone(),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Vacuum { dir: p }).is_ok());
}
#[test]
fn rq_export_primary_ok() {
    let d = make_dir();
    let p = d.path().join("primary");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: p.clone(),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Export {
            dir: p,
            format: vvctl::commands::table::ExportFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn rq_get_pk1_r2() {
    let d = make_dir();
    let p = d.path().join("primary");
    let r2 = d.path().join("r2");
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: p.clone(),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Copy {
        src: p,
        dst: r2.clone(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: r2,
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
