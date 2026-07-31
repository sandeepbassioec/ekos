//! [`Server`] — a TCP server over a [`TableEngine`] (VV-NET-004/005).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use vectorvolt_engine::TableEngine;

use crate::auth::{AuthConfig, AuthState};
use crate::protocol::{HEADER_SIZE, Header, Request, Response, encode_response, verify_payload};
use crate::{NetError, NetResult, error_code};

/// Server tuning.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub addr: SocketAddr,
    /// Connections served concurrently. Beyond this, accepted sockets wait.
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

    pub fn new(addr: SocketAddr) -> Self {
        ServerConfig {
            addr,
            max_connections: 256,
            nodelay: true,
            // Disabled by default. An embedded or single-process deployment does
            // not want a handshake, and a server that silently required one would
            // simply appear broken. `requires_auth()` reports which mode is
            // active, so a deployment can assert it rather than assume.
            auth: AuthConfig::disabled(),
        }
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
    /// Failed authentication attempts. Worth alerting on: a rising count is
    /// either a misconfigured client or someone guessing.
    pub auth_failures: AtomicU64,
    pub auth_successes: AtomicU64,
}

/// A running server.
pub struct Server {
    engine: Arc<TableEngine>,
    config: ServerConfig,
    listener: TcpListener,
    shutdown: Arc<AtomicBool>,
    stats: Arc<ServerStats>,
    live: Arc<AtomicU64>,
}

impl Server {
    /// Bind the listener. Does not begin serving.
    ///
    /// Binding is separate from serving so a caller can learn the real address
    /// before any client connects — which is what makes a test able to bind port 0
    /// and then connect to whatever the OS chose, instead of guessing a port and
    /// racing another test for it.
    pub fn bind(engine: Arc<TableEngine>, config: ServerConfig) -> NetResult<Self> {
        let listener = TcpListener::bind(config.addr)?;
        Ok(Server {
            engine,
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

            let engine = Arc::clone(&self.engine);
            let stats = Arc::clone(&self.stats);
            let live = Arc::clone(&self.live);
            let shutdown = Arc::clone(&self.shutdown);
            let auth = self.config.auth.clone();
            std::thread::spawn(move || {
                let _ = handle_connection(stream, &engine, &stats, &shutdown, &auth);
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
fn read_exact_or_eof(stream: &mut TcpStream, buf: &mut [u8]) -> NetResult<bool> {
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

fn handle_connection(
    mut stream: TcpStream,
    engine: &TableEngine,
    stats: &ServerStats,
    shutdown: &AtomicBool,
    auth_config: &AuthConfig,
) -> NetResult<()> {
    let mut header_buf = [0u8; HEADER_SIZE];
    // Auth state is per CONNECTION, not per process. Authenticating once must not
    // authorise anyone else, and a new socket starts over.
    let mut auth = AuthState::new(auth_config);

    loop {
        if shutdown.load(Ordering::Acquire) {
            return Ok(());
        }
        if !read_exact_or_eof(&mut stream, &mut header_buf)? {
            return Ok(());
        }

        // A bad header cannot be answered: without a valid request_id there is
        // nothing to correlate a reply to, and the stream position is unknown, so
        // anything after it would be parsed as a frame boundary that is not one.
        // Closing is the only honest response.
        let header = match Header::decode(&header_buf) {
            Ok(h) => h,
            Err(e) => {
                stats.protocol_errors.fetch_add(1, Ordering::Relaxed);
                return Err(e);
            }
        };

        // Allocation happens only after the header validated the length against
        // MAX_PAYLOAD.
        let mut payload = vec![0u8; header.payload_len as usize];
        if header.payload_len > 0 && !read_exact_or_eof(&mut stream, &mut payload)? {
            return Err(NetError::Truncated {
                need: header.payload_len as usize,
                got: 0,
            });
        }

        // From here the frame is well-formed enough to answer, so errors become
        // responses rather than disconnections. A client that sends one bad
        // request should learn why, not be dropped.
        let response = match verify_payload(&header, &payload)
            .and_then(|_| Request::decode(header.opcode, &payload))
        {
            Ok(req) => {
                stats.requests_handled.fetch_add(1, Ordering::Relaxed);
                match serve_authenticated(engine, req, &mut auth, auth_config, stats) {
                    Ok(r) => r,
                    // Lock-out: answer, then close. Replying first means the client
                    // learns why instead of seeing an unexplained reset.
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
/// `Err(response)` means "send this and close the connection" — used only for
/// lock-out, where continuing to serve would defeat the attempt limit.
///
/// ## What an unauthenticated connection may do
///
/// Only `Hello`, `Auth`, and `Ping`. Everything else is refused.
///
/// `Ping` is allowed deliberately: a load balancer or health check needs to know
/// the server is alive without holding credentials, and a liveness probe reveals
/// nothing that a successful TCP connect does not already reveal. `Stats` is NOT
/// allowed, even though it looks similarly harmless — row counts and byte totals
/// are information about the data, and an unauthenticated peer should learn
/// nothing about it.
fn serve_authenticated(
    engine: &TableEngine,
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
        other => Ok(serve_request(engine, other)),
    }
}

fn serve_request(engine: &TableEngine, req: Request) -> Response {
    match req {
        Request::Ping => Response::Pong,

        Request::Get { pk } => match engine.get(pk) {
            Ok(Some(row)) => Response::Row { values: row.values },
            Ok(None) => Response::NotFound,
            Err(e) => engine_error(e),
        },

        Request::Insert { pk, values } => match engine.insert(pk, values) {
            Ok(_) => Response::Ok { affected: 1 },
            Err(e) => engine_error(e),
        },

        Request::Delete { pk } => match engine.delete(pk) {
            Ok(_) => Response::Ok { affected: 1 },
            // A delete of a missing key is NOT an engine failure, it is a normal
            // outcome, and it gets its own code so a client can tell "nothing to
            // delete" from "the delete broke".
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("not found") || msg.contains("PkNotFound") {
                    Response::Ok { affected: 0 }
                } else {
                    engine_error(e)
                }
            }
        },

        Request::RangeScan { col, lo, hi, limit } => match engine.scan(col as usize, lo, hi) {
            Ok(rows) => {
                // limit 0 means "no limit" — a client asking for zero rows would
                // simply not send the request.
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

        // Handled by the auth gate before reaching here.
        Request::Hello | Request::Auth { .. } => Response::Error {
            code: error_code::MALFORMED,
            message: "auth messages are handled before dispatch".into(),
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
