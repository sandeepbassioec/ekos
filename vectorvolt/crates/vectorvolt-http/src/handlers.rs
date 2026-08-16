use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::server::AppState;
use crate::types::{
    AggResp, CountResp, CreateTableBody, ErrorResp, InsertBody, OkResp, PageResp, QueryResp,
    RangeQuery, RowResp, RowsResp, ScanPageQuery, ScanQuery, ScanSortedQuery, SchemaResp,
    SecondaryQuery, TablesResp, UpdateFieldBody,
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
                vectorvolt_query::QueryResponse::Count(n) => Json(QueryResp::Count { count: n }),
                vectorvolt_query::QueryResponse::Aggregate(v) => {
                    Json(QueryResp::Aggregate { value: v })
                }
                vectorvolt_query::QueryResponse::Page(page) => Json(QueryResp::Page {
                    rows: page
                        .rows
                        .into_iter()
                        .map(|r| RowResp {
                            pk: r.pk,
                            values: r.values,
                        })
                        .collect(),
                    next_cursor: page.next_cursor,
                }),
            };
            json.into_response()
        }
    }
}

// ── Layer 44: analytical endpoints ───────────────────────────────────────────

pub async fn count_rows(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<RangeQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.count_range(q.col, q.lo, q.hi) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(n) => {
                st.metrics.record_ok();
                Json(CountResp { count: n }).into_response()
            }
        },
    }
}

pub async fn sum_rows(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<RangeQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.sum_range(q.col, q.lo, q.hi) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(v) => {
                st.metrics.record_ok();
                Json(AggResp { value: Some(v) }).into_response()
            }
        },
    }
}

pub async fn min_rows(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<RangeQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.min_range(q.col, q.lo, q.hi) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(v) => {
                st.metrics.record_ok();
                Json(AggResp { value: v }).into_response()
            }
        },
    }
}

pub async fn max_rows(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<RangeQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.max_range(q.col, q.lo, q.hi) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(v) => {
                st.metrics.record_ok();
                Json(AggResp { value: v }).into_response()
            }
        },
    }
}

pub async fn avg_rows(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<RangeQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.avg_range(q.col, q.lo, q.hi) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(v) => {
                st.metrics.record_ok();
                Json(AggResp {
                    value: v.map(|f| f.round() as u64),
                })
                .into_response()
            }
        },
    }
}

pub async fn scan_sorted(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<ScanSortedQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.scan_sorted(q.col, q.lo, q.hi, q.sort, q.asc != 0, q.limit) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(rows) => {
                st.metrics.record_ok();
                Json(RowsResp {
                    rows: rows
                        .into_iter()
                        .map(|r| RowResp {
                            pk: r.pk,
                            values: r.values,
                        })
                        .collect(),
                })
                .into_response()
            }
        },
    }
}

pub async fn scan_page(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<ScanPageQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.scan_page(q.col, q.lo, q.hi, q.after, q.limit) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(page) => {
                st.metrics.record_ok();
                Json(PageResp {
                    rows: page
                        .rows
                        .into_iter()
                        .map(|r| RowResp {
                            pk: r.pk,
                            values: r.values,
                        })
                        .collect(),
                    next_cursor: page.next_cursor,
                })
                .into_response()
            }
        },
    }
}

// ── Layer 44: write/mutate endpoints ─────────────────────────────────────────

/// `PUT /tables/{table}/rows/{pk}` — upsert: insert or overwrite a row.
pub async fn upsert_row(
    State(st): State<Arc<AppState>>,
    Path((table, pk)): Path<(String, u64)>,
    Json(body): Json<InsertBody>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.upsert(pk, body.values.clone()) {
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

/// `PATCH /tables/{table}/rows/{pk}/field` — update a single field in a row.
pub async fn update_field(
    State(st): State<Arc<AppState>>,
    Path((table, pk)): Path<(String, u64)>,
    Json(body): Json<UpdateFieldBody>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.update_field(pk, body.col, body.val) {
            Ok(_) => {
                st.metrics.record_ok();
                Json(OkResp { ok: true }).into_response()
            }
            Err(e) => engine_err(&st, e).into_response(),
        },
    }
}

/// `DELETE /tables/{table}/rows/range?col=&lo=&hi=` — delete all rows in range.
pub async fn delete_range(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<RangeQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.delete_range(q.col, q.lo, q.hi) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(n) => {
                st.metrics
                    .rows_deleted
                    .fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
                st.metrics.record_ok();
                Json(CountResp { count: n as u64 }).into_response()
            }
        },
    }
}
