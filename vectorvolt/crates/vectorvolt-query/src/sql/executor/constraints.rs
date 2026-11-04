//! Row constraints (CHECK, foreign keys) and trigger execution.

use super::*;

impl SqlExecutor {
    /// B5 — parse a stored CHECK expression back into a row predicate.
    pub(super) fn parse_check_expr(check: &str) -> Result<Expr, String> {
        let sql = format!("SELECT * FROM __vv_check WHERE {check}");
        let tokens = Lexer::new(&sql).tokenize().map_err(|e| e.to_string())?;
        match SqlParser::new(tokens).parse().map_err(|e| e.to_string())? {
            Stmt::Select(sel) => sel
                .where_clause
                .ok_or_else(|| "empty CHECK expression".to_owned()),
            _ => Err("not a row predicate".to_owned()),
        }
    }

    /// B5 — reject, at CREATE time, any CHECK the row evaluator cannot
    /// enforce (instead of storing a constraint that never fires).
    pub(super) fn validate_checks(
        checks: &[Option<String>],
        schema: &TableSchema,
        table: &str,
    ) -> Result<(), String> {
        for (idx, check) in checks.iter().enumerate() {
            let Some(check) = check else { continue };
            let col = schema.get(idx).map(|c| c.name.as_str()).unwrap_or("?");
            let expr = Self::parse_check_expr(check)
                .map_err(|e| format!("unsupported CHECK constraint on column {col}: {e}"))?;
            Self::validate_policy_expr(&expr, schema, table)
                .map_err(|e| format!("unsupported CHECK constraint on column {col}: {e}"))?;
        }
        Ok(())
    }

    /// The open per-table txn id, without joining the table to the txn.
    pub(super) fn open_txn_for(&self, table: &str) -> Option<TxnId> {
        self.txn.lock_safe().as_ref()?.get(table).copied()
    }

    /// Point read that sees this session's staged (uncommitted) writes.
    pub(super) fn get_visible(
        &self,
        table: &str,
        engine: &vectorvolt_engine::TableEngine,
        pk: u64,
    ) -> Result<Option<EngineRow>, ExecError> {
        match self.open_txn_for(table) {
            Some(id) => engine.get_txn(id, pk),
            None => engine.get(pk),
        }
        .map_err(|e| ExecError::Engine(e.to_string()))
    }

    /// Full scan that overlays this session's staged (uncommitted) writes.
    pub(super) fn scan_visible(
        &self,
        table: &str,
        engine: &vectorvolt_engine::TableEngine,
    ) -> Result<Vec<EngineRow>, ExecError> {
        let mut rows = self.scan_live(engine)?;
        if let Some(id) = self.open_txn_for(table) {
            let ops = engine
                .txn_write_set(id)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
            if !ops.is_empty() {
                let staged: HashMap<u64, Option<Vec<u64>>> = ops.into_iter().collect();
                rows.retain(|r| !staged.contains_key(&r.pk));
                for (pk, values) in staged {
                    if let Some(values) = values {
                        rows.push(EngineRow { pk, values });
                    }
                }
            }
        }
        Ok(rows)
    }

    /// B5/B6 — enforce `table`'s CHECK constraints (23514) and outgoing
    /// FOREIGN KEYs (23503) on a row about to be written. `old` is the row
    /// being replaced (UPDATE / upsert); unchanged FK columns are not
    /// re-checked. Returns the error result to send back, if any.
    pub(super) fn check_row_constraints(
        &self,
        table: &str,
        schema: &TableSchema,
        vals: &[u64],
        old: Option<&[u64]>,
    ) -> Result<Option<QueryResult>, ExecError> {
        let checks = self
            .col_checks
            .lock_safe()
            .get(table)
            .cloned()
            .unwrap_or_default();
        if checks.iter().any(Option::is_some) {
            let row = EngineRow {
                pk: vals.first().copied().unwrap_or(0),
                values: vals.to_vec(),
            };
            for (idx, check) in checks.iter().enumerate() {
                let Some(check) = check else { continue };
                let col = schema.get(idx).map(|c| c.name.clone()).unwrap_or_default();
                let expr = match Self::parse_check_expr(check) {
                    Ok(e) => e,
                    Err(e) => {
                        return Ok(Some(sql_error(
                            "0A000",
                            &format!("CHECK constraint on column {col} cannot be evaluated: {e}"),
                        )));
                    }
                };
                if !self.eval_expr(&expr, &row, schema, table)? {
                    return Ok(Some(sql_error(
                        "23514",
                        &format!(
                            "new row for relation \"{table}\" violates check constraint \"{table}_{col}_check\" (column {col}: CHECK ({check}))"
                        ),
                    )));
                }
            }
        }

        let fks = self
            .foreign_keys
            .lock_safe()
            .get(table)
            .cloned()
            .unwrap_or_default();
        for (fk_col, ref_table, ref_col) in &fks {
            let Some((fk_idx, fk_def)) = schema.find(fk_col) else {
                continue;
            };
            let bits = vals.get(fk_idx).copied().unwrap_or(0);
            if old.is_some_and(|o| o.get(fk_idx).copied().unwrap_or(0) == bits) {
                continue;
            }
            let val_str = TypedValue::from_bits(fk_def.col_type, bits).display_str();
            if val_str == "0" || val_str == "NULL" {
                continue; // NULL FK values are allowed
            }
            let ref_engine = self
                .registry
                .get(ref_table)
                .map_err(|_| ExecError::TableNotFound(ref_table.clone()))?;
            let Some((ref_idx, ref_def)) = ref_engine.schema.find(ref_col) else {
                return Err(ExecError::ColumnNotFound(format!("{ref_table}.{ref_col}")));
            };
            let ref_type = ref_def.col_type;
            let found = if ref_idx == 0 {
                match TypedValue::parse(ref_type, &val_str) {
                    Ok(tv) => self
                        .get_visible(ref_table, &ref_engine, tv.to_bits())?
                        .is_some(),
                    Err(_) => false,
                }
            } else {
                self.scan_visible(ref_table, &ref_engine)?.iter().any(|r| {
                    TypedValue::from_bits(ref_type, r.values.get(ref_idx).copied().unwrap_or(0))
                        .display_str()
                        == val_str
                })
            };
            if !found {
                return Ok(Some(sql_error(
                    "23503",
                    &format!(
                        "insert or update on table \"{table}\" violates foreign key constraint: key ({fk_col})=({val_str}) is not present in table \"{ref_table}\" ({ref_col})"
                    ),
                )));
            }
        }
        Ok(None)
    }

    /// B6 — parent side of every FOREIGN KEY referencing `table` (RESTRICT,
    /// the only action the parser records): refuse to delete a row, or to
    /// change its referenced column, while child rows still point at it.
    /// `changes` is (old row, new row) with `None` for a delete.
    pub(super) fn check_parent_refs(
        &self,
        table: &str,
        schema: &TableSchema,
        changes: &[(&[u64], Option<&[u64]>)],
    ) -> Result<Option<QueryResult>, ExecError> {
        let referencing: Vec<(String, String, String)> = self
            .foreign_keys
            .lock_safe()
            .iter()
            .flat_map(|(child, fks)| {
                fks.iter()
                    .filter(|(_, ref_table, _)| ref_table == table)
                    .map(move |(col, _, ref_col)| (child.clone(), col.clone(), ref_col.clone()))
            })
            .collect();
        if referencing.is_empty() || changes.is_empty() {
            return Ok(None);
        }
        let deleted: HashSet<u64> = changes
            .iter()
            .filter(|(_, new)| new.is_none())
            .map(|(old, _)| old.first().copied().unwrap_or(0))
            .collect();
        for (child, fk_col, ref_col) in &referencing {
            let Some((ref_idx, ref_def)) = schema.find(ref_col) else {
                continue;
            };
            let at = |v: &[u64], i: usize| v.get(i).copied().unwrap_or(0);
            let gone: HashSet<String> = changes
                .iter()
                .filter(|(old, new)| new.is_none_or(|n| at(n, ref_idx) != at(old, ref_idx)))
                .map(|(old, _)| {
                    TypedValue::from_bits(ref_def.col_type, at(old, ref_idx)).display_str()
                })
                .filter(|v| v != "0" && v != "NULL")
                .collect();
            if gone.is_empty() {
                continue;
            }
            let Ok(child_engine) = self.registry.get(child) else {
                continue;
            };
            let Some((fk_idx, fk_def)) = child_engine.schema.find(fk_col) else {
                continue;
            };
            let fk_type = fk_def.col_type;
            let still_referenced = self
                .scan_visible(child, &child_engine)?
                .iter()
                .filter(|r| !(child == table && deleted.contains(&r.pk)))
                .map(|r| TypedValue::from_bits(fk_type, at(&r.values, fk_idx)).display_str())
                .find(|v| gone.contains(v));
            if let Some(v) = still_referenced {
                return Ok(Some(sql_error(
                    "23503",
                    &format!(
                        "update or delete on table \"{table}\" violates foreign key constraint on table \"{child}\": key ({ref_col})=({v}) is still referenced from {child}.{fk_col}"
                    ),
                )));
            }
        }
        Ok(None)
    }

    /// B7 — the SQL a trigger runs: a stored procedure's statements
    /// (`CREATE PROCEDURE f AS BEGIN ...; END`) or a function whose body is
    /// `;`-separated SQL statements.
    pub(super) fn trigger_body(&self, func: &str) -> Result<Vec<String>, String> {
        if let Some(body) = self
            .procedures
            .lock_safe()
            .get(&func.to_ascii_lowercase())
            .cloned()
        {
            return Ok(body);
        }
        let Some((_, body)) = self.udfs.lock_safe().get(func).cloned() else {
            return Err(format!("function {func}() does not exist"));
        };
        let stmts: Vec<String> = body
            .split(';')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(stmts)
    }

    /// B7 — SQL literal for a cell, for `NEW.col` / `OLD.col` substitution.
    pub(super) fn sql_literal(col_type: ColumnType, bits: u64) -> String {
        let v = TypedValue::from_bits(col_type, bits).display_str();
        if v.parse::<f64>().is_ok() || v == "true" || v == "false" {
            v
        } else {
            format!("'{}'", v.replace('\'', "''"))
        }
    }

    /// B7 — replace `NEW.col` / `OLD.col` in a trigger statement with the
    /// row's values.
    pub(super) fn substitute_row_refs(
        sql: &str,
        schema: &TableSchema,
        new: Option<&[u64]>,
        old: Option<&[u64]>,
    ) -> String {
        let mut cols: Vec<(usize, &ColumnDef)> = schema.columns().iter().enumerate().collect();
        // Longest names first so NEW.val is not clobbered by NEW.v.
        cols.sort_by_key(|(_, c)| std::cmp::Reverse(c.name.len()));
        let mut out = sql.to_owned();
        for (prefix, vals) in [("NEW.", new), ("OLD.", old)] {
            let Some(vals) = vals else { continue };
            for (i, c) in &cols {
                let lit = Self::sql_literal(c.col_type, vals.get(*i).copied().unwrap_or(0));
                for p in [prefix.to_owned(), prefix.to_ascii_lowercase()] {
                    out = out.replace(&format!("{p}{}", c.name), &lit);
                }
            }
        }
        out
    }

    /// B7 — validate CREATE TRIGGER: the table must exist and the function
    /// must resolve to SQL statements the executor can run.
    pub(super) fn validate_trigger(&self, def: &TriggerDef) -> Result<(), String> {
        let engine = self
            .registry
            .get(&def.table)
            .map_err(|_| format!("relation \"{}\" does not exist", def.table))?;
        let body = self.trigger_body(&def.func)?;
        if body.is_empty() {
            return Err(format!("trigger function {}() has an empty body", def.func));
        }
        let zeros = vec![0u64; engine.schema.col_count()];
        let is_proc = self
            .procedures
            .lock_safe()
            .contains_key(&def.func.to_ascii_lowercase());
        if !is_proc {
            for stmt in &body {
                let sql =
                    Self::substitute_row_refs(stmt, &engine.schema, Some(&zeros), Some(&zeros));
                let parsed = Lexer::new(&sql)
                    .tokenize()
                    .map_err(|e| e.to_string())
                    .and_then(|t| SqlParser::new(t).parse().map_err(|e| e.to_string()));
                if let Err(e) = parsed {
                    return Err(format!(
                        "function {}() cannot be used as a trigger: its body must be SQL statements ({e})",
                        def.func
                    ));
                }
            }
        }
        Ok(())
    }

    /// B7 — fire `table`'s row triggers for one row. Triggers run their SQL
    /// through `execute` in this session (same transaction); an error from
    /// any of them aborts the firing statement. Nesting is capped at
    /// `MAX_TRIGGER_DEPTH` (SQLSTATE 54001).
    pub(super) fn fire_triggers(
        &self,
        table: &str,
        timing: TriggerTiming,
        event: TriggerEvent,
        schema: &TableSchema,
        new: Option<&[u64]>,
        old: Option<&[u64]>,
    ) -> Result<Option<QueryResult>, ExecError> {
        use std::sync::atomic::Ordering::SeqCst;
        let mut defs: Vec<TriggerDef> = self
            .triggers
            .lock_safe()
            .values()
            .filter(|t| t.table == table && t.timing == timing && t.event == event)
            .cloned()
            .collect();
        if defs.is_empty() {
            return Ok(None);
        }
        // PostgreSQL fires same-kind triggers in name order.
        defs.sort_by(|a, b| a.name.cmp(&b.name));

        struct DepthGuard<'a>(&'a std::sync::atomic::AtomicU32);
        impl Drop for DepthGuard<'_> {
            fn drop(&mut self) {
                self.0.fetch_sub(1, SeqCst);
            }
        }
        let depth = self.trigger_depth.fetch_add(1, SeqCst);
        let _depth = DepthGuard(&self.trigger_depth);
        if depth >= MAX_TRIGGER_DEPTH {
            return Ok(Some(sql_error(
                "54001",
                &format!("trigger nesting depth exceeded (max {MAX_TRIGGER_DEPTH})"),
            )));
        }
        for def in defs {
            let body = match self.trigger_body(&def.func) {
                Ok(b) => b,
                Err(e) => return Ok(Some(sql_error("42883", &e))),
            };
            for stmt in body {
                let sql = Self::substitute_row_refs(&stmt, schema, new, old);
                if let r @ QueryResult::Error(_) = self.execute(&sql)? {
                    return Ok(Some(r));
                }
            }
        }
        Ok(None)
    }
}
