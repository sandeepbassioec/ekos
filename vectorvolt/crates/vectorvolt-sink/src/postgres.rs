//! [`PostgresSink`] — generates Postgres upsert/delete SQL from sink events (VV-SNK-001).
//!
//! This sink does NOT depend on tokio-postgres. It produces SQL strings that
//! callers can execute with whatever Postgres driver they use.  This keeps the
//! crate free of a heavy dependency while still being fully testable.

use async_trait::async_trait;

use crate::{
    error::SinkResult,
    sink::{Sink, SinkEvent, SinkEventKind},
};

/// Generates Postgres-compatible SQL for each sink event and records it.
///
/// In production code, replace `record_sql` with an actual `execute` call
/// against a tokio-postgres or sqlx connection.
pub struct PostgresSink {
    name: String,
    table: String,
    sql_log: parking_lot::Mutex<Vec<String>>,
}

impl PostgresSink {
    pub fn new(name: &str, table: &str) -> Self {
        Self {
            name: name.to_string(),
            table: table.to_string(),
            sql_log: parking_lot::Mutex::new(Vec::new()),
        }
    }

    /// Build an `INSERT ... ON CONFLICT DO UPDATE` statement for one row.
    pub fn upsert_sql(&self, pk: u64, values: &[u64]) -> String {
        let cols: Vec<String> = std::iter::once("pk".to_string())
            .chain((0..values.len()).map(|i| format!("col{}", i)))
            .collect();
        let vals: Vec<String> = std::iter::once(pk.to_string())
            .chain(values.iter().map(|v| v.to_string()))
            .collect();
        let set_clauses: Vec<String> = (0..values.len())
            .map(|i| format!("col{i} = EXCLUDED.col{i}"))
            .collect();

        format!(
            "INSERT INTO {table} ({cols}) VALUES ({vals}) ON CONFLICT (pk) DO UPDATE SET {set};",
            table = self.table,
            cols = cols.join(", "),
            vals = vals.join(", "),
            set = if set_clauses.is_empty() {
                "pk = EXCLUDED.pk".to_string()
            } else {
                set_clauses.join(", ")
            }
        )
    }

    /// Build a `DELETE FROM` statement for one pk.
    pub fn delete_sql(&self, pk: u64) -> String {
        format!("DELETE FROM {} WHERE pk = {};", self.table, pk)
    }

    /// All SQL statements generated so far (for testing).
    pub fn sql_log(&self) -> Vec<String> {
        self.sql_log.lock().clone()
    }
}

#[async_trait]
impl Sink for PostgresSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: SinkEvent) -> SinkResult<()> {
        let sql = match event.kind {
            SinkEventKind::Insert | SinkEventKind::Update => {
                self.upsert_sql(event.pk, &event.values)
            }
            SinkEventKind::Delete => self.delete_sql(event.pk),
        };
        self.sql_log.lock().push(sql);
        Ok(())
    }

    async fn flush(&self) -> SinkResult<()> {
        Ok(())
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::SinkEvent;

    fn sink() -> PostgresSink {
        PostgresSink::new("pg", "orders")
    }

    #[test]
    fn upsert_sql_contains_table_name() {
        let s = sink();
        let sql = s.upsert_sql(1, &[10, 20]);
        assert!(sql.contains("orders"), "sql: {sql}");
    }

    #[test]
    fn upsert_sql_contains_pk() {
        let s = sink();
        let sql = s.upsert_sql(42, &[99]);
        assert!(sql.contains("42"), "sql: {sql}");
    }

    #[test]
    fn upsert_sql_on_conflict() {
        let s = sink();
        let sql = s.upsert_sql(1, &[10]);
        assert!(sql.contains("ON CONFLICT"), "sql: {sql}");
    }

    #[test]
    fn delete_sql_contains_table_name() {
        let s = sink();
        let sql = s.delete_sql(5);
        assert!(sql.contains("orders"), "sql: {sql}");
    }

    #[test]
    fn delete_sql_contains_pk() {
        let s = sink();
        let sql = s.delete_sql(99);
        assert!(sql.contains("99"), "sql: {sql}");
    }

    #[test]
    fn delete_sql_starts_with_delete() {
        let s = sink();
        let sql = s.delete_sql(1);
        assert!(sql.starts_with("DELETE"), "sql: {sql}");
    }

    #[tokio::test]
    async fn send_insert_logs_sql() {
        let s = sink();
        s.send(SinkEvent::insert("orders", 1, vec![10, 20]))
            .await
            .unwrap();
        assert_eq!(s.sql_log().len(), 1);
        assert!(s.sql_log()[0].contains("INSERT"));
    }

    #[tokio::test]
    async fn send_delete_logs_sql() {
        let s = sink();
        s.send(SinkEvent::delete("orders", 3)).await.unwrap();
        assert_eq!(s.sql_log().len(), 1);
        assert!(s.sql_log()[0].contains("DELETE"));
    }

    #[tokio::test]
    async fn send_multiple_events() {
        let s = sink();
        for i in 0..5u64 {
            s.send(SinkEvent::insert("orders", i, vec![i]))
                .await
                .unwrap();
        }
        assert_eq!(s.sql_log().len(), 5);
    }

    #[test]
    fn upsert_sql_no_values() {
        let s = sink();
        let sql = s.upsert_sql(1, &[]);
        assert!(sql.contains("INSERT"), "sql: {sql}");
    }

    #[tokio::test]
    async fn flush_is_noop() {
        let s = sink();
        s.flush().await.unwrap();
    }

    #[test]
    fn name_matches_constructor() {
        let s = sink();
        assert_eq!(s.name(), "pg");
    }

    #[test]
    fn upsert_sql_update_set_clause() {
        let s = sink();
        let sql = s.upsert_sql(1, &[5, 10]);
        assert!(sql.contains("DO UPDATE SET"), "sql: {sql}");
    }
}
