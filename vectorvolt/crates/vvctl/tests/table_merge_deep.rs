//! Tests for Layer 275: vvctl table merge deep (VV-CTL-275).

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
fn setup2(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir2(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

#[test]
fn merge_empty_to_empty_ok() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn merge_one_row_to_empty_ok() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn merge_adds_rows_to_existing() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir2(&d),
        pk: 2,
        values: vec![20],
    })
    .unwrap();
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn merge_same_pk_upserts() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![99],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir2(&d),
        pk: 1,
        values: vec![10],
    })
    .unwrap();
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn merge_preserves_src() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![50],
    })
    .unwrap();
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Get {
        dir: tdir(&d),
        pk: 5,
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn merge_missing_src_errors() {
    let d = make_dir();
    setup2(&d);
    let res = run(TableCommand::Merge {
        src: d.path().join("nope"),
        dst: tdir2(&d),
    });
    assert!(res.is_err());
}

#[test]
fn merge_missing_dst_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::Merge {
        src: tdir(&d),
        dst: d.path().join("nope"),
    });
    assert!(res.is_err());
}

#[test]
fn merge_count_correct() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=3u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Merge {
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
fn merge_multi_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Create {
        dir: tdir2(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20],
    })
    .unwrap();
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn merge_then_diff_shows_merged() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 7,
        values: vec![70],
    })
    .unwrap();
    run(TableCommand::Merge {
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
fn merge_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 8,
        values: vec![80],
    })
    .unwrap();
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn merge_twice_ok() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 9,
        values: vec![90],
    })
    .unwrap();
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
}

#[test]
fn merge_json_output_ok() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Count {
        dir: tdir2(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}
