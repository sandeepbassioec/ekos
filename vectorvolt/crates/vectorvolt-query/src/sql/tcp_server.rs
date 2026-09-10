//! SQL-over-TCP server (S35, VV-NET-SQL-001..005).
//!
//! Protocol: newline-delimited JSON frames on a plain TCP stream.
//!
//! Request frame  (client → server):
//!   `{"sql":"SELECT * FROM t"}\n`
//!
//! Response frame (server → client):
//!   success: `{"ok":true,"type":"rows","columns":[...],"rows":[[...],...]}\n`
//!   success: `{"ok":true,"type":"affected","count":N}\n`
//!   success: `{"ok":true,"type":"ok"}\n`
//!   error:   `{"ok":false,"error":"..."}\n`
//!
//! Each TCP connection gets its own [`SqlExecutor`], so transactions are
//! per-connection — BEGIN on one connection is invisible to another.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::thread;

use serde_json::{Value, json};
use vectorvolt_engine::TableRegistry;

use crate::sql::executor::{QueryResult, SqlExecutor};

// ── SqlTcpServer ─────────────────────────────────────────────────────────────

/// A multi-threaded SQL-over-TCP server.
///
/// Each accepted connection runs on its own OS thread with a dedicated
/// [`SqlExecutor`] (no shared per-connection state).  The underlying
/// [`TableRegistry`] *is* shared across connections — it is already
/// `Arc`-wrapped and internally synchronised.
pub struct SqlTcpServer {
    registry: Arc<TableRegistry>,
}

impl SqlTcpServer {
    pub fn new(registry: Arc<TableRegistry>) -> Self {
        Self { registry }
    }

    /// Bind to `addr` and start accepting connections.
    ///
    /// Returns the [`TcpListener`] so the caller can read `.local_addr()`
    /// (useful for ephemeral-port tests).  Spawns a background thread that
    /// loops forever; drop the listener to stop accepting new connections.
    pub fn listen<A: ToSocketAddrs>(&self, addr: A) -> std::io::Result<TcpListener> {
        let listener = TcpListener::bind(addr)?;
        let registry = Arc::clone(&self.registry);
        let listener_clone = listener.try_clone()?;
        thread::spawn(move || {
            for stream in listener_clone.incoming() {
                match stream {
                    Ok(s) => {
                        let reg = Arc::clone(&registry);
                        thread::spawn(move || handle_connection(s, reg));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(listener)
    }
}

// ── Connection handler ────────────────────────────────────────────────────────

fn handle_connection(stream: TcpStream, registry: Arc<TableRegistry>) {
    let mut writer = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };
    let reader = BufReader::new(stream);
    let executor = SqlExecutor::new(registry);

    for line in reader.lines() {
        let line = match line {
            Ok(l) if l.is_empty() => continue,
            Ok(l) => l,
            Err(_) => break,
        };
        let response = handle_request(&executor, &line);
        let mut out = response.to_string();
        out.push('\n');
        if writer.write_all(out.as_bytes()).is_err() {
            break;
        }
    }
}

fn handle_request(executor: &SqlExecutor, raw: &str) -> Value {
    let sql = match serde_json::from_str::<Value>(raw) {
        Ok(v) => match v.get("sql").and_then(|s| s.as_str()) {
            Some(s) => s.to_owned(),
            None => {
                return json!({"ok": false, "error": "missing 'sql' field"});
            }
        },
        Err(e) => {
            return json!({"ok": false, "error": format!("json parse error: {e}")});
        }
    };

    match executor.execute(&sql) {
        Ok(result) => encode_result(result),
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

fn encode_result(result: QueryResult) -> Value {
    match result {
        QueryResult::Rows { columns, rows } => {
            json!({"ok": true, "type": "rows", "columns": columns, "rows": rows})
        }
        QueryResult::Affected(n) => {
            json!({"ok": true, "type": "affected", "count": n})
        }
        QueryResult::Created => json!({"ok": true, "type": "ok", "op": "created"}),
        QueryResult::Dropped => json!({"ok": true, "type": "ok", "op": "dropped"}),
        QueryResult::ExplainPlan(lines) => {
            json!({"ok": true, "type": "explain", "plan": lines})
        }
        QueryResult::TxBegin => json!({"ok": true, "type": "ok", "op": "begin"}),
        QueryResult::TxCommit => json!({"ok": true, "type": "ok", "op": "commit"}),
        QueryResult::TxRollback => json!({"ok": true, "type": "ok", "op": "rollback"}),
    }
}

// ── SqlSession (convenience client wrapper for tests) ─────────────────────────

/// Thin synchronous client for talking to a [`SqlTcpServer`].
pub struct SqlSession {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}

impl SqlSession {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Self {
            writer: stream,
            reader,
        })
    }

    /// Send one SQL statement and return the parsed JSON response.
    pub fn query(&mut self, sql: &str) -> serde_json::Result<Value> {
        let req = format!("{}\n", json!({"sql": sql}));
        self.writer.write_all(req.as_bytes()).unwrap();
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        serde_json::from_str(line.trim())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;
    use vectorvolt_engine::EngineOptions;

    fn start_server() -> (SqlTcpServer, TcpListener, TempDir) {
        let dir = TempDir::new().unwrap();
        let registry = Arc::new(TableRegistry::open(dir.path(), EngineOptions::default()).unwrap());
        let server = SqlTcpServer::new(Arc::clone(&registry));
        let listener = server.listen("127.0.0.1:0").unwrap();
        (server, listener, dir)
    }

    fn session(listener: &TcpListener) -> SqlSession {
        let addr = listener.local_addr().unwrap();
        // Small retry loop — the accept thread may not be ready immediately.
        for _ in 0..20 {
            if let Ok(s) = SqlSession::connect(addr) {
                return s;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        SqlSession::connect(addr).unwrap()
    }

    #[test]
    fn test_s35_ddl_over_tcp() {
        let (_srv, listener, _dir) = start_server();
        let mut s = session(&listener);
        let r = s.query("CREATE TABLE t (pk U64, v U64)").unwrap();
        assert_eq!(r["ok"], true);
        assert_eq!(r["op"], "created");
    }

    #[test]
    fn test_s35_insert_select_over_tcp() {
        let (_srv, listener, _dir) = start_server();
        let mut s = session(&listener);
        s.query("CREATE TABLE items (pk U64, price U64)").unwrap();
        s.query("INSERT INTO items (pk, price) VALUES (1, 500)")
            .unwrap();
        s.query("INSERT INTO items (pk, price) VALUES (2, 800)")
            .unwrap();
        let r = s.query("SELECT pk, price FROM items WHERE pk = 1").unwrap();
        assert_eq!(r["ok"], true);
        assert_eq!(r["type"], "rows");
        assert_eq!(r["rows"][0][1], "500");
    }

    #[test]
    fn test_s35_txn_commit_visible() {
        let (_srv, listener, _dir) = start_server();
        let mut s = session(&listener);
        s.query("CREATE TABLE acct (pk U64, bal U64)").unwrap();
        s.query("BEGIN").unwrap();
        s.query("INSERT INTO acct (pk, bal) VALUES (1, 1000)")
            .unwrap();
        let commit_r = s.query("COMMIT").unwrap();
        assert_eq!(commit_r["op"], "commit");
        // After commit, a new query (same connection) sees the row.
        let r = s.query("SELECT pk FROM acct WHERE pk = 1").unwrap();
        assert_eq!(r["rows"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_s35_txn_rollback_not_visible() {
        let (_srv, listener, _dir) = start_server();
        let mut s = session(&listener);
        s.query("CREATE TABLE ledger (pk U64, amount U64)").unwrap();
        s.query("BEGIN").unwrap();
        s.query("INSERT INTO ledger (pk, amount) VALUES (42, 999)")
            .unwrap();
        let rb_r = s.query("ROLLBACK").unwrap();
        assert_eq!(rb_r["op"], "rollback");
        // After rollback the row must not exist.
        let r = s.query("SELECT pk FROM ledger WHERE pk = 42").unwrap();
        assert_eq!(r["rows"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_s35_two_connections_isolated() {
        let (_srv, listener, _dir) = start_server();
        let mut s1 = session(&listener);
        let mut s2 = session(&listener);
        s1.query("CREATE TABLE shared (pk U64, v U64)").unwrap();
        // s1 begins a transaction and inserts — s2 must not see uncommitted data.
        s1.query("BEGIN").unwrap();
        s1.query("INSERT INTO shared (pk, v) VALUES (7, 77)")
            .unwrap();
        let r2 = s2.query("SELECT pk FROM shared WHERE pk = 7").unwrap();
        // s2 sees 0 rows (transaction not committed yet).
        assert_eq!(r2["rows"].as_array().unwrap().len(), 0);
        // Commit on s1.
        s1.query("COMMIT").unwrap();
        // Now s2 should see the row.
        let r2_after = s2.query("SELECT pk FROM shared WHERE pk = 7").unwrap();
        assert_eq!(r2_after["rows"].as_array().unwrap().len(), 1);
    }
}
