/// S28 — SQL gateway integration tests (`POST /sql`).
use std::sync::Arc;

use reqwest::Client;
use serde_json::{Value, json};
use tempfile::TempDir;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

const FLUSH: usize = 1_048_576;

struct Harness {
    base_url: String,
    _dir: TempDir,
    client: Client,
}

impl Harness {
    async fn start() -> Self {
        let dir = TempDir::new().unwrap();
        let opts = EngineOptions::new(FLUSH);
        let reg = Arc::new(TableRegistry::open(dir.path(), opts).unwrap());
        let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
        let srv = HttpServer::bind(reg, cfg).await.unwrap();
        let addr = srv.local_addr().unwrap();
        tokio::spawn(async move { srv.serve().await.unwrap() });
        Harness {
            base_url: format!("http://{addr}"),
            _dir: dir,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }

    async fn sql(&self, sql: &str) -> Value {
        self.client
            .post(format!("{}/sql", self.base_url))
            .json(&json!({ "sql": sql }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

#[tokio::test]
async fn test_sql_create_insert_select() {
    let h = Harness::start().await;

    let r = h.sql("CREATE TABLE items (pk U64, price U64)").await;
    assert_eq!(r["type"], "ok");
    assert_eq!(r["ok"], true);

    let r = h.sql("INSERT INTO items (pk, price) VALUES (1, 100)").await;
    assert_eq!(r["type"], "affected");
    assert_eq!(r["affected"], 1);

    let r = h.sql("INSERT INTO items (pk, price) VALUES (2, 200)").await;
    assert_eq!(r["affected"], 1);

    let r = h.sql("SELECT * FROM items").await;
    assert_eq!(r["type"], "rows");
    let rows = r["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn test_sql_where_filter() {
    let h = Harness::start().await;
    h.sql("CREATE TABLE nums (pk U64, val U64)").await;
    h.sql("INSERT INTO nums (pk, val) VALUES (1, 10)").await;
    h.sql("INSERT INTO nums (pk, val) VALUES (2, 20)").await;
    h.sql("INSERT INTO nums (pk, val) VALUES (3, 30)").await;

    let r = h.sql("SELECT * FROM nums WHERE pk = 2").await;
    let rows = r["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn test_sql_explain_full_scan() {
    let h = Harness::start().await;
    h.sql("CREATE TABLE ex (pk U64, v U64)").await;

    let r = h.sql("EXPLAIN SELECT * FROM ex").await;
    assert_eq!(r["type"], "explain");
    let plan = r["plan"].as_array().unwrap();
    assert!(
        plan.iter()
            .any(|l| l.as_str().unwrap().contains("FullScan"))
    );
}

#[tokio::test]
async fn test_sql_drop_table() {
    let h = Harness::start().await;
    h.sql("CREATE TABLE tmp (pk U64, v U64)").await;

    let r = h.sql("DROP TABLE tmp").await;
    assert_eq!(r["type"], "ok");
    assert_eq!(r["ok"], true);
}

#[tokio::test]
async fn test_sql_bad_query_returns_400() {
    let h = Harness::start().await;

    let resp = h
        .client
        .post(format!("{}/sql", h.base_url))
        .json(&json!({ "sql": "SELECTT * FROM nowhere" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert!(body["error"].as_str().is_some());
}
