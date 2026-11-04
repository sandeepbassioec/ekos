//! [`Server`] — a TCP server over a [`TableRegistry`] (VV-NET-004/005).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tracing::{debug, info_span};

use vectorvolt_engine::{TableEngine, TableRegistry};

use crate::auth::{AuthConfig, AuthState, Scope};
use crate::protocol::{HEADER_SIZE, Header, Request, Response, encode_response, verify_payload};
use crate::tls::{ServerConn, ServerTlsConfig};
use crate::transport::TransportKind;
use crate::{NetError, NetResult, error_code};

// ── Per-connection transaction state ─────────────────────────────────────────

enum TxnOp {
    Insert { pk: u64, values: Vec<u64> },
    Delete { pk: u64 },
}

struct Txn {
    id: u64,
    ops: Vec<TxnOp>,
}

/// Server tuning.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub addr: SocketAddr,
    /// Connections served concurrently. Beyond this, newly accepted sockets
    /// are closed immediately (and counted in
    /// [`ServerStats::connections_refused`]) rather than queued.
    ///
    /// A bound rather than unlimited: one thread per connection means an
    /// unbounded server lets a peer exhaust process threads by connecting
    /// repeatedly, which costs nothing on their side.
    pub max_connections: usize,
    /// Disable Nagle's algorithm.
    ///
    /// On by default. Nagle delays a small write waiting for more data to
    /// coalesce, which is exactly wrong for a request/response protocol where the
    /// response IS the whole message — it can add up to 40 ms to a reply that took
    /// microseconds to compute.
    pub nodelay: bool,
    /// Who may connect. Empty means authentication is disabled.
    pub auth: AuthConfig,
    /// TLS. `None` means plaintext.
    ///
    /// Separate from `auth` because they answer different questions and are
    /// independently useful: TLS without auth is an encrypted channel to an
    /// anonymous peer, auth without TLS is a known peer over a readable channel.
    /// Production wants both; the two knobs make it visible when only one is set.
    pub tls: Option<ServerTlsConfig>,

    /// Network transport to use for new connections (default: TCP).
    ///
    /// Set to [`TransportKind::Xdp`] (Linux only) to bypass the kernel TCP
    /// stack and receive frames directly via AF_XDP.  Requires `CAP_NET_RAW`.
    pub transport: TransportKind,
}

impl ServerConfig {
    /// Require authentication with the given credentials.
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }

    /// True if connections must authenticate before doing anything.
    pub fn requires_auth(&self) -> bool {
        self.auth.is_enabled()
    }

    /// Serve over TLS.
    pub fn with_tls(mut self, tls: ServerTlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// True if the transport is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.tls.is_some()
    }

    pub fn new(addr: SocketAddr) -> Self {
        ServerConfig {
            addr,
            max_connections: 256,
            nodelay: true,
            auth: AuthConfig::disabled(),
            tls: None,
            transport: TransportKind::Tcp,
        }
    }

    /// Use the given transport instead of TCP.
    pub fn with_transport(mut self, transport: TransportKind) -> Self {
        self.transport = transport;
        self
    }
}

/// Counters, for tests and operational visibility.
#[derive(Debug, Default)]
pub struct ServerStats {
    pub connections_accepted: AtomicU64,
    pub requests_handled: AtomicU64,
    pub protocol_errors: AtomicU64,
    /// Requests refused because the connection had not authenticated.
    pub unauthenticated_rejections: AtomicU64,
    /// Requests refused because the key lacked the required scope.
    pub forbidden_rejections: AtomicU64,
    /// Failed authentication attempts. Worth alerting on: a rising count is
    /// either a misconfigured client or someone guessing.
    pub auth_failures: AtomicU64,
    pub auth_successes: AtomicU64,
    /// Connections dropped because the TLS handshake failed.
    ///
    /// Counted separately from protocol errors: a handshake failure is a
    /// certificate or trust problem, not a malformed frame, and conflating them
    /// hides the difference between a misconfigured client and a hostile one.
    pub tls_handshake_failures: AtomicU64,
    /// Connections closed on accept because `max_connections` were already live.
    pub connections_refused: AtomicU64,
}

/// A running server.
pub struct Server {
    registry: Arc<TableRegistry>,
    config: ServerConfig,
    listener: TcpListener,
    shutdown: Arc<AtomicBool>,
    stats: Arc<ServerStats>,
    live: Arc<AtomicU64>,
}

impl Server {
    /// Single-table backward-compatible constructor.
    ///
    /// Wraps `engine` in a [`TableRegistry`] under the name `"default"` and
    /// auto-selects it for every new connection. Existing callers and tests that
    /// do not send `UseTable` continue to work unchanged.
    pub fn bind(engine: Arc<TableEngine>, config: ServerConfig) -> NetResult<Self> {
        Self::bind_registry(Arc::new(TableRegistry::new_with_default(engine)), config)
    }

    /// Multi-table constructor. New connections auto-select `"default"` if the
    /// registry contains it; otherwise the client must send `UseTable` first.
    pub fn bind_registry(registry: Arc<TableRegistry>, config: ServerConfig) -> NetResult<Self> {
        let listener = TcpListener::bind(config.addr)?;
        Ok(Server {
            registry,
            config,
            listener,
            shutdown: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(ServerStats::default()),
            live: Arc::new(AtomicU64::new(0)),
        })
    }

    /// The address actually bound. Differs from the requested one when port 0 was
    /// asked for.
    pub fn local_addr(&self) -> NetResult<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    pub fn stats(&self) -> Arc<ServerStats> {
        Arc::clone(&self.stats)
    }

    /// A handle that stops the accept loop.
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle {
            flag: Arc::clone(&self.shutdown),
            addr: self.listener.local_addr().ok(),
        }
    }

    /// Accept and serve until shut down. Blocks.
    pub fn serve(self) -> NetResult<()> {
        for stream in self.listener.incoming() {
            if self.shutdown.load(Ordering::Acquire) {
                break;
            }
            let stream = match stream {
                Ok(s) => s,
                // One failed accept is not fatal: a peer that connects and
                // immediately resets produces an error here, and dying on it would
                // let any client take the server down.
                Err(_) => continue,
            };

            if self.live.load(Ordering::Acquire) as usize >= self.config.max_connections {
                // Refuse by closing rather than queueing. A client that gets a
                // closed socket can back off; one left hanging cannot tell the
                // difference between a busy server and a dead one.
                self.stats
                    .connections_refused
                    .fetch_add(1, Ordering::Relaxed);
                drop(stream);
                continue;
            }

            if self.config.nodelay {
                let _ = stream.set_nodelay(true);
            }
            self.stats
                .connections_accepted
                .fetch_add(1, Ordering::Relaxed);
            self.live.fetch_add(1, Ordering::AcqRel);

            let peer = stream.peer_addr().ok();
            debug!(peer = ?peer, "accepted connection");

            let registry = Arc::clone(&self.registry);
            let stats = Arc::clone(&self.stats);
            let live = Arc::clone(&self.live);
            let shutdown = Arc::clone(&self.shutdown);
            let auth = self.config.auth.clone();
            let tls = self.config.tls.clone();
            std::thread::spawn(move || {
                let span = info_span!("vv.connection", peer = ?peer);
                let _enter = span.enter();
                // The handshake runs on the connection thread, not the accept
                // loop. It involves several round trips, so doing it inline would
                // let one slow or hostile peer stall every pending accept — a
                // denial of service that costs the attacker a single socket.
                match ServerConn::accept(stream, tls.as_ref()) {
                    Ok(conn) => {
                        let _ = handle_connection(conn, &registry, &stats, &shutdown, &auth);
                    }
                    Err(_) => {
                        stats.tls_handshake_failures.fetch_add(1, Ordering::Relaxed);
                    }
                }
                live.fetch_sub(1, Ordering::AcqRel);
            });
        }
        Ok(())
    }
}

/// Stops a running server.
pub struct ShutdownHandle {
    flag: Arc<AtomicBool>,
    addr: Option<SocketAddr>,
}

impl ShutdownHandle {
    /// Signal shutdown and unblock the accept loop.
    ///
    /// Setting the flag alone is not enough: `incoming()` is blocked inside
    /// `accept` and will not observe it until a connection arrives. A throwaway
    /// connection to our own address is the portable way to wake it — there is no
    /// cross-platform "cancel a blocking accept".
    pub fn shutdown(&self) {
        self.flag.store(true, Ordering::Release);
        if let Some(addr) = self.addr {
            let _ = TcpStream::connect(addr);
        }
    }
}

/// Read exactly `n` bytes, or report why not.
///
/// `read_exact` maps a clean close to `UnexpectedEof`, which is indistinguishable
/// from a truncated frame. They are treated differently: a peer that closes
/// between frames is normal and ends the loop quietly, whereas one that closes
/// mid-frame is a protocol error worth counting.
fn read_exact_or_eof<R: Read>(stream: &mut R, buf: &mut [u8]) -> NetResult<bool> {
    let mut read = 0;
    while read < buf.len() {
        match stream.read(&mut buf[read..]) {
            Ok(0) => {
                return if read == 0 {
                    Ok(false) // clean close on a frame boundary
                } else {
                    Err(NetError::Truncated {
                        need: buf.len(),
                        got: read,
                    })
                };
            }
            Ok(n) => read += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(NetError::Io(e)),
        }
    }
    Ok(true)
}

#[tracing::instrument(skip_all, name = "vv.handle_connection")]
fn handle_connection(
    mut stream: ServerConn,
    registry: &TableRegistry,
    stats: &ServerStats,
    shutdown: &AtomicBool,
    auth_config: &AuthConfig,
) -> NetResult<()> {
    let mut header_buf = [0u8; HEADER_SIZE];
    // Auth state is per CONNECTION, not per process.
    let mut auth = AuthState::new(auth_config);
    // Auto-select "default" if it exists, for backward compatibility.
    let mut selected: Option<Arc<TableEngine>> = registry.get("default").ok();
    // Optional active transaction. At most one per connection.
    let mut txn: Option<Txn> = None;
    let mut next_txn_id: u64 = 1;

    loop {
        if shutdown.load(Ordering::Acquire) {
            return Ok(());
        }
        if !read_exact_or_eof(&mut stream, &mut header_buf)? {
            return Ok(());
        }

        let header = match Header::decode(&header_buf) {
            Ok(h) => h,
            Err(e) => {
                stats.protocol_errors.fetch_add(1, Ordering::Relaxed);
                return Err(e);
            }
        };

        let mut payload = vec![0u8; header.payload_len as usize];
        if header.payload_len > 0 && !read_exact_or_eof(&mut stream, &mut payload)? {
            return Err(NetError::Truncated {
                need: header.payload_len as usize,
                got: 0,
            });
        }

        let response = match verify_payload(&header, &payload)
            .and_then(|_| Request::decode(header.opcode, &payload))
        {
            Ok(req) => {
                stats.requests_handled.fetch_add(1, Ordering::Relaxed);
                match serve_authenticated(
                    registry,
                    &mut selected,
                    &mut txn,
                    &mut next_txn_id,
                    req,
                    &mut auth,
                    auth_config,
                    stats,
                ) {
                    Ok(r) => r,
                    Err(r) => {
                        let frame = encode_response(header.request_id, &r)?;
                        stream.write_all(&frame)?;
                        stream.flush()?;
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                stats.protocol_errors.fetch_add(1, Ordering::Relaxed);
                Response::Error {
                    code: error_code::MALFORMED,
                    message: e.to_string(),
                }
            }
        };

        let frame = encode_response(header.request_id, &response)?;
        stream.write_all(&frame)?;
        stream.flush()?;
    }
}

/// Apply the authentication gate, then serve.
///
/// `Err(response)` means "send this and close the connection" (lock-out only).
#[allow(clippy::too_many_arguments)]
fn serve_authenticated(
    registry: &TableRegistry,
    selected: &mut Option<Arc<TableEngine>>,
    txn: &mut Option<Txn>,
    next_txn_id: &mut u64,
    req: Request,
    auth: &mut AuthState,
    config: &AuthConfig,
    stats: &ServerStats,
) -> Result<Response, Response> {
    match req {
        Request::Hello => {
            let nonce = auth.new_challenge();
            Ok(Response::Challenge {
                nonce: nonce.to_vec(),
            })
        }
        Request::Auth { key_id, response } => match auth.verify(config, &key_id, &response) {
            Ok(()) => {
                stats.auth_successes.fetch_add(1, Ordering::Relaxed);
                Ok(Response::Ok { affected: 0 })
            }
            Err(e) => {
                stats.auth_failures.fetch_add(1, Ordering::Relaxed);
                let resp = Response::Error {
                    code: error_code::AUTH_FAILED,
                    message: e.to_string(),
                };
                if auth.is_locked_out() {
                    Err(resp)
                } else {
                    Ok(resp)
                }
            }
        },
        Request::Ping => Ok(Response::Pong),
        other if !auth.is_authenticated() => {
            stats
                .unauthenticated_rejections
                .fetch_add(1, Ordering::Relaxed);
            let _ = other;
            Ok(Response::Error {
                code: error_code::UNAUTHENTICATED,
                message: "authenticate with Hello then Auth before issuing requests".into(),
            })
        }
        other => {
            if let Err(e) = auth.require(required_scope(&other)) {
                stats.forbidden_rejections.fetch_add(1, Ordering::Relaxed);
                return Ok(Response::Error {
                    code: error_code::FORBIDDEN,
                    message: e.to_string(),
                });
            }
            Ok(serve_request(registry, selected, txn, next_txn_id, other))
        }
    }
}

/// The scope an operation needs.
///
/// Exhaustive on purpose. A `_ =>` arm here would silently give every future
/// opcode the scope of whatever it fell through to, which for an authorisation
/// table is the wrong direction to fail in.
fn required_scope(req: &Request) -> Scope {
    match req {
        Request::Get { .. }
        | Request::RangeScan { .. }
        | Request::Stats
        | Request::SecondaryLookup { .. }
        | Request::FkExists { .. }
        | Request::ListTables
        | Request::GetMany { .. }
        | Request::Count { .. }
        | Request::DescribeTable
        | Request::TableExists { .. }
        | Request::ScanKeys { .. }
        | Request::MinMax { .. }
        | Request::Sum { .. }
        | Request::Top { .. }
        | Request::Bottom { .. }
        | Request::PkExists { .. }
        | Request::Average { .. }
        | Request::RankOf { .. }
        | Request::Percentile { .. }
        | Request::SumFields { .. }
        | Request::MaxField { .. }
        | Request::FilterScan { .. }
        | Request::CountDistinct { .. }
        | Request::Median { .. }
        | Request::BatchPkExists { .. }
        | Request::FieldBetween { .. }
        | Request::ScalarMin { .. }
        | Request::ScalarMax { .. }
        | Request::MultiGet { .. }
        | Request::ScanPage { .. }
        | Request::SumField { .. }
        | Request::BatchGet { .. }
        | Request::MaxRow { .. }
        | Request::MinRow { .. }
        | Request::UniqueCount { .. }
        | Request::TopN { .. }
        | Request::BottomN { .. }
        | Request::CountIf { .. }
        | Request::AbsDiff { .. }
        | Request::WindowRank { .. }
        | Request::WindowPct { .. }
        | Request::SumProduct { .. }
        | Request::NthValue { .. }
        | Request::DenseRankWin { .. }
        | Request::MultiColumnSum { .. }
        | Request::FieldHistogram { .. }
        | Request::ZScoreFilter { .. }
        | Request::RunningSum { .. }
        | Request::CountRange { .. }
        | Request::SumRange { .. }
        | Request::MinRange { .. }
        | Request::MaxRange { .. }
        | Request::AvgRange { .. }
        | Request::BatchGetField { .. } => Scope::ReadOnly,
        Request::Insert { .. }
        | Request::Delete { .. }
        | Request::BatchInsert { .. }
        | Request::BatchDelete { .. }
        | Request::Upsert { .. }
        | Request::Update { .. }
        | Request::BatchUpsert { .. }
        | Request::BatchUpdate { .. }
        | Request::GetAndDelete { .. }
        | Request::Increment { .. }
        | Request::CompareAndSwap { .. }
        | Request::SetField { .. }
        | Request::InsertIfAbsent { .. }
        | Request::DeleteRange { .. }
        | Request::PopMin { .. }
        | Request::PopMax { .. }
        | Request::MultiSet { .. }
        | Request::DecrementClamp { .. }
        | Request::GetOrInsert { .. }
        | Request::BatchSetField { .. }
        | Request::IncrementMulti { .. }
        | Request::CopyRow { .. }
        | Request::SwapField { .. }
        | Request::Clamp { .. }
        | Request::DeleteIf { .. }
        | Request::SetFieldIf { .. }
        | Request::BatchIncrementField { .. }
        | Request::BatchDeleteIf { .. }
        | Request::IncrementOrInsert { .. }
        | Request::ZeroField { .. }
        | Request::SetFieldRange { .. }
        | Request::IncrementRange { .. }
        | Request::DeleteMin { .. }
        | Request::DeleteMax { .. }
        | Request::MoveField { .. }
        | Request::SetIfGreater { .. }
        | Request::SetIfLess { .. }
        | Request::SwapFields { .. }
        | Request::CopyRange { .. }
        | Request::PropagateMax { .. }
        | Request::PropagateMin { .. }
        | Request::ClampRange { .. }
        | Request::BatchConditionalSet { .. }
        | Request::SetFieldMany { .. }
        | Request::FloorDiv { .. }
        | Request::MulField { .. }
        | Request::SetFieldAdd { .. }
        | Request::SetFieldMul { .. }
        | Request::SwapPkFields { .. }
        | Request::IncrementField { .. }
        | Request::DecrementField { .. }
        | Request::SetMin { .. }
        | Request::SetMax { .. }
        | Request::XorField { .. }
        | Request::AndField { .. }
        | Request::OrField { .. } => Scope::ReadWrite,
        Request::Flush
        | Request::Compact
        | Request::UseTable { .. }
        | Request::CreateTable { .. }
        | Request::DropTable { .. }
        | Request::Truncate => Scope::Admin,
        Request::BeginTxn | Request::CommitTxn { .. } | Request::RollbackTxn { .. } => {
            Scope::ReadWrite
        }
        Request::Ping | Request::Hello | Request::Auth { .. } => Scope::ReadOnly,
    }
}

fn serve_request(
    registry: &TableRegistry,
    selected: &mut Option<Arc<TableEngine>>,
    txn: &mut Option<Txn>,
    next_txn_id: &mut u64,
    req: Request,
) -> Response {
    // Catalog operations — no selected table required.
    match req {
        Request::Ping => return Response::Pong,

        Request::UseTable { name } => {
            return match registry.get(&name) {
                Ok(engine) => {
                    *selected = Some(engine);
                    Response::TableSelected
                }
                Err(e) => {
                    let msg = e.to_string();
                    Response::Error {
                        code: error_code::TABLE_NOT_FOUND,
                        message: msg,
                    }
                }
            };
        }

        Request::CreateTable { name, schema } => {
            return match registry.create_table(&name, schema) {
                Ok(engine) => {
                    *selected = Some(engine);
                    Response::TableSelected
                }
                Err(e) => {
                    let msg = e.to_string();
                    let code = if msg.contains("already exists") {
                        error_code::TABLE_EXISTS
                    } else {
                        error_code::ENGINE
                    };
                    Response::Error { code, message: msg }
                }
            };
        }

        Request::DropTable { name } => {
            // If the currently selected table is the one being dropped,
            // check its name and deselect so subsequent data ops get NOT_SELECTED cleanly.
            return match registry.drop_table(&name) {
                Ok(()) => {
                    // Deselect if the dropped table was the active one. We can't
                    // compare Arc pointers (drop_table returns ()), so we ask the
                    // registry whether the name is still there — it isn't, so any
                    // selected engine whose name matched is now dangling from our
                    // perspective. The simplest safe policy: always deselect after
                    // a DropTable so the client must UseTable again.
                    *selected = None;
                    Response::Ok { affected: 0 }
                }
                Err(e) => {
                    let msg = e.to_string();
                    Response::Error {
                        code: error_code::TABLE_NOT_FOUND,
                        message: msg,
                    }
                }
            };
        }

        Request::ListTables => {
            return Response::TableList {
                names: registry.list_tables(),
            };
        }

        Request::TableExists { name } => {
            return Response::Exists {
                found: registry.get(&name).is_ok(),
            };
        }

        Request::BeginTxn => {
            if txn.is_some() {
                return Response::Error {
                    code: error_code::TXN_ACTIVE,
                    message: "a transaction is already active on this connection".into(),
                };
            }
            let id = *next_txn_id;
            *next_txn_id = next_txn_id.wrapping_add(1).max(1);
            *txn = Some(Txn {
                id,
                ops: Vec::new(),
            });
            return Response::TxnStarted { txn_id: id };
        }

        Request::CommitTxn { txn_id } => {
            return match txn.take() {
                None => Response::Error {
                    code: error_code::NO_TXN,
                    message: "no active transaction".into(),
                },
                Some(t) if t.id != txn_id => {
                    *txn = Some(t);
                    Response::Error {
                        code: error_code::TXN_ID_MISMATCH,
                        message: format!("txn_id mismatch: expected {}", txn_id),
                    }
                }
                Some(t) => {
                    // Apply all ops atomically. Require a selected table.
                    let engine = match selected.as_deref() {
                        Some(e) => e,
                        None => {
                            // Put the txn back so the client can rollback.
                            *txn = Some(t);
                            return Response::Error {
                                code: error_code::NOT_SELECTED,
                                message: "no table selected — send UseTable first".into(),
                            };
                        }
                    };
                    // One atomic WAL record for the whole transaction: after a
                    // crash it is replayed entirely or not at all. A failure is
                    // reported, and then nothing of the transaction is applied.
                    let n = t.ops.len() as u64;
                    let ops = t
                        .ops
                        .into_iter()
                        .map(|op| match op {
                            TxnOp::Insert { pk, values } => (pk, Some(values)),
                            TxnOp::Delete { pk } => (pk, None),
                        })
                        .collect();
                    match engine.apply_batch(ops) {
                        Ok(_) => Response::Ok { affected: n },
                        Err(e) => Response::Error {
                            code: error_code::ENGINE,
                            message: format!("commit failed, nothing applied: {e}"),
                        },
                    }
                }
            };
        }

        Request::RollbackTxn { txn_id } => {
            return match txn.take() {
                None => Response::Error {
                    code: error_code::NO_TXN,
                    message: "no active transaction".into(),
                },
                Some(t) if t.id != txn_id => {
                    *txn = Some(t);
                    Response::Error {
                        code: error_code::TXN_ID_MISMATCH,
                        message: format!("txn_id mismatch: expected {}", txn_id),
                    }
                }
                Some(_) => Response::Ok { affected: 0 },
            };
        }

        _ => {}
    }

    // If a transaction is active, buffer Insert and Delete ops.
    if let Some(active_txn) = txn {
        match req {
            Request::Insert { pk, values } => {
                active_txn.ops.push(TxnOp::Insert { pk, values });
                return Response::Ok { affected: 1 };
            }
            Request::Delete { pk } => {
                active_txn.ops.push(TxnOp::Delete { pk });
                return Response::Ok { affected: 1 };
            }
            _ => {}
        }
    }

    // Data operations — require a selected table.
    let engine = match selected.as_deref() {
        Some(e) => e,
        None => {
            return Response::Error {
                code: error_code::NOT_SELECTED,
                message: "no table selected — send UseTable first".into(),
            };
        }
    };

    match req {
        Request::Get { pk } => match engine.get(pk) {
            Ok(Some(row)) => Response::Row { values: row.values },
            Ok(None) => Response::NotFound,
            Err(e) => engine_error(e),
        },

        Request::GetAndDelete { pk } => match engine.get(pk) {
            Ok(None) => Response::NotFound,
            Err(e) => engine_error(e),
            Ok(Some(row)) => {
                let values = row.values;
                let _ = engine.delete(pk);
                Response::Row { values }
            }
        },

        Request::Increment { pk, col, delta } => match engine.get(pk) {
            Ok(None) => Response::NotFound,
            Err(e) => engine_error(e),
            Ok(Some(row)) => {
                let col_idx = col as usize;
                if col_idx >= row.values.len() {
                    return Response::Error {
                        code: error_code::INVALID_ARGUMENT,
                        message: "column index out of range".to_string(),
                    };
                }
                let mut values = row.values;
                values[col_idx] = values[col_idx].saturating_add(delta);
                let new_val = values[col_idx];
                match engine.delete(pk) {
                    Err(e) => engine_error(e),
                    Ok(_) => match engine.insert(pk, values) {
                        Ok(_) => Response::Ok { affected: new_val },
                        Err(e) => engine_error(e),
                    },
                }
            }
        },

        Request::CompareAndSwap {
            pk,
            col,
            expected,
            replacement,
        } => match engine.get(pk) {
            Ok(None) => Response::NotFound,
            Err(e) => engine_error(e),
            Ok(Some(row)) => {
                let col_idx = col as usize;
                if col_idx >= row.values.len() {
                    return Response::Error {
                        code: error_code::INVALID_ARGUMENT,
                        message: "column index out of range".to_string(),
                    };
                }
                if row.values[col_idx] != expected {
                    return Response::Ok { affected: 0 };
                }
                let mut values = row.values;
                values[col_idx] = replacement;
                match engine.delete(pk) {
                    Err(e) => engine_error(e),
                    Ok(_) => match engine.insert(pk, values) {
                        Ok(_) => Response::Ok { affected: 1 },
                        Err(e) => engine_error(e),
                    },
                }
            }
        },

        Request::SetField { pk, col, value } => match engine.get(pk) {
            Ok(None) => Response::NotFound,
            Err(e) => engine_error(e),
            Ok(Some(row)) => {
                let col_idx = col as usize;
                if col_idx >= row.values.len() {
                    return Response::Error {
                        code: error_code::INVALID_ARGUMENT,
                        message: "column index out of range".to_string(),
                    };
                }
                let mut values = row.values;
                values[col_idx] = value;
                match engine.delete(pk) {
                    Err(e) => engine_error(e),
                    Ok(_) => match engine.insert(pk, values) {
                        Ok(_) => Response::Ok { affected: 1 },
                        Err(e) => engine_error(e),
                    },
                }
            }
        },

        Request::InsertIfAbsent { pk, values } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(Some(_)) => Response::Ok { affected: 0 },
            Ok(None) => match engine.insert(pk, values) {
                Ok(_) => Response::Ok { affected: 1 },
                Err(e) => engine_error(e),
            },
        },

        Request::DeleteRange { col, lo, hi } => {
            let rows = match engine.scan(col as usize, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let pks: Vec<u64> = rows.into_iter().map(|r| r.values[0]).collect();
            let mut count = 0u64;
            for pk in pks {
                match engine.get(pk) {
                    Ok(Some(_)) => match engine.delete(pk) {
                        Ok(_) => count += 1,
                        Err(e) => return engine_error(e),
                    },
                    Ok(None) => {}
                    Err(e) => return engine_error(e),
                }
            }
            Response::Ok { affected: count }
        }

        Request::PopMin { col, lo, hi } => {
            let rows = match engine.scan(col as usize, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut sorted = rows;
            sorted.sort_by_key(|r| r.values[col as usize]);
            let mut found = None;
            for row in sorted {
                let pk = row.values[0];
                match engine.get(pk) {
                    Ok(Some(_)) => match engine.delete(pk) {
                        Ok(_) => {
                            found = Some(row.values);
                            break;
                        }
                        Err(e) => return engine_error(e),
                    },
                    Ok(None) => continue,
                    Err(e) => return engine_error(e),
                }
            }
            match found {
                None => Response::NotFound,
                Some(values) => Response::Row { values },
            }
        }

        Request::PopMax { col, lo, hi } => {
            let rows = match engine.scan(col as usize, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut sorted = rows;
            sorted.sort_by_key(|r| std::cmp::Reverse(r.values[col as usize]));
            let mut found = None;
            for row in sorted {
                let pk = row.values[0];
                match engine.get(pk) {
                    Ok(Some(_)) => match engine.delete(pk) {
                        Ok(_) => {
                            found = Some(row.values);
                            break;
                        }
                        Err(e) => return engine_error(e),
                    },
                    Ok(None) => continue,
                    Err(e) => return engine_error(e),
                }
            }
            match found {
                None => Response::NotFound,
                Some(values) => Response::Row { values },
            }
        }

        Request::MultiSet { pk, pairs } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(None) => Response::NotFound,
            Ok(Some(row)) => {
                let mut values = row.values;
                for (col, value) in &pairs {
                    let idx = *col as usize;
                    if idx >= values.len() {
                        return Response::Error {
                            code: error_code::INVALID_ARGUMENT,
                            message: format!("column index {idx} out of range"),
                        };
                    }
                    values[idx] = *value;
                }
                match engine.delete(pk) {
                    Err(e) => engine_error(e),
                    Ok(_) => match engine.insert(pk, values) {
                        Ok(_) => Response::Ok { affected: 1 },
                        Err(e) => engine_error(e),
                    },
                }
            }
        },

        Request::DecrementClamp { pk, col, delta } => match engine.get(pk) {
            Ok(None) => Response::NotFound,
            Err(e) => engine_error(e),
            Ok(Some(row)) => {
                let col_idx = col as usize;
                if col_idx >= row.values.len() {
                    return Response::Error {
                        code: error_code::INVALID_ARGUMENT,
                        message: "column index out of range".to_string(),
                    };
                }
                let mut values = row.values;
                values[col_idx] = values[col_idx].saturating_sub(delta);
                let new_val = values[col_idx];
                match engine.delete(pk) {
                    Err(e) => engine_error(e),
                    Ok(_) => match engine.insert(pk, values) {
                        Ok(_) => Response::Ok { affected: new_val },
                        Err(e) => engine_error(e),
                    },
                }
            }
        },

        Request::GetOrInsert { pk, values } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(Some(row)) => Response::Row { values: row.values },
            Ok(None) => match engine.insert(pk, values.clone()) {
                Ok(_) => Response::Row { values },
                Err(e) => engine_error(e),
            },
        },

        Request::BatchSetField { col, value, pks } => {
            let col_idx = col as usize;
            let mut count = 0u64;
            for pk in pks {
                match engine.get(pk) {
                    Err(_) | Ok(None) => {}
                    Ok(Some(row)) => {
                        if col_idx >= row.values.len() {
                            continue;
                        }
                        let mut values = row.values;
                        values[col_idx] = value;
                        if engine.delete(pk).is_ok() && engine.insert(pk, values).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::Top { col, lo, hi, n } => {
            let rows = match engine.scan(col as usize, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut sorted = rows;
            sorted.sort_by_key(|r| std::cmp::Reverse(r.values[col as usize]));
            let limit = if n == 0 { sorted.len() } else { n as usize };
            let rows = sorted.into_iter().take(limit).map(|r| r.values).collect();
            Response::Rows { rows }
        }

        Request::Bottom { col, lo, hi, n } => {
            let rows = match engine.scan(col as usize, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut sorted = rows;
            sorted.sort_by_key(|r| r.values[col as usize]);
            let limit = if n == 0 { sorted.len() } else { n as usize };
            let rows = sorted.into_iter().take(limit).map(|r| r.values).collect();
            Response::Rows { rows }
        }

        Request::PkExists { pk } => {
            let found = match engine.get(pk) {
                Ok(v) => v.is_some(),
                Err(_) => false,
            };
            Response::Exists { found }
        }

        Request::Insert { pk, values } => match engine.insert(pk, values) {
            Ok(_) => Response::Ok { affected: 1 },
            Err(e) => engine_error(e),
        },

        Request::Delete { pk } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(None) => Response::Ok { affected: 0 },
            Ok(Some(_)) => match engine.delete(pk) {
                Ok(_) => Response::Ok { affected: 1 },
                Err(e) => engine_error(e),
            },
        },

        Request::RangeScan { col, lo, hi, limit } => match engine.scan(col as usize, lo, hi) {
            Ok(rows) => {
                let rows: Vec<Vec<u64>> = if limit == 0 {
                    rows.into_iter().map(|r| r.values).collect()
                } else {
                    rows.into_iter()
                        .take(limit as usize)
                        .map(|r| r.values)
                        .collect()
                };
                Response::Rows { rows }
            }
            Err(e) => engine_error(e),
        },

        Request::Sum { col, lo, hi } => match engine.scan(col as usize, lo, hi) {
            Err(e) => engine_error(e),
            Ok(rows) => {
                let col_idx = col as usize;
                let total = rows
                    .iter()
                    .fold(0u64, |acc, row| acc.saturating_add(row.values[col_idx]));
                Response::Ok { affected: total }
            }
        },

        Request::MinMax { col, lo, hi } => match engine.scan(col as usize, lo, hi) {
            Err(e) => engine_error(e),
            Ok(rows) => {
                if rows.is_empty() {
                    return Response::NotFound;
                }
                let col_idx = col as usize;
                let mut min = u64::MAX;
                let mut max = 0u64;
                let count = rows.len() as u64;
                for row in &rows {
                    let v = row.values[col_idx];
                    if v < min {
                        min = v;
                    }
                    if v > max {
                        max = v;
                    }
                }
                Response::MinMaxResult { min, max, count }
            }
        },

        Request::ScanKeys { col, lo, hi, limit } => match engine.scan(col as usize, lo, hi) {
            Ok(rows) => {
                let pks: Vec<u64> = if limit == 0 {
                    rows.into_iter().map(|r| r.values[0]).collect()
                } else {
                    rows.into_iter()
                        .take(limit as usize)
                        .map(|r| r.values[0])
                        .collect()
                };
                Response::Keys { pks }
            }
            Err(e) => engine_error(e),
        },

        Request::Flush => match engine.flush() {
            Ok(_) => Response::Ok { affected: 0 },
            Err(e) => engine_error(e),
        },

        Request::Compact => match engine.compact() {
            Ok(Some(r)) => Response::Ok {
                affected: r.rows_kept as u64,
            },
            Ok(None) => Response::Ok { affected: 0 },
            Err(e) => engine_error(e),
        },

        Request::SecondaryLookup { col_idx, value } => {
            match engine.secondary_scan(col_idx, value) {
                Ok(rows) => Response::Rows { rows },
                Err(e) => engine_error(e),
            }
        }

        Request::FkExists { value } => Response::Exists {
            found: engine.fk_exists(value),
        },

        Request::BatchInsert { rows } => {
            let mut succeeded: u32 = 0;
            let mut skipped: u32 = 0;
            for (pk, values) in rows {
                match engine.insert(pk, values) {
                    Ok(_) => succeeded = succeeded.saturating_add(1),
                    Err(_) => skipped = skipped.saturating_add(1),
                }
            }
            Response::BatchResult { succeeded, skipped }
        }

        Request::BatchDelete { pks } => {
            let mut succeeded: u32 = 0;
            let mut skipped: u32 = 0;
            for pk in pks {
                match engine.get(pk) {
                    Ok(Some(_)) => match engine.delete(pk) {
                        Ok(_) => succeeded = succeeded.saturating_add(1),
                        Err(e) => return engine_error(e),
                    },
                    Ok(None) => skipped = skipped.saturating_add(1),
                    Err(e) => return engine_error(e),
                }
            }
            Response::BatchResult { succeeded, skipped }
        }

        Request::Upsert { pk, values } => {
            // Delete silently (row may not exist), then insert.
            let _ = engine.delete(pk);
            match engine.insert(pk, values) {
                Ok(_) => Response::Ok { affected: 1 },
                Err(e) => engine_error(e),
            }
        }

        Request::GetMany { pks } => {
            let mut entries: Vec<(u64, Vec<u64>)> = Vec::with_capacity(pks.len());
            for pk in pks {
                if let Ok(Some(row)) = engine.get(pk) {
                    entries.push((pk, row.values));
                }
            }
            Response::RowSet { entries }
        }

        Request::BatchUpsert { rows } => {
            let count = rows.len() as u64;
            for (pk, values) in rows {
                let _ = engine.delete(pk);
                let _ = engine.insert(pk, values);
            }
            Response::Ok { affected: count }
        }

        Request::BatchUpdate { rows } => {
            let mut succeeded: u32 = 0;
            let mut skipped: u32 = 0;
            for (pk, values) in rows {
                match engine.get(pk) {
                    Ok(None) => skipped = skipped.saturating_add(1),
                    Err(_) => skipped = skipped.saturating_add(1),
                    Ok(Some(_)) => {
                        let _ = engine.delete(pk);
                        let _ = engine.insert(pk, values);
                        succeeded = succeeded.saturating_add(1);
                    }
                }
            }
            Response::BatchResult { succeeded, skipped }
        }

        Request::Count { col, lo, hi } => match engine.scan(col as usize, lo, hi) {
            Ok(rows) => Response::Ok {
                affected: rows.len() as u64,
            },
            Err(e) => engine_error(e),
        },

        Request::Truncate => {
            // scan() auto-flushes the write buffer before reading partitions,
            // so this captures all live rows. We delete each, then flush the
            // tombstones and compact to merge them with the insert records —
            // without compact, tombstones sit in a separate partition and scan
            // would still return the original insert rows on subsequent calls.
            match engine.scan(0, 0, u64::MAX) {
                Err(e) => engine_error(e),
                Ok(rows) => {
                    let pks: Vec<u64> = rows.into_iter().map(|r| r.values[0]).collect();
                    let count = pks.len() as u64;
                    for pk in &pks {
                        let _ = engine.delete(*pk);
                    }
                    // Flush tombstones into a partition, then compact to
                    // physically remove all rows from storage.
                    let _ = engine.flush();
                    let _ = engine.compact();
                    Response::Ok { affected: count }
                }
            }
        }

        Request::Update { pk, values } => match engine.get(pk) {
            Ok(None) => Response::Error {
                code: error_code::NOT_FOUND,
                message: format!("pk={pk} not found"),
            },
            Err(e) => engine_error(e),
            Ok(Some(_)) => {
                let _ = engine.delete(pk);
                match engine.insert(pk, values) {
                    Ok(_) => Response::Ok { affected: 1 },
                    Err(e) => engine_error(e),
                }
            }
        },

        Request::Stats => {
            let s = engine.stats();
            let bytes: u64 = engine.partition_meta().iter().map(|p| p.size_bytes).sum();
            Response::Stats {
                rows: s.rows_flushed,
                partitions: s.partition_count as u64,
                bytes,
            }
        }

        Request::DescribeTable => Response::Schema {
            columns: engine.schema.names(),
        },

        Request::IncrementMulti { pk, pairs } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(None) => Response::NotFound,
            Ok(Some(row)) => {
                let mut values = row.values;
                for (col, delta) in pairs {
                    let idx = col as usize;
                    if idx >= values.len() {
                        return Response::Error {
                            code: error_code::INVALID_ARGUMENT,
                            message: format!("col {idx} out of range"),
                        };
                    }
                    values[idx] = values[idx].saturating_add(delta);
                }
                match engine.delete(pk) {
                    Err(e) => engine_error(e),
                    Ok(_) => match engine.insert(pk, values.clone()) {
                        Ok(_) => Response::Row { values },
                        Err(e) => engine_error(e),
                    },
                }
            }
        },

        Request::CopyRow { src_pk, dst_pk } => match engine.get(src_pk) {
            Err(e) => engine_error(e),
            Ok(None) => Response::NotFound,
            Ok(Some(row)) => {
                let mut values = row.values;
                values[0] = dst_pk;
                // upsert: delete dst if present, then insert
                let _ = engine.delete(dst_pk);
                match engine.insert(dst_pk, values) {
                    Ok(_) => Response::Ok { affected: 1 },
                    Err(e) => engine_error(e),
                }
            }
        },

        Request::SwapField { pk1, pk2, col } => {
            let col_idx = col as usize;
            let row1 = match engine.get(pk1) {
                Err(e) => return engine_error(e),
                Ok(None) => return Response::NotFound,
                Ok(Some(r)) => r,
            };
            let row2 = match engine.get(pk2) {
                Err(e) => return engine_error(e),
                Ok(None) => return Response::NotFound,
                Ok(Some(r)) => r,
            };
            if col_idx >= row1.values.len() || col_idx >= row2.values.len() {
                return Response::Error {
                    code: error_code::INVALID_ARGUMENT,
                    message: format!("col {col_idx} out of range"),
                };
            }
            let mut v1 = row1.values;
            let mut v2 = row2.values;
            std::mem::swap(&mut v1[col_idx], &mut v2[col_idx]);
            if engine.delete(pk1).is_err() || engine.insert(pk1, v1).is_err() {
                return Response::Error {
                    code: error_code::ENGINE,
                    message: "swap failed on pk1".into(),
                };
            }
            if engine.delete(pk2).is_err() || engine.insert(pk2, v2).is_err() {
                return Response::Error {
                    code: error_code::ENGINE,
                    message: "swap failed on pk2".into(),
                };
            }
            Response::Ok { affected: 2 }
        }

        Request::Clamp { pk, col, lo, hi } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(None) => Response::NotFound,
            Ok(Some(row)) => {
                let col_idx = col as usize;
                if col_idx >= row.values.len() {
                    return Response::Error {
                        code: error_code::INVALID_ARGUMENT,
                        message: format!("col {col_idx} out of range"),
                    };
                }
                let mut values = row.values;
                values[col_idx] = values[col_idx].clamp(lo, hi);
                let new_val = values[col_idx];
                match engine.delete(pk) {
                    Err(e) => engine_error(e),
                    Ok(_) => match engine.insert(pk, values) {
                        Ok(_) => Response::Ok { affected: new_val },
                        Err(e) => engine_error(e),
                    },
                }
            }
        },

        Request::Average { col, lo, hi } => {
            let rows = match engine.scan(col as usize, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            if rows.is_empty() {
                return Response::NotFound;
            }
            let col_idx = col as usize;
            let mut sum: u64 = 0;
            let count = rows.len() as u64;
            for row in &rows {
                sum = sum.saturating_add(row.values[col_idx]);
            }
            Response::Row {
                values: vec![sum, count],
            }
        }

        Request::DeleteIf { pk, col, value } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(None) => Response::NotFound,
            Ok(Some(row)) => {
                let col_idx = col as usize;
                if col_idx >= row.values.len() {
                    return Response::Error {
                        code: error_code::INVALID_ARGUMENT,
                        message: format!("col {col_idx} out of range"),
                    };
                }
                if row.values[col_idx] != value {
                    return Response::Ok { affected: 0 };
                }
                match engine.delete(pk) {
                    Ok(_) => Response::Ok { affected: 1 },
                    Err(e) => engine_error(e),
                }
            }
        },

        Request::SetFieldIf {
            pk,
            col,
            expected,
            new_value,
        } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(None) => Response::NotFound,
            Ok(Some(row)) => {
                let col_idx = col as usize;
                if col_idx >= row.values.len() {
                    return Response::Error {
                        code: error_code::INVALID_ARGUMENT,
                        message: format!("col {col_idx} out of range"),
                    };
                }
                if row.values[col_idx] != expected {
                    return Response::Ok { affected: 0 };
                }
                let mut values = row.values;
                values[col_idx] = new_value;
                match engine.delete(pk) {
                    Err(e) => engine_error(e),
                    Ok(_) => match engine.insert(pk, values) {
                        Ok(_) => Response::Ok { affected: 1 },
                        Err(e) => engine_error(e),
                    },
                }
            }
        },

        Request::RankOf { pk, col, lo, hi } => {
            let col_idx = col as usize;
            let target_row = match engine.get(pk) {
                Err(e) => return engine_error(e),
                Ok(None) => return Response::NotFound,
                Ok(Some(r)) => r,
            };
            if col_idx >= target_row.values.len() {
                return Response::Error {
                    code: error_code::INVALID_ARGUMENT,
                    message: format!("col {col_idx} out of range"),
                };
            }
            let target_val = target_row.values[col_idx];
            // Check pk is actually within the scan range
            if target_val < lo || target_val > hi {
                return Response::NotFound;
            }
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let rank = rows
                .iter()
                .filter(|r| r.values[col_idx] < target_val)
                .count() as u64;
            Response::Ok { affected: rank }
        }

        Request::Percentile { col, lo, hi, pct } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            if rows.is_empty() {
                return Response::NotFound;
            }
            if pct > 100 {
                return Response::Error {
                    code: error_code::INVALID_ARGUMENT,
                    message: "pct must be 0–100".into(),
                };
            }
            let mut vals: Vec<u64> = rows.iter().map(|r| r.values[col_idx]).collect();
            vals.sort_unstable();
            let idx = (pct as usize * (vals.len() - 1)) / 100;
            Response::Ok {
                affected: vals[idx],
            }
        }

        Request::SumFields { pk, cols } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(None) => Response::NotFound,
            Ok(Some(row)) => {
                let mut sum: u64 = 0;
                for col in cols {
                    let idx = col as usize;
                    if idx >= row.values.len() {
                        return Response::Error {
                            code: error_code::INVALID_ARGUMENT,
                            message: format!("col {idx} out of range"),
                        };
                    }
                    sum = sum.saturating_add(row.values[idx]);
                }
                Response::Ok { affected: sum }
            }
        },

        Request::MaxField { pk, cols } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(None) => Response::NotFound,
            Ok(Some(row)) => {
                if cols.is_empty() {
                    return Response::Ok { affected: 0 };
                }
                let mut max_val: u64 = 0;
                for col in cols {
                    let idx = col as usize;
                    if idx >= row.values.len() {
                        return Response::Error {
                            code: error_code::INVALID_ARGUMENT,
                            message: format!("col {idx} out of range"),
                        };
                    }
                    if row.values[idx] > max_val {
                        max_val = row.values[idx];
                    }
                }
                Response::Ok { affected: max_val }
            }
        },

        Request::FilterScan {
            scan_col,
            lo,
            hi,
            filter_col,
            filter_val,
        } => {
            let rows = match engine.scan(scan_col as usize, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let filter_idx = filter_col as usize;
            let filtered: Vec<Vec<u64>> = rows
                .into_iter()
                .filter(|r| r.values.get(filter_idx).copied().unwrap_or(u64::MAX) == filter_val)
                .map(|r| r.values)
                .collect();
            Response::Rows { rows: filtered }
        }

        Request::BatchIncrementField { col, delta, pks } => {
            let col_idx = col as usize;
            let mut count: u64 = 0;
            for pk in pks {
                match engine.get(pk) {
                    Err(_) | Ok(None) => {}
                    Ok(Some(row)) => {
                        if col_idx < row.values.len() {
                            let mut values = row.values;
                            values[col_idx] = values[col_idx].saturating_add(delta);
                            if engine.delete(pk).is_ok() && engine.insert(pk, values).is_ok() {
                                count += 1;
                            }
                        }
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::CountDistinct { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut seen = std::collections::HashSet::new();
            for row in &rows {
                seen.insert(row.values[col_idx]);
            }
            Response::Ok {
                affected: seen.len() as u64,
            }
        }

        Request::Median { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            if rows.is_empty() {
                return Response::NotFound;
            }
            let mut vals: Vec<u64> = rows.iter().map(|r| r.values[col_idx]).collect();
            vals.sort_unstable();
            let mid = vals.len() / 2;
            Response::Ok {
                affected: vals[mid],
            }
        }

        Request::BatchDeleteIf { col, value, pks } => {
            let col_idx = col as usize;
            let mut count: u64 = 0;
            for pk in pks {
                match engine.get(pk) {
                    Err(_) | Ok(None) => {}
                    Ok(Some(row)) => {
                        if row.values.get(col_idx).copied() == Some(value)
                            && engine.delete(pk).is_ok()
                        {
                            count += 1;
                        }
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::BatchPkExists { pks } => {
            let mut count: u64 = 0;
            for pk in &pks {
                if matches!(engine.get(*pk), Ok(Some(_))) {
                    count += 1;
                }
            }
            Response::Ok { affected: count }
        }

        Request::FieldBetween {
            scan_col,
            lo,
            hi,
            filter_col,
            filter_lo,
            filter_hi,
        } => {
            let filt_idx = filter_col as usize;
            let rows = match engine.scan(scan_col as usize, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let out: Vec<Vec<u64>> = rows
                .into_iter()
                .filter(|r| {
                    let fv = r.values.get(filt_idx).copied().unwrap_or(0);
                    fv >= filter_lo && fv <= filter_hi
                })
                .map(|r| r.values)
                .collect();
            Response::Rows { rows: out }
        }

        Request::ScalarMin { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            match rows
                .into_iter()
                .map(|r| r.values.get(col_idx).copied().unwrap_or(0))
                .min()
            {
                Some(v) => Response::Ok { affected: v },
                None => Response::NotFound,
            }
        }

        Request::ScalarMax { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            match rows
                .into_iter()
                .map(|r| r.values.get(col_idx).copied().unwrap_or(0))
                .max()
            {
                Some(v) => Response::Ok { affected: v },
                None => Response::NotFound,
            }
        }

        Request::MultiGet { pk, cols } => match engine.get(pk) {
            Err(e) => engine_error(e),
            Ok(None) => Response::NotFound,
            Ok(Some(row)) => {
                let selected: Vec<u64> = cols
                    .iter()
                    .map(|&c| row.values.get(c as usize).copied().unwrap_or(0))
                    .collect();
                Response::Row { values: selected }
            }
        },

        Request::IncrementOrInsert {
            pk,
            col,
            delta,
            values,
        } => {
            let col_idx = col as usize;
            match engine.get(pk) {
                Ok(Some(mut row)) => {
                    let v = row.values.get(col_idx).copied().unwrap_or(0);
                    let new_val = v.saturating_add(delta);
                    row.values[col_idx] = new_val;
                    match engine.insert(pk, row.values) {
                        Ok(_) => Response::Ok { affected: new_val },
                        Err(e) => engine_error(e),
                    }
                }
                Ok(None) | Err(_) => match engine.insert(pk, values) {
                    Ok(_) => Response::Ok { affected: 0 },
                    Err(e) => engine_error(e),
                },
            }
        }

        Request::ZeroField { col, lo, hi } => {
            let col_idx = col as usize;
            let pks: Vec<u64> = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r.into_iter().map(|r| r.values[0]).collect(),
                Err(e) => return engine_error(e),
            };
            let mut count: u64 = 0;
            for pk in pks {
                if let Ok(Some(mut row)) = engine.get(pk)
                    && col_idx < row.values.len()
                {
                    row.values[col_idx] = 0;
                    if engine.insert(pk, row.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::ScanPage {
            col,
            lo,
            hi,
            offset,
            limit,
        } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let out: Vec<Vec<u64>> = rows
                .into_iter()
                .enumerate()
                .filter(|(i, _)| (*i as u32) >= offset)
                .take(limit as usize)
                .map(|(_, r)| r.values)
                .collect();
            Response::Rows { rows: out }
        }

        Request::SumField { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let sum = rows
                .into_iter()
                .map(|r| r.values.get(col_idx).copied().unwrap_or(0))
                .fold(0u64, |acc, v| acc.saturating_add(v));
            Response::Ok { affected: sum }
        }

        Request::BatchGet { pks } => {
            let mut out = Vec::new();
            for pk in pks {
                if let Ok(Some(row)) = engine.get(pk) {
                    out.push(row.values);
                }
            }
            Response::Rows { rows: out }
        }

        Request::SetFieldRange { col, value, lo, hi } => {
            let col_idx = col as usize;
            let pks: Vec<u64> = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r.into_iter().map(|r| r.values[0]).collect(),
                Err(e) => return engine_error(e),
            };
            let mut count: u64 = 0;
            for pk in pks {
                if let Ok(Some(mut row)) = engine.get(pk)
                    && col_idx < row.values.len()
                {
                    row.values[col_idx] = value;
                    if engine.insert(pk, row.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::MaxRow { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            match rows
                .into_iter()
                .max_by_key(|r| r.values.get(col_idx).copied().unwrap_or(0))
            {
                None => Response::NotFound,
                Some(row) => Response::Row { values: row.values },
            }
        }

        Request::MinRow { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            match rows
                .into_iter()
                .min_by_key(|r| r.values.get(col_idx).copied().unwrap_or(0))
            {
                None => Response::NotFound,
                Some(row) => Response::Row { values: row.values },
            }
        }

        Request::IncrementRange { col, delta, lo, hi } => {
            let col_idx = col as usize;
            let pks: Vec<u64> = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r.into_iter().map(|r| r.values[0]).collect(),
                Err(e) => return engine_error(e),
            };
            let mut count: u64 = 0;
            for pk in pks {
                if let Ok(Some(mut row)) = engine.get(pk)
                    && col_idx < row.values.len()
                {
                    row.values[col_idx] = row.values[col_idx].saturating_add(delta);
                    if engine.insert(pk, row.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::DeleteMin { col, lo, hi } => {
            let col_idx = col as usize;
            let mut rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            rows.sort_by_key(|r| r.values.get(col_idx).copied().unwrap_or(0));
            let mut result = Response::NotFound;
            for row in rows {
                let pk = row.values[0];
                if matches!(engine.get(pk), Ok(Some(_))) && engine.delete(pk).is_ok() {
                    result = Response::Ok { affected: pk };
                    break;
                }
            }
            result
        }

        Request::DeleteMax { col, lo, hi } => {
            let col_idx = col as usize;
            let mut rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            rows.sort_by_key(|r| std::cmp::Reverse(r.values.get(col_idx).copied().unwrap_or(0)));
            let mut result = Response::NotFound;
            for row in rows {
                let pk = row.values[0];
                if matches!(engine.get(pk), Ok(Some(_))) && engine.delete(pk).is_ok() {
                    result = Response::Ok { affected: pk };
                    break;
                }
            }
            result
        }

        Request::UniqueCount { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut seen = std::collections::HashSet::new();
            for row in rows {
                if matches!(engine.get(row.values[0]), Ok(Some(_)))
                    && let Some(&v) = row.values.get(col_idx)
                    && v != 0
                {
                    seen.insert(v);
                }
            }
            Response::Ok {
                affected: seen.len() as u64,
            }
        }

        Request::TopN { col, n, lo, hi } => {
            let col_idx = col as usize;
            let mut rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            rows.retain(|r| matches!(engine.get(r.values[0]), Ok(Some(_))));
            rows.sort_by_key(|r| std::cmp::Reverse(r.values.get(col_idx).copied().unwrap_or(0)));
            let taken: Vec<Vec<u64>> = rows
                .into_iter()
                .take(n as usize)
                .map(|r| r.values)
                .collect();
            Response::Rows { rows: taken }
        }

        Request::BottomN { col, n, lo, hi } => {
            let col_idx = col as usize;
            let mut rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            rows.retain(|r| matches!(engine.get(r.values[0]), Ok(Some(_))));
            rows.sort_by_key(|r| r.values.get(col_idx).copied().unwrap_or(0));
            let taken: Vec<Vec<u64>> = rows
                .into_iter()
                .take(n as usize)
                .map(|r| r.values)
                .collect();
            Response::Rows { rows: taken }
        }

        Request::MoveField {
            src_pk,
            dst_pk,
            col,
        } => {
            let col_idx = col as usize;
            let mut src = match engine.get(src_pk) {
                Ok(Some(r)) => r,
                Ok(None) => return Response::NotFound,
                Err(e) => return engine_error(e),
            };
            let mut dst = match engine.get(dst_pk) {
                Ok(Some(r)) => r,
                Ok(None) => return Response::NotFound,
                Err(e) => return engine_error(e),
            };
            if col_idx >= src.values.len() || col_idx >= dst.values.len() {
                return Response::Error {
                    code: error_code::SCHEMA,
                    message: "column index out of range".into(),
                };
            }
            let old_value = src.values[col_idx];
            dst.values[col_idx] = dst.values[col_idx].saturating_add(old_value);
            src.values[col_idx] = 0;
            if let Err(e) = engine.insert(src_pk, src.values) {
                return engine_error(e);
            }
            if let Err(e) = engine.insert(dst_pk, dst.values) {
                return engine_error(e);
            }
            Response::Ok {
                affected: old_value,
            }
        }

        Request::CountIf { col, value, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let count = rows
                .into_iter()
                .filter(|r| {
                    matches!(engine.get(r.values[0]), Ok(Some(_)))
                        && r.values.get(col_idx).copied().unwrap_or(u64::MAX) == value
                })
                .count();
            Response::Ok {
                affected: count as u64,
            }
        }

        Request::SetIfGreater { col, value, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut count = 0u64;
            for row in rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                    && live.values[col_idx] < value
                {
                    live.values[col_idx] = value;
                    if engine.insert(pk, live.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::SetIfLess { col, value, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut count = 0u64;
            for row in rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                    && live.values[col_idx] > value
                {
                    live.values[col_idx] = value;
                    if engine.insert(pk, live.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::AbsDiff { pk1, pk2, col } => {
            let col_idx = col as usize;
            let row1 = match engine.get(pk1) {
                Ok(Some(r)) => r,
                Ok(None) => return Response::NotFound,
                Err(e) => return engine_error(e),
            };
            let row2 = match engine.get(pk2) {
                Ok(Some(r)) => r,
                Ok(None) => return Response::NotFound,
                Err(e) => return engine_error(e),
            };
            if col_idx >= row1.values.len() || col_idx >= row2.values.len() {
                return Response::Error {
                    code: error_code::SCHEMA,
                    message: "column index out of range".into(),
                };
            }
            let v1 = row1.values[col_idx];
            let v2 = row2.values[col_idx];
            let diff = v1.abs_diff(v2);
            Response::Ok { affected: diff }
        }

        Request::SwapFields { pk1, pk2, col } => {
            let col_idx = col as usize;
            let mut row1 = match engine.get(pk1) {
                Ok(Some(r)) => r,
                Ok(None) => return Response::NotFound,
                Err(e) => return engine_error(e),
            };
            let mut row2 = match engine.get(pk2) {
                Ok(Some(r)) => r,
                Ok(None) => return Response::NotFound,
                Err(e) => return engine_error(e),
            };
            if col_idx >= row1.values.len() || col_idx >= row2.values.len() {
                return Response::Error {
                    code: error_code::SCHEMA,
                    message: "column index out of range".into(),
                };
            }
            std::mem::swap(&mut row1.values[col_idx], &mut row2.values[col_idx]);
            if let Err(e) = engine.insert(pk1, row1.values) {
                return engine_error(e);
            }
            if let Err(e) = engine.insert(pk2, row2.values) {
                return engine_error(e);
            }
            Response::Ok { affected: 2 }
        }

        Request::CopyRange {
            src_col,
            dst_col,
            lo,
            hi,
        } => {
            let src_idx = src_col as usize;
            let dst_idx = dst_col as usize;
            let rows = match engine.scan(src_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut count = 0u64;
            for row in rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && src_idx < live.values.len()
                    && dst_idx < live.values.len()
                {
                    live.values[dst_idx] = live.values[src_idx];
                    if engine.insert(pk, live.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::PropagateMax { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let live_rows: Vec<_> = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .collect();
            if live_rows.is_empty() {
                return Response::Ok { affected: 0 };
            }
            let max_val = live_rows
                .iter()
                .map(|r| r.values.get(col_idx).copied().unwrap_or(0))
                .max()
                .unwrap_or(0);
            for row in live_rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                {
                    live.values[col_idx] = max_val;
                    let _ = engine.insert(pk, live.values);
                }
            }
            Response::Ok { affected: max_val }
        }

        Request::PropagateMin { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let live_rows: Vec<_> = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .collect();
            if live_rows.is_empty() {
                return Response::Ok { affected: 0 };
            }
            let min_val = live_rows
                .iter()
                .map(|r| r.values.get(col_idx).copied().unwrap_or(u64::MAX))
                .min()
                .unwrap_or(0);
            for row in live_rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                {
                    live.values[col_idx] = min_val;
                    let _ = engine.insert(pk, live.values);
                }
            }
            Response::Ok { affected: min_val }
        }

        Request::ClampRange {
            col,
            min_val,
            max_val,
            lo,
            hi,
        } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut count = 0u64;
            for row in rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                {
                    let v = live.values[col_idx];
                    let clamped = v.clamp(min_val, max_val);
                    if clamped != v {
                        live.values[col_idx] = clamped;
                        if engine.insert(pk, live.values).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::BatchConditionalSet {
            col,
            expected,
            new_value,
            lo,
            hi,
        } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut count = 0u64;
            for row in rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                    && live.values[col_idx] == expected
                {
                    live.values[col_idx] = new_value;
                    if engine.insert(pk, live.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::WindowRank { pk, col, lo, hi } => {
            let col_idx = col as usize;
            let target = match engine.get(pk) {
                Ok(Some(r)) => r,
                Ok(None) => return Response::NotFound,
                Err(e) => return engine_error(e),
            };
            let target_val = target.values.get(col_idx).copied().unwrap_or(0);
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let rank = rows
                .into_iter()
                .filter(|r| {
                    matches!(engine.get(r.values[0]), Ok(Some(_)))
                        && r.values.get(col_idx).copied().unwrap_or(0) < target_val
                })
                .count() as u64
                + 1;
            Response::Ok { affected: rank }
        }

        Request::WindowPct { pk, col, lo, hi } => {
            let col_idx = col as usize;
            let target = match engine.get(pk) {
                Ok(Some(r)) => r,
                Ok(None) => return Response::NotFound,
                Err(e) => return engine_error(e),
            };
            let target_val = target.values.get(col_idx).copied().unwrap_or(0);
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let live: Vec<_> = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .collect();
            let total = live.len() as u64;
            if total == 0 {
                return Response::Ok { affected: 0 };
            }
            let below = live
                .iter()
                .filter(|r| r.values.get(col_idx).copied().unwrap_or(0) < target_val)
                .count() as u64;
            let pct = (below * 100) / total;
            Response::Ok { affected: pct }
        }

        Request::SumProduct {
            col_a,
            col_b,
            lo,
            hi,
        } => {
            let a_idx = col_a as usize;
            let b_idx = col_b as usize;
            let rows = match engine.scan(a_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let sum = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .map(|r| {
                    let a = r.values.get(a_idx).copied().unwrap_or(0);
                    let b = r.values.get(b_idx).copied().unwrap_or(0);
                    a.saturating_mul(b)
                })
                .fold(0u64, u64::saturating_add);
            Response::Ok { affected: sum }
        }

        Request::NthValue { col, n, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut vals: Vec<u64> = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .map(|r| r.values.get(col_idx).copied().unwrap_or(0))
                .collect();
            vals.sort_unstable();
            match vals.get(n as usize) {
                Some(&v) => Response::Ok { affected: v },
                None => Response::NotFound,
            }
        }

        Request::DenseRankWin { pk, col, lo, hi } => {
            let col_idx = col as usize;
            let target = match engine.get(pk) {
                Ok(Some(r)) => r,
                Ok(None) => return Response::NotFound,
                Err(e) => return engine_error(e),
            };
            let target_val = target.values.get(col_idx).copied().unwrap_or(0);
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut distinct_below: std::collections::BTreeSet<u64> = rows
                .into_iter()
                .filter(|r| {
                    matches!(engine.get(r.values[0]), Ok(Some(_)))
                        && r.values.get(col_idx).copied().unwrap_or(0) < target_val
                })
                .map(|r| r.values.get(col_idx).copied().unwrap_or(0))
                .collect();
            distinct_below.insert(target_val);
            let rank = distinct_below.len() as u64;
            Response::Ok { affected: rank }
        }

        Request::SetFieldMany { col, value, pks } => {
            let col_idx = col as usize;
            let mut count = 0u64;
            for pk in pks {
                if let Ok(Some(mut row)) = engine.get(pk)
                    && col_idx < row.values.len()
                {
                    row.values[col_idx] = value;
                    if engine.insert(pk, row.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::MultiColumnSum { pks, col } => {
            let col_idx = col as usize;
            let sum = pks
                .iter()
                .filter_map(|&pk| engine.get(pk).ok().flatten())
                .map(|r| r.values.get(col_idx).copied().unwrap_or(0))
                .fold(0u64, u64::saturating_add);
            Response::Ok { affected: sum }
        }

        Request::FieldHistogram {
            col,
            bucket_size,
            lo,
            hi,
        } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let bucket_sz = if bucket_size == 0 { 1 } else { bucket_size };
            let mut buckets: std::collections::BTreeMap<u64, u64> =
                std::collections::BTreeMap::new();
            for row in rows {
                if matches!(engine.get(row.values[0]), Ok(Some(_))) {
                    let v = row.values.get(col_idx).copied().unwrap_or(0);
                    let bucket_start = (v / bucket_sz) * bucket_sz;
                    *buckets.entry(bucket_start).or_insert(0) += 1;
                }
            }
            let out: Vec<Vec<u64>> = buckets.into_iter().map(|(k, v)| vec![k, v]).collect();
            Response::Rows { rows: out }
        }

        Request::ZScoreFilter { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let live: Vec<_> = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .collect();
            if live.is_empty() {
                return Response::Rows { rows: vec![] };
            }
            let n = live.len() as f64;
            let sum: f64 = live
                .iter()
                .map(|r| r.values.get(col_idx).copied().unwrap_or(0) as f64)
                .sum();
            let mean = sum / n;
            let variance = live
                .iter()
                .map(|r| {
                    let v = r.values.get(col_idx).copied().unwrap_or(0) as f64;
                    (v - mean) * (v - mean)
                })
                .sum::<f64>()
                / n;
            let stddev = variance.sqrt();
            let threshold = 2.0 * stddev;
            let filtered: Vec<Vec<u64>> = live
                .into_iter()
                .filter(|r| {
                    let v = r.values.get(col_idx).copied().unwrap_or(0) as f64;
                    (v - mean).abs() <= threshold
                })
                .map(|r| r.values)
                .collect();
            Response::Rows { rows: filtered }
        }

        Request::RunningSum { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut running = 0u64;
            let out: Vec<Vec<u64>> = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .map(|r| {
                    let v = r.values.get(col_idx).copied().unwrap_or(0);
                    running = running.saturating_add(v);
                    vec![r.values[0], v, running]
                })
                .collect();
            Response::Rows { rows: out }
        }

        Request::FloorDiv {
            col,
            divisor,
            lo,
            hi,
        } => {
            let col_idx = col as usize;
            if divisor == 0 {
                return Response::Error {
                    code: error_code::MALFORMED,
                    message: "divisor cannot be zero".into(),
                };
            }
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut count = 0u64;
            for row in rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                {
                    live.values[col_idx] /= divisor;
                    if engine.insert(pk, live.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::MulField {
            col,
            factor,
            lo,
            hi,
        } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut count = 0u64;
            for row in rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                {
                    live.values[col_idx] = live.values[col_idx].saturating_mul(factor);
                    if engine.insert(pk, live.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::SetFieldAdd { pk, col, addend } => {
            let col_idx = col as usize;
            match engine.get(pk) {
                Ok(Some(mut row)) => {
                    if col_idx >= row.values.len() {
                        return Response::Error {
                            code: error_code::SCHEMA,
                            message: "column index out of range".into(),
                        };
                    }
                    row.values[col_idx] = row.values[col_idx].saturating_add(addend);
                    let new_val = row.values[col_idx];
                    match engine.insert(pk, row.values) {
                        Ok(_) => Response::Ok { affected: new_val },
                        Err(e) => engine_error(e),
                    }
                }
                Ok(None) => Response::NotFound,
                Err(e) => engine_error(e),
            }
        }

        Request::SetFieldMul { pk, col, factor } => {
            let col_idx = col as usize;
            match engine.get(pk) {
                Ok(Some(mut row)) => {
                    if col_idx >= row.values.len() {
                        return Response::Error {
                            code: error_code::SCHEMA,
                            message: "column index out of range".into(),
                        };
                    }
                    row.values[col_idx] = row.values[col_idx].saturating_mul(factor);
                    let new_val = row.values[col_idx];
                    match engine.insert(pk, row.values) {
                        Ok(_) => Response::Ok { affected: new_val },
                        Err(e) => engine_error(e),
                    }
                }
                Ok(None) => Response::NotFound,
                Err(e) => engine_error(e),
            }
        }

        Request::SwapPkFields { pk, col_a, col_b } => {
            let a_idx = col_a as usize;
            let b_idx = col_b as usize;
            match engine.get(pk) {
                Ok(Some(mut row)) => {
                    if a_idx >= row.values.len() || b_idx >= row.values.len() {
                        return Response::Error {
                            code: error_code::SCHEMA,
                            message: "column index out of range".into(),
                        };
                    }
                    row.values.swap(a_idx, b_idx);
                    match engine.insert(pk, row.values) {
                        Ok(_) => Response::Ok { affected: 1 },
                        Err(e) => engine_error(e),
                    }
                }
                Ok(None) => Response::NotFound,
                Err(e) => engine_error(e),
            }
        }

        Request::CountRange { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let count = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .count() as u64;
            Response::Ok { affected: count }
        }

        Request::SumRange { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let sum = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .map(|r| r.values.get(col_idx).copied().unwrap_or(0))
                .fold(0u64, |a, v| a.saturating_add(v));
            Response::Ok { affected: sum }
        }

        Request::MinRange { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let min_val = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .filter_map(|r| r.values.get(col_idx).copied())
                .min();
            match min_val {
                Some(v) => Response::Ok { affected: v },
                None => Response::NotFound,
            }
        }

        Request::MaxRange { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let max_val = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .filter_map(|r| r.values.get(col_idx).copied())
                .max();
            match max_val {
                Some(v) => Response::Ok { affected: v },
                None => Response::NotFound,
            }
        }

        Request::AvgRange { col, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let vals: Vec<u64> = rows
                .into_iter()
                .filter(|r| matches!(engine.get(r.values[0]), Ok(Some(_))))
                .filter_map(|r| r.values.get(col_idx).copied())
                .collect();
            if vals.is_empty() {
                return Response::NotFound;
            }
            let sum: u64 = vals.iter().copied().fold(0u64, |a, v| a.saturating_add(v));
            Response::Ok {
                affected: sum / vals.len() as u64,
            }
        }

        Request::IncrementField { pk, col } => {
            let col_idx = col as usize;
            match engine.get(pk) {
                Ok(Some(mut row)) if col_idx < row.values.len() => {
                    row.values[col_idx] = row.values[col_idx].saturating_add(1);
                    let new_val = row.values[col_idx];
                    match engine.insert(pk, row.values) {
                        Ok(_) => Response::Ok { affected: new_val },
                        Err(e) => engine_error(e),
                    }
                }
                Ok(Some(_)) => Response::Error {
                    code: error_code::SCHEMA,
                    message: "column index out of range".into(),
                },
                Ok(None) => Response::NotFound,
                Err(e) => engine_error(e),
            }
        }

        Request::DecrementField { pk, col } => {
            let col_idx = col as usize;
            match engine.get(pk) {
                Ok(Some(mut row)) if col_idx < row.values.len() => {
                    row.values[col_idx] = row.values[col_idx].saturating_sub(1);
                    let new_val = row.values[col_idx];
                    match engine.insert(pk, row.values) {
                        Ok(_) => Response::Ok { affected: new_val },
                        Err(e) => engine_error(e),
                    }
                }
                Ok(Some(_)) => Response::Error {
                    code: error_code::SCHEMA,
                    message: "column index out of range".into(),
                },
                Ok(None) => Response::NotFound,
                Err(e) => engine_error(e),
            }
        }

        Request::SetMin { pk, col, value } => {
            let col_idx = col as usize;
            match engine.get(pk) {
                Ok(Some(mut row)) if col_idx < row.values.len() => {
                    row.values[col_idx] = row.values[col_idx].min(value);
                    let new_val = row.values[col_idx];
                    match engine.insert(pk, row.values) {
                        Ok(_) => Response::Ok { affected: new_val },
                        Err(e) => engine_error(e),
                    }
                }
                Ok(Some(_)) => Response::Error {
                    code: error_code::SCHEMA,
                    message: "column index out of range".into(),
                },
                Ok(None) => Response::NotFound,
                Err(e) => engine_error(e),
            }
        }

        Request::SetMax { pk, col, value } => {
            let col_idx = col as usize;
            match engine.get(pk) {
                Ok(Some(mut row)) if col_idx < row.values.len() => {
                    row.values[col_idx] = row.values[col_idx].max(value);
                    let new_val = row.values[col_idx];
                    match engine.insert(pk, row.values) {
                        Ok(_) => Response::Ok { affected: new_val },
                        Err(e) => engine_error(e),
                    }
                }
                Ok(Some(_)) => Response::Error {
                    code: error_code::SCHEMA,
                    message: "column index out of range".into(),
                },
                Ok(None) => Response::NotFound,
                Err(e) => engine_error(e),
            }
        }

        Request::XorField { col, mask, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut count = 0u64;
            for row in rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                {
                    live.values[col_idx] ^= mask;
                    if engine.insert(pk, live.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::AndField { col, mask, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut count = 0u64;
            for row in rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                {
                    live.values[col_idx] &= mask;
                    if engine.insert(pk, live.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::OrField { col, mask, lo, hi } => {
            let col_idx = col as usize;
            let rows = match engine.scan(col_idx, lo, hi) {
                Ok(r) => r,
                Err(e) => return engine_error(e),
            };
            let mut count = 0u64;
            for row in rows {
                let pk = row.values[0];
                if let Ok(Some(mut live)) = engine.get(pk)
                    && col_idx < live.values.len()
                {
                    live.values[col_idx] |= mask;
                    if engine.insert(pk, live.values).is_ok() {
                        count += 1;
                    }
                }
            }
            Response::Ok { affected: count }
        }

        Request::BatchGetField { col, pks } => {
            let col_idx = col as usize;
            let mut rows: Vec<Vec<u64>> = Vec::new();
            for pk in pks {
                if let Ok(Some(row)) = engine.get(pk) {
                    let val = row.values.get(col_idx).copied().unwrap_or(0);
                    rows.push(vec![pk, val]);
                }
            }
            Response::Rows { rows }
        }

        // Handled by auth gate or the catalog/txn blocks above.
        Request::Hello
        | Request::Auth { .. }
        | Request::Ping
        | Request::UseTable { .. }
        | Request::CreateTable { .. }
        | Request::DropTable { .. }
        | Request::ListTables
        | Request::TableExists { .. }
        | Request::BeginTxn
        | Request::CommitTxn { .. }
        | Request::RollbackTxn { .. } => Response::Error {
            code: error_code::MALFORMED,
            message: "unexpected dispatch".into(),
        },
    }
}

/// Map an engine error onto the wire.
///
/// The message is included because a client debugging a failure needs it, and the
/// engine's errors describe storage state rather than anything secret. If that
/// stops being true — once auth exists and errors could leak another tenant's
/// details — this is the single place to redact.
fn engine_error(e: vectorvolt_engine::EngineError) -> Response {
    let message = e.to_string();
    let code = if message.contains("duplicate") {
        error_code::DUPLICATE_KEY
    } else if message.contains("not found") {
        error_code::NOT_FOUND
    } else if message.contains("schema") || message.contains("column") {
        error_code::SCHEMA
    } else {
        error_code::ENGINE
    };
    Response::Error { code, message }
}
