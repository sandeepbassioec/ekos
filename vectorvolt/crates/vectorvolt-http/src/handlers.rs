use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::server::AppState;
use crate::types::{
    CreateTableBody, ErrorResp, InsertBody, OkResp, QueryResp, RowResp, RowsResp, ScanQuery,
    SchemaResp, SecondaryQuery, TablesResp,
};

fn engine_err(st: &AppState, e: impl std::fmt::Display) -> (StatusCode, Json<ErrorResp>) {
    st.metrics.record_err();
    let msg = e.to_string();
    let status = if msg.contains("NotFound") || msg.contains("not found") {
        StatusCode::NOT_FOUND
    } else if msg.contains("AlreadyExists") || msg.contains("already exists") {
        StatusCode::CONFLICT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    (status, Json(ErrorResp { error: msg }))
}

// ── row insert ────────────────────────────────────────────────────────────────

pub async fn insert_row(
    State(st): State<Arc<AppState>>,
    Path((table, pk)): Path<(String, u64)>,
    Json(body): Json<InsertBody>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.insert(pk, body.values.clone()) {
            Ok(_) => {
                st.metrics
                    .rows_inserted
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                st.metrics.record_ok();
                Json(RowResp {
                    pk,
                    values: body.values,
                })
                .into_response()
            }
            Err(e) => engine_err(&st, e).into_response(),
        },
    }
}

// ── row get ───────────────────────────────────────────────────────────────────

pub async fn get_row(
    State(st): State<Arc<AppState>>,
    Path((table, pk)): Path<(String, u64)>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.get(pk) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(None) => {
                st.metrics.record_err();
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResp {
                        error: format!("pk={pk} not found"),
                    }),
                )
                    .into_response()
            }
            Ok(Some(row)) => {
                st.metrics
                    .rows_fetched
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                st.metrics.record_ok();
                Json(RowResp {
                    pk: row.pk,
                    values: row.values,
                })
                .into_response()
            }
        },
    }
}

// ── row delete ────────────────────────────────────────────────────────────────

pub async fn delete_row(
    State(st): State<Arc<AppState>>,
    Path((table, pk)): Path<(String, u64)>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.delete(pk) {
            Ok(_) => {
                st.metrics
                    .rows_deleted
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                st.metrics.record_ok();
                Json(OkResp { ok: true }).into_response()
            }
            Err(e) => engine_err(&st, e).into_response(),
        },
    }
}

// ── scan ──────────────────────────────────────────────────────────────────────

pub async fn scan_rows(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<ScanQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.scan(q.col as usize, q.lo, q.hi) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(rows) => {
                let count = rows.len() as u64;
                st.metrics
                    .rows_fetched
                    .fetch_add(count, std::sync::atomic::Ordering::Relaxed);
                st.metrics
                    .scans_executed
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                st.metrics.record_ok();
                let resp = RowsResp {
                    rows: rows
                        .into_iter()
                        .map(|r| RowResp {
                            pk: r.pk,
                            values: r.values,
                        })
                        .collect(),
                };
                Json(resp).into_response()
            }
        },
    }
}

// ── secondary lookup ──────────────────────────────────────────────────────────

pub async fn secondary_lookup(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<SecondaryQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.secondary_scan(q.col, q.val) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(rows) => {
                let count = rows.len() as u64;
                st.metrics
                    .rows_fetched
                    .fetch_add(count, std::sync::atomic::Ordering::Relaxed);
                st.metrics
                    .secondary_lookups
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                st.metrics.record_ok();
                let resp = RowsResp {
                    rows: rows
                        .into_iter()
                        .map(|vals| {
                            let pk = vals.first().copied().unwrap_or(0);
                            RowResp { pk, values: vals }
                        })
                        .collect(),
                };
                Json(resp).into_response()
            }
        },
    }
}

// ── list tables ───────────────────────────────────────────────────────────────

pub async fn list_tables(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    st.metrics.record_ok();
    Json(TablesResp {
        tables: st.registry.list_tables(),
    })
}

// ── describe table ────────────────────────────────────────────────────────────

pub async fn describe_table(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => {
            st.metrics.record_ok();
            Json(SchemaResp {
                columns: eng.schema.clone(),
            })
            .into_response()
        }
    }
}

// ── create table ──────────────────────────────────────────────────────────────

pub async fn create_table(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Json(body): Json<CreateTableBody>,
) -> impl IntoResponse {
    match st.registry.create_table(&table, body.columns) {
        Ok(_) => {
            st.metrics.record_ok();
            (StatusCode::CREATED, Json(OkResp { ok: true })).into_response()
        }
        Err(e) => engine_err(&st, e).into_response(),
    }
}

// ── drop table ────────────────────────────────────────────────────────────────

pub async fn drop_table(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
) -> impl IntoResponse {
    match st.registry.drop_table(&table) {
        Ok(_) => {
            st.metrics.record_ok();
            Json(OkResp { ok: true }).into_response()
        }
        Err(e) => engine_err(&st, e).into_response(),
    }
}

// ── text query ────────────────────────────────────────────────────────────────

/// `POST /tables/{table}/query` — execute a text query via the VectorVolt DSL.
///
/// Request body: `text/plain` query string (e.g. `"GET pk=42"`, `"BEGIN"`,
/// `"SCAN col=1 lo=0 hi=100"`).
///
/// Response: JSON tagged-union keyed on `"type"`:
/// `"row"`, `"rows"`, `"txn_started"`, `"schema"`, `"stats"`, `"ok"`, `"not_found"`.
pub async fn query_handler(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    body: String,
) -> impl IntoResponse {
    let eng = match st.registry.get(&table) {
        Err(e) => return engine_err(&st, e).into_response(),
        Ok(eng) => eng,
    };

    match vectorvolt_query::run(&eng, &body) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(resp) => {
            st.metrics.record_ok();
            let json = match resp {
                vectorvolt_query::QueryResponse::Row(row) => Json(QueryResp::Row {
                    pk: row.pk,
                    values: row.values,
                }),
                vectorvolt_query::QueryResponse::Rows(rows) => Json(QueryResp::Rows {
                    rows: rows
                        .into_iter()
                        .map(|r| RowResp {
                            pk: r.pk,
                            values: r.values,
                        })
                        .collect(),
                }),
                vectorvolt_query::QueryResponse::TxnStarted(id) => {
                    Json(QueryResp::TxnStarted { txn_id: id })
                }
                vectorvolt_query::QueryResponse::Schema(cols) => {
                    Json(QueryResp::Schema { columns: cols })
                }
                vectorvolt_query::QueryResponse::Stats(s) => Json(QueryResp::Stats {
                    rows_buffered: s.rows_buffered,
                    rows_flushed: s.rows_flushed,
                    partition_count: s.partition_count,
                    schema: s.schema,
                }),
                vectorvolt_query::QueryResponse::Ok => Json(QueryResp::Ok),
                vectorvolt_query::QueryResponse::NotFound => Json(QueryResp::NotFound),
            };
            json.into_response()
        }
    }
}
