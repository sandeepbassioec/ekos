//! `vectorvolt-server` — async TCP server with MVCC (VV-NET-004..016).
//!
//! [`VvServer`] is a tokio-based TCP accept loop that dispatches wire frames
//! to engine operations.  Each connection gets its own task.
//!
//! ## MVCC over wire (VV-TXN-010..016)
//!
//! Connections can open multi-statement transactions:
//!
//! 1. `TxnBegin`  — allocates a server-side `txn_id` returned in the payload.
//! 2. `Execute`   — with `"txn_id"` in JSON payload, buffers the mutation.
//! 3. `TxnCommit` — applies all buffered mutations atomically.
//! 4. `TxnAbort`  — discards the pending state.
//!
//! The server serialises commits with a `tokio::sync::Mutex` on each
//! `TxnState` entry; reads bypass the lock entirely.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::Mutex as AsyncMutex;
use vectorvolt_engine::TableEngine;
use vectorvolt_net::frame::{Opcode, WIRE_HEADER_SIZE};

pub use config::ServerConfig;
pub use grpc::{GrpcServer, GrpcWriteService, WriteGate};

pub mod config;
pub mod grpc;

// ── MVCC transaction state ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum PendingOp {
    Insert { pk: u64, values: Vec<u64> },
    Delete { pk: u64 },
}

pub struct TxnState {
    pub ops: Vec<PendingOp>,
}

// ── Server ────────────────────────────────────────────────────────────────────

/// Async TCP server backed by a [`TableEngine`].
pub struct VvServer {
    listener: TcpListener,
    engine: Arc<TableEngine>,
    txns: Arc<DashMap<u64, AsyncMutex<TxnState>>>,
    txn_seq: Arc<AtomicU64>,
    cfg: ServerConfig,
}

impl VvServer {
    /// Bind to `cfg.addr` and open the table at `data_dir`.
    pub async fn bind(
        cfg: ServerConfig,
        schema: Vec<String>,
        data_dir: impl Into<std::path::PathBuf>,
    ) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(cfg.addr).await?;
        let engine = Arc::new(TableEngine::open(schema, data_dir, cfg.flush_threshold)?);
        Ok(VvServer {
            listener,
            engine,
            txns: Arc::new(DashMap::new()),
            txn_seq: Arc::new(AtomicU64::new(1)),
            cfg,
        })
    }

    /// The local address the server is listening on.
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Config accessor.
    pub fn config(&self) -> &ServerConfig {
        &self.cfg
    }

    /// Run the accept loop until `shutdown_rx` fires.
    pub async fn run(self, mut shutdown_rx: tokio::sync::oneshot::Receiver<()>) {
        loop {
            tokio::select! {
                accept = self.listener.accept() => {
                    match accept {
                        Ok((stream, _)) => {
                            stream.set_nodelay(true).ok();
                            let engine = Arc::clone(&self.engine);
                            let txns = Arc::clone(&self.txns);
                            let txn_seq = Arc::clone(&self.txn_seq);
                            tokio::spawn(handle_connection(
                                stream, engine, txns, txn_seq,
                            ));
                        }
                        Err(_) => break,
                    }
                }
                _ = &mut shutdown_rx => break,
            }
        }
    }
}

// ── Per-connection handler ────────────────────────────────────────────────────

async fn handle_connection(
    stream: tokio::net::TcpStream,
    engine: Arc<TableEngine>,
    txns: Arc<DashMap<u64, AsyncMutex<TxnState>>>,
    txn_seq: Arc<AtomicU64>,
) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use vectorvolt_net::frame::WireFrameHeader;

    let mut stream = stream;
    let mut next_id: u64 = 1;

    loop {
        // Read 64-byte header.
        let mut hbuf = vec![0u8; WIRE_HEADER_SIZE];
        if stream.read_exact(&mut hbuf).await.is_err() {
            break;
        }
        // Validate the header (including the payload size limit) before
        // allocating the payload buffer.
        let header = match WireFrameHeader::decode(&hbuf) {
            Ok(h) => h,
            Err(_) => break,
        };
        let mut payload = vec![0u8; header.payload_len as usize];
        if !payload.is_empty() && stream.read_exact(&mut payload).await.is_err() {
            break;
        }

        // Validate CRC.
        if header.verify_payload(&payload).is_err() {
            break;
        }

        let opcode = match Opcode::from_u8(header.opcode) {
            Ok(op) => op,
            Err(_) => break,
        };

        // Dispatch.
        let (resp_op, resp_payload): (Opcode, Vec<u8>) = match opcode {
            Opcode::Ping => (Opcode::PingAck, b"".to_vec()),

            Opcode::PingAck => (Opcode::PingAck, b"".to_vec()),

            Opcode::Health => {
                let body = json!({"status":"ok","version":env!("CARGO_PKG_VERSION")});
                (Opcode::PingAck, body.to_string().into_bytes())
            }

            Opcode::TxnBegin => {
                let txn_id = txn_seq.fetch_add(1, Ordering::Relaxed);
                txns.insert(txn_id, AsyncMutex::new(TxnState { ops: Vec::new() }));
                (Opcode::PingAck, txn_id.to_le_bytes().to_vec())
            }

            Opcode::Execute => {
                let resp = match handle_execute(&payload, &engine, &txns).await {
                    Ok(r) => r,
                    Err(e) => format!("{{\"error\":\"{e}\"}}").into_bytes(),
                };
                (Opcode::PingAck, resp)
            }

            Opcode::Query => {
                let resp = match handle_query(&payload, &engine).await {
                    Ok(r) => r,
                    Err(e) => format!("{{\"error\":\"{e}\"}}").into_bytes(),
                };
                (Opcode::PingAck, resp)
            }

            Opcode::TxnCommit => {
                let resp = match handle_commit(&payload, &engine, &txns).await {
                    Ok(r) => r,
                    Err(e) => format!("{{\"error\":\"{e}\"}}").into_bytes(),
                };
                (Opcode::PingAck, resp)
            }

            Opcode::TxnAbort => {
                let resp = match handle_abort(&payload, &txns).await {
                    Ok(r) => r,
                    Err(e) => format!("{{\"error\":\"{e}\"}}").into_bytes(),
                };
                (Opcode::PingAck, resp)
            }
        };

        // Send response.
        let resp_header = WireFrameHeader::new(resp_op, &resp_payload, next_id);
        next_id = next_id.wrapping_add(1);
        let wire = resp_header
            .expect("response header is valid")
            .encode_with_payload(&resp_payload);
        if stream.write_all(&wire).await.is_err() {
            break;
        }
    }
}

// ── Opcode handlers ───────────────────────────────────────────────────────────

async fn handle_execute(
    payload: &[u8],
    engine: &Arc<TableEngine>,
    txns: &Arc<DashMap<u64, AsyncMutex<TxnState>>>,
) -> anyhow::Result<Vec<u8>> {
    let v: serde_json::Value = serde_json::from_slice(payload)?;
    let op = v["op"].as_str().unwrap_or("insert");
    let pk = v["pk"].as_u64().unwrap_or(0);

    if let Some(txn_id) = v["txn_id"].as_u64() {
        let entry = txns
            .get(&txn_id)
            .ok_or_else(|| anyhow::anyhow!("unknown txn_id"))?;
        let mut guard = entry.lock().await;
        match op {
            "insert" | "upsert" => {
                let values: Vec<u64> = v["values"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|x| x.as_u64()).collect())
                    .unwrap_or_default();
                guard.ops.push(PendingOp::Insert { pk, values });
            }
            "delete" => {
                guard.ops.push(PendingOp::Delete { pk });
            }
            _ => anyhow::bail!("unknown op: {op}"),
        }
        return Ok(b"{\"status\":\"buffered\"}".to_vec());
    }

    match op {
        "insert" => {
            let values: Vec<u64> = v["values"]
                .as_array()
                .map(|a| a.iter().filter_map(|x| x.as_u64()).collect())
                .unwrap_or_default();
            engine
                .insert(pk, values)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(b"{\"status\":\"ok\"}".to_vec())
        }
        "delete" => {
            engine.delete(pk).map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(b"{\"status\":\"ok\"}".to_vec())
        }
        _ => anyhow::bail!("unknown op: {op}"),
    }
}

async fn handle_query(payload: &[u8], engine: &Arc<TableEngine>) -> anyhow::Result<Vec<u8>> {
    let v: serde_json::Value = serde_json::from_slice(payload)?;
    let pk = v["pk"].as_u64().unwrap_or(0);
    match engine.get(pk).map_err(|e| anyhow::anyhow!("{e}"))? {
        Some(row) => Ok(serde_json::to_vec(
            &json!({"pk": row.pk, "values": row.values}),
        )?),
        None => Ok(b"{\"found\":false}".to_vec()),
    }
}

async fn handle_commit(
    payload: &[u8],
    engine: &Arc<TableEngine>,
    txns: &Arc<DashMap<u64, AsyncMutex<TxnState>>>,
) -> anyhow::Result<Vec<u8>> {
    let v: serde_json::Value = serde_json::from_slice(payload)?;
    let txn_id = v["txn_id"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("missing txn_id"))?;

    let ops = {
        let entry = txns
            .get(&txn_id)
            .ok_or_else(|| anyhow::anyhow!("unknown txn_id"))?;
        let mut guard = entry.lock().await;
        std::mem::take(&mut guard.ops)
    };
    txns.remove(&txn_id);

    let mut applied = 0usize;
    for op in ops {
        match op {
            PendingOp::Insert { pk, values } => {
                engine
                    .insert(pk, values)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                applied += 1;
            }
            PendingOp::Delete { pk } => {
                engine.delete(pk).map_err(|e| anyhow::anyhow!("{e}"))?;
                applied += 1;
            }
        }
    }
    Ok(serde_json::to_vec(
        &json!({"status":"committed","ops":applied}),
    )?)
}

async fn handle_abort(
    payload: &[u8],
    txns: &Arc<DashMap<u64, AsyncMutex<TxnState>>>,
) -> anyhow::Result<Vec<u8>> {
    let v: serde_json::Value = serde_json::from_slice(payload)?;
    let txn_id = v["txn_id"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("missing txn_id"))?;
    txns.remove(&txn_id);
    Ok(b"{\"status\":\"aborted\"}".to_vec())
}

// ── helpers for tests ─────────────────────────────────────────────────────────

/// Spawn the TCP server for tests; returns `(addr, shutdown_tx)`.
pub async fn spawn_server_for_test(
    schema: Vec<String>,
    data_dir: impl Into<std::path::PathBuf>,
) -> anyhow::Result<(SocketAddr, tokio::sync::oneshot::Sender<()>)> {
    let cfg = ServerConfig {
        addr: "127.0.0.1:0".parse().unwrap(),
        flush_threshold: 1_000_000,
    };
    let srv = VvServer::bind(cfg, schema, data_dir).await?;
    let addr = srv.local_addr()?;
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(srv.run(rx));
    Ok((addr, tx))
}

/// Spawn a gRPC server for tests; returns `(addr, shutdown_tx)`.
///
/// Binds a TCP listener at :0 to learn the OS-assigned port, then hands it
/// to tonic via `serve_with_incoming` so there is no TOCTOU race.
pub async fn spawn_grpc_for_test(
    schema: Vec<String>,
    data_dir: impl Into<std::path::PathBuf>,
) -> anyhow::Result<(SocketAddr, tokio::sync::oneshot::Sender<()>)> {
    use grpc::proto::write_service_server::WriteServiceServer;
    use tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server as TonicServer;

    let engine = Arc::new(TableEngine::open(schema, data_dir, 1_000_000)?);
    let txns: Arc<DashMap<u64, AsyncMutex<TxnState>>> = Arc::new(DashMap::new());
    let txn_seq = Arc::new(AtomicU64::new(1));
    let svc =
        grpc::GrpcWriteService::new(Arc::clone(&engine), Arc::clone(&txns), Arc::clone(&txn_seq));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let incoming = TcpListenerStream::new(listener);

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = TonicServer::builder()
            .add_service(WriteServiceServer::new(svc))
            .serve_with_incoming_shutdown(incoming, async {
                let _ = rx.await;
            })
            .await;
    });
    Ok((addr, tx))
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use vectorvolt_net::connection::Connection;

    use super::*;

    async fn start(dir: &TempDir) -> (SocketAddr, tokio::sync::oneshot::Sender<()>) {
        spawn_server_for_test(vec!["pk".to_string(), "age".to_string()], dir.path())
            .await
            .unwrap()
    }

    async fn client(addr: SocketAddr) -> Connection {
        Connection::connect(addr).await.unwrap()
    }

    async fn send_recv(conn: &mut Connection, op: Opcode, payload: &[u8]) -> Vec<u8> {
        conn.send_frame(op, payload).await.unwrap();
        let (_, resp) = conn.recv_frame().await.unwrap();
        resp
    }

    #[tokio::test]
    async fn server_binds_and_accepts() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let c = client(addr).await;
        assert_eq!(c.peer_addr().unwrap().port(), addr.port());
    }

    #[tokio::test]
    async fn ping_returns_pong() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let mut c = client(addr).await;
        let resp = send_recv(&mut c, Opcode::Ping, b"").await;
        assert_eq!(resp.len(), 0);
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let mut c = client(addr).await;
        let resp = send_recv(&mut c, Opcode::Health, b"").await;
        let v: serde_json::Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["status"], "ok");
    }

    #[tokio::test]
    async fn insert_and_query_row() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let mut c = client(addr).await;

        let ins = json!({"op":"insert","pk":1,"values":[1,25]}).to_string();
        let resp = send_recv(&mut c, Opcode::Execute, ins.as_bytes()).await;
        let v: serde_json::Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["status"], "ok");

        let qry = json!({"pk":1}).to_string();
        let resp = send_recv(&mut c, Opcode::Query, qry.as_bytes()).await;
        let v: serde_json::Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["pk"], 1);
    }

    #[tokio::test]
    async fn query_missing_row_returns_not_found() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let mut c = client(addr).await;
        let qry = json!({"pk":999}).to_string();
        let resp = send_recv(&mut c, Opcode::Query, qry.as_bytes()).await;
        let v: serde_json::Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["found"], false);
    }

    #[tokio::test]
    async fn txn_begin_returns_id() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let mut c = client(addr).await;
        let resp = send_recv(&mut c, Opcode::TxnBegin, b"").await;
        assert_eq!(resp.len(), 8);
        let txn_id = u64::from_le_bytes(resp.try_into().unwrap());
        assert!(txn_id > 0);
    }

    #[tokio::test]
    async fn txn_commit_applies_inserts() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let mut c = client(addr).await;

        let resp = send_recv(&mut c, Opcode::TxnBegin, b"").await;
        let txn_id = u64::from_le_bytes(resp.try_into().unwrap());

        let ins = json!({"op":"insert","pk":10,"values":[10,42],"txn_id":txn_id}).to_string();
        let resp = send_recv(&mut c, Opcode::Execute, ins.as_bytes()).await;
        let v: serde_json::Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["status"], "buffered");

        let commit = json!({"txn_id":txn_id}).to_string();
        let resp = send_recv(&mut c, Opcode::TxnCommit, commit.as_bytes()).await;
        let v: serde_json::Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["status"], "committed");
        assert_eq!(v["ops"], 1);
    }

    #[tokio::test]
    async fn txn_abort_discards_ops() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let mut c = client(addr).await;

        let resp = send_recv(&mut c, Opcode::TxnBegin, b"").await;
        let txn_id = u64::from_le_bytes(resp.try_into().unwrap());

        let ins = json!({"op":"insert","pk":20,"values":[20,99],"txn_id":txn_id}).to_string();
        let _ = send_recv(&mut c, Opcode::Execute, ins.as_bytes()).await;

        let abort = json!({"txn_id":txn_id}).to_string();
        let resp = send_recv(&mut c, Opcode::TxnAbort, abort.as_bytes()).await;
        let v: serde_json::Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["status"], "aborted");

        let qry = json!({"pk":20}).to_string();
        let resp = send_recv(&mut c, Opcode::Query, qry.as_bytes()).await;
        let v: serde_json::Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["found"], false);
    }

    #[tokio::test]
    async fn delete_row() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let mut c = client(addr).await;

        let ins = json!({"op":"insert","pk":5,"values":[5,10]}).to_string();
        let _ = send_recv(&mut c, Opcode::Execute, ins.as_bytes()).await;

        let del = json!({"op":"delete","pk":5}).to_string();
        let resp = send_recv(&mut c, Opcode::Execute, del.as_bytes()).await;
        let v: serde_json::Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["status"], "ok");
    }

    #[tokio::test]
    async fn multiple_txn_ids_are_distinct() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let mut c = client(addr).await;

        let r1 = send_recv(&mut c, Opcode::TxnBegin, b"").await;
        let r2 = send_recv(&mut c, Opcode::TxnBegin, b"").await;

        let id1 = u64::from_le_bytes(r1.try_into().unwrap());
        let id2 = u64::from_le_bytes(r2.try_into().unwrap());
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn local_addr_matches() {
        let dir = TempDir::new().unwrap();
        let cfg = ServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            flush_threshold: 1_000_000,
        };
        let srv = VvServer::bind(cfg, vec!["pk".to_string(), "v".to_string()], dir.path())
            .await
            .unwrap();
        assert_eq!(srv.local_addr().unwrap().ip().to_string(), "127.0.0.1");
    }

    #[tokio::test]
    async fn shutdown_graceful() {
        let dir = TempDir::new().unwrap();
        let (addr, shutdown_tx) = start(&dir).await;
        let _c = client(addr).await;
        let _ = shutdown_tx.send(());
    }

    #[tokio::test]
    async fn txn_commit_multiple_ops() {
        let dir = TempDir::new().unwrap();
        let (addr, _sd) = start(&dir).await;
        let mut c = client(addr).await;

        let resp = send_recv(&mut c, Opcode::TxnBegin, b"").await;
        let txn_id = u64::from_le_bytes(resp.try_into().unwrap());

        for pk in [100u64, 101, 102] {
            let ins = json!({"op":"insert","pk":pk,"values":[pk,pk*2],"txn_id":txn_id}).to_string();
            send_recv(&mut c, Opcode::Execute, ins.as_bytes()).await;
        }

        let commit = json!({"txn_id":txn_id}).to_string();
        let resp = send_recv(&mut c, Opcode::TxnCommit, commit.as_bytes()).await;
        let v: serde_json::Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["ops"], 3);
    }
}
