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
fn merge_empty_src_into_dst() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins2(&d, 1, 10);
    assert!(
        run(TableCommand::Merge {
            src: tdir(&d),
            dst: tdir2(&d)
        })
        .is_ok()
    );
}

#[test]
fn merge_rows_into_empty_dst() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 1, 10);
    assert!(
        run(TableCommand::Merge {
            src: tdir(&d),
            dst: tdir2(&d)
        })
        .is_ok()
    );
}

#[test]
fn merge_non_overlapping_pks() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=3 {
        ins(&d, i, i * 10);
    }
    for i in 4..=6 {
        ins2(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Merge {
            src: tdir(&d),
            dst: tdir2(&d)
        })
        .is_ok()
    );
}

#[test]
fn merge_overlapping_pks() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 1, 100);
    ins2(&d, 1, 200);
    assert!(
        run(TableCommand::Merge {
            src: tdir(&d),
            dst: tdir2(&d)
        })
        .is_ok()
    );
}

#[test]
fn merge_then_count() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=3 {
        ins(&d, i, i);
    }
    for i in 4..=6 {
        ins2(&d, i, i);
    }
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Count {
            dir: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn merge_then_scan_dst() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=4 {
        ins(&d, i, i * 5);
    }
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: tdir2(&d),
            col: 1,
            lo: 0,
            hi: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn merge_both_empty() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    assert!(
        run(TableCommand::Merge {
            src: tdir(&d),
            dst: tdir2(&d)
        })
        .is_ok()
    );
}

#[test]
fn merge_after_flush() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 5, 50);
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Merge {
            src: tdir(&d),
            dst: tdir2(&d)
        })
        .is_ok()
    );
}

#[test]
fn merge_large_src() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=20 {
        ins(&d, i, i * 3);
    }
    assert!(
        run(TableCommand::Merge {
            src: tdir(&d),
            dst: tdir2(&d)
        })
        .is_ok()
    );
}

#[test]
fn merge_then_get_merged_row() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 7, 77);
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Get {
            dir: tdir2(&d),
            pk: 7,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn merge_then_stats() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=5 {
        ins(&d, i, i * 2);
    }
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn merge_idempotent_empty_src() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    for i in 1..=3 {
        ins2(&d, i, i);
    }
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Merge {
            src: tdir(&d),
            dst: tdir2(&d)
        })
        .is_ok()
    );
}

#[test]
fn merge_then_keys() {
    let d = make_dir();
    setup(&d);
    setup2(&d);
    ins(&d, 10, 100);
    ins(&d, 20, 200);
    run(TableCommand::Merge {
        src: tdir(&d),
        dst: tdir2(&d),
    })
    .unwrap();
    assert!(
        run(TableCommand::Keys {
            dir: tdir2(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
