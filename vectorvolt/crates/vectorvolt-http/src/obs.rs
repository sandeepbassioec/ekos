use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::AppState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// ── health ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct HealthResp {
    pub status: &'static str,
    pub version: &'static str,
    pub tables: usize,
    pub requests_total: u64,
    pub requests_ok: u64,
    pub requests_err: u64,
}

pub async fn health(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    Json(HealthResp {
        status: "ok",
        version: VERSION,
        tables: st.registry.list_tables().len(),
        requests_total: st.metrics.requests_total.load(Ordering::Relaxed),
        requests_ok: st.metrics.requests_ok.load(Ordering::Relaxed),
        requests_err: st.metrics.requests_err.load(Ordering::Relaxed),
    })
}

// ── metrics (Prometheus text format) ─────────────────────────────────────────

pub async fn prometheus_metrics(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    let m = &st.metrics;
    let tables = st.registry.list_tables();

    let mut out = String::with_capacity(2048);

    // Gateway-level counters
    push_counter(
        &mut out,
        "vectorvolt_http_requests_total",
        "Total HTTP requests received",
        m.requests_total.load(Ordering::Relaxed),
        &[],
    );
    push_counter(
        &mut out,
        "vectorvolt_http_requests_ok_total",
        "HTTP requests that returned 2xx",
        m.requests_ok.load(Ordering::Relaxed),
        &[],
    );
    push_counter(
        &mut out,
        "vectorvolt_http_requests_err_total",
        "HTTP requests that returned 4xx/5xx",
        m.requests_err.load(Ordering::Relaxed),
        &[],
    );
    push_counter(
        &mut out,
        "vectorvolt_rows_inserted_total",
        "Rows successfully inserted via HTTP",
        m.rows_inserted.load(Ordering::Relaxed),
        &[],
    );
    push_counter(
        &mut out,
        "vectorvolt_rows_deleted_total",
        "Rows deleted via HTTP",
        m.rows_deleted.load(Ordering::Relaxed),
        &[],
    );
    push_counter(
        &mut out,
        "vectorvolt_rows_fetched_total",
        "Rows returned by get/scan/secondary via HTTP",
        m.rows_fetched.load(Ordering::Relaxed),
        &[],
    );
    push_counter(
        &mut out,
        "vectorvolt_scans_total",
        "Range scan operations executed",
        m.scans_executed.load(Ordering::Relaxed),
        &[],
    );
    push_counter(
        &mut out,
        "vectorvolt_secondary_lookups_total",
        "Secondary index lookups executed",
        m.secondary_lookups.load(Ordering::Relaxed),
        &[],
    );

    // Per-table engine stats
    out.push_str(
        "# HELP vectorvolt_table_rows_buffered Rows in the write buffer (not yet flushed)\n",
    );
    out.push_str("# TYPE vectorvolt_table_rows_buffered gauge\n");
    for name in &tables {
        if let Ok(eng) = st.registry.get(name) {
            let s = eng.stats();
            out.push_str(&format!(
                "vectorvolt_table_rows_buffered{{table=\"{name}\"}} {}\n",
                s.rows_buffered
            ));
        }
    }

    out.push_str("# HELP vectorvolt_table_rows_flushed_total Rows written to partition files\n");
    out.push_str("# TYPE vectorvolt_table_rows_flushed_total counter\n");
    for name in &tables {
        if let Ok(eng) = st.registry.get(name) {
            let s = eng.stats();
            out.push_str(&format!(
                "vectorvolt_table_rows_flushed_total{{table=\"{name}\"}} {}\n",
                s.rows_flushed
            ));
        }
    }

    out.push_str("# HELP vectorvolt_table_partitions Partition files currently loaded\n");
    out.push_str("# TYPE vectorvolt_table_partitions gauge\n");
    for name in &tables {
        if let Ok(eng) = st.registry.get(name) {
            let s = eng.stats();
            out.push_str(&format!(
                "vectorvolt_table_partitions{{table=\"{name}\"}} {}\n",
                s.partition_count
            ));
        }
    }

    (
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        out,
    )
}

fn push_counter(out: &mut String, name: &str, help: &str, val: u64, labels: &[(&str, &str)]) {
    out.push_str(&format!("# HELP {name} {help}\n"));
    out.push_str(&format!("# TYPE {name} counter\n"));
    if labels.is_empty() {
        out.push_str(&format!("{name} {val}\n"));
    } else {
        let lstr: String = labels
            .iter()
            .map(|(k, v)| format!("{k}=\"{v}\""))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&format!("{name}{{{lstr}}} {val}\n"));
    }
}
