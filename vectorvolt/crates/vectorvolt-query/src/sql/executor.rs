use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use thiserror::Error;
use vectorvolt_engine::{
    ColumnDef, ColumnType, EngineRow, TableRegistry, TableSchema, TxnId, TypedValue,
};

use crate::sql::ast::*;
use crate::sql::fts::TextStore;
use crate::sql::lexer::{LexError, Lexer};
use crate::sql::parser::{ParseError, SqlParser};

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
            Stmt::Delete(s) => self.exec_delete(s),
            Stmt::CreateTable(s) => self.exec_create_table(s),
            Stmt::DropTable(s) => self.exec_drop_table(s),
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

        // Determine output columns
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
            })
            .collect();

        // Use range scan (pushdown) when WHERE is a simple single-column predicate;
        // fall back to full scan otherwise.
        let all_rows = if let Some(expr) = &stmt.where_clause
            && let Some((col_idx, lo, hi)) = Self::try_pushdown(expr, schema)
        {
            engine
                .flush()
                .map_err(|e| ExecError::Engine(e.to_string()))?;
            engine
                .scan(col_idx, lo, hi)
                .map_err(|e| ExecError::Engine(e.to_string()))?
        } else {
            self.scan_live(&engine)?
        };

        // Filter by WHERE (still needed for non-pushdown predicates or compound exprs)
        let filtered =
            self.filter_rows(all_rows, stmt.where_clause.as_ref(), schema, &stmt.table)?;

        // Sort by ORDER BY
        let mut sorted = filtered;
        if let Some((col_name, asc)) = &stmt.order_by
            && let Some((idx, col_def)) = schema.find(col_name)
        {
            let col_type = col_def.col_type;
            let asc = *asc;
            sorted.sort_by(|a, b| {
                let va = TypedValue::from_bits(col_type, a.values.get(idx).copied().unwrap_or(0));
                let vb = TypedValue::from_bits(col_type, b.values.get(idx).copied().unwrap_or(0));
                let cmp = va.display_str().cmp(&vb.display_str());
                if asc { cmp } else { cmp.reverse() }
            });
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

        // Build output
        let out_col_names: Vec<String> = out_cols.iter().map(|(n, _)| n.clone()).collect();
        let rows: Vec<Vec<String>> = final_rows
            .iter()
            .map(|row| {
                out_cols
                    .iter()
                    .map(|(_, idx)| {
                        let bits = row.values.get(*idx).copied().unwrap_or(0);
                        let col_type = schema
                            .get(*idx)
                            .map(|c| c.col_type)
                            .unwrap_or(ColumnType::U64);
                        TypedValue::from_bits(col_type, bits).display_str()
                    })
                    .collect()
            })
            .collect();

        Ok(QueryResult::Rows {
            columns: out_col_names,
            rows,
        })
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
        } else {
            engine
                .insert(pk, row_vals)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
        }

        // Persist text values for TEXT columns into the FTS sidecar.
        for (col_name, expr) in stmt.columns.iter().zip(stmt.values.iter()) {
            if let Some((_, col_def)) = schema.find(col_name)
                && col_def.col_type == ColumnType::Text
                && let Expr::Lit(LitVal::Str(text)) = expr
            {
                let base = self.registry.base_dir();
                if let Ok(mut store) = TextStore::load(base, &stmt.table, col_name) {
                    store.insert(pk, text.clone());
                    let _ = store.save();
                }
            }
        }

        Ok(QueryResult::Affected(1))
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
    ///   `col OP lit` or `col OP lit AND col OP lit` (same column, collapsible to range).
    fn try_pushdown(expr: &Expr, schema: &TableSchema) -> Option<(usize, u64, u64)> {
        match expr {
            Expr::BinOp { op, left, right } => {
                // Single comparison: col OP lit
                if let (Expr::Col(name), Expr::Lit(LitVal::Int(n))) =
                    (left.as_ref(), right.as_ref())
                    && let Some((idx, _)) = schema.find(name)
                {
                    let v = *n as u64;
                    return match op {
                        BinOp::Eq => Some((idx, v, v)),
                        BinOp::Gte => Some((idx, v, u64::MAX)),
                        BinOp::Gt => Some((idx, v.saturating_add(1), u64::MAX)),
                        BinOp::Lte => Some((idx, 0, v)),
                        BinOp::Lt => Some((idx, 0, v.saturating_sub(1))),
                        _ => None,
                    };
                }
                // AND of two comparisons on the same column → range
                if *op == BinOp::And
                    && let (Some((i1, lo, _)), Some((i2, _, hi))) = (
                        Self::try_pushdown(left, schema),
                        Self::try_pushdown(right, schema),
                    )
                    && i1 == i2
                {
                    return Some((i1, lo, hi));
                }
                None
            }
            _ => None,
        }
    }

    /// Build and return an EXPLAIN plan for a SELECT without executing it.
    fn explain_select(&self, stmt: SelectStmt) -> Result<QueryResult, ExecError> {
        let engine = self
            .registry
            .get(&stmt.table)
            .map_err(|_| ExecError::TableNotFound(stmt.table.clone()))?;
        let schema = &engine.schema;

        let mut plan: Vec<String> = Vec::new();

        // Scan node
        if let Some(expr) = &stmt.where_clause
            && let Some((col_idx, lo, hi)) = Self::try_pushdown(expr, schema)
        {
            let col_name = schema.get(col_idx).map(|c| c.name.as_str()).unwrap_or("?");
            plan.push(format!(
                "RangeScan  table={} col={} lo={} hi={}",
                stmt.table, col_name, lo, hi
            ));
        } else {
            plan.push(format!("FullScan   table={}", stmt.table));
        }

        // Filter node (always shown when WHERE exists)
        if stmt.where_clause.is_some() {
            plan.push("Filter     WHERE <expr>".to_string());
        }

        // Project node
        let cols: Vec<String> = stmt
            .columns
            .iter()
            .map(|c| match c {
                SelectExpr::Star => "*".to_string(),
                SelectExpr::Col(n) => n.clone(),
            })
            .collect();
        plan.push(format!("Project    {}", cols.join(", ")));

        // Sort node
        if let Some((col, asc)) = &stmt.order_by {
            plan.push(format!(
                "Sort       {} {}",
                col,
                if *asc { "ASC" } else { "DESC" }
            ));
        }

        // Limit node
        if let Some(n) = stmt.limit {
            plan.push(format!("Limit      {n}"));
        }

        Ok(QueryResult::ExplainPlan(plan))
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
            Expr::BinOp { .. } | Expr::Match { .. } => Err(ExecError::TypeError(
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
                assert!(lines[0].contains("FullScan"));
                assert!(lines[0].contains("table=t"));
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
                assert!(lines[0].contains("RangeScan"), "got: {}", lines[0]);
                assert!(lines[0].contains("col=pk"));
                assert!(lines[0].contains("lo=5"));
                assert!(lines[0].contains("hi=5"));
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
}
