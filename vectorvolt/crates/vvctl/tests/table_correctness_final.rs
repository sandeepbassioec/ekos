//! Tests for Layer 301: Final correctness suite (VV-CTL-301).

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
fn final_create_schema_ok() {
    let d = make_dir();
    setup(&d);
    assert!(
        run(TableCommand::Schema {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn final_insert_get_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
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
fn final_update_get_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 100);
    run(TableCommand::Update {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        val: 999,
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
fn final_delete_count_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Delete {
        dir: tdir(&d),
        pk: 3,
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
fn final_flush_compact_vacuum_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Compact { dir: tdir(&d) }).unwrap();
    assert!(run(TableCommand::Vacuum { dir: tdir(&d) }).is_ok());
}

#[test]
fn final_scan_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=10 {
        ins(&d, i, i * 10);
    }
    assert!(
        run(TableCommand::Scan {
            dir: tdir(&d),
            col: 1,
            lo: 10,
            hi: 100,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn final_keys_pk_range_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Keys {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
    assert!(
        run(TableCommand::PkRange {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn final_stats_info_size_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::Stats {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
    run(TableCommand::Info {
        dir: tdir(&d),
        output: OutputFormat::Table,
    })
    .unwrap();
    assert!(
        run(TableCommand::Size {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn final_head_tail_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=8 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Head {
        dir: tdir(&d),
        n: 3,
        output: OutputFormat::Table,
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

#[test]
fn final_check_recount_ok() {
    let d = make_dir();
    setup(&d);
    for i in 1..=5 {
        ins(&d, i, i * 10);
    }
    run(TableCommand::Flush { dir: tdir(&d) }).unwrap();
    run(TableCommand::Check { dir: tdir(&d) }).unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: tdir(&d),
            output: OutputFormat::Table
        })
        .is_ok()
    );
}

#[test]
fn final_cas_correctness_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 50);
    run(TableCommand::CompareAndSwap {
        dir: tdir(&d),
        pk: 1,
        col: 1,
        expected: 50,
        replacement: 200,
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
fn final_increment_decrement_ok() {
    let d = make_dir();
    setup(&d);
    ins(&d, 1, 10);
    run(TableCommand::IncrementField {
        dir: tdir(&d),
        pk: 1,
        col: 1,
    })
    .unwrap();
    run(TableCommand::DecrementField {
        dir: tdir(&d),
        pk: 1,
        col: 1,
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
fn final_missing_dir_errors() {
    let d = make_dir();
    assert!(
        run(TableCommand::Count {
            dir: d.path().join("nope"),
            output: OutputFormat::Table
        })
        .is_err()
    );
}
