//! Unit tests for the SQL executor, split by sprint range. Shared helpers live here.

use super::*;
use vectorvolt_engine::EngineOptions;

fn setup() -> (SqlExecutor, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let registry = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
    (SqlExecutor::new(registry), dir)
}

/// `setup()` plus a configured column-encryption key (S142/S189).
fn setup_with_key() -> (SqlExecutor, tempfile::TempDir) {
    let (ex, dir) = setup();
    ex.set_column_encryption_key(vectorvolt_crypto::KeyStore::generate().unwrap());
    (ex, dir)
}

/// Two independent `SqlExecutor` "connections" sharing one underlying
/// `TableRegistry`, so writes from one are visible to the other exactly
/// as if they came from a different client.
fn setup_two_conns() -> (SqlExecutor, SqlExecutor, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let registry = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
    let ex1 = SqlExecutor::new(Arc::clone(&registry));
    let ex2 = SqlExecutor::new(registry);
    (ex1, ex2, dir)
}

fn select_first_val(r: QueryResult) -> String {
    match r {
        QueryResult::Rows { rows, .. } => rows[0][0].clone(),
        other => panic!("expected rows, got {other:?}"),
    }
}

fn select_row_count(r: QueryResult) -> usize {
    match r {
        QueryResult::Rows { rows, .. } => rows.len(),
        other => panic!("expected rows, got {other:?}"),
    }
}

fn row_count(r: QueryResult) -> usize {
    match r {
        QueryResult::Rows { rows, .. } => rows.len(),
        QueryResult::WithNotice { result, .. } => row_count(*result),
        other => panic!("expected rows, got {other:?}"),
    }
}

mod part1;
mod part2;
mod part3;
mod part4;
