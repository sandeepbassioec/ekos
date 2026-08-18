use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;

use crate::server::AppState;
use crate::types::{
    AggResp, BatchDeleteResp, BatchGetResp, BatchInsertBody, BatchInsertResp, BatchPksBody,
    BucketResp, CasBody, CasResp, CdcCompactQuery, CdcCompactResp, CdcLogQuery, CdcLogResp,
    CdcStatsResp, ConditionalInsertResp, CountResp, CreateTableBody, CreateViewBody, DistinctResp,
    ErrorResp, GroupByAggQuery, GroupByQuery, GroupEntryResp, GroupResp, HistogramQuery,
    HistogramResp, IncrDecrWhereQuery, InsertBody, InsertTtlBody, OkResp, PageResp,
    PercentileQuery, PercentileResp, PurgedResp, QueryResp, RangeQuery, RegisterWebhookBody,
    RegisterWebhookResp, RowResp, RowsResp, ScanPageQuery, ScanProjectQuery, ScanProjectResp,
    ScanQuery, ScanSortedQuery, ScanWhereQuery, SchemaResp, SecondaryQuery, SnapshotInfoResp,
    SnapshotResp, SubscribeQuery, TablesResp, TtlNowQuery, UpdateFieldBody, UpdateWhereQuery,
    UpdatedResp, ValueCountEntry, ValueCountsResp, ViewEntryResp, ViewInfoResp, ViewStateResp,
    ViewsListResp, WebhookEntryResp, WebhooksListResp, WindowAggQuery, WindowCountQuery,
    WindowRankQuery, WindowResp, WindowRowResp,
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
                vectorvolt_query::QueryResponse::BatchRows(results) => Json(QueryResp::BatchRows {
                    rows: results
                        .into_iter()
                        .map(|opt| {
                            opt.map(|r| RowResp {
                                pk: r.pk,
                                values: r.values,
                            })
                        })
                        .collect(),
                }),
                vectorvolt_query::QueryResponse::BatchDeleted(n) => {
                    Json(QueryResp::BatchDeleted { deleted: n })
                }
                vectorvolt_query::QueryResponse::ScanWhereRows(rows) => {
                    Json(QueryResp::ScanWhere {
                        rows: rows
                            .into_iter()
                            .map(|r| RowResp {
                                pk: r.pk,
                                values: r.values,
                            })
                            .collect(),
                    })
                }
                vectorvolt_query::QueryResponse::ProjectedRows(rows) => {
                    Json(QueryResp::ProjectedRows {
                        rows: rows
                            .into_iter()
                            .map(|r| RowResp {
                                pk: r.pk,
                                values: r.values,
                            })
                            .collect(),
                    })
                }
                vectorvolt_query::QueryResponse::ConditionalInserted(b) => {
                    Json(QueryResp::ConditionalInserted { inserted: b })
                }
                vectorvolt_query::QueryResponse::Swapped(b) => {
                    Json(QueryResp::Swapped { swapped: b })
                }
                vectorvolt_query::QueryResponse::BatchInserted(n) => {
                    Json(QueryResp::BatchInserted { inserted: n })
                }
                vectorvolt_query::QueryResponse::TtlRow(Some(row)) => Json(QueryResp::TtlRow {
                    pk: row.pk,
                    values: row.values,
                }),
                vectorvolt_query::QueryResponse::TtlRow(None) => Json(QueryResp::NotFound),
                vectorvolt_query::QueryResponse::TtlRows(rows) => Json(QueryResp::TtlRows {
                    rows: rows
                        .into_iter()
                        .map(|r| RowResp {
                            pk: r.pk,
                            values: r.values,
                        })
                        .collect(),
                }),
                vectorvolt_query::QueryResponse::Purged(n) => Json(QueryResp::Purged { purged: n }),
            };
            json.into_response()
        }
    }
}

// ── Layer 57: column projection scan ─────────────────────────────────────────

pub async fn scan_project(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<ScanProjectQuery>,
) -> impl IntoResponse {
    let projection = q.into_projection();
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.scan_project(q.filter_col, q.lo, q.hi, &projection) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(rows) => {
                st.metrics.record_ok();
                Json(ScanProjectResp {
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

// ── Layer 55: batch operations ────────────────────────────────────────────────

pub async fn batch_get(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Json(body): Json<BatchPksBody>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.batch_get(&body.pks) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(results) => {
                st.metrics.record_ok();
                Json(BatchGetResp {
                    rows: results
                        .into_iter()
                        .map(|opt| {
                            opt.map(|r| RowResp {
                                pk: r.pk,
                                values: r.values,
                            })
                        })
                        .collect(),
                })
                .into_response()
            }
        },
    }
}

pub async fn batch_delete(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Json(body): Json<BatchPksBody>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.batch_delete(&body.pks) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(n) => {
                st.metrics.record_ok();
                Json(BatchDeleteResp { deleted: n }).into_response()
            }
        },
    }
}

// ── Layer 58: conditional writes ─────────────────────────────────────────────

pub async fn insert_if_absent(
    State(st): State<Arc<AppState>>,
    Path((table, pk)): Path<(String, u64)>,
    Json(body): Json<InsertBody>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.insert_if_absent(pk, body.values) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(inserted) => {
                st.metrics.record_ok();
                Json(ConditionalInsertResp { inserted }).into_response()
            }
        },
    }
}

pub async fn compare_and_swap(
    State(st): State<Arc<AppState>>,
    Path((table, pk)): Path<(String, u64)>,
    Json(body): Json<CasBody>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.compare_and_swap(pk, &body.expected, body.new_values) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(swapped) => {
                st.metrics.record_ok();
                Json(CasResp { swapped }).into_response()
            }
        },
    }
}

// ── Layer 59: batch insert ────────────────────────────────────────────────────

pub async fn batch_insert(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Json(body): Json<BatchInsertBody>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => {
            let rows = body.rows.into_iter().map(|r| (r.pk, r.values));
            match eng.batch_insert(rows) {
                Err(e) => engine_err(&st, e).into_response(),
                Ok(n) => {
                    st.metrics.record_ok();
                    Json(BatchInsertResp { inserted: n }).into_response()
                }
            }
        }
    }
}

// ── Layer 60: TTL operations ──────────────────────────────────────────────────

pub async fn insert_with_ttl(
    State(st): State<Arc<AppState>>,
    Path((table, pk)): Path<(String, u64)>,
    Json(body): Json<InsertTtlBody>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.insert_with_ttl(pk, body.values, body.expire_at) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(()) => {
                st.metrics.record_ok();
                Json(OkResp { ok: true }).into_response()
            }
        },
    }
}

pub async fn get_with_ttl(
    State(st): State<Arc<AppState>>,
    Path((table, pk)): Path<(String, u64)>,
    Query(q): Query<TtlNowQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.get_with_ttl(pk, q.now) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(Some(row)) => {
                st.metrics.record_ok();
                Json(RowResp {
                    pk: row.pk,
                    values: row.values,
                })
                .into_response()
            }
            Ok(None) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResp {
                    error: "not found or expired".into(),
                }),
            )
                .into_response(),
        },
    }
}

pub async fn scan_with_ttl(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<TtlNowQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.scan_with_ttl(q.now) {
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

pub async fn purge_expired(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<TtlNowQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.purge_expired(q.now) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(n) => {
                st.metrics.record_ok();
                Json(PurgedResp { purged: n }).into_response()
            }
        },
    }
}

// ── Layer 56: multi-predicate compound scan ───────────────────────────────────

pub async fn scan_where(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(q): Query<ScanWhereQuery>,
) -> impl IntoResponse {
    let predicates = q.into_predicates();
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.scan_where(&predicates) {
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

// ── Layer 65: CDC / SSE subscribe ────────────────────────────────────────────

/// `GET /tables/{table}/subscribe[?col=&lo=&hi=]`
///
/// Opens a Server-Sent Events stream.  Each event is a JSON-serialised
/// [`ChangeEvent`].  The optional `col`/`lo`/`hi` query parameters filter
/// events: only events whose `values[col]` falls in `[lo, hi]` are forwarded.
/// Delete events (no values) are suppressed when a filter is active.
pub async fn subscribe_handler(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(query): Query<SubscribeQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => {
            // Subscribe FIRST (before reading the log) so we don't miss events
            // that arrive while we're replaying history.
            let rx = eng.subscribe_changes();

            // Build historical prefix when ?from=<lsn> is present.
            let history: Vec<_> = if let Some(from_lsn) = query.from {
                eng.read_log_from(from_lsn).unwrap_or_default()
            } else {
                vec![]
            };
            let last_replayed = history.last().map(|e| e.lsn).unwrap_or(0);

            let hist_stream = tokio_stream::iter(history).map(Ok::<_, std::convert::Infallible>);

            let live_stream = BroadcastStream::new(rx).filter_map(move |result| {
                match result {
                    Ok(e) if e.lsn > last_replayed => Some(Ok(e)),
                    Ok(_) => None,  // already covered by the historical prefix
                    Err(_) => None, // lagged receiver — skip and continue
                }
            });

            let combined = hist_stream.chain(live_stream).filter_map(move |result| {
                let ev = match result {
                    Ok(e) => e,
                    Err(_) => return None,
                };
                if let (Some(col), Some(lo), Some(hi)) = (query.col, query.lo, query.hi) {
                    match &ev.values {
                        Some(vals) if col < vals.len() && vals[col] >= lo && vals[col] <= hi => {}
                        _ => return None,
                    }
                }
                let json = serde_json::to_string(&ev).ok()?;
                Some(Ok::<Event, std::convert::Infallible>(
                    Event::default().data(json),
                ))
            });

            Sse::new(combined)
                .keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(30)))
                .into_response()
        }
    }
}

// ── Layer 66: Materialized Views ──────────────────────────────────────────────

use vectorvolt_engine::{ViewDefinition, ViewKind};

fn kind_to_str(k: &ViewKind) -> &'static str {
    match k {
        ViewKind::Count => "count",
        ViewKind::Sum { .. } => "sum",
        ViewKind::Min { .. } => "min",
        ViewKind::Max { .. } => "max",
    }
}

pub async fn create_view(
    State(st): State<Arc<AppState>>,
    Json(body): Json<CreateViewBody>,
) -> impl IntoResponse {
    let kind = match body.kind.as_str() {
        "count" => ViewKind::Count,
        "sum" => match body.agg_col {
            Some(c) => ViewKind::Sum { agg_col: c },
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "agg_col required for sum"})),
                )
                    .into_response();
            }
        },
        "min" => match body.agg_col {
            Some(c) => ViewKind::Min { agg_col: c },
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "agg_col required for min"})),
                )
                    .into_response();
            }
        },
        "max" => match body.agg_col {
            Some(c) => ViewKind::Max { agg_col: c },
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "agg_col required for max"})),
                )
                    .into_response();
            }
        },
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("unknown kind '{other}'")})),
            )
                .into_response();
        }
    };
    let def = ViewDefinition {
        name: body.name,
        table: body.table,
        filter_col: body.filter_col,
        lo: body.lo,
        hi: body.hi,
        group_col: body.group_col,
        kind,
    };
    let backfill = body.backfill.unwrap_or(false);
    match st.views.create(def, &st.registry, backfill) {
        Ok(()) => (StatusCode::CREATED, Json(OkResp { ok: true })).into_response(),
        Err(e) => {
            let status = if e.contains("already exists") {
                StatusCode::CONFLICT
            } else if e.contains("not found") || e.contains("NotFound") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(serde_json::json!({"error": e}))).into_response()
        }
    }
}

pub async fn list_views(State(st): State<Arc<AppState>>) -> impl IntoResponse {
    let defs = st.views.list();
    let views: Vec<ViewInfoResp> = defs
        .into_iter()
        .map(|d| ViewInfoResp {
            name: d.name,
            table: d.table,
            kind: kind_to_str(&d.kind).to_string(),
            filter_col: d.filter_col,
            lo: d.lo,
            hi: d.hi,
            group_col: d.group_col,
        })
        .collect();
    Json(ViewsListResp { views })
}

pub async fn get_view(
    State(st): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match st.views.get_entries(&name) {
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("view '{name}' not found")})),
        )
            .into_response(),
        Some(entries) => {
            let entries: Vec<ViewEntryResp> = entries
                .into_iter()
                .map(|e| ViewEntryResp {
                    group_val: e.group_val,
                    agg_val: e.agg_val,
                })
                .collect();
            Json(ViewStateResp { name, entries }).into_response()
        }
    }
}

pub async fn drop_view(
    State(st): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if st.views.drop_view(&name) {
        Json(OkResp { ok: true }).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("view '{name}' not found")})),
        )
            .into_response()
    }
}

// ── Layer 67: CDC Event Log ───────────────────────────────────────────────────

pub async fn cdc_log_handler(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(query): Query<CdcLogQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => {
            let from_lsn = query.from.unwrap_or(0);
            match eng.read_log_from(from_lsn) {
                Err(e) => engine_err(&st, e).into_response(),
                Ok(events) => {
                    let count = events.len();
                    let serialized: Vec<serde_json::Value> = events
                        .into_iter()
                        .filter_map(|e| serde_json::to_value(e).ok())
                        .collect();
                    Json(CdcLogResp {
                        events: serialized,
                        count,
                    })
                    .into_response()
                }
            }
        }
    }
}

// ── Layer 68: CDC Log Stats + Compaction ─────────────────────────────────────

pub async fn cdc_stats_handler(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.cdc_log_stats() {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(s) => Json(CdcStatsResp {
                event_count: s.event_count,
                byte_size: s.byte_size,
                min_lsn: s.min_lsn,
                max_lsn: s.max_lsn,
            })
            .into_response(),
        },
    }
}

pub async fn cdc_compact_handler(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Query(query): Query<CdcCompactQuery>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.compact_cdc_log(query.retain_lsn) {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(removed) => Json(CdcCompactResp { removed }).into_response(),
        },
    }
}

// ── Layer 69: WAL Snapshot ────────────────────────────────────────────────────

/// `POST /tables/{table}/snapshot` — flush, compact, and write snapshot marker.
pub async fn take_snapshot_handler(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.take_snapshot() {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(meta) => {
                st.metrics.record_ok();
                Json(SnapshotResp {
                    row_count: meta.row_count,
                    byte_size: meta.byte_size,
                })
                .into_response()
            }
        },
    }
}

/// `GET /tables/{table}/snapshot` — return snapshot metadata if one exists.
pub async fn snapshot_info_handler(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => match eng.snapshot_info() {
            Err(e) => engine_err(&st, e).into_response(),
            Ok(None) => Json(SnapshotInfoResp {
                exists: false,
                row_count: 0,
                byte_size: 0,
            })
            .into_response(),
            Ok(Some(meta)) => {
                st.metrics.record_ok();
                Json(SnapshotInfoResp {
                    exists: true,
                    row_count: meta.row_count,
                    byte_size: meta.byte_size,
                })
                .into_response()
            }
        },
    }
}

// ── Layer 70: CDC Webhook Sinks ───────────────────────────────────────────────

/// `POST /tables/{table}/webhooks` — register a new webhook sink.
pub async fn register_webhook_handler(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Json(body): Json<RegisterWebhookBody>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(eng) => {
            let rx = eng.subscribe_changes();
            let id = st.webhooks.register(table, body.url, rx);
            st.metrics.record_ok();
            Json(RegisterWebhookResp { id }).into_response()
        }
    }
}

/// `GET /tables/{table}/webhooks` — list all sinks for a table.
pub async fn list_webhooks_handler(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(_) => {
            let webhooks = st
                .webhooks
                .list_for_table(&table)
                .into_iter()
                .map(|e| WebhookEntryResp {
                    id: e.id,
                    table: e.table,
                    url: e.url,
                })
                .collect();
            Json(WebhooksListResp { webhooks }).into_response()
        }
    }
}

/// `DELETE /tables/{table}/webhooks/{id}` — remove a sink.
pub async fn delete_webhook_handler(
    State(st): State<Arc<AppState>>,
    Path((table, id)): Path<(String, u64)>,
) -> impl IntoResponse {
    match st.registry.get(&table) {
        Err(e) => engine_err(&st, e).into_response(),
        Ok(_) => {
            if !st.webhooks.belongs_to(id, &table) {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResp {
                        error: format!("webhook {id} not found for table '{table}'"),
                    }),
                )
                    .into_response();
            }
            st.webhooks.remove(id);
            Json(OkResp { ok: true }).into_response()
        }
    }
}
