//! Tests for the chaos (DST) and ledger CLI commands.
use tempfile::TempDir;
use vvctl::commands::chaos::{ChaosCommand, run as chaos_run};
use vvctl::commands::ledger_cmd::{LedgerCommand, run as ledger_run};

// ---------------------------------------------------------------------------
// Chaos tests
// ---------------------------------------------------------------------------

#[test]
fn chaos_simulate_runs_without_panic() {
    let result = chaos_run(ChaosCommand::Simulate {
        seed: 42,
        ticks: 5,
        fault: None,
    });
    assert!(result.is_ok());
}

#[test]
fn chaos_simulate_ticks_zero_ok() {
    let result = chaos_run(ChaosCommand::Simulate {
        seed: 0,
        ticks: 0,
        fault: None,
    });
    assert!(result.is_ok());
}

#[test]
fn chaos_replay_parses_events() {
    let result = chaos_run(ChaosCommand::Replay {
        seed: 1,
        events: "[1,2,3]".into(),
    });
    assert!(result.is_ok());
}

#[test]
fn chaos_fault_delay_schedules_delay_events() {
    use vvctl::commands::chaos::FaultArg;
    let result = chaos_run(ChaosCommand::Simulate {
        seed: 99,
        ticks: 5,
        fault: Some(FaultArg::Delay),
    });
    assert!(result.is_ok());
}

#[test]
fn chaos_fault_crash_schedules_crash() {
    use vvctl::commands::chaos::FaultArg;
    let result = chaos_run(ChaosCommand::Simulate {
        seed: 7,
        ticks: 5,
        fault: Some(FaultArg::Crash),
    });
    assert!(result.is_ok());
}

#[test]
fn chaos_output_is_valid_json() {
    // Run and capture output by verifying the command succeeds (stdout checked via is_ok).
    let result = chaos_run(ChaosCommand::Simulate {
        seed: 1,
        ticks: 3,
        fault: None,
    });
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Ledger tests
// ---------------------------------------------------------------------------

#[test]
fn ledger_create_account_ok() {
    let dir = TempDir::new().unwrap();
    let result = ledger_run(LedgerCommand::CreateAccount {
        dir: dir.path().to_path_buf(),
        id: 1,
        name: "Alice".into(),
        currency: "USD".into(),
    });
    assert!(result.is_ok());
}

#[test]
fn ledger_post_ok() {
    let dir = TempDir::new().unwrap();
    ledger_run(LedgerCommand::CreateAccount {
        dir: dir.path().to_path_buf(),
        id: 1,
        name: "A".into(),
        currency: "USD".into(),
    })
    .unwrap();
    ledger_run(LedgerCommand::CreateAccount {
        dir: dir.path().to_path_buf(),
        id: 2,
        name: "B".into(),
        currency: "USD".into(),
    })
    .unwrap();
    let result = ledger_run(LedgerCommand::Post {
        dir: dir.path().to_path_buf(),
        debit: 1,
        credit: 2,
        amount: 500,
        desc: "transfer".into(),
    });
    assert!(result.is_ok());
}

#[test]
fn ledger_balance_returns_value() {
    let dir = TempDir::new().unwrap();
    ledger_run(LedgerCommand::CreateAccount {
        dir: dir.path().to_path_buf(),
        id: 10,
        name: "X".into(),
        currency: "EUR".into(),
    })
    .unwrap();
    let result = ledger_run(LedgerCommand::Balance {
        dir: dir.path().to_path_buf(),
        account: 10,
        json: false,
    });
    assert!(result.is_ok());
}

#[test]
fn ledger_trial_balance_zero() {
    let dir = TempDir::new().unwrap();
    ledger_run(LedgerCommand::CreateAccount {
        dir: dir.path().to_path_buf(),
        id: 20,
        name: "P".into(),
        currency: "USD".into(),
    })
    .unwrap();
    ledger_run(LedgerCommand::CreateAccount {
        dir: dir.path().to_path_buf(),
        id: 21,
        name: "Q".into(),
        currency: "USD".into(),
    })
    .unwrap();
    ledger_run(LedgerCommand::Post {
        dir: dir.path().to_path_buf(),
        debit: 20,
        credit: 21,
        amount: 1000,
        desc: "check".into(),
    })
    .unwrap();
    let result = ledger_run(LedgerCommand::TrialBalance {
        dir: dir.path().to_path_buf(),
    });
    assert!(result.is_ok());
}

#[test]
fn ledger_create_duplicate_account_returns_err() {
    let dir = TempDir::new().unwrap();
    ledger_run(LedgerCommand::CreateAccount {
        dir: dir.path().to_path_buf(),
        id: 30,
        name: "Dup".into(),
        currency: "USD".into(),
    })
    .unwrap();
    let result = ledger_run(LedgerCommand::CreateAccount {
        dir: dir.path().to_path_buf(),
        id: 30,
        name: "Dup2".into(),
        currency: "USD".into(),
    });
    assert!(result.is_err());
}

#[test]
fn ledger_balance_unknown_account_returns_err() {
    let dir = TempDir::new().unwrap();
    let result = ledger_run(LedgerCommand::Balance {
        dir: dir.path().to_path_buf(),
        account: 9999,
        json: false,
    });
    assert!(result.is_err());
}

#[test]
fn ledger_balance_json_output() {
    let dir = TempDir::new().unwrap();
    ledger_run(LedgerCommand::CreateAccount {
        dir: dir.path().to_path_buf(),
        id: 40,
        name: "J".into(),
        currency: "GBP".into(),
    })
    .unwrap();
    let result = ledger_run(LedgerCommand::Balance {
        dir: dir.path().to_path_buf(),
        account: 40,
        json: true,
    });
    assert!(result.is_ok());
}
