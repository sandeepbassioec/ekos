use tempfile::TempDir;
use vvctl::commands::wal::{ReplayOutput, WalCommand, run};

fn make_wal_dir() -> TempDir {
    TempDir::new().unwrap()
}

#[test]
fn checkpoint_empty_dir_ok() {
    let d = make_wal_dir();
    let r = run(WalCommand::Checkpoint {
        dir: d.path().to_path_buf(),
    });
    assert!(r.is_ok());
}

#[test]
fn checkpoint_creates_file() {
    let d = make_wal_dir();
    run(WalCommand::Checkpoint {
        dir: d.path().to_path_buf(),
    })
    .unwrap();
    // verify dir still exists
    assert!(d.path().exists());
}

#[test]
fn checkpoint_twice_ok() {
    let d = make_wal_dir();
    run(WalCommand::Checkpoint {
        dir: d.path().to_path_buf(),
    })
    .unwrap();
    let r = run(WalCommand::Checkpoint {
        dir: d.path().to_path_buf(),
    });
    assert!(r.is_ok());
}

#[test]
fn checkpoint_then_status_ok() {
    let d = make_wal_dir();
    run(WalCommand::Checkpoint {
        dir: d.path().to_path_buf(),
    })
    .unwrap();
    let r = run(WalCommand::Status {
        dir: d.path().to_string_lossy().to_string(),
    });
    assert!(r.is_ok());
}

#[test]
fn checkpoint_then_verify_ok() {
    let d = make_wal_dir();
    run(WalCommand::Checkpoint {
        dir: d.path().to_path_buf(),
    })
    .unwrap();
    let r = run(WalCommand::Verify {
        dir: d.path().to_string_lossy().to_string(),
        output: ReplayOutput::Table,
    });
    assert!(r.is_ok());
}

#[test]
fn checkpoint_then_replay_ok() {
    let d = make_wal_dir();
    run(WalCommand::Checkpoint {
        dir: d.path().to_path_buf(),
    })
    .unwrap();
    let r = run(WalCommand::Replay {
        dir: d.path().to_path_buf(),
        limit: 0,
        output: ReplayOutput::Table,
    });
    assert!(r.is_ok());
}

#[test]
fn checkpoint_dir_must_exist_or_be_created() {
    let d = make_wal_dir();
    let sub = d.path().join("subwal");
    std::fs::create_dir_all(&sub).unwrap();
    let r = run(WalCommand::Checkpoint { dir: sub });
    assert!(r.is_ok());
}

#[test]
fn status_before_checkpoint() {
    let d = make_wal_dir();
    let r = run(WalCommand::Status {
        dir: d.path().to_string_lossy().to_string(),
    });
    assert!(r.is_ok());
}

#[test]
fn checkpoint_then_stats_ok() {
    let d = make_wal_dir();
    run(WalCommand::Checkpoint {
        dir: d.path().to_path_buf(),
    })
    .unwrap();
    let r = run(WalCommand::Stats {
        dir: d.path().to_path_buf(),
        output: ReplayOutput::Table,
    });
    assert!(r.is_ok());
}

#[test]
fn checkpoint_json_replay() {
    let d = make_wal_dir();
    run(WalCommand::Checkpoint {
        dir: d.path().to_path_buf(),
    })
    .unwrap();
    let r = run(WalCommand::Replay {
        dir: d.path().to_path_buf(),
        limit: 0,
        output: ReplayOutput::Json,
    });
    assert!(r.is_ok());
}

#[test]
fn checkpoint_export_ok() {
    let d = make_wal_dir();
    run(WalCommand::Checkpoint {
        dir: d.path().to_path_buf(),
    })
    .unwrap();
    let r = run(WalCommand::Export {
        dir: d.path().to_path_buf(),
        limit: None,
        output: ReplayOutput::Table,
    });
    assert!(r.is_ok());
}

#[test]
fn checkpoint_multiple_times_same_dir() {
    let d = make_wal_dir();
    for _ in 0..3 {
        run(WalCommand::Checkpoint {
            dir: d.path().to_path_buf(),
        })
        .unwrap();
    }
    assert!(d.path().exists());
}

#[test]
fn verify_after_multiple_checkpoints() {
    let d = make_wal_dir();
    for _ in 0..3 {
        run(WalCommand::Checkpoint {
            dir: d.path().to_path_buf(),
        })
        .unwrap();
    }
    let r = run(WalCommand::Verify {
        dir: d.path().to_string_lossy().to_string(),
        output: ReplayOutput::Table,
    });
    assert!(r.is_ok());
}
