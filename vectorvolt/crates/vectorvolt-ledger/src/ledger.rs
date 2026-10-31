//! Double-entry ledger built on top of `AccountStore`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

use crate::account::AccountStore;
use crate::error::{LedgerError, LedgerResult};
use crate::persist;

/// A single double-entry ledger record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: u64,
    pub debit_account: u64,
    pub credit_account: u64,
    pub amount_cents: i64,
    pub description: String,
    pub timestamp_ns: u64,
}

/// Double-entry ledger.
pub struct Ledger {
    pub accounts: AccountStore,
    entries_path: PathBuf,
    entries: Arc<RwLock<Vec<LedgerEntry>>>,
    next_id: AtomicU64,
    /// Serialises `persist`: writers share one `.tmp`, and the snapshot is
    /// taken under this lock so an older state never overwrites a newer one.
    persist_lock: Mutex<()>,
}

impl Ledger {
    /// Open (or create) the ledger at `dir`.
    pub fn open(dir: &Path) -> LedgerResult<Self> {
        std::fs::create_dir_all(dir)?;
        let entries_path = dir.join("entries.json");
        let entries: Vec<LedgerEntry> = match persist::read_committed(&entries_path)? {
            Some(raw) => serde_json::from_slice(&raw)?,
            None => Vec::new(),
        };
        let next_id = entries.iter().map(|e| e.id).max().unwrap_or(0) + 1;
        Ok(Self {
            accounts: AccountStore::open(dir)?,
            entries_path,
            entries: Arc::new(RwLock::new(entries)),
            next_id: AtomicU64::new(next_id),
            persist_lock: Mutex::new(()),
        })
    }

    fn persist_entries(&self) -> LedgerResult<()> {
        let _guard = self.persist_lock.lock();
        let raw = serde_json::to_vec(&*self.entries.read())?;
        persist::write_atomic(&self.entries_path, &raw)?;
        Ok(())
    }

    fn now_ns() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    /// Post a debit/credit pair.  The debit account is debited (balance
    /// decreases) and the credit account is credited (balance increases).
    /// Returns the new entry id.
    pub fn post(
        &self,
        debit: u64,
        credit: u64,
        amount_cents: i64,
        desc: &str,
    ) -> LedgerResult<u64> {
        // Validate both accounts exist.
        if self.accounts.get(debit)?.is_none() {
            return Err(LedgerError::AccountNotFound(debit));
        }
        if self.accounts.get(credit)?.is_none() {
            return Err(LedgerError::AccountNotFound(credit));
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let entry = LedgerEntry {
            id,
            debit_account: debit,
            credit_account: credit,
            amount_cents,
            description: desc.to_string(),
            timestamp_ns: Self::now_ns(),
        };
        self.entries.write().push(entry);
        self.persist_entries()?;
        // Apply balances: debit account decreases, credit account increases.
        self.accounts.apply_delta(debit, -amount_cents)?;
        self.accounts.apply_delta(credit, amount_cents)?;
        Ok(id)
    }

    /// Return the current balance of `account_id`.
    pub fn balance(&self, account_id: u64) -> LedgerResult<i64> {
        self.accounts.balance(account_id)
    }

    /// Return all entries that involve `account_id` (as debit or credit).
    pub fn entries_for(&self, account_id: u64) -> LedgerResult<Vec<LedgerEntry>> {
        Ok(self
            .entries
            .read()
            .iter()
            .filter(|e| e.debit_account == account_id || e.credit_account == account_id)
            .cloned()
            .collect())
    }

    /// Sum of all balances.  In a correct double-entry system this is always 0.
    pub fn trial_balance(&self) -> LedgerResult<i64> {
        let accounts = self.accounts.list()?;
        Ok(accounts.iter().map(|a| a.balance_cents).sum())
    }
}
