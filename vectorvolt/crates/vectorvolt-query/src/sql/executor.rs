use std::sync::Arc;

use thiserror::Error;
use vectorvolt_engine::{ColumnDef, ColumnType, EngineRow, TableRegistry, TableSchema, TypedValue};

use crate::sql::ast::*;
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
}

pub struct SqlExecutor {
    registry: Arc<TableRegistry>,
}

impl SqlExecutor {
    pub fn new(registry: Arc<TableRegistry>) -> Self {
        Self { registry }
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
        }
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

        // Get all rows via scan on PK column (col 0), then verify via get() to
        // exclude rows deleted since the last compaction (tombstones).
        let all_rows = self.scan_live(&engine)?;

        // Filter by WHERE
        let filtered = self.filter_rows(all_rows, stmt.where_clause.as_ref(), schema)?;

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
        engine
            .insert(pk, row_vals)
            .map_err(|e| ExecError::Engine(e.to_string()))?;

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

        let to_delete = self.filter_rows(all_rows, stmt.where_clause.as_ref(), schema)?;
        let count = to_delete.len() as u64;

        for row in to_delete {
            engine
                .delete(row.pk)
                .map_err(|e| ExecError::Engine(e.to_string()))?;
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
    ) -> Result<Vec<EngineRow>, ExecError> {
        match where_clause {
            None => Ok(rows),
            Some(expr) => rows
                .into_iter()
                .filter_map(|row| match self.eval_expr(expr, &row, schema) {
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
    ) -> Result<bool, ExecError> {
        match expr {
            Expr::BinOp { op, left, right } => {
                match op {
                    BinOp::And => {
                        Ok(self.eval_expr(left, row, schema)?
                            && self.eval_expr(right, row, schema)?)
                    }
                    BinOp::Or => {
                        Ok(self.eval_expr(left, row, schema)?
                            || self.eval_expr(right, row, schema)?)
                    }
                    _ => {
                        // Comparison
                        let lv = self.eval_value(left, row, schema)?;
                        let rv = self.eval_value(right, row, schema)?;
                        Ok(compare_values(&lv, op, &rv))
                    }
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
            Expr::BinOp { .. } => Err(ExecError::TypeError(
                "nested binop in value position".into(),
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
}
