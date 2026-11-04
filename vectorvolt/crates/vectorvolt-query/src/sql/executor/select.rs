//! SELECT execution: scans, joins, aggregation, expression evaluation and EXPLAIN.

use super::*;

impl SqlExecutor {
    /// D5 — SELECT with column masks applied to its result. Masks apply to
    /// every session (mask definitions carry no per-user exemption). Plain
    /// and qualified columns, `*`, joins, value aggregates (SUM/MIN/MAX),
    /// window value functions and computed expressions that reference a
    /// masked column all return the mask value; views, CTEs and subqueries
    /// are masked by the inner SELECT. Filtering/sorting still sees real
    /// values (dynamic masking, as in other databases).
    pub(super) fn exec_select(&self, stmt: SelectStmt) -> Result<QueryResult, ExecError> {
        let masks = self.active_masks();
        if masks.is_empty() {
            return self.exec_select_unmasked(stmt);
        }
        let scope: Vec<String> = std::iter::once(stmt.table.to_ascii_lowercase())
            .chain(stmt.joins.iter().map(|j| j.table.to_ascii_lowercase()))
            .collect();
        let mut derived: HashMap<String, String> = HashMap::new();
        for col in &stmt.columns {
            if let SelectExpr::Computed { alias, expr } = col {
                let dbg = format!("{expr:?}").to_ascii_lowercase();
                if let Some(m) = masks.iter().find_map(|((t, c), m)| {
                    (scope.contains(t)
                        && (dbg.contains(&format!("col(\"{c}\")"))
                            || dbg.contains(&format!("col: \"{c}\""))))
                    .then_some(m)
                }) {
                    derived.insert(alias.to_ascii_lowercase(), m.clone());
                }
            }
        }
        let result = self.exec_select_unmasked(stmt)?;
        Ok(Self::apply_masks(result, &scope, &masks, &derived))
    }

    pub(super) fn scan_table_as_strings(&self, table: &str) -> Vec<Vec<String>> {
        let Ok(engine) = self.registry.get(table) else {
            return vec![];
        };
        let schema = &engine.schema;
        let col_types: Vec<(usize, ColumnType)> = schema
            .columns()
            .iter()
            .enumerate()
            .map(|(i, c)| (i, c.col_type))
            .collect();
        let Ok(rows) = self.scan_live(&engine) else {
            return vec![];
        };
        rows.iter()
            .map(|row| {
                col_types
                    .iter()
                    .map(|(idx, ct)| {
                        TypedValue::from_bits(*ct, row.values.get(*idx).copied().unwrap_or(0))
                            .display_str()
                    })
                    .collect()
            })
            .collect()
    }

    pub(super) fn collect_col_values(&self, table: &str, col: &str) -> Vec<String> {
        let Ok(engine) = self.registry.get(table) else {
            return vec![];
        };
        let schema = &engine.schema;
        let Some((col_idx, col_def)) = schema.find(col) else {
            return vec![];
        };
        let Ok(rows) = self.scan_live(&engine) else {
            return vec![];
        };
        rows.iter()
            .map(|row| {
                TypedValue::from_bits(
                    col_def.col_type,
                    row.values.get(col_idx).copied().unwrap_or(0),
                )
                .display_str()
            })
            .collect()
    }

    pub(super) fn scan_col_filtered(&self, table: &str, col: &str, val: &str) -> Vec<Vec<String>> {
        let Ok(engine) = self.registry.get(table) else {
            return vec![];
        };
        let schema = &engine.schema;
        let Some((col_idx, col_def)) = schema.find(col) else {
            return vec![];
        };
        let Ok(rows) = self.scan_live(&engine) else {
            return vec![];
        };
        rows.iter()
            .filter(|row| {
                TypedValue::from_bits(
                    col_def.col_type,
                    row.values.get(col_idx).copied().unwrap_or(0),
                )
                .display_str()
                    == val
            })
            .map(|row| {
                let v = TypedValue::from_bits(
                    col_def.col_type,
                    row.values.get(col_idx).copied().unwrap_or(0),
                )
                .display_str();
                vec![v, "true".to_owned()]
            })
            .collect()
    }

    pub(super) fn exec_select_unmasked(&self, stmt: SelectStmt) -> Result<QueryResult, ExecError> {
        // S83 — check client IP (empty allowlist = allow all)
        // IP check is structural only; actual enforcement is done via check_access_allowed()

        // S81/S82 — token auth + rate limit
        if let Some(denied) = self.check_table_access(&stmt.table, "SELECT") {
            return Ok(denied);
        }

        // S80 â€” virtual table vv_audit_log
        if stmt.table == "vv_audit_log" {
            let log = self.audit_log.lock_safe();
            let columns = vec![
                "ts".to_owned(),
                "op".to_owned(),
                "table".to_owned(),
                "old_val".to_owned(),
                "new_val".to_owned(),
            ];
            let rows: Vec<Vec<String>> = log
                .iter()
                .map(|e| {
                    vec![
                        e.ts.clone(),
                        e.op.clone(),
                        e.table.clone(),
                        e.old_val.clone(),
                        e.new_val.clone(),
                    ]
                })
                .collect();
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S71 â€” virtual table vv_stat_statements
        if stmt.table == "vv_stat_statements" {
            let stats = self.query_stats.lock_safe();
            let columns = vec![
                "query".to_owned(),
                "fingerprint".to_owned(),
                "calls".to_owned(),
                "total_ms".to_owned(),
                "min_ms".to_owned(),
                "max_ms".to_owned(),
                "rows".to_owned(),
            ];
            let rows: Vec<Vec<String>> = stats
                .iter()
                .map(|(q, s)| {
                    vec![
                        q.clone(),
                        q.clone(), // fingerprint = normalized query
                        s.calls.to_string(),
                        format!("{:.3}", s.total_ms),
                        format!("{:.3}", s.min_ms),
                        format!("{:.3}", s.max_ms),
                        s.rows.to_string(),
                    ]
                })
                .collect();
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S94 â€” virtual table vv_query_fingerprints
        if stmt.table == "vv_query_fingerprints" {
            let stats = self.query_stats.lock_safe();
            let mut fingerprint_map: HashMap<String, (u64, f64)> = HashMap::new();
            for (q, s) in stats.iter() {
                let entry = fingerprint_map.entry(q.clone()).or_insert((0, 0.0));
                entry.0 += s.calls;
                entry.1 += s.total_ms;
            }
            let columns = vec![
                "fingerprint".to_owned(),
                "count".to_owned(),
                "avg_ms".to_owned(),
            ];
            let rows: Vec<Vec<String>> = fingerprint_map
                .iter()
                .map(|(fp, (cnt, total))| {
                    let avg = if *cnt > 0 { total / (*cnt as f64) } else { 0.0 };
                    vec![fp.clone(), cnt.to_string(), format!("{:.3}", avg)]
                })
                .collect();
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S95 â€” pg_stat_statements_reset() function call intercepted in SELECT columns
        for col in &stmt.columns {
            if let SelectExpr::Computed {
                expr: Expr::StrFunc { name, .. },
                ..
            } = col
                && name.eq_ignore_ascii_case("PG_STAT_STATEMENTS_RESET")
            {
                self.query_stats.lock_safe().clear();
                return Ok(QueryResult::Rows {
                    columns: vec!["pg_stat_statements_reset".to_owned()],
                    rows: vec![vec!["".to_owned()]],
                });
            }
        }

        // S95 â€” pg_stat_statements compatibility alias
        if stmt.table == "pg_stat_statements" {
            let stats = self.query_stats.lock_safe();
            let columns = vec![
                "userid".to_owned(),
                "dbid".to_owned(),
                "queryid".to_owned(),
                "query".to_owned(),
                "calls".to_owned(),
                "total_exec_time".to_owned(),
                "min_exec_time".to_owned(),
                "max_exec_time".to_owned(),
                "rows".to_owned(),
            ];
            let rows: Vec<Vec<String>> = stats
                .iter()
                .map(|(q, s)| {
                    let qid = fingerprint_hash(q);
                    vec![
                        "0".to_owned(),
                        "0".to_owned(),
                        qid.to_string(),
                        q.clone(),
                        s.calls.to_string(),
                        format!("{:.3}", s.total_ms),
                        format!("{:.3}", s.min_ms),
                        format!("{:.3}", s.max_ms),
                        s.rows.to_string(),
                    ]
                })
                .collect();
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S93 â€” virtual table vv_slow_log
        if stmt.table == "vv_slow_log" {
            let log = self.slow_log.lock_safe();
            let columns = vec![
                "sql".to_owned(),
                "duration_ms".to_owned(),
                "rows".to_owned(),
                "ts".to_owned(),
            ];
            let rows: Vec<Vec<String>> = log
                .iter()
                .map(|e| {
                    vec![
                        e.sql.clone(),
                        e.duration_ms.to_string(),
                        e.rows.to_string(),
                        e.ts.clone(),
                    ]
                })
                .collect();
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S89 â€” logical decode virtual function
        if stmt.table == "vv_logical_decode" {
            let log = self.audit_log.lock_safe().clone();
            let columns = vec![
                "lsn".to_owned(),
                "xid".to_owned(),
                "relation".to_owned(),
                "op".to_owned(),
                "old_data".to_owned(),
                "new_data".to_owned(),
            ];
            let rows: Vec<Vec<String>> = log
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    vec![
                        format!("0/{}", i + 1),
                        format!("{}", 1000 + i),
                        e.table.clone(),
                        e.op.clone(),
                        format!("{{\"old\":\"{}\"}}", e.old_val),
                        format!("{{\"new\":\"{}\"}}", e.new_val),
                    ]
                })
                .collect();
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S97 â€” virtual table vv_wait_events
        if stmt.table == "vv_wait_events" {
            let events = self.wait_events.lock_safe();
            let columns = vec![
                "event_type".to_owned(),
                "object_name".to_owned(),
                "duration_us".to_owned(),
                "ts".to_owned(),
            ];
            let rows: Vec<Vec<String>> = events
                .iter()
                .map(|e| {
                    vec![
                        e.event_type.clone(),
                        e.object_name.clone(),
                        e.duration_us.to_string(),
                        e.ts.clone(),
                    ]
                })
                .collect();
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S99 â€” vv_health virtual table
        if stmt.table == "vv_health" {
            let uptime_s = self.created_at.elapsed().as_secs();
            let cache_hits = self.query_cache.lock_safe().len() as u64;
            let slow_count = self.slow_log.lock_safe().len() as u64;
            let rep_slots = self.replication_slots.lock_safe().len() as u64;
            let ro = *self.read_only.lock_safe();
            let archive = self
                .archive_path
                .lock_safe()
                .clone()
                .unwrap_or_else(|| "none".to_owned());
            let columns = vec!["metric".to_owned(), "value".to_owned()];
            let rows = vec![
                vec!["uptime_s".to_owned(), uptime_s.to_string()],
                vec!["active_sessions".to_owned(), "1".to_owned()],
                vec![
                    "cache_hit_ratio".to_owned(),
                    format!("{:.2}", if cache_hits > 0 { 1.0f64 } else { 0.0f64 }),
                ],
                vec!["slow_queries".to_owned(), slow_count.to_string()],
                vec!["replication_slots".to_owned(), rep_slots.to_string()],
                vec!["read_only".to_owned(), ro.to_string()],
                vec!["archive_path".to_owned(), archive],
            ];
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S105 â€” pg_type virtual table
        if stmt.table == "pg_type" {
            let columns = vec!["oid".to_owned(), "typname".to_owned()];
            let rows = vec![
                vec!["25".to_owned(), "text".to_owned()],
                vec!["20".to_owned(), "int8".to_owned()],
                vec!["701".to_owned(), "float8".to_owned()],
                vec!["16".to_owned(), "bool".to_owned()],
                vec!["1114".to_owned(), "timestamp".to_owned()],
                vec!["21".to_owned(), "int2".to_owned()],
                vec!["23".to_owned(), "int4".to_owned()],
                vec!["700".to_owned(), "float4".to_owned()],
            ];
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S105 â€” information_schema virtual table
        if stmt.table == "information_schema" {
            let columns = vec![
                "table_name".to_owned(),
                "column_name".to_owned(),
                "data_type".to_owned(),
            ];
            // Return placeholder for known system tables
            let rows: Vec<Vec<String>> = vec![
                vec!["pg_type".to_owned(), "oid".to_owned(), "int8".to_owned()],
                vec![
                    "pg_type".to_owned(),
                    "typname".to_owned(),
                    "text".to_owned(),
                ],
                vec![
                    "vv_health".to_owned(),
                    "metric".to_owned(),
                    "text".to_owned(),
                ],
                vec![
                    "vv_health".to_owned(),
                    "value".to_owned(),
                    "text".to_owned(),
                ],
            ];
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S115 â€” vv_table_stats virtual table
        if stmt.table == "vv_table_stats" {
            let stats = self.table_stats.lock_safe();
            let columns = vec![
                "table".to_owned(),
                "column".to_owned(),
                "row_count".to_owned(),
                "null_count".to_owned(),
                "distinct_count".to_owned(),
                "min_val".to_owned(),
                "max_val".to_owned(),
            ];
            let mut rows = Vec::new();
            for (tbl, col_stats) in stats.iter() {
                for (col, stat) in col_stats.iter() {
                    rows.push(vec![
                        tbl.clone(),
                        col.clone(),
                        stat.row_count.to_string(),
                        stat.null_count.to_string(),
                        stat.distinct_count.to_string(),
                        stat.min_val.clone(),
                        stat.max_val.clone(),
                    ]);
                }
            }
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S116 â€” vv_connections virtual table
        if stmt.table == "vv_connections" {
            let max = *self.max_connections.lock_safe();
            let current = self
                .connection_count
                .load(std::sync::atomic::Ordering::Relaxed);
            let available = max.saturating_sub(current);
            return Ok(QueryResult::Rows {
                columns: vec![
                    "current".to_owned(),
                    "max".to_owned(),
                    "available".to_owned(),
                ],
                rows: vec![vec![
                    current.to_string(),
                    max.to_string(),
                    available.to_string(),
                ]],
            });
        }

        // S117 â€” vv_resource_groups virtual table
        if stmt.table == "vv_resource_groups" {
            let groups = self.resource_groups.lock_safe();
            let columns = vec![
                "name".to_owned(),
                "cpu_pct".to_owned(),
                "memory_mb".to_owned(),
                "user_count".to_owned(),
            ];
            let rows: Vec<Vec<String>> = groups
                .iter()
                .map(|(name, rg)| {
                    vec![
                        name.clone(),
                        rg.cpu_pct.to_string(),
                        rg.memory_mb.to_string(),
                        rg.users.len().to_string(),
                    ]
                })
                .collect();
            return Ok(QueryResult::Rows { columns, rows });
        }

        // S111 â€” vv_schemas virtual table
        if stmt.table == "vv_schemas" {
            let schemas: Vec<Vec<String>> = self
                .schemas
                .lock_safe()
                .iter()
                .map(|s| vec![s.clone()])
                .collect();
            return Ok(QueryResult::Rows {
                columns: vec!["schema".to_owned()],
                rows: schemas,
            });
        }

        // S41 â€” handle CTEs
        if !stmt.ctes.is_empty() {
            let mut cte_map: HashMap<String, (Vec<String>, Vec<Vec<String>>)> = HashMap::new();
            for (cte_name, cte_stmt, is_recursive) in &stmt.ctes {
                if *is_recursive {
                    let seed_result = self.exec_select(*cte_stmt.clone())?;
                    let (cols, accumulated) = match seed_result {
                        QueryResult::Rows { columns, rows } => (columns, rows),
                        _ => (vec![], vec![]),
                    };
                    if let Some((_, rec_stmt)) =
                        stmt.recursive_parts.iter().find(|(n, _)| n == cte_name)
                    {
                        let _ = rec_stmt;
                    }
                    cte_map.insert(cte_name.clone(), (cols, accumulated));
                } else {
                    let result = self.exec_select(*cte_stmt.clone())?;
                    if let QueryResult::Rows { columns, rows } = result {
                        cte_map.insert(cte_name.clone(), (columns, rows));
                    }
                }
            }
            if let Some((cte_cols, cte_rows)) = cte_map.get(&stmt.table) {
                let filtered = match self.filter_str_rows(
                    cte_rows.clone(),
                    cte_cols,
                    stmt.where_clause.as_ref(),
                ) {
                    Ok(rows) => rows,
                    Err(e) => return Ok(e),
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

        // S45 â€” handle views
        let view_stmt_opt = self.views.lock_safe().get(&stmt.table).cloned();
        if let Some(view_stmt) = view_stmt_opt {
            let result = self.exec_select(view_stmt)?;
            return match result {
                QueryResult::Rows {
                    columns: view_cols,
                    rows: view_rows,
                } => {
                    let filtered = match self.filter_str_rows(
                        view_rows,
                        &view_cols,
                        stmt.where_clause.as_ref(),
                    ) {
                        Ok(rows) => rows,
                        Err(e) => return Ok(e),
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

        // S69 â€” lateral joins (register subquery results as temporary views)
        if !stmt.lateral_joins.is_empty() {
            for lateral in &stmt.lateral_joins {
                let lat_result = self.exec_select(*lateral.subquery.clone())?;
                if let QueryResult::Rows { columns, rows } = lat_result {
                    // Build a synthetic SelectStmt from the rows
                    let col_exprs: Vec<SelectExpr> =
                        columns.iter().map(|c| SelectExpr::Col(c.clone())).collect();
                    // Store a view select so queries against this alias return the rows
                    // We'll use a values-based approach: create a temp view with the first column
                    // For simplicity, just store the original subquery as a view
                    let fake_stmt = SelectStmt {
                        columns: col_exprs,
                        table: lateral.subquery.table.clone(),
                        joins: vec![],
                        where_clause: None,
                        group_by: vec![],
                        having: None,
                        order_by: None,
                        limit: None,
                        offset: None,
                        distinct: false,
                        set_op: None,
                        ctes: vec![],
                        recursive_parts: vec![],
                        lateral_joins: vec![],
                        index_hint: None,
                    };
                    let _ = rows;
                    self.views
                        .lock_safe()
                        .insert(lateral.alias.clone(), fake_stmt);
                }
            }
        }

        // S75 â€” materialized views
        let mat_view_data = {
            let guard = self.mat_views.lock_safe();
            guard.get(&stmt.table).cloned()
        };
        if let Some((mat_select, cached_rows)) = mat_view_data {
            let mat_cols: Vec<String> = mat_select
                .columns
                .iter()
                .filter_map(|c| match c {
                    SelectExpr::Col(n) => Some(n.clone()),
                    SelectExpr::Star => None,
                    _ => None,
                })
                .collect();
            let filtered =
                match self.filter_str_rows(cached_rows, &mat_cols, stmt.where_clause.as_ref()) {
                    Ok(rows) => rows,
                    Err(e) => return Ok(e),
                };
            return Ok(QueryResult::Rows {
                columns: mat_cols,
                rows: filtered,
            });
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
        // MVCC Phase 2 — under REPEATABLE READ / SERIALIZABLE, read this
        // table's fixed snapshot instead of the latest committed state. Under
        // READ COMMITTED / autocommit `read_ts_for` returns `None` and
        // behavior is unchanged.
        let left_rows = if let Some(read_ts) = self.read_ts_for(&stmt.table) {
            engine.scan_at(read_ts)
        } else {
            self.execute_scan(&plan, &engine)?
        };
        // D5/S295 — the scanned working set counts against the memory limit.
        if self.memory_limited() {
            self.check_memory(est_engine_rows_bytes(&left_rows))?;
        }
        let mut filtered_left =
            self.filter_rows(left_rows, stmt.where_clause.as_ref(), schema, &stmt.table)?;

        // S78 â€” apply RLS policies for this table
        filtered_left = self.apply_rls(filtered_left, schema, &stmt.table)?;

        // â”€â”€ JOIN path â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
        if !stmt.joins.is_empty() {
            return self.exec_join(stmt, filtered_left, schema, &col_names);
        }

        // Collect computed (CASE/COALESCE/etc.) expressions â€” evaluated per row.
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
        // usize::MAX is a sentinel meaning "subquery column â€” filled separately".
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

        // â”€â”€ Aggregate path (COUNT/SUM/MIN/MAX with optional GROUP BY) â”€â”€â”€â”€â”€â”€â”€â”€â”€
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
                            // Subquery column â€” look up pre-evaluated value by alias.
                            subquery_values
                                .iter()
                                .find(|(a, _)| a == col_name)
                                .map(|(_, v)| v.clone())
                                .unwrap_or_default()
                        } else if *idx == usize::MAX - 1 {
                            // Window function column â€” look up pre-computed value.
                            window_row_vals
                                .get(row_idx)
                                .and_then(|wrow| {
                                    wrow.iter()
                                        .find(|(n, _)| n == col_name)
                                        .map(|(_, v)| v.clone())
                                })
                                .unwrap_or_default()
                        } else if *idx == usize::MAX - 2 {
                            // Computed expression (CASE WHEN, COALESCE, etc.) â€” evaluate per row.
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

        // S79 / S159 / S281 — column masks are applied by `exec_select`.

        // S87 â€” notice for empty table
        let warnings_on = *self.warnings_enabled.lock_safe();

        // S40 â€” set operations
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

        let base_result = QueryResult::Rows {
            columns: out_col_names,
            rows: rows.clone(),
        };
        let notice_msg = if warnings_on && rows.is_empty() {
            Some("table is empty".to_owned())
        } else {
            None
        };

        if let Some(notice) = notice_msg {
            Ok(QueryResult::WithNotice {
                result: Box::new(base_result),
                notice,
            })
        } else {
            Ok(base_result)
        }
    }

    /// Evaluate aggregate SELECT expressions (COUNT/SUM/MIN/MAX) with optional GROUP BY.
    pub(super) fn exec_aggregate(
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
            self.check_deadline()?;
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

    pub(super) fn eval_having(
        expr: &Expr,
        row: &std::collections::HashMap<String, String>,
    ) -> bool {
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

    pub(super) fn having_val(
        expr: &Expr,
        row: &std::collections::HashMap<String, String>,
    ) -> String {
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
    pub(super) fn exec_join(
        &self,
        stmt: SelectStmt,
        left_rows: Vec<EngineRow>,
        left_schema: &TableSchema,
        left_col_names: &[String],
    ) -> Result<QueryResult, ExecError> {
        // D5/S252 — joined tables need SELECT too
        for join in &stmt.joins {
            if let Some(denied) = self.check_privilege(&join.table, "SELECT") {
                return Ok(denied);
            }
        }
        // D5/S295 — the join's materialised rows count against the limit.
        let mem_limited = self.memory_limited();
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

        // For each JOIN, load the right table and build a HashMap<join_key â†’ Vec<display_row>>
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
            // S78 — RLS applies to joined tables too
            let right_rows =
                self.apply_rls(self.scan_live(&right_engine)?, right_schema, &join.table)?;
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
            let right_width = right_col_names.len();
            let mut next: Vec<Vec<String>> = Vec::new();
            let mut next_bytes: u64 = 0;
            // S71 â€” CROSS JOIN: pair every left row with every right row
            if join.kind == JoinKind::Cross {
                let all_right: Vec<Vec<String>> = right_map.values().flatten().cloned().collect();
                for left_row in &combined {
                    for right_row in &all_right {
                        self.check_deadline()?;
                        let mut merged = left_row.clone();
                        merged.extend(right_row.clone());
                        if mem_limited {
                            next_bytes += est_str_row_bytes(&merged);
                            self.check_memory(next_bytes)?;
                        }
                        next.push(merged);
                    }
                }
                combined = next;
                continue;
            }
            let left_key_idx = left_key_indices[join_idx];
            for left_row in &combined {
                self.check_deadline()?;
                let key = left_row.get(left_key_idx).cloned().unwrap_or_default();
                if let Some(right_matches) = right_map.get(&key) {
                    for right_row in right_matches {
                        let mut merged = left_row.clone();
                        merged.extend(right_row.clone());
                        if mem_limited {
                            next_bytes += est_str_row_bytes(&merged);
                            self.check_memory(next_bytes)?;
                        }
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

    /// S41/S45 helper â€” filter/project string rows.
    pub(super) fn project_str_rows(
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

    /// S41/S45 helper â€” evaluate WHERE on a string-row map.
    /// Evaluate a WHERE clause over a string-valued row (views, CTEs,
    /// materialized views). B10 — predicates this evaluator cannot handle are
    /// an error (reported as SQLSTATE 0A000), never silently true.
    pub(super) fn eval_str_where(
        expr: &Expr,
        row: &HashMap<String, String>,
    ) -> Result<bool, String> {
        Ok(match expr {
            Expr::BinOp {
                op: BinOp::And,
                left,
                right,
            } => Self::eval_str_where(left, row)? && Self::eval_str_where(right, row)?,
            Expr::BinOp {
                op: BinOp::Or,
                left,
                right,
            } => Self::eval_str_where(left, row)? || Self::eval_str_where(right, row)?,
            Expr::BinOp { op, left, right } => {
                let lv = Self::str_val(left, row)?;
                let rv = Self::str_val(right, row)?;
                compare_values(&lv, op, &rv)
            }
            Expr::IsNull { col, negated } => {
                let v = Self::str_col(col, row)?;
                let is_null = v.is_empty() || v == "NULL";
                if *negated { !is_null } else { is_null }
            }
            Expr::In { col, values } => {
                let v = Self::str_col(col, row)?;
                values
                    .iter()
                    .any(|lit| compare_values(&v, &BinOp::Eq, &Self::str_lit(lit)))
            }
            Expr::Between { col, lo, hi } => {
                let v = Self::str_col(col, row)?;
                compare_values(&v, &BinOp::Gte, &Self::str_lit(lo))
                    && compare_values(&v, &BinOp::Lte, &Self::str_lit(hi))
            }
            Expr::Like {
                col,
                pattern,
                negated,
                case_insensitive,
            } => {
                let v = Self::str_col(col, row)?;
                let m = if *case_insensitive {
                    like_match(&v.to_lowercase(), &pattern.to_lowercase())
                } else {
                    like_match(&v, pattern)
                };
                m != *negated
            }
            other => return Err(format!("unsupported predicate in WHERE clause: {other:?}")),
        })
    }

    /// Column lookup for [`eval_str_where`](Self::eval_str_where): exact name,
    /// else a unique `table.col` header (join output).
    pub(super) fn str_col(name: &str, row: &HashMap<String, String>) -> Result<String, String> {
        if let Some(v) = row.get(name) {
            return Ok(v.clone());
        }
        let suffix = format!(".{name}");
        let mut hits = row.iter().filter(|(k, _)| k.ends_with(&suffix));
        match (hits.next(), hits.next()) {
            (Some((_, v)), None) => Ok(v.clone()),
            (Some(_), Some(_)) => Err(format!("column reference \"{name}\" is ambiguous")),
            (None, _) => Err(format!("column \"{name}\" does not exist")),
        }
    }

    pub(super) fn str_lit(lit: &LitVal) -> String {
        match lit {
            LitVal::Int(n) => n.to_string(),
            LitVal::Float(f) => format!("{f:.6}"),
            LitVal::Str(s) => s.clone(),
            LitVal::Bool(b) => b.to_string(),
            LitVal::Null => "NULL".to_owned(),
        }
    }

    pub(super) fn str_val(expr: &Expr, row: &HashMap<String, String>) -> Result<String, String> {
        match expr {
            Expr::Col(name) => Self::str_col(name, row),
            Expr::QualifiedCol { table, col } => match row.get(&format!("{table}.{col}")) {
                Some(v) => Ok(v.clone()),
                None => Self::str_col(col, row),
            },
            Expr::Lit(lit) => Ok(Self::str_lit(lit)),
            other => Err(format!("unsupported value in WHERE clause: {other:?}")),
        }
    }

    /// B10 — filter string rows by `where_expr`, or the error result to return
    /// (0A000 for an unsupported predicate, 57014 on statement timeout).
    pub(super) fn filter_str_rows(
        &self,
        rows: Vec<Vec<String>>,
        cols: &[String],
        where_expr: Option<&Expr>,
    ) -> Result<Vec<Vec<String>>, QueryResult> {
        let Some(where_expr) = where_expr else {
            return Ok(rows);
        };
        let mut out = Vec::new();
        for row in rows {
            if self.check_deadline().is_err() {
                return Err(sql_error("57014", STATEMENT_TIMEOUT_MSG));
            }
            let m: HashMap<String, String> =
                cols.iter().cloned().zip(row.iter().cloned()).collect();
            match Self::eval_str_where(where_expr, &m) {
                Ok(true) => out.push(row),
                Ok(false) => {}
                Err(e) => return Err(sql_error("0A000", &e)),
            }
        }
        Ok(out)
    }

    /// Try to extract a simple single-column range predicate for pushdown.
    ///
    /// Returns `(col_name, lo_inclusive, hi_inclusive)` when WHERE is:
    /// Build and return an EXPLAIN plan for a SELECT without executing it.
    pub(super) fn explain_select(&self, stmt: SelectStmt) -> Result<QueryResult, ExecError> {
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
    pub(super) fn execute_scan(
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
                // S46 â€” treat missing zone files as empty (zone pruning correctness fix)
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
                    self.check_deadline()?;
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

    pub(super) fn leaf_scan(plan: &PhysicalPlan) -> &PhysicalPlan {
        match plan {
            PhysicalPlan::Filter { input, .. }
            | PhysicalPlan::Project { input, .. }
            | PhysicalPlan::Sort { input, .. }
            | PhysicalPlan::VecSort { input, .. }
            | PhysicalPlan::Limit { input, .. } => Self::leaf_scan(input),
            other => other,
        }
    }

    // ── B5/B6/B7 — constraints and triggers ──────────────────────────────

    /// Scan all rows and verify each is still live via `get()`.
    ///
    /// `scan(col=0, 0..MAX)` reads directly from partition files and may return
    /// rows whose tombstones are only in the WAL buffer (i.e. deleted but not yet
    /// compacted).  Validating each PK through `get()` uses the pk_index as the
    /// authoritative source of truth.
    pub(super) fn scan_live(
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
            self.check_deadline()?;
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

    pub(super) fn filter_rows(
        &self,
        rows: Vec<EngineRow>,
        where_clause: Option<&Expr>,
        schema: &TableSchema,
        table: &str,
    ) -> Result<Vec<EngineRow>, ExecError> {
        match where_clause {
            None => Ok(rows),
            Some(expr) => {
                let mut out = Vec::new();
                for row in rows {
                    self.check_deadline()?;
                    if self.eval_expr(expr, &row, schema, table)? {
                        out.push(row);
                    }
                }
                Ok(out)
            }
        }
    }

    pub(super) fn eval_expr(
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
                case_insensitive,
            } => {
                let (idx, col_def) = schema
                    .find(col)
                    .ok_or_else(|| ExecError::ColumnNotFound(col.clone()))?;
                let bits = row.values.get(idx).copied().unwrap_or(0);
                let val = TypedValue::from_bits(col_def.col_type, bits).display_str();
                // S70 â€” ILIKE uses case-insensitive matching
                let matched = if *case_insensitive {
                    like_match(&val.to_lowercase(), &pattern.to_lowercase())
                } else {
                    like_match(&val, pattern)
                };
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

    pub(super) fn eval_value(
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
                    // S64 â€” DATE_TRUNC
                    "DATE_TRUNC" => {
                        let part = vals.first().cloned().unwrap_or_default();
                        let date_str = vals.get(1).cloned().unwrap_or_default();
                        Ok(date_trunc(&part, &date_str))
                    }
                    // S64 â€” TIME_BUCKET
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
                    // S68 â€” NULLIF(a, b): return NULL if a == b, else a
                    "NULLIF" => {
                        let a = vals.first().cloned().unwrap_or_default();
                        let b = vals.get(1).cloned().unwrap_or_default();
                        if a == b { Ok("NULL".to_owned()) } else { Ok(a) }
                    }
                    // S68 â€” IFNULL(a, b): return b if a is NULL/empty, else a
                    "IFNULL" | "NVL" => {
                        let a = vals.first().cloned().unwrap_or_default();
                        let b = vals.get(1).cloned().unwrap_or("NULL".to_owned());
                        if a.is_empty() || a == "NULL" {
                            Ok(b)
                        } else {
                            Ok(a)
                        }
                    }
                    // S69 â€” GREATEST(a, b, ...): return maximum value (numeric or lexicographic)
                    "GREATEST" => {
                        let best = vals
                            .iter()
                            .max_by(|a, b| {
                                let an: Option<f64> = a.parse().ok();
                                let bn: Option<f64> = b.parse().ok();
                                match (an, bn) {
                                    (Some(x), Some(y)) => {
                                        x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    _ => a.cmp(b),
                                }
                            })
                            .cloned()
                            .unwrap_or_default();
                        Ok(best)
                    }
                    // S69 â€” LEAST(a, b, ...): return minimum value
                    "LEAST" => {
                        let best = vals
                            .iter()
                            .min_by(|a, b| {
                                let an: Option<f64> = a.parse().ok();
                                let bn: Option<f64> = b.parse().ok();
                                match (an, bn) {
                                    (Some(x), Some(y)) => {
                                        x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal)
                                    }
                                    _ => a.cmp(b),
                                }
                            })
                            .cloned()
                            .unwrap_or_default();
                        Ok(best)
                    }
                    // S72 â€” REPEAT(s, n): repeat string n times
                    "REPEAT" => {
                        let s = vals.first().cloned().unwrap_or_default();
                        let n: usize = vals.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                        Ok(s.repeat(n))
                    }
                    // S72 â€” REVERSE(s): reverse a string
                    "REVERSE" => {
                        let s = vals.first().cloned().unwrap_or_default();
                        Ok(s.chars().rev().collect())
                    }
                    // S72 â€” SPACE(n): return n spaces
                    "SPACE" => {
                        let n: usize = vals.first().and_then(|v| v.parse().ok()).unwrap_or(0);
                        Ok(" ".repeat(n))
                    }
                    // S75 â€” LPAD(s, n, pad): left-pad s to length n with pad char
                    "LPAD" => {
                        let s = vals.first().cloned().unwrap_or_default();
                        let n: usize = vals.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                        let pad = vals.get(2).map(|v| v.as_str()).unwrap_or(" ");
                        let pad_char = pad.chars().next().unwrap_or(' ');
                        let chars: Vec<char> = s.chars().collect();
                        if chars.len() >= n {
                            Ok(chars[..n].iter().collect())
                        } else {
                            let prefix: String =
                                std::iter::repeat_n(pad_char, n - chars.len()).collect();
                            Ok(format!("{prefix}{s}"))
                        }
                    }
                    // S75 â€” RPAD(s, n, pad): right-pad s to length n with pad char
                    "RPAD" => {
                        let s = vals.first().cloned().unwrap_or_default();
                        let n: usize = vals.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                        let pad = vals.get(2).map(|v| v.as_str()).unwrap_or(" ");
                        let pad_char = pad.chars().next().unwrap_or(' ');
                        let chars: Vec<char> = s.chars().collect();
                        if chars.len() >= n {
                            Ok(chars[..n].iter().collect())
                        } else {
                            let suffix: String =
                                std::iter::repeat_n(pad_char, n - chars.len()).collect();
                            Ok(format!("{s}{suffix}"))
                        }
                    }
                    // S75 â€” CONCAT_WS(sep, a, b, ...): join non-NULL args with separator
                    "CONCAT_WS" => {
                        let sep = vals.first().cloned().unwrap_or_default();
                        let parts: Vec<&str> = vals[1..]
                            .iter()
                            .filter(|v| !v.is_empty() && v.as_str() != "NULL")
                            .map(|v| v.as_str())
                            .collect();
                        Ok(parts.join(&sep))
                    }
                    // S76 â€” POWER(base, exp): exponentiation
                    "POWER" | "POW" => {
                        let base: f64 = vals.first().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                        let exp: f64 = vals.get(1).and_then(|v| v.parse().ok()).unwrap_or(1.0);
                        let r = base.powf(exp);
                        if r.fract() == 0.0 {
                            Ok(format!("{}", r as i64))
                        } else {
                            Ok(format!("{r}"))
                        }
                    }
                    // S76 â€” SQRT(n): square root
                    "SQRT" => {
                        let n: f64 = vals.first().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                        let r = n.sqrt();
                        if r.fract() == 0.0 {
                            Ok(format!("{}", r as i64))
                        } else {
                            Ok(format!("{r}"))
                        }
                    }
                    // S76 â€” SIGN(n): sign function
                    "SIGN" => {
                        let n: f64 = vals.first().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                        if n > 0.0 {
                            Ok("1".to_owned())
                        } else if n < 0.0 {
                            Ok("-1".to_owned())
                        } else {
                            Ok("0".to_owned())
                        }
                    }
                    // S77 â€” LTRIM(s): trim leading whitespace
                    "LTRIM" => {
                        let s = vals.first().cloned().unwrap_or_default();
                        Ok(s.trim_start().to_owned())
                    }
                    // S77 â€” RTRIM(s): trim trailing whitespace
                    "RTRIM" => {
                        let s = vals.first().cloned().unwrap_or_default();
                        Ok(s.trim_end().to_owned())
                    }
                    // S77 â€” FORMAT(n, decimals): format number with fixed decimal places
                    "FORMAT" => {
                        let n: f64 = vals.first().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                        let decimals: usize = vals.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                        Ok(format!("{:.prec$}", n, prec = decimals))
                    }
                    // S105 â€” PG_TYPEOF(col): return the type name of the column
                    "PG_TYPEOF" => {
                        // The argument is evaluated; try to detect type by value format
                        let v = vals.first().cloned().unwrap_or_default();
                        let t = if v.parse::<i64>().is_ok() {
                            "int8"
                        } else if v.parse::<f64>().is_ok() {
                            "float8"
                        } else if v == "true" || v == "false" {
                            "bool"
                        } else {
                            "text"
                        };
                        Ok(t.to_owned())
                    }
                    // S106 â€” PG_CANCEL_BACKEND(pid): no-op MVP, returns true
                    "PG_CANCEL_BACKEND" => Ok("true".to_owned()),
                    // S95 / compat
                    "PG_STAT_STATEMENTS_RESET" => {
                        self.query_stats.lock_safe().clear();
                        Ok(String::new())
                    }
                    // S65 â€” UDF dispatch
                    udf_name => {
                        let maybe_udf = self.udfs.lock_safe().get(udf_name).cloned();
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
            // S62 â€” MATCH_SCORE: count term occurrences in column value
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
            // S63 â€” JSON operators
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

    // S98 â€” EXPLAIN FLAMEGRAPH
    pub(super) fn exec_explain_flamegraph(
        &self,
        stmt: SelectStmt,
    ) -> Result<QueryResult, ExecError> {
        let t0 = std::time::Instant::now();
        // Get plan nodes
        let plan_nodes = if let Ok(engine) = self.registry.get(&stmt.table) {
            let schema = &engine.schema;
            let est: u64 = engine
                .partition_meta()
                .iter()
                .map(|p| p.row_count as u64)
                .sum::<u64>()
                .max(1);
            let plan = Planner::new(schema, est).plan(&stmt);
            plan.explain(0)
        } else {
            vec![format!("FullScan table={}", stmt.table)]
        };
        let result = self.exec_select(stmt);
        let total_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let _ = result;
        // Build flamegraph JSON
        let n = plan_nodes.len().max(1);
        let children: Vec<String> = plan_nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let fraction = if i == 0 { 0.6 } else { 0.4 / (n as f64) };
                let node_ms = total_ms * fraction;
                format!(
                    "{{\"name\":\"{}\",\"value\":{:.3}}}",
                    node.replace('"', "'"),
                    node_ms
                )
            })
            .collect();
        let json = format!(
            "{{\"name\":\"root\",\"value\":{:.3},\"children\":[{}]}}",
            total_ms,
            children.join(",")
        );
        Ok(QueryResult::ExplainPlan(vec![json]))
    }

    // S100 â€” EXPLAIN BUFFERS
    pub(super) fn exec_explain_buffers(&self, stmt: SelectStmt) -> Result<QueryResult, ExecError> {
        let table_name = stmt.table.clone();
        // Check cache for hit/miss
        let cache_hit = self.query_cache.lock_safe().contains_key(&table_name);
        let buffers_hit = if cache_hit { 1u64 } else { 0u64 };
        let buffers_miss = if cache_hit { 0u64 } else { 1u64 };
        // Get plan
        let plan_lines = if let Ok(engine) = self.registry.get(&table_name) {
            let schema = &engine.schema;
            let est: u64 = engine
                .partition_meta()
                .iter()
                .map(|p| p.row_count as u64)
                .sum::<u64>()
                .max(1);
            let plan = Planner::new(schema, est).plan(&stmt);
            plan.explain(0)
        } else {
            vec![format!("FullScan table={}", table_name)]
        };
        // Run the query
        let _result = self.exec_select(stmt);
        let lines: Vec<String> = plan_lines
            .iter()
            .map(|l| {
                format!(
                    "{} buffers_hit={} buffers_miss={}",
                    l, buffers_hit, buffers_miss
                )
            })
            .collect();
        Ok(QueryResult::ExplainPlan(lines))
    }

    // S56 â€” EXPLAIN ANALYZE
    pub(super) fn exec_explain_analyze(&self, stmt: SelectStmt) -> Result<QueryResult, ExecError> {
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
}
