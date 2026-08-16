use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::server::AppState;
use crate::types::{
    AggResp, BucketResp, CountResp, CreateTableBody, DistinctResp, ErrorResp, GroupByAggQuery,
    GroupByQuery, GroupEntryResp, GroupResp, HistogramQuery, HistogramResp, IncrDecrWhereQuery,
    InsertBody, OkResp, PageResp, PercentileQuery, PercentileResp, QueryResp, RangeQuery, RowResp,
    RowsResp, ScanPageQuery, ScanQuery, ScanSortedQuery, SchemaResp, SecondaryQuery, TablesResp,
    UpdateFieldBody, UpdateWhereQuery, UpdatedResp, ValueCountEntry, ValueCountsResp,
    WindowAggQuery, WindowCountQuery, WindowRankQuery, WindowResp, WindowRowResp,
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
                vectorvolt_query::QueryResponse::Histogram(bkts) => Json(QueryResp::Histogram {
                    buckets: bkts
                        .into_iter()
                        .map(|b| BucketResp {
                            lo: b.lo,
                            hi: b.hi,
                            count: b.count,
                        })
                        .collect(),
                }),
                vectorvolt_query::QueryResponse::Percentile(v) => {
                    Json(QueryResp::Percentile { value: v })
                }
                vectorvolt_query::QueryResponse::GroupResult(pairs) => Json(QueryResp::Group {
                    entries: pairs
                        .into_iter()
                        .map(|(g, a)| GroupEntryResp {
                            group_val: g,
                            agg_val: a,
                        })
                        .collect(),
                }),
                vectorvolt_query::QueryResponse::WindowRows(wrows) => Json(QueryResp::Window {
                    rows: wrows
                        .into_iter()
                        .map(|w| WindowRowResp {
                            pk: w.pk,
                            window_val: w.window_val,
                        })
                        .collect(),
                }),
                vectorvolt_query::QueryResponse::DistinctValues(vals) => {
                    Json(QueryResp::Distinct { values: vals })
                }
                vectorvolt_query::QueryResponse::ValueCounts(pairs) => {
                    Json(QueryResp::ValueCounts {
                        entries: pairs
                            .into_iter()
                            .map(|(v, c)| ValueCountEntry { value: v, count: c })
                            .collect(),
                    })
                }
                vectorvolt_query::QueryResponse::UpdatedCount(n) => {
                    Json(QueryResp::Updated { updated: n })
                }
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

// ── Layer 50: histogram + percentile endpoints ────────────────────────────────

/// `GET /tables/{table}/histogram?col=&lo=&hi=&buckets=`
pub async fn histogram(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<HistogramQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.histogram_range(q.col, q.lo, q.hi, q.buckets) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(bkts) => {
                st.metrics.record_ok();
                Json(HistogramResp {
                    buckets: bkts
                        .into_iter()
                        .map(|b| BucketResp {
                            lo: b.lo,
                            hi: b.hi,
                            count: b.count,
                        })
                        .collect(),
                })
                .into_response()
            }
        },
    }
}

/// `GET /tables/{table}/percentile?col=&lo=&hi=&p=` (`p` is 0–100)
pub async fn percentile(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<PercentileQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.percentile_range(q.col, q.lo, q.hi, q.p as f64) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(v) => {
                st.metrics.record_ok();
                Json(PercentileResp { value: v }).into_response()
            }
        },
    }
}

// ── Layer 51: group-by aggregate endpoints ────────────────────────────────────

fn to_group_resp(pairs: Vec<(u64, u64)>) -> GroupResp {
    GroupResp {
        entries: pairs
            .into_iter()
            .map(|(g, a)| GroupEntryResp {
                group_val: g,
                agg_val: a,
            })
            .collect(),
    }
}

/// `GET /tables/{table}/group/count?filter_col=&lo=&hi=&group_col=`
pub async fn group_count(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<GroupByQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.group_by_count(q.filter_col, q.lo, q.hi, q.group_col) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(pairs) => {
                st.metrics.record_ok();
                Json(to_group_resp(pairs)).into_response()
            }
        },
    }
}

/// `GET /tables/{table}/group/sum?filter_col=&lo=&hi=&group_col=&agg_col=`
pub async fn group_sum(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<GroupByAggQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.group_by_sum(q.filter_col, q.lo, q.hi, q.group_col, q.agg_col) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(pairs) => {
                st.metrics.record_ok();
                Json(to_group_resp(pairs)).into_response()
            }
        },
    }
}

/// `GET /tables/{table}/group/min?filter_col=&lo=&hi=&group_col=&agg_col=`
pub async fn group_min(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<GroupByAggQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.group_by_min(q.filter_col, q.lo, q.hi, q.group_col, q.agg_col) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(pairs) => {
                st.metrics.record_ok();
                Json(to_group_resp(pairs)).into_response()
            }
        },
    }
}

/// `GET /tables/{table}/group/max?filter_col=&lo=&hi=&group_col=&agg_col=`
pub async fn group_max(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<GroupByAggQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.group_by_max(q.filter_col, q.lo, q.hi, q.group_col, q.agg_col) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(pairs) => {
                st.metrics.record_ok();
                Json(to_group_resp(pairs)).into_response()
            }
        },
    }
}

// ── Layer 52: window function endpoints ───────────────────────────────────────

fn to_window_resp(rows: Vec<vectorvolt_engine::WindowRow>) -> WindowResp {
    WindowResp {
        rows: rows
            .into_iter()
            .map(|w| WindowRowResp {
                pk: w.pk,
                window_val: w.window_val,
            })
            .collect(),
    }
}

/// `GET /tables/{table}/window/running_sum?filter_col=&lo=&hi=&agg_col=`
pub async fn window_running_sum(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<WindowAggQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.running_sum(q.filter_col, q.lo, q.hi, q.agg_col) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(rows) => {
                st.metrics.record_ok();
                Json(to_window_resp(rows)).into_response()
            }
        },
    }
}

/// `GET /tables/{table}/window/running_count?filter_col=&lo=&hi=`
pub async fn window_running_count(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<WindowCountQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.running_count(q.filter_col, q.lo, q.hi) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(rows) => {
                st.metrics.record_ok();
                Json(to_window_resp(rows)).into_response()
            }
        },
    }
}

/// `GET /tables/{table}/window/rank?filter_col=&lo=&hi=&sort_col=&asc=`
pub async fn window_rank(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<WindowRankQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.rank_by(q.filter_col, q.lo, q.hi, q.sort_col, q.asc != 0) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(rows) => {
                st.metrics.record_ok();
                Json(to_window_resp(rows)).into_response()
            }
        },
    }
}

/// `GET /tables/{table}/window/dense_rank?filter_col=&lo=&hi=&sort_col=&asc=`
pub async fn window_dense_rank(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<WindowRankQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.dense_rank_by(q.filter_col, q.lo, q.hi, q.sort_col, q.asc != 0) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(rows) => {
                st.metrics.record_ok();
                Json(to_window_resp(rows)).into_response()
            }
        },
    }
}

// ── Layer 53: distinct + value_counts endpoints ───────────────────────────────

/// `GET /tables/{table}/distinct?col=&lo=&hi=`
pub async fn distinct(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<RangeQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.distinct_values(q.col, q.lo, q.hi) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(vals) => {
                st.metrics.record_ok();
                Json(DistinctResp { values: vals }).into_response()
            }
        },
    }
}

/// `GET /tables/{table}/value_counts?col=&lo=&hi=`
pub async fn value_counts(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<RangeQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.value_counts(q.col, q.lo, q.hi) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(pairs) => {
                st.metrics.record_ok();
                Json(ValueCountsResp {
                    entries: pairs
                        .into_iter()
                        .map(|(v, c)| ValueCountEntry { value: v, count: c })
                        .collect(),
                })
                .into_response()
            }
        },
    }
}

// ── Layer 54: bulk predicate field updates ────────────────────────────────────

pub async fn update_field_where(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<UpdateWhereQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.update_field_where(&[(q.filter_col, q.lo, q.hi)], q.target_col, q.val)
        {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(n) => {
                st.metrics.record_ok();
                Json(UpdatedResp { updated: n }).into_response()
            }
        },
    }
}

pub async fn increment_field_where(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<IncrDecrWhereQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => {
            match eng.increment_field_where(&[(q.filter_col, q.lo, q.hi)], q.target_col, q.delta) {
                Err(e) => engine_err(&st, e).into_response(),
                Ok(n) => {
                    st.metrics.record_ok();
                    Json(UpdatedResp { updated: n }).into_response()
                }
            }
        }
    }
}

pub async fn decrement_field_where(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<IncrDecrWhereQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => {
            match eng.decrement_field_where(&[(q.filter_col, q.lo, q.hi)], q.target_col, q.delta) {
                Err(e) => engine_err(&st, e).into_response(),
                Ok(n) => {
                    st.metrics.record_ok();
                    Json(UpdatedResp { updated: n }).into_response()
                }
            }
        }
    }
}
