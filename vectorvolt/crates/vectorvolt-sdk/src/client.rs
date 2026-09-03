//! VectorVoltClient — async SDK wrapping the blocking net Client (VV-SDK-001).
use crate::SdkError;

/// Async VectorVolt client. Each method spawns a blocking task.
pub struct VectorVoltClient {
    addr: String,
}

impl VectorVoltClient {
    /// Create a client. Does not check connectivity — that happens on first call.
    pub async fn connect(addr: &str) -> Result<Self, SdkError> {
        Ok(Self {
            addr: addr.to_string(),
        })
    }

    fn map_err(e: impl std::fmt::Display) -> SdkError {
        SdkError::Net(e.to_string())
    }

    /// Insert a row. Calls use_table first, then insert.
    pub async fn insert(&self, table: &str, pk: u64, values: &[u64]) -> Result<(), SdkError> {
        let addr = self.addr.clone();
        let table = table.to_string();
        let values = values.to_vec();
        tokio::task::spawn_blocking(move || {
            let mut c = vectorvolt_net::Client::connect(&addr).map_err(Self::map_err)?;
            c.use_table(&table).map_err(Self::map_err)?;
            c.insert(pk, values).map_err(Self::map_err)
        })
        .await
        .map_err(|e| SdkError::Op(e.to_string()))?
    }

    /// Get a row by primary key.
    pub async fn get(&self, table: &str, pk: u64) -> Result<Option<Vec<u64>>, SdkError> {
        let addr = self.addr.clone();
        let table = table.to_string();
        tokio::task::spawn_blocking(move || {
            let mut c = vectorvolt_net::Client::connect(&addr).map_err(Self::map_err)?;
            c.use_table(&table).map_err(Self::map_err)?;
            c.get(pk).map_err(Self::map_err)
        })
        .await
        .map_err(|e| SdkError::Op(e.to_string()))?
    }

    /// Delete a row by primary key.
    pub async fn delete(&self, table: &str, pk: u64) -> Result<(), SdkError> {
        let addr = self.addr.clone();
        let table = table.to_string();
        tokio::task::spawn_blocking(move || {
            let mut c = vectorvolt_net::Client::connect(&addr).map_err(Self::map_err)?;
            c.use_table(&table).map_err(Self::map_err)?;
            c.delete(pk).map_err(Self::map_err).map(|_| ())
        })
        .await
        .map_err(|e| SdkError::Op(e.to_string()))?
    }

    /// Scan all rows in a table via full range scan.
    pub async fn scan(&self, table: &str) -> Result<Vec<(u64, Vec<u64>)>, SdkError> {
        let addr = self.addr.clone();
        let table = table.to_string();
        tokio::task::spawn_blocking(move || {
            let mut c = vectorvolt_net::Client::connect(&addr).map_err(Self::map_err)?;
            c.use_table(&table).map_err(Self::map_err)?;
            let rows = c.range_scan(0, 0, u64::MAX, 0).map_err(Self::map_err)?;
            Ok(rows
                .into_iter()
                .enumerate()
                .map(|(i, vals)| (i as u64, vals))
                .collect())
        })
        .await
        .map_err(|e| SdkError::Op(e.to_string()))?
    }

    /// Ping the server and return round-trip duration.
    pub async fn ping(&self) -> Result<std::time::Duration, SdkError> {
        let addr = self.addr.clone();
        tokio::task::spawn_blocking(move || {
            let start = std::time::Instant::now();
            let mut c = vectorvolt_net::Client::connect(&addr).map_err(Self::map_err)?;
            c.ping().map_err(Self::map_err)?;
            Ok(start.elapsed())
        })
        .await
        .map_err(|e| SdkError::Op(e.to_string()))?
    }

    /// Return a health status string.
    pub async fn health(&self) -> Result<String, SdkError> {
        let addr = self.addr.clone();
        tokio::task::spawn_blocking(move || {
            let mut c = vectorvolt_net::Client::connect(&addr).map_err(Self::map_err)?;
            let (rows, _parts, bytes) = c.stats().map_err(Self::map_err)?;
            Ok(format!("ok rows={rows} bytes={bytes}"))
        })
        .await
        .map_err(|e| SdkError::Op(e.to_string()))?
    }

    /// Begin a transaction and return the txn_id.
    pub async fn begin_txn(&self) -> Result<u64, SdkError> {
        let addr = self.addr.clone();
        tokio::task::spawn_blocking(move || {
            let mut c = vectorvolt_net::Client::connect(&addr).map_err(Self::map_err)?;
            c.begin_txn().map_err(Self::map_err)
        })
        .await
        .map_err(|e| SdkError::Op(e.to_string()))?
    }

    /// Commit a transaction.
    pub async fn commit_txn(&self, txn_id: u64) -> Result<(), SdkError> {
        let addr = self.addr.clone();
        tokio::task::spawn_blocking(move || {
            let mut c = vectorvolt_net::Client::connect(&addr).map_err(Self::map_err)?;
            c.commit_txn(txn_id).map_err(Self::map_err).map(|_| ())
        })
        .await
        .map_err(|e| SdkError::Op(e.to_string()))?
    }

    /// Abort/rollback a transaction.
    pub async fn abort_txn(&self, txn_id: u64) -> Result<(), SdkError> {
        let addr = self.addr.clone();
        tokio::task::spawn_blocking(move || {
            let mut c = vectorvolt_net::Client::connect(&addr).map_err(Self::map_err)?;
            c.rollback_txn(txn_id).map_err(Self::map_err)
        })
        .await
        .map_err(|e| SdkError::Op(e.to_string()))?
    }
}
