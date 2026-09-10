use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use thiserror::Error;
use vectorvolt_engine::{
    ColumnDef, ColumnType, EngineRow, TableRegistry, TableSchema, TxnId, TypedValue,
};

use crate::sql::ast::*;
use crate::sql::fts::TextStore;
use crate::sql::hnsw::HnswIndex;
use crate::sql::lexer::{LexError, Lexer};
use crate::sql::parser::{ParseError, SqlParser};
use crate::sql::planner::{PhysicalPlan, Planner};
use crate::sql::vec_store::{DistanceMetric, VectorStore, parse_vec_literal};

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("lex error: {0}")]
    Lex(#[from] LexError),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("table not found: {0}")]
    TableNotFound(String),
    #[error("column not found: {0}")]
    ColumnNotFound(String),
    #[error("type error: {0}")]
    TypeError(String),
    #[error("engine error: {0}")]
    Engine(String),
    #[error("wrong column count: expected {expected}, got {got}")]
    ColumnCountMismatch { expected: usize, got: usize },
    #[error("no active transaction")]
    NoActiveTxn,
    #[error("transaction already active")]
    TxnAlreadyActive,
}

#[derive(Debug, Clone)]
pub enum QueryResult {
    Rows {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Affected(u64),
    Created,
    Dropped,
    ExplainPlan(Vec<String>),
    TxBegin,
    TxCommit,
    TxRollback,
}

/// Per-table open transaction id, keyed by table name.
type TxnMap = HashMap<String, TxnId>;

pub struct SqlExecutor {
    registry: Arc<TableRegistry>,
    /// Active transaction state. `None` = auto-commit mode.
    txn: Mutex<Option<TxnMap>>,
}

impl SqlExecutor {
    pub fn new(registry: Arc<TableRegistry>) -> Self {
        Self {
            registry,
            txn: Mutex::new(None),
        }
    }

    pub fn execute(&self, sql: &str) -> Result<QueryResult, ExecError> {
        let tokens = Lexer::new(sql).tokenize()?;
        let stmt = SqlParser::new(tokens).parse()?;
        self.execute_stmt(stmt)
    }

    fn execute_stmt(&self, stmt: Stmt) -> Result<QueryResult, ExecError> {
        match stmt {
            Stmt::Select(s) => self.exec_select(s),
            Stmt::Insert(s) => self.exec_insert(s),
            Stmt::InsertSelect(s) => self.exec_insert_select(s),
            Stmt::Update(s) => self.exec_update(s),
            Stmt::Delete(s) => self.exec_delete(s),
            Stmt::CreateTable(s) => self.exec_create_table(s),
            Stmt::DropTable(s) => self.exec_drop_table(s),
            Stmt::AlterTable(s) => self.exec_alter_table(s),
            Stmt::Truncate(s) => self.exec_truncate(s),
            Stmt::RenameTable(s) => self.exec_rename_table(s),
            Stmt::Explain(s) => self.explain_select(*s),
            Stmt::Begin => self.exec_begin(),
            Stmt::Commit => self.exec_commit(),
            Stmt::Rollback => self.exec_rollback(),
        }
    }

    fn exec_begin(&self) -> Result<QueryResult, ExecError> {
        let mut guard = self.txn.lock().unwrap();
        if guard.is_some() {
            return Err(ExecError::TxnAlreadyActive);
        }
        *guard = Some(HashMap::new());
        Ok(QueryResult::TxBegin)
    }

    fn exec_commit(&self) -> Result<QueryResult, ExecError> {
        let mut guard = self.txn.lock().unwrap();
        let txn_map = guard.take().ok_or(ExecError::NoActiveTxn)?;
        for (table, txn_id) in txn_map {
            let engine = self
                .registry
                .get(&table)
                .map_err(|_| ExecError::TableNotFound(table.clone()))?;
            engine
                .commit_txn(txn_id)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
        }
        Ok(QueryResult::TxCommit)
    }

    fn exec_rollback(&self) -> Result<QueryResult, ExecError> {
        let mut guard = self.txn.lock().unwrap();
        let txn_map = guard.take().ok_or(ExecError::NoActiveTxn)?;
        for (table, txn_id) in txn_map {
            if let Ok(engine) = self.registry.get(&table) {
                let _ = engine.rollback_txn(txn_id);
            }
        }
        Ok(QueryResult::TxRollback)
    }

    /// Return the active txn_id for `table`, creating one via `engine.begin()` if needed.
    /// Returns `None` when not in a transaction (auto-commit mode).
    fn txn_id_for(&self, table: &str) -> Option<TxnId> {
        let mut guard = self.txn.lock().unwrap();
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

    fn exec_select(&self, stmt: SelectStmt) -> Result<QueryResult, ExecError> {
        let engine = self
            .registry
            .get(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;

        let schema = &engine.schema;
        let col_names: Vec<String> = schema.columns().iter().map(|c| c.name.clone()).collect();

        // Use the query planner to choose scan strategy.
        let est_rows: u64 = engine
            .partition_meta()
            .iter()
            .map(|p| p.row_count as u64)
            .sum::<u64>()
            .max(1);
        let planner = Planner::new(schema, est_rows);
        let plan = planner.plan(&stmt);
        let left_rows = self.execute_scan(&plan, &engine)?;
        let filtered_left =
            self.filter_rows(left_rows, stmt.where_clause.as_ref(), schema, &stmt.table)?;

        // ── JOIN path ──────────────────────────────────────────────────────────
        if !stmt.joins.is_empty() {
            return self.exec_join(stmt, filtered_left, schema, &col_names);
        }

        // Collect scalar subqueries from SELECT list for later evaluation.
        // usize::MAX is a sentinel meaning "subquery column — filled separately".
        let subquery_cols: Vec<(String, SelectStmt)> = stmt
            .columns
            .iter()
            .filter_map(|expr| {
                if let SelectExpr::Subquery { subquery, alias } = expr {
                    Some((alias.clone(), *subquery.clone()))
                } else {
                    None
                }
            })
            .collect();

        // Determine output columns (Agg variants handled separately below)
        let out_cols: Vec<(String, usize)> = stmt
            .columns
            .iter()
            .flat_map(|expr| match expr {
                SelectExpr::Star => col_names
                    .iter()
                    .enumerate()
                    .map(|(i, n)| (n.clone(), i))
                    .collect::<Vec<_>>(),
                SelectExpr::Col(name) => {
                    if let Some((idx, _)) = schema.find(name) {
                        vec![(name.clone(), idx)]
                    } else {
                        vec![]
                    }
                }
                SelectExpr::QualifiedCol { col, .. } => {
                    if let Some((idx, _)) = schema.find(col) {
                        vec![(col.clone(), idx)]
                    } else {
                        vec![]
                    }
                }
                SelectExpr::Subquery { alias, .. } => vec![(alias.clone(), usize::MAX)],
                SelectExpr::Agg { .. } => vec![],
            })
            .collect();

        let filtered = filtered_left;

        // ── Aggregate path (COUNT/SUM/MIN/MAX with optional GROUP BY) ─────────
        let has_agg = stmt
            .columns
            .iter()
            .any(|c| matches!(c, SelectExpr::Agg { .. }));
        if has_agg {
            return self.exec_aggregate(filtered, &stmt, schema);
        }

        // Sort by ORDER BY
        let mut sorted = filtered;
        match &stmt.order_by {
            Some(OrderBy::Col {
                name: col_name,
                asc,
            }) => {
                if let Some((idx, col_def)) = schema.find(col_name) {
                    let col_type = col_def.col_type;
                    let asc = *asc;
                    sorted.sort_by(|a, b| {
                        let va = TypedValue::from_bits(
                            col_type,
                            a.values.get(idx).copied().unwrap_or(0),
                        );
                        let vb = TypedValue::from_bits(
                            col_type,
                            b.values.get(idx).copied().unwrap_or(0),
                        );
                        let cmp = va.display_str().cmp(&vb.display_str());
                        if asc { cmp } else { cmp.reverse() }
                    });
                }
            }
            Some(OrderBy::VecDistance { col, query }) => {
                let base = self.registry.base_dir();
                // When LIMIT is set, try the HNSW index for approximate ANN.
                // Fall back to brute-force VectorStore when the index is empty or absent.
                let k = stmt.limit.unwrap_or(u64::MAX) as usize;
                let hnsw_results = HnswIndex::load(base, &stmt.table, col, DistanceMetric::L2)
                    .ok()
                    .map(|idx| idx.search(query, k.max(1)))
                    .filter(|r| !r.is_empty());

                let dist_map: std::collections::HashMap<u64, f32> =
                    if let Some(results) = hnsw_results {
                        results.into_iter().collect()
                    } else {
                        VectorStore::load(base, &stmt.table, col)
                            .map(|store| {
                                store
                                    .ranked(query, DistanceMetric::L2)
                                    .into_iter()
                                    .collect()
                            })
                            .unwrap_or_default()
                    };
                sorted.sort_by(|a, b| {
                    let da = dist_map.get(&a.pk).copied().unwrap_or(f32::MAX);
                    let db = dist_map.get(&b.pk).copied().unwrap_or(f32::MAX);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            None => {}
        }

        // Apply LIMIT
        if let Some(lim) = stmt.limit {
            sorted.truncate(lim as usize);
        }

        // Deduplicate if DISTINCT
        let final_rows = if stmt.distinct {
            let mut seen = std::collections::HashSet::new();
            sorted.into_iter().filter(|r| seen.insert(r.pk)).collect()
        } else {
            sorted
        };

        // Pre-evaluate scalar subqueries (each returns a single value shared across all rows).
        let subquery_values: Vec<(String, String)> = subquery_cols
            .into_iter()
            .map(|(alias, sq)| {
                let val = self
                    .exec_select(sq)
                    .ok()
                    .and_then(|r| match r {
                        QueryResult::Rows { rows, .. } => {
                            rows.into_iter().next().and_then(|r| r.into_iter().next())
                        }
                        _ => None,
                    })
                    .unwrap_or_default();
                (alias, val)
            })
            .collect();

        // Build output
        let out_col_names: Vec<String> = out_cols.iter().map(|(n, _)| n.clone()).collect();
        let rows: Vec<Vec<String>> = final_rows
            .iter()
            .map(|row| {
                out_cols
                    .iter()
                    .map(|(col_name, idx)| {
                        if *idx == usize::MAX {
                            // Subquery column — look up pre-evaluated value by alias.
                            subquery_values
                                .iter()
                                .find(|(a, _)| a == col_name)
                                .map(|(_, v)| v.clone())
                                .unwrap_or_default()
                        } else {
                            let bits = row.values.get(*idx).copied().unwrap_or(0);
                            let col_type = schema
                                .get(*idx)
                                .map(|c| c.col_type)
                                .unwrap_or(ColumnType::U64);
                            TypedValue::from_bits(col_type, bits).display_str()
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(QueryResult::Rows {
            columns: out_col_names,
            rows,
        })
    }

    /// Evaluate aggregate SELECT expressions (COUNT/SUM/MIN/MAX) with optional GROUP BY.
    fn exec_aggregate(
        &self,
        rows: Vec<vectorvolt_engine::EngineRow>,
        stmt: &SelectStmt,
        schema: &vectorvolt_engine::TableSchema,
    ) -> Result<QueryResult, ExecError> {
        use std::collections::BTreeMap;

        let group_col_idx: Option<usize> = stmt
            .group_by
            .as_ref()
            .and_then(|col| schema.find(col).map(|(i, _)| i));

        let mut groups: BTreeMap<String, Vec<&vectorvolt_engine::EngineRow>> = BTreeMap::new();
        for row in &rows {
            let key = match group_col_idx {
                Some(idx) => row
                    .values
                    .get(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                None => String::new(),
            };
            groups.entry(key).or_default().push(row);
        }

        let mut col_headers: Vec<String> = Vec::new();
        if let Some(gcol) = &stmt.group_by {
            col_headers.push(gcol.clone());
        }
        for expr in &stmt.columns {
            if let SelectExpr::Agg { func, col } = expr {
                col_headers.push(match func {
                    AggFunc::Count => "count".to_owned(),
                    AggFunc::Sum => format!("sum({})", col.as_deref().unwrap_or("?")),
                    AggFunc::Min => format!("min({})", col.as_deref().unwrap_or("?")),
                    AggFunc::Max => format!("max({})", col.as_deref().unwrap_or("?")),
                });
            }
        }

        let mut out_rows: Vec<Vec<String>> = Vec::new();
        for (group_key, group_rows) in &groups {
            let mut out_row: Vec<String> = Vec::new();
            if stmt.group_by.is_some() {
                out_row.push(group_key.clone());
            }
            for expr in &stmt.columns {
                if let SelectExpr::Agg { func, col } = expr {
                    let col_idx = col.as_ref().and_then(|c| schema.find(c).map(|(i, _)| i));
                    let val_str = match func {
                        AggFunc::Count => group_rows.len().to_string(),
                        AggFunc::Sum => {
                            let s: u64 = group_rows
                                .iter()
                                .map(|r| {
                                    col_idx.and_then(|i| r.values.get(i)).copied().unwrap_or(0)
                                })
                                .sum();
                            s.to_string()
                        }
                        AggFunc::Min => {
                            let m = group_rows
                                .iter()
                                .map(|r| {
                                    col_idx
                                        .and_then(|i| r.values.get(i))
                                        .copied()
                                        .unwrap_or(u64::MAX)
                                })
                                .min();
                            m.map(|v| v.to_string()).unwrap_or_default()
                        }
                        AggFunc::Max => {
                            let m = group_rows
                                .iter()
                                .map(|r| {
                                    col_idx.and_then(|i| r.values.get(i)).copied().unwrap_or(0)
                                })
                                .max();
                            m.map(|v| v.to_string()).unwrap_or_default()
                        }
                    };
                    out_row.push(val_str);
                }
            }
            out_rows.push(out_row);
        }

        if let Some(having) = &stmt.having {
            out_rows.retain(|row| {
                let row_map: std::collections::HashMap<String, String> = col_headers
                    .iter()
                    .cloned()
                    .zip(row.iter().cloned())
                    .collect();
                Self::eval_having(having, &row_map)
            });
        }

        Ok(QueryResult::Rows {
            columns: col_headers,
            rows: out_rows,
        })
    }

    fn eval_having(expr: &Expr, row: &std::collections::HashMap<String, String>) -> bool {
        match expr {
            Expr::BinOp { op, left, right } => match op {
                BinOp::And => Self::eval_having(left, row) && Self::eval_having(right, row),
                BinOp::Or => Self::eval_having(left, row) || Self::eval_having(right, row),
                _ => {
                    let lv = Self::having_val(left, row);
                    let rv = Self::having_val(right, row);
                    compare_values(&lv, op, &rv)
                }
            },
            _ => false,
        }
    }

    fn having_val(expr: &Expr, row: &std::collections::HashMap<String, String>) -> String {
        match expr {
            Expr::Col(name) => row.get(name).cloned().unwrap_or_default(),
            Expr::Lit(lit) => match lit {
                LitVal::Int(n) => n.to_string(),
                LitVal::Float(f) => format!("{f:.6}"),
                LitVal::Str(s) => s.clone(),
                LitVal::Bool(b) => b.to_string(),
            },
            _ => String::new(),
        }
    }

    /// Hash-join left rows against one or more right tables.
    /// Produces a flat string-row result (`QueryResult::Rows`).
    fn exec_join(
        &self,
        stmt: SelectStmt,
        left_rows: Vec<EngineRow>,
        left_schema: &TableSchema,
        left_col_names: &[String],
    ) -> Result<QueryResult, ExecError> {
        // Build combined rows: each element is (left_values, right_values_per_join_table)
        // We work in display-string space for simplicity.

        // Collect left display rows
        let left_display: Vec<Vec<String>> = left_rows
            .iter()
            .map(|row| {
                left_col_names
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let col_type = left_schema
                            .get(i)
                            .map(|c| c.col_type)
                            .unwrap_or(ColumnType::U64);
                        TypedValue::from_bits(col_type, row.values.get(i).copied().unwrap_or(0))
                            .display_str()
                    })
                    .collect()
            })
            .collect();

        // For each JOIN, load the right table and build a HashMap<join_key → Vec<display_row>>
        type RightMap = HashMap<String, Vec<Vec<String>>>;
        let mut right_tables: Vec<(Vec<String>, RightMap)> = Vec::new();
        for join in &stmt.joins {
            let right_engine = self
                .registry
                .get(&join.table)
                .map_err(|_| ExecError::TableNotFound(join.table.clone()))?;
            let right_schema = &right_engine.schema;
            let right_col_names: Vec<String> = right_schema
                .columns()
                .iter()
                .map(|c| c.name.clone())
                .collect();
            let right_rows = self.scan_live(&right_engine)?;
            let right_col_count = right_schema.col_count();

            // Find join key index in right table
            let right_key_idx = right_schema
                .find(&join.right_col)
                .map(|(i, _)| i)
                .unwrap_or(0);

            let mut map: HashMap<String, Vec<Vec<String>>> = HashMap::new();
            for row in &right_rows {
                let right_col_type = right_schema
                    .get(right_key_idx)
                    .map(|c| c.col_type)
                    .unwrap_or(ColumnType::U64);
                let key = TypedValue::from_bits(
                    right_col_type,
                    row.values.get(right_key_idx).copied().unwrap_or(0),
                )
                .display_str();
                let display: Vec<String> = (0..right_col_count)
                    .map(|i| {
                        let ct = right_schema
                            .get(i)
                            .map(|c| c.col_type)
                            .unwrap_or(ColumnType::U64);
                        TypedValue::from_bits(ct, row.values.get(i).copied().unwrap_or(0))
                            .display_str()
                    })
                    .collect();
                map.entry(key).or_default().push(display);
            }
            right_tables.push((right_col_names, map));
        }

        // Find left key index for each join
        let left_key_indices: Vec<usize> = stmt
            .joins
            .iter()
            .map(|j| left_schema.find(&j.left_col).map(|(i, _)| i).unwrap_or(0))
            .collect();

        // Produce combined rows (nested-loop over joins)
        let mut combined: Vec<Vec<String>> = left_display;
        for (join_idx, (join, (right_col_names, right_map))) in
            stmt.joins.iter().zip(right_tables.iter()).enumerate()
        {
            let left_key_idx = left_key_indices[join_idx];
            let right_width = right_col_names.len();
            let mut next: Vec<Vec<String>> = Vec::new();
            for left_row in &combined {
                let key = left_row.get(left_key_idx).cloned().unwrap_or_default();
                if let Some(right_matches) = right_map.get(&key) {
                    for right_row in right_matches {
                        let mut merged = left_row.clone();
                        merged.extend(right_row.clone());
                        next.push(merged);
                    }
                } else if join.kind == JoinKind::Left {
                    let mut merged = left_row.clone();
                    merged.extend(vec![String::new(); right_width]);
                    next.push(merged);
                }
                // INNER JOIN: drop unmatched left rows (do nothing)
            }
            combined = next;
        }

        // Build column headers
        let mut all_col_names: Vec<String> = left_col_names.to_vec();
        for (join, (right_col_names, _)) in stmt.joins.iter().zip(right_tables.iter()) {
            for col in right_col_names {
                all_col_names.push(format!("{}.{}", join.table, col));
            }
        }

        // Project SELECT columns
        let (out_headers, out_rows) = if stmt.columns.iter().any(|c| matches!(c, SelectExpr::Star))
        {
            (all_col_names, combined)
        } else {
            let mut headers = Vec::new();
            let mut rows: Vec<Vec<String>> = combined.iter().map(|_| Vec::new()).collect();
            for col_expr in &stmt.columns {
                let col_name = match col_expr {
                    SelectExpr::Col(c) | SelectExpr::QualifiedCol { col: c, .. } => c.clone(),
                    _ => continue,
                };
                let col_idx = all_col_names
                    .iter()
                    .position(|n| n == &col_name || n.ends_with(&format!(".{col_name}")));
                if let Some(idx) = col_idx {
                    headers.push(col_name);
                    for (i, row) in combined.iter().enumerate() {
                        rows[i].push(row.get(idx).cloned().unwrap_or_default());
                    }
                }
            }
            (headers, rows)
        };

        // Apply LIMIT
        let mut out_rows = out_rows;
        if let Some(lim) = stmt.limit {
            out_rows.truncate(lim as usize);
        }

        Ok(QueryResult::Rows {
            columns: out_headers,
            rows: out_rows,
        })
    }

    fn exec_insert_select(&self, stmt: InsertSelectStmt) -> Result<QueryResult, ExecError> {
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
            let pk = vals[0];
            engine
                .insert(pk, vals)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
            count += 1;
        }
        Ok(QueryResult::Affected(count))
    }

    fn exec_alter_table(&self, stmt: AlterTableStmt) -> Result<QueryResult, ExecError> {
        match stmt.op {
            AlterOp::AddColumn(col_def) => {
                let col = ColumnDef::new(col_def.name, ColumnType::from_token(&col_def.col_type));
                self.registry
                    .alter_table(&stmt.table, &[col], &[])
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
            AlterOp::DropColumn(col_name) => {
                self.registry
                    .alter_table(&stmt.table, &[], &[col_name.as_str()])
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
        }
        Ok(QueryResult::Affected(0))
    }

    fn exec_truncate(&self, stmt: TruncateStmt) -> Result<QueryResult, ExecError> {
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

    fn exec_rename_table(&self, stmt: RenameTableStmt) -> Result<QueryResult, ExecError> {
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

    fn exec_insert(&self, stmt: InsertStmt) -> Result<QueryResult, ExecError> {
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

        for (col_name, expr) in stmt.columns.iter().zip(stmt.values.iter()) {
            let (idx, col_def) = schema
                .find(col_name)
                .ok_or_else(|| ExecError::ColumnNotFound(col_name.clone()))?;
            let bits = self.expr_to_bits(expr, col_def.col_type)?;
            row_vals[idx] = bits;
        }

        let pk = row_vals[0];
        if let Some(txn_id) = self.txn_id_for(&stmt.table) {
            engine
                .insert_txn(txn_id, pk, row_vals)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
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

        Ok(QueryResult::Affected(1))
    }

    fn exec_update(&self, stmt: UpdateStmt) -> Result<QueryResult, ExecError> {
        let engine = self
            .registry
            .get(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
        let schema = &engine.schema;

        let all_rows = self.scan_live(&engine)?;
        let to_update =
            self.filter_rows(all_rows, stmt.where_clause.as_ref(), schema, &stmt.table)?;
        let count = to_update.len() as u64;

        for row in to_update {
            let mut new_vals = row.values.clone();
            for (col_name, expr) in &stmt.assignments {
                let (idx, col_def) = schema
                    .find(col_name)
                    .ok_or_else(|| ExecError::ColumnNotFound(col_name.clone()))?;
                new_vals[idx] = self.expr_to_bits(expr, col_def.col_type)?;
            }
            if let Some(txn_id) = self.txn_id_for(&stmt.table) {
                engine
                    .delete_txn(txn_id, row.pk)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                engine
                    .insert_txn(txn_id, row.pk, new_vals)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            } else {
                engine
                    .update(row.pk, new_vals)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
        }

        Ok(QueryResult::Affected(count))
    }

    fn exec_delete(&self, stmt: DeleteStmt) -> Result<QueryResult, ExecError> {
        let engine = self
            .registry
            .get(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;

        let schema = &engine.schema;

        // Get all live rows then filter
        let all_rows = self.scan_live(&engine)?;

        let to_delete =
            self.filter_rows(all_rows, stmt.where_clause.as_ref(), schema, &stmt.table)?;
        let count = to_delete.len() as u64;

        if let Some(txn_id) = self.txn_id_for(&stmt.table) {
            for row in to_delete {
                engine
                    .delete_txn(txn_id, row.pk)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
        } else {
            for row in to_delete {
                engine
                    .delete(row.pk)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
            }
        }

        Ok(QueryResult::Affected(count))
    }

    fn exec_create_table(&self, stmt: CreateTableStmt) -> Result<QueryResult, ExecError> {
        let cols: Vec<ColumnDef> = stmt
            .columns
            .iter()
            .map(|c| ColumnDef::new(c.name.clone(), ColumnType::from_token(&c.col_type)))
            .collect();

        let schema = TableSchema::new(cols).map_err(|e| ExecError::Engine(e.to_string()))?;

        self.registry
            .create_table_typed(&stmt.table, schema)
            .map_err(|e| ExecError::Engine(e.to_string()))?;

        Ok(QueryResult::Created)
    }

    fn exec_drop_table(&self, stmt: DropTableStmt) -> Result<QueryResult, ExecError> {
        self.registry
            .drop_table(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
        Ok(QueryResult::Dropped)
    }

    /// Try to extract a simple single-column range predicate for pushdown.
    ///
    /// Returns `(col_name, lo_inclusive, hi_inclusive)` when WHERE is:
    /// Build and return an EXPLAIN plan for a SELECT without executing it.
    fn explain_select(&self, stmt: SelectStmt) -> Result<QueryResult, ExecError> {
        let engine = self
            .registry
            .get(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
        let est_rows: u64 = engine
            .partition_meta()
            .iter()
            .map(|p| p.row_count as u64)
            .sum::<u64>()
            .max(1);
        let planner = Planner::new(&engine.schema, est_rows);
        let plan = planner.plan(&stmt);
        Ok(QueryResult::ExplainPlan(plan.explain(0)))
    }

    /// Execute the scan portion of a physical plan (FullScan or RangeScan at root).
    fn execute_scan(
        &self,
        plan: &PhysicalPlan,
        engine: &vectorvolt_engine::TableEngine,
    ) -> Result<Vec<EngineRow>, ExecError> {
        // Walk to the leaf scan node (skip Project/Filter/Sort/Limit wrappers).
        let leaf = Self::leaf_scan(plan);
        match leaf {
            PhysicalPlan::RangeScan {
                col_idx, lo, hi, ..
            } => {
                engine
                    .flush()
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                let raw = engine
                    .scan(*col_idx, *lo, *hi)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                // Re-fetch each PK through get() so updates/deletes in the WAL
                // are reflected, same as scan_live does for FullScan.
                let mut seen = std::collections::HashSet::new();
                let mut live = Vec::with_capacity(raw.len());
                for row in raw {
                    if !seen.insert(row.pk) {
                        continue;
                    }
                    if let Some(current) = engine
                        .get(row.pk)
                        .map_err(|e| ExecError::Engine(e.to_string()))?
                    {
                        live.push(current);
                    }
                }
                Ok(live)
            }
            _ => self.scan_live(engine),
        }
    }

    fn leaf_scan(plan: &PhysicalPlan) -> &PhysicalPlan {
        match plan {
            PhysicalPlan::Filter { input, .. }
            | PhysicalPlan::Project { input, .. }
            | PhysicalPlan::Sort { input, .. }
            | PhysicalPlan::VecSort { input, .. }
            | PhysicalPlan::Limit { input, .. } => Self::leaf_scan(input),
            other => other,
        }
    }

    /// Scan all rows and verify each is still live via `get()`.
    ///
    /// `scan(col=0, 0..MAX)` reads directly from partition files and may return
    /// rows whose tombstones are only in the WAL buffer (i.e. deleted but not yet
    /// compacted).  Validating each PK through `get()` uses the pk_index as the
    /// authoritative source of truth.
    fn scan_live(
        &self,
        engine: &vectorvolt_engine::TableEngine,
    ) -> Result<Vec<EngineRow>, ExecError> {
        // Flush any buffered rows so scan() can see them in partition files.
        engine
            .flush()
            .map_err(|e| ExecError::Engine(e.to_string()))?;
        let raw = engine
            .scan(0, 0, u64::MAX)
            .map_err(|e| ExecError::Engine(e.to_string()))?;
        let mut seen = std::collections::HashSet::new();
        let mut live = Vec::with_capacity(raw.len());
        for row in raw {
            if !seen.insert(row.pk) {
                continue;
            }
            if let Some(current) = engine
                .get(row.pk)
                .map_err(|e| ExecError::Engine(e.to_string()))?
            {
                live.push(current);
            }
        }
        Ok(live)
    }

    fn filter_rows(
        &self,
        rows: Vec<EngineRow>,
        where_clause: Option<&Expr>,
        schema: &TableSchema,
        table: &str,
    ) -> Result<Vec<EngineRow>, ExecError> {
        match where_clause {
            None => Ok(rows),
            Some(expr) => rows
                .into_iter()
                .filter_map(|row| match self.eval_expr(expr, &row, schema, table) {
                    Ok(true) => Some(Ok(row)),
                    Ok(false) => None,
                    Err(e) => Some(Err(e)),
                })
                .collect(),
        }
    }

    fn eval_expr(
        &self,
        expr: &Expr,
        row: &EngineRow,
        schema: &TableSchema,
        table: &str,
    ) -> Result<bool, ExecError> {
        match expr {
            Expr::BinOp { op, left, right } => {
                match op {
                    BinOp::And => Ok(self.eval_expr(left, row, schema, table)?
                        && self.eval_expr(right, row, schema, table)?),
                    BinOp::Or => Ok(self.eval_expr(left, row, schema, table)?
                        || self.eval_expr(right, row, schema, table)?),
                    _ => {
                        // Comparison
                        let lv = self.eval_value(left, row, schema)?;
                        let rv = self.eval_value(right, row, schema)?;
                        Ok(compare_values(&lv, op, &rv))
                    }
                }
            }
            Expr::Match { col, term } => {
                let base = self.registry.base_dir();
                match TextStore::load(base, table, col) {
                    Ok(store) => {
                        let index = store.build_index();
                        let hits = index.search(term);
                        Ok(hits.binary_search(&row.pk).is_ok())
                    }
                    Err(_) => Ok(false),
                }
            }
            Expr::In { col, values } => {
                let (idx, col_def) = schema
                    .find(col)
                    .ok_or_else(|| ExecError::ColumnNotFound(col.clone()))?;
                let bits = row.values.get(idx).copied().unwrap_or(0);
                let cell = TypedValue::from_bits(col_def.col_type, bits).display_str();
                Ok(values.iter().any(|v| {
                    let vs = match v {
                        LitVal::Int(n) => n.to_string(),
                        LitVal::Float(f) => format!("{f:.6}"),
                        LitVal::Str(s) => s.clone(),
                        LitVal::Bool(b) => b.to_string(),
                    };
                    compare_values(&cell, &BinOp::Eq, &vs)
                }))
            }
            Expr::Between { col, lo, hi } => {
                let (idx, col_def) = schema
                    .find(col)
                    .ok_or_else(|| ExecError::ColumnNotFound(col.clone()))?;
                let bits = row.values.get(idx).copied().unwrap_or(0);
                let cell = TypedValue::from_bits(col_def.col_type, bits).display_str();
                let lo_s = match lo {
                    LitVal::Int(n) => n.to_string(),
                    LitVal::Float(f) => format!("{f:.6}"),
                    LitVal::Str(s) => s.clone(),
                    LitVal::Bool(b) => b.to_string(),
                };
                let hi_s = match hi {
                    LitVal::Int(n) => n.to_string(),
                    LitVal::Float(f) => format!("{f:.6}"),
                    LitVal::Str(s) => s.clone(),
                    LitVal::Bool(b) => b.to_string(),
                };
                Ok(compare_values(&cell, &BinOp::Gte, &lo_s)
                    && compare_values(&cell, &BinOp::Lte, &hi_s))
            }
            Expr::InSubquery { col, subquery } => {
                let (idx, col_def) = schema
                    .find(col)
                    .ok_or_else(|| ExecError::ColumnNotFound(col.clone()))?;
                let bits = row.values.get(idx).copied().unwrap_or(0);
                let cell = TypedValue::from_bits(col_def.col_type, bits).display_str();
                let result = self.exec_select(*subquery.clone())?;
                let vals: Vec<String> = match result {
                    QueryResult::Rows { rows, .. } => rows
                        .into_iter()
                        .filter_map(|r| r.into_iter().next())
                        .collect(),
                    _ => vec![],
                };
                Ok(vals.iter().any(|v| compare_values(&cell, &BinOp::Eq, v)))
            }
            Expr::Exists { subquery } => {
                let result = self.exec_select(*subquery.clone())?;
                Ok(match result {
                    QueryResult::Rows { rows, .. } => !rows.is_empty(),
                    _ => false,
                })
            }
            _ => Err(ExecError::TypeError("expected boolean expression".into())),
        }
    }

    fn eval_value(
        &self,
        expr: &Expr,
        row: &EngineRow,
        schema: &TableSchema,
    ) -> Result<String, ExecError> {
        match expr {
            Expr::Col(name) => {
                let (idx, col_def) = schema
                    .find(name)
                    .ok_or_else(|| ExecError::ColumnNotFound(name.clone()))?;
                let bits = row.values.get(idx).copied().unwrap_or(0);
                Ok(TypedValue::from_bits(col_def.col_type, bits).display_str())
            }
            Expr::Lit(lit) => Ok(match lit {
                LitVal::Int(n) => n.to_string(),
                LitVal::Float(f) => format!("{f:.6}"),
                LitVal::Str(s) => s.clone(),
                LitVal::Bool(b) => b.to_string(),
            }),
            Expr::QualifiedCol { col, .. } => {
                let (idx, col_def) = schema
                    .find(col)
                    .ok_or_else(|| ExecError::ColumnNotFound(col.clone()))?;
                let bits = row.values.get(idx).copied().unwrap_or(0);
                Ok(TypedValue::from_bits(col_def.col_type, bits).display_str())
            }
            Expr::BinOp { .. }
            | Expr::Match { .. }
            | Expr::In { .. }
            | Expr::Between { .. }
            | Expr::InSubquery { .. }
            | Expr::Exists { .. } => Err(ExecError::TypeError(
                "expected scalar value expression".into(),
            )),
        }
    }

    fn expr_to_bits(&self, expr: &Expr, col_type: ColumnType) -> Result<u64, ExecError> {
        let s = match expr {
            Expr::Lit(lit) => match lit {
                LitVal::Int(n) => n.to_string(),
                LitVal::Float(f) => format!("{f:.6}"),
                LitVal::Str(s) => s.clone(),
                LitVal::Bool(b) => b.to_string(),
            },
            _ => return Err(ExecError::TypeError("expected literal in INSERT".into())),
        };
        TypedValue::parse(col_type, &s)
            .map(|v| v.to_bits())
            .map_err(|e| ExecError::TypeError(e.to_string()))
    }
}

fn compare_values(left: &str, op: &BinOp, right: &str) -> bool {
    // Try numeric comparison first
    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
        return match op {
            BinOp::Eq => (l - r).abs() < f64::EPSILON,
            BinOp::Neq => (l - r).abs() >= f64::EPSILON,
            BinOp::Lt => l < r,
            BinOp::Lte => l <= r,
            BinOp::Gt => l > r,
            BinOp::Gte => l >= r,
            _ => false,
        };
    }
    // String comparison
    match op {
        BinOp::Eq => left == right,
        BinOp::Neq => left != right,
        BinOp::Lt => left < right,
        BinOp::Lte => left <= right,
        BinOp::Gt => left > right,
        BinOp::Gte => left >= right,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorvolt_engine::EngineOptions;

    fn setup() -> (SqlExecutor, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let registry = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
        (SqlExecutor::new(registry), dir)
    }

    #[test]
    fn test_create_table_ok() {
        let (ex, _dir) = setup();
        let r = ex.execute("CREATE TABLE t (pk U64, val U64)");
        assert!(matches!(r, Ok(QueryResult::Created)));
    }

    #[test]
    fn test_create_table_duplicate_err() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        let r = ex.execute("CREATE TABLE t (pk U64)");
        assert!(r.is_err());
    }

    #[test]
    fn test_drop_table_ok() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        let r = ex.execute("DROP TABLE t");
        assert!(matches!(r, Ok(QueryResult::Dropped)));
    }

    #[test]
    fn test_drop_table_not_found() {
        let (ex, _dir) = setup();
        let r = ex.execute("DROP TABLE nope");
        assert!(r.is_err());
    }

    #[test]
    fn test_insert_and_select_star() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
            .unwrap();
        match ex.execute("SELECT * FROM t").unwrap() {
            QueryResult::Rows { columns: _, rows } => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0][0], "1");
                assert_eq!(rows[0][1], "42");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_select_where_eq() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
            .unwrap();
        match ex.execute("SELECT * FROM t WHERE pk = 1").unwrap() {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
            _ => panic!(),
        }
    }

    #[test]
    fn test_select_limit() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        for i in 1u64..=5 {
            ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
                .unwrap();
        }
        match ex.execute("SELECT * FROM t LIMIT 2").unwrap() {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
            _ => panic!(),
        }
    }

    #[test]
    fn test_delete_where() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
            .unwrap();
        let r = ex.execute("DELETE FROM t WHERE pk = 1").unwrap();
        assert!(matches!(r, QueryResult::Affected(1)));
        match ex.execute("SELECT * FROM t").unwrap() {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
            _ => panic!(),
        }
    }

    #[test]
    fn test_delete_no_where() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
            .unwrap();
        let r = ex.execute("DELETE FROM t").unwrap();
        assert!(matches!(r, QueryResult::Affected(2)));
    }

    #[test]
    fn test_select_table_not_found() {
        let (ex, _dir) = setup();
        let r = ex.execute("SELECT * FROM missing");
        assert!(r.is_err());
    }

    #[test]
    fn test_insert_column_count_mismatch() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        let r = ex.execute("INSERT INTO t (pk) VALUES (1, 2)");
        assert!(r.is_err());
    }

    #[test]
    fn test_select_specific_columns() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 99)")
            .unwrap();
        match ex.execute("SELECT val FROM t").unwrap() {
            QueryResult::Rows { columns, rows } => {
                assert_eq!(columns, vec!["val"]);
                assert_eq!(rows[0][0], "99");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_where_gt() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 5)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 15)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (3, 25)")
            .unwrap();
        match ex.execute("SELECT * FROM t WHERE val > 10").unwrap() {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
            _ => panic!(),
        }
    }

    #[test]
    fn test_where_and() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 5)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 15)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (3, 25)")
            .unwrap();
        match ex
            .execute("SELECT * FROM t WHERE val > 5 AND val < 20")
            .unwrap()
        {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
            _ => panic!(),
        }
    }

    #[test]
    fn test_explain_full_scan() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        match ex.execute("EXPLAIN SELECT * FROM t").unwrap() {
            QueryResult::ExplainPlan(lines) => {
                assert!(
                    lines
                        .iter()
                        .any(|l| l.contains("FullScan") && l.contains("table=t"))
                );
                assert!(lines.iter().any(|l| l.contains("Project")));
            }
            _ => panic!("expected ExplainPlan"),
        }
    }

    #[test]
    fn test_explain_range_scan_pushdown() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        match ex.execute("EXPLAIN SELECT * FROM t WHERE pk = 5").unwrap() {
            QueryResult::ExplainPlan(lines) => {
                let range = lines.iter().find(|l| l.contains("RangeScan"));
                let range = range.expect("expected RangeScan node");
                assert!(range.contains("col=pk"), "got: {range}");
                assert!(range.contains("lo=5"), "got: {range}");
                assert!(range.contains("hi=5"), "got: {range}");
            }
            _ => panic!("expected ExplainPlan"),
        }
    }

    #[test]
    fn test_explain_limit_and_order() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        match ex
            .execute("EXPLAIN SELECT pk FROM t ORDER BY val DESC LIMIT 10")
            .unwrap()
        {
            QueryResult::ExplainPlan(lines) => {
                assert!(
                    lines
                        .iter()
                        .any(|l| l.contains("Sort") && l.contains("DESC"))
                );
                assert!(
                    lines
                        .iter()
                        .any(|l| l.contains("Limit") && l.contains("10"))
                );
            }
            _ => panic!("expected ExplainPlan"),
        }
    }

    #[test]
    fn test_pushdown_range_narrows_results() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        for i in 1u64..=10 {
            ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
                .unwrap();
        }
        // pk >= 5 AND pk <= 7 → expect exactly rows 5,6,7
        match ex
            .execute("SELECT * FROM t WHERE pk >= 5 AND pk <= 7")
            .unwrap()
        {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 3),
            _ => panic!(),
        }
    }

    #[test]
    fn test_begin_commit() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        assert!(matches!(ex.execute("BEGIN").unwrap(), QueryResult::TxBegin));
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
            .unwrap();
        assert!(matches!(
            ex.execute("COMMIT").unwrap(),
            QueryResult::TxCommit
        ));
        match ex.execute("SELECT * FROM t").unwrap() {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
            _ => panic!(),
        }
    }

    #[test]
    fn test_begin_rollback() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("BEGIN").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
            .unwrap();
        assert!(matches!(
            ex.execute("ROLLBACK").unwrap(),
            QueryResult::TxRollback
        ));
        match ex.execute("SELECT * FROM t").unwrap() {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 0),
            _ => panic!(),
        }
    }

    #[test]
    fn test_double_begin_err() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64)").unwrap();
        ex.execute("BEGIN").unwrap();
        assert!(matches!(
            ex.execute("BEGIN"),
            Err(ExecError::TxnAlreadyActive)
        ));
        ex.execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_commit_without_begin_err() {
        let (ex, _dir) = setup();
        assert!(matches!(ex.execute("COMMIT"), Err(ExecError::NoActiveTxn)));
    }

    #[test]
    fn test_abort_alias() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64)").unwrap();
        ex.execute("BEGIN").unwrap();
        ex.execute("INSERT INTO t (pk) VALUES (1)").unwrap();
        assert!(matches!(
            ex.execute("ABORT").unwrap(),
            QueryResult::TxRollback
        ));
        match ex.execute("SELECT * FROM t").unwrap() {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 0),
            _ => panic!(),
        }
    }

    #[test]
    fn test_fts_match_basic() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE docs (pk U64, body TEXT)").unwrap();
        ex.execute("INSERT INTO docs (pk, body) VALUES (1, 'hello world')")
            .unwrap();
        ex.execute("INSERT INTO docs (pk, body) VALUES (2, 'foo bar')")
            .unwrap();
        match ex
            .execute("SELECT * FROM docs WHERE MATCH(body, 'hello')")
            .unwrap()
        {
            QueryResult::Rows { rows, .. } => {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0][0], "1");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_fts_match_no_hit() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE docs (pk U64, body TEXT)").unwrap();
        ex.execute("INSERT INTO docs (pk, body) VALUES (1, 'hello world')")
            .unwrap();
        match ex
            .execute("SELECT * FROM docs WHERE MATCH(body, 'rust')")
            .unwrap()
        {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 0),
            _ => panic!(),
        }
    }

    #[test]
    fn test_fts_match_case_insensitive() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE docs (pk U64, body TEXT)").unwrap();
        ex.execute("INSERT INTO docs (pk, body) VALUES (1, 'Hello World')")
            .unwrap();
        match ex
            .execute("SELECT * FROM docs WHERE MATCH(body, 'hello')")
            .unwrap()
        {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
            _ => panic!(),
        }
    }

    #[test]
    fn test_fts_match_multiple_rows() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE docs (pk U64, body TEXT)").unwrap();
        ex.execute("INSERT INTO docs (pk, body) VALUES (1, 'rust is fast')")
            .unwrap();
        ex.execute("INSERT INTO docs (pk, body) VALUES (2, 'rust and safety')")
            .unwrap();
        ex.execute("INSERT INTO docs (pk, body) VALUES (3, 'python is slow')")
            .unwrap();
        match ex
            .execute("SELECT * FROM docs WHERE MATCH(body, 'rust')")
            .unwrap()
        {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
            _ => panic!(),
        }
    }

    #[test]
    fn test_vec_distance_order_nearest_first() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE vecs (pk U64, emb vector(2))")
            .unwrap();
        // pk=1: [1.0, 0.0], pk=2: [10.0, 0.0], pk=3: [2.0, 0.0]
        ex.execute("INSERT INTO vecs (pk, emb) VALUES (1, '[1.0, 0.0]')")
            .unwrap();
        ex.execute("INSERT INTO vecs (pk, emb) VALUES (2, '[10.0, 0.0]')")
            .unwrap();
        ex.execute("INSERT INTO vecs (pk, emb) VALUES (3, '[2.0, 0.0]')")
            .unwrap();
        // Query [1.5, 0.0]: closest is pk=1 (dist=0.5), then pk=3 (dist=0.5... no, 0.5 vs 0.5)
        // Actually query [0.0, 0.0]: pk=1 dist=1, pk=3 dist=2, pk=2 dist=10
        match ex
            .execute("SELECT pk FROM vecs ORDER BY vec_distance(emb, '[0.0, 0.0]') LIMIT 3")
            .unwrap()
        {
            QueryResult::Rows { rows, .. } => {
                assert_eq!(rows.len(), 3);
                assert_eq!(rows[0][0], "1"); // nearest
                assert_eq!(rows[1][0], "3");
                assert_eq!(rows[2][0], "2"); // farthest
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_vec_distance_limit_k() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE vecs (pk U64, emb vector(2))")
            .unwrap();
        for i in 1u64..=5 {
            ex.execute(&format!(
                "INSERT INTO vecs (pk, emb) VALUES ({i}, '[{}.0, 0.0]')",
                i * 10
            ))
            .unwrap();
        }
        // Query [0,0]: top-2 should be pk=1 (dist=10) and pk=2 (dist=20)
        match ex
            .execute("SELECT pk FROM vecs ORDER BY vec_distance(emb, '[0.0, 0.0]') LIMIT 2")
            .unwrap()
        {
            QueryResult::Rows { rows, .. } => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0][0], "1");
                assert_eq!(rows[1][0], "2");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_vec_no_sidecar_returns_all() {
        // Without vector sidecar file (column is u64, not vector type), ORDER BY
        // vec_distance falls back gracefully — rows returned unsorted with dist=MAX.
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
            .unwrap();
        match ex
            .execute("SELECT pk FROM t ORDER BY vec_distance(val, '[0.0]') LIMIT 10")
            .unwrap()
        {
            QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 2),
            _ => panic!(),
        }
    }

    #[test]
    fn test_vec_distance_parse_vec_literal() {
        use crate::sql::vec_store::parse_vec_literal;
        assert_eq!(
            parse_vec_literal("[1.0, 2.0, 3.0]"),
            Some(vec![1.0, 2.0, 3.0])
        );
        assert_eq!(parse_vec_literal("[0.5]"), Some(vec![0.5]));
        assert!(parse_vec_literal("not a vec").is_none());
    }

    // ── Planner tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_planner_chooses_range_scan_for_eq() {
        use crate::sql::lexer::Lexer;
        use crate::sql::parser::SqlParser;
        use crate::sql::planner::Planner;
        use vectorvolt_engine::{ColumnDef, ColumnType, TableSchema};

        let schema = TableSchema::new(vec![
            ColumnDef::new("pk", ColumnType::U64),
            ColumnDef::new("val", ColumnType::U64),
        ])
        .unwrap();
        let tokens = Lexer::new("SELECT * FROM t WHERE pk = 5")
            .tokenize()
            .unwrap();
        let stmt = match SqlParser::new(tokens).parse().unwrap() {
            crate::sql::ast::Stmt::Select(s) => s,
            _ => panic!(),
        };
        let plan = Planner::new(&schema, 10_000).plan(&stmt);
        assert!(
            plan.explain(0).iter().any(|l| l.contains("RangeScan")),
            "expected RangeScan in plan"
        );
    }

    #[test]
    fn test_planner_chooses_full_scan_no_where() {
        use crate::sql::lexer::Lexer;
        use crate::sql::parser::SqlParser;
        use crate::sql::planner::Planner;
        use vectorvolt_engine::{ColumnDef, ColumnType, TableSchema};

        let schema = TableSchema::new(vec![ColumnDef::new("pk", ColumnType::U64)]).unwrap();
        let tokens = Lexer::new("SELECT * FROM t").tokenize().unwrap();
        let stmt = match SqlParser::new(tokens).parse().unwrap() {
            crate::sql::ast::Stmt::Select(s) => s,
            _ => panic!(),
        };
        let plan = Planner::new(&schema, 500).plan(&stmt);
        let lines = plan.explain(0);
        assert!(lines.iter().any(|l| l.contains("FullScan")));
        assert!(!lines.iter().any(|l| l.contains("RangeScan")));
    }

    #[test]
    fn test_planner_explain_vec_sort_node() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE vecs (pk U64, emb vector(2))")
            .unwrap();
        match ex
            .execute("EXPLAIN SELECT pk FROM vecs ORDER BY vec_distance(emb, '[1.0,0.0]') LIMIT 3")
            .unwrap()
        {
            QueryResult::ExplainPlan(lines) => {
                assert!(lines.iter().any(|l| l.contains("VecSort")));
                assert!(lines.iter().any(|l| l.contains("Limit") && l.contains("3")));
            }
            _ => panic!("expected ExplainPlan"),
        }
    }

    #[test]
    fn test_planner_est_rows_range_less_than_full() {
        use crate::sql::lexer::Lexer;
        use crate::sql::parser::SqlParser;
        use crate::sql::planner::Planner;
        use vectorvolt_engine::{ColumnDef, ColumnType, TableSchema};

        let schema = TableSchema::new(vec![
            ColumnDef::new("pk", ColumnType::U64),
            ColumnDef::new("v", ColumnType::U64),
        ])
        .unwrap();
        let tokens = Lexer::new("SELECT * FROM t WHERE pk = 1")
            .tokenize()
            .unwrap();
        let stmt = match SqlParser::new(tokens).parse().unwrap() {
            crate::sql::ast::Stmt::Select(s) => s,
            _ => panic!(),
        };
        let plan = Planner::new(&schema, 10_000).plan(&stmt);
        assert!(
            plan.est_rows() < 10_000,
            "expected est_rows < 10_000, got {}",
            plan.est_rows()
        );
    }

    #[test]
    fn test_update_basic() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
        let r = ex.execute("UPDATE t SET v = 99 WHERE pk = 1");
        assert!(matches!(r, Ok(QueryResult::Affected(1))));
        let rows = ex.execute("SELECT * FROM t WHERE pk = 1").unwrap();
        if let QueryResult::Rows { rows, .. } = rows {
            assert_eq!(rows[0][1], "99");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_update_no_match_returns_zero() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
        let r = ex.execute("UPDATE t SET v = 99 WHERE pk = 999");
        assert!(matches!(r, Ok(QueryResult::Affected(0))));
    }

    #[test]
    fn test_update_multiple_rows() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 5)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (2, 5)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (3, 9)").unwrap();
        // UPDATE all rows where v = 5 — should update 2 rows
        let r = ex.execute("UPDATE t SET v = 7 WHERE v = 5");
        assert!(matches!(r, Ok(QueryResult::Affected(2))));
    }

    #[test]
    fn test_update_no_where_updates_all() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 1)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (2, 2)").unwrap();
        let r = ex.execute("UPDATE t SET v = 0");
        assert!(matches!(r, Ok(QueryResult::Affected(2))));
    }

    #[test]
    fn test_agg_count_star() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (3, 30)").unwrap();
        let r = ex.execute("SELECT COUNT(*) FROM t").unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert_eq!(columns, vec!["count"]);
            assert_eq!(rows[0][0], "3");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_agg_sum_min_max() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (3, 30)").unwrap();
        let r = ex.execute("SELECT SUM(v), MIN(v), MAX(v) FROM t").unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert_eq!(columns, vec!["sum(v)", "min(v)", "max(v)"]);
            assert_eq!(rows[0][0], "60");
            assert_eq!(rows[0][1], "10");
            assert_eq!(rows[0][2], "30");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_agg_count_with_where() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 5)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (2, 15)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (3, 25)").unwrap();
        let r = ex.execute("SELECT COUNT(*) FROM t WHERE v >= 15").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "2");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_agg_group_by() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, cat U64, v U64)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (1, 1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (2, 1, 20)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (3, 2, 30)")
            .unwrap();
        let r = ex.execute("SELECT COUNT(*) FROM t GROUP BY cat").unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert_eq!(columns, vec!["cat", "count"]);
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s32_alter_add_column() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
        let r = ex.execute("ALTER TABLE t ADD COLUMN extra U64");
        assert!(r.is_ok(), "{r:?}");
        let rows = ex.execute("SELECT * FROM t").unwrap();
        if let QueryResult::Rows { columns, .. } = rows {
            assert!(columns.iter().any(|c| c == "extra"), "extra column missing");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s32_alter_drop_column() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, a U64, b U64)").unwrap();
        ex.execute("INSERT INTO t (pk, a, b) VALUES (1, 1, 2)")
            .unwrap();
        let r = ex.execute("ALTER TABLE t DROP COLUMN b");
        assert!(r.is_ok(), "{r:?}");
        let rows = ex.execute("SELECT * FROM t").unwrap();
        if let QueryResult::Rows { columns, .. } = rows {
            assert!(
                !columns.iter().any(|c| c == "b"),
                "b column should be dropped"
            );
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s32_insert_or_replace() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
        ex.execute("INSERT OR REPLACE INTO t (pk, v) VALUES (1, 99)")
            .unwrap();
        let rows = ex.execute("SELECT * FROM t").unwrap();
        if let QueryResult::Rows { rows, .. } = rows {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][1], "99");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s32_having() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, cat U64, v U64)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (1, 1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (2, 1, 20)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (3, 2, 30)")
            .unwrap();
        let r = ex
            .execute("SELECT COUNT(*) FROM t GROUP BY cat HAVING count > 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][1], "2");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s32_where_in() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        for i in 1u64..=5 {
            ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex.execute("SELECT * FROM t WHERE pk IN (1, 3, 5)").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 3);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s32_where_between() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        for i in 1u64..=10 {
            ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex
            .execute("SELECT * FROM t WHERE v BETWEEN 3 AND 7")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 5);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s32_insert_select() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE src (pk U64, v U64)").unwrap();
        ex.execute("CREATE TABLE dst (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO src (pk, v) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO src (pk, v) VALUES (2, 20)")
            .unwrap();
        let r = ex.execute("INSERT INTO dst SELECT * FROM src").unwrap();
        assert!(matches!(r, QueryResult::Affected(2)));
        let rows = ex.execute("SELECT * FROM dst").unwrap();
        if let QueryResult::Rows { rows, .. } = rows {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s32_truncate() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
        let r = ex.execute("TRUNCATE TABLE t");
        assert!(r.is_ok(), "{r:?}");
        let rows = ex.execute("SELECT * FROM t").unwrap();
        if let QueryResult::Rows { rows, .. } = rows {
            assert_eq!(rows.len(), 0);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s32_rename_table() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE old_t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO old_t (pk, v) VALUES (1, 42)")
            .unwrap();
        let r = ex.execute("RENAME TABLE old_t TO new_t");
        assert!(r.is_ok(), "{r:?}");
        assert!(ex.execute("SELECT * FROM old_t").is_err());
        let rows = ex.execute("SELECT * FROM new_t").unwrap();
        if let QueryResult::Rows { rows, .. } = rows {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][1], "42");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s33_inner_join_basic() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE orders (pk U64, user_id U64, amount U64)")
            .unwrap();
        ex.execute("CREATE TABLE users (pk U64, name U64)").unwrap();
        ex.execute("INSERT INTO orders (pk, user_id, amount) VALUES (1, 10, 100)")
            .unwrap();
        ex.execute("INSERT INTO orders (pk, user_id, amount) VALUES (2, 20, 200)")
            .unwrap();
        ex.execute("INSERT INTO users (pk, name) VALUES (10, 1)")
            .unwrap();
        ex.execute("INSERT INTO users (pk, name) VALUES (20, 2)")
            .unwrap();
        let r = ex
            .execute("SELECT * FROM orders INNER JOIN users ON user_id = pk")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s33_inner_join_no_match() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE a (pk U64, b_id U64)").unwrap();
        ex.execute("CREATE TABLE b (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO a (pk, b_id) VALUES (1, 99)")
            .unwrap();
        ex.execute("INSERT INTO b (pk, v) VALUES (1, 42)").unwrap();
        let r = ex
            .execute("SELECT * FROM a INNER JOIN b ON b_id = pk")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 0);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s33_left_join_keeps_unmatched() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE a (pk U64, b_id U64)").unwrap();
        ex.execute("CREATE TABLE b (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO a (pk, b_id) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO a (pk, b_id) VALUES (2, 99)")
            .unwrap();
        ex.execute("INSERT INTO b (pk, v) VALUES (10, 42)").unwrap();
        let r = ex
            .execute("SELECT * FROM a LEFT JOIN b ON b_id = pk")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s33_join_with_where() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE o (pk U64, uid U64, amt U64)")
            .unwrap();
        ex.execute("CREATE TABLE u (pk U64, active U64)").unwrap();
        ex.execute("INSERT INTO o (pk, uid, amt) VALUES (1, 1, 50)")
            .unwrap();
        ex.execute("INSERT INTO o (pk, uid, amt) VALUES (2, 2, 150)")
            .unwrap();
        ex.execute("INSERT INTO u (pk, active) VALUES (1, 1)")
            .unwrap();
        ex.execute("INSERT INTO u (pk, active) VALUES (2, 1)")
            .unwrap();
        let r = ex
            .execute("SELECT * FROM o INNER JOIN u ON uid = pk WHERE amt > 100")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s33_join_keyword_no_inner() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE a (pk U64, b_id U64)").unwrap();
        ex.execute("CREATE TABLE b (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO a (pk, b_id) VALUES (1, 5)")
            .unwrap();
        ex.execute("INSERT INTO b (pk, v) VALUES (5, 99)").unwrap();
        let r = ex.execute("SELECT * FROM a JOIN b ON b_id = pk").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][1], "5");
        } else {
            panic!("expected rows");
        }
    }

    // ── S34: SQL Subqueries ───────────────────────────────────────────────────

    #[test]
    fn test_s34_in_subquery_basic() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE orders (pk U64, amount U64)")
            .unwrap();
        ex.execute("CREATE TABLE vip (pk U64, order_id U64)")
            .unwrap();
        ex.execute("INSERT INTO orders (pk, amount) VALUES (1, 100)")
            .unwrap();
        ex.execute("INSERT INTO orders (pk, amount) VALUES (2, 200)")
            .unwrap();
        ex.execute("INSERT INTO orders (pk, amount) VALUES (3, 300)")
            .unwrap();
        ex.execute("INSERT INTO vip (pk, order_id) VALUES (10, 1)")
            .unwrap();
        ex.execute("INSERT INTO vip (pk, order_id) VALUES (11, 3)")
            .unwrap();
        let r = ex
            .execute("SELECT pk FROM orders WHERE pk IN (SELECT order_id FROM vip)")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            let pks: Vec<&str> = rows.iter().map(|r| r[0].as_str()).collect();
            assert!(pks.contains(&"1"), "pk 1 should match");
            assert!(pks.contains(&"3"), "pk 3 should match");
            assert!(!pks.contains(&"2"), "pk 2 should not match");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s34_in_subquery_with_where() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE items (pk U64, price U64)")
            .unwrap();
        ex.execute("CREATE TABLE featured (pk U64, item_pk U64, active U64)")
            .unwrap();
        ex.execute("INSERT INTO items (pk, price) VALUES (1, 50)")
            .unwrap();
        ex.execute("INSERT INTO items (pk, price) VALUES (2, 80)")
            .unwrap();
        ex.execute("INSERT INTO items (pk, price) VALUES (3, 120)")
            .unwrap();
        ex.execute("INSERT INTO featured (pk, item_pk, active) VALUES (1, 2, 1)")
            .unwrap();
        ex.execute("INSERT INTO featured (pk, item_pk, active) VALUES (2, 3, 0)")
            .unwrap();
        let r = ex
            .execute(
                "SELECT pk FROM items WHERE pk IN (SELECT item_pk FROM featured WHERE active = 1)",
            )
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "2");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s34_exists_true() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE users (pk U64, role U64)").unwrap();
        ex.execute("CREATE TABLE admins (pk U64, user_id U64)")
            .unwrap();
        ex.execute("INSERT INTO users (pk, role) VALUES (1, 1)")
            .unwrap();
        ex.execute("INSERT INTO admins (pk, user_id) VALUES (1, 1)")
            .unwrap();
        let r = ex
            .execute("SELECT pk FROM users WHERE EXISTS (SELECT pk FROM admins WHERE user_id = 1)")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s34_exists_false() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE users (pk U64, role U64)").unwrap();
        ex.execute("CREATE TABLE admins (pk U64, user_id U64)")
            .unwrap();
        ex.execute("INSERT INTO users (pk, role) VALUES (1, 1)")
            .unwrap();
        // No admin rows — EXISTS should return false, filtering out all users
        let r = ex
            .execute("SELECT pk FROM users WHERE EXISTS (SELECT pk FROM admins WHERE user_id = 1)")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 0);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s34_scalar_subquery_in_select() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE products (pk U64, price U64)")
            .unwrap();
        ex.execute("CREATE TABLE counts (pk U64, total U64)")
            .unwrap();
        ex.execute("INSERT INTO products (pk, price) VALUES (1, 42)")
            .unwrap();
        ex.execute("INSERT INTO counts (pk, total) VALUES (1, 99)")
            .unwrap();
        let r = ex
            .execute("SELECT pk, (SELECT total FROM counts WHERE pk = 1) AS cnt FROM products")
            .unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"cnt".to_owned()));
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][1], "99");
        } else {
            panic!("expected rows");
        }
    }
}
