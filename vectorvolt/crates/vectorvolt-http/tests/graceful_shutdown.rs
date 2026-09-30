/// Gap #6 — HTTP gateway graceful shutdown.
///
/// `serve_with_shutdown` must (a) keep serving requests normally until the
/// shutdown signal fires, and (b) stop accepting new connections once it does,
/// without needing the whole process to exit first.
use std::sync::Arc;

use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

const FLUSH_THRESHOLD: usize = 1_048_576;

async fn start_registry(dir: &TempDir) -> Arc<TableRegistry> {
    let opts = EngineOptions::new(FLUSH_THRESHOLD);
    Arc::new(TableRegistry::open(dir.path(), opts).unwrap())
}

#[tokio::test]
async fn serves_requests_before_shutdown_signal() {
    let dir = TempDir::new().unwrap();
    let reg = start_registry(&dir).await;
    let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
    let srv = HttpServer::bind(reg, cfg).await.unwrap();
    let addr = srv.local_addr().unwrap();

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let handle = tokio::spawn(async move {
        srv.serve_with_shutdown(async {
            let _ = rx.await;
        })
        .await
    });

    let resp = reqwest::get(format!("http://{addr}/health")).await.unwrap();
    assert!(resp.status().is_success());

    tx.send(()).unwrap();
    handle.await.unwrap().unwrap();
}

#[tokio::test]
async fn stops_accepting_new_connections_after_shutdown_signal() {
    let dir = TempDir::new().unwrap();
    let reg = start_registry(&dir).await;
    let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
    let srv = HttpServer::bind(reg, cfg).await.unwrap();
    let addr = srv.local_addr().unwrap();

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    // Fire shutdown immediately, before any request is made.
    tx.send(()).unwrap();
    let handle = tokio::spawn(async move {
        srv.serve_with_shutdown(async {
            let _ = rx.await;
        })
        .await
    });
    handle.await.unwrap().unwrap();

    // The listener is gone once serve_with_shutdown returns — a fresh
    // connection attempt to the same address must fail.
    let result = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap()
        .get(format!("http://{addr}/health"))
        .send()
        .await;
    assert!(result.is_err());
}
