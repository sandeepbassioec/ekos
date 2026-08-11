use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use vectorvolt_engine::TableRegistry;

use crate::types::{
    CreateTableBody, ErrorResp, InsertBody, OkResp, RowResp, RowsResp, ScanQuery, SchemaResp,
    SecondaryQuery, TablesResp,
};

pub type Reg = Arc<TableRegistry>;

fn engine_err(e: impl std::fmt::Display) -> (StatusCode, Json<ErrorResp>) {
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
    State(reg): State<Reg>,
    Path((table, pk)): Path<(String, u64)>,
    Json(body): Json<InsertBody>,
) -> impl IntoResponse {
    match reg.get(&table) {
        Err(e) => engine_err(e).into_response(),
        Ok(eng) => match eng.insert(pk, body.values.clone()) {
            Ok(_) => Json(RowResp {
                pk,
                values: body.values,
            })
            .into_response(),
            Err(e) => engine_err(e).into_response(),
        },
    }
}

// ── row get ───────────────────────────────────────────────────────────────────

pub async fn get_row(
    State(reg): State<Reg>,
    Path((table, pk)): Path<(String, u64)>,
) -> impl IntoResponse {
    match reg.get(&table) {
        Err(e) => engine_err(e).into_response(),
        Ok(eng) => match eng.get(pk) {
            Err(e) => engine_err(e).into_response(),
            Ok(None) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResp {
                    error: format!("pk={pk} not found"),
                }),
            )
                .into_response(),
            Ok(Some(row)) => Json(RowResp {
                pk: row.pk,
                values: row.values,
            })
            .into_response(),
        },
    }
}

// ── row delete ────────────────────────────────────────────────────────────────

pub async fn delete_row(
    State(reg): State<Reg>,
    Path((table, pk)): Path<(String, u64)>,
) -> impl IntoResponse {
    match reg.get(&table) {
        Err(e) => engine_err(e).into_response(),
        Ok(eng) => match eng.delete(pk) {
            Ok(_) => Json(OkResp { ok: true }).into_response(),
            Err(e) => engine_err(e).into_response(),
        },
    }
}

// ── scan ──────────────────────────────────────────────────────────────────────

pub async fn scan_rows(
    State(reg): State<Reg>,
    Path(table): Path<String>,
    Query(q): Query<ScanQuery>,
) -> impl IntoResponse {
    match reg.get(&table) {
        Err(e) => engine_err(e).into_response(),
        Ok(eng) => match eng.scan(q.col as usize, q.lo, q.hi) {
            Err(e) => engine_err(e).into_response(),
            Ok(rows) => {
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
    State(reg): State<Reg>,
    Path(table): Path<String>,
    Query(q): Query<SecondaryQuery>,
) -> impl IntoResponse {
    match reg.get(&table) {
        Err(e) => engine_err(e).into_response(),
        Ok(eng) => match eng.secondary_scan(q.col, q.val) {
            Err(e) => engine_err(e).into_response(),
            Ok(rows) => {
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

pub async fn list_tables(State(reg): State<Reg>) -> impl IntoResponse {
    Json(TablesResp {
        tables: reg.list_tables(),
    })
}

// ── describe table ────────────────────────────────────────────────────────────

pub async fn describe_table(
    State(reg): State<Reg>,
    Path(table): Path<String>,
) -> impl IntoResponse {
    match reg.get(&table) {
        Err(e) => engine_err(e).into_response(),
        Ok(eng) => Json(SchemaResp {
            columns: eng.schema.clone(),
        })
        .into_response(),
    }
}

// ── create table ──────────────────────────────────────────────────────────────

pub async fn create_table(
    State(reg): State<Reg>,
    Path(table): Path<String>,
    Json(body): Json<CreateTableBody>,
) -> impl IntoResponse {
    match reg.create_table(&table, body.columns) {
        Ok(_) => (StatusCode::CREATED, Json(OkResp { ok: true })).into_response(),
        Err(e) => engine_err(e).into_response(),
    }
}

// ── drop table ────────────────────────────────────────────────────────────────

pub async fn drop_table(State(reg): State<Reg>, Path(table): Path<String>) -> impl IntoResponse {
    match reg.drop_table(&table) {
        Ok(_) => Json(OkResp { ok: true }).into_response(),
        Err(e) => engine_err(e).into_response(),
    }
}
