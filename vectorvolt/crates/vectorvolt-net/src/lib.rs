//! `vectorvolt-net` — wire protocol, server, and client (Layer 7).
//!
//! Before this, VectorVolt was a Rust library: correct, fast, and reachable only
//! by linking it. This is the layer that lets something else connect.
//!
//! | Module | What it is |
//! |---|---|
//! | [`protocol`] | Pure codec over byte slices. No I/O, so hostile input is testable without a socket. |
//! | [`server`] | Thread-per-connection TCP server over a `TableEngine`. |
//! | [`client`] | Blocking client. |
//!
//! ## Why thread-per-connection, and not async
//!
//! An async runtime is the reflexive choice and it is not obviously right here.
//! VectorVolt targets *latency*, and a thread-per-connection server has no
//! scheduler between the socket and the engine — a request is handled on the
//! thread that read it, start to finish. Tokio buys throughput at high connection
//! counts by multiplexing, at the cost of a scheduling hop and a work-stealing
//! queue on the critical path.
//!
//! It is also the shape that matches the roadmap: thread-per-core with pinned
//! threads (VV-HAL-027) is a refinement of this, and a rewrite away from async.
//!
//! The cost is real and stated: one OS thread per connection does not scale to
//! tens of thousands of idle clients. That is the trade, and it is revisitable —
//! the protocol codec is entirely independent of how bytes are moved.

pub mod auth;
pub mod client;
pub mod protocol;
pub mod server;
pub mod tls;

pub use auth::{AuthConfig, AuthError, AuthState, Scope, Secret};
pub use client::Client;
pub use protocol::{Header, OpCode, Request, Response};
pub use server::{Server, ServerConfig, ServerStats};
pub use tls::{ClientTlsConfig, ServerTlsConfig};
pub use vectorvolt_engine::TableRegistry;

/// Errors from framing, parsing, or transport.
#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("frame does not start with the VectorVolt magic")]
    BadMagic,

    #[error("unsupported protocol version {0}")]
    UnsupportedVersion(u8),

    #[error("unknown opcode {0}")]
    UnknownOpcode(u8),

    #[error("opcode {0} is a response, not a request")]
    NotARequest(u8),

    #[error("opcode {0} is a request, not a response")]
    NotAResponse(u8),

    #[error("payload of {len} bytes exceeds the {max}-byte limit")]
    PayloadTooLarge { len: u32, max: u32 },

    #[error("truncated: needed {need} bytes, got {got}")]
    Truncated { need: usize, got: usize },

    #[error("payload CRC mismatch: expected {expected:08x}, computed {actual:08x}")]
    CrcMismatch { expected: u32, actual: u32 },

    #[error("malformed message: {0}")]
    Malformed(&'static str),

    #[error("engine error: {0}")]
    Engine(String),

    #[error("authentication: {0}")]
    Auth(#[from] auth::AuthError),

    #[error("TLS: {0}")]
    Tls(String),
}

pub type NetResult<T> = Result<T, NetError>;

/// Simple network address configuration (VV-NET-185).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetConfig {
    pub addr: String,
    pub port: u16,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1".to_string(),
            port: 7700,
        }
    }
}

impl NetConfig {
    pub fn new(addr: impl Into<String>, port: u16) -> Self {
        Self {
            addr: addr.into(),
            port,
        }
    }

    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
}

/// Bind a TCP listener to the address in `config` (VV-NET-186).
pub fn bind(config: &NetConfig) -> anyhow::Result<std::net::TcpListener> {
    let listener = std::net::TcpListener::bind(config.socket_addr())?;
    Ok(listener)
}

/// Simple frame encoding: 4-byte little-endian length prefix + payload (VV-NET-189).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub len: u32,
    pub payload: Vec<u8>,
}

pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let len = payload.len() as u32;
    let mut out = Vec::with_capacity(4 + payload.len());
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(payload);
    out
}

pub fn decode_frame(data: &[u8]) -> Option<Frame> {
    if data.len() < 4 {
        return None;
    }
    let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if data.len() < 4 + len as usize {
        return None;
    }
    Some(Frame {
        len,
        payload: data[4..4 + len as usize].to_vec(),
    })
}

/// Lightweight net client stub (VV-NET-190).
#[derive(Debug, Clone)]
pub struct NetClient {
    pub addr: String,
}

impl NetClient {
    pub fn connect(addr: &str) -> anyhow::Result<Self> {
        Ok(Self {
            addr: addr.to_string(),
        })
    }

    pub fn ping(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Connection pool stub (VV-NET-195).
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    pub max: usize,
}

impl ConnectionPool {
    pub fn new(max: usize) -> Self {
        Self { max }
    }

    pub fn size(&self) -> usize {
        self.max
    }
}

/// TLS configuration stub (VV-NET-199).
#[derive(Debug, Clone, Default)]
pub struct TlsConfig {
    pub enabled: bool,
}

pub fn apply_tls(_cfg: &TlsConfig, listener: std::net::TcpListener) -> std::net::TcpListener {
    listener
}

/// Error codes carried in [`Response::Error`].
///
/// Numeric and stable so a client can branch on them without parsing English —
/// the message is for humans and may change.
pub mod error_code {
    /// The request was well-formed but the operation failed.
    pub const ENGINE: u16 = 1;
    /// The request could not be parsed.
    pub const MALFORMED: u16 = 2;
    /// The primary key already exists.
    pub const DUPLICATE_KEY: u16 = 3;
    /// The primary key does not exist.
    pub const NOT_FOUND: u16 = 4;
    /// The row does not match the table schema.
    pub const SCHEMA: u16 = 5;
    /// The connection has not authenticated.
    pub const UNAUTHENTICATED: u16 = 6;
    /// Authentication failed.
    pub const AUTH_FAILED: u16 = 7;
    /// Authenticated, but this key lacks the required scope.
    ///
    /// Distinct from UNAUTHENTICATED on purpose: "log in" and "you cannot do this"
    /// need different responses from a client, and conflating them sends operators
    /// hunting for a credential problem that does not exist.
    pub const FORBIDDEN: u16 = 8;
    /// No table has been selected on this connection.
    pub const NOT_SELECTED: u16 = 9;
    /// Named table not found in the registry.
    pub const TABLE_NOT_FOUND: u16 = 10;
    /// Named table already exists.
    pub const TABLE_EXISTS: u16 = 11;
    /// A transaction is already active on this connection.
    pub const TXN_ACTIVE: u16 = 12;
    /// No transaction is active on this connection.
    pub const NO_TXN: u16 = 13;
    /// The provided txn_id does not match the active transaction.
    pub const TXN_ID_MISMATCH: u16 = 14;
    /// A request argument is out of range or otherwise invalid.
    pub const INVALID_ARGUMENT: u16 = 15;
}
