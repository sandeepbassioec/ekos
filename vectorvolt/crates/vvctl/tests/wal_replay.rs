//! Integration tests for Layer 136: vvctl wal replay (VV-WAL-012).

use std::path::PathBuf;
use tempfile::TempDir;
use vectorvolt_wal::{WalConfig, WalWriter};
use vvctl::commands::wal::{ReplayOutput, WalCommand, cmd_replay};

fn make_dir() -> TempDir {
    TempDir::new().unwrap()
}

/// Create a WAL with `n` frames in `dir` and return the wal subdir path.
fn write_wal_frames(dir: &TempDir, n: usize) -> PathBuf {
    let wal_dir = dir.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    let cfg = WalConfig::default()
        .with_dir(wal_dir.to_str().unwrap())
        .with_fsync(false);
    let writer = WalWriter::open(cfg).unwrap();
    for i in 0..n {
        writer.append(format!("record-{}", i).into_bytes()).unwrap();
    }
    writer.flush().unwrap();
    wal_dir
}

// ── 1. Missing dir errors ─────────────────────────────────────────────────────

#[test]
fn replay_missing_dir_errors() {
    let dir = PathBuf::from("/nonexistent/wal/path/that/does/not/exist");
    let result = cmd_replay(&dir, 0, &ReplayOutput::Table);
    assert!(result.is_err(), "expected error for missing dir");
}

// ── 2. Empty dir succeeds with 0 records ─────────────────────────────────────

#[test]
fn replay_empty_dir_succeeds() {
    let tmp = make_dir();
    let wal_dir = tmp.path().join("wal");
    std::fs::create_dir_all(&wal_dir).unwrap();
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// ── 3. Table output format succeeds ──────────────────────────────────────────

#[test]
fn replay_table_output_succeeds() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 5);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// ── 4. JSON output format succeeds ───────────────────────────────────────────

#[test]
fn replay_json_output_succeeds() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 5);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Json).unwrap();
}

// ── 5. Limit 0 reads all records ─────────────────────────────────────────────

#[test]
fn replay_limit_zero_reads_all() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 10);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// ── 6. With segments after writes returns Ok ─────────────────────────────────

#[test]
fn replay_with_segments_after_writes() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 3);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// ── 7. Single record insert, table output ────────────────────────────────────

#[test]
fn replay_one_record_table() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 1);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// ── 8. Single record insert, json output ─────────────────────────────────────

#[test]
fn replay_one_record_json() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 1);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Json).unwrap();
}

// ── 9. Limit = 1 caps reads ──────────────────────────────────────────────────

#[test]
fn replay_limit_one_succeeds() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 20);
    cmd_replay(&wal_dir, 1, &ReplayOutput::Table).unwrap();
}

// ── 10. Limit = 100, fewer records available ─────────────────────────────────

#[test]
fn replay_limit_100_with_fewer_records() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 5);
    cmd_replay(&wal_dir, 100, &ReplayOutput::Table).unwrap();
}

// ── 11. Many records, table output ───────────────────────────────────────────

#[test]
fn replay_many_records_table() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 50);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Table).unwrap();
}

// ── 12. Many records, json output ────────────────────────────────────────────

#[test]
fn replay_many_records_json() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 50);
    cmd_replay(&wal_dir, 0, &ReplayOutput::Json).unwrap();
}

// ── 13. WalCommand::Replay variant dispatches correctly ──────────────────────

#[test]
fn replay_via_run_dispatch() {
    let tmp = make_dir();
    let wal_dir = write_wal_frames(&tmp, 4);
    let cmd = WalCommand::Replay {
        dir: wal_dir,
        limit: 0,
        output: ReplayOutput::Table,
    };
    vvctl::commands::wal::run(cmd).unwrap();
}
