//! Gap-register batch B regression tests.

use super::*;
use vectorvolt_engine::EngineOptions;

fn setup() -> (SqlExecutor, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let registry = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
    (SqlExecutor::new(registry), dir)
}

fn rows(r: QueryResult) -> Vec<Vec<String>> {
    match r {
        QueryResult::Rows { rows, .. } => rows,
        other => panic!("expected rows, got {other:?}"),
    }
}

fn sorted_rows(ex: &SqlExecutor, sql: &str) -> Vec<Vec<String>> {
    let mut r = rows(ex.execute(sql).unwrap());
    r.sort();
    r
}

fn assert_sqlstate(r: &QueryResult, code: &str) {
    assert!(
        matches!(r, QueryResult::Error(e) if e.starts_with(&format!("[{code}]"))),
        "expected SQLSTATE {code}, got {r:?}"
    );
}

fn values_list(n: u64) -> String {
    (1..=n)
        .map(|i| format!("({i}, {i})"))
        .collect::<Vec<_>>()
        .join(", ")
}

// ── B5 — CHECK ────────────────────────────────────────────────────────

#[test]
fn b5_compound_check_enforced_on_insert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ck (pk U64, score U64 CHECK (score >= 10 AND score <= 100))")
        .unwrap();
    ex.execute("INSERT INTO ck (pk, score) VALUES (1, 50)")
        .unwrap();
    let r = ex
        .execute("INSERT INTO ck (pk, score) VALUES (2, 500)")
        .unwrap();
    assert_sqlstate(&r, "23514");
    assert!(matches!(&r, QueryResult::Error(e) if e.contains("score")));
    let r = ex
        .execute("INSERT INTO ck (pk, score) VALUES (3, 5)")
        .unwrap();
    assert_sqlstate(&r, "23514");
    assert_eq!(sorted_rows(&ex, "SELECT pk FROM ck"), vec![vec!["1"]]);
}

#[test]
fn b5_check_enforced_on_update_and_upsert() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE cu2 (pk U64, score U64 CHECK (score >= 10))")
        .unwrap();
    ex.execute("INSERT INTO cu2 (pk, score) VALUES (1, 50)")
        .unwrap();
    let r = ex.execute("UPDATE cu2 SET score = 1 WHERE pk = 1").unwrap();
    assert_sqlstate(&r, "23514");
    let r = ex
        .execute("INSERT INTO cu2 (pk, score) VALUES (1, 60) ON CONFLICT DO UPDATE SET score = 2")
        .unwrap();
    assert_sqlstate(&r, "23514");
    assert_eq!(
        sorted_rows(&ex, "SELECT score FROM cu2 WHERE pk = 1"),
        vec![vec!["50"]]
    );
    ex.execute("UPDATE cu2 SET score = 70 WHERE pk = 1")
        .unwrap();
    assert_eq!(
        sorted_rows(&ex, "SELECT score FROM cu2 WHERE pk = 1"),
        vec![vec!["70"]]
    );
}

#[test]
fn b5_unsupported_check_rejected_at_create() {
    let (ex, _dir) = setup();
    let r = ex
        .execute("CREATE TABLE cx (pk U64, v U64 CHECK (v IN (SELECT pk FROM cx)))")
        .unwrap();
    assert_sqlstate(&r, "0A000");
    let r = ex
        .execute("CREATE TABLE cy (pk U64, v U64 CHECK (nosuch > 0))")
        .unwrap();
    assert_sqlstate(&r, "0A000");
    assert!(
        ex.execute("SELECT * FROM cx").is_err(),
        "table must not exist"
    );
}

// ── B6 — FOREIGN KEY ──────────────────────────────────────────────────

#[test]
fn b6_parent_delete_restricted_while_referenced() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE fp (pk U64)").unwrap();
    ex.execute("CREATE TABLE fc (pk U64, pid U64, FOREIGN KEY (pid) REFERENCES fp (pk))")
        .unwrap();
    ex.execute("INSERT INTO fp (pk) VALUES (1)").unwrap();
    ex.execute("INSERT INTO fp (pk) VALUES (2)").unwrap();
    ex.execute("INSERT INTO fc (pk, pid) VALUES (10, 1)")
        .unwrap();
    let r = ex.execute("DELETE FROM fp WHERE pk = 1").unwrap();
    assert_sqlstate(&r, "23503");
    assert_eq!(sorted_rows(&ex, "SELECT pk FROM fp").len(), 2);
    // Unreferenced parent rows can go; once the child is gone so can 1.
    ex.execute("DELETE FROM fp WHERE pk = 2").unwrap();
    ex.execute("DELETE FROM fc WHERE pk = 10").unwrap();
    let r = ex.execute("DELETE FROM fp WHERE pk = 1").unwrap();
    assert!(matches!(r, QueryResult::Affected(1)), "{r:?}");
}

#[test]
fn b6_child_update_and_parent_key_update_checked() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE pp (pk U64, code U64)").unwrap();
    ex.execute(
        "CREATE TABLE cc (pk U64, code_ref U64, FOREIGN KEY (code_ref) REFERENCES pp (code))",
    )
    .unwrap();
    ex.execute("INSERT INTO pp (pk, code) VALUES (1, 7)")
        .unwrap();
    ex.execute("INSERT INTO cc (pk, code_ref) VALUES (1, 7)")
        .unwrap();
    let r = ex
        .execute("UPDATE cc SET code_ref = 999 WHERE pk = 1")
        .unwrap();
    assert_sqlstate(&r, "23503");
    let r = ex.execute("UPDATE pp SET code = 8 WHERE pk = 1").unwrap();
    assert_sqlstate(&r, "23503");
    assert_eq!(sorted_rows(&ex, "SELECT code FROM pp"), vec![vec!["7"]]);
}

#[test]
fn b6_parent_staged_in_same_txn_is_visible() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE tp (pk U64)").unwrap();
    ex.execute("CREATE TABLE tc (pk U64, pid U64, FOREIGN KEY (pid) REFERENCES tp (pk))")
        .unwrap();
    ex.execute("BEGIN").unwrap();
    ex.execute("INSERT INTO tp (pk) VALUES (5)").unwrap();
    let r = ex
        .execute("INSERT INTO tc (pk, pid) VALUES (1, 5)")
        .unwrap();
    assert!(matches!(r, QueryResult::Affected(1)), "{r:?}");
    ex.execute("COMMIT").unwrap();
    assert_eq!(sorted_rows(&ex, "SELECT pid FROM tc"), vec![vec!["5"]]);
}

// ── B7 — triggers ─────────────────────────────────────────────────────

#[test]
fn b7_row_triggers_fire_with_new_and_old() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE tt (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE tlog (pk U64, v U64)").unwrap();
    ex.execute(
        "CREATE PROCEDURE log_ins AS BEGIN INSERT INTO tlog (pk, v) VALUES (NEW.pk, NEW.v); END",
    )
    .unwrap();
    ex.execute(
        "CREATE PROCEDURE log_upd AS BEGIN INSERT INTO tlog (pk, v) VALUES (100, OLD.v); END",
    )
    .unwrap();
    ex.execute("CREATE PROCEDURE log_del AS BEGIN DELETE FROM tlog WHERE pk = OLD.pk; END")
        .unwrap();
    for sql in [
        "CREATE TRIGGER a_ins AFTER INSERT ON tt FOR EACH ROW EXECUTE FUNCTION log_ins()",
        "CREATE TRIGGER b_upd BEFORE UPDATE ON tt FOR EACH ROW EXECUTE FUNCTION log_upd()",
        "CREATE TRIGGER c_del AFTER DELETE ON tt FOR EACH ROW EXECUTE FUNCTION log_del()",
    ] {
        let r = ex.execute(sql).unwrap();
        assert!(matches!(r, QueryResult::Created), "{sql}: {r:?}");
    }
    ex.execute("INSERT INTO tt (pk, v) VALUES (7, 70)").unwrap();
    assert_eq!(
        sorted_rows(&ex, "SELECT * FROM tlog"),
        vec![vec!["7", "70"]]
    );
    ex.execute("UPDATE tt SET v = 71 WHERE pk = 7").unwrap();
    assert_eq!(
        sorted_rows(&ex, "SELECT * FROM tlog"),
        vec![vec!["100", "70"], vec!["7", "70"]]
    );
    ex.execute("DELETE FROM tt WHERE pk = 7").unwrap();
    assert_eq!(
        sorted_rows(&ex, "SELECT * FROM tlog"),
        vec![vec!["100", "70"]]
    );
}

#[test]
fn b7_failing_before_trigger_aborts_statement() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ta1 (pk U64)").unwrap();
    ex.execute("CREATE TABLE guard (pk U64, v U64 CHECK (v > 0))")
        .unwrap();
    ex.execute("CREATE PROCEDURE deny AS BEGIN INSERT INTO guard (pk, v) VALUES (NEW.pk, 0); END")
        .unwrap();
    ex.execute("CREATE TRIGGER deny_t BEFORE INSERT ON ta1 FOR EACH ROW EXECUTE FUNCTION deny()")
        .unwrap();
    let r = ex.execute("INSERT INTO ta1 (pk) VALUES (1)").unwrap();
    assert_sqlstate(&r, "23514");
    assert!(sorted_rows(&ex, "SELECT * FROM ta1").is_empty());
}

#[test]
fn b7_recursive_trigger_hits_depth_limit() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE rec (pk U64, v U64)").unwrap();
    ex.execute("INSERT INTO rec (pk, v) VALUES (1, 0)").unwrap();
    // An AFTER UPDATE trigger that updates its own row recurses forever.
    ex.execute("CREATE PROCEDURE again AS BEGIN UPDATE rec SET v = 1 WHERE pk = NEW.pk; END")
        .unwrap();
    ex.execute("CREATE TRIGGER again_t AFTER UPDATE ON rec FOR EACH ROW EXECUTE FUNCTION again()")
        .unwrap();
    let r = ex.execute("UPDATE rec SET v = 2 WHERE pk = 1").unwrap();
    assert_sqlstate(&r, "54001");
}

#[test]
fn b7_unsupported_trigger_function_rejected() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE tu (pk U64)").unwrap();
    ex.execute("CREATE FUNCTION ident(r) RETURNS VOID AS 'r'")
        .unwrap();
    let r = ex
        .execute("CREATE TRIGGER tu_t AFTER INSERT ON tu FOR EACH ROW EXECUTE FUNCTION ident()")
        .unwrap();
    assert_sqlstate(&r, "0A000");
    let r = ex
        .execute("CREATE TRIGGER tu_m AFTER INSERT ON tu FOR EACH ROW EXECUTE FUNCTION nope()")
        .unwrap();
    assert_sqlstate(&r, "0A000");
    assert!(ex.triggers.lock_safe().is_empty());
}

// ── B8 — statement timeout ────────────────────────────────────────────

#[test]
fn b8_timeout_cancels_long_join_mid_execution() {
    let (ex, _dir) = setup();
    for t in ["ja", "jb", "jc"] {
        ex.execute(&format!("CREATE TABLE {t} (pk U64, v U64)"))
            .unwrap();
        ex.execute(&format!(
            "INSERT INTO {t} (pk, v) VALUES {}",
            values_list(80)
        ))
        .unwrap();
    }
    let sql = "SELECT * FROM ja CROSS JOIN jb CROSS JOIN jc";
    let t0 = std::time::Instant::now();
    let full = ex.execute(sql).unwrap();
    let baseline = t0.elapsed();
    assert_eq!(rows(full).len(), 80 * 80 * 80);

    ex.execute("SET STATEMENT_TIMEOUT 10").unwrap();
    let t0 = std::time::Instant::now();
    let r = ex.execute(sql).unwrap();
    let cancelled_after = t0.elapsed();
    assert_sqlstate(&r, "57014");
    assert!(matches!(&r, QueryResult::Error(e) if e.contains("statement timeout")));
    assert!(
        cancelled_after < baseline / 2,
        "cancelled after {cancelled_after:?}, full run took {baseline:?}"
    );
    // The session is usable afterwards.
    ex.execute("SET STATEMENT_TIMEOUT 999999").unwrap();
    assert_eq!(rows(ex.execute("SELECT * FROM ja").unwrap()).len(), 80);
}

// ── B9 — ROLLBACK TO SAVEPOINT ────────────────────────────────────────

#[test]
fn b9_rollback_to_reverts_later_changes_to_earlier_tables() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE sp_t (pk U64, v U64)").unwrap();
    ex.execute("CREATE TABLE sp_u (pk U64, v U64)").unwrap();
    ex.execute("BEGIN").unwrap();
    ex.execute("INSERT INTO sp_t (pk, v) VALUES (1, 10)")
        .unwrap();
    ex.execute("SAVEPOINT s").unwrap();
    ex.execute("INSERT INTO sp_t (pk, v) VALUES (1, 99)")
        .unwrap();
    ex.execute("INSERT INTO sp_t (pk, v) VALUES (2, 20)")
        .unwrap();
    ex.execute("INSERT INTO sp_u (pk, v) VALUES (3, 30)")
        .unwrap();
    let r = ex.execute("ROLLBACK TO SAVEPOINT s").unwrap();
    assert!(matches!(r, QueryResult::TxRollback), "{r:?}");
    ex.execute("INSERT INTO sp_u (pk, v) VALUES (4, 40)")
        .unwrap();
    ex.execute("COMMIT").unwrap();
    assert_eq!(
        sorted_rows(&ex, "SELECT * FROM sp_t"),
        vec![vec!["1", "10"]]
    );
    assert_eq!(
        sorted_rows(&ex, "SELECT * FROM sp_u"),
        vec![vec!["4", "40"]]
    );
}

// ── B10 — WHERE over views / CTEs / materialized views ───────────────

#[test]
fn b10_view_predicates_filter_or_error() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE vt (pk U64, v U64)").unwrap();
    ex.execute(&format!("INSERT INTO vt (pk, v) VALUES {}", values_list(5)))
        .unwrap();
    ex.execute("CREATE VIEW vw AS SELECT pk, v FROM vt")
        .unwrap();
    assert_eq!(
        sorted_rows(&ex, "SELECT pk FROM vw WHERE pk IN (2, 4)"),
        vec![vec!["2"], vec!["4"]]
    );
    assert_eq!(
        sorted_rows(&ex, "SELECT pk FROM vw WHERE v BETWEEN 2 AND 3"),
        vec![vec!["2"], vec!["3"]]
    );
    assert_eq!(
        sorted_rows(&ex, "SELECT pk FROM vw WHERE pk = 1 OR v > 4"),
        vec![vec!["1"], vec!["5"]]
    );
    let r = ex
        .execute("SELECT pk FROM vw WHERE pk IN (SELECT pk FROM vt WHERE v = 1)")
        .unwrap();
    assert_sqlstate(&r, "0A000");
}

// ── B11 — autocommit writes take SQL_COMMIT_LOCK ─────────────────────

#[test]
fn b11_autocommit_write_waits_for_commit_lock() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE ac (pk U64, v U64)").unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        // Simulate a SERIALIZABLE commit in progress.
        let held = SQL_COMMIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let ex = &ex;
        scope.spawn(move || {
            let r = ex.execute("INSERT INTO ac (pk, v) VALUES (1, 1)");
            tx.send(r.is_ok()).unwrap();
        });
        assert!(
            rx.recv_timeout(std::time::Duration::from_millis(300))
                .is_err(),
            "autocommit write must not apply while a commit holds the lock"
        );
        drop(held);
        assert!(rx.recv_timeout(std::time::Duration::from_secs(30)).unwrap());
    });
    // Reads never take the lock.
    let held = SQL_COMMIT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(rows(ex.execute("SELECT * FROM ac").unwrap()).len(), 1);
    drop(held);
}

// ── B16 — poison-tolerant locks ──────────────────────────────────────

#[test]
fn b16_poisoned_mutex_does_not_wedge_executor() {
    let (ex, _dir) = setup();
    ex.execute("CREATE TABLE pz (pk U64)").unwrap();
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _g = ex.views.lock().unwrap();
        panic!("poison the views mutex");
    }));
    assert!(ex.views.is_poisoned());
    ex.execute("CREATE VIEW pzv AS SELECT pk FROM pz").unwrap();
    ex.execute("INSERT INTO pz (pk) VALUES (1)").unwrap();
    assert_eq!(rows(ex.execute("SELECT * FROM pzv").unwrap()).len(), 1);
}
