//! Layer 87 — WAL adaptive batch window (VV-WAL-011).
//!
//! Tests for `BatcherConfig::with_adaptive_window`, the `current_window_ns()`
//! accessor, and the feedback loop that adjusts the flush interval based on
//! the EMA flush latency.

use tempfile::TempDir;
use vectorvolt_wal::{BatcherConfig, GroupCommitBatcher, WalConfig, WalWriter};

fn writer(dir: &TempDir) -> WalWriter {
    let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);
    WalWriter::open(cfg).unwrap()
}

// ── BatcherConfig builder ─────────────────────────────────────────────────────

#[test]
fn batcher_config_adaptive_fields_default_zero() {
    let cfg = BatcherConfig::default();
    assert_eq!(cfg.adaptive_window_min_ns, 0);
    assert_eq!(cfg.adaptive_window_max_ns, 0);
}

#[test]
fn batcher_config_with_adaptive_window_sets_fields() {
    let cfg = BatcherConfig::default().with_adaptive_window(50_000, 5_000_000);
    assert_eq!(cfg.adaptive_window_min_ns, 50_000);
    assert_eq!(cfg.adaptive_window_max_ns, 5_000_000);
}

#[test]
fn wal_config_adaptive_fields_default_zero() {
    let cfg = WalConfig::default();
    assert_eq!(cfg.adaptive_window_min_ns, 0);
    assert_eq!(cfg.adaptive_window_max_ns, 0);
}

#[test]
fn wal_config_with_adaptive_window_sets_fields() {
    let cfg = WalConfig::default().with_adaptive_window(100_000, 2_000_000);
    assert_eq!(cfg.adaptive_window_min_ns, 100_000);
    assert_eq!(cfg.adaptive_window_max_ns, 2_000_000);
}

// ── current_window_ns accessor ────────────────────────────────────────────────

#[tokio::test]
async fn initial_window_matches_flush_interval() {
    let dir = TempDir::new().unwrap();
    let batcher = GroupCommitBatcher::start(
        writer(&dir),
        tokio::time::Duration::from_micros(400),
        4 * 1024 * 1024,
        256,
    );
    let window = batcher.current_window_ns();
    assert_eq!(
        window, 400_000,
        "initial window must equal flush_interval in ns"
    );
}

#[tokio::test]
async fn initial_window_with_adaptive_enabled_matches_flush_interval() {
    let dir = TempDir::new().unwrap();
    let cfg = BatcherConfig {
        flush_interval: tokio::time::Duration::from_micros(300),
        max_batch_bytes: 1024 * 1024,
        channel_capacity: 64,
        adaptive_window_min_ns: 100_000,
        adaptive_window_max_ns: 10_000_000,
    };
    let batcher = GroupCommitBatcher::start_with_config(writer(&dir), cfg);
    assert_eq!(batcher.current_window_ns(), 300_000);
}

#[tokio::test]
async fn current_window_is_readable_without_any_submits() {
    let dir = TempDir::new().unwrap();
    let batcher = GroupCommitBatcher::start(
        writer(&dir),
        tokio::time::Duration::from_micros(500),
        4 * 1024 * 1024,
        64,
    );
    let w = batcher.current_window_ns();
    assert!(w > 0, "window must be positive");
}

// ── adaptive disabled when min=max=0 ─────────────────────────────────────────

#[tokio::test]
async fn window_stays_at_flush_interval_when_adaptive_disabled() {
    let dir = TempDir::new().unwrap();
    let interval_ns: u64 = 500_000;
    let batcher = GroupCommitBatcher::start(
        writer(&dir),
        tokio::time::Duration::from_nanos(interval_ns),
        1024 * 1024,
        64,
    );
    let before = batcher.current_window_ns();
    // Submit and commit a batch to trigger a flush cycle.
    let fut = batcher.submit(b"probe".to_vec()).unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    let _ = fut.wait().await;
    let after = batcher.current_window_ns();
    assert_eq!(before, interval_ns);
    assert_eq!(
        after, interval_ns,
        "window must not change when adaptive disabled"
    );
}

// ── adaptive clamps to min/max ────────────────────────────────────────────────

#[tokio::test]
async fn adaptive_window_clamps_to_min_after_fast_flushes() {
    let dir = TempDir::new().unwrap();
    let min_ns: u64 = 200_000;
    let max_ns: u64 = 10_000_000;
    let cfg = BatcherConfig {
        flush_interval: tokio::time::Duration::from_micros(500),
        max_batch_bytes: 1024,
        channel_capacity: 256,
        adaptive_window_min_ns: min_ns,
        adaptive_window_max_ns: max_ns,
    };
    let batcher = GroupCommitBatcher::start_with_config(writer(&dir), cfg);

    // Flush several small batches quickly — EMA latency should be very low,
    // so the adaptive window should want to go below min and be clamped.
    for i in 0u32..20 {
        let fut = batcher.submit(i.to_le_bytes().to_vec()).unwrap();
        let _ = fut.wait().await;
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

    let w = batcher.current_window_ns();
    assert!(w >= min_ns, "window must not go below min_ns: got {w}");
    assert!(w <= max_ns, "window must not exceed max_ns: got {w}");
}

#[tokio::test]
async fn adaptive_window_never_exceeds_max() {
    let dir = TempDir::new().unwrap();
    let min_ns: u64 = 50_000;
    let max_ns: u64 = 1_000_000;
    let cfg = BatcherConfig {
        flush_interval: tokio::time::Duration::from_micros(500),
        max_batch_bytes: 1024,
        channel_capacity: 64,
        adaptive_window_min_ns: min_ns,
        adaptive_window_max_ns: max_ns,
    };
    let batcher = GroupCommitBatcher::start_with_config(writer(&dir), cfg);

    for i in 0u32..10 {
        let _ = batcher
            .submit(i.to_le_bytes().to_vec())
            .unwrap()
            .wait()
            .await;
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

    assert!(batcher.current_window_ns() <= max_ns);
}

// ── commits still work correctly with adaptive ────────────────────────────────

#[tokio::test]
async fn all_commits_succeed_with_adaptive_enabled() {
    let dir = TempDir::new().unwrap();
    let cfg = BatcherConfig {
        flush_interval: tokio::time::Duration::from_micros(200),
        max_batch_bytes: 4 * 1024 * 1024,
        channel_capacity: 512,
        adaptive_window_min_ns: 50_000,
        adaptive_window_max_ns: 5_000_000,
    };
    let batcher = GroupCommitBatcher::start_with_config(writer(&dir), cfg);

    let n = 50u32;
    let mut futs = Vec::new();
    for i in 0..n {
        futs.push(batcher.submit(i.to_le_bytes().to_vec()).unwrap());
    }
    let mut tips = Vec::new();
    for f in futs {
        tips.push(f.wait().await.unwrap());
    }
    let seqs: std::collections::HashSet<u64> = tips.iter().map(|t| t.sequence).collect();
    assert_eq!(
        seqs.len(),
        n as usize,
        "each commit must get a unique sequence"
    );
}

#[tokio::test]
async fn metrics_accumulate_during_adaptive_run() {
    let dir = TempDir::new().unwrap();
    let cfg = BatcherConfig {
        flush_interval: tokio::time::Duration::from_micros(200),
        max_batch_bytes: 4 * 1024 * 1024,
        channel_capacity: 256,
        adaptive_window_min_ns: 100_000,
        adaptive_window_max_ns: 2_000_000,
    };
    let batcher = GroupCommitBatcher::start_with_config(writer(&dir), cfg);

    for i in 0u32..30 {
        let _ = batcher
            .submit(i.to_le_bytes().to_vec())
            .unwrap()
            .wait()
            .await;
    }
    let snap = batcher.metrics().snapshot();
    assert_eq!(snap.frames_committed, 30);
    assert!(snap.fsync_count >= 1);
}

#[tokio::test]
async fn shutdown_drains_correctly_with_adaptive_enabled() {
    let dir = TempDir::new().unwrap();
    let cfg = BatcherConfig {
        flush_interval: tokio::time::Duration::from_millis(50),
        max_batch_bytes: 4 * 1024 * 1024,
        channel_capacity: 64,
        adaptive_window_min_ns: 1_000_000,
        adaptive_window_max_ns: 100_000_000,
    };
    let batcher = GroupCommitBatcher::start_with_config(writer(&dir), cfg);

    for i in 0u32..5 {
        let _ = batcher
            .submit(i.to_le_bytes().to_vec())
            .unwrap()
            .wait()
            .await;
    }
    batcher.shutdown();
    assert_eq!(batcher.metrics().frames_committed(), 5);
}

#[tokio::test]
async fn window_is_in_bounds_after_multiple_flushes() {
    let dir = TempDir::new().unwrap();
    let min_ns: u64 = 80_000;
    let max_ns: u64 = 3_000_000;
    let cfg = BatcherConfig {
        flush_interval: tokio::time::Duration::from_micros(500),
        max_batch_bytes: 1024,
        channel_capacity: 256,
        adaptive_window_min_ns: min_ns,
        adaptive_window_max_ns: max_ns,
    };
    let batcher = GroupCommitBatcher::start_with_config(writer(&dir), cfg);

    for i in 0u32..15 {
        let _ = batcher
            .submit(i.to_le_bytes().to_vec())
            .unwrap()
            .wait()
            .await;
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let w = batcher.current_window_ns();
    assert!(
        w >= min_ns && w <= max_ns,
        "window {w} must be in [{min_ns}, {max_ns}]"
    );
}
