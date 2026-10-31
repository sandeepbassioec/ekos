//! gRPC write server — tonic implementation of `WriteService` (VV-NET-004..016 / S16).
//!
//! Runs on a separate port (default :50051) alongside the existing TCP server.
//! All RPCs share the same `TableEngine` Arc and the same MVCC `DashMap`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use dashmap::DashMap;
use tokio::sync::Mutex as AsyncMutex;
use tonic::{Request, Response, Status, transport::Server};
use vectorvolt_engine::TableEngine;

use crate::TxnState;

// Generated protobuf + tonic stubs.
pub mod proto {
    tonic::include_proto!("vectorvolt");
}

use proto::{
    AbortTxnRequest, BeginTxnReply, BeginTxnRequest, CommitReply, CommitTxnRequest, DeleteRequest,
    ExecuteRequest, InsertRequest, PingReply, PingRequest, QueryReply, QueryRequest, WriteReply,
    write_service_server::{WriteService, WriteServiceServer},
};

// ── Service state ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct GrpcWriteService {
    engine: Arc<TableEngine>,
    txns: Arc<DashMap<u64, AsyncMutex<TxnState>>>,
    txn_seq: Arc<AtomicU64>,
}

impl GrpcWriteService {
    pub fn new(
        engine: Arc<TableEngine>,
        txns: Arc<DashMap<u64, AsyncMutex<TxnState>>>,
        txn_seq: Arc<AtomicU64>,
    ) -> Self {
        Self {
            engine,
            txns,
            txn_seq,
        }
    }
}

// ── tonic service impl ────────────────────────────────────────────────────────

#[tonic::async_trait]
impl WriteService for GrpcWriteService {
    async fn ping(&self, _req: Request<PingRequest>) -> Result<Response<PingReply>, Status> {
        Ok(Response::new(PingReply {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }))
    }

    async fn insert(&self, req: Request<InsertRequest>) -> Result<Response<WriteReply>, Status> {
        let r = req.into_inner();
        self.engine
            .insert(r.pk, r.values)
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(WriteReply {
            status: "ok".into(),
            error: String::new(),
        }))
    }

    async fn delete(&self, req: Request<DeleteRequest>) -> Result<Response<WriteReply>, Status> {
        let r = req.into_inner();
        self.engine
            .delete(r.pk)
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(WriteReply {
            status: "ok".into(),
            error: String::new(),
        }))
    }

    async fn query(&self, req: Request<QueryRequest>) -> Result<Response<QueryReply>, Status> {
        let r = req.into_inner();
        match self
            .engine
            .get(r.pk)
            .map_err(|e| Status::internal(e.to_string()))?
        {
            Some(row) => Ok(Response::new(QueryReply {
                found: true,
                pk: row.pk,
                values: row.values,
            })),
            None => Ok(Response::new(QueryReply {
                found: false,
                pk: 0,
                values: vec![],
            })),
        }
    }

    async fn begin_txn(
        &self,
        _req: Request<BeginTxnRequest>,
    ) -> Result<Response<BeginTxnReply>, Status> {
        let txn_id = self.txn_seq.fetch_add(1, Ordering::Relaxed);
        self.txns
            .insert(txn_id, AsyncMutex::new(TxnState { ops: Vec::new() }));
        Ok(Response::new(BeginTxnReply { txn_id }))
    }

    async fn execute(&self, req: Request<ExecuteRequest>) -> Result<Response<WriteReply>, Status> {
        let r = req.into_inner();
        let entry = self
            .txns
            .get(&r.txn_id)
            .ok_or_else(|| Status::not_found("unknown txn_id"))?;
        let mut guard = entry.lock().await;
        match r.op.as_str() {
            "insert" | "upsert" => {
                guard.ops.push(crate::PendingOp::Insert {
                    pk: r.pk,
                    values: r.values,
                });
            }
            "delete" => {
                guard.ops.push(crate::PendingOp::Delete { pk: r.pk });
            }
            op => return Err(Status::invalid_argument(format!("unknown op: {op}"))),
        }
        Ok(Response::new(WriteReply {
            status: "buffered".into(),
            error: String::new(),
        }))
    }

    async fn commit_txn(
        &self,
        req: Request<CommitTxnRequest>,
    ) -> Result<Response<CommitReply>, Status> {
        let txn_id = req.into_inner().txn_id;
        let ops = {
            let entry = self
                .txns
                .get(&txn_id)
                .ok_or_else(|| Status::not_found("unknown txn_id"))?;
            let mut guard = entry.lock().await;
            std::mem::take(&mut guard.ops)
        };
        self.txns.remove(&txn_id);

        // All ops as one atomic WAL record (see `TableEngine::apply_batch`).
        let applied = ops.len() as u64;
        let batch = ops
            .into_iter()
            .map(|op| match op {
                crate::PendingOp::Insert { pk, values } => (pk, Some(values)),
                crate::PendingOp::Delete { pk } => (pk, None),
            })
            .collect();
        self.engine
            .apply_batch(batch)
            .map_err(|e| Status::internal(format!("commit failed, nothing applied: {e}")))?;
        Ok(Response::new(CommitReply {
            status: "committed".into(),
            ops: applied,
        }))
    }

    async fn abort_txn(
        &self,
        req: Request<AbortTxnRequest>,
    ) -> Result<Response<WriteReply>, Status> {
        let txn_id = req.into_inner().txn_id;
        self.txns.remove(&txn_id);
        Ok(Response::new(WriteReply {
            status: "aborted".into(),
            error: String::new(),
        }))
    }
}

// ── GrpcServer ────────────────────────────────────────────────────────────────

/// Wraps `WriteServiceServer` with a tokio-graceful shutdown.
pub struct GrpcServer {
    addr: SocketAddr,
    svc: GrpcWriteService,
    /// When set, every RPC must carry `authorization: Bearer <token>`.
    token: Option<Arc<str>>,
}

/// Constant-time equality, so a wrong token cannot be found byte by byte by
/// timing the rejection.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Reject any request whose bearer token does not match `expected`.
///
/// `Status` is large, but it is the error type tonic interceptors must return.
#[allow(clippy::result_large_err)]
fn check_token(expected: &str, req: &Request<()>) -> Result<(), Status> {
    let presented = req
        .metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    match presented {
        Some(t) if ct_eq(t.as_bytes(), expected.as_bytes()) => Ok(()),
        _ => Err(Status::unauthenticated("missing or invalid bearer token")),
    }
}

impl GrpcServer {
    pub fn new(
        addr: SocketAddr,
        engine: Arc<TableEngine>,
        txns: Arc<DashMap<u64, AsyncMutex<TxnState>>>,
        txn_seq: Arc<AtomicU64>,
    ) -> Self {
        Self {
            addr,
            svc: GrpcWriteService::new(engine, txns, txn_seq),
            token: None,
        }
    }

    /// Require `authorization: Bearer <token>` on every RPC.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(Arc::from(token.into()));
        self
    }

    /// Convenience constructor that creates its own MVCC txn state.
    pub fn standalone(addr: SocketAddr, engine: Arc<TableEngine>) -> Self {
        let txns = Arc::new(DashMap::new());
        let txn_seq = Arc::new(AtomicU64::new(1));
        Self::new(addr, engine, txns, txn_seq)
    }

    /// Serve until `shutdown_rx` fires.
    #[allow(clippy::result_large_err)] // the interceptor closure must return tonic's `Status`
    pub async fn run(self, shutdown_rx: tokio::sync::oneshot::Receiver<()>) -> anyhow::Result<()> {
        let token = self.token.clone();
        let svc =
            WriteServiceServer::with_interceptor(self.svc, move |req: Request<()>| match &token {
                Some(t) => check_token(t, &req).map(|()| req),
                None => Ok(req),
            });
        Server::builder()
            .add_service(svc)
            .serve_with_shutdown(self.addr, async {
                let _ = shutdown_rx.await;
            })
            .await?;
        Ok(())
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
