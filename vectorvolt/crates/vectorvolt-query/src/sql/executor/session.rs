//! Per-session guards: idle timeout, role and privilege checks, memory limit,
//! result masking and the cooperative statement deadline.

use super::*;

impl SqlExecutor {
    /// D5/S283 — record "now" as this session's last activity.
    pub(super) fn touch_activity(&self) {
        let now = self.created_at.elapsed().as_nanos() as u64;
        self.s283_last_activity_ns
            .store(now, std::sync::atomic::Ordering::Relaxed);
    }

    /// D5/S283 — enforce `SET CONNECTION IDLE TIMEOUT`. A session whose
    /// previous statement ended (or which was opened) longer ago than the
    /// timeout gets SQLSTATE 57P05 for this statement and any open
    /// transaction is rolled back. The executor cannot close the socket, so
    /// the session is then treated as freshly started (its next statement
    /// runs normally); listeners that want to drop the connection can
    /// match on 57P05. A timeout of 0 disables the check.
    pub(super) fn check_idle_timeout(&self) -> Option<QueryResult> {
        let timeout_ms = (*self.s283_idle_timeout_ms.lock_safe())?;
        if timeout_ms == 0 {
            return None;
        }
        let now = self.created_at.elapsed().as_nanos() as u64;
        let last = self
            .s283_last_activity_ns
            .swap(now, std::sync::atomic::Ordering::Relaxed);
        if now.saturating_sub(last) <= timeout_ms.saturating_mul(1_000_000) {
            return None;
        }
        let had_txn = self.txn.lock_safe().is_some();
        if had_txn {
            let _ = self.exec_rollback();
        }
        Some(sql_error(
            "57P05",
            &format!(
                "terminating session due to idle timeout ({timeout_ms} ms){}",
                if had_txn {
                    "; open transaction rolled back"
                } else {
                    ""
                }
            ),
        ))
    }

    /// D5/S252 — set this session's role (as `SET ROLE`). `None` clears it.
    /// Ignored while the role is pinned by
    /// [`set_authenticated_role`](Self::set_authenticated_role).
    pub fn set_session_role(&self, role: Option<&str>) {
        if self
            .s252_role_pinned
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }
        *self.s252_session_role.lock_safe() = role.map(|r| r.trim().to_ascii_lowercase());
    }

    /// D5/S252 — fix this session's role to the authenticated user (for
    /// listeners that authenticate connections); `SET ROLE` to a different
    /// role is then refused.
    pub fn set_authenticated_role(&self, role: &str) {
        *self.s252_session_role.lock_safe() = Some(role.trim().to_ascii_lowercase());
        self.s252_role_pinned
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// D5/S252 — the current session role, if any.
    pub fn session_role(&self) -> Option<String> {
        self.s252_session_role.lock_safe().clone()
    }

    /// D5/S252 — table privilege check. Tables that never had a GRANT are
    /// open (backwards compatible); for governed tables the session role
    /// must hold `privilege` (or ALL) on the table.
    pub(super) fn check_privilege(&self, table: &str, privilege: &str) -> Option<QueryResult> {
        let table = table.to_ascii_lowercase();
        if !self.s252_governed_tables.lock_safe().contains(&table) {
            return None;
        }
        let Some(role) = self.s252_session_role.lock_safe().clone() else {
            return Some(sql_error(
                "42501",
                &format!("permission denied for table {table}: no session role set (SET ROLE)"),
            ));
        };
        let granted = self
            .s252_role_grants
            .lock_safe()
            .iter()
            .any(|(r, t, p)| r == &role && t == &table && (p == privilege || p == "ALL"));
        if granted {
            None
        } else {
            Some(sql_error(
                "42501",
                &format!("permission denied for table {table}: role {role} lacks {privilege}"),
            ))
        }
    }

    /// D5/S273/S278 — DDL guard. DDL is refused when the table's schema
    /// (the `schema.` prefix, else the current search_path) or the table
    /// itself is named in `LOCK SCHEMA`. `destructive` DDL (ALTER, DROP,
    /// RENAME) is also refused on a frozen table.
    pub(super) fn check_ddl_allowed(&self, table: &str, destructive: bool) -> Option<QueryResult> {
        let table = table.to_ascii_lowercase();
        let schema = match table.split_once('.') {
            Some((s, _)) => s.to_owned(),
            None => self
                .search_path
                .lock_safe()
                .split(',')
                .next()
                .unwrap_or("public")
                .trim()
                .to_ascii_lowercase(),
        };
        {
            let locks = self.s278_schema_locks.lock_safe();
            if locks.contains(&schema) || locks.contains(&table) {
                return Some(sql_error(
                    "55000",
                    &format!(
                        "schema {schema} is locked: DDL on table {table} is not allowed (UNLOCK SCHEMA first)"
                    ),
                ));
            }
        }
        if destructive && self.s273_frozen.lock_safe().contains(&table) {
            return Some(sql_error(
                "55000",
                &format!("table {table} is frozen: DDL is not allowed (UNFREEZE TABLE first)"),
            ));
        }
        None
    }

    /// D5 — guards evaluated before a parsed statement runs: S278 schema
    /// lock and S273 freeze for table DDL.
    pub(super) fn pre_dispatch_guard(&self, stmt: &Stmt) -> Option<QueryResult> {
        match stmt {
            Stmt::CreateTable(s) => self.check_ddl_allowed(&s.table, false),
            Stmt::DropTable(s) => self.check_ddl_allowed(&s.table, true),
            Stmt::AlterTable(s) => self.check_ddl_allowed(&s.table, true),
            Stmt::RenameTable(s) => self
                .check_ddl_allowed(&s.old_name, true)
                .or_else(|| self.check_ddl_allowed(&s.new_name, false)),
            Stmt::CreateIndex(s) => self.check_ddl_allowed(&s.table, false),
            Stmt::CreateTrigger(def) => self.check_ddl_allowed(&def.table, false),
            Stmt::CreatePolicy { table, .. } => self.check_ddl_allowed(table, false),
            Stmt::CreatePartition { parent, .. } => self.check_ddl_allowed(parent, false),
            _ => None,
        }
    }

    /// D5/S295 — fail with a memory-limit error when `bytes` (an estimate of
    /// the memory a query has materialised) exceeds the configured limit.
    pub(super) fn check_memory(&self, bytes: u64) -> Result<(), ExecError> {
        let limit = self
            .s295_memory_limit_bytes
            .load(std::sync::atomic::Ordering::Relaxed);
        if limit != 0 && bytes > limit {
            return Err(ExecError::Engine(format!(
                "{MEMORY_LIMIT_MSG}: query needs ~{bytes} bytes, limit is {limit} bytes"
            )));
        }
        Ok(())
    }

    /// D5/S295 — whether a memory limit is configured.
    pub(super) fn memory_limited(&self) -> bool {
        self.s295_memory_limit_bytes
            .load(std::sync::atomic::Ordering::Relaxed)
            != 0
    }

    /// D5/S159/S79/S281 — every active column mask as
    /// (table, column) -> replacement value, keys lower-cased.
    pub(super) fn active_masks(&self) -> HashMap<(String, String), String> {
        let mut out: HashMap<(String, String), String> = HashMap::new();
        for ((t, c), v) in self.s281_result_masks.lock_safe().iter() {
            out.insert((t.to_ascii_lowercase(), c.to_ascii_lowercase()), v.clone());
        }
        for ((t, c), v) in self.masks.lock_safe().iter() {
            out.insert((t.to_ascii_lowercase(), c.to_ascii_lowercase()), v.clone());
        }
        let policies = self.mask_policies.lock_safe();
        for (key, policy) in self.applied_masks.lock_safe().iter() {
            if let (Some((t, c)), Some((_, expr))) = (key.split_once('.'), policies.get(policy)) {
                out.insert(
                    (t.to_ascii_lowercase(), c.to_ascii_lowercase()),
                    expr.clone(),
                );
            }
        }
        out
    }

    /// D5 — the mask for a result column header, if it exposes a masked
    /// column of a table in `scope`: `col`, `table.col`, or a value-returning
    /// function header such as `min(col)` / `first_value(col)` (COUNT and
    /// ranking functions do not expose the value).
    pub(super) fn mask_for_header<'m>(
        header: &str,
        scope: &[String],
        masks: &'m HashMap<(String, String), String>,
    ) -> Option<&'m String> {
        let h = header.trim().to_ascii_lowercase();
        if let Some(open) = h.find('(')
            && h.ends_with(')')
        {
            let func = h[..open].trim();
            if matches!(
                func,
                "count" | "row_number" | "rank" | "dense_rank" | "ntile" | "percent_rank"
            ) {
                return None;
            }
            let arg = h[open + 1..h.len() - 1]
                .split(',')
                .next()
                .unwrap_or("")
                .trim();
            if arg.is_empty() || arg.contains('(') {
                return None;
            }
            return Self::mask_for_header(arg, scope, masks);
        }
        if let Some((t, c)) = h.split_once('.')
            && let Some(m) = masks.get(&(t.to_owned(), c.to_owned()))
        {
            return Some(m);
        }
        scope
            .iter()
            .find_map(|t| masks.get(&(t.to_ascii_lowercase(), h.clone())))
    }

    pub(super) fn apply_masks(
        result: QueryResult,
        scope: &[String],
        masks: &HashMap<(String, String), String>,
        derived: &HashMap<String, String>,
    ) -> QueryResult {
        match result {
            QueryResult::Rows { columns, rows } => {
                let per_col: Vec<Option<String>> = columns
                    .iter()
                    .map(|h| {
                        derived
                            .get(&h.to_ascii_lowercase())
                            .or_else(|| Self::mask_for_header(h, scope, masks))
                            .cloned()
                    })
                    .collect();
                if per_col.iter().all(Option::is_none) {
                    return QueryResult::Rows { columns, rows };
                }
                let rows = rows
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .enumerate()
                            .map(|(i, v)| match per_col.get(i) {
                                Some(Some(m)) => m.clone(),
                                _ => v,
                            })
                            .collect()
                    })
                    .collect();
                QueryResult::Rows { columns, rows }
            }
            QueryResult::WithNotice { result, notice } => QueryResult::WithNotice {
                result: Box::new(Self::apply_masks(*result, scope, masks, derived)),
                notice,
            },
            other => other,
        }
    }

    /// B8 — start the statement deadline if a timeout is set and none is
    /// running. Returns whether this call owns (and must clear) it.
    pub(super) fn arm_deadline(&self) -> bool {
        use std::sync::atomic::Ordering::Relaxed;
        let Some(ms) = *self.statement_timeout_ms.lock_safe() else {
            return false;
        };
        if self.stmt_deadline_ns.load(Relaxed) != 0 {
            return false;
        }
        let now = self.created_at.elapsed().as_nanos() as u64;
        let deadline = now.saturating_add(ms.saturating_mul(1_000_000)).max(1);
        self.stmt_timed_out.store(false, Relaxed);
        self.stmt_deadline_ns.store(deadline, Relaxed);
        true
    }

    /// B8 — cooperative cancellation point for row loops (scan, filter,
    /// join, aggregate, update, delete): fails once the deadline passed.
    pub(super) fn check_deadline(&self) -> Result<(), ExecError> {
        use std::sync::atomic::Ordering::Relaxed;
        let deadline = self.stmt_deadline_ns.load(Relaxed);
        if deadline != 0 && self.created_at.elapsed().as_nanos() as u64 >= deadline {
            self.stmt_timed_out.store(true, Relaxed);
            return Err(ExecError::Engine(STATEMENT_TIMEOUT_MSG.to_owned()));
        }
        Ok(())
    }
}
