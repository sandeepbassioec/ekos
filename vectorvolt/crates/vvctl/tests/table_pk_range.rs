//! Integration tests for Layer 143: vvctl table pk-range (VV-CTL-143).

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
fn pk_range_empty_table_ok() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pk_range_empty_json() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn pk_range_single_row() {
    let d = make_dir();
    create(&d);
    insert(&d, 42, 0);
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pk_range_multiple_rows_table() {
    let d = make_dir();
    create(&d);
    for i in [5u64, 1, 10, 3] {
        insert(&d, i, 0);
    }
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pk_range_multiple_rows_json() {
    let d = make_dir();
    create(&d);
    for i in [5u64, 1, 10, 3] {
        insert(&d, i, 0);
    }
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn pk_range_missing_dir_errors() {
    assert!(
        run(TableCommand::PkRange {
            dir: PathBuf::from("/no/such"),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn pk_range_min_is_first_pk() {
    let d = make_dir();
    create(&d);
    for i in [2u64, 5, 8] {
        insert(&d, i, 0);
    }
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pk_range_two_rows() {
    let d = make_dir();
    create(&d);
    insert(&d, 1, 0);
    insert(&d, 100, 0);
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pk_range_large_pks() {
    let d = make_dir();
    create(&d);
    insert(&d, u64::MAX / 2, 0);
    insert(&d, 0, 0);
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pk_range_json_null_on_empty() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn pk_range_table_output_empty_message() {
    let d = make_dir();
    create(&d);
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pk_range_sequential_pks() {
    let d = make_dir();
    create(&d);
    for i in 0..5u64 {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn pk_range_reversed_insert_order() {
    let d = make_dir();
    create(&d);
    for i in (0..5u64).rev() {
        insert(&d, i, i);
    }
    assert!(
        run(TableCommand::PkRange {
            dir: tbl(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
