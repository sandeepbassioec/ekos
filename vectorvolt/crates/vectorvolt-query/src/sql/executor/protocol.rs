//! Meta commands, cursors, prepared statements and the pgwire portal API.

use super::*;

impl SqlExecutor {
    // S101/S102 â€” meta-commands
    pub(super) fn exec_meta_cmd(
        &self,
        cmd: &str,
        arg: Option<&str>,
    ) -> Result<QueryResult, ExecError> {
        match cmd {
            "\\dt" | "\\t" => {
                // List all tables
                let columns = vec!["name".to_owned(), "type".to_owned()];
                let rows = vec![
                    vec!["(user tables)".to_owned(), "table".to_owned()],
                    vec!["vv_health".to_owned(), "virtual".to_owned()],
                    vec!["pg_type".to_owned(), "virtual".to_owned()],
                ];
                Ok(QueryResult::Rows { columns, rows })
            }
            "\\d" => {
                let tbl = arg.unwrap_or("");
                if tbl.is_empty() {
                    // list tables
                    let columns = vec!["name".to_owned()];
                    let rows = vec![vec![tbl.to_owned()]];
                    return Ok(QueryResult::Rows { columns, rows });
                }
                // Describe table
                if let Ok(engine) = self.registry.get(tbl) {
                    let schema = &engine.schema;
                    let columns = vec!["column".to_owned(), "type".to_owned()];
                    let rows: Vec<Vec<String>> = schema
                        .columns()
                        .iter()
                        .map(|c| vec![c.name.clone(), format!("{:?}", c.col_type).to_lowercase()])
                        .collect();
                    Ok(QueryResult::Rows { columns, rows })
                } else {
                    Ok(QueryResult::Rows {
                        columns: vec!["error".to_owned()],
                        rows: vec![vec![format!("table {} not found", tbl)]],
                    })
                }
            }
            "\\di" => {
                let columns = vec![
                    "index_name".to_owned(),
                    "table".to_owned(),
                    "column".to_owned(),
                ];
                Ok(QueryResult::Rows {
                    columns,
                    rows: vec![],
                })
            }
            "\\dv" => {
                let views: Vec<Vec<String>> = self
                    .views
                    .lock_safe()
                    .keys()
                    .map(|k| vec![k.clone(), "view".to_owned()])
                    .collect();
                let columns = vec!["name".to_owned(), "type".to_owned()];
                Ok(QueryResult::Rows {
                    columns,
                    rows: views,
                })
            }
            // S102
            "\\l" => {
                let columns = vec!["databases".to_owned()];
                let rows = vec![vec!["vectorvolt".to_owned()]];
                Ok(QueryResult::Rows { columns, rows })
            }
            "\\timing" => {
                let mut t = self.timing_enabled.lock_safe();
                *t = !*t;
                let state = if *t { "on" } else { "off" };
                Ok(QueryResult::Ok(format!("Timing is {}", state)))
            }
            "\\x" => {
                let mut e = self.expanded_mode.lock_safe();
                *e = !*e;
                let state = if *e { "on" } else { "off" };
                Ok(QueryResult::Ok(format!("Expanded display is {}", state)))
            }
            "\\c" => {
                let db = arg.unwrap_or("vectorvolt");
                Ok(QueryResult::Ok(format!(
                    "You are now connected to database \"{}\"",
                    db
                )))
            }
            _ => Ok(QueryResult::Ok(format!("Unknown meta-command: {}", cmd))),
        }
    }

    // S103 â€” DECLARE CURSOR
    pub(super) fn exec_declare_cursor(
        &self,
        name: String,
        select: SelectStmt,
    ) -> Result<QueryResult, ExecError> {
        let result = self.exec_select(select)?;
        let rows = match result {
            QueryResult::Rows { rows, .. } => rows,
            _ => vec![],
        };
        self.cursors
            .lock_safe()
            .insert(name, CursorState { rows, pos: 0 });
        Ok(QueryResult::Affected(0))
    }

    // S103 â€” FETCH FROM cursor
    pub(super) fn exec_fetch_cursor(
        &self,
        name: &str,
        count: u64,
    ) -> Result<QueryResult, ExecError> {
        let mut guard = self.cursors.lock_safe();
        let state = guard
            .get_mut(name)
            .ok_or_else(|| ExecError::Engine(format!("cursor {} not found", name)))?;
        let start = state.pos;
        let take = if count == 0 {
            state.rows.len() - start
        } else {
            count as usize
        };
        let end = (start + take).min(state.rows.len());
        let fetched = state.rows[start..end].to_vec();
        state.pos = end;
        // Use a dummy column header
        let cols = fetched
            .first()
            .map(|r| (0..r.len()).map(|i| format!("col{}", i)).collect())
            .unwrap_or_default();
        Ok(QueryResult::Rows {
            columns: cols,
            rows: fetched,
        })
    }

    // S103 â€” CLOSE cursor
    pub(super) fn exec_close_cursor(&self, name: &str) -> Result<QueryResult, ExecError> {
        self.cursors.lock_safe().remove(name);
        Ok(QueryResult::Affected(0))
    }

    // S57 â€” Prepared statements
    pub(super) fn exec_prepare(&self, name: String, stmt: Stmt) -> Result<QueryResult, ExecError> {
        self.prepared.lock_safe().insert(name, stmt);
        Ok(QueryResult::Affected(0))
    }

    pub(super) fn exec_execute_prepared(
        &self,
        name: String,
        params: Vec<Expr>,
    ) -> Result<QueryResult, ExecError> {
        let stmt = self
            .prepared
            .lock_safe()
            .get(&name)
            .cloned()
            .ok_or_else(|| ExecError::Engine(format!("prepared statement '{}' not found", name)))?;
        let substituted = substitute_params(stmt, &params);
        self.execute_stmt(substituted)
    }

    // S85 â€” pgwire extended query protocol helpers
    pub fn pgwire_parse(&self, name: &str, sql: &str) -> Result<(), ExecError> {
        let tokens = Lexer::new(sql).tokenize()?;
        let stmt = SqlParser::new(tokens).parse()?;
        self.prepared.lock_safe().insert(name.to_owned(), stmt);
        Ok(())
    }

    pub fn pgwire_bind(&self, portal: &str, stmt_name: &str, params: Vec<LitVal>) {
        self.bound_portals
            .lock_safe()
            .insert(portal.to_owned(), (stmt_name.to_owned(), params));
    }

    pub fn pgwire_execute_portal(&self, portal: &str) -> Result<QueryResult, ExecError> {
        let (stmt_name, params) = self
            .bound_portals
            .lock_safe()
            .get(portal)
            .cloned()
            .ok_or_else(|| ExecError::Engine(format!("portal '{}' not bound", portal)))?;
        let param_exprs: Vec<Expr> = params.into_iter().map(Expr::Lit).collect();
        self.exec_execute_prepared(stmt_name, param_exprs)
    }

    pub fn pgwire_describe(&self, stmt_name: &str) -> Vec<String> {
        let stmt_opt = self.prepared.lock_safe().get(stmt_name).cloned();
        match stmt_opt {
            Some(Stmt::Select(s)) => s
                .columns
                .iter()
                .map(|c| match c {
                    SelectExpr::Col(n) => n.clone(),
                    SelectExpr::Star => "*".to_owned(),
                    SelectExpr::Agg { func, .. } => match func {
                        AggFunc::Count => "count".to_owned(),
                        AggFunc::Sum => "sum".to_owned(),
                        AggFunc::Min => "min".to_owned(),
                        AggFunc::Max => "max".to_owned(),
                    },
                    _ => "expr".to_owned(),
                })
                .collect(),
            _ => vec![],
        }
    }

    // S83 â€” check whether a client IP is allowed
    /// S116 — the server-wide connection limit (`SET MAX_CONNECTIONS n`),
    /// shared by every session from [`new_session`](Self::new_session).
    pub fn max_connections(&self) -> u64 {
        *self.max_connections.lock_safe()
    }

    pub fn check_ip_allowed(&self, client_ip: &str) -> bool {
        let ips = self.allowed_ips.lock_safe();
        if ips.is_empty() {
            true
        } else {
            ips.contains(client_ip)
        }
    }
}
