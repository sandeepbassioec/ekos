//! Shared plain-wire-protocol client helper for the small network commands
//! (`ping`, `status`, `bench-net`, `server status`).

use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::Args;
use vectorvolt_net::Client;

/// Connection options shared by the network commands.
#[derive(Args, Debug, Clone)]
pub struct WireOpts {
    /// Connect + per-call timeout in milliseconds.
    #[arg(long, default_value = "5000")]
    pub timeout_ms: u64,
    /// 32-byte shared secret (hex) when the server requires authentication.
    #[arg(long)]
    pub secret: Option<String>,
    /// Key ID sent with `--secret`.
    #[arg(long, default_value = "default")]
    pub key_id: String,
}

impl Default for WireOpts {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            secret: None,
            key_id: "default".into(),
        }
    }
}

/// Resolve `host:port` (IPv6 literals may be bracketed) to a socket address.
pub fn resolve(host: &str, port: u16) -> Result<SocketAddr> {
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if host.is_empty() {
        bail!("host must not be empty");
    }
    (host, port)
        .to_socket_addrs()
        .with_context(|| format!("resolve {host}:{port}"))?
        .next()
        .with_context(|| format!("no address for {host}:{port}"))
}

/// Open a (plaintext, optionally authenticated) wire-protocol connection.
pub fn open(host: &str, port: u16, opts: &WireOpts) -> Result<Client> {
    let addr = resolve(host, port)?;
    let timeout = Duration::from_millis(opts.timeout_ms.max(1));
    let client = match &opts.secret {
        Some(hex_secret) => {
            let bytes = hex::decode(hex_secret.trim())
                .context("secret must be hex (e.g. from `vvctl keygen`)")?;
            let secret: [u8; 32] = bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("secret must be exactly 32 bytes (64 hex chars)"))?;
            Client::connect_authenticated(addr, &opts.key_id, &secret)
                .with_context(|| format!("connect+auth to {addr}"))?
        }
        None => {
            Client::connect_timeout(addr, timeout).with_context(|| format!("connect to {addr}"))?
        }
    };
    client.set_timeout(Some(timeout))?;
    Ok(client)
}
