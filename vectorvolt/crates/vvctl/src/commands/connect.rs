//! `vvctl connect` — interactive / scripted client for a running VectorVolt server.
//!
//! ## Subcommands
//!
//! ```text
//! vvctl connect ping                        # liveness check
//! vvctl connect get <pk>
//! vvctl connect insert <pk> [val...]
//! vvctl connect delete <pk>
//! vvctl connect scan --col 1 --lo 100 --hi 200 [--limit 50]
//! vvctl connect stats
//! vvctl connect flush
//! vvctl connect compact
//! ```
//!
//! ## Authentication
//!
//! Pass `--secret <hex>` to authenticate. The key-id defaults to `"default"` but
//! can be overridden with `--key-id`.
//!
//! ## TLS
//!
//! Pass `--ca <pem>` to trust a specific CA (e.g. a self-signed cert from `vvctl serve`).
//! Without `--ca`, the system trust store is used.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use vectorvolt_net::Client;
use vectorvolt_net::tls::ClientTlsConfig;

// ── Top-level args shared by every subcommand ─────────────────────────────────

#[derive(Args)]
pub struct ConnectArgs {
    /// Server address.
    #[arg(long, short, default_value = "127.0.0.1:5995")]
    pub addr: SocketAddr,

    /// 32-byte shared secret (hex) for authentication.
    #[arg(long)]
    pub secret: Option<String>,

    /// Key ID to send with `--secret`.
    #[arg(long, default_value = "default")]
    pub key_id: String,

    /// PEM CA certificate to trust (for TLS with self-signed certs).
    #[arg(long)]
    pub ca: Option<PathBuf>,

    /// TLS server name (SNI). Defaults to the hostname in `--addr`.
    #[arg(long)]
    pub tls_name: Option<String>,

    /// Connection + call timeout in milliseconds.
    #[arg(long, default_value = "5000")]
    pub timeout_ms: u64,

    #[command(subcommand)]
    pub command: ConnectCommand,
}

#[derive(Subcommand)]
pub enum ConnectCommand {
    /// Check if the server is alive (no auth required).
    Ping,

    /// Retrieve a row by primary key.
    Get {
        /// Primary key.
        pk: u64,
    },

    /// Insert a row.
    Insert {
        /// Primary key.
        pk: u64,
        /// Column values in schema order (excluding pk).
        #[arg(num_args = 0..)]
        values: Vec<u64>,
    },

    /// Delete a row by primary key.
    Delete {
        /// Primary key.
        pk: u64,
    },

    /// Range scan on a column (both bounds inclusive).
    Scan {
        /// Column index (0 = pk).
        #[arg(long, default_value = "0")]
        col: u32,
        /// Minimum value (inclusive).
        #[arg(long, default_value = "0")]
        lo: u64,
        /// Maximum value (inclusive).
        #[arg(long, default_value = "18446744073709551615")]
        hi: u64,
        /// Maximum rows to return (0 = unlimited).
        #[arg(long, default_value = "0")]
        limit: u32,
    },

    /// Show server-side table statistics.
    Stats,

    /// Flush the write buffer to a new partition file.
    Flush,

    /// Compact all partitions into one, removing deleted rows.
    Compact,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(args: ConnectArgs) -> Result<()> {
    let timeout = Duration::from_millis(args.timeout_ms);
    let mut client = open_client(&args, timeout)?;

    match args.command {
        ConnectCommand::Ping => {
            client.ping().with_context(|| "ping")?;
            println!("pong — server is alive at {}", args.addr);
        }

        ConnectCommand::Get { pk } => {
            match client.get(pk).with_context(|| format!("get pk={pk}"))? {
                None => println!("not found: pk={pk}"),
                Some(values) => {
                    println!("pk={pk}");
                    for (i, v) in values.iter().enumerate() {
                        println!("  col[{i}] = {v}");
                    }
                }
            }
        }

        ConnectCommand::Insert { pk, values } => {
            client
                .insert(pk, values)
                .with_context(|| format!("insert pk={pk}"))?;
            println!("inserted pk={pk}");
        }

        ConnectCommand::Delete { pk } => {
            let existed = client
                .delete(pk)
                .with_context(|| format!("delete pk={pk}"))?;
            if existed {
                println!("deleted pk={pk}");
            } else {
                println!("not found (nothing deleted): pk={pk}");
            }
        }

        ConnectCommand::Scan { col, lo, hi, limit } => {
            let rows = client
                .range_scan(col, lo, hi, limit)
                .with_context(|| format!("scan col={col} [{lo},{hi}]"))?;
            if rows.is_empty() {
                println!("no rows matched col[{col}] in [{lo}, {hi}]");
                return Ok(());
            }
            // Print as tab-separated values; column headers use generic names
            // because the client does not know the schema.
            let col_count = rows[0].len();
            let header: Vec<String> = (0..col_count).map(|i| format!("col[{i}]")).collect();
            println!("{}", header.join("\t"));
            println!(
                "{}",
                header
                    .iter()
                    .map(|_| "────────")
                    .collect::<Vec<_>>()
                    .join("\t")
            );
            for row in &rows {
                let line: Vec<String> = row.iter().map(|v| v.to_string()).collect();
                println!("{}", line.join("\t"));
            }
            println!("\n{} row(s)", rows.len());
        }

        ConnectCommand::Stats => {
            let (rows, partitions, bytes) = client.stats().with_context(|| "stats")?;
            println!("rows       : {rows}");
            println!("partitions : {partitions}");
            println!("bytes      : {bytes}");
        }

        ConnectCommand::Flush => {
            client.flush().with_context(|| "flush")?;
            println!("flush ok");
        }

        ConnectCommand::Compact => {
            let kept = client.compact().with_context(|| "compact")?;
            println!("compact ok — {kept} row(s) kept");
        }
    }

    Ok(())
}

// ── Client construction ───────────────────────────────────────────────────────

fn open_client(args: &ConnectArgs, timeout: Duration) -> Result<Client> {
    // Parse optional secret before borrowing args.
    let secret_bytes: Option<([u8; 32], &str)> = if let Some(hex) = &args.secret {
        let bytes = hex::decode(hex.trim())
            .with_context(|| "secret must be hex (e.g. from `openssl rand -hex 32`)")?;
        if bytes.len() != 32 {
            bail!("secret must be exactly 32 bytes (64 hex chars)");
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Some((arr, args.key_id.as_str()))
    } else {
        None
    };

    let client = if let Some(ca_path) = &args.ca {
        // TLS with a custom CA (e.g. self-signed cert from `vvctl serve --cert`).
        let server_name = args
            .tls_name
            .clone()
            .unwrap_or_else(|| args.addr.ip().to_string());
        let ca_pem = std::fs::read(ca_path)
            .with_context(|| format!("read CA cert {}", ca_path.display()))?;
        let tls = ClientTlsConfig::pin_certificate(&ca_pem, &server_name)
            .with_context(|| format!("pin CA cert {}", ca_path.display()))?;

        let mut c = Client::connect_tls(args.addr, &tls)
            .with_context(|| format!("connect TLS to {}", args.addr))?;
        c.set_timeout(Some(timeout))?;

        if let Some((secret, key_id)) = &secret_bytes {
            c.authenticate(key_id, secret)
                .with_context(|| "authenticate")?;
        }
        c
    } else if let Some((secret, key_id)) = &secret_bytes {
        // Plaintext with auth.
        let c = Client::connect_authenticated(args.addr, key_id, secret)
            .with_context(|| format!("connect+auth to {}", args.addr))?;
        c.set_timeout(Some(timeout))?;
        c
    } else {
        // Plaintext, no auth.
        let c = Client::connect_timeout(args.addr, timeout)
            .with_context(|| format!("connect to {}", args.addr))?;
        c.set_timeout(Some(timeout))?;
        c
    };

    Ok(client)
}
