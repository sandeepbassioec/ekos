//! Transactions (BEGIN / COMMIT / ROLLBACK / SAVEPOINT), MVCC read timestamps
//! and table-level access checks (grants, RLS).

use super::*;

impl SqlExecutor {
    /// B11 (PP-25) — an autocommit write applies immediately, so it takes
    /// the same lock transactional commits hold from their SERIALIZABLE
    /// conflict check through their last table commit; otherwise it could
    /// land between the two. Writes inside a transaction only stage rows and
    /// take nothing; reads never take it.
    pub(super) fn autocommit_write_lock(&self) -> Option<SqlCommitGuard> {
        if self.txn.lock_safe().is_some() {
            return None;
        }
        Some(sql_commit_lock())
    }

    pub(super) fn exec_begin(&self, level: IsolationLevel) -> Result<QueryResult, ExecError> {
        let mut guard = self.txn.lock_safe();
        if guard.is_some() {
            return Err(ExecError::TxnAlreadyActive);
        }
        *guard = Some(HashMap::new());
        drop(guard);
        self.savepoints.lock_safe().clear();
        *self.mvcc_txn.lock_safe() = Some(MvccTxnState::new(level));
        Ok(QueryResult::TxBegin)
    }

    pub(super) fn exec_commit(&self) -> Result<QueryResult, ExecError> {
        let mut guard = self.txn.lock_safe();
        let txn_map = guard.take().ok_or(ExecError::NoActiveTxn)?;
        drop(guard);
        let mvcc_state = self.mvcc_txn.lock_safe().take();
        self.savepoints.lock_safe().clear();

        // Held from the conflict check through the last table commit, so no
        // other commit can land between "nothing conflicts" and "our writes
        // are applied" (PP-25). Autocommit writes take it too (B11, see
        // `autocommit_write_lock`).
        let _commit_guard = sql_commit_lock();

        // MVCC Phase 2 — SERIALIZABLE write-write conflict detection. For
        // every pk this transaction wrote, compare the value visible at its
        // fixed read_ts against the value committed right now (before we
        // apply our own buffered writes): if they differ, some other
        // transaction committed a change to a row we also touched after we
        // took our snapshot, so this commit must fail.
        if let Some(state) = &mvcc_state
            && matches!(state.isolation, IsolationLevel::Serializable)
        {
            let mut conflict = false;
            'outer: for (table, pks) in &state.written {
                let Some(&read_ts) = state.read_ts.get(table) else {
                    continue;
                };
                let Ok(engine) = self.registry.get(table) else {
                    continue;
                };
                for &pk in pks {
                    let snapshot_val = engine.get_at(pk, read_ts).map(|r| r.values);
                    let latest_val = engine.get(pk).ok().flatten().map(|r| r.values);
                    if snapshot_val != latest_val {
                        conflict = true;
                        break 'outer;
                    }
                }
            }
            if conflict {
                for (table, txn_id) in txn_map {
                    if let Ok(engine) = self.registry.get(&table) {
                        let _ = engine.rollback_txn(txn_id);
                    }
                }
                return Ok(sql_error(
                    "40001",
                    "could not serialize access due to concurrent update",
                ));
            }
        }

        // MVCC Phase 3 — atomic cross-table commit via a durable intent log.
        //
        // Phase A (PREPARE): record which tables/txn ids belong to this
        // logical commit *before* touching any table, and fsync it. If the
        // process crashes after this point but before Phase B completes, the
        // dangling PREPARING record is what `recover_pending_transactions`
        // uses to finish the job (see `vectorvolt_engine::txn_intent_log`).
        let tables: Vec<TableTxn> = txn_map
            .iter()
            .map(|(table, &txn_id)| TableTxn {
                table: table.clone(),
                txn_id,
            })
            .collect();

        // A single-table transaction still goes through the same intent-log
        // path (PREPARE + per-table commit + mark-committed) — the log write
        // is cheap and this keeps the code path uniform and always tested,
        // with no behavioral change for the common case.
        if !tables.is_empty() {
            let n = self
                .txn_uuid_counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let txn_uuid = format!("{}-{}-{n}", std::process::id(), Self::now_ns());

            // The write sets go into PREPARE so recovery does not depend on
            // the in-memory staging buffers surviving (they do not survive a
            // crash) or on per-table txn ids (they restart at zero).
            let mut write_set = Vec::with_capacity(txn_map.len());
            for (table, txn_id) in &txn_map {
                let engine = self
                    .registry
                    .get(table)
                    .map_err(|_| ExecError::TableNotFound(table.clone()))?;
                // A txn id the engine no longer knows was already applied (or
                // never staged anything): there is nothing left to record.
                let ops = match engine.txn_write_set(*txn_id) {
                    Ok(ops) => ops,
                    Err(vectorvolt_engine::EngineError::TxnNotFound(_)) => Vec::new(),
                    Err(e) => return Err(ExecError::Engine(e.to_string())),
                };
                write_set.push(TableWriteSet {
                    table: table.clone(),
                    ops,
                });
            }
            self.txn_intent_log
                .prepare_with_write_set(&txn_uuid, tables.clone(), write_set.clone())
                .map_err(|e| ExecError::Engine(e.to_string()))?;

            // Phase B (COMMIT ALL): apply every table's buffered mutations.
            // Each table commit is one atomic WAL record; progress is logged
            // after each so recovery re-applies only the tables that had not
            // landed.
            let mut applied = Vec::with_capacity(txn_map.len());
            for (table, txn_id) in &txn_map {
                let engine = self
                    .registry
                    .get(table)
                    .map_err(|_| ExecError::TableNotFound(table.clone()))?;
                if let Err(e) = engine.commit_txn(*txn_id) {
                    // Do NOT write a COMMITTED record — the dangling
                    // PREPARING record is the source of truth for recovery,
                    // which re-applies this table's write set from the log.
                    return Ok(sql_error(
                        "40003",
                        &format!(
                            "cross-table commit failed on table '{table}': {e}; transaction intent {txn_uuid} left pending for recovery"
                        ),
                    ));
                }
                applied.push(table.clone());
                if txn_map.len() > 1 {
                    self.txn_intent_log
                        .mark_tables_applied(
                            &txn_uuid,
                            tables.clone(),
                            write_set.clone(),
                            applied.clone(),
                        )
                        .map_err(|e| ExecError::Engine(e.to_string()))?;
                }
            }

            self.txn_intent_log
                .mark_committed(&txn_uuid, tables)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
        }
        Ok(QueryResult::TxCommit)
    }

    pub(super) fn now_ns() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    }

    pub(super) fn exec_rollback(&self) -> Result<QueryResult, ExecError> {
        let mut guard = self.txn.lock_safe();
        let txn_map = guard.take().ok_or(ExecError::NoActiveTxn)?;
        drop(guard);
        *self.mvcc_txn.lock_safe() = None;
        self.savepoints.lock_safe().clear();
        for (table, txn_id) in txn_map {
            if let Ok(engine) = self.registry.get(&table) {
                let _ = engine.rollback_txn(txn_id);
            }
        }
        Ok(QueryResult::TxRollback)
    }

    /// Return the active txn_id for `table`, creating one via `engine.begin()` if needed.
    /// Returns `None` when not in a transaction (auto-commit mode).
    pub(super) fn txn_id_for(&self, table: &str) -> Option<TxnId> {
        // Fix this table's read snapshot (if REPEATABLE READ / SERIALIZABLE)
        // at first touch, whether that touch is a read or a write.
        self.ensure_read_ts(table);

        let mut guard = self.txn.lock_safe();
        let map = guard.as_mut()?;
        if let Some(&id) = map.get(table) {
            return Some(id);
        }
        // Lazy begin: open a txn for this table on first DML touch.
        let engine = self.registry.get(table).ok()?;
        let id = engine.begin();
        map.insert(table.to_owned(), id);
        Some(id)
    }

    /// Fix `table`'s read_ts once (lazily, on first touch) when the active
    /// transaction is REPEATABLE READ / SERIALIZABLE. No-op in READ COMMITTED
    /// / autocommit, or if this table's read_ts is already fixed.
    pub(super) fn ensure_read_ts(&self, table: &str) {
        let mut guard = self.mvcc_txn.lock_safe();
        let Some(state) = guard.as_mut() else {
            return;
        };
        if !state.snapshot_active() || state.read_ts.contains_key(table) {
            return;
        }
        drop(guard);
        let Ok(engine) = self.registry.get(table) else {
            return;
        };
        let ts = engine.begin_snapshot();
        if let Some(state) = self.mvcc_txn.lock_safe().as_mut() {
            state.read_ts.entry(table.to_owned()).or_insert(ts);
        }
    }

    /// The fixed read_ts for `table` under the active transaction, or `None`
    /// when reads should see the latest committed value (READ COMMITTED,
    /// autocommit, or no active transaction).
    pub(super) fn read_ts_for(&self, table: &str) -> Option<u64> {
        self.ensure_read_ts(table);
        self.mvcc_txn
            .lock_safe()
            .as_ref()?
            .read_ts
            .get(table)
            .copied()
    }

    /// Record that this transaction wrote `pk` in `table` (SERIALIZABLE only —
    /// used at commit time for write-write conflict detection).
    pub(super) fn record_write(&self, table: &str, pk: u64) {
        let mut guard = self.mvcc_txn.lock_safe();
        if let Some(state) = guard.as_mut()
            && matches!(state.isolation, IsolationLevel::Serializable)
        {
            state
                .written
                .entry(table.to_owned())
                .or_default()
                .insert(pk);
        }
    }

    /// S81/S82 — per-table token auth and rate limit. Applied to every read
    /// and write path (SELECT, INSERT/UPSERT, INSERT ... SELECT, UPDATE,
    /// DELETE, TRUNCATE, COPY FROM). Returns the error result to send back
    /// when access is denied.
    ///
    /// D5 — also enforces S252 grants (`privilege` is SELECT, INSERT,
    /// UPDATE, DELETE or TRUNCATE) and S273 freeze (any non-SELECT access to
    /// a frozen table is refused).
    pub(super) fn check_table_access(&self, table: &str, privilege: &str) -> Option<QueryResult> {
        if let Some(denied) = self.check_privilege(table, privilege) {
            return Some(denied);
        }
        if privilege != "SELECT"
            && self
                .s273_frozen
                .lock_safe()
                .contains(&table.to_ascii_lowercase())
        {
            return Some(sql_error(
                "55000",
                &format!("table {table} is frozen: writes are not allowed (UNFREEZE TABLE first)"),
            ));
        }
        // S81 — auth check
        if let Some(required) = self.table_tokens.lock_safe().get(table).cloned() {
            let sess = self.session_token.lock_safe().clone();
            if sess.as_deref() != Some(&required) {
                return Some(sql_error("28000", "auth required"));
            }
        }

        // S82 — rate limit check
        let mut limits = self.rate_limits.lock_safe();
        if let Some(entry) = limits.get_mut(table) {
            let elapsed = entry.2.elapsed().as_secs_f64();
            if elapsed > 1.0 {
                entry.1 = 0;
                entry.2 = std::time::Instant::now();
            }
            entry.1 += 1;
            if entry.1 > entry.0 {
                return Some(sql_error("HY000", "rate limit exceeded"));
            }
        }
        None
    }

    /// S78 — apply every RLS policy defined on `table` (policies are
    /// restrictive: a row must satisfy all of them). Used by SELECT (including
    /// the right side of joins), UPDATE and DELETE so a statement can never
    /// read or modify a row the policies hide.
    pub(super) fn apply_rls(
        &self,
        rows: Vec<EngineRow>,
        schema: &TableSchema,
        table: &str,
    ) -> Result<Vec<EngineRow>, ExecError> {
        let pol_exprs: Vec<Expr> = self
            .policies
            .lock_safe()
            .values()
            .filter(|(tbl, _)| tbl == table)
            .map(|(_, expr)| expr.clone())
            .collect();
        let mut rows = rows;
        for pol_expr in pol_exprs {
            rows = self.filter_rows(rows, Some(&pol_expr), schema, table)?;
        }
        Ok(rows)
    }

    /// S78 — validate a CREATE POLICY predicate. Only self-contained row
    /// predicates over existing columns of `table` are accepted; anything the
    /// row filter cannot evaluate per row (subqueries, placeholders, full-text
    /// MATCH, unknown columns) is rejected so a stored policy always applies.
    pub(super) fn validate_policy_expr(
        expr: &Expr,
        schema: &TableSchema,
        table: &str,
    ) -> Result<(), String> {
        fn col_ok(c: &str, schema: &TableSchema) -> Result<(), String> {
            schema
                .find(c)
                .map(|_| ())
                .ok_or_else(|| format!("unknown column '{c}' in policy expression"))
        }
        fn value(e: &Expr, schema: &TableSchema, table: &str) -> Result<(), String> {
            match e {
                Expr::Col(c) => col_ok(c, schema),
                Expr::QualifiedCol { table: t, col } => {
                    if t != table {
                        return Err(format!("policy expression references other table '{t}'"));
                    }
                    col_ok(col, schema)
                }
                Expr::Lit(_) => Ok(()),
                Expr::Arith { left, right, .. } => {
                    value(left, schema, table)?;
                    value(right, schema, table)
                }
                Expr::ScalarFn { arg, .. } => value(arg, schema, table),
                Expr::Cast { expr, .. } => value(expr, schema, table),
                Expr::Coalesce { args } | Expr::StrFunc { args, .. } => {
                    args.iter().try_for_each(|a| value(a, schema, table))
                }
                Expr::Case { branches, else_val } => {
                    for (c, v) in branches {
                        pred(c, schema, table)?;
                        value(v, schema, table)?;
                    }
                    else_val
                        .as_deref()
                        .map_or(Ok(()), |v| value(v, schema, table))
                }
                Expr::DateExtract { expr, .. } => value(expr, schema, table),
                Expr::JsonGet { expr, .. } | Expr::JsonGetText { expr, .. } => {
                    value(expr, schema, table)
                }
                other => Err(format!("unsupported value in policy expression: {other:?}")),
            }
        }
        fn pred(e: &Expr, schema: &TableSchema, table: &str) -> Result<(), String> {
            match e {
                Expr::BinOp {
                    op: BinOp::And | BinOp::Or,
                    left,
                    right,
                } => {
                    pred(left, schema, table)?;
                    pred(right, schema, table)
                }
                Expr::BinOp { left, right, .. } => {
                    value(left, schema, table)?;
                    value(right, schema, table)
                }
                Expr::In { col, .. }
                | Expr::Between { col, .. }
                | Expr::IsNull { col, .. }
                | Expr::Like { col, .. } => col_ok(col, schema),
                other => Err(format!(
                    "unsupported policy expression (must be a row predicate): {other:?}"
                )),
            }
        }
        pred(expr, schema, table)
    }

    // S53 â€” Savepoints
    pub(super) fn exec_savepoint(&self, name: String) -> Result<QueryResult, ExecError> {
        // B9 — snapshot every touched table's staged write set, so ROLLBACK
        // TO can restore it exactly (including later changes to tables the
        // transaction had already touched before the savepoint).
        let current = self.txn.lock_safe().clone().unwrap_or_default();
        let mut snapshot = SavepointSnap::new();
        for (table, txn_id) in current {
            let engine = self
                .registry
                .get(&table)
                .map_err(|_| ExecError::TableNotFound(table.clone()))?;
            let ops = engine
                .txn_write_set(txn_id)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
            snapshot.insert(table, ops);
        }
        self.savepoints.lock_safe().insert(name, snapshot);
        Ok(QueryResult::TxBegin)
    }

    pub(super) fn exec_release_savepoint(&self, name: String) -> Result<QueryResult, ExecError> {
        self.savepoints.lock_safe().remove(&name);
        Ok(QueryResult::TxCommit)
    }

    pub(super) fn exec_rollback_to(&self, name: String) -> Result<QueryResult, ExecError> {
        let Some(current) = self.txn.lock_safe().clone() else {
            return Ok(sql_error(
                "25P01",
                "ROLLBACK TO SAVEPOINT can only be used in transaction blocks",
            ));
        };
        let Some(sp) = self.savepoints.lock_safe().get(&name).cloned() else {
            return Ok(sql_error(
                "3B001",
                &format!("savepoint \"{name}\" does not exist"),
            ));
        };
        // B9 — discard every table's staged writes and replay the write set
        // recorded at SAVEPOINT time into a fresh per-table txn. Tables first
        // touched after the savepoint simply drop out of the transaction.
        let mut restored = TxnMap::new();
        for (table, txn_id) in &current {
            let Ok(engine) = self.registry.get(table) else {
                continue;
            };
            let _ = engine.rollback_txn(*txn_id);
            if let Some(ops) = sp.get(table) {
                let id = engine.begin();
                for (pk, op) in ops {
                    match op {
                        Some(values) => engine.insert_txn(id, *pk, values.clone()),
                        None => engine.delete_txn(id, *pk),
                    }
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                }
                restored.insert(table.clone(), id);
            }
        }
        *self.txn.lock_safe() = Some(restored);
        Ok(QueryResult::TxRollback)
    }
}
