//! INSERT / UPDATE / DELETE / COPY execution.

use super::*;

impl SqlExecutor {
    pub(super) fn exec_insert_select(
        &self,
        stmt: InsertSelectStmt,
    ) -> Result<QueryResult, ExecError> {
        // S81/S82 — the target table is checked here, the source by exec_select
        if let Some(denied) = self.check_table_access(&stmt.table, "INSERT") {
            return Ok(denied);
        }
        let result = self.exec_select(*stmt.select)?;
        let (src_cols, src_rows) = match result {
            QueryResult::Rows { columns, rows } => (columns, rows),
            _ => return Err(ExecError::Engine("expected rows from SELECT".into())),
        };
        let engine = self
            .registry
            .get(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
        let schema = &engine.schema;
        let col_count = schema.col_count();
        let mut count = 0u64;
        for row_strs in src_rows {
            let mut vals: Vec<u64> = vec![0u64; col_count];
            for (col_name, val_str) in src_cols.iter().zip(row_strs.iter()) {
                if let Some((idx, col_def)) = schema.find(col_name)
                    && let Ok(tv) = TypedValue::parse(col_def.col_type, val_str)
                {
                    vals[idx] = tv.to_bits();
                }
            }
            self.check_deadline()?;
            // B5/B6 — CHECK constraints and foreign keys
            if let Some(err) = self.check_row_constraints(&stmt.table, schema, &vals, None)? {
                return Ok(err);
            }
            let pk = vals[0];
            engine
                .insert(pk, vals)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
            count += 1;
        }
        Ok(QueryResult::Affected(count))
    }

    pub(super) fn exec_insert(&self, stmt: InsertStmt) -> Result<QueryResult, ExecError> {
        if *self.read_only.lock_safe() {
            return Ok(sql_error("25006", "read-only transaction"));
        }
        // S81/S82 — token auth + rate limit apply to writes too
        if let Some(denied) = self.check_table_access(&stmt.table, "INSERT") {
            return Ok(denied);
        }
        let engine = self
            .registry
            .get(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;

        let schema = &engine.schema;

        if stmt.columns.len() != stmt.values.len() {
            return Err(ExecError::ColumnCountMismatch {
                expected: stmt.columns.len(),
                got: stmt.values.len(),
            });
        }

        // Build a full-width values vec in schema column order
        let col_count = schema.col_count();
        let mut row_vals: Vec<u64> = vec![0u64; col_count];

        // S59 â€” apply defaults for missing columns
        if let Some(defaults) = self.col_defaults.lock_safe().get(&stmt.table).cloned() {
            for (col_idx, maybe_default) in defaults.iter().enumerate() {
                if let Some(default_val) = maybe_default {
                    let col_name = schema
                        .get(col_idx)
                        .map(|c| c.name.clone())
                        .unwrap_or_default();
                    if !stmt.columns.contains(&col_name) {
                        let col_type = schema
                            .get(col_idx)
                            .map(|c| c.col_type)
                            .unwrap_or(ColumnType::U64);
                        let bits_result = match default_val {
                            LitVal::Int(n) => TypedValue::parse(col_type, &n.to_string()),
                            LitVal::Float(f) => TypedValue::parse(col_type, &f.to_string()),
                            LitVal::Str(s) => TypedValue::parse(col_type, s),
                            LitVal::Bool(b) => TypedValue::parse(col_type, &b.to_string()),
                            LitVal::Null => Ok(TypedValue::from_bits(col_type, 0)),
                        };
                        if let Ok(tv) = bits_result {
                            row_vals[col_idx] = tv.to_bits();
                        }
                    }
                }
            }
        }

        for (col_name, expr) in stmt.columns.iter().zip(stmt.values.iter()) {
            let (idx, col_def) = schema
                .find(col_name)
                .ok_or_else(|| ExecError::ColumnNotFound(col_name.clone()))?;
            let bits = self.expr_to_bits(expr, col_def.col_type)?;
            row_vals[idx] = bits;
        }

        // B7 — BEFORE INSERT row triggers
        if let Some(err) = self.fire_triggers(
            &stmt.table,
            TriggerTiming::Before,
            TriggerEvent::Insert,
            schema,
            Some(&row_vals),
            None,
        )? {
            return Ok(err);
        }

        let pk = row_vals[0];

        // S44 â€” ON CONFLICT DO NOTHING
        if stmt.on_conflict_ignore {
            let already_exists = engine
                .get(pk)
                .map_err(|e| ExecError::Engine(e.to_string()))?
                .is_some();
            if already_exists {
                return Ok(QueryResult::Affected(0));
            }
        }

        // S44 â€” ON CONFLICT DO UPDATE
        if let Some(updates) = &stmt.on_conflict_update {
            let already_exists = engine
                .get(pk)
                .map_err(|e| ExecError::Engine(e.to_string()))?
                .is_some();
            if already_exists {
                let current = engine
                    .get(pk)
                    .map_err(|e| ExecError::Engine(e.to_string()))?
                    .ok_or_else(|| ExecError::Engine("row vanished".into()))?;
                let mut new_vals = current.values.clone();
                for (col_name, expr) in updates {
                    if let Some((idx, col_def)) = schema.find(col_name) {
                        new_vals[idx] = self.expr_to_bits(expr, col_def.col_type)?;
                    }
                }
                // B5/B6/B7 — the upsert's UPDATE is checked and fires UPDATE
                // triggers like any other update.
                if let Some(err) = self.fire_triggers(
                    &stmt.table,
                    TriggerTiming::Before,
                    TriggerEvent::Update,
                    schema,
                    Some(&new_vals),
                    Some(&current.values),
                )? {
                    return Ok(err);
                }
                if let Some(err) = self.check_row_constraints(
                    &stmt.table,
                    schema,
                    &new_vals,
                    Some(&current.values),
                )? {
                    return Ok(err);
                }
                let changes = [(current.values.as_slice(), Some(new_vals.as_slice()))];
                if let Some(err) = self.check_parent_refs(&stmt.table, schema, &changes)? {
                    return Ok(err);
                }
                engine
                    .update(pk, new_vals.clone())
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                if let Some(err) = self.fire_triggers(
                    &stmt.table,
                    TriggerTiming::After,
                    TriggerEvent::Update,
                    schema,
                    Some(&new_vals),
                    Some(&current.values),
                )? {
                    return Ok(err);
                }
                return Ok(QueryResult::Affected(1));
            }
        }

        // B5/B6 — CHECK constraints and foreign keys on the new row
        if let Some(err) = self.check_row_constraints(&stmt.table, schema, &row_vals, None)? {
            return Ok(err);
        }
        let after_vals = row_vals.clone();

        if let Some(txn_id) = self.txn_id_for(&stmt.table) {
            engine
                .insert_txn(txn_id, pk, row_vals)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
            self.record_write(&stmt.table, pk);
        } else if stmt.or_replace
            && engine
                .get(pk)
                .map_err(|e| ExecError::Engine(e.to_string()))?
                .is_some()
        {
            engine
                .update(pk, row_vals)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
        } else {
            engine
                .insert(pk, row_vals)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
        }

        // Persist text values for TEXT columns into the FTS sidecar.
        // Persist vector values for Vector columns into the VectorStore sidecar.
        for (col_name, expr) in stmt.columns.iter().zip(stmt.values.iter()) {
            if let Some((_, col_def)) = schema.find(col_name) {
                let base = self.registry.base_dir();
                match col_def.col_type {
                    ColumnType::Text => {
                        if let Expr::Lit(LitVal::Str(text)) = expr
                            && let Ok(mut store) = TextStore::load(base, &stmt.table, col_name)
                        {
                            store.insert(pk, text.clone());
                            let _ = store.save();
                        }
                    }
                    ColumnType::Vector(_) => {
                        if let Expr::Lit(LitVal::Str(vec_str)) = expr
                            && let Some(vec) = parse_vec_literal(vec_str)
                        {
                            if let Ok(mut store) = VectorStore::load(base, &stmt.table, col_name) {
                                store.insert(pk, vec.clone());
                                let _ = store.save();
                            }
                            if let Ok(mut idx) =
                                HnswIndex::load(base, &stmt.table, col_name, DistanceMetric::L2)
                            {
                                idx.insert(pk, vec);
                                let _ = idx.save();
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // S80 â€” audit log
        {
            let new_val = stmt
                .values
                .iter()
                .map(|e| match e {
                    Expr::Lit(LitVal::Str(s)) => s.clone(),
                    Expr::Lit(LitVal::Int(n)) => n.to_string(),
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join(",");
            self.audit_log.lock_safe().push(AuditEntry {
                ts: "2026-09-11 00:00:00".to_owned(),
                op: "INSERT".to_owned(),
                table: stmt.table.clone(),
                old_val: String::new(),
                new_val,
            });
        }

        // B7 — AFTER INSERT row triggers
        if let Some(err) = self.fire_triggers(
            &stmt.table,
            TriggerTiming::After,
            TriggerEvent::Insert,
            schema,
            Some(&after_vals),
            None,
        )? {
            return Ok(err);
        }

        // S73 â€” extra rows
        for row_vals_exprs in stmt.extra_rows {
            let extra_insert = InsertStmt {
                table: stmt.table.clone(),
                columns: stmt.columns.clone(),
                values: row_vals_exprs,
                or_replace: stmt.or_replace,
                on_conflict_update: None,
                on_conflict_ignore: false,
                extra_rows: vec![],
            };
            self.check_deadline()?;
            let r = self.exec_insert(extra_insert)?;
            if matches!(r, QueryResult::Error(_)) {
                return Ok(r);
            }
        }

        Ok(QueryResult::Affected(1))
    }

    pub(super) fn exec_update(&self, stmt: UpdateStmt) -> Result<QueryResult, ExecError> {
        if *self.read_only.lock_safe() {
            return Ok(sql_error("25006", "read-only transaction"));
        }
        // S81/S82 — token auth + rate limit apply to writes too
        if let Some(denied) = self.check_table_access(&stmt.table, "UPDATE") {
            return Ok(denied);
        }
        let engine = self
            .registry
            .get(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
        let schema = &engine.schema;

        let all_rows = self.scan_live(&engine)?;
        let to_update =
            self.filter_rows(all_rows, stmt.where_clause.as_ref(), schema, &stmt.table)?;
        // S78 — only rows visible under the table's RLS policies may be updated
        let to_update = self.apply_rls(to_update, schema, &stmt.table)?;
        let count = to_update.len() as u64;

        // B5/B6/B7 — compute every new row, fire BEFORE triggers and check
        // constraints before anything is written, so a violation leaves the
        // table untouched.
        let mut planned: Vec<(EngineRow, Vec<u64>)> = Vec::with_capacity(to_update.len());
        for row in to_update {
            self.check_deadline()?;
            let mut new_vals = row.values.clone();
            for (col_name, expr) in &stmt.assignments {
                let (idx, col_def) = schema
                    .find(col_name)
                    .ok_or_else(|| ExecError::ColumnNotFound(col_name.clone()))?;
                new_vals[idx] = self.expr_to_bits(expr, col_def.col_type)?;
            }
            if let Some(err) = self.fire_triggers(
                &stmt.table,
                TriggerTiming::Before,
                TriggerEvent::Update,
                schema,
                Some(&new_vals),
                Some(&row.values),
            )? {
                return Ok(err);
            }
            if let Some(err) =
                self.check_row_constraints(&stmt.table, schema, &new_vals, Some(&row.values))?
            {
                return Ok(err);
            }
            planned.push((row, new_vals));
        }
        let changes: Vec<(&[u64], Option<&[u64]>)> = planned
            .iter()
            .map(|(row, new_vals)| (row.values.as_slice(), Some(new_vals.as_slice())))
            .collect();
        if let Some(err) = self.check_parent_refs(&stmt.table, schema, &changes)? {
            return Ok(err);
        }
        drop(changes);

        for (row, new_vals) in planned {
            self.check_deadline()?;
            if let Some(txn_id) = self.txn_id_for(&stmt.table) {
                engine
                    .delete_txn(txn_id, row.pk)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                engine
                    .insert_txn(txn_id, row.pk, new_vals.clone())
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                self.record_write(&stmt.table, row.pk);
            } else {
                engine
                    .update(row.pk, new_vals.clone())
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
            if let Some(err) = self.fire_triggers(
                &stmt.table,
                TriggerTiming::After,
                TriggerEvent::Update,
                schema,
                Some(&new_vals),
                Some(&row.values),
            )? {
                return Ok(err);
            }
        }

        // S80 â€” audit log for UPDATE
        self.audit_log.lock_safe().push(AuditEntry {
            ts: "2026-09-11 00:00:00".to_owned(),
            op: "UPDATE".to_owned(),
            table: stmt.table.clone(),
            old_val: String::new(),
            new_val: count.to_string(),
        });

        Ok(QueryResult::Affected(count))
    }

    pub(super) fn exec_delete(&self, stmt: DeleteStmt) -> Result<QueryResult, ExecError> {
        if *self.read_only.lock_safe() {
            return Ok(sql_error("25006", "read-only transaction"));
        }
        // S81/S82 — token auth + rate limit apply to writes too
        if let Some(denied) = self.check_table_access(&stmt.table, "DELETE") {
            return Ok(denied);
        }
        let engine = self
            .registry
            .get(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;

        let schema = &engine.schema;

        // Get all live rows then filter
        let all_rows = self.scan_live(&engine)?;

        let to_delete =
            self.filter_rows(all_rows, stmt.where_clause.as_ref(), schema, &stmt.table)?;
        // S78 — only rows visible under the table's RLS policies may be deleted
        let to_delete = self.apply_rls(to_delete, schema, &stmt.table)?;
        let count = to_delete.len() as u64;

        // B7 — BEFORE DELETE row triggers; B6 — RESTRICT on referenced rows.
        for row in &to_delete {
            self.check_deadline()?;
            if let Some(err) = self.fire_triggers(
                &stmt.table,
                TriggerTiming::Before,
                TriggerEvent::Delete,
                schema,
                None,
                Some(&row.values),
            )? {
                return Ok(err);
            }
        }
        let changes: Vec<(&[u64], Option<&[u64]>)> = to_delete
            .iter()
            .map(|row| (row.values.as_slice(), None))
            .collect();
        if let Some(err) = self.check_parent_refs(&stmt.table, schema, &changes)? {
            return Ok(err);
        }
        drop(changes);

        for row in &to_delete {
            self.check_deadline()?;
            if let Some(txn_id) = self.txn_id_for(&stmt.table) {
                engine
                    .delete_txn(txn_id, row.pk)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                self.record_write(&stmt.table, row.pk);
            } else {
                engine
                    .delete(row.pk)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
            if let Some(err) = self.fire_triggers(
                &stmt.table,
                TriggerTiming::After,
                TriggerEvent::Delete,
                schema,
                None,
                Some(&row.values),
            )? {
                return Ok(err);
            }
        }

        // S80 â€” audit log for DELETE
        self.audit_log.lock_safe().push(AuditEntry {
            ts: "2026-09-11 00:00:00".to_owned(),
            op: "DELETE".to_owned(),
            table: stmt.table.clone(),
            old_val: count.to_string(),
            new_val: String::new(),
        });

        // S114 â€” track dead rows for VACUUM
        *self
            .dead_rows
            .lock_safe()
            .entry(stmt.table.clone())
            .or_insert(0) += count;

        Ok(QueryResult::Affected(count))
    }

    // S74 â€” COPY FROM
    pub(super) fn exec_copy_from(
        &self,
        table: String,
        path: String,
        has_header: bool,
    ) -> Result<QueryResult, ExecError> {
        // S81/S82 — token auth + rate limit (covers COPY FROM STDIN too)
        if let Some(denied) = self.check_table_access(&table, "INSERT") {
            return Ok(denied);
        }
        // S104 â€” COPY FROM STDIN with inline tab-delimited data
        if let Some(data) = path.strip_prefix("__STDIN__:") {
            return self.exec_copy_from_stdin(table, data.to_owned());
        }
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        let engine = self
            .registry
            .get(&table)
            .map_err(|_| ExecError::TableNotFound(table.clone()))?;
        let schema = &engine.schema;
        let col_names: Vec<String> = schema.columns().iter().map(|c| c.name.clone()).collect();
        let file = File::open(&path).map_err(|e| ExecError::Engine(format!("COPY: {e}")))?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        if has_header {
            let _ = lines.next();
        }
        let mut count = 0u64;
        for line in lines {
            let line = line.map_err(|e| ExecError::Engine(format!("COPY read: {e}")))?;
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != col_names.len() {
                continue;
            }
            let values: Vec<Expr> = parts
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let v = v.trim();
                    let col_def = schema.columns().get(i);
                    match col_def.map(|c| c.col_type) {
                        Some(ColumnType::U64) | Some(ColumnType::I64) => {
                            if let Ok(n) = v.parse::<i64>() {
                                Expr::Lit(LitVal::Int(n))
                            } else {
                                Expr::Lit(LitVal::Str(v.to_owned()))
                            }
                        }
                        Some(ColumnType::F64) => {
                            if let Ok(f) = v.parse::<f64>() {
                                Expr::Lit(LitVal::Float(f))
                            } else {
                                Expr::Lit(LitVal::Str(v.to_owned()))
                            }
                        }
                        _ => Expr::Lit(LitVal::Str(v.to_owned())),
                    }
                })
                .collect();
            let insert = InsertStmt {
                table: table.clone(),
                columns: col_names.clone(),
                values,
                or_replace: false,
                on_conflict_update: None,
                on_conflict_ignore: false,
                extra_rows: vec![],
            };
            self.exec_insert(insert)?;
            count += 1;
        }
        Ok(QueryResult::Affected(count))
    }

    // S104 â€” COPY FROM STDIN with inline tab-delimited data
    pub(super) fn exec_copy_from_stdin(
        &self,
        table: String,
        data: String,
    ) -> Result<QueryResult, ExecError> {
        let engine = self
            .registry
            .get(&table)
            .map_err(|_| ExecError::TableNotFound(table.clone()))?;
        let schema = &engine.schema;
        let col_names: Vec<String> = schema.columns().iter().map(|c| c.name.clone()).collect();
        let mut count = 0u64;
        for line in data.lines() {
            let line = line.trim();
            if line == "\\." || line == "." {
                break;
            }
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != col_names.len() {
                continue;
            }
            let values: Vec<Expr> = parts
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let v = v.trim();
                    let col_def = schema.columns().get(i);
                    match col_def.map(|c| c.col_type) {
                        Some(ColumnType::U64) | Some(ColumnType::I64) => {
                            if let Ok(n) = v.parse::<i64>() {
                                Expr::Lit(LitVal::Int(n))
                            } else {
                                Expr::Lit(LitVal::Str(v.to_owned()))
                            }
                        }
                        Some(ColumnType::F64) => {
                            if let Ok(f) = v.parse::<f64>() {
                                Expr::Lit(LitVal::Float(f))
                            } else {
                                Expr::Lit(LitVal::Str(v.to_owned()))
                            }
                        }
                        _ => Expr::Lit(LitVal::Str(v.to_owned())),
                    }
                })
                .collect();
            let insert = InsertStmt {
                table: table.clone(),
                columns: col_names.clone(),
                values,
                or_replace: false,
                on_conflict_update: None,
                on_conflict_ignore: false,
                extra_rows: vec![],
            };
            self.exec_insert(insert)?;
            count += 1;
        }
        Ok(QueryResult::Affected(count))
    }

    pub(super) fn expr_to_bits(&self, expr: &Expr, col_type: ColumnType) -> Result<u64, ExecError> {
        let s = match expr {
            Expr::Lit(lit) => match lit {
                LitVal::Int(n) => n.to_string(),
                LitVal::Float(f) => format!("{f:.6}"),
                LitVal::Str(s) => s.clone(),
                LitVal::Bool(b) => b.to_string(),
                LitVal::Null => "0".to_owned(),
            },
            _ => return Err(ExecError::TypeError("expected literal in INSERT".into())),
        };
        TypedValue::parse(col_type, &s)
            .map(|v| v.to_bits())
            .map_err(|e| ExecError::TypeError(e.to_string()))
    }
}
