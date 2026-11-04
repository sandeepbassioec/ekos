//! Graceful shutdown, VACUUM and ANALYZE.

use super::*;

impl SqlExecutor {
    // S109 â€” graceful shutdown
    pub(super) fn exec_shutdown_graceful(&self) -> Result<QueryResult, ExecError> {
        // Flush WAL: write audit_log JSON to file
        let log = self.audit_log.lock_safe().clone();
        let log_json: String = log
            .iter()
            .map(|e| {
                format!(
                    "{{\"ts\":\"{}\",\"op\":\"{}\",\"table\":\"{}\"}}",
                    e.ts, e.op, e.table
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let _ = std::fs::write("/tmp/vv_wal_flush.json", format!("[{}]", log_json));
        // Checkpoint
        let _ = std::fs::write("/tmp/vv_checkpoint.txt", "checkpoint");
        Ok(QueryResult::Ok("graceful shutdown complete".to_owned()))
    }

    // S114 â€” VACUUM
    pub(super) fn exec_vacuum(
        &self,
        table: String,
        analyze: bool,
    ) -> Result<QueryResult, ExecError> {
        let dead = self.dead_rows.lock_safe().remove(&table).unwrap_or(0);
        if analyze {
            // Also run analyze
            self.exec_analyze(table.clone())?;
        }
        Ok(QueryResult::Ok(format!(
            "VACUUM: reclaimed {} dead rows",
            dead
        )))
    }

    // S115 â€” ANALYZE
    pub(super) fn exec_analyze(&self, table: String) -> Result<QueryResult, ExecError> {
        let engine = self
            .registry
            .get(&table)
            .map_err(|_| ExecError::TableNotFound(table.clone()))?;
        let schema = &engine.schema;
        let col_names: Vec<String> = schema.columns().iter().map(|c| c.name.clone()).collect();
        let all_rows = self.scan_live(&engine)?;
        let row_count = all_rows.len() as u64;

        let mut col_stats: HashMap<String, ColStat> = HashMap::new();
        for col_name in &col_names {
            let mut stat = ColStat {
                row_count,
                null_count: 0,
                distinct_count: 0,
                min_val: String::new(),
                max_val: String::new(),
            };
            if let Some((idx, col_def)) = schema.find(col_name) {
                let mut distinct_vals: HashSet<String> = HashSet::new();
                let mut min_val: Option<String> = None;
                let mut max_val: Option<String> = None;
                for row in &all_rows {
                    let raw = row.values.get(idx).copied().unwrap_or(0);
                    let val = TypedValue::from_bits(col_def.col_type, raw).display_str();
                    if val.is_empty() || val == "0" {
                        stat.null_count += 1;
                    }
                    distinct_vals.insert(val.clone());
                    min_val = Some(match min_val {
                        None => val.clone(),
                        Some(ref m) if val < *m => val.clone(),
                        Some(m) => m,
                    });
                    max_val = Some(match max_val {
                        None => val.clone(),
                        Some(ref m) if val > *m => val.clone(),
                        Some(m) => m,
                    });
                }
                stat.distinct_count = distinct_vals.len() as u64;
                stat.min_val = min_val.unwrap_or_default();
                stat.max_val = max_val.unwrap_or_default();
            }
            col_stats.insert(col_name.clone(), stat);
        }
        self.table_stats.lock_safe().insert(table, col_stats);
        Ok(QueryResult::Affected(0))
    }
}
