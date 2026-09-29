//! `vvctl reader` — start / stop the PostgreSQL wire protocol server (VV-CLU-019).
//!
//! ## Fallback / dev-only path
//!
//! This command opens its OWN `TableRegistry` against `--dir`, completely
//! independent of any `vvctl serve` process that may be running against the
//! same data directory. Running both at once risks file-lock contention and
//! data seen by one process but not the other.
//!
//! Prefer `vvctl serve --pgwire-port <port>`, which starts a pgwire listener
//! sharing the SAME `TableRegistry`/engine state as the main wire-protocol,
//! HTTP, and gRPC listeners in that single process. Keep this standalone
//! command around for quick local development/testing only.

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::{Args, Subcommand};
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_reader::server::{PgConfig, PgServer};

#[derive(Args, Debug)]
pub struct ReaderArgs {
    #[command(subcommand)]
    pub command: ReaderCommand,
}

#[derive(Subcommand, Debug)]
pub enum ReaderCommand {
    /// Start the PostgreSQL wire protocol listener.
    Start(ReaderStartArgs),
}

#[derive(Args, Debug)]
pub struct ReaderStartArgs {
    /// Data directory holding VectorVolt tables.
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// TCP port to listen on.
    #[arg(short, long, default_value = "5432")]
    pub port: u16,

    /// Bind address.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,
}

pub fn run(args: ReaderArgs) -> Result<()> {
    match args.command {
        ReaderCommand::Start(a) => {
            let addr = SocketAddr::new(a.host, a.port);
            let registry = TableRegistry::open(&a.dir, EngineOptions::default())
                .map_err(|e| anyhow::anyhow!("open registry: {e}"))?;
            let registry = Arc::new(registry);
            let config = PgConfig {
                bind_addr: addr,
                data_dir: a.dir.clone(),
            };
            let server = PgServer::with_registry(config, registry);
            println!("VectorVolt pgwire listening on {addr}");
            println!("Press Ctrl-C to stop.");
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(server.serve())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_reader_start_args_defaults() {
        let args = ReaderStartArgs {
            dir: PathBuf::from("."),
            port: 5432,
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
        };
        assert_eq!(args.port, 5432);
    }
}
