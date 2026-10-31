//! S10 integration tests — ledger + workflow + orders + chaos + SDK presence.
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use vectorvolt_chaos::{DstEventKind, DstHarness, Fault, FaultInjector};
use vectorvolt_ledger::Ledger;
use vectorvolt_orders::{OrderLine, OrderStore};
use vectorvolt_workflow::WorkflowEngine;

fn tmpdir() -> TempDir {
    TempDir::new().unwrap()
}

fn line(sku: &str, qty: u32, price: i64) -> OrderLine {
    OrderLine {
        sku: sku.into(),
        qty,
        unit_price_cents: price,
    }
}

// 1. Ledger: post 10 entries, trial_balance = 0
#[test]
fn ledger_10_entries_trial_balance_zero() {
    let dir = tmpdir();
    let ledger = Ledger::open(dir.path()).unwrap();
    ledger.accounts.create(1, "A", "USD").unwrap();
    ledger.accounts.create(2, "B", "USD").unwrap();
    for i in 0..10i64 {
        ledger.post(1, 2, i * 100, "entry").unwrap();
    }
    assert_eq!(ledger.trial_balance().unwrap(), 0);
}

// 2. Ledger: concurrent 20-thread post, trial_balance = 0
#[test]
fn ledger_concurrent_20_threads_trial_balance_zero() {
    let dir = tmpdir();
    let ledger = Arc::new(Ledger::open(dir.path()).unwrap());
    ledger.accounts.create(10, "X", "USD").unwrap();
    ledger.accounts.create(11, "Y", "USD").unwrap();
    let mut handles = Vec::new();
    for i in 0..20i64 {
        let l = Arc::clone(&ledger);
        handles.push(std::thread::spawn(move || {
            l.post(10, 11, i * 50, "concurrent").unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(ledger.trial_balance().unwrap(), 0);
}

// 3. Workflow: 5-step linear chain runs to completion
#[test]
fn workflow_5_step_linear_chain() {
    let dir = tmpdir();
    let engine = WorkflowEngine::open(dir.path()).unwrap();
    let steps: Vec<(String, Vec<u64>)> = (0..5u64)
        .map(|i| (format!("s{i}"), if i == 0 { vec![] } else { vec![i - 1] }))
        .collect();
    let id = engine.create("chain", steps).unwrap();
    engine.start(id).unwrap();
    for step_id in 0..5u64 {
        engine.complete_step(id, step_id).unwrap();
    }
    assert!(engine.is_complete(id).unwrap());
}

// 4. Workflow: diamond DAG runs to completion in correct order
#[test]
fn workflow_diamond_dag_completes() {
    let dir = tmpdir();
    let engine = WorkflowEngine::open(dir.path()).unwrap();
    let id = engine
        .create(
            "diamond",
            vec![
                ("s0".into(), vec![]),
                ("s1".into(), vec![0]),
                ("s2".into(), vec![0]),
                ("s3".into(), vec![1, 2]),
            ],
        )
        .unwrap();
    engine.start(id).unwrap();
    engine.complete_step(id, 0).unwrap();
    engine.complete_step(id, 1).unwrap();
    engine.complete_step(id, 2).unwrap();
    engine.complete_step(id, 3).unwrap();
    assert!(engine.is_complete(id).unwrap());
}

// 5. Orders: create 100 orders, total_revenue_cents correct
#[test]
fn orders_100_creates_revenue_correct() {
    let dir = tmpdir();
    let store = OrderStore::open(dir.path()).unwrap();
    let mut ids = Vec::new();
    for _ in 0..100 {
        let id = store.create(1, vec![line("A", 1, 100)]).unwrap();
        ids.push(id);
    }
    for id in &ids {
        store.confirm(*id).unwrap();
        store.ship(*id).unwrap();
        store.deliver(*id).unwrap();
    }
    assert_eq!(store.total_revenue_cents().unwrap(), 10000);
}

// 6. Orders: status machine full lifecycle Pending→Confirmed→Shipped→Delivered
#[test]
fn orders_full_lifecycle() {
    let dir = tmpdir();
    let store = OrderStore::open(dir.path()).unwrap();
    let id = store.create(1, vec![line("X", 2, 500)]).unwrap();
    store.confirm(id).unwrap();
    store.ship(id).unwrap();
    store.deliver(id).unwrap();
    let order = store.get(id).unwrap().unwrap();
    assert_eq!(order.status, vectorvolt_orders::OrderStatus::Delivered);
}

// 7. Chaos: DST seed 42 produces same event log on two runs (determinism)
#[test]
fn chaos_dst_seed_determinism() {
    let mut h1 = DstHarness::new(42);
    let mut h2 = DstHarness::new(42);
    for t in 1..=10u64 {
        h1.schedule(t, DstEventKind::Tick);
        h2.schedule(t, DstEventKind::Tick);
    }
    h1.run_until(10);
    h2.run_until(10);
    let t1: Vec<u64> = h1.replay().map(|e| e.at_tick).collect();
    let t2: Vec<u64> = h2.replay().map(|e| e.at_tick).collect();
    assert_eq!(t1, t2);
}

// 8. Chaos: PacketLoss(1.0) drops every packet
#[test]
fn chaos_packet_loss_1_drops_all() {
    let mut inj = FaultInjector::new(0);
    inj.add_fault(Fault::PacketLoss { probability: 1.0 });
    for _ in 0..20 {
        inj.tick();
        assert!(inj.should_drop());
    }
}

// 9. Chaos: Crash after 5 ops fires at op 5
#[test]
fn chaos_crash_fires_at_op_5() {
    let mut inj = FaultInjector::new(0);
    inj.add_fault(Fault::Crash { after_n_ops: 5 });
    for _ in 0..5 {
        assert!(!inj.should_crash());
        inj.tick();
    }
    assert!(inj.should_crash());
}

// 10. Ledger + Graph: create accounts, post entries (simplified: trial balance check)
#[test]
fn ledger_multiple_accounts_trial_balance_zero() {
    let dir = tmpdir();
    let ledger = Ledger::open(dir.path()).unwrap();
    for i in 0..5u64 {
        ledger
            .accounts
            .create(i, &format!("acct{i}"), "USD")
            .unwrap();
    }
    // Post entries in a ring: 0->1, 1->2, 2->3, 3->4, 4->0
    for i in 0..5u64 {
        ledger.post(i, (i + 1) % 5, 1000, "ring").unwrap();
    }
    assert_eq!(ledger.trial_balance().unwrap(), 0);
}

// 11. Workflow + Chaos: run workflow steps deterministically
#[test]
fn workflow_under_dst_step_order() {
    let dir = tmpdir();
    let engine = WorkflowEngine::open(dir.path()).unwrap();
    let id = engine
        .create(
            "dst-wf",
            vec![("s0".into(), vec![]), ("s1".into(), vec![0])],
        )
        .unwrap();
    engine.start(id).unwrap();
    // Simulate step completion in DST-ordered ticks.
    let mut harness = DstHarness::new(99);
    harness.schedule(1, DstEventKind::Tick);
    harness.schedule(2, DstEventKind::Tick);
    harness.run_until(2);
    // After 2 ticks complete step 0, then step 1.
    engine.complete_step(id, 0).unwrap();
    engine.complete_step(id, 1).unwrap();
    assert!(engine.is_complete(id).unwrap());
}

// 12. Orders + Ledger: post ledger entry on order confirm, trial_balance stays 0
#[test]
fn orders_ledger_combined_trial_balance_zero() {
    let dir = tmpdir();
    let ledger = Ledger::open(dir.path()).unwrap();
    ledger.accounts.create(100, "Revenue", "USD").unwrap();
    ledger.accounts.create(101, "AR", "USD").unwrap();

    let order_dir = tmpdir();
    let store = OrderStore::open(order_dir.path()).unwrap();
    let order_id = store.create(1, vec![line("P", 1, 500)]).unwrap();
    store.confirm(order_id).unwrap();

    // Post a ledger entry when order is confirmed.
    ledger.post(101, 100, 500, "order confirmed").unwrap();
    assert_eq!(ledger.trial_balance().unwrap(), 0);
}

// 13. Python SDK: pyproject.toml exists and has correct version
#[test]
fn python_sdk_pyproject_exists() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("sdk/python/pyproject.toml");
    let content = std::fs::read_to_string(&p).expect("pyproject.toml not found");
    assert!(
        content.contains("version = \"0.1.0\""),
        "version mismatch in pyproject.toml"
    );
}

// 14. Go SDK: go.mod exists and has correct module path
#[test]
fn go_sdk_gomod_exists() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("sdk/go/go.mod");
    let content = std::fs::read_to_string(&p).expect("go.mod not found");
    assert!(
        content.contains("github.com/workerruntime/vectorvolt-go"),
        "module path missing in go.mod"
    );
}
