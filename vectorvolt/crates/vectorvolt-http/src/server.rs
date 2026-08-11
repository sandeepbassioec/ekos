use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::routing::{delete, get, post};
use tokio::net::TcpListener;
use vectorvolt_engine::TableRegistry;

use crate::handlers;
use crate::metrics::Metrics;
use crate::obs;

/// Shared state injected into every axum handler.
pub struct AppState {
    pub registry: Arc<TableRegistry>,
    pub metrics: Arc<Metrics>,
}

/// Configuration for the HTTP gateway.
pub struct HttpServerConfig {
    pub addr: SocketAddr,
}

impl HttpServerConfig {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

/// Axum-backed HTTP/REST gateway.
pub struct HttpServer {
    listener: TcpListener,
    app: Router,
}

impl HttpServer {
    /// Bind the HTTP server to `config.addr` backed by the given registry.
    pub async fn bind(
        registry: Arc<TableRegistry>,
        config: HttpServerConfig,
    ) -> std::io::Result<Self> {
        let metrics = Metrics::new();
        let app = build_router(registry, metrics);
        let listener = TcpListener::bind(config.addr).await?;
        Ok(Self { listener, app })
    }

    /// The actual local address (useful when port 0 was requested).
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Run until the process is killed. This future never returns `Ok`.
    pub async fn serve(self) -> std::io::Result<()> {
        axum::serve(self.listener, self.app).await
    }
}

fn build_router(registry: Arc<TableRegistry>, metrics: Arc<Metrics>) -> Router {
    let state = Arc::new(AppState { registry, metrics });
    Router::new()
        // Observability
        .route("/health", get(obs::health))
        .route("/metrics", get(obs::prometheus_metrics))
        // Table management
        .route("/tables", get(handlers::list_tables))
        .route("/tables/{table}", post(handlers::create_table))
        .route("/tables/{table}", delete(handlers::drop_table))
        .route("/tables/{table}/schema", get(handlers::describe_table))
        // Row operations
        .route("/tables/{table}/rows/{pk}", post(handlers::insert_row))
        .route("/tables/{table}/rows/{pk}", get(handlers::get_row))
        .route("/tables/{table}/rows/{pk}", delete(handlers::delete_row))
        // Queries
        .route("/tables/{table}/scan", get(handlers::scan_rows))
        .route("/tables/{table}/secondary", get(handlers::secondary_lookup))
        .with_state(state)
}
