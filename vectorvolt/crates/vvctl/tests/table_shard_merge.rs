//! Tests for Layer 362.
use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::table::{ExportFormat, OutputFormat, TableCommand, run};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}
fn mk(d: &TempDir, sub: &str) -> PathBuf {
    d.path().join(sub)
}
fn setup_at(p: &PathBuf) {
    run(TableCommand::Create {
        dir: p.clone(),
        columns: vec!["pk:u64".into(), "val:u64".into()],
    })
    .unwrap();
}

#[test]
fn shard_merge_sa_5rows() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    for i in 0..5u64 {
        run(TableCommand::Insert {
            dir: sa.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: sa,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn shard_merge_sb_5rows() {
    let d = make_dir();
    let sb = mk(&d, "sb");
    setup_at(&sb);
    for i in 5..10u64 {
        run(TableCommand::Insert {
            dir: sb.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Count {
            dir: sb,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn shard_merge_copy_sb_into_sa() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    let sb = mk(&d, "sb");
    setup_at(&sb);
    for i in 0..5u64 {
        run(TableCommand::Insert {
            dir: sa.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    for i in 5..10u64 {
        run(TableCommand::Insert {
            dir: sb.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(run(TableCommand::Merge { src: sb, dst: sa }).is_ok());
}
#[test]
fn shard_merge_count_10() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    let sb = mk(&d, "sb");
    setup_at(&sb);
    for i in 0..5u64 {
        run(TableCommand::Insert {
            dir: sa.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    for i in 5..10u64 {
        run(TableCommand::Insert {
            dir: sb.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Merge {
        src: sb,
        dst: sa.clone(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Count {
            dir: sa,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn shard_merge_scan_10() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    let sb = mk(&d, "sb");
    setup_at(&sb);
    for i in 0..5u64 {
        run(TableCommand::Insert {
            dir: sa.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    for i in 5..10u64 {
        run(TableCommand::Insert {
            dir: sb.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    run(TableCommand::Merge {
        src: sb,
        dst: sa.clone(),
    })
    .unwrap();
    assert!(
        run(TableCommand::Scan {
            dir: sa,
            col: 0,
            lo: 0,
            hi: u64::MAX,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn shard_merge_diff_ok() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    let sb = mk(&d, "sb");
    setup_at(&sb);
    for i in 0..3u64 {
        run(TableCommand::Insert {
            dir: sa.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    for i in 0..3u64 {
        run(TableCommand::Insert {
            dir: sb.clone(),
            pk: i,
            values: vec![i],
        })
        .unwrap();
    }
    assert!(
        run(TableCommand::Diff {
            a: sa,
            b: sb,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn shard_merge_check_sa() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    run(TableCommand::Insert {
        dir: sa.clone(),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Check { dir: sa }).is_ok());
}
#[test]
fn shard_merge_vacuum_sa() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    run(TableCommand::Insert {
        dir: sa.clone(),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Vacuum { dir: sa }).is_ok());
}
#[test]
fn shard_merge_stats_ok() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    run(TableCommand::Insert {
        dir: sa.clone(),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Stats {
            dir: sa,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn shard_merge_recount_ok() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    run(TableCommand::Insert {
        dir: sa.clone(),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Recount {
            dir: sa,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn shard_merge_info_ok() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    run(TableCommand::Insert {
        dir: sa.clone(),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Info {
            dir: sa,
            output: OutputFormat::Table
        })
        .is_ok()
    );
}
#[test]
fn shard_merge_export_ok() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    run(TableCommand::Insert {
        dir: sa.clone(),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(
        run(TableCommand::Export {
            dir: sa,
            format: ExportFormat::Json
        })
        .is_ok()
    );
}
#[test]
fn shard_merge_compact_ok() {
    let d = make_dir();
    let sa = mk(&d, "sa");
    setup_at(&sa);
    run(TableCommand::Insert {
        dir: sa.clone(),
        pk: 1,
        values: vec![1],
    })
    .unwrap();
    assert!(run(TableCommand::Compact { dir: sa }).is_ok());
}
