//! Integration test for wiring `recover_pending_transactions` into
//! `vvctl serve`'s actual startup path (the documented follow-up from MVCC
//! Phase 3, which left it as a standalone-tested-but-unwired function).
//!
//! Simulates a crash: a "PREPARING" intent record is written for a
//! transaction whose per-table `commit_txn` was never actually invoked, then
//! `run_startup_recovery` (the function `vvctl serve` now calls before
//! accepting connections) is exercised directly against a real
//! `TableRegistry` — proving the wiring works without needing to spawn the
//! full server (threads, sockets, Ctrl-C handler).

use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry, TableTxn, TxnIntentLog};
use vvctl::commands::serve::run_startup_recovery;

fn open_registry(dir: &TempDir) -> Arc<TableRegistry> {
    Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap())
}

#[test]
fn recovers_dangling_transaction_at_startup() {
    let dir = TempDir::new().unwrap();
    let registry = open_registry(&dir);
    registry
        .create_table("orders", vec!["pk".to_owned(), "total".to_owned()])
        .unwrap();

    let engine = registry.get("orders").unwrap();
    let txn_id = engine.begin();
    engine.insert_txn(txn_id, 1, vec![1, 100]).unwrap();
    // Simulate a crash: the intent log says COMMIT was attempted, but
    // commit_txn was never actually called on the engine.
    let log_path = registry.base_dir().join("_txn_intent.log");
    let log = TxnIntentLog::open(&log_path).unwrap();
    log.prepare(
        "crash-test-txn",
        vec![TableTxn {
            table: "orders".to_owned(),
            txn_id,
        }],
    )
    .unwrap();

    // Row must not be visible yet — commit_txn was never called.
    assert!(engine.get(1).unwrap().is_none());

    run_startup_recovery(&registry);

    // Recovery must have rolled the dangling transaction forward.
    assert!(engine.get(1).unwrap().is_some());
    let records = log.read_all().unwrap();
    assert!(
        records.iter().any(|r| r.txn_uuid == "crash-test-txn"
            && r.state == vectorvolt_engine::IntentState::Committed)
    );
}

#[test]
fn no_intent_log_file_is_a_silent_no_op() {
    let dir = TempDir::new().unwrap();
    let registry = open_registry(&dir);
    // No _txn_intent.log exists at all — must not panic or error.
    run_startup_recovery(&registry);
}

#[test]
fn already_committed_transaction_is_left_alone() {
    let dir = TempDir::new().unwrap();
    let registry = open_registry(&dir);
    registry
        .create_table("orders", vec!["pk".to_owned(), "total".to_owned()])
        .unwrap();
    let engine = registry.get("orders").unwrap();
    let txn_id = engine.begin();
    engine.insert_txn(txn_id, 2, vec![2, 200]).unwrap();
    engine.commit_txn(txn_id).unwrap();

    let log_path = registry.base_dir().join("_txn_intent.log");
    let log = TxnIntentLog::open(&log_path).unwrap();
    log.prepare(
        "already-done-txn",
        vec![TableTxn {
            table: "orders".to_owned(),
            txn_id,
        }],
    )
    .unwrap();

    // Row is already visible (real commit already happened).
    assert!(engine.get(2).unwrap().is_some());
    run_startup_recovery(&registry);
    // Recovery must not error or duplicate anything — row still there once.
    assert!(engine.get(2).unwrap().is_some());
}
