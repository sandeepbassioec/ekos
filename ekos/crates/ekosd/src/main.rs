use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tonic::transport::Server;
use tracing::info;

mod config;
mod mapping;
mod service;

pub mod proto {
    pub mod ekos {
        #[allow(clippy::result_large_err, clippy::double_must_use)]
        pub mod v1 {
            tonic::include_proto!("ekos.v1");
        }
    }
}

use proto::ekos::v1::kernel_service_server::KernelServiceServer;
use service::EkosKernelService;

#[derive(Parser)]
#[command(
    name    = "ekosd",
    version = env!("CARGO_PKG_VERSION"),
    about   = "EKOS Knowledge Runtime — garvionlabs.com",
)]
struct Cli {
    /// Path to ekosd.toml (defaults to ./ekosd.toml)
    #[arg(long, default_value = "ekosd.toml")]
    config: PathBuf,

    /// Override bind address (e.g. 0.0.0.0:50051)
    #[arg(long)]
    bind: Option<String>,

    /// Override data directory
    #[arg(long)]
    data_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut cfg = config::Config::load(&cli.config)?;
    if let Some(b) = cli.bind {
        cfg.bind = b;
    }
    if let Some(d) = cli.data_dir {
        cfg.data_dir = d;
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&cfg.log_level)),
        )
        .with_target(false)
        .compact()
        .init();

    std::fs::create_dir_all(&cfg.data_dir)?;

    let store_cfg = vectorvolt_store::StoreConfig::new(&cfg.data_dir)
        .with_flush_threshold(cfg.store.flush_threshold_bytes)
        .with_compaction_trigger(cfg.store.compaction_trigger)
        .with_fsync(cfg.store.fsync);

    let store = ekos_kernel::KoStore::open(store_cfg)?;
    let svc = EkosKernelService::new(store);

    let addr = cfg.bind.parse()?;
    info!("ekosd listening on {addr}");
    info!("data directory: {}", cfg.data_dir.display());

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Handle Ctrl-C / SIGTERM for graceful shutdown
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = sigterm.recv() => {}
            }
        }
        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
        }
        info!("shutdown signal received — draining in-flight RPCs");
        let _ = shutdown_tx.send(());
    });

    Server::builder()
        .add_service(KernelServiceServer::new(svc))
        .serve_with_shutdown(addr, async {
            let _ = shutdown_rx.await;
        })
        .await?;

    info!("ekosd stopped cleanly");
    Ok(())
}
