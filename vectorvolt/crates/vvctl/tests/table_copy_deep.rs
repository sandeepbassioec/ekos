//! Tests for Layer 274: vvctl table copy deep (VV-CTL-274).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn tdir2(d: &TempDir) -> PathBuf {
    d.path().join("tbl2")
}
fn setup(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

#[test]
fn copy_empty_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn copy_one_row_table_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn copy_ten_row_table_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn copy_preserves_all_values() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![50],
    })
    .unwrap();
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir2(&d),
        pk: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn copy_src_unchanged() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 3,
        values: vec![30],
    })
    .unwrap();
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn copy_dst_same_row_count() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir2(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn copy_then_diff_empty() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Diff {
        a: tdir(&d),
        b: tdir2(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn copy_missing_src_errors() {
    let d = make_dir();
    let res = run(TableCommand::Copy {
        src: d.path().join("nope"),
        dst: tdir2(&d),
    });
    assert!(res.is_err());
}

#[test]
fn copy_existing_dst_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn copy_multi_col_table_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20],
    })
    .unwrap();
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir2(&d),
        pk: 1,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn copy_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 7,
        values: vec![70],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn copy_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 8,
        values: vec![80],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn copy_large_table_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..20u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}
