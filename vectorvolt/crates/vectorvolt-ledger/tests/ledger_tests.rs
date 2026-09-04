use std::sync::Arc;
use tempfile::TempDir;
use vectorvolt_ledger::{Ledger, LedgerError};

fn setup() -> (Ledger, TempDir) {
    let dir = TempDir::new().unwrap();
    let ledger = Ledger::open(dir.path()).unwrap();
    (ledger, dir)
}

#[test]
fn create_account() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(1, "Alice", "USD").unwrap();
    let acct = ledger.accounts.get(1).unwrap().unwrap();
    assert_eq!(acct.name, "Alice");
}

#[test]
fn get_account() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(2, "Bob", "USD").unwrap();
    let acct = ledger.accounts.get(2).unwrap();
    assert!(acct.is_some());
    assert_eq!(acct.unwrap().id, 2);
}

#[test]
fn balance_starts_zero() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(3, "Carol", "USD").unwrap();
    assert_eq!(ledger.balance(3).unwrap(), 0);
}

#[test]
fn post_debit_credit() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(10, "A", "USD").unwrap();
    ledger.accounts.create(11, "B", "USD").unwrap();
    let id = ledger.post(10, 11, 500, "transfer").unwrap();
    assert!(id >= 1);
}

#[test]
fn balance_after_post_correct() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(20, "Debit", "USD").unwrap();
    ledger.accounts.create(21, "Credit", "USD").unwrap();
    ledger.post(20, 21, 1000, "test").unwrap();
    assert_eq!(ledger.balance(20).unwrap(), -1000);
    assert_eq!(ledger.balance(21).unwrap(), 1000);
}

#[test]
fn double_entry_debit_credit_sums_to_zero() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(30, "X", "USD").unwrap();
    ledger.accounts.create(31, "Y", "USD").unwrap();
    ledger.post(30, 31, 250, "check").unwrap();
    assert_eq!(ledger.trial_balance().unwrap(), 0);
}

#[test]
fn entries_for_returns_correct() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(40, "P", "USD").unwrap();
    ledger.accounts.create(41, "Q", "USD").unwrap();
    ledger.post(40, 41, 100, "a").unwrap();
    ledger.post(40, 41, 200, "b").unwrap();
    let entries = ledger.entries_for(40).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn multiple_posts() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(50, "M", "USD").unwrap();
    ledger.accounts.create(51, "N", "USD").unwrap();
    for i in 0..5 {
        ledger.post(50, 51, i * 10, "multi").unwrap();
    }
    assert_eq!(ledger.trial_balance().unwrap(), 0);
}

#[test]
fn trial_balance_zero() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(60, "A1", "USD").unwrap();
    ledger.accounts.create(61, "A2", "USD").unwrap();
    ledger.accounts.create(62, "A3", "USD").unwrap();
    ledger.post(60, 61, 500, "t1").unwrap();
    ledger.post(61, 62, 200, "t2").unwrap();
    assert_eq!(ledger.trial_balance().unwrap(), 0);
}

#[test]
fn post_with_invalid_account_returns_err() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(70, "Valid", "USD").unwrap();
    let result = ledger.post(70, 9999, 100, "bad");
    assert!(matches!(result, Err(LedgerError::AccountNotFound(9999))));
}

#[test]
fn list_accounts() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(80, "L1", "USD").unwrap();
    ledger.accounts.create(81, "L2", "USD").unwrap();
    let list = ledger.accounts.list().unwrap();
    assert_eq!(list.len(), 2);
}

#[test]
fn concurrent_posts_trial_balance_zero() {
    let dir = TempDir::new().unwrap();
    let ledger = Arc::new(Ledger::open(dir.path()).unwrap());
    ledger.accounts.create(90, "C1", "USD").unwrap();
    ledger.accounts.create(91, "C2", "USD").unwrap();

    let mut handles = Vec::new();
    for i in 0..10u64 {
        let l = Arc::clone(&ledger);
        handles.push(std::thread::spawn(move || {
            l.post(90, 91, (i * 10) as i64, "concurrent").unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(ledger.trial_balance().unwrap(), 0);
}

#[test]
fn entries_sorted_by_timestamp() {
    let (ledger, _dir) = setup();
    ledger.accounts.create(100, "T1", "USD").unwrap();
    ledger.accounts.create(101, "T2", "USD").unwrap();
    ledger.post(100, 101, 10, "first").unwrap();
    ledger.post(100, 101, 20, "second").unwrap();
    let entries = ledger.entries_for(100).unwrap();
    let ts: Vec<u64> = entries.iter().map(|e| e.timestamp_ns).collect();
    let mut sorted = ts.clone();
    sorted.sort();
    assert_eq!(ts, sorted);
}
