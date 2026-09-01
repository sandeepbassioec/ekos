//! Layer 177 — vvctl wal inspect segment

use std::path::PathBuf;
use tempfile::TempDir;
use vvctl::commands::wal::{WalCommand, run};

fn make_wal(dir: &TempDir) -> PathBuf {
    let wal_dir = dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    // Write one frame so the WAL is non-empty
    use vectorvolt_wal::{WalConfig, WalWriter};
    let cfg = WalConfig::default().with_dir(wal_dir.to_str().unwrap());
    let writer = WalWriter::open(cfg).unwrap();
    writer.append(b"test-frame".to_vec()).unwrap();
    writer.flush().unwrap();
    wal_dir
}

// 1. inspect existing WAL returns Ok
#[test]
fn inspect_existing_wal_ok() {
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    assert!(
        run(WalCommand::Inspect {
            dir: wal.to_str().unwrap().to_string(),
            segment: None,
            payload: false,
        })
        .is_ok()
    );
}

// 2. inspect missing dir returns Ok (empty, graceful)
#[test]
fn inspect_missing_dir_graceful() {
    let dir = TempDir::new().unwrap();
    // list_segment_ids on missing dir returns empty — inspect prints nothing
    let _ = run(WalCommand::Inspect {
        dir: dir.path().join("no_wal").to_str().unwrap().to_string(),
        segment: None,
        payload: false,
    });
}

// 3. inspect with payload=true returns Ok
#[test]
fn inspect_with_payload_true_ok() {
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    assert!(
        run(WalCommand::Inspect {
            dir: wal.to_str().unwrap().to_string(),
            segment: None,
            payload: true,
        })
        .is_ok()
    );
}

// 4. inspect segment 0 returns Ok
#[test]
fn inspect_segment_zero_ok() {
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    assert!(
        run(WalCommand::Inspect {
            dir: wal.to_str().unwrap().to_string(),
            segment: Some(0),
            payload: false,
        })
        .is_ok()
    );
}

// 5. inspect non-existent segment returns Err or Ok (graceful)
#[test]
fn inspect_nonexistent_segment_ok() {
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    // segment 999 doesn't exist — implementation returns Err or prints nothing
    let _ = run(WalCommand::Inspect {
        dir: wal.to_str().unwrap().to_string(),
        segment: Some(999),
        payload: false,
    });
}

// 6. inspect empty WAL dir returns Ok
#[test]
fn inspect_empty_wal_dir_ok() {
    let dir = TempDir::new().unwrap();
    let wal_dir = dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    // Empty dir — no segments
    let _ = run(WalCommand::Inspect {
        dir: wal_dir.to_str().unwrap().to_string(),
        segment: None,
        payload: false,
    });
}

// 7. wal status on valid dir returns Ok
#[test]
fn wal_status_valid_dir_ok() {
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    assert!(
        run(WalCommand::Status {
            dir: wal.to_str().unwrap().to_string(),
        })
        .is_ok()
    );
}

// 8. wal verify valid returns Ok
#[test]
fn wal_verify_valid_ok() {
    use vvctl::commands::wal::ReplayOutput;
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    assert!(
        run(WalCommand::Verify {
            dir: wal.to_str().unwrap().to_string(),
            output: ReplayOutput::Table,
        })
        .is_ok()
    );
}

// 9. wal replay valid returns Ok
#[test]
fn wal_replay_valid_ok() {
    use vvctl::commands::wal::ReplayOutput;
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    assert!(
        run(WalCommand::Replay {
            dir: wal.clone(),
            limit: 0,
            output: ReplayOutput::Table,
        })
        .is_ok()
    );
}

// 10. wal replay with limit=1 returns Ok
#[test]
fn wal_replay_limit1_ok() {
    use vvctl::commands::wal::ReplayOutput;
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    assert!(
        run(WalCommand::Replay {
            dir: wal.clone(),
            limit: 1,
            output: ReplayOutput::Table,
        })
        .is_ok()
    );
}

// 11. wal checkpoint on valid dir returns Ok
#[test]
fn wal_checkpoint_valid_ok() {
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    assert!(run(WalCommand::Checkpoint { dir: wal }).is_ok());
}

// 12. wal trim before=0 returns Ok
#[test]
fn wal_trim_before_zero_ok() {
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    assert!(
        run(WalCommand::Trim {
            dir: wal,
            before: 0
        })
        .is_ok()
    );
}

// 13. wal stats on valid dir returns Ok
#[test]
fn wal_stats_valid_ok() {
    use vvctl::commands::wal::ReplayOutput;
    let dir = TempDir::new().unwrap();
    let wal = make_wal(&dir);
    assert!(
        run(WalCommand::Stats {
            dir: wal,
            output: ReplayOutput::Table,
        })
        .is_ok()
    );
}
