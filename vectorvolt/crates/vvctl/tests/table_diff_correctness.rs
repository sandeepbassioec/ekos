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
fn ins(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tdir(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}
fn ins2(d: &TempDir, pk: u64, val: u64) {
    run(TableCommand::Insert {
        dir: tdir2(d),
        pk,
        values: vec![val],
    })
    .unwrap();
}

#[test]
fn diff_identical_tables() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 1, 10);
    ins2(&d, 1, 10);
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_empty_tables() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_different_values() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 1, 10);
    ins2(&d, 1, 99);
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_extra_row_in_a() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins2(&d, 1, 10);
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_extra_row_in_b() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 1, 10);
    ins2(&d, 1, 10);
    ins2(&d, 2, 20);
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_json_output() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 1, 10);
    ins2(&d, 1, 20);
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn diff_many_rows_same() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=10 {
        ins(&d, i, i * 5);
        ins2(&d, i, i * 5);
    }
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_many_rows_different() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=5 {
        ins(&d, i, i);
        ins2(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_a_empty_b_has_rows() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=3 {
        ins2(&d, i, i * 7);
    }
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_a_has_rows_b_empty() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=3 {
        ins(&d, i, i * 7);
    }
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_after_flush() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 1, 100);
    ins2(&d, 1, 200);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_symmetric_check() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 1, 10);
    ins2(&d, 2, 20);
    assert!(
        run(TableCommand::Diff {
            a: tdir2(&d),
            b: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn diff_partial_overlap() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    ins2(&d, 2, 20);
    ins2(&d, 3, 30);
    assert!(
        run(TableCommand::Diff {
            a: tdir(&d),
            b: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
