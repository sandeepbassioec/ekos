//! Cost-based SQL query planner (S27, VV-QRY-040..045).
//!
//! Converts a [`SelectStmt`] into a [`PhysicalPlan`] tree. The executor drives
//! the tree top-down; EXPLAIN serialises it to a human-readable string.
//!
//! ## Cost model (v1 — row-count heuristic)
//!
//! | Scan | Estimated cost |
//! |------|----------------|
//! | FullScan | `table_rows` |
//! | RangeScan | `table_rows × selectivity` (0.1 for point, 0.5 for range) |
//! | VecScan | `table_rows` (brute-force; HNSW is S29) |
//!
//! The planner picks the lowest-cost scan. If a RangeScan exists on the
//! primary-key column its selectivity drops to 0.01 (PK is unique).

use std::collections::HashMap;

use vectorvolt_engine::TableSchema;

use crate::sql::ast::{BinOp, Expr, LitVal, OrderBy, SelectStmt};

// ── PhysicalPlan ─────────────────────────────────────────────────────────────

/// A physical plan node. Plans form a tree: leaves produce rows, inner nodes
/// transform or combine them.
#[derive(Debug, Clone)]
pub enum PhysicalPlan {
    /// Read all rows from a table.
    FullScan { table: String, est_rows: u64 },
    /// Read rows whose `col_idx` value falls in `[lo, hi]`.
    RangeScan {
        table: String,
        col_idx: usize,
        col_name: String,
        lo: u64,
        hi: u64,
        est_rows: u64,
    },
    /// Apply a WHERE predicate to rows from `input`.
    Filter {
        input: Box<PhysicalPlan>,
        predicate: Expr,
    },
    /// Emit only the named columns from `input`.
    Project {
        input: Box<PhysicalPlan>,
        columns: Vec<String>,
    },
    /// Sort rows from `input` by a plain column.
    Sort {
        input: Box<PhysicalPlan>,
        col: String,
        asc: bool,
    },
    /// Sort rows from `input` by vector distance (brute-force).
    VecSort {
        input: Box<PhysicalPlan>,
        col: String,
        query: Vec<f32>,
    },
    /// Limit output to at most `n` rows.
    Limit { input: Box<PhysicalPlan>, n: u64 },
}

impl PhysicalPlan {
    /// Estimated row count for this node.
    pub fn est_rows(&self) -> u64 {
        match self {
            Self::FullScan { est_rows, .. } => *est_rows,
            Self::RangeScan { est_rows, .. } => *est_rows,
            Self::Filter { input, .. } => input.est_rows() / 4, // rough filter selectivity
            Self::Project { input, .. } => input.est_rows(),
            Self::Sort { input, .. } => input.est_rows(),
            Self::VecSort { input, .. } => input.est_rows(),
            Self::Limit { n, .. } => *n,
        }
    }

    /// Render as indented EXPLAIN lines (depth-first pre-order).
    pub fn explain(&self, depth: usize) -> Vec<String> {
        let pad = "  ".repeat(depth);
        let mut lines = Vec::new();
        match self {
            Self::FullScan { table, est_rows } => {
                lines.push(format!("{pad}FullScan   table={table} est={est_rows}"));
            }
            Self::RangeScan {
                table,
                col_name,
                lo,
                hi,
                est_rows,
                ..
            } => {
                lines.push(format!(
                    "{pad}RangeScan  table={table} col={col_name} lo={lo} hi={hi} est={est_rows}"
                ));
            }
            Self::Filter { input, .. } => {
                lines.push(format!("{pad}Filter     WHERE <expr>"));
                lines.extend(input.explain(depth + 1));
            }
            Self::Project { input, columns } => {
                lines.push(format!("{pad}Project    {}", columns.join(", ")));
                lines.extend(input.explain(depth + 1));
            }
            Self::Sort { input, col, asc } => {
                lines.push(format!(
                    "{pad}Sort       {col} {}",
                    if *asc { "ASC" } else { "DESC" }
                ));
                lines.extend(input.explain(depth + 1));
            }
            Self::VecSort { input, col, .. } => {
                lines.push(format!("{pad}VecSort    vec_distance({col}) ASC"));
                lines.extend(input.explain(depth + 1));
            }
            Self::Limit { input, n } => {
                lines.push(format!("{pad}Limit      {n}"));
                lines.extend(input.explain(depth + 1));
            }
        }
        lines
    }
}

// ── Planner ──────────────────────────────────────────────────────────────────

/// Statistics hint passed to the planner (row count per table).
pub type TableStats = HashMap<String, u64>;

/// Converts a [`SelectStmt`] into a [`PhysicalPlan`].
pub struct Planner<'a> {
    schema: &'a TableSchema,
    stats: u64, // estimated table row count
}

impl<'a> Planner<'a> {
    pub fn new(schema: &'a TableSchema, est_table_rows: u64) -> Self {
        Self {
            schema,
            stats: est_table_rows,
        }
    }

    pub fn plan(&self, stmt: &SelectStmt) -> PhysicalPlan {
        // ── Choose scan ────────────────────────────────────────────────────
        let scan = self.choose_scan(&stmt.table, stmt.where_clause.as_ref());

        // ── Filter (only when WHERE isn't fully covered by the scan) ───────
        let after_filter = if let Some(expr) = &stmt.where_clause {
            // Always wrap in Filter — the executor re-checks all rows.
            // (A future optimizer could elide for pure-pushdown predicates.)
            PhysicalPlan::Filter {
                input: Box::new(scan),
                predicate: expr.clone(),
            }
        } else {
            scan
        };

        // ── Project ────────────────────────────────────────────────────────
        let col_names: Vec<String> = stmt
            .columns
            .iter()
            .flat_map(|c| match c {
                crate::sql::ast::SelectExpr::Star => self
                    .schema
                    .columns()
                    .iter()
                    .map(|cd| cd.name.clone())
                    .collect::<Vec<_>>(),
                crate::sql::ast::SelectExpr::Col(name) => vec![name.clone()],
                crate::sql::ast::SelectExpr::QualifiedCol { col, .. } => vec![col.clone()],
                crate::sql::ast::SelectExpr::Subquery { alias, .. } => vec![alias.clone()],
                crate::sql::ast::SelectExpr::WindowFunc { func, .. } => vec![func.col_name()],
                crate::sql::ast::SelectExpr::Computed { alias, .. } => vec![alias.clone()],
                crate::sql::ast::SelectExpr::Agg { func, col } => {
                    use crate::sql::ast::AggFunc;
                    let label = match func {
                        AggFunc::Count => "count".to_owned(),
                        AggFunc::Sum => format!("sum({})", col.as_deref().unwrap_or("?")),
                        AggFunc::Min => format!("min({})", col.as_deref().unwrap_or("?")),
                        AggFunc::Max => format!("max({})", col.as_deref().unwrap_or("?")),
                    };
                    vec![label]
                }
            })
            .collect();
        let after_project = PhysicalPlan::Project {
            input: Box::new(after_filter),
            columns: col_names,
        };

        // ── Sort ───────────────────────────────────────────────────────────
        let after_sort = match &stmt.order_by {
            Some(OrderBy::Col { name, asc }) => PhysicalPlan::Sort {
                input: Box::new(after_project),
                col: name.clone(),
                asc: *asc,
            },
            Some(OrderBy::VecDistance { col, query }) => PhysicalPlan::VecSort {
                input: Box::new(after_project),
                col: col.clone(),
                query: query.clone(),
            },
            None => after_project,
        };

        // ── Limit ──────────────────────────────────────────────────────────
        if let Some(n) = stmt.limit {
            PhysicalPlan::Limit {
                input: Box::new(after_sort),
                n,
            }
        } else {
            after_sort
        }
    }

    /// Choose between FullScan and RangeScan using a cost heuristic.
    fn choose_scan(&self, table: &str, where_clause: Option<&Expr>) -> PhysicalPlan {
        if let Some(expr) = where_clause
            && let Some((col_idx, lo, hi)) = self.try_pushdown(expr)
        {
            let col_name = self
                .schema
                .get(col_idx)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            let is_pk = col_idx == 0;
            let selectivity: f64 = if lo == hi {
                if is_pk { 0.001 } else { 0.01 } // point lookup
            } else {
                if is_pk { 0.1 } else { 0.5 } // range
            };
            let range_cost = (self.stats as f64 * selectivity).max(1.0) as u64;
            // RangeScan is always at least as good as FullScan — prefer it whenever
            // a pushdown predicate exists (even on empty/tiny tables).
            return PhysicalPlan::RangeScan {
                table: table.to_owned(),
                col_idx,
                col_name,
                lo,
                hi,
                est_rows: range_cost,
            };
        }
        PhysicalPlan::FullScan {
            table: table.to_owned(),
            est_rows: self.stats,
        }
    }

    /// Extract a single-column range predicate suitable for engine pushdown.
    /// Returns `(col_idx, lo, hi)` or `None`.
    pub fn try_pushdown(&self, expr: &Expr) -> Option<(usize, u64, u64)> {
        match expr {
            Expr::BinOp { op, left, right } => {
                if let (Expr::Col(name), Expr::Lit(LitVal::Int(n))) =
                    (left.as_ref(), right.as_ref())
                    && let Some((idx, _)) = self.schema.find(name)
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
                // AND of two comparisons on same column → range
                if *op == BinOp::And
                    && let (Some((i1, lo, _)), Some((i2, _, hi))) =
                        (self.try_pushdown(left), self.try_pushdown(right))
                    && i1 == i2
                {
                    return Some((i1, lo, hi));
                }
                None
            }
            _ => None,
        }
    }
}
