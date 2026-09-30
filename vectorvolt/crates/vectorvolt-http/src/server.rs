use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post, put};
use tokio::net::TcpListener;
use vectorvolt_engine::{TableRegistry, ViewRegistry};
use vectorvolt_query::SqlExecutor;

use crate::handlers;
use crate::metrics::Metrics;
use crate::multi_txn::MultiTxnRegistry;
use crate::obs;
use crate::webhook::WebhookRegistry;

/// Shared state injected into every axum handler.
pub struct AppState {
    pub registry: Arc<TableRegistry>,
    pub views: Arc<ViewRegistry>,
    pub metrics: Arc<Metrics>,
    pub webhooks: Arc<WebhookRegistry>,
    pub multi_txns: Arc<MultiTxnRegistry>,
    /// SQL executor for `POST /sql`.
    pub sql: Arc<SqlExecutor>,
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
        let views = Arc::new(ViewRegistry::new());
        let webhooks = Arc::new(WebhookRegistry::new());
        let multi_txns = Arc::new(MultiTxnRegistry::new());
        let sql = Arc::new(SqlExecutor::new(Arc::clone(&registry)));
        let app = build_router(
            registry,
            views,
            webhooks,
            multi_txns,
            sql,
            metrics,
            config.api_token,
        );
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

    /// Run until `shutdown` resolves, then stop accepting new connections and
    /// wait for in-flight requests to finish (axum's own graceful drain) before
    /// returning `Ok(())` (Gap #6: no listener in `vvctl serve` used to drain on
    /// shutdown at all).
    pub async fn serve_with_shutdown(
        self,
        shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> std::io::Result<()> {
        axum::serve(self.listener, self.app)
            .with_graceful_shutdown(shutdown)
            .await
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
    views: Arc<ViewRegistry>,
    webhooks: Arc<WebhookRegistry>,
    multi_txns: Arc<MultiTxnRegistry>,
    sql: Arc<SqlExecutor>,
    metrics: Arc<Metrics>,
    api_token: Option<String>,
) -> Router {
    let state = Arc::new(AppState {
        registry,
        views,
        webhooks,
        multi_txns,
        sql,
        metrics,
        api_token,
    });
    Router::new()
        // Observability
        .route("/health", get(obs::health))
        .route("/metrics", get(obs::prometheus_metrics))
        // SQL gateway (S28)
        .route("/sql", post(handlers::sql_handler))
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
        // Layer 44: analytical endpoints
        .route("/tables/{table}/count", get(handlers::count_rows))
        .route("/tables/{table}/sum", get(handlers::sum_rows))
        .route("/tables/{table}/min", get(handlers::min_rows))
        .route("/tables/{table}/max", get(handlers::max_rows))
        .route("/tables/{table}/avg", get(handlers::avg_rows))
        .route("/tables/{table}/scan/sorted", get(handlers::scan_sorted))
        .route("/tables/{table}/scan/page", get(handlers::scan_page))
        // Layer 44: write/mutate endpoints
        .route("/tables/{table}/rows/{pk}", put(handlers::upsert_row))
        .route(
            "/tables/{table}/rows/{pk}/field",
            patch(handlers::update_field),
        )
        .route("/tables/{table}/rows/range", delete(handlers::delete_range))
        // Layer 50: histogram + percentile
        .route("/tables/{table}/histogram", get(handlers::histogram))
        .route("/tables/{table}/percentile", get(handlers::percentile))
        // Layer 51: group-by aggregates
        .route("/tables/{table}/group/count", get(handlers::group_count))
        .route("/tables/{table}/group/sum", get(handlers::group_sum))
        .route("/tables/{table}/group/min", get(handlers::group_min))
        .route("/tables/{table}/group/max", get(handlers::group_max))
        // Layer 52: window functions
        .route(
            "/tables/{table}/window/running_sum",
            get(handlers::window_running_sum),
        )
        .route(
            "/tables/{table}/window/running_count",
            get(handlers::window_running_count),
        )
        .route("/tables/{table}/window/rank", get(handlers::window_rank))
        .route(
            "/tables/{table}/window/dense_rank",
            get(handlers::window_dense_rank),
        )
        // Layer 53: distinct + value_counts
        .route("/tables/{table}/distinct", get(handlers::distinct))
        .route("/tables/{table}/value_counts", get(handlers::value_counts))
        // Layer 54: bulk predicate field updates
        .route(
            "/tables/{table}/update_where",
            get(handlers::update_field_where),
        )
        .route(
            "/tables/{table}/increment_where",
            get(handlers::increment_field_where),
        )
        .route(
            "/tables/{table}/decrement_where",
            get(handlers::decrement_field_where),
        )
        // Layer 55: batch operations
        .route("/tables/{table}/batch/get", post(handlers::batch_get))
        .route("/tables/{table}/batch/delete", post(handlers::batch_delete))
        // Layer 56: multi-predicate compound scan
        .route("/tables/{table}/scan_where", get(handlers::scan_where))
        // Layer 57: column projection scan
        .route("/tables/{table}/scan/project", get(handlers::scan_project))
        // Layer 58: conditional writes
        .route(
            "/tables/{table}/rows/{pk}/if_absent",
            post(handlers::insert_if_absent),
        )
        .route(
            "/tables/{table}/rows/{pk}/cas",
            post(handlers::compare_and_swap),
        )
        // Layer 59: batch insert
        .route("/tables/{table}/batch/insert", post(handlers::batch_insert))
        // Layer 60: TTL operations
        .route(
            "/tables/{table}/rows/{pk}/ttl",
            post(handlers::insert_with_ttl),
        )
        .route(
            "/tables/{table}/get_with_ttl/{pk}",
            get(handlers::get_with_ttl),
        )
        .route(
            "/tables/{table}/scan_with_ttl",
            get(handlers::scan_with_ttl),
        )
        .route(
            "/tables/{table}/purge_expired",
            post(handlers::purge_expired),
        )
        // Layer 65: CDC / SSE subscribe
        .route(
            "/tables/{table}/subscribe",
            get(handlers::subscribe_handler),
        )
        // Layer 66: Materialized Views DDL
        .route("/views", post(handlers::create_view))
        .route("/views", get(handlers::list_views))
        .route("/views/{name}", get(handlers::get_view))
        .route("/views/{name}", delete(handlers::drop_view))
        // Layer 67: CDC Event Log
        .route("/tables/{table}/cdc/log", get(handlers::cdc_log_handler))
        // Layer 68: CDC Stats + Compaction
        .route(
            "/tables/{table}/cdc/stats",
            get(handlers::cdc_stats_handler),
        )
        .route(
            "/tables/{table}/cdc/compact",
            post(handlers::cdc_compact_handler),
        )
        // Layer 69: WAL Snapshot
        .route(
            "/tables/{table}/snapshot",
            post(handlers::take_snapshot_handler).get(handlers::snapshot_info_handler),
        )
        // Layer 70: CDC Webhook Sinks
        .route(
            "/tables/{table}/webhooks",
            post(handlers::register_webhook_handler).get(handlers::list_webhooks_handler),
        )
        .route(
            "/tables/{table}/webhooks/{id}",
            delete(handlers::delete_webhook_handler),
        )
        // Layer 71: Multi-Table Transactions
        .route("/multi_txn", post(handlers::multi_txn_begin_handler))
        .route("/multi_txn/{id}", get(handlers::multi_txn_info_handler))
        .route(
            "/multi_txn/{id}/ops",
            post(handlers::multi_txn_stage_handler),
        )
        .route(
            "/multi_txn/{id}/commit",
            post(handlers::multi_txn_commit_handler),
        )
        .route(
            "/multi_txn/{id}/rollback",
            post(handlers::multi_txn_rollback_handler),
        )
        .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            auth_middleware,
        ))
        .with_state(state)
}
