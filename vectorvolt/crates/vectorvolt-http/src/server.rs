use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
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
    /// When `Some`, every non-observability request must carry
    /// `Authorization: Bearer <token>`.
    pub api_token: Option<String>,
}

/// Configuration for the HTTP gateway.
pub struct HttpServerConfig {
    pub addr: SocketAddr,
    /// Optional Bearer token. When set, every request to data/DDL routes
    /// must supply `Authorization: Bearer <token>`.
    pub api_token: Option<String>,
}

impl HttpServerConfig {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            api_token: None,
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.api_token = Some(token.into());
        self
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
        let app = build_router(registry, metrics, config.api_token);
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

/// Middleware that enforces `Authorization: Bearer <token>` when the gateway
/// was configured with a token.  `/health` and `/metrics` are always public.
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if let Some(expected) = &state.api_token {
        let path = req.uri().path();
        if path != "/health" && path != "/metrics" {
            let ok = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .map(|v| v == format!("Bearer {expected}"))
                .unwrap_or(false);
            if !ok {
                return (
                    StatusCode::UNAUTHORIZED,
                    axum::Json(serde_json::json!({ "error": "unauthorized" })),
                )
                    .into_response();
            }
        }
    }
    next.run(req).await
}

fn build_router(
    registry: Arc<TableRegistry>,
    metrics: Arc<Metrics>,
    api_token: Option<String>,
) -> Router {
    let state = Arc::new(AppState {
        registry,
        metrics,
        api_token,
    });
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
        // Text query DSL
        .route("/tables/{table}/query", post(handlers::query_handler))
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            auth_middleware,
        ))
        .with_state(state)
}
