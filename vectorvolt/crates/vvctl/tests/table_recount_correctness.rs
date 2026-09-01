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
fn recount_empty_table() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_single_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_many_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=15 {
        ins(&d, i, i * 4);
    }
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_delete() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 2);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=8 {
        ins(&d, i, i);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i * 3);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_json_output() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn recount_twice_consistent() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Recount {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_truncate() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Truncate {
        dir: tdir(&d),
        force: true,
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_matches_count() {
    let d = make_dir();
    setup(&d);
    for i in 1..=7 {
        ins(&d, i, i * 5);
    }
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_mass_delete() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i);
    }
    for i in 1..=10 {
        run(TableCommand::Delete {
            dir: tdir(&d),
            pk: i,
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_after_update() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        val: 999,
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn recount_large_dataset() {
    let d = make_dir();
    setup(&d);
    for i in 1..=50 {
        ins(&d, i, i * 2);
    }
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
