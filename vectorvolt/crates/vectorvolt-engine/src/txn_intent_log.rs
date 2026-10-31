//! MVCC Phase 3 — durable transaction intent log for atomic cross-table commit.
//!
//! A single SQL transaction can touch several tables, each with its own
//! per-table [`crate::TxnId`] and its own [`crate::TableEngine::commit_txn`].
//! Without an outer record of "these N table-commits belong to one logical
//! unit", a crash (or an error) between committing table 1 and table 2 leaves
//! the database in a state where the caller's transaction was partially
//! applied — violating atomicity.
//!
//! This module implements a lightweight two-phase commit: before touching any
//! table, we durably record the *intent* (which tables / txn ids belong to
//! this logical transaction). Only after every table has been committed do we
//! durably record the outcome. If the process dies in between, the dangling
//! `Preparing` record is the breadcrumb [`recover_pending_transactions`] uses
//! to finish the job.
//!
//! ## Format
//!
//! Append-only, one JSON object per line (JSON-lines), matching the
//! `serde_json`-based serialization already used elsewhere in this crate
//! (e.g. `yaml_schema`/`schema` round-trip through `serde_json` internally).
//! A plain file (not the segment-based `vectorvolt-wal`) is used because the
//! only durability primitive we need is "append + fsync", and reusing the
//! frame/hash-chain machinery in `vectorvolt-wal` would add segment rotation,
//! checksum-chain and replay complexity this small log doesn't need.
//!
//! ## Idempotency finding (read before changing behavior)
//!
//! [`crate::TableEngine::commit_txn`] drains the per-table MVCC transaction
//! buffer (`MvccStore::drain_commit`) and *removes* the transaction from the
//! store as part of committing it. Calling `commit_txn` a second time with
//! the same [`crate::TxnId`] after a successful commit therefore returns
//! `EngineError::TxnNotFound` — it does **not** double-apply the mutations
//! (the buffer is gone, so there is nothing left to re-apply). That makes a
//! second `commit_txn` call *safe to retry* in the sense that it can never
//! corrupt data, but it also means recovery cannot distinguish "already
//! committed" from "never existed" other than by treating `TxnNotFound` as
//! "already applied, nothing to do".
//!
//! Because of this, recovery here **rolls forward**: for every table in a
//! dangling `Preparing` record it calls `commit_txn` again, treating
//! `TxnNotFound` as success (the table was already committed before the
//! crash/error) and any other error as a real failure that keeps the record
//! dangling for a later recovery attempt. Rolling back instead is not a safe
//! alternative here: once even one table's `commit_txn` has succeeded that
//! table's change is permanent (and, in a real process restart, any
//! not-yet-committed table's staged mutations are in-memory only and are
//! gone anyway — see the crate-level note below), so "undo everything" is
//! not achievable and rolling forward the remaining tables is the only
//! option that can restore atomicity across the tables that are still
//! recoverable.
//!
//! ### Write sets (crash recovery)
//!
//! The per-table staging buffers live in memory only, and per-table txn ids
//! restart at zero with the process, so after a real crash `commit_txn` on a
//! recorded id finds nothing (or the wrong thing). A PREPARE record therefore
//! also carries each table's full **write set** — the final put/delete for
//! every pk — and a `TableApplied` progress record is appended after each
//! table commits. Recovery re-applies the write set of every table not yet
//! marked applied via [`crate::TableEngine::apply_batch`], which is a single
//! atomic WAL record per table and idempotent, so a recovered transaction is
//! complete on every table rather than "closed" with some writes missing.
//!
//! Records written before write sets existed are still handled the old way
//! (roll forward by txn id, treating `TxnNotFound` as already applied).

use crate::error::EngineResult;
use crate::mvcc::TxnId;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// One table + its per-table transaction id, as recorded in an intent record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableTxn {
    pub table: String,
    pub txn_id: TxnId,
}

/// The lifecycle state of a logical multi-table transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentState {
    Preparing,
    Committed,
    RolledBack,
}

/// One table's staged changes in a cross-table transaction: the final put
/// (`Some(values)`) or delete (`None`) for each pk it touched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableWriteSet {
    pub table: String,
    pub ops: Vec<(u64, Option<Vec<u64>>)>,
}

/// A single durable record in the intent log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentRecord {
    pub txn_uuid: String,
    pub tables: Vec<TableTxn>,
    pub state: IntentState,
    /// Every table's write set, recorded at PREPARE. Empty in records written
    /// before write sets existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub write_set: Vec<TableWriteSet>,
    /// Tables whose write set is already durably applied.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applied: Vec<String>,
}

/// Outcome of resolving one dangling record during recovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryAction {
    pub txn_uuid: String,
    pub resolved_state: IntentState,
    /// Tables where `commit_txn` was actually (re-)invoked and returned Ok,
    /// as opposed to those found already-committed (`TxnNotFound`).
    pub tables_committed: Vec<String>,
    /// Tables that could not be resolved (a real error other than
    /// `TxnNotFound`) — the record is left dangling for a future attempt.
    pub tables_failed: Vec<String>,
}

/// Durable append-only JSON-lines log of multi-table transaction intents.
///
/// One [`TxnIntentLog`] is meant to be shared (e.g. behind an `Arc`) by a
/// single `SqlExecutor` / engine instance. All appends go through an
/// internal mutex so callers don't need external synchronization.
pub struct TxnIntentLog {
    path: PathBuf,
    file: Mutex<File>,
}

impl TxnIntentLog {
    /// Open (creating if necessary) the intent log at `path`, appending to
    /// any existing content.
    pub fn open(path: impl Into<PathBuf>) -> EngineResult<Self> {
        let path = path.into();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(TxnIntentLog {
            path,
            file: Mutex::new(file),
        })
    }

    fn append(&self, record: &IntentRecord) -> EngineResult<()> {
        let mut line = serde_json::to_string(record)
            .map_err(|e| crate::error::EngineError::Other(format!("intent log encode: {e}")))?;
        line.push('\n');
        let mut file = self.file.lock().unwrap();
        file.write_all(line.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }

    /// Phase A — durably record the intent to commit `tables` as one atomic
    /// unit, before any table's `commit_txn` is invoked.
    pub fn prepare(&self, txn_uuid: &str, tables: Vec<TableTxn>) -> EngineResult<()> {
        self.prepare_with_write_set(txn_uuid, tables, Vec::new())
    }

    /// Durably record a transaction's intent together with its write sets,
    /// before any table is touched.
    pub fn prepare_with_write_set(
        &self,
        txn_uuid: &str,
        tables: Vec<TableTxn>,
        write_set: Vec<TableWriteSet>,
    ) -> EngineResult<()> {
        self.append(&IntentRecord {
            txn_uuid: txn_uuid.to_owned(),
            tables,
            state: IntentState::Preparing,
            write_set,
            applied: Vec::new(),
        })
    }

    /// Record that the tables in `applied` have committed their write sets.
    ///
    /// Appended after each table so recovery re-applies only what had not
    /// landed yet, instead of overwriting later writes to the same rows.
    pub fn mark_tables_applied(
        &self,
        txn_uuid: &str,
        tables: Vec<TableTxn>,
        write_set: Vec<TableWriteSet>,
        applied: Vec<String>,
    ) -> EngineResult<()> {
        self.append(&IntentRecord {
            txn_uuid: txn_uuid.to_owned(),
            tables,
            state: IntentState::Preparing,
            write_set,
            applied,
        })
    }

    pub fn mark_committed(&self, txn_uuid: &str, tables: Vec<TableTxn>) -> EngineResult<()> {
        self.append(&IntentRecord {
            txn_uuid: txn_uuid.to_owned(),
            tables,
            state: IntentState::Committed,
            write_set: Vec::new(),
            applied: Vec::new(),
        })
    }

    pub fn mark_rolled_back(&self, txn_uuid: &str, tables: Vec<TableTxn>) -> EngineResult<()> {
        self.append(&IntentRecord {
            txn_uuid: txn_uuid.to_owned(),
            tables,
            state: IntentState::RolledBack,
            write_set: Vec::new(),
            applied: Vec::new(),
        })
    }

    pub fn read_all(&self) -> EngineResult<Vec<IntentRecord>> {
        read_all_records(&self.path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn read_all_records(path: &Path) -> EngineResult<Vec<IntentRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<IntentRecord>(&line) {
            Ok(rec) => out.push(rec),
            Err(_) => {
                // A truncated final line (partial write during a crash) is
                // tolerated and skipped rather than treated as fatal — the
                // truncated record, if it was a PREPARING record, simply
                // never got a matching COMMITTED and is handled by whichever
                // earlier well-formed record exists (there is none, so it
                // is silently dropped: nothing was ever committed for it).
                continue;
            }
        }
    }
    Ok(out)
}

/// Find dangling `Preparing` records (no later `Committed` / `RolledBack` for
/// the same `txn_uuid`) in the log at `log_path`.
pub fn find_dangling(records: &[IntentRecord]) -> Vec<IntentRecord> {
    let mut latest: std::collections::HashMap<&str, &IntentRecord> =
        std::collections::HashMap::new();
    for rec in records {
        latest.insert(rec.txn_uuid.as_str(), rec);
    }
    latest
        .into_values()
        .filter(|rec| rec.state == IntentState::Preparing)
        .cloned()
        .collect()
}

/// Recover every dangling `Preparing` transaction found in the intent log at
/// `log_path` against tables resolved via `resolve`, rolling each forward
/// (see module docs for why roll-forward is the safe choice) and writing a
/// final `Committed` / `RolledBack` record so recovery is itself idempotent.
///
/// `resolve` maps a table name to its [`crate::TableEngine`] (via whatever
/// registry the caller has open); tables that can't be resolved are treated
/// as a failure to commit and leave the record dangling.
pub fn recover_pending_transactions<F>(
    log_path: impl AsRef<Path>,
    resolve: F,
) -> EngineResult<Vec<RecoveryAction>>
where
    F: Fn(&str) -> Option<std::sync::Arc<crate::engine::TableEngine>>,
{
    let log = TxnIntentLog::open(log_path.as_ref())?;
    let records = log.read_all()?;
    let dangling = find_dangling(&records);

    let mut actions = Vec::new();
    for rec in dangling {
        let mut committed = Vec::new();
        let mut failed = Vec::new();
        if !rec.write_set.is_empty() {
            for ws in &rec.write_set {
                if rec.applied.contains(&ws.table) {
                    committed.push(ws.table.clone());
                    continue;
                }
                let Some(engine) = resolve(&ws.table) else {
                    failed.push(ws.table.clone());
                    continue;
                };
                match engine.apply_batch(ws.ops.clone()) {
                    Ok(_) => committed.push(ws.table.clone()),
                    Err(_) => failed.push(ws.table.clone()),
                }
            }
        }
        for t in rec.tables.iter().filter(|_| rec.write_set.is_empty()) {
            let Some(engine) = resolve(&t.table) else {
                failed.push(t.table.clone());
                continue;
            };
            match engine.commit_txn(t.txn_id) {
                Ok(()) => committed.push(t.table.clone()),
                Err(crate::error::EngineError::TxnNotFound(_)) => {
                    // Already committed before the crash/error — roll-forward no-op.
                    committed.push(t.table.clone());
                }
                Err(_) => failed.push(t.table.clone()),
            }
        }

        let resolved_state = if failed.is_empty() {
            IntentState::Committed
        } else {
            // Left dangling in spirit, but we still write a record so a
            // repeated recovery run doesn't reprocess the tables that did
            // succeed as "new" — the next attempt will see this record as
            // the latest for txn_uuid and, since it's still `Preparing`,
            // will retry exactly the tables that failed once more.
            IntentState::Preparing
        };

        if resolved_state == IntentState::Committed {
            log.mark_committed(&rec.txn_uuid, rec.tables.clone())?;
        } else if !rec.write_set.is_empty() {
            // Remember what did land so a later attempt only retries the rest.
            log.mark_tables_applied(
                &rec.txn_uuid,
                rec.tables.clone(),
                rec.write_set.clone(),
                committed.clone(),
            )?;
        }

        actions.push(RecoveryAction {
            txn_uuid: rec.txn_uuid,
            resolved_state,
            tables_committed: committed,
            tables_failed: failed,
        });
    }
    Ok(actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{EngineOptions, TableEngine};
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn write_set_is_reapplied_after_a_restart_lost_the_staging_buffer() {
        let dir = TempDir::new().unwrap();
        let a = Arc::new(
            TableEngine::open(
                vec!["pk".into(), "v".into()],
                dir.path().join("a"),
                1_000_000,
            )
            .unwrap(),
        );
        let b = Arc::new(
            TableEngine::open(
                vec!["pk".into(), "v".into()],
                dir.path().join("b"),
                1_000_000,
            )
            .unwrap(),
        );
        b.insert(9, vec![9, 9]).unwrap();
        let log = TxnIntentLog::open(log_path(&dir)).unwrap();
        let tables = vec![
            TableTxn {
                table: "a".into(),
                txn_id: 0,
            },
            TableTxn {
                table: "b".into(),
                txn_id: 0,
            },
        ];
        let ws = vec![
            TableWriteSet {
                table: "a".into(),
                ops: vec![(1, Some(vec![1, 10]))],
            },
            TableWriteSet {
                table: "b".into(),
                ops: vec![(9, None), (2, Some(vec![2, 20]))],
            },
        ];
        log.prepare_with_write_set("t", tables.clone(), ws.clone())
            .unwrap();
        // Table a committed before the crash; b did not.
        a.apply_batch(ws[0].ops.clone()).unwrap();
        log.mark_tables_applied("t", tables, ws, vec!["a".into()])
            .unwrap();
        // Something else wrote a's row after the transaction; recovery must not clobber it.
        a.insert(1, vec![1, 11]).unwrap();

        let actions = recover_pending_transactions(log_path(&dir), |name| match name {
            "a" => Some(Arc::clone(&a)),
            "b" => Some(Arc::clone(&b)),
            _ => None,
        })
        .unwrap();
        assert_eq!(actions[0].resolved_state, IntentState::Committed);
        assert_eq!(a.get(1).unwrap().unwrap().values, vec![1, 11]);
        assert!(b.get(9).unwrap().is_none());
        assert_eq!(b.get(2).unwrap().unwrap().values, vec![2, 20]);
        assert!(find_dangling(&log.read_all().unwrap()).is_empty());
    }

    fn log_path(dir: &TempDir) -> PathBuf {
        dir.path().join("txn_intent.log")
    }

    #[test]
    fn prepare_then_committed_round_trips() {
        let dir = TempDir::new().unwrap();
        let log = TxnIntentLog::open(log_path(&dir)).unwrap();
        let tables = vec![
            TableTxn {
                table: "orders".into(),
                txn_id: 1,
            },
            TableTxn {
                table: "order_items".into(),
                txn_id: 2,
            },
        ];
        log.prepare("txn-a", tables.clone()).unwrap();
        log.mark_committed("txn-a", tables).unwrap();

        let records = log.read_all().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].state, IntentState::Preparing);
        assert_eq!(records[1].state, IntentState::Committed);
        assert!(find_dangling(&records).is_empty());
    }

    #[test]
    fn preparing_with_no_committed_is_dangling() {
        let dir = TempDir::new().unwrap();
        let log = TxnIntentLog::open(log_path(&dir)).unwrap();
        let tables = vec![TableTxn {
            table: "orders".into(),
            txn_id: 1,
        }];
        log.prepare("txn-b", tables).unwrap();

        let records = log.read_all().unwrap();
        let dangling = find_dangling(&records);
        assert_eq!(dangling.len(), 1);
        assert_eq!(dangling[0].txn_uuid, "txn-b");
    }

    fn open_table(dir: &TempDir, name: &str) -> Arc<TableEngine> {
        let path = dir.path().join(name);
        Arc::new(
            TableEngine::open_with(
                vec!["id".into(), "v".into()],
                &path,
                EngineOptions::default(),
            )
            .unwrap(),
        )
    }

    #[test]
    fn recovery_rolls_forward_dangling_two_table_txn() {
        let dir = TempDir::new().unwrap();
        let orders = open_table(&dir, "orders");
        let items = open_table(&dir, "order_items");

        let t1 = orders.begin();
        orders.insert_txn(t1, 1, vec![1, 100]).unwrap();
        let t2 = items.begin();
        items.insert_txn(t2, 1, vec![1, 7]).unwrap();

        let tables = vec![
            TableTxn {
                table: "orders".into(),
                txn_id: t1,
            },
            TableTxn {
                table: "order_items".into(),
                txn_id: t2,
            },
        ];
        let log = TxnIntentLog::open(log_path(&dir)).unwrap();
        log.prepare("txn-c", tables).unwrap();
        // Simulate a crash: neither commit_txn call ran yet.
        drop(log);

        let orders2 = orders.clone();
        let items2 = items.clone();
        let actions = recover_pending_transactions(log_path(&dir), move |name| match name {
            "orders" => Some(orders2.clone()),
            "order_items" => Some(items2.clone()),
            _ => None,
        })
        .unwrap();

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].resolved_state, IntentState::Committed);
        assert_eq!(orders.get(1).unwrap().unwrap().values, vec![1, 100]);
        assert_eq!(items.get(1).unwrap().unwrap().values, vec![1, 7]);

        // Recovery must be idempotent — running it again resolves nothing new.
        let orders3 = orders.clone();
        let items3 = items.clone();
        let actions2 = recover_pending_transactions(log_path(&dir), move |name| match name {
            "orders" => Some(orders3.clone()),
            "order_items" => Some(items3.clone()),
            _ => None,
        })
        .unwrap();
        assert!(actions2.is_empty());
    }

    #[test]
    fn recovery_handles_partial_prior_commit_via_txn_not_found() {
        let dir = TempDir::new().unwrap();
        let orders = open_table(&dir, "orders");
        let items = open_table(&dir, "order_items");

        let t1 = orders.begin();
        orders.insert_txn(t1, 1, vec![1, 100]).unwrap();
        let t2 = items.begin();
        items.insert_txn(t2, 1, vec![1, 7]).unwrap();

        // Table 1 already committed before the "crash"; table 2 was not.
        orders.commit_txn(t1).unwrap();

        let tables = vec![
            TableTxn {
                table: "orders".into(),
                txn_id: t1,
            },
            TableTxn {
                table: "order_items".into(),
                txn_id: t2,
            },
        ];
        let log = TxnIntentLog::open(log_path(&dir)).unwrap();
        log.prepare("txn-d", tables).unwrap();

        let actions = recover_pending_transactions(log_path(&dir), move |name| match name {
            "orders" => Some(orders.clone()),
            "order_items" => Some(items.clone()),
            _ => None,
        })
        .unwrap();

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].resolved_state, IntentState::Committed);
        // orders was resolved via the TxnNotFound no-op path, items via a real commit.
        assert_eq!(actions[0].tables_committed.len(), 2);
    }
}
