//! Tests for Layer 290: ColStats deep correctness (VV-CTL-290).

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
fn col_stats_fresh_table_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_after_5_inserts_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_col_val_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_json_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "val".into(),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn col_stats_table_format_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_missing_dir_errors() {
    let d = make_dir();
    let bad = d.path().join("no_such");
    assert!(
        run(TableCommand::ColStats {
            dir: bad,
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn col_stats_nonexistent_col_errors() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "nonexistent".into(),
            output: OutputFormat::Table
        })
        .is_err()
    );
}

#[test]
fn col_stats_col_pk_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "pk".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_multi_col_table_ok() {
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
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "a".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_after_flush_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_after_compact_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_after_truncate_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn col_stats_after_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Vacuum { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::ColStats {
            dir: tdir(&d),
            col: "val".into(),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
