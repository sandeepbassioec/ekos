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

pub use auth::{AuthConfig, AuthError, AuthState, Scope, Secret};
pub use client::Client;
pub use protocol::{Header, OpCode, Request, Response};
pub use server::{Server, ServerConfig};

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
}

pub type NetResult<T> = Result<T, NetError>;

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
}
