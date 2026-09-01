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
fn sum_field_basic() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
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
fn sum_field_empty_table() {
    let d = make_dir();
    setup(&d);
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
fn avg_field_basic() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 5);
    }
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn avg_field_single_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 42);
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn median_field_odd_count() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins(&d, 3, 30);
    assert!(
        run(TableCommand::MedianField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn median_field_even_count() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins(&d, 3, 30);
    ins(&d, 4, 40);
    assert!(
        run(TableCommand::MedianField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scalar_min_basic() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 50);
    ins(&d, 2, 10);
    ins(&d, 3, 80);
    assert!(
        run(TableCommand::ScalarMin {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scalar_max_basic() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 50);
    ins(&d, 2, 10);
    ins(&d, 3, 80);
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
fn scalar_min_single_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 7);
    assert!(
        run(TableCommand::ScalarMin {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn scalar_max_single_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 7);
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
fn sum_field_json_output() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 {
        ins(&d, i, i * 100);
    }
    assert!(
        run(TableCommand::SumField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn avg_field_json_output() {
    let d = make_dir();
    setup(&d);
    for i in 1..=3 {
        ins(&d, i, i * 100);
    }
    assert!(
        run(TableCommand::AvgField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn aggregates_after_flush_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i * 9);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::SumField {
            dir: tdir(&d),
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
