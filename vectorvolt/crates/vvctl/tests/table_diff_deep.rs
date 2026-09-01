//! Tests for Layer 276: vvctl table diff deep (VV-CTL-276).

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
fn diff_identical_zero_changes() {
    let d = make_dir();
    setup(&d);
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
fn diff_added_row_detected() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10],
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
fn diff_removed_row_detected() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Insert {
        dir: tdir2(&d),
        pk: 2,
        values: vec![20],
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
fn diff_changed_value_detected() {
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
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 99,
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
fn diff_missing_dir_a_errors() {
    let d = make_dir();
    setup2(&d);
    let res = run(TableCommand::Diff {
        a: d.path().join("nope"),
        b: tdir2(&d),
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn diff_missing_dir_b_errors() {
    let d = make_dir();
    setup(&d);
    let res = run(TableCommand::Diff {
        a: tdir(&d),
        b: d.path().join("nope"),
        output: OutputFormat::Table,
    });
    assert!(res.is_err());
}

#[test]
fn diff_json_ok() {
    let d = make_dir();
    setup(&d);
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    run(TableCommand::Diff {
        a: tdir(&d),
        b: tdir2(&d),
        output: OutputFormat::Json,
    })
    .unwrap();
}

#[test]
fn diff_table_ok() {
    let d = make_dir();
    setup(&d);
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
fn diff_empty_vs_empty_zero() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    run(TableCommand::Diff {
        a: tdir(&d),
        b: tdir2(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn diff_five_added_rows_ok() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10],
        })
        .unwrap();
    }
    run(TableCommand::Diff {
        a: tdir(&d),
        b: tdir2(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn diff_three_changed_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3u64 {
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
    for i in 1..=3u64 {
        run(TableCommand::Update {
            dir: tdir(&d),
            pk: i,
            col: 1,
            val: i * 100,
        })
        .unwrap();
    }
    run(TableCommand::Diff {
        a: tdir(&d),
        b: tdir2(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}

#[test]
fn diff_mixed_add_change_ok() {
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
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 2,
        values: vec![20],
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
fn diff_after_merge_ok() {
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
    run(TableCommand::Diff {
        a: tdir(&d),
        b: tdir2(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
}
