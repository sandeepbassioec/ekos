//! [`PostgresSink`] - real Postgres sink on `tokio-postgres` (VV-SNK-001).
//!
//! Events are buffered; `flush` runs them inside one transaction using
//! parameterized statements and awaits `COMMIT`. Reaching `batch_size`
//! buffered events triggers an automatic flush.
//!
//! Table layout expected by the sink: `pk BIGINT PRIMARY KEY, col0 BIGINT,
//! col1 BIGINT, ...`. `u64` values above `i64::MAX` are rejected with an
//! error rather than silently wrapped.
//!
//! Connections use `NoTls` (pure Rust, no native TLS dependency); run behind a
//! trusted network or a TLS-terminating proxy.

use std::time::Duration;

use async_trait::async_trait;
use tokio_postgres::{Client, NoTls, types::ToSql};

use crate::{
    error::{SinkError, SinkResult},
    sink::{Sink, SinkEvent, SinkEventKind},
};

/// Connection settings for [`PostgresSink`].
#[derive(Debug, Clone)]
pub struct PostgresSinkConfig {
    /// libpq-style connection string / URL.
    pub conn_str: String,
    /// Target table (`name` or `schema.name`; identifiers are validated).
    pub table: String,
    /// Auto-flush once this many events are buffered (min 1).
    pub batch_size: usize,
    pub connect_timeout: Duration,
}

impl PostgresSinkConfig {
    pub fn new(conn_str: impl Into<String>, table: impl Into<String>) -> Self {
        Self {
            conn_str: conn_str.into(),
            table: table.into(),
            batch_size: 100,
            connect_timeout: Duration::from_secs(5),
        }
    }
}

/// Validate `name` / `schema.name`: ASCII alphanumerics and `_`, not starting
/// with a digit. Prevents SQL injection through the table identifier.
pub fn validate_table(table: &str) -> SinkResult<()> {
    let ok_ident = |s: &str| {
        !s.is_empty()
            && !s.starts_with(|c: char| c.is_ascii_digit())
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    };
    let parts: Vec<&str> = table.split('.').collect();
    if (1..=2).contains(&parts.len()) && parts.iter().all(|p| ok_ident(p)) {
        Ok(())
    } else {
        Err(SinkError::Other(format!("invalid table name: {table:?}")))
    }
}

/// Build the parameterized upsert: `$1` = pk, `$2..` = column values.
pub fn upsert_sql(table: &str, n_values: usize) -> String {
    let cols: Vec<String> = std::iter::once("pk".to_string())
        .chain((0..n_values).map(|i| format!("col{i}")))
        .collect();
    let params: Vec<String> = (1..=cols.len()).map(|i| format!("${i}")).collect();
    let set: Vec<String> = (0..n_values)
        .map(|i| format!("col{i} = EXCLUDED.col{i}"))
        .collect();
    format!(
        "INSERT INTO {table} ({}) VALUES ({}) ON CONFLICT (pk) DO UPDATE SET {}",
        cols.join(", "),
        params.join(", "),
        if set.is_empty() {
            "pk = EXCLUDED.pk".to_string()
        } else {
            set.join(", ")
        }
    )
}

/// Build the parameterized delete: `$1` = pk.
pub fn delete_sql(table: &str) -> String {
    format!("DELETE FROM {table} WHERE pk = $1")
}

fn to_i64(v: u64) -> SinkResult<i64> {
    i64::try_from(v).map_err(|_| SinkError::Other(format!("value {v} exceeds BIGINT range")))
}

/// Statement + bound parameters for one event.
fn plan(table: &str, ev: &SinkEvent) -> SinkResult<(String, Vec<i64>)> {
    match ev.kind {
        SinkEventKind::Insert | SinkEventKind::Update => {
            let mut p = Vec::with_capacity(ev.values.len() + 1);
            p.push(to_i64(ev.pk)?);
            for v in &ev.values {
                p.push(to_i64(*v)?);
            }
            Ok((upsert_sql(table, ev.values.len()), p))
        }
        SinkEventKind::Delete => Ok((delete_sql(table), vec![to_i64(ev.pk)?])),
    }
}

/// Postgres sink with transactional batched writes.
pub struct PostgresSink {
    name: String,
    table: String,
    batch_size: usize,
    client: tokio::sync::Mutex<Client>,
    buffer: tokio::sync::Mutex<Vec<SinkEvent>>,
}

impl PostgresSink {
    /// Connect to Postgres. Fails if the server is unreachable.
    pub async fn connect(name: &str, cfg: PostgresSinkConfig) -> SinkResult<Self> {
        validate_table(&cfg.table)?;
        let (client, connection) = tokio::time::timeout(
            cfg.connect_timeout,
            tokio_postgres::connect(&cfg.conn_str, NoTls),
        )
        .await
        .map_err(|_| SinkError::Io("postgres connect: timed out".into()))?
        .map_err(|e| SinkError::Io(format!("postgres connect: {e}")))?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("postgres sink connection error: {e}");
            }
        });
        Ok(Self {
            name: name.to_string(),
            table: cfg.table,
            batch_size: cfg.batch_size.max(1),
            client: tokio::sync::Mutex::new(client),
            buffer: tokio::sync::Mutex::new(Vec::new()),
        })
    }

    /// Run `events` in one transaction and await commit.
    async fn write_batch(&self, events: &[SinkEvent]) -> SinkResult<()> {
        let planned: Vec<(String, Vec<i64>)> = events
            .iter()
            .map(|e| plan(&self.table, e))
            .collect::<SinkResult<_>>()?;
        let mut client = self.client.lock().await;
        let tx = client
            .transaction()
            .await
            .map_err(|e| SinkError::Io(format!("postgres begin: {e}")))?;
        for (sql, params) in &planned {
            let refs: Vec<&(dyn ToSql + Sync)> =
                params.iter().map(|p| p as &(dyn ToSql + Sync)).collect();
            tx.execute(sql.as_str(), &refs)
                .await
                .map_err(|e| SinkError::Io(format!("postgres execute: {e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| SinkError::Io(format!("postgres commit: {e}")))
    }

    async fn flush_locked(&self, buf: &mut Vec<SinkEvent>) -> SinkResult<()> {
        if buf.is_empty() {
            return Ok(());
        }
        // Events stay buffered until the transaction commits.
        self.write_batch(buf).await?;
        buf.clear();
        Ok(())
    }
}

#[async_trait]
impl Sink for PostgresSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: SinkEvent) -> SinkResult<()> {
        let mut buf = self.buffer.lock().await;
        buf.push(event);
        if buf.len() >= self.batch_size {
            self.flush_locked(&mut buf).await?;
        }
        Ok(())
    }

    /// Commit all buffered events in one transaction.
    async fn flush(&self) -> SinkResult<()> {
        let mut buf = self.buffer.lock().await;
        self.flush_locked(&mut buf).await
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_sql_parameterized() {
        let sql = upsert_sql("orders", 2);
        assert_eq!(
            sql,
            "INSERT INTO orders (pk, col0, col1) VALUES ($1, $2, $3) ON CONFLICT (pk) DO UPDATE SET col0 = EXCLUDED.col0, col1 = EXCLUDED.col1"
        );
    }

    #[test]
    fn upsert_sql_no_values() {
        let sql = upsert_sql("t", 0);
        assert!(sql.contains("VALUES ($1)"), "sql: {sql}");
        assert!(sql.ends_with("pk = EXCLUDED.pk"), "sql: {sql}");
    }

    #[test]
    fn delete_sql_parameterized() {
        assert_eq!(delete_sql("orders"), "DELETE FROM orders WHERE pk = $1");
    }

    #[test]
    fn plan_insert_and_delete() {
        let (sql, p) = plan("t", &SinkEvent::insert("t", 5, vec![1, 2])).unwrap();
        assert!(sql.starts_with("INSERT"));
        assert_eq!(p, vec![5, 1, 2]);
        let (sql, p) = plan("t", &SinkEvent::delete("t", 9)).unwrap();
        assert!(sql.starts_with("DELETE"));
        assert_eq!(p, vec![9]);
    }

    #[test]
    fn plan_rejects_overflow() {
        assert!(plan("t", &SinkEvent::insert("t", u64::MAX, vec![])).is_err());
        assert!(plan("t", &SinkEvent::insert("t", 1, vec![u64::MAX])).is_err());
    }

    #[test]
    fn table_validation() {
        assert!(validate_table("orders").is_ok());
        assert!(validate_table("public.orders_1").is_ok());
        assert!(validate_table("orders; DROP TABLE x").is_err());
        assert!(validate_table("a.b.c").is_err());
        assert!(validate_table("1abc").is_err());
        assert!(validate_table("").is_err());
    }

    #[tokio::test]
    async fn connect_rejects_bad_table() {
        let cfg = PostgresSinkConfig::new("host=127.0.0.1 port=1", "x;y");
        assert!(PostgresSink::connect("pg", cfg).await.is_err());
    }

    #[tokio::test]
    async fn connect_unreachable_errors() {
        let cfg = PostgresSinkConfig::new("host=127.0.0.1 port=1 user=x connect_timeout=3", "t");
        let r = PostgresSink::connect("pg", cfg).await;
        assert!(r.is_err(), "unreachable server must be an error");
    }

    /// Needs a live server: `VV_TEST_PG=postgres://user:pw@host/db`. Creates a
    /// table `vv_sink_test(pk BIGINT PRIMARY KEY, col0 BIGINT)` and drops it.
    #[tokio::test]
    #[ignore = "requires VV_TEST_PG"]
    async fn live_upsert_flush_delete() {
        let url = std::env::var("VV_TEST_PG").expect("VV_TEST_PG");
        let (c, conn) = tokio_postgres::connect(&url, NoTls).await.unwrap();
        tokio::spawn(conn);
        c.batch_execute(
            "DROP TABLE IF EXISTS vv_sink_test; CREATE TABLE vv_sink_test(pk BIGINT PRIMARY KEY, col0 BIGINT)",
        )
        .await
        .unwrap();
        let sink = PostgresSink::connect("pg", PostgresSinkConfig::new(&url, "vv_sink_test"))
            .await
            .unwrap();
        sink.send(SinkEvent::insert("t", 1, vec![10]))
            .await
            .unwrap();
        sink.send(SinkEvent::insert("t", 1, vec![11]))
            .await
            .unwrap();
        sink.send(SinkEvent::insert("t", 2, vec![20]))
            .await
            .unwrap();
        sink.flush().await.unwrap();
        let rows = c
            .query("SELECT pk, col0 FROM vv_sink_test ORDER BY pk", &[])
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get::<_, i64>(1), 11);
        sink.send(SinkEvent::delete("t", 2)).await.unwrap();
        sink.flush().await.unwrap();
        let n = c
            .query("SELECT 1 FROM vv_sink_test", &[])
            .await
            .unwrap()
            .len();
        assert_eq!(n, 1);
        c.batch_execute("DROP TABLE vv_sink_test").await.unwrap();
    }
}
