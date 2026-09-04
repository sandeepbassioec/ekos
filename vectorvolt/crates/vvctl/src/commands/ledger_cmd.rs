//! `vvctl ledger` — double-entry ledger commands.

use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;
use vectorvolt_ledger::Ledger;

#[derive(Subcommand)]
pub enum LedgerCommand {
    /// Create a new account.
    CreateAccount {
        /// Ledger data directory.
        #[arg(long)]
        dir: PathBuf,
        /// Account ID.
        #[arg(long)]
        id: u64,
        /// Account name.
        #[arg(long)]
        name: String,
        /// Currency code (e.g. USD).
        #[arg(long)]
        currency: String,
    },

    /// Post a double-entry transaction.
    Post {
        /// Ledger data directory.
        #[arg(long)]
        dir: PathBuf,
        /// Debit account ID.
        #[arg(long)]
        debit: u64,
        /// Credit account ID.
        #[arg(long)]
        credit: u64,
        /// Amount in cents.
        #[arg(long)]
        amount: i64,
        /// Transaction description.
        #[arg(long)]
        desc: String,
    },

    /// Show the balance of an account.
    Balance {
        /// Ledger data directory.
        #[arg(long)]
        dir: PathBuf,
        /// Account ID.
        #[arg(long)]
        account: u64,
        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Show the trial balance (sum of all accounts — should be 0).
    TrialBalance {
        /// Ledger data directory.
        #[arg(long)]
        dir: PathBuf,
    },
}

pub fn run(cmd: LedgerCommand) -> Result<()> {
    match cmd {
        LedgerCommand::CreateAccount {
            dir,
            id,
            name,
            currency,
        } => {
            let ledger = Ledger::open(&dir)?;
            ledger.accounts.create(id, &name, &currency)?;
            println!("account {id} created");
        }

        LedgerCommand::Post {
            dir,
            debit,
            credit,
            amount,
            desc,
        } => {
            let ledger = Ledger::open(&dir)?;
            let entry_id = ledger.post(debit, credit, amount, &desc)?;
            println!("posted entry {entry_id}");
        }

        LedgerCommand::Balance { dir, account, json } => {
            let ledger = Ledger::open(&dir)?;
            let balance = ledger.balance(account)?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({ "account": account, "balance_cents": balance })
                );
            } else {
                println!("account {account} balance: {balance} cents");
            }
        }

        LedgerCommand::TrialBalance { dir } => {
            let ledger = Ledger::open(&dir)?;
            let total = ledger.trial_balance()?;
            println!("trial balance: {total}");
        }
    }
    Ok(())
}
