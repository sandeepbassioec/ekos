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
fn window_rank_single_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn window_rank_multiple_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 3,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn window_rank_first_row() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 1,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn window_rank_last_row() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 5,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn window_rank_nonexistent_pk() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 999,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn window_rank_json_output() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 7);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 2,
            col: 1,
            output: OutputFormat::Json
        })
        .is_ok()
    );
}

#[test]
fn window_rank_tied_values() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 50);
    ins(&d, 2, 50);
    ins(&d, 3, 50);
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 2,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn window_rank_after_flush() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 3);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 3,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn window_rank_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i * 4);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 4,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn window_rank_after_update() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 5);
    }
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 3,
        col: 1,
        val: 999,
    })
    .unwrap();
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 3,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn window_rank_pk_column() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 2);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 3,
            col: 0,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn window_rank_all_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, (6 - i) * 10);
    }
    for i in 1..=5 {
        assert!(
            run(TableCommand::WindowRank {
                dir: tdir(&d),
                pk: i,
                col: 1,
                output: OutputFormat::Table
            })
            .is_ok()
        );
    }
}

#[test]
fn window_rank_large_dataset() {
    let d = make_dir();
    setup(&d);
    for i in 1..=20 {
        ins(&d, i, i * 6);
    }
    assert!(
        run(TableCommand::WindowRank {
            dir: tdir(&d),
            pk: 10,
            col: 1,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
