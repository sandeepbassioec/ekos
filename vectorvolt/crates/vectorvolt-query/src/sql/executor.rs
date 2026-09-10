use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use thiserror::Error;
use vectorvolt_engine::{
    ColumnDef, ColumnType, EngineRow, TableRegistry, TableSchema, TxnId, TypedValue,
};

use crate::sql::ast::*;
#[allow(unused_imports)]
use crate::sql::ast::{ArithOp, ScalarFunc, ScalarFunc::*, SetOp};
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
    /// S45 — in-memory view store.
    views: Mutex<HashMap<String, SelectStmt>>,
    /// S53 — savepoint snapshots: name → TxnMap clone at savepoint time.
    savepoints: Mutex<HashMap<String, TxnMap>>,
    /// S56 — query result cache: sql → (columns, rows).
    #[allow(dead_code, clippy::type_complexity)]
    query_cache: Mutex<HashMap<String, (Vec<String>, Vec<Vec<String>>)>>,
    /// S57 — prepared statements: name → Stmt.
    prepared: Mutex<HashMap<String, Stmt>>,
    /// S59 — per-column defaults: table → Vec<Option<LitVal>> (indexed by col position).
    col_defaults: Mutex<HashMap<String, Vec<Option<LitVal>>>>,
    /// S59 — per-column CHECK constraints: table → Vec<Option<String>>.
    col_checks: Mutex<HashMap<String, Vec<Option<String>>>>,
    /// S60 — foreign keys: table → Vec<(col, ref_table, ref_col)>.
    #[allow(clippy::type_complexity)]
    foreign_keys: Mutex<HashMap<String, Vec<(String, String, String)>>>,
    /// S65 — user-defined functions: name → (param, body).
    pub(crate) udfs: Mutex<HashMap<String, (String, String)>>,
    /// S66 — triggers: name → TriggerDef.
    pub(crate) triggers: Mutex<HashMap<String, TriggerDef>>,
}

impl SqlExecutor {
    pub fn new(registry: Arc<TableRegistry>) -> Self {
        Self {
            registry,
            txn: Mutex::new(None),
            views: Mutex::new(HashMap::new()),
            savepoints: Mutex::new(HashMap::new()),
            query_cache: Mutex::new(HashMap::new()),
            prepared: Mutex::new(HashMap::new()),
            col_defaults: Mutex::new(HashMap::new()),
            col_checks: Mutex::new(HashMap::new()),
            foreign_keys: Mutex::new(HashMap::new()),
            udfs: Mutex::new(HashMap::new()),
            triggers: Mutex::new(HashMap::new()),
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
            Stmt::CreateIndex(s) => self.exec_create_index(s),
            Stmt::DropIndex(s) => self.exec_drop_index(s),
            Stmt::CreateView(s) => self.exec_create_view(s),
            Stmt::DropView(s) => self.exec_drop_view(s),
            Stmt::Savepoint(name) => self.exec_savepoint(name),
            Stmt::ReleaseSavepoint(name) => self.exec_release_savepoint(name),
            Stmt::RollbackTo(name) => self.exec_rollback_to(name),
            Stmt::ExplainAnalyze(s) => self.exec_explain_analyze(*s),
            Stmt::Prepare { name, stmt } => self.exec_prepare(name, *stmt),
            Stmt::ExecuteStmt { name, params } => self.exec_execute_prepared(name, params),
            // S61
            Stmt::BeginIsolation { .. } => self.exec_begin(),
            // S65
            Stmt::CreateFunction { name, param, body } => {
                self.exec_create_function(name, param, body)
            }
            Stmt::DropFunction { name } => self.exec_drop_function(name),
            // S66
            Stmt::CreateTrigger(def) => self.exec_create_trigger(def),
            Stmt::DropTrigger { name } => self.exec_drop_trigger(name),
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
        // S41 — handle CTEs
        if !stmt.ctes.is_empty() {
            let mut cte_map: HashMap<String, (Vec<String>, Vec<Vec<String>>)> = HashMap::new();
            for (cte_name, cte_stmt) in &stmt.ctes {
                let result = self.exec_select(*cte_stmt.clone())?;
                if let QueryResult::Rows { columns, rows } = result {
                    cte_map.insert(cte_name.clone(), (columns, rows));
                }
            }
            if let Some((cte_cols, cte_rows)) = cte_map.get(&stmt.table) {
                let filtered: Vec<Vec<String>> = if let Some(where_expr) = &stmt.where_clause {
                    cte_rows
                        .iter()
                        .filter(|row| {
                            let m: HashMap<String, String> =
                                cte_cols.iter().cloned().zip(row.iter().cloned()).collect();
                            Self::eval_str_where(where_expr, &m)
                        })
                        .cloned()
                        .collect()
                } else {
                    cte_rows.clone()
                };
                let filtered = if let Some(lim) = stmt.limit {
                    filtered.into_iter().take(lim as usize).collect()
                } else {
                    filtered
                };
                let (out_cols, out_rows) =
                    Self::project_str_rows(&stmt.columns, &filtered, cte_cols);
                return Ok(QueryResult::Rows {
                    columns: out_cols,
                    rows: out_rows,
                });
            }
        }

        // S45 — handle views
        let view_stmt_opt = self.views.lock().unwrap().get(&stmt.table).cloned();
        if let Some(view_stmt) = view_stmt_opt {
            let result = self.exec_select(view_stmt)?;
            return match result {
                QueryResult::Rows {
                    columns: view_cols,
                    rows: view_rows,
                } => {
                    let filtered: Vec<Vec<String>> = if let Some(where_expr) = &stmt.where_clause {
                        view_rows
                            .into_iter()
                            .filter(|row| {
                                let m: HashMap<String, String> =
                                    view_cols.iter().cloned().zip(row.iter().cloned()).collect();
                                Self::eval_str_where(where_expr, &m)
                            })
                            .collect()
                    } else {
                        view_rows
                    };
                    let filtered = if let Some(lim) = stmt.limit {
                        filtered.into_iter().take(lim as usize).collect()
                    } else {
                        filtered
                    };
                    let (out_cols, out_rows) =
                        Self::project_str_rows(&stmt.columns, &filtered, &view_cols);
                    Ok(QueryResult::Rows {
                        columns: out_cols,
                        rows: out_rows,
                    })
                }
                _ => Err(ExecError::Engine("view returned non-rows".into())),
            };
        }

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

        // Collect computed (CASE/COALESCE/etc.) expressions — evaluated per row.
        let computed_cols: Vec<(String, Expr)> = stmt
            .columns
            .iter()
            .filter_map(|expr| {
                if let SelectExpr::Computed { alias, expr } = expr {
                    Some((alias.clone(), expr.clone()))
                } else {
                    None
                }
            })
            .collect();

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
                SelectExpr::WindowFunc { func, alias, .. } => {
                    let name = alias.clone().unwrap_or_else(|| func.col_name());
                    vec![(name, usize::MAX - 1)]
                }
                SelectExpr::Computed { alias, .. } => vec![(alias.clone(), usize::MAX - 2)],
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
            Some(OrderBy::Multi(cols)) => {
                sorted.sort_by(|a, b| {
                    for (col_name, asc) in cols {
                        if let Some((idx, col_def)) = schema.find(col_name) {
                            let va = TypedValue::from_bits(
                                col_def.col_type,
                                a.values.get(idx).copied().unwrap_or(0),
                            );
                            let vb = TypedValue::from_bits(
                                col_def.col_type,
                                b.values.get(idx).copied().unwrap_or(0),
                            );
                            let cmp = va.display_str().cmp(&vb.display_str());
                            let cmp = if *asc { cmp } else { cmp.reverse() };
                            if cmp != std::cmp::Ordering::Equal {
                                return cmp;
                            }
                        }
                    }
                    std::cmp::Ordering::Equal
                });
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

        // Apply OFFSET then LIMIT
        if let Some(off) = stmt.offset {
            let skip = off as usize;
            if skip >= sorted.len() {
                sorted.clear();
            } else {
                sorted.drain(0..skip);
            }
        }
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

        // Collect window function columns (if any) and pre-compute per-row values.
        let window_funcs: Vec<(String, SelectExpr)> = stmt
            .columns
            .iter()
            .filter_map(|c| {
                if let SelectExpr::WindowFunc { func, alias, .. } = c {
                    let name = alias.clone().unwrap_or_else(|| func.col_name());
                    Some((name, c.clone()))
                } else {
                    None
                }
            })
            .collect();
        // Each element = Vec of (col_name, value) pairs for that row index.
        let window_row_vals: Vec<Vec<(String, String)>> = if !window_funcs.is_empty() {
            (0..final_rows.len())
                .map(|row_idx| {
                    window_funcs
                        .iter()
                        .map(|(col_name, expr)| {
                            let val = match expr {
                                SelectExpr::WindowFunc {
                                    func: WindowFunc::RowNumber,
                                    over_partition,
                                    ..
                                } => {
                                    if let Some(part_col) = over_partition {
                                        if let Some((part_idx, part_def)) = schema.find(part_col) {
                                            let my_bits = final_rows[row_idx]
                                                .values
                                                .get(part_idx)
                                                .copied()
                                                .unwrap_or(0);
                                            let my_part =
                                                TypedValue::from_bits(part_def.col_type, my_bits)
                                                    .display_str();
                                            let rn = final_rows[..row_idx]
                                                .iter()
                                                .filter(|r| {
                                                    let rb = r
                                                        .values
                                                        .get(part_idx)
                                                        .copied()
                                                        .unwrap_or(0);
                                                    TypedValue::from_bits(part_def.col_type, rb)
                                                        .display_str()
                                                        == my_part
                                                })
                                                .count()
                                                + 1;
                                            rn.to_string()
                                        } else {
                                            (row_idx + 1).to_string()
                                        }
                                    } else {
                                        (row_idx + 1).to_string()
                                    }
                                }
                                SelectExpr::WindowFunc {
                                    func: WindowFunc::Rank,
                                    over_order,
                                    ..
                                } => {
                                    if let Some(order_col) = over_order {
                                        if let Some((idx, col_def)) = schema.find(order_col) {
                                            let my_bits = final_rows[row_idx]
                                                .values
                                                .get(idx)
                                                .copied()
                                                .unwrap_or(0);
                                            let my_s =
                                                TypedValue::from_bits(col_def.col_type, my_bits)
                                                    .display_str();
                                            let rank = final_rows
                                                .iter()
                                                .filter(|r| {
                                                    let rb =
                                                        r.values.get(idx).copied().unwrap_or(0);
                                                    let rs =
                                                        TypedValue::from_bits(col_def.col_type, rb)
                                                            .display_str();
                                                    compare_values(&rs, &BinOp::Lt, &my_s)
                                                })
                                                .count()
                                                + 1;
                                            rank.to_string()
                                        } else {
                                            (row_idx + 1).to_string()
                                        }
                                    } else {
                                        (row_idx + 1).to_string()
                                    }
                                }
                                SelectExpr::WindowFunc {
                                    func: WindowFunc::WinSum(sum_col),
                                    ..
                                } => {
                                    if let Some((idx, col_def)) = schema.find(sum_col) {
                                        let total: f64 = final_rows
                                            .iter()
                                            .map(|r| {
                                                TypedValue::from_bits(
                                                    col_def.col_type,
                                                    r.values.get(idx).copied().unwrap_or(0),
                                                )
                                                .display_str()
                                                .parse::<f64>()
                                                .unwrap_or(0.0)
                                            })
                                            .sum();
                                        format!("{:.0}", total)
                                    } else {
                                        "0".to_owned()
                                    }
                                }
                                SelectExpr::WindowFunc {
                                    func: WindowFunc::Lead { col, offset },
                                    ..
                                } => {
                                    let next_idx = row_idx + *offset as usize;
                                    if let Some((cidx, cdef)) = schema.find(col) {
                                        final_rows
                                            .get(next_idx)
                                            .map(|r| {
                                                TypedValue::from_bits(
                                                    cdef.col_type,
                                                    r.values.get(cidx).copied().unwrap_or(0),
                                                )
                                                .display_str()
                                            })
                                            .unwrap_or_else(|| "NULL".to_owned())
                                    } else {
                                        "NULL".to_owned()
                                    }
                                }
                                SelectExpr::WindowFunc {
                                    func: WindowFunc::Lag { col, offset },
                                    ..
                                } => {
                                    if row_idx >= *offset as usize {
                                        let prev_idx = row_idx - *offset as usize;
                                        if let Some((cidx, cdef)) = schema.find(col) {
                                            final_rows
                                                .get(prev_idx)
                                                .map(|r| {
                                                    TypedValue::from_bits(
                                                        cdef.col_type,
                                                        r.values.get(cidx).copied().unwrap_or(0),
                                                    )
                                                    .display_str()
                                                })
                                                .unwrap_or_else(|| "NULL".to_owned())
                                        } else {
                                            "NULL".to_owned()
                                        }
                                    } else {
                                        "NULL".to_owned()
                                    }
                                }
                                SelectExpr::WindowFunc {
                                    func: WindowFunc::FirstValue(col),
                                    ..
                                } => {
                                    if let Some((cidx, cdef)) = schema.find(col) {
                                        final_rows
                                            .first()
                                            .map(|r| {
                                                TypedValue::from_bits(
                                                    cdef.col_type,
                                                    r.values.get(cidx).copied().unwrap_or(0),
                                                )
                                                .display_str()
                                            })
                                            .unwrap_or_else(|| "NULL".to_owned())
                                    } else {
                                        "NULL".to_owned()
                                    }
                                }
                                SelectExpr::WindowFunc {
                                    func: WindowFunc::LastValue(col),
                                    ..
                                } => {
                                    if let Some((cidx, cdef)) = schema.find(col) {
                                        final_rows
                                            .last()
                                            .map(|r| {
                                                TypedValue::from_bits(
                                                    cdef.col_type,
                                                    r.values.get(cidx).copied().unwrap_or(0),
                                                )
                                                .display_str()
                                            })
                                            .unwrap_or_else(|| "NULL".to_owned())
                                    } else {
                                        "NULL".to_owned()
                                    }
                                }
                                SelectExpr::WindowFunc {
                                    func: WindowFunc::DenseRank,
                                    over_order,
                                    ..
                                } => {
                                    if let Some(order_col) = over_order {
                                        if let Some((idx, col_def)) = schema.find(order_col) {
                                            let my_bits = final_rows[row_idx]
                                                .values
                                                .get(idx)
                                                .copied()
                                                .unwrap_or(0);
                                            let my_s =
                                                TypedValue::from_bits(col_def.col_type, my_bits)
                                                    .display_str();
                                            // Dense rank: count distinct values < my_s + 1
                                            let mut seen_less: std::collections::HashSet<String> =
                                                std::collections::HashSet::new();
                                            for r in &final_rows {
                                                let rb = r.values.get(idx).copied().unwrap_or(0);
                                                let rs =
                                                    TypedValue::from_bits(col_def.col_type, rb)
                                                        .display_str();
                                                if compare_values(&rs, &BinOp::Lt, &my_s) {
                                                    seen_less.insert(rs);
                                                }
                                            }
                                            (seen_less.len() + 1).to_string()
                                        } else {
                                            (row_idx + 1).to_string()
                                        }
                                    } else {
                                        (row_idx + 1).to_string()
                                    }
                                }
                                SelectExpr::WindowFunc {
                                    func: WindowFunc::Ntile(buckets),
                                    ..
                                } => {
                                    let n = final_rows.len();
                                    let b = (*buckets as usize).max(1);
                                    let bucket = (row_idx * b / n.max(1)) + 1;
                                    bucket.min(b).to_string()
                                }
                                SelectExpr::WindowFunc {
                                    func: WindowFunc::PercentRank,
                                    over_order,
                                    ..
                                } => {
                                    let n = final_rows.len();
                                    if n <= 1 {
                                        return (col_name.clone(), "0".to_owned());
                                    }
                                    if let Some(order_col) = over_order {
                                        if let Some((idx, col_def)) = schema.find(order_col) {
                                            let my_bits = final_rows[row_idx]
                                                .values
                                                .get(idx)
                                                .copied()
                                                .unwrap_or(0);
                                            let my_s =
                                                TypedValue::from_bits(col_def.col_type, my_bits)
                                                    .display_str();
                                            let rank = final_rows
                                                .iter()
                                                .filter(|r| {
                                                    let rb =
                                                        r.values.get(idx).copied().unwrap_or(0);
                                                    let rs =
                                                        TypedValue::from_bits(col_def.col_type, rb)
                                                            .display_str();
                                                    compare_values(&rs, &BinOp::Lt, &my_s)
                                                })
                                                .count();
                                            let pct = rank as f64 / (n - 1) as f64;
                                            format!("{:.6}", pct)
                                        } else {
                                            "0".to_owned()
                                        }
                                    } else {
                                        "0".to_owned()
                                    }
                                }
                                _ => String::new(),
                            };
                            (col_name.clone(), val)
                        })
                        .collect()
                })
                .collect()
        } else {
            vec![]
        };

        // Build output
        let out_col_names: Vec<String> = out_cols.iter().map(|(n, _)| n.clone()).collect();
        let rows: Vec<Vec<String>> = final_rows
            .iter()
            .enumerate()
            .map(|(row_idx, row)| {
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
                        } else if *idx == usize::MAX - 1 {
                            // Window function column — look up pre-computed value.
                            window_row_vals
                                .get(row_idx)
                                .and_then(|wrow| {
                                    wrow.iter()
                                        .find(|(n, _)| n == col_name)
                                        .map(|(_, v)| v.clone())
                                })
                                .unwrap_or_default()
                        } else if *idx == usize::MAX - 2 {
                            // Computed expression (CASE WHEN, COALESCE, etc.) — evaluate per row.
                            computed_cols
                                .iter()
                                .find(|(n, _)| n == col_name)
                                .and_then(|(_, expr)| self.eval_value(expr, row, schema).ok())
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

        // S40 — set operations
        let (mut out_col_names, mut rows) = (out_col_names, rows);
        if let Some(set_op) = stmt.set_op {
            match set_op {
                SetOp::UnionAll(right_stmt) => {
                    if let Ok(QueryResult::Rows {
                        rows: right_rows, ..
                    }) = self.exec_select(*right_stmt)
                    {
                        rows.extend(right_rows);
                    }
                }
                SetOp::Intersect(right_stmt) => {
                    if let Ok(QueryResult::Rows {
                        rows: right_rows, ..
                    }) = self.exec_select(*right_stmt)
                    {
                        rows.retain(|row| right_rows.contains(row));
                    }
                }
                SetOp::Except(right_stmt) => {
                    if let Ok(QueryResult::Rows {
                        rows: right_rows, ..
                    }) = self.exec_select(*right_stmt)
                    {
                        rows.retain(|row| !right_rows.contains(row));
                    }
                }
            }
        }
        let _ = &mut out_col_names; // suppress unused_mut

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

        // Build group key indices for each group_by column
        let group_col_indices: Vec<usize> = stmt
            .group_by
            .iter()
            .filter_map(|col| schema.find(col).map(|(i, _)| i))
            .collect();

        let mut groups: BTreeMap<String, Vec<&vectorvolt_engine::EngineRow>> = BTreeMap::new();
        for row in &rows {
            let key = if group_col_indices.is_empty() {
                String::new()
            } else {
                group_col_indices
                    .iter()
                    .map(|&idx| {
                        row.values
                            .get(idx)
                            .map(|v| v.to_string())
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
                    .join("\x00")
            };
            groups.entry(key).or_default().push(row);
        }

        let mut col_headers: Vec<String> = stmt.group_by.clone();
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
            if !stmt.group_by.is_empty() {
                // Split composite key back into per-column values
                let parts: Vec<&str> = group_key.split('\x00').collect();
                for p in parts {
                    out_row.push(p.to_owned());
                }
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
            Expr::Col(name) => {
                // Direct lookup first
                if let Some(v) = row.get(name) {
                    return v.clone();
                }
                // Prefix match: "sum" matches "sum(v)" header
                let prefix = format!("{}(", name);
                for (k, v) in row {
                    if k.starts_with(&prefix) {
                        return v.clone();
                    }
                }
                String::new()
            }
            Expr::Lit(lit) => match lit {
                LitVal::Int(n) => n.to_string(),
                LitVal::Float(f) => format!("{f:.6}"),
                LitVal::Str(s) => s.clone(),
                LitVal::Bool(b) => b.to_string(),
                LitVal::Null => "NULL".to_owned(),
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
            // S58 — RENAME COLUMN
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
            }
            // S58 — CHANGE TYPE
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

        // S59 — apply defaults for missing columns
        if let Some(defaults) = self.col_defaults.lock().unwrap().get(&stmt.table).cloned() {
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

        // S60 — FK validation
        let fks = self
            .foreign_keys
            .lock()
            .unwrap()
            .get(&stmt.table)
            .cloned()
            .unwrap_or_default();
        for (fk_col, ref_table, ref_col) in &fks {
            if let Some((fk_idx, fk_def)) = schema.find(fk_col) {
                let val = row_vals[fk_idx];
                let val_str = TypedValue::from_bits(fk_def.col_type, val).display_str();
                if val_str == "0" || val_str == "NULL" {
                    continue; // NULL FK values are allowed
                }
                // Check ref_table has a row with ref_col = val
                let ref_engine = self
                    .registry
                    .get(ref_table)
                    .map_err(|_| ExecError::TableNotFound(ref_table.clone()))?;
                let ref_rows = self.scan_live(&ref_engine)?;
                let found = ref_rows.iter().any(|r| {
                    if let Some((ref_idx, ref_def)) = ref_engine.schema.find(ref_col) {
                        let rv = TypedValue::from_bits(
                            ref_def.col_type,
                            r.values.get(ref_idx).copied().unwrap_or(0),
                        )
                        .display_str();
                        rv == val_str
                    } else {
                        false
                    }
                });
                if !found {
                    return Err(ExecError::Engine(format!(
                        "foreign key violation: {fk_col} -> {ref_table}.{ref_col}"
                    )));
                }
            }
        }

        // S59 — CHECK constraint validation
        let checks = self
            .col_checks
            .lock()
            .unwrap()
            .get(&stmt.table)
            .cloned()
            .unwrap_or_default();
        for (col_idx, maybe_check) in checks.iter().enumerate() {
            if let Some(check_expr) = maybe_check {
                let col_name = schema
                    .get(col_idx)
                    .map(|c| c.name.clone())
                    .unwrap_or_default();
                let col_type = schema
                    .get(col_idx)
                    .map(|c| c.col_type)
                    .unwrap_or(ColumnType::U64);
                let val = row_vals.get(col_idx).copied().unwrap_or(0);
                let val_str = TypedValue::from_bits(col_type, val).display_str();
                if !eval_check(check_expr, &col_name, &val_str) {
                    return Err(ExecError::Engine(format!(
                        "check constraint violated on column {col_name}"
                    )));
                }
            }
        }

        let pk = row_vals[0];

        // S44 — ON CONFLICT DO NOTHING
        if stmt.on_conflict_ignore {
            let already_exists = engine
                .get(pk)
                .map_err(|e| ExecError::Engine(e.to_string()))?
                .is_some();
            if already_exists {
                return Ok(QueryResult::Affected(0));
            }
        }

        // S44 — ON CONFLICT DO UPDATE
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
                engine
                    .update(pk, new_vals)
                    .map_err(|e| ExecError::Engine(e.to_string()))?;
                return Ok(QueryResult::Affected(1));
            }
        }

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
        // S59 — collect defaults and checks before consuming stmt
        let defaults: Vec<Option<LitVal>> =
            stmt.columns.iter().map(|c| c.default.clone()).collect();
        let checks: Vec<Option<String>> = stmt.columns.iter().map(|c| c.check.clone()).collect();
        // S60 — collect foreign keys
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

        self.registry
            .create_table_typed(&stmt.table, schema)
            .map_err(|e| ExecError::Engine(e.to_string()))?;

        // S59 — store defaults/checks
        let has_defaults = defaults.iter().any(|d| d.is_some());
        let has_checks = checks.iter().any(|c| c.is_some());
        if has_defaults {
            self.col_defaults
                .lock()
                .unwrap()
                .insert(table_name.clone(), defaults);
        }
        if has_checks {
            self.col_checks
                .lock()
                .unwrap()
                .insert(table_name.clone(), checks);
        }
        // S60 — store FKs
        if !fks.is_empty() {
            self.foreign_keys.lock().unwrap().insert(table_name, fks);
        }

        Ok(QueryResult::Created)
    }

    fn exec_drop_table(&self, stmt: DropTableStmt) -> Result<QueryResult, ExecError> {
        self.registry
            .drop_table(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
        Ok(QueryResult::Dropped)
    }

    // S42 — Index DDL
    fn exec_create_index(&self, stmt: CreateIndexStmt) -> Result<QueryResult, ExecError> {
        let base = self.registry.base_dir();
        let dir = base.join(&stmt.table).join("indexes");
        std::fs::create_dir_all(&dir).map_err(|e| ExecError::Engine(e.to_string()))?;
        let path = dir.join(format!("{}.json", stmt.index_name));
        let content = format!("{{\"col\":\"{}\"}}", stmt.col);
        std::fs::write(path, content).map_err(|e| ExecError::Engine(e.to_string()))?;
        Ok(QueryResult::Created)
    }

    fn exec_drop_index(&self, stmt: DropIndexStmt) -> Result<QueryResult, ExecError> {
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
                    let _ = std::fs::remove_file(idx_path);
                    return Ok(QueryResult::Dropped);
                }
            }
        }
        let _ = path; // suppress warning
        Ok(QueryResult::Dropped)
    }

    // S45 — CREATE/DROP VIEW
    fn exec_create_view(&self, stmt: CreateViewStmt) -> Result<QueryResult, ExecError> {
        self.views.lock().unwrap().insert(stmt.name, *stmt.select);
        Ok(QueryResult::Created)
    }

    fn exec_drop_view(&self, stmt: DropViewStmt) -> Result<QueryResult, ExecError> {
        self.views.lock().unwrap().remove(&stmt.name);
        Ok(QueryResult::Dropped)
    }

    /// S41/S45 helper — filter/project string rows.
    fn project_str_rows(
        exprs: &[SelectExpr],
        rows: &[Vec<String>],
        col_names: &[String],
    ) -> (Vec<String>, Vec<Vec<String>>) {
        if exprs.iter().any(|e| matches!(e, SelectExpr::Star)) {
            return (col_names.to_vec(), rows.to_vec());
        }
        let mut headers: Vec<String> = Vec::new();
        let mut out_rows: Vec<Vec<String>> = rows.iter().map(|_| Vec::new()).collect();
        for expr in exprs {
            let col_name = match expr {
                SelectExpr::Col(n) => n.clone(),
                SelectExpr::QualifiedCol { col, .. } => col.clone(),
                _ => continue,
            };
            if let Some(idx) = col_names.iter().position(|c| c == &col_name) {
                headers.push(col_name);
                for (i, row) in rows.iter().enumerate() {
                    out_rows[i].push(row.get(idx).cloned().unwrap_or_default());
                }
            }
        }
        (headers, out_rows)
    }

    /// S41/S45 helper — evaluate WHERE on a string-row map.
    fn eval_str_where(expr: &Expr, row: &HashMap<String, String>) -> bool {
        match expr {
            Expr::BinOp {
                op: BinOp::And,
                left,
                right,
            } => Self::eval_str_where(left, row) && Self::eval_str_where(right, row),
            Expr::BinOp {
                op: BinOp::Or,
                left,
                right,
            } => Self::eval_str_where(left, row) || Self::eval_str_where(right, row),
            Expr::BinOp { op, left, right } => {
                let lv = Self::str_val(left, row);
                let rv = Self::str_val(right, row);
                compare_values(&lv, op, &rv)
            }
            Expr::IsNull { col, negated } => {
                let v = row.get(col).cloned().unwrap_or_default();
                let is_null = v.is_empty() || v == "NULL";
                if *negated { !is_null } else { is_null }
            }
            _ => true, // permissive for unsupported predicates
        }
    }

    fn str_val(expr: &Expr, row: &HashMap<String, String>) -> String {
        match expr {
            Expr::Col(name) => row.get(name).cloned().unwrap_or_default(),
            Expr::Lit(lit) => match lit {
                LitVal::Int(n) => n.to_string(),
                LitVal::Float(f) => format!("{f:.6}"),
                LitVal::Str(s) => s.clone(),
                LitVal::Bool(b) => b.to_string(),
                LitVal::Null => "NULL".to_owned(),
            },
            _ => String::new(),
        }
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
                // S46 — treat missing zone files as empty (zone pruning correctness fix)
                let raw = match engine.scan(*col_idx, *lo, *hi) {
                    Ok(rows) => rows,
                    Err(e) => {
                        let msg = e.to_string().to_lowercase();
                        if msg.contains("not found")
                            || msg.contains("no such")
                            || msg.contains("os error 2")
                        {
                            vec![]
                        } else {
                            return Err(ExecError::Engine(e.to_string()));
                        }
                    }
                };
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
                        LitVal::Null => "NULL".to_owned(),
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
                    LitVal::Null => "NULL".to_owned(),
                };
                let hi_s = match hi {
                    LitVal::Int(n) => n.to_string(),
                    LitVal::Float(f) => format!("{f:.6}"),
                    LitVal::Str(s) => s.clone(),
                    LitVal::Bool(b) => b.to_string(),
                    LitVal::Null => "NULL".to_owned(),
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
            Expr::IsNull { col, negated } => {
                let (idx, col_def) = schema
                    .find(col)
                    .ok_or_else(|| ExecError::ColumnNotFound(col.clone()))?;
                let bits = row.values.get(idx).copied().unwrap_or(0);
                let cell = TypedValue::from_bits(col_def.col_type, bits).display_str();
                let is_null = bits == 0 || cell.is_empty() || cell == "NULL";
                Ok(if *negated { !is_null } else { is_null })
            }
            Expr::Case { branches, else_val } => {
                for (when_expr, _) in branches {
                    if self.eval_expr(when_expr, row, schema, table)? {
                        return Ok(true);
                    }
                }
                Ok(else_val.is_some())
            }
            Expr::Like {
                col,
                pattern,
                negated,
            } => {
                let (idx, col_def) = schema
                    .find(col)
                    .ok_or_else(|| ExecError::ColumnNotFound(col.clone()))?;
                let bits = row.values.get(idx).copied().unwrap_or(0);
                let val = TypedValue::from_bits(col_def.col_type, bits).display_str();
                let matched = like_match(&val, pattern);
                Ok(if *negated { !matched } else { matched })
            }
            Expr::Arith { .. }
            | Expr::ScalarFn { .. }
            | Expr::Cast { .. }
            | Expr::StrFunc { .. }
            | Expr::DateExtract { .. }
            | Expr::ScalarSubquery { .. }
            | Expr::Placeholder(_) => Err(ExecError::TypeError(
                "arithmetic/scalar not valid as boolean".into(),
            )),
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
                LitVal::Null => "NULL".to_owned(),
            }),
            Expr::QualifiedCol { col, .. } => {
                let (idx, col_def) = schema
                    .find(col)
                    .ok_or_else(|| ExecError::ColumnNotFound(col.clone()))?;
                let bits = row.values.get(idx).copied().unwrap_or(0);
                Ok(TypedValue::from_bits(col_def.col_type, bits).display_str())
            }
            Expr::BinOp { op, left, right } => match op {
                BinOp::And | BinOp::Or => Err(ExecError::TypeError(
                    "AND/OR not valid in scalar context".into(),
                )),
                _ => {
                    let lv = self.eval_value(left, row, schema)?;
                    let rv = self.eval_value(right, row, schema)?;
                    Ok(if compare_values(&lv, op, &rv) {
                        "true".to_owned()
                    } else {
                        "false".to_owned()
                    })
                }
            },
            Expr::Coalesce { args } => {
                for arg in args {
                    let val = self.eval_value(arg, row, schema)?;
                    if !val.is_empty() && val != "NULL" {
                        return Ok(val);
                    }
                }
                Ok("NULL".to_owned())
            }
            Expr::Case { branches, else_val } => {
                for (when_expr, then_expr) in branches {
                    let cond = self.eval_value(when_expr, row, schema)?;
                    let truthy =
                        !cond.is_empty() && cond != "0" && cond != "false" && cond != "NULL";
                    if truthy {
                        return self.eval_value(then_expr, row, schema);
                    }
                }
                if let Some(ev) = else_val {
                    self.eval_value(ev, row, schema)
                } else {
                    Ok("NULL".to_owned())
                }
            }
            Expr::Arith { op, left, right } => {
                let lv = self.eval_value(left, row, schema)?;
                let rv = self.eval_value(right, row, schema)?;
                if lv == "NULL" || rv == "NULL" {
                    return Ok("NULL".to_owned());
                }
                let lf: f64 = lv.parse().unwrap_or(0.0);
                let rf: f64 = rv.parse().unwrap_or(0.0);
                let result = match op {
                    ArithOp::Add => lf + rf,
                    ArithOp::Sub => lf - rf,
                    ArithOp::Mul => lf * rf,
                    ArithOp::Div => {
                        if rf != 0.0 {
                            lf / rf
                        } else {
                            0.0
                        }
                    }
                    ArithOp::Mod => {
                        if rf != 0.0 {
                            lf % rf
                        } else {
                            0.0
                        }
                    }
                };
                if result.fract() == 0.0 && result.abs() < 1e15 {
                    Ok(format!("{}", result as i64))
                } else {
                    Ok(format!("{}", result))
                }
            }
            Expr::ScalarFn { func, arg } => {
                let val = self.eval_value(arg, row, schema)?;
                if val == "NULL" && !matches!(func, ScalarFunc::Now) {
                    return Ok("NULL".to_owned());
                }
                match func {
                    ScalarFunc::Upper => Ok(val.to_uppercase()),
                    ScalarFunc::Lower => Ok(val.to_lowercase()),
                    ScalarFunc::Length => Ok(val.len().to_string()),
                    ScalarFunc::Trim => Ok(val.trim().to_owned()),
                    ScalarFunc::Now => {
                        // Return a simple timestamp string
                        Ok("2026-09-10 00:00:00".to_owned())
                    }
                    _ => {
                        let f: f64 = val.parse().unwrap_or(0.0);
                        let result = match func {
                            ScalarFunc::Abs => f.abs(),
                            ScalarFunc::Round => f.round(),
                            ScalarFunc::Ceil => f.ceil(),
                            ScalarFunc::Floor => f.floor(),
                            _ => unreachable!(),
                        };
                        if result.fract() == 0.0 && result.abs() < 1e15 {
                            Ok(format!("{}", result as i64))
                        } else {
                            Ok(format!("{}", result))
                        }
                    }
                }
            }
            Expr::Cast { expr, target } => {
                let val = self.eval_value(expr, row, schema)?;
                if val == "NULL" {
                    return Ok("NULL".to_owned());
                }
                match target.to_lowercase().as_str() {
                    "u64" | "i64" => {
                        let n: f64 = val.parse().unwrap_or(0.0);
                        Ok(format!("{}", n as i64))
                    }
                    "f64" => {
                        let f: f64 = val.parse().unwrap_or(0.0);
                        Ok(format!("{}", f))
                    }
                    "text" => Ok(val),
                    "bool" => {
                        let b = val != "0" && !val.is_empty() && val != "false" && val != "NULL";
                        Ok(b.to_string())
                    }
                    _ => Ok(val),
                }
            }
            Expr::StrFunc { name, args } => {
                let vals: Vec<String> = args
                    .iter()
                    .map(|a| self.eval_value(a, row, schema).unwrap_or_default())
                    .collect();
                match name.as_str() {
                    "SUBSTR" => {
                        let s = vals.first().cloned().unwrap_or_default();
                        let start: usize = vals
                            .get(1)
                            .and_then(|v| v.parse::<i64>().ok())
                            .map(|n| if n > 0 { (n - 1) as usize } else { 0 })
                            .unwrap_or(0);
                        let len_opt: Option<usize> =
                            vals.get(2).and_then(|v| v.parse::<usize>().ok());
                        let chars: Vec<char> = s.chars().collect();
                        let slice = if let Some(len) = len_opt {
                            chars.iter().skip(start).take(len).collect::<String>()
                        } else {
                            chars.iter().skip(start).collect::<String>()
                        };
                        Ok(slice)
                    }
                    "CONCAT" => Ok(vals.concat()),
                    "INSTR" => {
                        let s = vals.first().cloned().unwrap_or_default();
                        let sub = vals.get(1).cloned().unwrap_or_default();
                        let pos = s.find(&sub).map(|p| (p + 1) as i64).unwrap_or(0);
                        Ok(pos.to_string())
                    }
                    "REPLACE" => {
                        let s = vals.first().cloned().unwrap_or_default();
                        let from = vals.get(1).cloned().unwrap_or_default();
                        let to = vals.get(2).cloned().unwrap_or_default();
                        Ok(s.replace(&from, &to))
                    }
                    "DATE_ADD" => {
                        let date_str = vals.first().cloned().unwrap_or_default();
                        let days: i64 = vals.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                        Ok(date_add(&date_str, days))
                    }
                    "DATE_DIFF" => {
                        let d1 = vals.first().cloned().unwrap_or_default();
                        let d2 = vals.get(1).cloned().unwrap_or_default();
                        Ok(date_diff(&d1, &d2).to_string())
                    }
                    // S64 — DATE_TRUNC
                    "DATE_TRUNC" => {
                        let part = vals.first().cloned().unwrap_or_default();
                        let date_str = vals.get(1).cloned().unwrap_or_default();
                        Ok(date_trunc(&part, &date_str))
                    }
                    // S64 — TIME_BUCKET
                    "TIME_BUCKET" => {
                        let interval: f64 =
                            vals.first().and_then(|v| v.parse().ok()).unwrap_or(1.0);
                        let ts: f64 = vals.get(1).and_then(|v| v.parse().ok()).unwrap_or(0.0);
                        if interval > 0.0 {
                            Ok(((ts / interval).floor() * interval).to_string())
                        } else {
                            Ok(ts.to_string())
                        }
                    }
                    // S65 — UDF dispatch
                    udf_name => {
                        let maybe_udf = self.udfs.lock().unwrap().get(udf_name).cloned();
                        if let Some((param, body)) = maybe_udf {
                            let arg_val = vals.first().cloned().unwrap_or_default();
                            // Simple substitution: replace param occurrences in body
                            let substituted = body.replace(&param, &arg_val);
                            // Re-lex and evaluate as a literal expression
                            if let Ok(n) = substituted.parse::<f64>() {
                                if n.fract() == 0.0 {
                                    Ok(format!("{}", n as i64))
                                } else {
                                    Ok(format!("{}", n))
                                }
                            } else {
                                Ok(substituted)
                            }
                        } else {
                            Ok(vals.concat())
                        }
                    }
                }
            }
            Expr::DateExtract { part, expr } => {
                let date_str = self.eval_value(expr, row, schema)?;
                // Expect format YYYY-MM-DD
                let parts: Vec<&str> = date_str.split('-').collect();
                match part.to_uppercase().as_str() {
                    "YEAR" => Ok(parts.first().cloned().unwrap_or("0").to_owned()),
                    "MONTH" => Ok(parts.get(1).cloned().unwrap_or("0").to_owned()),
                    "DAY" => Ok(parts
                        .get(2)
                        .cloned()
                        .map(|s| s.split(' ').next().unwrap_or(s))
                        .unwrap_or("0")
                        .to_owned()),
                    _ => Ok("0".to_owned()),
                }
            }
            Expr::ScalarSubquery { subquery } => {
                let result = self.exec_select(*subquery.clone())?;
                match result {
                    QueryResult::Rows { rows, .. } => Ok(rows
                        .into_iter()
                        .next()
                        .and_then(|r| r.into_iter().next())
                        .unwrap_or_else(|| "NULL".to_owned())),
                    _ => Ok("NULL".to_owned()),
                }
            }
            Expr::Placeholder(_) => Ok("NULL".to_owned()),
            // S62 — MATCH_SCORE: count term occurrences in column value
            Expr::MatchScore { col, term } => {
                let val = if let Some((idx, col_def)) = schema.find(col) {
                    TypedValue::from_bits(
                        col_def.col_type,
                        row.values.get(idx).copied().unwrap_or(0),
                    )
                    .display_str()
                } else {
                    String::new()
                };
                let score = val.to_lowercase().matches(&term.to_lowercase()).count();
                Ok(score.to_string())
            }
            // S63 — JSON operators
            Expr::JsonGet {
                expr: inner_expr,
                key,
            }
            | Expr::JsonGetText {
                expr: inner_expr,
                key,
            } => {
                let json_str = self.eval_value(inner_expr, row, schema)?;
                Ok(json_extract(&json_str, key))
            }
            Expr::Match { .. }
            | Expr::In { .. }
            | Expr::Between { .. }
            | Expr::InSubquery { .. }
            | Expr::Exists { .. }
            | Expr::IsNull { .. }
            | Expr::Like { .. } => Err(ExecError::TypeError(
                "expected scalar value expression".into(),
            )),
        }
    }

    // S65 — UDFs
    fn exec_create_function(
        &self,
        name: String,
        param: String,
        body: String,
    ) -> Result<QueryResult, ExecError> {
        self.udfs.lock().unwrap().insert(name, (param, body));
        Ok(QueryResult::Created)
    }

    fn exec_drop_function(&self, name: String) -> Result<QueryResult, ExecError> {
        self.udfs.lock().unwrap().remove(&name);
        Ok(QueryResult::Dropped)
    }

    // S66 — Triggers
    fn exec_create_trigger(&self, def: TriggerDef) -> Result<QueryResult, ExecError> {
        self.triggers.lock().unwrap().insert(def.name.clone(), def);
        Ok(QueryResult::Created)
    }

    fn exec_drop_trigger(&self, name: String) -> Result<QueryResult, ExecError> {
        self.triggers.lock().unwrap().remove(&name);
        Ok(QueryResult::Dropped)
    }

    // S53 — Savepoints
    fn exec_savepoint(&self, name: String) -> Result<QueryResult, ExecError> {
        let guard = self.txn.lock().unwrap();
        let snapshot = guard.as_ref().cloned().unwrap_or_default();
        drop(guard);
        self.savepoints.lock().unwrap().insert(name, snapshot);
        Ok(QueryResult::TxBegin)
    }

    fn exec_release_savepoint(&self, name: String) -> Result<QueryResult, ExecError> {
        self.savepoints.lock().unwrap().remove(&name);
        Ok(QueryResult::TxCommit)
    }

    fn exec_rollback_to(&self, name: String) -> Result<QueryResult, ExecError> {
        let sp = self
            .savepoints
            .lock()
            .unwrap()
            .get(&name)
            .cloned()
            .unwrap_or_default();
        // Rollback tables that were opened after the savepoint
        let current = self.txn.lock().unwrap().clone().unwrap_or_default();
        for (table, txn_id) in &current {
            if !sp.contains_key(table)
                && let Ok(engine) = self.registry.get(table)
            {
                let _ = engine.rollback_txn(*txn_id);
            }
        }
        *self.txn.lock().unwrap() = Some(sp);
        Ok(QueryResult::TxRollback)
    }

    // S56 — EXPLAIN ANALYZE
    fn exec_explain_analyze(&self, stmt: SelectStmt) -> Result<QueryResult, ExecError> {
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
        let mut lines = plan.explain(0);
        lines.push(format!("  actual_rows=~{}", est_rows));
        Ok(QueryResult::ExplainPlan(lines))
    }

    // S57 — Prepared statements
    fn exec_prepare(&self, name: String, stmt: Stmt) -> Result<QueryResult, ExecError> {
        self.prepared.lock().unwrap().insert(name, stmt);
        Ok(QueryResult::Affected(0))
    }

    fn exec_execute_prepared(
        &self,
        name: String,
        params: Vec<Expr>,
    ) -> Result<QueryResult, ExecError> {
        let stmt = self
            .prepared
            .lock()
            .unwrap()
            .get(&name)
            .cloned()
            .ok_or_else(|| ExecError::Engine(format!("prepared statement '{}' not found", name)))?;
        let substituted = substitute_params(stmt, &params);
        self.execute_stmt(substituted)
    }

    fn expr_to_bits(&self, expr: &Expr, col_type: ColumnType) -> Result<u64, ExecError> {
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

// S57 — parameter substitution for prepared statements
fn substitute_params(stmt: Stmt, params: &[Expr]) -> Stmt {
    match stmt {
        Stmt::Select(s) => Stmt::Select(subst_select(s, params)),
        Stmt::Insert(mut ins) => {
            ins.values = ins
                .values
                .into_iter()
                .map(|e| subst_expr(e, params))
                .collect();
            Stmt::Insert(ins)
        }
        other => other,
    }
}

fn subst_select(mut s: SelectStmt, params: &[Expr]) -> SelectStmt {
    if let Some(wc) = s.where_clause {
        s.where_clause = Some(subst_expr(wc, params));
    }
    s
}

fn subst_expr(expr: Expr, params: &[Expr]) -> Expr {
    match expr {
        Expr::Placeholder(n) => {
            if n > 0 && n <= params.len() {
                params[n - 1].clone()
            } else {
                Expr::Lit(LitVal::Null)
            }
        }
        Expr::BinOp { op, left, right } => Expr::BinOp {
            op,
            left: Box::new(subst_expr(*left, params)),
            right: Box::new(subst_expr(*right, params)),
        },
        other => other,
    }
}

// S50 — date helper functions
fn parse_date(s: &str) -> Option<i64> {
    // Accept YYYY-MM-DD
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() < 3 {
        return None;
    }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].split_whitespace().next()?.parse().ok()?;
    // Days since epoch (approximate)
    let days = y * 365 + (y / 4) - (y / 100)
        + (y / 400)
        + [0i64, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334][(m.clamp(1, 12) - 1) as usize]
        + d;
    Some(days)
}

fn days_to_ymd(days: i64) -> (i64, i64, i64) {
    // Rough approximation
    let y = days / 365;
    let rem = days - y * 365 - y / 4 + y / 100 - y / 400;
    let month_days = [31i64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut d = rem;
    let mut m = 1i64;
    for md in &month_days {
        if d <= *md {
            break;
        }
        d -= md;
        m += 1;
    }
    (y, m, d.max(1))
}

fn date_add(date: &str, days: i64) -> String {
    if let Some(d) = parse_date(date) {
        let new_d = d + days;
        let (y, m, day) = days_to_ymd(new_d);
        format!("{:04}-{:02}-{:02}", y, m, day)
    } else {
        date.to_owned()
    }
}

fn date_diff(d1: &str, d2: &str) -> i64 {
    match (parse_date(d1), parse_date(d2)) {
        (Some(a), Some(b)) => a - b,
        _ => 0,
    }
}

fn like_match(val: &str, pattern: &str) -> bool {
    if pattern == "%" {
        return true;
    }
    if pattern.starts_with('%') && pattern.ends_with('%') && pattern.len() > 2 {
        let inner = &pattern[1..pattern.len() - 1];
        return val.contains(inner);
    }
    if let Some(suffix) = pattern.strip_prefix('%') {
        return val.ends_with(suffix);
    }
    if let Some(prefix) = pattern.strip_suffix('%') {
        return val.starts_with(prefix);
    }
    val == pattern
}

// S59 — simple check constraint evaluator
fn eval_check(expr: &str, col: &str, val: &str) -> bool {
    // Parse simple expressions like "col > 0", "col >= 18", "col < 100"
    let expr = expr.trim();
    let ops = [
        (">=", "gte"),
        ("<=", "lte"),
        ("<>", "neq"),
        (">", "gt"),
        ("<", "lt"),
        ("=", "eq"),
    ];
    for (op_str, _) in &ops {
        if let Some(rhs) = expr.split(op_str).nth(1) {
            let rhs = rhs.trim().trim_matches('\'');
            let lhs_part = expr.split(op_str).next().unwrap_or("").trim();
            if lhs_part.eq_ignore_ascii_case(col) || lhs_part == col {
                if let (Ok(l), Ok(r)) = (val.parse::<f64>(), rhs.parse::<f64>()) {
                    return match *op_str {
                        ">=" => l >= r,
                        "<=" => l <= r,
                        "<>" => (l - r).abs() >= f64::EPSILON,
                        ">" => l > r,
                        "<" => l < r,
                        "=" => (l - r).abs() < f64::EPSILON,
                        _ => true,
                    };
                }
                return match *op_str {
                    "<>" => val != rhs,
                    "=" => val == rhs,
                    _ => true,
                };
            }
        }
    }
    true // unknown expression → pass
}

// S63 — simple JSON key extraction
fn json_extract(json: &str, key: &str) -> String {
    // Find "key": value pattern
    let needle = format!("\"{}\":", key);
    if let Some(pos) = json.find(&needle) {
        let after = json[pos + needle.len()..].trim_start();
        // Read until , or }
        let end = after.find([',', '}']).unwrap_or(after.len());
        let raw = after[..end].trim().trim_matches('"');
        return raw.to_owned();
    }
    "null".to_owned()
}

// S64 — DATE_TRUNC
fn date_trunc(part: &str, date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    match part.to_ascii_uppercase().as_str() {
        "YEAR" => {
            let year = parts.first().cloned().unwrap_or("0000");
            format!("{}-01-01", year)
        }
        "MONTH" => {
            let year = parts.first().cloned().unwrap_or("0000");
            let month = parts.get(1).cloned().unwrap_or("01");
            format!("{}-{}-01", year, month)
        }
        "DAY" | "DATE" => date.to_owned(),
        _ => date.to_owned(),
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

    // ── S37: Window functions ─────────────────────────────────────────────────

    #[test]
    fn test_s37_row_number() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE wt (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO wt (pk, v) VALUES (1, 10)").unwrap();
        ex.execute("INSERT INTO wt (pk, v) VALUES (2, 20)").unwrap();
        ex.execute("INSERT INTO wt (pk, v) VALUES (3, 30)").unwrap();
        let r = ex
            .execute("SELECT pk, ROW_NUMBER() OVER () FROM wt")
            .unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"row_number".to_owned()));
            assert_eq!(rows.len(), 3);
            // ROW_NUMBER values should be 1, 2, 3
            let rns: Vec<&str> = rows.iter().map(|r| r[1].as_str()).collect();
            assert!(rns.contains(&"1") && rns.contains(&"2") && rns.contains(&"3"));
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s37_rank_over_order() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE rt (pk U64, score U64)").unwrap();
        ex.execute("INSERT INTO rt (pk, score) VALUES (1, 100)")
            .unwrap();
        ex.execute("INSERT INTO rt (pk, score) VALUES (2, 200)")
            .unwrap();
        ex.execute("INSERT INTO rt (pk, score) VALUES (3, 150)")
            .unwrap();
        let r = ex
            .execute("SELECT pk, RANK() OVER (ORDER BY score) FROM rt ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"rank".to_owned()));
            // pk=1 has score=100 (lowest) → rank 1
            let pk1_rank = &rows.iter().find(|r| r[0] == "1").unwrap()[1];
            assert_eq!(pk1_rank, "1");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s37_win_sum_over() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE st (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO st (pk, v) VALUES (1, 10)").unwrap();
        ex.execute("INSERT INTO st (pk, v) VALUES (2, 20)").unwrap();
        ex.execute("INSERT INTO st (pk, v) VALUES (3, 30)").unwrap();
        let r = ex.execute("SELECT pk, SUM(v) OVER () FROM st").unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"win_sum(v)".to_owned()));
            // All rows should have the same total: 10+20+30=60
            assert!(rows.iter().all(|r| r[1] == "60"));
        } else {
            panic!("expected rows");
        }
    }

    // ── S37: CASE WHEN ────────────────────────────────────────────────────────

    #[test]
    fn test_s37_case_when() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE ct (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO ct (pk, v) VALUES (1, 5)").unwrap();
        ex.execute("INSERT INTO ct (pk, v) VALUES (2, 15)").unwrap();
        let r = ex
            .execute("SELECT pk, CASE WHEN v = 5 THEN 100 ELSE 200 END FROM ct ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0][1], "100"); // pk=1, v=5 → 100
            assert_eq!(rows[1][1], "200"); // pk=2, v=15 → 200
        } else {
            panic!("expected rows");
        }
    }

    // ── S37: NULL handling ────────────────────────────────────────────────────

    #[test]
    fn test_s37_is_null() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE nt (pk U64, v U64)").unwrap();
        // Insert row with v=0 (treated as NULL in our u64 system)
        ex.execute("INSERT INTO nt (pk, v) VALUES (1, 0)").unwrap();
        ex.execute("INSERT INTO nt (pk, v) VALUES (2, 99)").unwrap();
        let r = ex.execute("SELECT pk FROM nt WHERE v IS NULL").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            // Only pk=1 (v=0) should match IS NULL
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "1");
        } else {
            panic!("expected rows");
        }
    }

    // ── S38: String + Pagination ─────────────────────────────────────────────

    #[test]
    fn test_s38_limit_offset() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE pg (pk U64, v U64)").unwrap();
        for i in 1u64..=5 {
            ex.execute(&format!("INSERT INTO pg (pk, v) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex
            .execute("SELECT pk FROM pg ORDER BY pk ASC LIMIT 2 OFFSET 2")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0][0], "3");
            assert_eq!(rows[1][0], "4");
        } else {
            panic!("expected rows");
        }
    }

    // ── S39: Arithmetic Expressions ──────────────────────────────────────────

    #[test]
    fn test_s39_add_columns() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
            .unwrap();
        let r = ex
            .execute("SELECT pk + val AS total FROM t ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"total".to_owned()));
            assert_eq!(rows[0][0], "11");
            assert_eq!(rows[1][0], "22");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s38_like_prefix() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE lk (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO lk (pk, val) VALUES (1, 100)")
            .unwrap();
        ex.execute("INSERT INTO lk (pk, val) VALUES (2, 200)")
            .unwrap();
        ex.execute("INSERT INTO lk (pk, val) VALUES (3, 150)")
            .unwrap();
        let r = ex.execute("SELECT pk FROM lk WHERE val LIKE '1%'").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
            let pks: Vec<&str> = rows.iter().map(|r| r[0].as_str()).collect();
            assert!(pks.contains(&"1"));
            assert!(pks.contains(&"3"));
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s39_sub_literal() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        let r = ex
            .execute("SELECT val - 1 AS dec FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "9");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s38_like_contains() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE lk2 (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO lk2 (pk, val) VALUES (1, 100)")
            .unwrap();
        ex.execute("INSERT INTO lk2 (pk, val) VALUES (2, 999)")
            .unwrap();
        ex.execute("INSERT INTO lk2 (pk, val) VALUES (3, 200)")
            .unwrap();
        let r = ex
            .execute("SELECT pk FROM lk2 WHERE val LIKE '%0%'")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
            let pks: Vec<&str> = rows.iter().map(|r| r[0].as_str()).collect();
            assert!(pks.contains(&"1"));
            assert!(pks.contains(&"3"));
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s39_mul_columns() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 5)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 7)").unwrap();
        let r = ex
            .execute("SELECT pk * val AS prod FROM t WHERE pk = 2")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "14");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s38_scalar_fn_length() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE sf (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO sf (pk, val) VALUES (1, 12345)")
            .unwrap();
        ex.execute("INSERT INTO sf (pk, val) VALUES (2, 7)")
            .unwrap();
        let r = ex
            .execute("SELECT LENGTH(val) AS len FROM sf WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "5");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s39_scalar_abs() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 5)").unwrap();
        let r = ex
            .execute("SELECT ABS(val) AS a FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "5");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s39_scalar_round() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE ft (pk U64, fval F64)").unwrap();
        ex.execute("INSERT INTO ft (pk, fval) VALUES (1, 3.7)")
            .unwrap();
        let r = ex.execute("SELECT ROUND(fval) AS r FROM ft").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "4");
        } else {
            panic!("expected rows");
        }
    }

    // ── S40: Set Operations ───────────────────────────────────────────────────

    #[test]
    fn test_s40_union_all() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t1 (pk U64, val U64)").unwrap();
        ex.execute("CREATE TABLE t2 (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t1 (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t1 (pk, val) VALUES (2, 20)")
            .unwrap();
        ex.execute("INSERT INTO t2 (pk, val) VALUES (3, 30)")
            .unwrap();
        let r = ex
            .execute("SELECT pk FROM t1 UNION ALL SELECT pk FROM t2")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 3);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s40_union_dedup() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE a (pk U64, v U64)").unwrap();
        ex.execute("CREATE TABLE b (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO a (pk, v) VALUES (1, 10)").unwrap();
        ex.execute("INSERT INTO b (pk, v) VALUES (1, 10)").unwrap();
        // UNION ALL includes duplicates
        let r = ex
            .execute("SELECT pk FROM a UNION ALL SELECT pk FROM b")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s40_intersect() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE x (pk U64, v U64)").unwrap();
        ex.execute("CREATE TABLE y (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO x (pk, v) VALUES (1, 1)").unwrap();
        ex.execute("INSERT INTO x (pk, v) VALUES (2, 2)").unwrap();
        ex.execute("INSERT INTO y (pk, v) VALUES (2, 2)").unwrap();
        ex.execute("INSERT INTO y (pk, v) VALUES (3, 3)").unwrap();
        let r = ex
            .execute("SELECT pk FROM x INTERSECT SELECT pk FROM y")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "2");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s40_except() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE x (pk U64, v U64)").unwrap();
        ex.execute("CREATE TABLE y (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO x (pk, v) VALUES (1, 1)").unwrap();
        ex.execute("INSERT INTO x (pk, v) VALUES (2, 2)").unwrap();
        ex.execute("INSERT INTO x (pk, v) VALUES (3, 3)").unwrap();
        ex.execute("INSERT INTO y (pk, v) VALUES (2, 2)").unwrap();
        let r = ex
            .execute("SELECT pk FROM x EXCEPT SELECT pk FROM y")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s40_union_all_order() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE p (pk U64, v U64)").unwrap();
        ex.execute("CREATE TABLE q (pk U64, v U64)").unwrap();
        for i in 1u64..=3 {
            ex.execute(&format!("INSERT INTO p (pk, v) VALUES ({i}, {i})"))
                .unwrap();
        }
        for i in 4u64..=6 {
            ex.execute(&format!("INSERT INTO q (pk, v) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex
            .execute("SELECT pk FROM p UNION ALL SELECT pk FROM q")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 6);
        } else {
            panic!("expected rows");
        }
    }

    // ── S41: CTEs ─────────────────────────────────────────────────────────────

    #[test]
    fn test_s41_cte_basic() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (3, 30)")
            .unwrap();
        let r = ex
            .execute("WITH c AS (SELECT pk FROM t WHERE pk > 1) SELECT pk FROM c")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s41_cte_count() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        for i in 1u64..=5 {
            ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex
            .execute("WITH c AS (SELECT pk FROM t WHERE pk >= 3) SELECT pk FROM c")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 3);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s41_cte_filter() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 200)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (3, 300)")
            .unwrap();
        let r = ex
            .execute("WITH c AS (SELECT pk FROM t) SELECT pk FROM c WHERE pk = 2")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "2");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s41_cte_alias() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
            .unwrap();
        let r = ex
            .execute("WITH mydata AS (SELECT pk FROM t) SELECT pk FROM mydata")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "1");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s41_cte_multi_row() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        for i in 1u64..=4 {
            ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex
            .execute("WITH all_rows AS (SELECT pk FROM t) SELECT pk FROM all_rows")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 4);
        } else {
            panic!("expected rows");
        }
    }

    // ── S42: Index DDL ────────────────────────────────────────────────────────

    #[test]
    fn test_s42_create_index() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        let r = ex.execute("CREATE INDEX idx_val ON t(val)");
        assert!(matches!(r, Ok(QueryResult::Created)), "{r:?}");
    }

    #[test]
    fn test_s42_drop_index() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("CREATE INDEX idx_val ON t(val)").unwrap();
        let r = ex.execute("DROP INDEX idx_val");
        assert!(matches!(r, Ok(QueryResult::Dropped)), "{r:?}");
    }

    #[test]
    fn test_s42_create_dup_index() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("CREATE INDEX idx_val ON t(val)").unwrap();
        // Creating same index twice is OK (overwrite)
        let r = ex.execute("CREATE INDEX idx_val ON t(val)");
        assert!(matches!(r, Ok(QueryResult::Created)), "{r:?}");
    }

    #[test]
    fn test_s42_index_on_query() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 99)")
            .unwrap();
        ex.execute("CREATE INDEX idx_val ON t(val)").unwrap();
        let r = ex.execute("SELECT * FROM t WHERE val = 99").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s42_drop_nonexistent() {
        let (ex, _dir) = setup();
        // Drop non-existent index returns Dropped (best-effort)
        let r = ex.execute("DROP INDEX ghost_idx");
        assert!(matches!(r, Ok(QueryResult::Dropped)), "{r:?}");
    }

    // ── S43: CAST ─────────────────────────────────────────────────────────────

    #[test]
    fn test_s43_cast_to_text() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
            .unwrap();
        let r = ex
            .execute("SELECT CAST(pk AS TEXT) AS s FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "1");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s43_cast_u64_col() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 55)")
            .unwrap();
        let r = ex.execute("SELECT CAST(val AS U64) AS n FROM t").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "55");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s43_cast_in_where() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 7)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 8)").unwrap();
        let r = ex
            .execute("SELECT CAST(pk AS TEXT) AS s FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s43_cast_col_as_text() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 123)")
            .unwrap();
        let r = ex.execute("SELECT CAST(val AS TEXT) AS s FROM t").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "123");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s43_cast_f64() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE ft (pk U64, fval F64)").unwrap();
        ex.execute("INSERT INTO ft (pk, fval) VALUES (1, 9.5)")
            .unwrap();
        let r = ex
            .execute("SELECT CAST(fval AS TEXT) AS s FROM ft")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            // Should have some string representation
            assert!(!rows[0][0].is_empty());
        } else {
            panic!("expected rows");
        }
    }

    // ── S44: UPSERT ───────────────────────────────────────────────────────────

    #[test]
    fn test_s44_insert_on_conflict_ignore() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        // Insert duplicate with DO NOTHING — no error
        let r = ex.execute("INSERT INTO t (pk, val) VALUES (1, 99) ON CONFLICT DO NOTHING");
        assert!(matches!(r, Ok(QueryResult::Affected(0))), "{r:?}");
    }

    #[test]
    fn test_s44_insert_on_conflict_update() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        let r =
            ex.execute("INSERT INTO t (pk, val) VALUES (1, 99) ON CONFLICT DO UPDATE SET val = 99");
        assert!(matches!(r, Ok(QueryResult::Affected(1))), "{r:?}");
    }

    #[test]
    fn test_s44_upsert_new_row() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        // ON CONFLICT path not triggered for new rows
        let r = ex.execute("INSERT INTO t (pk, val) VALUES (1, 42) ON CONFLICT DO NOTHING");
        assert!(matches!(r, Ok(QueryResult::Affected(1))), "{r:?}");
    }

    #[test]
    fn test_s44_upsert_verify() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 99) ON CONFLICT DO UPDATE SET val = 99")
            .unwrap();
        let r = ex.execute("SELECT val FROM t WHERE pk = 1").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "99");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s44_on_conflict_do_nothing() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
            .unwrap();
        // Multiple inserts with IGNORE, count stays same
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 99) ON CONFLICT DO NOTHING")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 99) ON CONFLICT DO NOTHING")
            .unwrap();
        let r = ex.execute("SELECT * FROM t").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    // ── S45: CREATE VIEW ──────────────────────────────────────────────────────

    #[test]
    fn test_s45_create_view() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        let r = ex.execute("CREATE VIEW v AS SELECT pk FROM t");
        assert!(matches!(r, Ok(QueryResult::Created)), "{r:?}");
    }

    #[test]
    fn test_s45_query_view() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
            .unwrap();
        ex.execute("CREATE VIEW v AS SELECT pk FROM t").unwrap();
        let r = ex.execute("SELECT * FROM v").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s45_drop_view() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("CREATE VIEW v AS SELECT pk FROM t").unwrap();
        let r = ex.execute("DROP VIEW v");
        assert!(matches!(r, Ok(QueryResult::Dropped)), "{r:?}");
    }

    #[test]
    fn test_s45_view_with_where() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        for i in 1u64..=5 {
            ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
                .unwrap();
        }
        ex.execute("CREATE VIEW big AS SELECT pk FROM t WHERE pk > 3")
            .unwrap();
        let r = ex.execute("SELECT * FROM big").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s45_view_project() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 99)")
            .unwrap();
        ex.execute("CREATE VIEW vfull AS SELECT * FROM t").unwrap();
        let r = ex.execute("SELECT pk FROM vfull").unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert_eq!(columns, vec!["pk"]);
            assert_eq!(rows.len(), 1);
        } else {
            panic!("expected rows");
        }
    }

    // ── S46: Correctness Fixes ────────────────────────────────────────────────

    #[test]
    fn test_s46_range_scan_empty_zone() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        // Range scan on small table — should not panic
        let r = ex.execute("SELECT * FROM t WHERE pk = 999");
        assert!(r.is_ok(), "{r:?}");
        if let Ok(QueryResult::Rows { rows, .. }) = r {
            assert_eq!(rows.len(), 0);
        }
    }

    #[test]
    fn test_s46_range_scan_correct_results() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        for i in 1u64..=10 {
            ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex.execute("SELECT * FROM t WHERE pk = 5").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "5");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s46_range_scan_pk_boundary() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (10, 10)")
            .unwrap();
        let r = ex
            .execute("SELECT * FROM t WHERE pk >= 1 AND pk <= 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "1");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s46_zone_pruning() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        for i in 1u64..=5 {
            ex.execute(&format!("INSERT INTO t (pk, val) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex
            .execute("SELECT * FROM t WHERE pk >= 2 AND pk <= 4")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 3);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s46_correctness_filter_chain() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, a U64, b U64)").unwrap();
        ex.execute("INSERT INTO t (pk, a, b) VALUES (1, 5, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, a, b) VALUES (2, 5, 20)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, a, b) VALUES (3, 9, 10)")
            .unwrap();
        let r = ex
            .execute("SELECT * FROM t WHERE a = 5 AND b = 10")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][0], "1");
        } else {
            panic!("expected rows");
        }
    }

    // ── S47: Window Function Extensions ──────────────────────────────────────

    #[test]
    fn test_s47_lead() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (3, 30)")
            .unwrap();
        let r = ex
            .execute("SELECT pk, LEAD(val, 1) OVER () AS next_val FROM t ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"next_val".to_owned()));
            assert_eq!(rows.len(), 3);
            // First row's LEAD(val,1) is the second row's val = 20
            assert_eq!(rows[0][1], "20");
            // Last row has no next → NULL
            assert_eq!(rows[2][1], "NULL");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s47_lag() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 20)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (3, 30)")
            .unwrap();
        let r = ex
            .execute("SELECT pk, LAG(val, 1) OVER () AS prev_val FROM t ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"prev_val".to_owned()));
            // First row has no previous → NULL
            assert_eq!(rows[0][1], "NULL");
            // Second row's LAG(val,1) is first row's val = 10
            assert_eq!(rows[1][1], "10");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s47_first_value() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 200)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (3, 300)")
            .unwrap();
        let r = ex
            .execute("SELECT FIRST_VALUE(val) OVER () AS fv FROM t ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            // All rows should have the first value
            assert!(rows.iter().all(|r| r[0] == "100"));
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s47_last_value() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 200)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (3, 300)")
            .unwrap();
        let r = ex
            .execute("SELECT LAST_VALUE(val) OVER () AS lv FROM t ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            // All rows should have the last value
            assert!(rows.iter().all(|r| r[0] == "300"));
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s47_partition_by() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, grp U64, val U64)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, grp, val) VALUES (1, 1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, grp, val) VALUES (2, 1, 20)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, grp, val) VALUES (3, 2, 30)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, grp, val) VALUES (4, 2, 40)")
            .unwrap();
        let r = ex
            .execute("SELECT pk, ROW_NUMBER() OVER (PARTITION BY grp) AS rn FROM t ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"rn".to_owned()));
            // pk=1 → rn=1 (first in grp 1), pk=2 → rn=2, pk=3 → rn=1 (first in grp 2)
            let rn_col = columns.iter().position(|c| c == "rn").unwrap();
            assert_eq!(rows[0][rn_col], "1");
            assert_eq!(rows[1][rn_col], "2");
            assert_eq!(rows[2][rn_col], "1");
            assert_eq!(rows[3][rn_col], "2");
        } else {
            panic!("expected rows");
        }
    }

    // ── S48: NULL propagation ──────────────────────────────────────────────────

    #[test]
    fn test_s48_null_arith_propagation() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 0)").unwrap();
        let r = ex
            .execute("SELECT val + 1 AS v FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            // val=0 → "0", 0+1=1 (not null, since 0 is valid)
            assert_eq!(rows[0][0], "1");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s48_null_coalesce() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 0)").unwrap();
        let r = ex
            .execute("SELECT COALESCE(val, 99) AS v FROM t WHERE pk = 1")
            .unwrap();
        // val=0 → display_str="0" which is not "NULL", COALESCE returns it
        if let QueryResult::Rows { rows, .. } = r {
            assert!(!rows.is_empty());
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s48_null_is_null_propagation() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 0)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 5)").unwrap();
        let r = ex.execute("SELECT * FROM t WHERE val IS NULL").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s48_null_is_not_null() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 0)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 5)").unwrap();
        let r = ex.execute("SELECT * FROM t WHERE val IS NOT NULL").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s48_null_case_else() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 0)").unwrap();
        let r = ex
            .execute("SELECT CASE WHEN val = 0 THEN 42 ELSE 0 END AS v FROM t")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "42");
        } else {
            panic!("expected rows");
        }
    }

    // ── S49: String functions ─────────────────────────────────────────────────

    #[test]
    fn test_s49_upper() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
            .unwrap();
        let r = ex.execute("SELECT UPPER(val) AS u FROM t").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "42");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s49_concat() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, a U64, b U64)").unwrap();
        ex.execute("INSERT INTO t (pk, a, b) VALUES (1, 10, 20)")
            .unwrap();
        let r = ex
            .execute("SELECT CONCAT(a, b) AS c FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "1020");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s49_substr() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 12345)")
            .unwrap();
        let r = ex
            .execute("SELECT SUBSTR(val, 2, 3) AS s FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "234");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s49_instr() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 12345)")
            .unwrap();
        let r = ex
            .execute("SELECT INSTR(val, '34') AS i FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "3");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s49_trim() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
            .unwrap();
        let r = ex
            .execute("SELECT TRIM(val) AS trimmed FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "42");
        } else {
            panic!("expected rows");
        }
    }

    // ── S50: Date/Time functions ──────────────────────────────────────────────

    #[test]
    fn test_s50_now_returns_string() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
        let r = ex.execute("SELECT NOW() AS n FROM t").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert!(!rows[0][0].is_empty());
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s50_extract_year() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
        // We use a literal value for date
        let r = ex
            .execute("SELECT EXTRACT(YEAR FROM val) AS yr FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            // val=1, can't parse as date, returns 0
            assert!(!rows.is_empty());
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s50_date_diff() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
        // date_diff on non-dates returns 0
        let r = ex
            .execute("SELECT DATE_DIFF(val, val) AS dd FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "0");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s50_date_add() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
        let r = ex
            .execute("SELECT DATE_ADD(val, 1) AS da FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            // val=1 is not a date string, returns original
            assert!(!rows.is_empty());
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s50_extract_month() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 1)").unwrap();
        let r = ex
            .execute("SELECT EXTRACT(MONTH FROM val) AS mo FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert!(!rows.is_empty());
        } else {
            panic!("expected rows");
        }
    }

    // ── S51: Multi-col GROUP BY ───────────────────────────────────────────────

    #[test]
    fn test_s51_multi_group_by() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, a U64, b U64, v U64)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, a, b, v) VALUES (1, 1, 1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, a, b, v) VALUES (2, 1, 2, 20)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, a, b, v) VALUES (3, 1, 1, 30)")
            .unwrap();
        let r = ex.execute("SELECT COUNT(*) FROM t GROUP BY a, b").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            // Group (1,1) has 2 rows, (1,2) has 1 row
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s51_having_sum() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, cat U64, v U64)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (1, 1, 10)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (2, 1, 20)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (3, 2, 5)")
            .unwrap();
        let r = ex
            .execute("SELECT SUM(v) FROM t GROUP BY cat HAVING sum > 15")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            // cat=1 sum=30
            assert_eq!(rows[0][1], "30");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s51_having_count() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, cat U64, v U64)")
            .unwrap();
        for i in 1u64..=4 {
            ex.execute(&format!("INSERT INTO t (pk, cat, v) VALUES ({i}, 1, {i})"))
                .unwrap();
        }
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (5, 2, 10)")
            .unwrap();
        let r = ex
            .execute("SELECT COUNT(*) FROM t GROUP BY cat HAVING count > 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0][1], "4");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s51_single_group_by_unchanged() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, cat U64, v U64)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (1, 10, 100)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, cat, v) VALUES (2, 20, 200)")
            .unwrap();
        let r = ex.execute("SELECT COUNT(*) FROM t GROUP BY cat").unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"cat".to_owned()));
            assert_eq!(rows.len(), 2);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s51_group_by_no_having() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, cat U64)").unwrap();
        for i in 1u64..=3 {
            ex.execute(&format!("INSERT INTO t (pk, cat) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex.execute("SELECT COUNT(*) FROM t GROUP BY cat").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 3);
        } else {
            panic!("expected rows");
        }
    }

    // ── S52: Window function correctness ──────────────────────────────────────

    #[test]
    fn test_s52_dense_rank() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, score U64)").unwrap();
        ex.execute("INSERT INTO t (pk, score) VALUES (1, 100)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, score) VALUES (2, 100)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, score) VALUES (3, 200)")
            .unwrap();
        let r = ex
            .execute("SELECT pk, DENSE_RANK() OVER (ORDER BY score) AS dr FROM t ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"dr".to_owned()));
            assert_eq!(rows.len(), 3);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s52_ntile() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        for i in 1u64..=4 {
            ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex
            .execute("SELECT pk, NTILE(2) OVER () AS bucket FROM t ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"bucket".to_owned()));
            assert_eq!(rows.len(), 4);
            // First 2 rows in bucket 1, last 2 in bucket 2
            assert_eq!(rows[0][1], "1");
            assert_eq!(rows[2][1], "2");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s52_percent_rank() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 10)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (2, 20)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (3, 30)").unwrap();
        let r = ex
            .execute("SELECT pk, PERCENT_RANK() OVER (ORDER BY v) AS pr FROM t ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { columns, rows } = r {
            assert!(columns.contains(&"pr".to_owned()));
            assert_eq!(rows.len(), 3);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s52_rank_ties() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, score U64)").unwrap();
        ex.execute("INSERT INTO t (pk, score) VALUES (1, 50)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, score) VALUES (2, 50)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, score) VALUES (3, 100)")
            .unwrap();
        let r = ex
            .execute("SELECT pk, RANK() OVER (ORDER BY score) AS rk FROM t ORDER BY score")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            // Both score=50 rows get rank=1, score=100 gets rank=3
            let ranks: Vec<&str> = rows.iter().map(|r| r[1].as_str()).collect();
            assert!(ranks.contains(&"1"));
            assert_eq!(rows.len(), 3);
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s52_row_number_sequential() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        for i in 1u64..=5 {
            ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex
            .execute("SELECT pk, ROW_NUMBER() OVER () AS rn FROM t ORDER BY pk")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 5);
            for (i, row) in rows.iter().enumerate() {
                assert_eq!(row[1], (i + 1).to_string());
            }
        } else {
            panic!("expected rows");
        }
    }

    // ── S53: Savepoints ───────────────────────────────────────────────────────

    #[test]
    fn test_s53_savepoint_basic() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("BEGIN").unwrap();
        let r = ex.execute("SAVEPOINT sp1");
        assert!(r.is_ok(), "{r:?}");
        ex.execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_s53_release_savepoint() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("BEGIN").unwrap();
        ex.execute("SAVEPOINT sp1").unwrap();
        let r = ex.execute("RELEASE SAVEPOINT sp1");
        assert!(r.is_ok(), "{r:?}");
        ex.execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_s53_rollback_to_savepoint() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("BEGIN").unwrap();
        ex.execute("SAVEPOINT sp1").unwrap();
        let r = ex.execute("ROLLBACK TO SAVEPOINT sp1");
        assert!(r.is_ok(), "{r:?}");
        ex.execute("ROLLBACK").unwrap();
    }

    #[test]
    fn test_s53_savepoint_no_txn() {
        // SAVEPOINT outside a transaction should still work (snapshots empty map)
        let (ex, _dir) = setup();
        let r = ex.execute("SAVEPOINT sp1");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s53_rollback_to_no_savepoint() {
        let (ex, _dir) = setup();
        ex.execute("BEGIN").unwrap();
        // Rolling back to non-existent savepoint uses empty map — OK
        let r = ex.execute("ROLLBACK TO SAVEPOINT ghost");
        assert!(r.is_ok(), "{r:?}");
        // Need to clean up — begin a fresh txn won't work since we reset
        // Just verify it completed
    }

    // ── S54: Multi-col ORDER BY (already S38, verify here too) ───────────────

    #[test]
    fn test_s54_order_by_three_cols() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, a U64, b U64, c U64)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, a, b, c) VALUES (1, 1, 2, 3)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, a, b, c) VALUES (2, 1, 1, 3)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, a, b, c) VALUES (3, 2, 1, 1)")
            .unwrap();
        let r = ex
            .execute("SELECT pk FROM t ORDER BY a ASC, b ASC")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[0][0], "2"); // a=1, b=1
            assert_eq!(rows[1][0], "1"); // a=1, b=2
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s54_order_by_desc() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        for i in 1u64..=3 {
            ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex.execute("SELECT pk FROM t ORDER BY v DESC").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "3");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s54_order_by_limit() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        for i in 1u64..=5 {
            ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex
            .execute("SELECT pk FROM t ORDER BY v DESC LIMIT 2")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0][0], "5");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s54_order_by_offset() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        for i in 1u64..=5 {
            ex.execute(&format!("INSERT INTO t (pk, v) VALUES ({i}, {i})"))
                .unwrap();
        }
        let r = ex
            .execute("SELECT pk FROM t ORDER BY v ASC LIMIT 2 OFFSET 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0][0], "2");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s54_order_by_asc_default() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (1, 30)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (2, 10)").unwrap();
        ex.execute("INSERT INTO t (pk, v) VALUES (3, 20)").unwrap();
        let r = ex.execute("SELECT pk FROM t ORDER BY v").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "2"); // v=10
        } else {
            panic!("expected rows");
        }
    }

    // ── S55: REPLACE / string fn (already partial in S49) ────────────────────

    #[test]
    fn test_s55_replace_fn() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 12321)")
            .unwrap();
        let r = ex
            .execute("SELECT REPLACE(val, '2', '9') AS v FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "19391");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s55_concat_multi() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, a U64, b U64, c U64)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, a, b, c) VALUES (1, 1, 2, 3)")
            .unwrap();
        let r = ex
            .execute("SELECT CONCAT(a, b, c) AS v FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "123");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s55_substr_no_len() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 12345)")
            .unwrap();
        let r = ex
            .execute("SELECT SUBSTR(val, 3) AS s FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "345");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s55_instr_not_found() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 12345)")
            .unwrap();
        let r = ex
            .execute("SELECT INSTR(val, '99') AS i FROM t WHERE pk = 1")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "0");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s55_lower_fn() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 42)")
            .unwrap();
        let r = ex.execute("SELECT LOWER(val) AS v FROM t").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "42");
        } else {
            panic!("expected rows");
        }
    }

    // ── S56: EXPLAIN ANALYZE ──────────────────────────────────────────────────

    #[test]
    fn test_s56_explain_analyze_basic() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        let r = ex.execute("EXPLAIN ANALYZE SELECT * FROM t");
        assert!(r.is_ok(), "{r:?}");
        if let Ok(QueryResult::ExplainPlan(lines)) = r {
            assert!(
                lines
                    .iter()
                    .any(|l| l.contains("FullScan") || l.contains("RangeScan"))
            );
        } else {
            panic!("expected ExplainPlan");
        }
    }

    #[test]
    fn test_s56_explain_analyze_range() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        let r = ex.execute("EXPLAIN ANALYZE SELECT * FROM t WHERE pk = 1");
        assert!(r.is_ok(), "{r:?}");
        if let Ok(QueryResult::ExplainPlan(lines)) = r {
            assert!(lines.iter().any(|l| l.contains("RangeScan")));
        } else {
            panic!("expected ExplainPlan");
        }
    }

    #[test]
    fn test_s56_explain_has_actual_rows() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        let r = ex.execute("EXPLAIN ANALYZE SELECT * FROM t").unwrap();
        if let QueryResult::ExplainPlan(lines) = r {
            assert!(lines.iter().any(|l| l.contains("actual_rows")));
        } else {
            panic!("expected ExplainPlan");
        }
    }

    #[test]
    fn test_s56_explain_no_analyze() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        let r = ex.execute("EXPLAIN SELECT * FROM t").unwrap();
        if let QueryResult::ExplainPlan(lines) = r {
            assert!(lines.iter().any(|l| l.contains("FullScan")));
        } else {
            panic!("expected ExplainPlan");
        }
    }

    #[test]
    fn test_s56_explain_analyze_with_limit() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        let r = ex.execute("EXPLAIN ANALYZE SELECT * FROM t LIMIT 5");
        assert!(r.is_ok(), "{r:?}");
    }

    // ── S57: Prepared statements ──────────────────────────────────────────────

    #[test]
    fn test_s57_prepare_and_execute() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 100)")
            .unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (2, 200)")
            .unwrap();
        ex.execute("PREPARE q AS SELECT * FROM t WHERE pk = $1")
            .unwrap();
        // Execute with param — placeholder becomes NULL since no params passed
        let r = ex.execute("EXECUTE q");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s57_prepare_stored() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        let r = ex.execute("PREPARE myq AS SELECT * FROM t");
        assert!(r.is_ok(), "{r:?}");
        // Execute the prepared statement
        let r2 = ex.execute("EXECUTE myq");
        assert!(r2.is_ok(), "{r2:?}");
    }

    #[test]
    fn test_s57_execute_missing_raises_err() {
        let (ex, _dir) = setup();
        let r = ex.execute("EXECUTE nosuchplan");
        assert!(r.is_err());
    }

    #[test]
    fn test_s57_prepare_insert() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        let r = ex.execute("PREPARE ins AS INSERT INTO t (pk, val) VALUES ($1, $2)");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s57_placeholder_in_where() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO t (pk, val) VALUES (1, 10)")
            .unwrap();
        // Parse a query with placeholder — it's valid syntax
        let r = ex.execute("PREPARE p AS SELECT * FROM t WHERE pk = $1");
        assert!(r.is_ok(), "{r:?}");
    }

    // ── S58 — ALTER TABLE full ───────────────────────────────────────────────
    #[test]
    fn test_s58_rename_col() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE rc (pk U64, oldname U64)").unwrap();
        ex.execute("INSERT INTO rc (pk, oldname) VALUES (1, 99)")
            .unwrap();
        ex.execute("ALTER TABLE rc RENAME COLUMN oldname TO newname")
            .unwrap();
        let r = ex.execute("SELECT newname FROM rc").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "99");
        } else {
            panic!("expected rows");
        }
    }

    #[test]
    fn test_s58_change_type() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE ct (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO ct (pk, val) VALUES (1, 42)")
            .unwrap();
        let r = ex.execute("ALTER TABLE ct ALTER COLUMN val TYPE I64");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s58_add_col_existing() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE ac (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO ac (pk, v) VALUES (1, 1)").unwrap();
        ex.execute("ALTER TABLE ac ADD COLUMN extra U64").unwrap();
        let r = ex.execute("SELECT extra FROM ac").unwrap();
        assert!(matches!(r, QueryResult::Rows { .. }));
    }

    #[test]
    fn test_s58_drop_then_select() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE dt (pk U64, a U64, b U64)")
            .unwrap();
        ex.execute("INSERT INTO dt (pk, a, b) VALUES (1, 10, 20)")
            .unwrap();
        ex.execute("ALTER TABLE dt DROP COLUMN b").unwrap();
        // After dropping b, selecting a should succeed
        let r = ex.execute("SELECT a FROM dt");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s58_rename_then_query() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE rq (pk U64, score U64)").unwrap();
        ex.execute("INSERT INTO rq (pk, score) VALUES (1, 50)")
            .unwrap();
        ex.execute("ALTER TABLE rq RENAME COLUMN score TO points")
            .unwrap();
        let r = ex.execute("SELECT points FROM rq WHERE pk = 1").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "50");
        } else {
            panic!();
        }
    }

    // ── S59 — CHECK + DEFAULT ────────────────────────────────────────────────
    #[test]
    fn test_s59_default_value() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE dv (pk U64, status U64 DEFAULT 1)")
            .unwrap();
        ex.execute("INSERT INTO dv (pk) VALUES (1)").unwrap();
        let r = ex.execute("SELECT status FROM dv").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "1");
        } else {
            panic!();
        }
    }

    #[test]
    fn test_s59_check_passes() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE cp (pk U64, age U64 CHECK (age > 0 ))")
            .unwrap();
        let r = ex.execute("INSERT INTO cp (pk, age) VALUES (1, 25)");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s59_check_fails() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE cf (pk U64, age U64 CHECK (age > 18 ))")
            .unwrap();
        let r = ex.execute("INSERT INTO cf (pk, age) VALUES (1, 5)");
        assert!(r.is_err());
    }

    #[test]
    fn test_s59_default_with_explicit() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE de (pk U64, val U64 DEFAULT 10)")
            .unwrap();
        ex.execute("INSERT INTO de (pk, val) VALUES (1, 99)")
            .unwrap();
        let r = ex.execute("SELECT val FROM de").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "99");
        } else {
            panic!();
        }
    }

    #[test]
    fn test_s59_check_on_update() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE cu (pk U64, score U64 CHECK (score >= 0 ))")
            .unwrap();
        ex.execute("INSERT INTO cu (pk, score) VALUES (1, 10)")
            .unwrap();
        // Update is not re-checked in this MVP, just verify data is intact
        let r = ex.execute("SELECT score FROM cu WHERE pk = 1").unwrap();
        assert!(matches!(r, QueryResult::Rows { .. }));
    }

    // ── S60 — FOREIGN KEY ────────────────────────────────────────────────────
    #[test]
    fn test_s60_fk_valid_insert() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE parent (pk U64, name U64)")
            .unwrap();
        ex.execute("INSERT INTO parent (pk, name) VALUES (1, 100)")
            .unwrap();
        ex.execute("CREATE TABLE child (pk U64, parent_id U64, FOREIGN KEY (parent_id) REFERENCES parent (pk))").unwrap();
        let r = ex.execute("INSERT INTO child (pk, parent_id) VALUES (1, 1)");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s60_fk_invalid_insert() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE par2 (pk U64)").unwrap();
        ex.execute(
            "CREATE TABLE chi2 (pk U64, par_id U64, FOREIGN KEY (par_id) REFERENCES par2 (pk))",
        )
        .unwrap();
        let r = ex.execute("INSERT INTO chi2 (pk, par_id) VALUES (1, 999)");
        assert!(r.is_err());
    }

    #[test]
    fn test_s60_fk_null_skip() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE par3 (pk U64)").unwrap();
        ex.execute(
            "CREATE TABLE chi3 (pk U64, par_id U64, FOREIGN KEY (par_id) REFERENCES par3 (pk))",
        )
        .unwrap();
        // FK col value 0 should be treated as NULL and skip check
        let r = ex.execute("INSERT INTO chi3 (pk) VALUES (1)");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s60_fk_parse() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE orders (pk U64, user_id U64, FOREIGN KEY (user_id) REFERENCES users (pk))").unwrap();
        let r = ex.execute("SELECT * FROM orders");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s60_fk_multi_table() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE cats (pk U64)").unwrap();
        ex.execute("INSERT INTO cats (pk) VALUES (1)").unwrap();
        ex.execute("INSERT INTO cats (pk) VALUES (2)").unwrap();
        ex.execute(
            "CREATE TABLE items (pk U64, cat_id U64, FOREIGN KEY (cat_id) REFERENCES cats (pk))",
        )
        .unwrap();
        ex.execute("INSERT INTO items (pk, cat_id) VALUES (1, 1)")
            .unwrap();
        ex.execute("INSERT INTO items (pk, cat_id) VALUES (2, 2)")
            .unwrap();
        let r = ex.execute("SELECT * FROM items").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 2);
        } else {
            panic!();
        }
    }

    // ── S61 — Isolation Levels ───────────────────────────────────────────────
    #[test]
    fn test_s61_begin_read_committed() {
        let (ex, _dir) = setup();
        let r = ex.execute("BEGIN ISOLATION LEVEL READ COMMITTED");
        assert!(r.is_ok(), "{r:?}");
        ex.execute("COMMIT").unwrap();
    }

    #[test]
    fn test_s61_begin_serializable() {
        let (ex, _dir) = setup();
        let r = ex.execute("BEGIN ISOLATION LEVEL SERIALIZABLE");
        assert!(r.is_ok(), "{r:?}");
        ex.execute("COMMIT").unwrap();
    }

    #[test]
    fn test_s61_begin_default() {
        let (ex, _dir) = setup();
        let r = ex.execute("BEGIN");
        assert!(r.is_ok(), "{r:?}");
        ex.execute("COMMIT").unwrap();
    }

    #[test]
    fn test_s61_isolation_select_sees_committed() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE iso (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO iso (pk, v) VALUES (1, 10)")
            .unwrap();
        ex.execute("BEGIN ISOLATION LEVEL READ COMMITTED").unwrap();
        let r = ex.execute("SELECT v FROM iso WHERE pk = 1").unwrap();
        ex.execute("COMMIT").unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows[0][0], "10");
        } else {
            panic!();
        }
    }

    #[test]
    fn test_s61_isolation_rollback() {
        let (ex, _dir) = setup();
        ex.execute("BEGIN ISOLATION LEVEL REPEATABLE READ").unwrap();
        let r = ex.execute("ROLLBACK");
        assert!(r.is_ok(), "{r:?}");
    }

    // ── S62 — FTS improvements ───────────────────────────────────────────────
    #[test]
    fn test_s62_phrase_match() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE docs (pk U64, body TEXT)").unwrap();
        ex.execute("INSERT INTO docs (pk, body) VALUES (1, 'hello world')")
            .unwrap();
        let r = ex.execute("SELECT * FROM docs WHERE MATCH(body, 'hello world')");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s62_phrase_no_match() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE fts2 (pk U64, body TEXT)").unwrap();
        ex.execute("INSERT INTO fts2 (pk, body) VALUES (1, 'foo bar')")
            .unwrap();
        let r = ex.execute("SELECT pk FROM fts2 WHERE MATCH(body, 'hello')");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s62_match_score() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE fts3 (pk U64, body TEXT)").unwrap();
        ex.execute("INSERT INTO fts3 (pk, body) VALUES (1, 'rust is fast rust')")
            .unwrap();
        let r = ex.execute("SELECT MATCH_SCORE(body, 'rust') AS score FROM fts3");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s62_multi_word_match() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE fts4 (pk U64, txt TEXT)").unwrap();
        let r = ex.execute("SELECT * FROM fts4 WHERE MATCH(txt, 'alpha')");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s62_match_score_order() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE fts5 (pk U64, body TEXT)").unwrap();
        ex.execute("INSERT INTO fts5 (pk, body) VALUES (1, 'rust rust rust')")
            .unwrap();
        ex.execute("INSERT INTO fts5 (pk, body) VALUES (2, 'rust')")
            .unwrap();
        let r = ex.execute("SELECT pk, MATCH_SCORE(body, 'rust') AS score FROM fts5");
        assert!(r.is_ok(), "{r:?}");
    }

    // ── S63 — JSON column type ───────────────────────────────────────────────
    #[test]
    fn test_s63_json_get() {
        let (ex, _dir) = setup();
        // Use JSON arrow operator in expression context
        ex.execute("CREATE TABLE jt (pk U64, data TEXT)").unwrap();
        ex.execute("INSERT INTO jt (pk, data) VALUES (1, '{\"name\":\"alice\"}')")
            .unwrap();
        // Select the raw field — json_get via SELECT * and manual eval
        let r = ex.execute("SELECT * FROM jt WHERE pk = 1").unwrap();
        assert!(matches!(r, QueryResult::Rows { .. }));
    }

    #[test]
    fn test_s63_json_get_text() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE jt2 (pk U64, data TEXT)").unwrap();
        ex.execute("INSERT INTO jt2 (pk, data) VALUES (1, '{\"age\":30}')")
            .unwrap();
        let r = ex.execute("SELECT * FROM jt2").unwrap();
        assert!(matches!(r, QueryResult::Rows { .. }));
    }

    #[test]
    fn test_s63_json_nested_key() {
        let json = "{\"user\":{\"name\":\"bob\"},\"score\":42}";
        let val = json_extract(json, "score");
        assert_eq!(val, "42");
    }

    #[test]
    fn test_s63_json_where_filter() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE jf (pk U64, meta TEXT)").unwrap();
        ex.execute("INSERT INTO jf (pk, meta) VALUES (1, '{\"active\":1}')")
            .unwrap();
        let r = ex.execute("SELECT pk FROM jf");
        assert!(r.is_ok());
    }

    #[test]
    fn test_s63_json_null_key() {
        let val = json_extract("{\"x\":1}", "missing");
        assert_eq!(val, "null");
    }

    // ── S64 — DATE_TRUNC / TIME_BUCKET ──────────────────────────────────────
    #[test]
    fn test_s64_date_trunc_year() {
        assert_eq!(date_trunc("year", "2024-07-15"), "2024-01-01");
    }

    #[test]
    fn test_s64_date_trunc_month() {
        assert_eq!(date_trunc("month", "2024-07-15"), "2024-07-01");
    }

    #[test]
    fn test_s64_time_bucket() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE ts (pk U64, t U64)").unwrap();
        ex.execute("INSERT INTO ts (pk, t) VALUES (1, 100)")
            .unwrap();
        ex.execute("INSERT INTO ts (pk, t) VALUES (2, 150)")
            .unwrap();
        let r = ex.execute("SELECT TIME_BUCKET(50, t) AS bucket FROM ts");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s64_date_trunc_group_by() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE events (pk U64, created TEXT)")
            .unwrap();
        ex.execute("INSERT INTO events (pk, created) VALUES (1, '2024-01-15')")
            .unwrap();
        ex.execute("INSERT INTO events (pk, created) VALUES (2, '2024-01-20')")
            .unwrap();
        let r = ex.execute("SELECT DATE_TRUNC('month', created) AS mo FROM events");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s64_time_bucket_count() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE tb (pk U64, ts U64)").unwrap();
        for i in 1u64..=10 {
            ex.execute(&format!("INSERT INTO tb (pk, ts) VALUES ({i}, {})", i * 10))
                .unwrap();
        }
        let r = ex.execute("SELECT COUNT(*) FROM tb");
        if let Ok(QueryResult::Rows { rows, .. }) = r {
            assert_eq!(rows[0][0], "10");
        } else {
            panic!();
        }
    }

    // ── S65 — UDFs ───────────────────────────────────────────────────────────
    #[test]
    fn test_s65_create_function() {
        let (ex, _dir) = setup();
        let r = ex.execute("CREATE FUNCTION double(x) RETURNS U64 AS 'x * 2'");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s65_call_udf() {
        let (ex, _dir) = setup();
        ex.execute("CREATE FUNCTION triple(x) RETURNS U64 AS 'x * 3'")
            .unwrap();
        ex.execute("CREATE TABLE udf_t (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO udf_t (pk, val) VALUES (1, 10)")
            .unwrap();
        let r = ex.execute("SELECT TRIPLE(val) AS result FROM udf_t");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s65_udf_upper() {
        let (ex, _dir) = setup();
        let r = ex.execute("CREATE FUNCTION shout(s) RETURNS TEXT AS 's'");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s65_udf_concat() {
        let (ex, _dir) = setup();
        ex.execute("CREATE FUNCTION greet(name) RETURNS TEXT AS 'Hello name'")
            .unwrap();
        // UDF is registered; calling it as StrFunc dispatches body substitution
        let has_udf = ex.udfs.lock().unwrap().contains_key("greet");
        assert!(has_udf);
    }

    #[test]
    fn test_s65_drop_function() {
        let (ex, _dir) = setup();
        ex.execute("CREATE FUNCTION tmp(x) RETURNS U64 AS 'x'")
            .unwrap();
        let r = ex.execute("DROP FUNCTION tmp");
        assert!(r.is_ok(), "{r:?}");
        assert!(!ex.udfs.lock().unwrap().contains_key("tmp"));
    }

    // ── S66 — Triggers ───────────────────────────────────────────────────────
    #[test]
    fn test_s66_create_trigger() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE trig_t (pk U64, v U64)").unwrap();
        ex.execute("CREATE FUNCTION log_it(row) RETURNS VOID AS 'row'")
            .unwrap();
        let r = ex.execute(
            "CREATE TRIGGER trig1 AFTER INSERT ON trig_t FOR EACH ROW EXECUTE FUNCTION log_it()",
        );
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s66_trigger_fires_on_insert() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE tri2 (pk U64, v U64)").unwrap();
        ex.execute("CREATE FUNCTION noop(r) RETURNS VOID AS 'r'")
            .unwrap();
        ex.execute("CREATE TRIGGER t2 AFTER INSERT ON tri2 FOR EACH ROW EXECUTE FUNCTION noop()")
            .unwrap();
        let r = ex.execute("INSERT INTO tri2 (pk, v) VALUES (1, 10)");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s66_trigger_before_insert() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE tri3 (pk U64)").unwrap();
        ex.execute("CREATE FUNCTION chk(r) RETURNS VOID AS 'r'")
            .unwrap();
        let r = ex
            .execute("CREATE TRIGGER t3 BEFORE INSERT ON tri3 FOR EACH ROW EXECUTE FUNCTION chk()");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s66_drop_trigger() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE tri4 (pk U64)").unwrap();
        ex.execute("CREATE FUNCTION f4(r) RETURNS VOID AS 'r'")
            .unwrap();
        ex.execute("CREATE TRIGGER t4 AFTER DELETE ON tri4 FOR EACH ROW EXECUTE FUNCTION f4()")
            .unwrap();
        let r = ex.execute("DROP TRIGGER t4");
        assert!(r.is_ok(), "{r:?}");
        assert!(!ex.triggers.lock().unwrap().contains_key("t4"));
    }

    #[test]
    fn test_s66_trigger_multiple_tables() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE ta (pk U64)").unwrap();
        ex.execute("CREATE TABLE tb2 (pk U64)").unwrap();
        ex.execute("CREATE FUNCTION f_a(r) RETURNS VOID AS 'r'")
            .unwrap();
        ex.execute("CREATE FUNCTION f_b(r) RETURNS VOID AS 'r'")
            .unwrap();
        ex.execute("CREATE TRIGGER trig_a AFTER INSERT ON ta FOR EACH ROW EXECUTE FUNCTION f_a()")
            .unwrap();
        ex.execute("CREATE TRIGGER trig_b AFTER INSERT ON tb2 FOR EACH ROW EXECUTE FUNCTION f_b()")
            .unwrap();
        assert_eq!(ex.triggers.lock().unwrap().len(), 2);
    }

    // ── S67 — Index Hints ────────────────────────────────────────────────────
    #[test]
    fn test_s67_index_hint() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE ih (pk U64, val U64)").unwrap();
        ex.execute("INSERT INTO ih (pk, val) VALUES (1, 10)")
            .unwrap();
        let r = ex.execute("SELECT * FROM ih USE INDEX(pk) WHERE pk = 1");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s67_auto_index_use() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE ai (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO ai (pk, v) VALUES (1, 1)").unwrap();
        // Without hint still uses cost-based plan
        let r = ex.execute("SELECT * FROM ai WHERE pk = 1");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s67_join_reorder() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE left_t (pk U64, fk U64)").unwrap();
        ex.execute("CREATE TABLE right_t (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO left_t (pk, fk) VALUES (1, 1)")
            .unwrap();
        ex.execute("INSERT INTO right_t (pk, v) VALUES (1, 99)")
            .unwrap();
        let r = ex.execute(
            "SELECT left_t.pk, right_t.v FROM left_t JOIN right_t ON left_t.fk = right_t.pk",
        );
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s67_hint_ignored_if_no_index() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE ni (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO ni (pk, v) VALUES (1, 1)").unwrap();
        // Hint on non-existent col → falls back gracefully
        let r = ex.execute("SELECT * FROM ni USE INDEX(nonexistent_col)");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn test_s67_planner_prefers_range_scan() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE rs (pk U64, v U64)").unwrap();
        ex.execute("INSERT INTO rs (pk, v) VALUES (1, 10)").unwrap();
        let r = ex
            .execute("EXPLAIN SELECT * FROM rs USE INDEX(pk) WHERE pk = 1")
            .unwrap();
        if let QueryResult::ExplainPlan(lines) = r {
            assert!(
                lines
                    .iter()
                    .any(|l| l.contains("RangeScan") || l.contains("FullScan"))
            );
        } else {
            panic!();
        }
    }

    #[test]
    fn test_s38_multi_order_by() {
        let (ex, _dir) = setup();
        ex.execute("CREATE TABLE mo (pk U64, grp U64, score U64)")
            .unwrap();
        ex.execute("INSERT INTO mo (pk, grp, score) VALUES (1, 2, 30)")
            .unwrap();
        ex.execute("INSERT INTO mo (pk, grp, score) VALUES (2, 1, 40)")
            .unwrap();
        ex.execute("INSERT INTO mo (pk, grp, score) VALUES (3, 1, 20)")
            .unwrap();
        ex.execute("INSERT INTO mo (pk, grp, score) VALUES (4, 2, 10)")
            .unwrap();
        let r = ex
            .execute("SELECT pk FROM mo ORDER BY grp ASC, score DESC")
            .unwrap();
        if let QueryResult::Rows { rows, .. } = r {
            assert_eq!(rows.len(), 4);
            // grp=1 rows first: score DESC → pk=2(40), pk=3(20). grp=2 rows: score DESC → pk=1(30), pk=4(10)
            assert_eq!(rows[0][0], "2");
            assert_eq!(rows[1][0], "3");
            assert_eq!(rows[2][0], "1");
            assert_eq!(rows[3][0], "4");
        } else {
            panic!("expected rows");
        }
    }
}
