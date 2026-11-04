//! VectorVoltClient — async SDK wrapping the blocking net Client (VV-SDK-001).
//!
//! The client keeps ONE persistent connection (established lazily on first
//! use and re-established after transport errors). A server-side transaction
//! is bound to its connection, so `begin_txn` ... `commit_txn`/`abort_txn`
//! spans calls because every call reuses that same connection. Calls are
//! serialised on the connection.
use std::sync::{Arc, Mutex, MutexGuard};

use crate::SdkError;

/// One live connection plus the per-connection state we have established on it.
struct Conn {
    client: vectorvolt_net::Client,
    /// Table last selected with `use_table` on this connection.
    table: Option<String>,
    /// Open transaction on this connection, if any.
    txn: Option<u64>,
}

type Shared = Arc<Mutex<Option<Conn>>>;

fn lock(shared: &Shared) -> MutexGuard<'_, Option<Conn>> {
    shared.lock().unwrap_or_else(|p| p.into_inner())
}

/// Async VectorVolt client. Each method runs a blocking task on the single
/// persistent connection.
pub struct VectorVoltClient {
    addr: String,
    conn: Shared,
}

impl VectorVoltClient {
    /// Create a client. Does not check connectivity — the connection is
    /// established on first call.
    pub async fn connect(addr: &str) -> Result<Self, SdkError> {
        Ok(Self {
            addr: addr.to_string(),
            conn: Arc::new(Mutex::new(None)),
        })
    }

    fn map_err(e: impl std::fmt::Display) -> SdkError {
        SdkError::Net(e.to_string())
    }

    /// Run `f` on the persistent connection (connecting if needed) on a
    /// blocking thread. On a transport-level failure the connection is
    /// dropped (it may be desynchronised, and the server aborts its open
    /// transaction); application-level engine errors keep it, so an open
    /// transaction survives a rejected statement.
    async fn with_conn<T, F>(&self, f: F) -> Result<T, SdkError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Conn) -> Result<T, vectorvolt_net::NetError> + Send + 'static,
    {
        let addr = self.addr.clone();
        let shared = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let mut guard = lock(&shared);
            if guard.is_none() {
                let client = vectorvolt_net::Client::connect(&addr).map_err(Self::map_err)?;
                *guard = Some(Conn {
                    client,
                    table: None,
                    txn: None,
                });
            }
            let conn = guard.as_mut().expect("connection just established");
            match f(conn) {
                Ok(v) => Ok(v),
                Err(e) => {
                    if !matches!(e, vectorvolt_net::NetError::Engine(_)) {
                        *guard = None;
                    }
                    Err(Self::map_err(e))
                }
            }
        })
        .await
        .map_err(|e| SdkError::Op(e.to_string()))?
    }

    /// Select `table` on the connection if it is not already selected.
    fn ensure_table(conn: &mut Conn, table: &str) -> Result<(), vectorvolt_net::NetError> {
        if conn.table.as_deref() != Some(table) {
            conn.client.use_table(table)?;
            conn.table = Some(table.to_string());
        }
        Ok(())
    }

    /// Insert a row. Selects the table (once per connection), then inserts.
    /// Inside an open transaction the insert is buffered until commit.
    pub async fn insert(&self, table: &str, pk: u64, values: &[u64]) -> Result<(), SdkError> {
        let table = table.to_string();
        let values = values.to_vec();
        self.with_conn(move |c| {
            Self::ensure_table(c, &table)?;
            c.client.insert(pk, values)
        })
        .await
    }

    /// Get a row by primary key.
    pub async fn get(&self, table: &str, pk: u64) -> Result<Option<Vec<u64>>, SdkError> {
        let table = table.to_string();
        self.with_conn(move |c| {
            Self::ensure_table(c, &table)?;
            c.client.get(pk)
        })
        .await
    }

    /// Delete a row by primary key.
    pub async fn delete(&self, table: &str, pk: u64) -> Result<(), SdkError> {
        let table = table.to_string();
        self.with_conn(move |c| {
            Self::ensure_table(c, &table)?;
            c.client.delete(pk).map(|_| ())
        })
        .await
    }

    /// Scan all rows in a table via full range scan.
    pub async fn scan(&self, table: &str) -> Result<Vec<(u64, Vec<u64>)>, SdkError> {
        let table = table.to_string();
        self.with_conn(move |c| {
            Self::ensure_table(c, &table)?;
            let rows = c.client.range_scan(0, 0, u64::MAX, 0)?;
            Ok(rows
                .into_iter()
                .enumerate()
                .map(|(i, vals)| (i as u64, vals))
                .collect())
        })
        .await
    }

    /// Ping the server and return round-trip duration (includes connecting
    /// if no connection is open yet).
    pub async fn ping(&self) -> Result<std::time::Duration, SdkError> {
        let start = std::time::Instant::now();
        self.with_conn(|c| c.client.ping()).await?;
        Ok(start.elapsed())
    }

    /// Return a health status string.
    pub async fn health(&self) -> Result<String, SdkError> {
        self.with_conn(|c| {
            let (rows, _parts, bytes) = c.client.stats()?;
            Ok(format!("ok rows={rows} bytes={bytes}"))
        })
        .await
    }

    /// Begin a transaction on the persistent connection and return its txn_id.
    /// Subsequent calls on this client run inside the transaction until
    /// [`commit_txn`](Self::commit_txn) or [`abort_txn`](Self::abort_txn).
    pub async fn begin_txn(&self) -> Result<u64, SdkError> {
        self.with_conn(|c| {
            if c.txn.is_some() {
                return Err(vectorvolt_net::NetError::Engine(
                    "a transaction is already open on this client".into(),
                ));
            }
            let id = c.client.begin_txn()?;
            c.txn = Some(id);
            Ok(id)
        })
        .await
    }

    fn check_txn(c: &Conn, txn_id: u64) -> Result<(), vectorvolt_net::NetError> {
        if c.txn == Some(txn_id) {
            Ok(())
        } else {
            Err(vectorvolt_net::NetError::Engine(format!(
                "transaction {txn_id} is not open on this client's connection"
            )))
        }
    }

    /// Commit a transaction.
    pub async fn commit_txn(&self, txn_id: u64) -> Result<(), SdkError> {
        self.with_conn(move |c| {
            Self::check_txn(c, txn_id)?;
            // Whatever the outcome the transaction is finished for this client.
            c.txn = None;
            c.client.commit_txn(txn_id).map(|_| ())
        })
        .await
    }

    /// Abort/rollback a transaction.
    pub async fn abort_txn(&self, txn_id: u64) -> Result<(), SdkError> {
        self.with_conn(move |c| {
            Self::check_txn(c, txn_id)?;
            c.txn = None;
            c.client.rollback_txn(txn_id)
        })
        .await
    }
}
