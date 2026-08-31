//! Integration tests for Layer 158: vvctl table diff (VV-CTL-158).

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn create(dir: &Path) {
    run(TableCommand::Create {
        dir: dir.to_path_buf(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

fn insert(dir: &Path, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: dir.to_path_buf(),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn diff_both_empty_table_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    let b = d.path().join("b");
    create(&a);
    create(&b);
    assert!(
        run(TableCommand::Diff {
            a,
            b,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_both_empty_json_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    let b = d.path().join("b");
    create(&a);
    create(&b);
    assert!(
        run(TableCommand::Diff {
            a,
            b,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn diff_identical_tables_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    let b = d.path().join("b");
    create(&a);
    create(&b);
    insert(&a, 1, 10);
    insert(&b, 1, 10);
    assert!(
        run(TableCommand::Diff {
            a,
            b,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_added_rows_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    let b = d.path().join("b");
    create(&a);
    create(&b);
    insert(&b, 1, 10);
    assert!(
        run(TableCommand::Diff {
            a,
            b,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_removed_rows_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    let b = d.path().join("b");
    create(&a);
    create(&b);
    insert(&a, 1, 10);
    assert!(
        run(TableCommand::Diff {
            a,
            b,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_changed_rows_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    let b = d.path().join("b");
    create(&a);
    create(&b);
    insert(&a, 1, 10);
    insert(&b, 1, 99);
    assert!(
        run(TableCommand::Diff {
            a,
            b,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_json_with_diff_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    let b = d.path().join("b");
    create(&a);
    create(&b);
    insert(&a, 1, 1);
    insert(&b, 2, 2);
    assert!(
        run(TableCommand::Diff {
            a,
            b,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn diff_missing_a_errors() {
    let d = make_dir();
    let b = d.path().join("b");
    create(&b);
    assert!(
        run(TableCommand::Diff {
            a: PathBuf::from("/no/such"),
            b,
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn diff_missing_b_errors() {
    let d = make_dir();
    let a = d.path().join("a");
    create(&a);
    assert!(
        run(TableCommand::Diff {
            a,
            b: PathBuf::from("/no/such"),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn diff_multiple_rows_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    let b = d.path().join("b");
    create(&a);
    create(&b);
    for i in 0..5u64 {
        insert(&a, i, i);
    }
    for i in 3..8u64 {
        insert(&b, i, i);
    }
    assert!(
        run(TableCommand::Diff {
            a,
            b,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_same_table_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    create(&a);
    insert(&a, 1, 1);
    assert!(
        run(TableCommand::Diff {
            a: a.clone(),
            b: a,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_json_identical_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    let b = d.path().join("b");
    create(&a);
    create(&b);
    for i in 0..3u64 {
        insert(&a, i, i);
        insert(&b, i, i);
    }
    assert!(
        run(TableCommand::Diff {
            a,
            b,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn diff_empty_vs_filled_ok() {
    let d = make_dir();
    let a = d.path().join("a");
    let b = d.path().join("b");
    create(&a);
    create(&b);
    for i in 0..5u64 {
        insert(&b, i, i);
    }
    assert!(
        run(TableCommand::Diff {
            a,
            b,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
