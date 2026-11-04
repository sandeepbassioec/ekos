//! D5 — masking, grants, freeze, schema lock, idle timeout and memory limit
//! are enforced by the SQL executor (spec refs S159, S252, S273, S278, S283,
//! S295). Each section proves the setting changes behaviour and that the
//! default (never configured) behaviour is unchanged.

use std::sync::Arc;
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_query::sql::executor::{QueryResult, SqlExecutor};

fn setup() -> (SqlExecutor, TempDir) {
    let dir = TempDir::new().unwrap();
    let registry = TableRegistry::open(dir.path(), EngineOptions::default()).unwrap();
    (SqlExecutor::new(Arc::new(registry)), dir)
}

fn rows_of(r: QueryResult) -> (Vec<String>, Vec<Vec<String>>) {
    match r {
        QueryResult::Rows { columns, rows } => (columns, rows),
        QueryResult::WithNotice { result, .. } => rows_of(*result),
        other => panic!("expected rows, got {other:?}"),
    }
}

fn q(ex: &SqlExecutor, sql: &str) -> QueryResult {
    ex.execute(sql).unwrap_or_else(|e| panic!("{sql}: {e}"))
}

fn is_err(r: &QueryResult, code: &str) -> bool {
    matches!(r, QueryResult::Error(m) if m.starts_with(&format!("[{code}]")))
}

fn users(ex: &SqlExecutor) {
    q(ex, "CREATE TABLE users (pk U64, salary U64, age U64)");
    q(
        ex,
        "INSERT INTO users (pk, salary, age) VALUES (1, 5000, 30)",
    );
    q(
        ex,
        "INSERT INTO users (pk, salary, age) VALUES (2, 7000, 40)",
    );
}

// ── S159 — masking ──────────────────────────────────────────────────────────

fn mask_salary(ex: &SqlExecutor) {
    q(ex, "CREATE MASK POLICY sal_mask MASK '****'");
    q(ex, "APPLY MASK POLICY sal_mask TO users salary");
}

#[test]
fn s159_mask_applies_to_star_and_named_columns() {
    let (ex, _d) = setup();
    users(&ex);
    mask_salary(&ex);
    let (cols, rows) = rows_of(q(&ex, "SELECT * FROM users"));
    let si = cols.iter().position(|c| c == "salary").unwrap();
    let ai = cols.iter().position(|c| c == "age").unwrap();
    assert!(rows.iter().all(|r| r[si] == "****"));
    assert!(rows.iter().any(|r| r[ai] == "30"), "unmasked column intact");
    let (_, rows) = rows_of(q(&ex, "SELECT salary FROM users WHERE pk = 1"));
    assert_eq!(rows, vec![vec!["****".to_owned()]]);
}

#[test]
fn s159_mask_applies_to_aggregates_and_expressions() {
    let (ex, _d) = setup();
    users(&ex);
    mask_salary(&ex);
    let (_, rows) = rows_of(q(&ex, "SELECT MAX(salary) FROM users"));
    assert_eq!(rows[0][0], "****");
    // COUNT does not expose the value
    let (_, rows) = rows_of(q(&ex, "SELECT COUNT(*) FROM users"));
    assert_eq!(rows[0][0], "2");
    let (_, rows) = rows_of(q(&ex, "SELECT salary + 1 AS s2 FROM users WHERE pk = 1"));
    assert_eq!(rows[0][0], "****");
}

#[test]
fn s159_mask_applies_through_join() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "CREATE TABLE orders (pk U64, user_id U64)");
    q(&ex, "INSERT INTO orders (pk, user_id) VALUES (10, 1)");
    mask_salary(&ex);
    let (cols, rows) = rows_of(q(
        &ex,
        "SELECT * FROM orders INNER JOIN users ON user_id = pk",
    ));
    let si = cols.iter().position(|c| c.ends_with("salary")).unwrap();
    assert_eq!(rows[0][si], "****");
}

#[test]
fn s159_mask_applies_to_every_session_and_can_be_removed() {
    let (ex, _d) = setup();
    users(&ex);
    mask_salary(&ex);
    let other = ex.new_session();
    let (_, rows) = rows_of(q(&other, "SELECT salary FROM users WHERE pk = 2"));
    assert_eq!(rows[0][0], "****");
    q(&ex, "REMOVE MASK POLICY FROM users salary");
    let (_, rows) = rows_of(q(&other, "SELECT salary FROM users WHERE pk = 2"));
    assert_eq!(rows[0][0], "7000");
}

#[test]
fn s159_default_no_mask_returns_real_values() {
    let (ex, _d) = setup();
    users(&ex);
    // a policy that was created but never applied masks nothing
    q(&ex, "CREATE MASK POLICY sal_mask MASK '****'");
    let (_, rows) = rows_of(q(&ex, "SELECT salary FROM users WHERE pk = 1"));
    assert_eq!(rows[0][0], "5000");
}

// ── S252 — grants ───────────────────────────────────────────────────────────

#[test]
fn s252_no_grants_means_open_access() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "INSERT INTO users (pk, salary, age) VALUES (3, 1, 1)");
    rows_of(q(&ex, "SELECT * FROM users"));
    q(&ex, "SET ROLE anyone");
    rows_of(q(&ex, "SELECT * FROM users"));
}

#[test]
fn s252_select_grant_enforced() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "GRANT SELECT ON users TO ROLE analyst");
    // no role set → denied
    assert!(is_err(&q(&ex, "SELECT * FROM users"), "42501"));
    q(&ex, "SET ROLE intern");
    assert!(is_err(&q(&ex, "SELECT * FROM users"), "42501"));
    q(&ex, "SET ROLE analyst");
    rows_of(q(&ex, "SELECT * FROM users"));
    // SELECT does not imply write privileges
    assert!(is_err(
        &q(&ex, "INSERT INTO users (pk, salary, age) VALUES (9, 1, 1)"),
        "42501"
    ));
    assert!(is_err(
        &q(&ex, "UPDATE users SET age = 1 WHERE pk = 1"),
        "42501"
    ));
    assert!(is_err(&q(&ex, "DELETE FROM users WHERE pk = 1"), "42501"));
}

#[test]
fn s252_write_grants_and_all() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "GRANT INSERT, UPDATE ON users TO ROLE writer");
    q(&ex, "GRANT ALL PRIVILEGES ON users TO ROLE admin");
    q(&ex, "SET ROLE writer");
    assert!(matches!(
        q(&ex, "INSERT INTO users (pk, salary, age) VALUES (9, 1, 1)"),
        QueryResult::Affected(_)
    ));
    assert!(matches!(
        q(&ex, "UPDATE users SET age = 2 WHERE pk = 9"),
        QueryResult::Affected(_)
    ));
    assert!(is_err(&q(&ex, "DELETE FROM users WHERE pk = 9"), "42501"));
    assert!(is_err(&q(&ex, "SELECT * FROM users"), "42501"));
    q(&ex, "SET ROLE admin");
    assert!(matches!(
        q(&ex, "DELETE FROM users WHERE pk = 9"),
        QueryResult::Affected(_)
    ));
    rows_of(q(&ex, "SELECT * FROM users"));
}

#[test]
fn s252_join_needs_select_on_joined_table() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "CREATE TABLE orders (pk U64, user_id U64)");
    q(&ex, "GRANT SELECT ON users TO ROLE r1");
    q(&ex, "SET ROLE r1");
    // orders is ungoverned, users is governed and granted
    rows_of(q(
        &ex,
        "SELECT * FROM orders INNER JOIN users ON user_id = pk",
    ));
    q(&ex, "REVOKE SELECT ON users FROM ROLE r1");
    assert!(is_err(
        &q(&ex, "SELECT * FROM orders INNER JOIN users ON user_id = pk"),
        "42501"
    ));
}

#[test]
fn s252_revoking_last_grant_keeps_table_governed() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "GRANT SELECT ON users TO ROLE analyst");
    q(&ex, "REVOKE SELECT ON users FROM ROLE analyst");
    q(&ex, "SET ROLE analyst");
    assert!(is_err(&q(&ex, "SELECT * FROM users"), "42501"));
}

#[test]
fn s252_grants_are_server_wide_roles_per_session() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "GRANT SELECT ON users TO ROLE analyst");
    let s2 = ex.new_session();
    assert!(is_err(&q(&s2, "SELECT * FROM users"), "42501"));
    s2.set_session_role(Some("analyst"));
    rows_of(q(&s2, "SELECT * FROM users"));
    // the first session's role is untouched
    assert_eq!(ex.session_role(), None);
}

#[test]
fn s252_authenticated_role_cannot_be_changed() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "GRANT SELECT ON users TO ROLE admin");
    let s = ex.new_session();
    s.set_authenticated_role("intern");
    assert!(is_err(&q(&s, "SET ROLE admin"), "42501"));
    assert!(is_err(&q(&s, "SELECT * FROM users"), "42501"));
    let (_, rows) = rows_of(q(&s, "SHOW ROLE"));
    assert_eq!(rows[0][0], "intern");
}

// ── S273 — freeze ───────────────────────────────────────────────────────────

#[test]
fn s273_frozen_table_rejects_writes_but_allows_reads() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "FREEZE TABLE users");
    for sql in [
        "INSERT INTO users (pk, salary, age) VALUES (9, 1, 1)",
        "UPDATE users SET age = 1 WHERE pk = 1",
        "DELETE FROM users WHERE pk = 1",
        "TRUNCATE TABLE users",
        "ALTER TABLE users ADD COLUMN extra U64",
        "DROP TABLE users",
    ] {
        assert!(is_err(&q(&ex, sql), "55000"), "{sql} should be refused");
    }
    let (_, rows) = rows_of(q(&ex, "SELECT * FROM users"));
    assert_eq!(rows.len(), 2, "data unchanged");
    // applies to other sessions too
    let s2 = ex.new_session();
    assert!(is_err(&q(&s2, "DELETE FROM users WHERE pk = 1"), "55000"));
}

#[test]
fn s273_unfreeze_restores_writes() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "FREEZE TABLE users");
    q(&ex, "UNFREEZE TABLE users");
    assert!(matches!(
        q(&ex, "INSERT INTO users (pk, salary, age) VALUES (9, 1, 1)"),
        QueryResult::Affected(_)
    ));
}

#[test]
fn s273_default_unfrozen_tables_writable() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "CREATE TABLE other (pk U64)");
    q(&ex, "FREEZE TABLE other");
    assert!(matches!(
        q(&ex, "DELETE FROM users WHERE pk = 1"),
        QueryResult::Affected(_)
    ));
}

// ── S278 — schema lock ──────────────────────────────────────────────────────

#[test]
fn s278_locked_schema_rejects_ddl() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "LOCK SCHEMA public");
    for sql in [
        "CREATE TABLE t2 (pk U64)",
        "ALTER TABLE users ADD COLUMN extra U64",
        "CREATE INDEX idx_age ON users(age)",
        "DROP TABLE users",
    ] {
        assert!(is_err(&q(&ex, sql), "55000"), "{sql} should be refused");
    }
    // DML is still allowed
    assert!(matches!(
        q(&ex, "INSERT INTO users (pk, salary, age) VALUES (9, 1, 1)"),
        QueryResult::Affected(_)
    ));
    q(&ex, "UNLOCK SCHEMA public");
    assert!(matches!(
        q(&ex, "CREATE TABLE t2 (pk U64)"),
        QueryResult::Created
    ));
}

#[test]
fn s278_drop_index_refused_while_locked() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "CREATE INDEX idx_age ON users(age)");
    q(&ex, "LOCK SCHEMA users");
    assert!(is_err(&q(&ex, "DROP INDEX idx_age"), "55000"));
}

#[test]
fn s278_default_and_other_schema_unaffected() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "LOCK SCHEMA analytics");
    assert!(matches!(
        q(&ex, "CREATE TABLE t2 (pk U64)"),
        QueryResult::Created
    ));
    assert!(matches!(q(&ex, "DROP TABLE t2"), QueryResult::Dropped));
}

// ── S283 — idle timeout ─────────────────────────────────────────────────────

#[test]
fn s283_idle_session_refused_and_txn_rolled_back() {
    let (ex, _d) = setup();
    users(&ex);
    let s = ex.new_session();
    q(&s, "SET CONNECTION IDLE TIMEOUT 100ms");
    q(&s, "BEGIN");
    q(&s, "INSERT INTO users (pk, salary, age) VALUES (9, 1, 1)");
    std::thread::sleep(std::time::Duration::from_millis(250));
    let r = q(&s, "SELECT * FROM users");
    assert!(is_err(&r, "57P05"), "got {r:?}");
    // the open transaction was rolled back
    let (_, rows) = rows_of(q(&s, "SELECT * FROM users"));
    assert_eq!(rows.len(), 2);
}

#[test]
fn s283_active_session_not_affected() {
    let (ex, _d) = setup();
    users(&ex);
    q(&ex, "SET CONNECTION IDLE TIMEOUT 2s");
    for _ in 0..5 {
        rows_of(q(&ex, "SELECT * FROM users"));
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn s283_default_no_timeout() {
    let (ex, _d) = setup();
    users(&ex);
    std::thread::sleep(std::time::Duration::from_millis(150));
    rows_of(q(&ex, "SELECT * FROM users"));
    q(&ex, "SET CONNECTION IDLE TIMEOUT 100ms");
    q(&ex, "RESET CONNECTION IDLE TIMEOUT");
    std::thread::sleep(std::time::Duration::from_millis(150));
    rows_of(q(&ex, "SELECT * FROM users"));
}

// ── S295 — memory limit ─────────────────────────────────────────────────────

fn many_rows(ex: &SqlExecutor, n: u64) {
    q(ex, "CREATE TABLE big (pk U64, v U64)");
    for i in 0..n {
        q(
            ex,
            &format!("INSERT INTO big (pk, v) VALUES ({i}, {})", i * 7),
        );
    }
}

#[test]
fn s295_query_over_limit_fails() {
    let (ex, _d) = setup();
    many_rows(&ex, 200);
    q(&ex, "SET QUERY MEMORY LIMIT 1KB");
    let r = q(&ex, "SELECT * FROM big");
    assert!(is_err(&r, "53200"), "got {r:?}");
    // a join that multiplies rows is stopped too
    let r = q(&ex, "SELECT * FROM big CROSS JOIN big");
    assert!(is_err(&r, "53200"), "got {r:?}");
}

#[test]
fn s295_query_under_limit_succeeds_and_reset_lifts_limit() {
    let (ex, _d) = setup();
    many_rows(&ex, 200);
    q(&ex, "SET QUERY MEMORY LIMIT 64MB");
    assert_eq!(rows_of(q(&ex, "SELECT * FROM big")).1.len(), 200);
    q(&ex, "SET QUERY MEMORY LIMIT 1KB");
    q(&ex, "RESET QUERY RESOURCE LIMITS");
    assert_eq!(rows_of(q(&ex, "SELECT * FROM big")).1.len(), 200);
}

#[test]
fn s295_invalid_limit_rejected() {
    let (ex, _d) = setup();
    assert!(is_err(&q(&ex, "SET QUERY MEMORY LIMIT lots"), "22023"));
    let (_, rows) = rows_of(q(&ex, "SHOW QUERY RESOURCE LIMITS"));
    assert_eq!(rows[0][0], "NONE");
}

#[test]
fn s295_default_no_limit() {
    let (ex, _d) = setup();
    many_rows(&ex, 200);
    assert_eq!(rows_of(q(&ex, "SELECT * FROM big")).1.len(), 200);
}
