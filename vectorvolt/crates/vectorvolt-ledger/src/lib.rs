//! `vectorvolt-ledger` — double-entry financial ledger reference domain.

pub mod account;
pub mod error;
pub mod ledger;
mod persist;

pub use account::{Account, AccountStore};
pub use error::{LedgerError, LedgerResult};
pub use ledger::{Ledger, LedgerEntry};
