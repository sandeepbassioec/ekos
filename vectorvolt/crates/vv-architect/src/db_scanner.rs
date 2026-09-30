//! Real database schema introspection — `--scan-db`.
//!
//! Phase scope: SQLite only, via `rusqlite` (bundled, no system SQLite
//! needed) — this is genuinely testable without a live server. Postgres and
//! MySQL (via connection string, per the plan's original spec) are
//! documented follow-ups: they need a running server this environment can't
//! spin up for tests, so implementing them without any way to verify against
//! a real instance would be exactly the kind of unverified guess this
//! project has been actively avoiding.
//!
//! Unlike the code scanner's naming-convention heuristic, this reads REAL
//! foreign key constraints declared in the schema (`PRAGMA foreign_key_list`)
//! — authoritative, not inferred.

use std::collections::BTreeSet;
use std::path::Path;

use rusqlite::Connection;

use crate::scanner::{DetectedEntity, ScanResult};

/// Open a SQLite database file and build a [`ScanResult`] from its real
/// table + foreign-key schema. Table names become entity names; each FK
/// constraint becomes a reference edge, exactly like the code scanner's
/// naming-convention heuristic but sourced from authoritative schema
/// metadata instead of a guess.
pub fn scan_sqlite(path: &Path) -> anyhow::Result<ScanResult> {
    let conn = Connection::open(path)?;
    let mut result = ScanResult::default();

    let mut tables_stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )?;
    let table_names: Vec<String> = tables_stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<_, _>>()?;

    for table in &table_names {
        // Quote the table name: PRAGMA doesn't accept an unquoted reserved
        // word like `order` even though it's a valid (quoted) table name.
        let mut fk_stmt = conn.prepare(&format!("PRAGMA foreign_key_list(\"{table}\")"))?;
        let referenced_tables: Vec<String> = fk_stmt
            .query_map([], |row| row.get::<_, String>(2))? // column 2 = referenced table
            .collect::<Result<_, _>>()?;
        let references: BTreeSet<String> = referenced_tables
            .into_iter()
            .filter(|referenced| referenced != table && table_names.contains(referenced))
            .collect();

        result.entities.insert(
            table.clone(),
            DetectedEntity {
                name: table.clone(),
                references,
            },
        );
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp_db(setup_sql: &str) -> (Connection, tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(setup_sql).unwrap();
        // Close this handle so scan_sqlite can open its own fresh connection.
        drop(conn);
        let conn2 = Connection::open(&path).unwrap();
        (conn2, dir, path)
    }

    #[test]
    fn detects_tables_with_no_fk_as_isolated_entities() {
        let (_conn, _dir, path) = open_temp_db("CREATE TABLE product (id INTEGER PRIMARY KEY);");
        let result = scan_sqlite(&path).unwrap();
        assert_eq!(result.entities.len(), 1);
        assert!(result.entities["product"].references.is_empty());
    }

    #[test]
    fn detects_real_fk_constraint_between_tables() {
        let (_conn, _dir, path) = open_temp_db(
            "CREATE TABLE \"order\" (id INTEGER PRIMARY KEY);
             CREATE TABLE order_item (
                 id INTEGER PRIMARY KEY,
                 order_id INTEGER,
                 FOREIGN KEY (order_id) REFERENCES \"order\"(id)
             );",
        );
        let result = scan_sqlite(&path).unwrap();
        assert!(result.entities["order_item"].references.contains("order"));
        assert!(result.entities["order"].references.is_empty());
    }

    #[test]
    fn aggregate_root_detection_works_from_real_schema() {
        let (_conn, _dir, path) = open_temp_db(
            "CREATE TABLE \"order\" (id INTEGER PRIMARY KEY);
             CREATE TABLE order_item (
                 id INTEGER PRIMARY KEY,
                 order_id INTEGER,
                 FOREIGN KEY (order_id) REFERENCES \"order\"(id)
             );
             CREATE TABLE payment (
                 id INTEGER PRIMARY KEY,
                 order_id INTEGER,
                 FOREIGN KEY (order_id) REFERENCES \"order\"(id)
             );",
        );
        let result = scan_sqlite(&path).unwrap();
        let roots = result.aggregate_roots();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].0, "order");
        let mut children = roots[0].1.clone();
        children.sort();
        assert_eq!(children, vec!["order_item", "payment"]);
    }

    #[test]
    fn sqlite_internal_tables_are_excluded() {
        let (_conn, _dir, path) = open_temp_db("CREATE TABLE product (id INTEGER PRIMARY KEY);");
        let result = scan_sqlite(&path).unwrap();
        assert!(!result.entities.keys().any(|k| k.starts_with("sqlite_")));
    }

    #[test]
    fn empty_database_yields_no_entities() {
        let (_conn, _dir, path) = open_temp_db("");
        let result = scan_sqlite(&path).unwrap();
        assert!(result.entities.is_empty());
    }
}
