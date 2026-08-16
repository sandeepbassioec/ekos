/// Layer 58 — Conditional writes via HTTP + DSL.
///
/// New routes:
///   POST /tables/{t}/rows/{pk}/if_absent  body: {"values":[...]}  → {"inserted":bool}
///   POST /tables/{t}/rows/{pk}/cas        body: {"expected":[...],"new_values":[...]} → {"swapped":bool}
///
/// New DSL statements:
///   INSERT IF_ABSENT <pk> <v1> [v2 ...]
///   CAS <pk> EXPECTED <v1> [v2 ...] NEW <v1> [v2 ...]
use std::sync::Arc;

use reqwest::Client;
use serde_json::Value;
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
    async fn start(table: &str, cols: &[&str]) -> Self {
        let dir = TempDir::new().unwrap();
        let opts = EngineOptions::new(FLUSH);
        let reg = Arc::new(TableRegistry::open(dir.path(), opts).unwrap());
        let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
        let srv = HttpServer::bind(reg, cfg).await.unwrap();
        let addr = srv.local_addr().unwrap();
        tokio::spawn(async move { srv.serve().await.unwrap() });
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        let base = format!("http://{addr}");
        client
            .post(format!("{base}/tables/{table}"))
            .json(&serde_json::json!({ "columns": cols }))
            .send()
            .await
            .unwrap();
        Harness {
            base_url: base,
            _dir: dir,
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    async fn insert(&self, table: &str, pk: u64, values: &[u64]) {
        self.client
            .post(self.url(&format!("/tables/{table}/rows/{pk}")))
            .json(&serde_json::json!({ "values": values }))
            .send()
            .await
            .unwrap();
    }

    async fn get(&self, table: &str, pk: u64) -> Value {
        self.client
            .get(self.url(&format!("/tables/{table}/rows/{pk}")))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

// ── insert_if_absent HTTP ─────────────────────────────────────────────────────

#[tokio::test]
async fn if_absent_inserts_when_row_does_not_exist() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/rows/1/if_absent"))
        .json(&serde_json::json!({ "values": [1, 42] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["inserted"], true);
    let row = h.get("t", 1).await;
    assert_eq!(row["values"][1].as_u64().unwrap(), 42);
}

#[tokio::test]
async fn if_absent_does_not_overwrite_existing_row() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 2, &[2, 99]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/rows/2/if_absent"))
        .json(&serde_json::json!({ "values": [2, 55] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["inserted"], false);
    let row = h.get("t", 2).await;
    assert_eq!(
        row["values"][1].as_u64().unwrap(),
        99,
        "original value preserved"
    );
}

#[tokio::test]
async fn if_absent_second_call_returns_false() {
    let h = Harness::start("t", &["pk", "v"]).await;
    let first: Value = h
        .client
        .post(h.url("/tables/t/rows/5/if_absent"))
        .json(&serde_json::json!({ "values": [5, 10] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let second: Value = h
        .client
        .post(h.url("/tables/t/rows/5/if_absent"))
        .json(&serde_json::json!({ "values": [5, 20] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(first["inserted"], true);
    assert_eq!(second["inserted"], false);
}

// ── compare_and_swap HTTP ────────────────────────────────────────────────────

#[tokio::test]
async fn cas_swaps_when_expected_matches() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 10, &[10, 100]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/rows/10/cas"))
        .json(&serde_json::json!({ "expected": [10, 100], "new_values": [10, 200] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["swapped"], true);
    let row = h.get("t", 10).await;
    assert_eq!(row["values"][1].as_u64().unwrap(), 200);
}

#[tokio::test]
async fn cas_does_not_swap_when_expected_differs() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 11, &[11, 100]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/rows/11/cas"))
        .json(&serde_json::json!({ "expected": [11, 999], "new_values": [11, 200] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["swapped"], false);
    let row = h.get("t", 11).await;
    assert_eq!(
        row["values"][1].as_u64().unwrap(),
        100,
        "original value preserved"
    );
}

#[tokio::test]
async fn cas_is_atomic_second_swap_sees_new_value() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 20, &[20, 1]).await;
    let r1: Value = h
        .client
        .post(h.url("/tables/t/rows/20/cas"))
        .json(&serde_json::json!({ "expected": [20, 1], "new_values": [20, 2] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let r2: Value = h
        .client
        .post(h.url("/tables/t/rows/20/cas"))
        .json(&serde_json::json!({ "expected": [20, 2], "new_values": [20, 3] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(r1["swapped"], true);
    assert_eq!(r2["swapped"], true);
    let row = h.get("t", 20).await;
    assert_eq!(row["values"][1].as_u64().unwrap(), 3);
}

// ── DSL: INSERT IF_ABSENT ─────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_insert_if_absent_inserts_new_row() {
    let h = Harness::start("t", &["pk", "x"]).await;
    let resp = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("INSERT IF_ABSENT 7 7 33")
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["inserted"], true);
}

#[tokio::test]
async fn dsl_insert_if_absent_no_overwrite() {
    let h = Harness::start("t", &["pk", "x"]).await;
    h.insert("t", 8, &[8, 77]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("INSERT IF_ABSENT 8 8 99")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["inserted"], false);
    let row = h.get("t", 8).await;
    assert_eq!(row["values"][1].as_u64().unwrap(), 77);
}

// ── DSL: CAS ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dsl_cas_swaps_on_match() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 30, &[30, 10]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("CAS 30 EXPECTED 30 10 NEW 30 20")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["swapped"], true);
    let row = h.get("t", 30).await;
    assert_eq!(row["values"][1].as_u64().unwrap(), 20);
}

#[tokio::test]
async fn dsl_cas_no_swap_on_mismatch() {
    let h = Harness::start("t", &["pk", "v"]).await;
    h.insert("t", 31, &[31, 5]).await;
    let resp: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("CAS 31 EXPECTED 31 999 NEW 31 100")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(resp["swapped"], false);
    let row = h.get("t", 31).await;
    assert_eq!(row["values"][1].as_u64().unwrap(), 5);
}

// ── HTTP and DSL results agree ────────────────────────────────────────────────

#[tokio::test]
async fn http_and_dsl_if_absent_agree() {
    let h = Harness::start("t", &["pk", "v"]).await;
    // HTTP inserts pk=100
    let http_r: Value = h
        .client
        .post(h.url("/tables/t/rows/100/if_absent"))
        .json(&serde_json::json!({ "values": [100, 7] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // DSL tries same pk
    let dsl_r: Value = h
        .client
        .post(h.url("/tables/t/query"))
        .header("content-type", "text/plain")
        .body("INSERT IF_ABSENT 100 100 8")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(http_r["inserted"], true);
    assert_eq!(dsl_r["inserted"], false);
}
