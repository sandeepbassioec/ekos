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
fn head_empty_table() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn head_n_larger_than_rows() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn head_n_equals_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 3);
    }
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn head_n_smaller_than_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn head_n_one() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 7);
    }
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn head_json_output() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 3,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn tail_empty_table() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Tail {
            dir: tdir(&d),
            n: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_n_larger_than_rows() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    ins(&d, 2, 20);
    assert!(
        run(TableCommand::Tail {
            dir: tdir(&d),
            n: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_n_smaller_than_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i * 2);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tdir(&d),
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_n_one() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 9);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tdir(&d),
            n: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_json_output() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::Tail {
            dir: tdir(&d),
            n: 2,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn head_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i * 4);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Head {
            dir: tdir(&d),
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn tail_after_delete() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i * 4);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 6,
    })
    .unwrap();
    assert!(
        run(TableCommand::Tail {
            dir: tdir(&d),
            n: 3,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
