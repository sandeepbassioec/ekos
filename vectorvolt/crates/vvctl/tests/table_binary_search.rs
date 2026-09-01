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
fn get_exact_match_single_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 42, 999);
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 42,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn get_first_key_many_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=20 {
        ins(&d, i, i * 10);
    }
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
fn get_last_key_many_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=20 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 20,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn get_middle_key_many_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=20 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 10,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn get_miss_returns_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 999,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn get_on_empty_table() {
    let d = make_dir();
    setup(&d);
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
fn get_boundary_zero() {
    let d = make_dir();
    setup(&d);
    ins(&d, 0, 1);
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn get_large_pk() {
    let d = make_dir();
    setup(&d);
    ins(&d, u64::MAX / 2, 77);
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: u64::MAX / 2,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn get_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
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
fn get_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i * 3);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 7,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn get_json_output() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 33);
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 3,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn get_many_keys_sequentially() {
    let d = make_dir();
    setup(&d);
    for i in 1..=15 {
        ins(&d, i * 3, i * 7);
    }
    for i in 1..=15 {
        assert!(
            run(TableCommand::Get {
                dir: tdir(&d),
                pk: i * 3,
                output: OutputFormat::Table
            })
            .is_ok()
        );
    }
}

#[test]
fn get_boundary_adjacent_keys() {
    let d = make_dir();
    setup(&d);
    ins(&d, 100, 1);
    ins(&d, 101, 2);
    ins(&d, 102, 3);
    assert!(
        run(TableCommand::Get {
            dir: tdir(&d),
            pk: 101,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
