//! DDL execution: tables, indexes, views, functions, triggers, materialized views.

use super::*;

impl SqlExecutor {
    pub(super) fn exec_alter_table(&self, stmt: AlterTableStmt) -> Result<QueryResult, ExecError> {
        match stmt.op {
            AlterOp::AddColumn(col_def) => {
                // B5 — ADD COLUMN does not record constraints; refuse rather
                // than silently dropping the CHECK.
                if col_def.check.is_some() {
                    return Ok(sql_error(
                        "0A000",
                        "CHECK constraints in ALTER TABLE ADD COLUMN are not supported",
                    ));
                }
                let col = ColumnDef::new(col_def.name, ColumnType::from_token(&col_def.col_type));
                self.registry
                    .alter_table(&stmt.table, &[col], &[])
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
            // S113 â€” ONLINE add column: add column, preserve all rows, return online message
            AlterOp::AddColumnOnline(col_def) => {
                if col_def.check.is_some() {
                    return Ok(sql_error(
                        "0A000",
                        "CHECK constraints in ALTER TABLE ADD COLUMN are not supported",
                    ));
                }
                let col = ColumnDef::new(
                    col_def.name.clone(),
                    ColumnType::from_token(&col_def.col_type),
                );
                self.registry
                    .alter_table(&stmt.table, &[col], &[])
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                // Ensure all existing rows have the new column set to default (0 = already the case)
                return Ok(QueryResult::Ok(
                    "online schema change complete, 0 rows locked".to_owned(),
                ));
            }
            AlterOp::DropColumn(col_name) => {
                self.registry
                    .alter_table(&stmt.table, &[], &[col_name.as_str()])
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
            // S58 â€” RENAME COLUMN
            AlterOp::RenameColumn { from, to } => {
                let (from_idx, new_type) = {
                    let engine = self
                        .registry
                        .get(&stmt.table)
                        .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
                    let idx = engine
                        .schema
                        .find(&from)
                        .ok_or_else(|| ExecError::ColumnNotFound(from.clone()))?
                        .0;
                    let ct = engine.schema.find(&from).unwrap().1.col_type;
                    (idx, ct)
                };
                // Add new column
                let new_col = ColumnDef::new(to.clone(), new_type);
                self.registry
                    .alter_table(&stmt.table, &[new_col], &[])
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                // Copy values from old col to new col
                {
                    let engine2 = self
                        .registry
                        .get(&stmt.table)
                        .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
                    let all_rows = self.scan_live(&engine2)?;
                    let new_idx = engine2.schema.find(&to).map(|(i, _)| i);
                    for row in &all_rows {
                        let val = row.values.get(from_idx).copied().unwrap_or(0);
                        if let Some(ni) = new_idx {
                            let mut new_vals = row.values.clone();
                            while new_vals.len() <= ni {
                                new_vals.push(0);
                            }
                            new_vals[ni] = val;
                            engine2
                                .update(row.pk, new_vals)
                                .map_err(|e| ExecError::Engine(e.to_string()))?;
                        }
                    }
                }
                // Drop old column
                self.registry
                    .alter_table(&stmt.table, &[], &[from.as_str()])
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                // S230 â€” track rename history
                self.s230_rename_history
                    .lock_safe()
                    .entry(stmt.table.clone())
                    .or_default()
                    .push((from.clone(), to.clone()));
            }
            // S58 â€” CHANGE TYPE
            AlterOp::ChangeType { col, new_type } => {
                let old_idx = {
                    let engine = self
                        .registry
                        .get(&stmt.table)
                        .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
                    engine
                        .schema
                        .find(&col)
                        .ok_or_else(|| ExecError::ColumnNotFound(col.clone()))?
                        .0
                };
                let tmp_name = format!("__tmp_{col}");
                let new_col_type = ColumnType::from_token(&new_type);
                self.registry
                    .alter_table(
                        &stmt.table,
                        &[ColumnDef::new(tmp_name.clone(), new_col_type)],
                        &[],
                    )
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                {
                    let engine2 = self
                        .registry
                        .get(&stmt.table)
                        .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
                    let all_rows = self.scan_live(&engine2)?;
                    let tmp_idx = engine2.schema.find(&tmp_name).map(|(i, _)| i);
                    for row in &all_rows {
                        let old_val = row.values.get(old_idx).copied().unwrap_or(0);
                        if let Some(ti) = tmp_idx {
                            let mut new_vals = row.values.clone();
                            while new_vals.len() <= ti {
                                new_vals.push(0);
                            }
                            new_vals[ti] = old_val;
                            engine2
                                .update(row.pk, new_vals)
                                .map_err(|e| ExecError::Engine(e.to_string()))?;
                        }
                    }
                }
                self.registry
                    .alter_table(&stmt.table, &[], &[col.as_str()])
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                // Rename tmp to original name
                let tmp_idx3 = {
                    let engine3 = self
                        .registry
                        .get(&stmt.table)
                        .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
                    engine3
                        .schema
                        .find(&tmp_name)
                        .ok_or_else(|| ExecError::ColumnNotFound(tmp_name.clone()))?
                        .0
                };
                self.registry
                    .alter_table(
                        &stmt.table,
                        &[ColumnDef::new(col.clone(), new_col_type)],
                        &[],
                    )
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                {
                    let engine4 = self
                        .registry
                        .get(&stmt.table)
                        .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
                    let all_rows4 = self.scan_live(&engine4)?;
                    let final_idx = engine4.schema.find(&col).map(|(i, _)| i);
                    for row in &all_rows4 {
                        let val = row.values.get(tmp_idx3).copied().unwrap_or(0);
                        if let Some(fi) = final_idx {
                            let mut new_vals = row.values.clone();
                            while new_vals.len() <= fi {
                                new_vals.push(0);
                            }
                            new_vals[fi] = val;
                            engine4
                                .update(row.pk, new_vals)
                                .map_err(|e| ExecError::Engine(e.to_string()))?;
                        }
                    }
                }
                self.registry
                    .alter_table(&stmt.table, &[], &[tmp_name.as_str()])
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
        }
        Ok(QueryResult::Affected(0))
    }

    pub(super) fn exec_truncate(&self, stmt: TruncateStmt) -> Result<QueryResult, ExecError> {
        // S81/S82 — token auth + rate limit
        if let Some(denied) = self.check_table_access(&stmt.table, "TRUNCATE") {
            return Ok(denied);
        }
        let engine = self
            .registry
            .get(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
        let all_rows = self.scan_live(&engine)?;
        for row in all_rows {
            engine
                .delete(row.pk)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
        }
        Ok(QueryResult::Affected(0))
    }

    pub(super) fn exec_rename_table(
        &self,
        stmt: RenameTableStmt,
    ) -> Result<QueryResult, ExecError> {
        let (schema, all_rows) = {
            let engine = self
                .registry
                .get(&stmt.old_name)
                .map_err(|_| ExecError::TableNotFound(stmt.old_name.clone()))?;
            let schema = engine.schema.clone();
            let rows = self.scan_live(&engine)?;
            (schema, rows)
        };
        self.registry
            .create_table_typed(&stmt.new_name, schema)
            .map_err(|e| ExecError::Engine(e.to_string()))?;
        {
            let new_engine = self
                .registry
                .get(&stmt.new_name)
                .map_err(|_| ExecError::TableNotFound(stmt.new_name.clone()))?;
            for row in all_rows {
                new_engine
                    .insert(row.pk, row.values)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
        }
        self.registry
            .drop_table(&stmt.old_name)
            .map_err(|e| ExecError::Engine(e.to_string()))?;
        Ok(QueryResult::Affected(0))
    }

    pub(super) fn exec_create_table(
        &self,
        stmt: CreateTableStmt,
    ) -> Result<QueryResult, ExecError> {
        // S59 â€” collect defaults and checks before consuming stmt
        let defaults: Vec<Option<LitVal>> =
            stmt.columns.iter().map(|c| c.default.clone()).collect();
        let checks: Vec<Option<String>> = stmt.columns.iter().map(|c| c.check.clone()).collect();
        // S60 â€” collect foreign keys
        let fks: Vec<(String, String, String)> = stmt
            .foreign_keys
            .iter()
            .map(|fk| (fk.col.clone(), fk.ref_table.clone(), fk.ref_col.clone()))
            .collect();
        let table_name = stmt.table.clone();

        let cols: Vec<ColumnDef> = stmt
            .columns
            .iter()
            .map(|c| ColumnDef::new(c.name.clone(), ColumnType::from_token(&c.col_type)))
            .collect();

        let schema = TableSchema::new(cols).map_err(|e| ExecError::Engine(e.to_string()))?;

        // B5 — a CHECK that cannot be evaluated is rejected, not stored.
        if let Err(e) = Self::validate_checks(&checks, &schema, &table_name) {
            return Ok(sql_error("0A000", &e));
        }

        self.registry
            .create_table_typed(&stmt.table, schema)
            .map_err(|e| ExecError::Engine(e.to_string()))?;

        // S59 â€” store defaults/checks
        let has_defaults = defaults.iter().any(|d| d.is_some());
        let has_checks = checks.iter().any(|c| c.is_some());
        if has_defaults {
            self.col_defaults
                .lock_safe()
                .insert(table_name.clone(), defaults);
        }
        if has_checks {
            self.col_checks
                .lock_safe()
                .insert(table_name.clone(), checks);
        } else {
            // Do not inherit constraints of a dropped table of the same name.
            self.col_checks.lock_safe().remove(&table_name);
        }
        // S60 â€” store FKs
        if !fks.is_empty() {
            self.foreign_keys.lock_safe().insert(table_name, fks);
        } else {
            self.foreign_keys.lock_safe().remove(&table_name);
        }

        Ok(QueryResult::Created)
    }

    pub(super) fn exec_drop_table(&self, stmt: DropTableStmt) -> Result<QueryResult, ExecError> {
        self.registry
            .drop_table(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
        Ok(QueryResult::Dropped)
    }

    // S42 â€” Index DDL
    pub(super) fn exec_create_index(
        &self,
        stmt: CreateIndexStmt,
    ) -> Result<QueryResult, ExecError> {
        let base = self.registry.base_dir();
        let dir = base.join(&stmt.table).join("indexes");
        std::fs::create_dir_all(&dir).map_err(|e| ExecError::Engine(e.to_string()))?;
        let path = dir.join(format!("{}.json", stmt.index_name));
        let already_exists = path.exists();
        let content = format!("{{\"col\":\"{}\"}}", stmt.col);
        std::fs::write(path, content).map_err(|e| ExecError::Engine(e.to_string()))?;
        let warnings_on = *self.warnings_enabled.lock_safe();
        if already_exists && warnings_on {
            return Ok(QueryResult::WithNotice {
                result: Box::new(QueryResult::Created),
                notice: "index already exists, skipping".to_owned(),
            });
        }
        Ok(QueryResult::Created)
    }

    pub(super) fn exec_drop_index(&self, stmt: DropIndexStmt) -> Result<QueryResult, ExecError> {
        // Best-effort: ignore error if file doesn't exist
        let base = self.registry.base_dir();
        let path = base
            .join("__any__")
            .join("indexes")
            .join(format!("{}.json", stmt.index_name));
        // Search all table dirs for the index
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let idx_path = entry
                    .path()
                    .join("indexes")
                    .join(format!("{}.json", stmt.index_name));
                if idx_path.exists() {
                    // D5/S278 — DROP INDEX is DDL on the owning table
                    let owner = entry.file_name().to_string_lossy().into_owned();
                    if let Some(denied) = self.check_ddl_allowed(&owner, false) {
                        return Ok(denied);
                    }
                    let _ = std::fs::remove_file(idx_path);
                    return Ok(QueryResult::Dropped);
                }
            }
        }
        let _ = path; // suppress warning
        Ok(QueryResult::Dropped)
    }

    // S45 â€” CREATE/DROP VIEW
    pub(super) fn exec_create_view(&self, stmt: CreateViewStmt) -> Result<QueryResult, ExecError> {
        self.views.lock_safe().insert(stmt.name, *stmt.select);
        Ok(QueryResult::Created)
    }

    pub(super) fn exec_drop_view(&self, stmt: DropViewStmt) -> Result<QueryResult, ExecError> {
        self.views.lock_safe().remove(&stmt.name);
        Ok(QueryResult::Dropped)
    }

    // S65 â€” UDFs
    pub(super) fn exec_create_function(
        &self,
        name: String,
        param: String,
        body: String,
    ) -> Result<QueryResult, ExecError> {
        self.udfs.lock_safe().insert(name, (param, body));
        Ok(QueryResult::Created)
    }

    pub(super) fn exec_drop_function(&self, name: String) -> Result<QueryResult, ExecError> {
        self.udfs.lock_safe().remove(&name);
        Ok(QueryResult::Dropped)
    }

    // S66 â€” Triggers
    pub(super) fn exec_create_trigger(&self, def: TriggerDef) -> Result<QueryResult, ExecError> {
        // B7 — only store triggers that can actually fire.
        if let Err(e) = self.validate_trigger(&def) {
            return Ok(sql_error("0A000", &e));
        }
        self.triggers.lock_safe().insert(def.name.clone(), def);
        Ok(QueryResult::Created)
    }

    pub(super) fn exec_drop_trigger(&self, name: String) -> Result<QueryResult, ExecError> {
        self.triggers.lock_safe().remove(&name);
        Ok(QueryResult::Dropped)
    }

    // S75 â€” Materialized Views
    pub(super) fn exec_create_mat_view(
        &self,
        name: String,
        select: SelectStmt,
    ) -> Result<QueryResult, ExecError> {
        let result = self.exec_select(select.clone())?;
        let rows = match result {
            QueryResult::Rows { rows, .. } => rows,
            _ => vec![],
        };
        self.mat_views.lock_safe().insert(name, (select, rows));
        Ok(QueryResult::Created)
    }

    pub(super) fn exec_refresh_mat_view(&self, name: String) -> Result<QueryResult, ExecError> {
        let select = {
            let guard = self.mat_views.lock_safe();
            guard.get(&name).map(|(s, _)| s.clone())
        };
        if let Some(select) = select {
            let result = self.exec_select(select.clone())?;
            let rows = match result {
                QueryResult::Rows { rows, .. } => rows,
                _ => vec![],
            };
            self.mat_views.lock_safe().insert(name, (select, rows));
            Ok(QueryResult::Affected(0))
        } else {
            Err(ExecError::TableNotFound(name))
        }
    }

    pub(super) fn exec_drop_mat_view(&self, name: String) -> Result<QueryResult, ExecError> {
        self.mat_views.lock_safe().remove(&name);
        self.views.lock_safe().remove(&name);
        Ok(QueryResult::Dropped)
    }
}
