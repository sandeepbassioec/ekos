//! Tests for Layer 214: vvctl table count-distinct (VV-CTL-214).

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
fn count_distinct_empty_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn count_distinct_single_row_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn count_distinct_all_same_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, 99);
    }
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn count_distinct_all_different_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..5u64 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn count_distinct_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 7);
    ins(&d, 2, 7);
    ins(&d, 3, 8);
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn count_distinct_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::CountDistinct {
            dir: d.path().join("nope"),
            col: 1,
            output: OutputFormat::Table
        })
        .is_err()
    );
}
#[test]
fn count_distinct_col_zero_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 0);
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn count_distinct_ten_rows_ok() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i % 3);
    }
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn count_distinct_json_ten_rows() {
    let d = make_dir();
    setup(&d);
    for i in 0..10u64 {
        ins(&d, i, i % 5);
    }
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn count_distinct_col_stored() {
    let cmd = TableCommand::CountDistinct {
        dir: PathBuf::from("/tmp"),
        col: 2,
        output: OutputFormat::Table,
    };
    if let TableCommand::CountDistinct { col, .. } = cmd {
        assert_eq!(col, 2);
    } else {
        panic!();
    }
}
#[test]
fn count_distinct_output_stored() {
    let cmd = TableCommand::CountDistinct {
        dir: PathBuf::from("/tmp"),
        col: 1,
        output: OutputFormat::Json,
    };
    if let TableCommand::CountDistinct { output, .. } = cmd {
        assert!(matches!(output, OutputFormat::Json));
    } else {
        panic!();
    }
}
#[test]
fn count_distinct_multi_col_ok() {
    let d = make_dir();
    run(TableCommand::Create {
        dir: tdir(&d),
        columns: vec!["pk:u64".into(), "a:u64".into(), "b:u64".into()],
    })
    .unwrap();
    for i in 0..5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i % 2, i % 3],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn count_distinct_mixed_values_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins(&d, 3, 10);
    ins(&d, 4, 30);
    assert!(
        run(TableCommand::CountDistinct {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
