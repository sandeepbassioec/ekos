//! Integration tests for Layer 142: vvctl table tail (VV-CTL-142).

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tbl(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}

fn create(d: &TempDir) {
    run(TableCommand::Create {
        dir: tbl(d),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

fn insert(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tbl(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn tail_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_n1_fewer_than_total() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_n_greater_than_rows_returns_all() {
    let d = make_dir();
    create(&d);
    for i in 0..3u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_json_empty() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 5,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn tail_json_with_rows() {
    let d = make_dir();
    create(&d);
    for i in 0..10u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 3,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn tail_default_n10() {
    let d = make_dir();
    create(&d);
    for i in 0..20u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_n0_returns_ok() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 1);
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_missing_dir_errors() {
    assert!(
        run(TableCommand::Tail {
            dir: PathBuf::from("/no/such/dir"),
            n: 5,
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn tail_single_row() {
    let d = make_dir();
    create(&d);
    insert(&d, 42, 99);
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_n5_of_5_rows() {
    let d = make_dir();
    create(&d);
    for i in 1..=5u64 {
        insert(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_json_single_row() {
    let d = make_dir();
    create(&d);
    insert(&d, 7, 77);
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn tail_table_output_n3_of_10() {
    let d = make_dir();
    create(&d);
    for i in 0..10u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_exact_count_matches() {
    let d = make_dir();
    create(&d);
    for i in 0..7u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tbl(&d),
            n: 7,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
