//! `vvctl sql -e` executes real SQL against a table directory and reports
//! failures through the exit status.

use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vvctl::commands::sql::{SqlArgs, run};

fn exec(d: &TempDir, sql: &str) -> anyhow::Result<()> {
    run(SqlArgs {
        dir: d.path().into(),
        exec: Some(sql.into()),
        output: "csv".into(),
    })
}

#[test]
fn sql_exec_creates_inserts_and_persists() {
    let d = TempDir::new().unwrap();
    exec(&d, "CREATE TABLE t (id BIGINT, v BIGINT)").unwrap();
    exec(&d, "INSERT INTO t (id, v) VALUES (1, 10)").unwrap();
    exec(&d, "SELECT * FROM t WHERE id = 1").unwrap();
    let r = TableRegistry::open(d.path(), EngineOptions::default()).unwrap();
    assert!(r.contains("t"));
    assert!(r.get("t").unwrap().get(1).unwrap().is_some());
}

#[test]
fn sql_exec_failure_is_nonzero() {
    let d = TempDir::new().unwrap();
    assert!(exec(&d, "SELECT * FROM missing_table").is_err());
    assert!(exec(&d, "THIS IS NOT SQL").is_err());
}
