use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn tdir(d: &TempDir) -> PathBuf {
    d.path().join("tbl")
}
fn tdst(d: &TempDir) -> PathBuf {
    d.path().join("tbl_dst")
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
fn copy_empty_table() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Copy {
            src: tdir(&d),
            dst: tdst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_single_row() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    assert!(
        run(TableCommand::Copy {
            src: tdir(&d),
            dst: tdst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_many_rows() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i * 7);
    }
    assert!(
        run(TableCommand::Copy {
            src: tdir(&d),
            dst: tdst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_then_get_from_dst() {
    let d = make_dir();
    setup(&d);
    ins(&d, 5, 50);
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdst(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdst(&d),
            pk: 5,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn copy_then_count_dst() {
    let d = make_dir();
    setup(&d);
    for i in 1..=7 {
        ins(&d, i, i);
    }
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdst(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tdst(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn copy_then_scan_dst() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 4);
    }
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdst(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdst(&d),
            col: 1,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn copy_preserves_src() {
    let d = make_dir();
    setup(&d);
    for i in 1..=4 {
        ins(&d, i, i * 3);
    }
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdst(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn copy_after_flush() {
    let d = make_dir();
    setup(&d);
    ins(&d, 3, 30);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Copy {
            src: tdir(&d),
            dst: tdst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_after_compact() {
    let d = make_dir();
    setup(&d);
    for i in 1..=6 {
        ins(&d, i, i * 9);
    }
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Copy {
            src: tdir(&d),
            dst: tdst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_then_stats_dst() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 11);
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdst(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdst(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn copy_then_keys_dst() {
    let d = make_dir();
    setup(&d);
    ins(&d, 10, 100);
    ins(&d, 20, 200);
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdst(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Keys {
            dir: tdst(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn copy_large_table() {
    let d = make_dir();
    setup(&d);
    for i in 1..=30 {
        ins(&d, i, i * 13);
    }
    assert!(
        run(TableCommand::Copy {
            src: tdir(&d),
            dst: tdst(&d)
        })
        .is_ok()
    );
}

#[test]
fn copy_then_schema_dst() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 1);
    run(TableCommand::Copy {
        src: tdir(&d),
        dst: tdst(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Schema {
            dir: tdst(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
