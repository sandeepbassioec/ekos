//! Tests for Layer 392.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn setup4col(d: &TempDir) {
    run(TableCommand::Create {
        dir: tdir(d),
        columns: vec![
            "pk:u64".into(),
            "x:u64".into(),
            "y:u64".into(),
            "z:u64".into(),
        ],
    })
    .unwrap();
}

#[test]
fn vecins_insert_5() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20, i * 30],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_scan_col1() {
    let d = make_dir();
    setup4col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_get_pk1() {
    let d = make_dir();
    setup4col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 1,
        values: vec![10, 20, 30],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_get_pk2() {
    let d = make_dir();
    setup4col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 2,
        values: vec![20, 40, 60],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_get_pk3() {
    let d = make_dir();
    setup4col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 3,
        values: vec![30, 60, 90],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_get_pk4() {
    let d = make_dir();
    setup4col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 4,
        values: vec![40, 80, 120],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 4,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_get_pk5() {
    let d = make_dir();
    setup4col(&d);
    run(TableCommand::Insert {
        dir: tdir(&d),
        pk: 5,
        values: vec![50, 100, 150],
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_sum_col1() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20, i * 30],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::SumField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_avg_col2() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20, i * 30],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_scalar_min_col3() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20, i * 30],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::ScalarMin {
            dir: tdir(&d),
            col: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_scalar_max_col1() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i * 10, i * 20, i * 30],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::ScalarMax {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn vecins_check_ok() {
    let d = make_dir();
    setup4col(&d);
    assert!(run(TableCommand::Check { dir: tdir(&d) }).is_ok());
}
#[test]
fn vecins_count_5() {
    let d = make_dir();
    setup4col(&d);
    for i in 1..=5u64 {
        run(TableCommand::Insert {
            dir: tdir(&d),
            pk: i,
            values: vec![i, i, i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
