//! Account store for double-entry bookkeeping.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::error::{LedgerError, LedgerResult};

/// A single account in the ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: u64,
    pub name: String,
    pub balance_cents: i64,
    pub currency: String,
}

/// Persistent account store backed by a JSON file.
pub struct AccountStore {
    path: PathBuf,
    accounts: Arc<RwLock<std::collections::HashMap<u64, Account>>>,
}

impl AccountStore {
    /// Open (or create) the account store at `dir`.
    pub fn open(dir: &Path) -> LedgerResult<Self> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join("accounts.json");
        let accounts = if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            serde_json::from_str(&raw)?
        } else {
            std::collections::HashMap::new()
        };
        Ok(Self {
            path,
            accounts: Arc::new(RwLock::new(accounts)),
        })
    }

    fn persist(&self) -> LedgerResult<()> {
        let raw = serde_json::to_string(&*self.accounts.read())?;
        std::fs::write(&self.path, raw)?;
        Ok(())
    }

    /// Create a new account with zero balance.
    pub fn create(&self, id: u64, name: &str, currency: &str) -> LedgerResult<()> {
        let mut map = self.accounts.write();
        if map.contains_key(&id) {
            return Err(LedgerError::AccountExists(id));
        }
        map.insert(
            id,
            Account {
                id,
                name: name.to_string(),
                balance_cents: 0,
                currency: currency.to_string(),
            },
        );
        drop(map);
        self.persist()
    }

    /// Fetch an account by id.
    pub fn get(&self, id: u64) -> LedgerResult<Option<Account>> {
        Ok(self.accounts.read().get(&id).cloned())
    }

    /// Return the current balance (in cents) for `id`.
    pub fn balance(&self, id: u64) -> LedgerResult<i64> {
        self.accounts
            .read()
            .get(&id)
            .map(|a| a.balance_cents)
            .ok_or(LedgerError::AccountNotFound(id))
    }

    /// List all accounts.
    pub fn list(&self) -> LedgerResult<Vec<Account>> {
        Ok(self.accounts.read().values().cloned().collect())
    }

    /// Apply a balance delta to `id`. Used internally by the ledger post.
    pub(crate) fn apply_delta(&self, id: u64, delta: i64) -> LedgerResult<()> {
        let mut map = self.accounts.write();
        let acct = map.get_mut(&id).ok_or(LedgerError::AccountNotFound(id))?;
        acct.balance_cents += delta;
        drop(map);
        self.persist()
    }
}
