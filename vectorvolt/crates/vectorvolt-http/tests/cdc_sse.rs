/// Layer 65 — CDC / SSE streaming subscriptions.
///
/// Route:
///   GET /tables/{t}/subscribe[?col=&lo=&hi=]   → text/event-stream
///
/// Each SSE `data:` line is a JSON-serialised `ChangeEvent`:
///   {"kind":"insert","pk":1,"values":[1,10,99]}
///   {"kind":"delete","pk":1,"values":null}
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde_json::Value;
use tempfile::TempDir;
use tokio_stream::StreamExt as _;
use vectorvolt_engine::{EngineOptions, TableRegistry};
use vectorvolt_http::{HttpServer, HttpServerConfig};

const FLUSH: usize = 1_048_576;

// ── harness ───────────────────────────────────────────────────────────────────

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
            .timeout(Duration::from_secs(5))
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
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Collect up to `n` SSE events from the stream within `timeout`.
///
/// Opens the subscription, triggers work via `trigger`, then reads SSE chunks
/// until we have `n` events or the timeout fires.
async fn collect_events(
    client: &Client,
    sub_url: &str,
    trigger: impl std::future::Future<Output = ()>,
    n: usize,
    timeout: Duration,
) -> Vec<Value> {
    let resp = client
        .get(sub_url)
        .timeout(Duration::from_secs(10)) // long timeout — SSE never ends
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    // Kick off the mutations after the subscription is open.
    trigger.await;

    let mut stream = resp.bytes_stream();
    let mut buf = String::new();
    let mut events: Vec<Value> = Vec::new();

    let _ = tokio::time::timeout(timeout, async {
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();
            buf.push_str(&String::from_utf8_lossy(&chunk));
            // Parse all complete `data:` lines in the buffer.
            while let Some(nl) = buf.find('\n') {
                let line: String = buf.drain(..=nl).collect();
                let line = line.trim();
                if let Some(json) = line.strip_prefix("data:")
                    && let Ok(v) = serde_json::from_str(json.trim())
                {
                    events.push(v);
                    if events.len() >= n {
                        return;
                    }
                }
            }
        }
    })
    .await;

    events
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_subscribe_status_ok() {
    let h = Harness::start("orders", &["pk", "qty", "price"]).await;
    let resp = h
        .client
        .get(h.url("/tables/orders/subscribe"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("text/event-stream"),
        "expected SSE content-type, got {ct}"
    );
}

#[tokio::test]
async fn test_subscribe_unknown_table_returns_404() {
    let h = Harness::start("t", &["pk"]).await;
    let resp = h
        .client
        .get(h.url("/tables/ghost/subscribe"))
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn test_subscribe_receives_insert_event() {
    let h = Harness::start("logs", &["pk", "level", "ts"]).await;
    let sub_url = h.url("/tables/logs/subscribe");
    let insert_url = h.url("/tables/logs/rows/1");
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            client
                .post(&insert_url)
                .json(&serde_json::json!({ "values": [1, 2, 99] }))
                .send()
                .await
                .unwrap();
        },
        1,
        Duration::from_secs(2),
    )
    .await;

    assert!(!events.is_empty(), "expected at least 1 event");
    let ev = &events[0];
    assert_eq!(ev["kind"], "insert");
    assert_eq!(ev["pk"], 1);
    assert_eq!(ev["values"], serde_json::json!([1, 2, 99]));
}

#[tokio::test]
async fn test_subscribe_receives_delete_event() {
    let h = Harness::start("items", &["pk", "qty"]).await;
    // Pre-insert row.
    h.client
        .post(h.url("/tables/items/rows/5"))
        .json(&serde_json::json!({ "values": [5, 10] }))
        .send()
        .await
        .unwrap();

    let sub_url = h.url("/tables/items/subscribe");
    let del_url = h.url("/tables/items/rows/5");
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            client.delete(&del_url).send().await.unwrap();
        },
        1,
        Duration::from_secs(2),
    )
    .await;

    assert!(!events.is_empty(), "expected a delete event");
    let ev = &events[0];
    assert_eq!(ev["kind"], "delete");
    assert_eq!(ev["pk"], 5);
    assert!(ev["values"].is_null());
}

#[tokio::test]
async fn test_subscribe_receives_upsert_event() {
    let h = Harness::start("prices", &["pk", "val"]).await;
    let sub_url = h.url("/tables/prices/subscribe");
    let ups_url = h.url("/tables/prices/rows/7");
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            client
                .put(&ups_url)
                .json(&serde_json::json!({ "values": [7, 42] }))
                .send()
                .await
                .unwrap();
        },
        1,
        Duration::from_secs(2),
    )
    .await;

    assert!(!events.is_empty(), "expected an upsert (insert) event");
    assert_eq!(events[0]["kind"], "insert");
    assert_eq!(events[0]["pk"], 7);
}

#[tokio::test]
async fn test_subscribe_filter_passes_matching() {
    let h = Harness::start("rows", &["pk", "score"]).await;
    // Filter col=1, lo=50, hi=100 — only score in [50,100] passes.
    let sub_url = format!("{}?col=1&lo=50&hi=100", h.url("/tables/rows/subscribe"));
    let ins_url = h.url("/tables/rows/rows/1");
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            client
                .post(&ins_url)
                .json(&serde_json::json!({ "values": [1, 75] }))
                .send()
                .await
                .unwrap();
        },
        1,
        Duration::from_secs(2),
    )
    .await;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["values"][1], 75);
}

#[tokio::test]
async fn test_subscribe_filter_blocks_nonmatching() {
    let h = Harness::start("scores", &["pk", "val"]).await;
    // Filter col=1, lo=80, hi=100 — inserting val=20 should NOT produce an event.
    let sub_url = format!("{}?col=1&lo=80&hi=100", h.url("/tables/scores/subscribe"));
    let ins_url = h.url("/tables/scores/rows/2");
    let ins_url2 = h.url("/tables/scores/rows/3");
    let client = h.client.clone();

    // Insert the non-matching row, then after a delay insert a matching row as a sentinel.
    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            // Non-matching insert (val=20, filter requires [80,100]).
            client
                .post(&ins_url)
                .json(&serde_json::json!({ "values": [2, 20] }))
                .send()
                .await
                .unwrap();
            // Matching sentinel insert.
            client
                .post(&ins_url2)
                .json(&serde_json::json!({ "values": [3, 90] }))
                .send()
                .await
                .unwrap();
        },
        1,
        Duration::from_secs(2),
    )
    .await;

    // Only the sentinel (val=90) should arrive.
    assert_eq!(events.len(), 1, "expected exactly 1 event (sentinel only)");
    assert_eq!(events[0]["values"][1], 90);
}

#[tokio::test]
async fn test_subscribe_delete_suppressed_by_filter() {
    let h = Harness::start("del_test", &["pk", "score"]).await;
    // Pre-insert a row.
    h.client
        .post(h.url("/tables/del_test/rows/1"))
        .json(&serde_json::json!({ "values": [1, 50] }))
        .send()
        .await
        .unwrap();

    // Filter active: delete events (no values) should be suppressed.
    let sub_url = format!("{}?col=1&lo=0&hi=100", h.url("/tables/del_test/subscribe"));
    let del_url = h.url("/tables/del_test/rows/1");
    let ins_url = h.url("/tables/del_test/rows/2");
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            // Delete — should be suppressed by filter.
            client.delete(&del_url).send().await.unwrap();
            // Sentinel insert that passes the filter.
            client
                .post(&ins_url)
                .json(&serde_json::json!({ "values": [2, 60] }))
                .send()
                .await
                .unwrap();
        },
        1,
        Duration::from_secs(2),
    )
    .await;

    // Only the sentinel insert should arrive (delete suppressed).
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["kind"], "insert");
}

#[tokio::test]
async fn test_subscribe_no_filter_receives_delete() {
    let h = Harness::start("nd_test", &["pk", "v"]).await;
    h.client
        .post(h.url("/tables/nd_test/rows/10"))
        .json(&serde_json::json!({ "values": [10, 5] }))
        .send()
        .await
        .unwrap();

    let sub_url = h.url("/tables/nd_test/subscribe");
    let del_url = h.url("/tables/nd_test/rows/10");
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            client.delete(&del_url).send().await.unwrap();
        },
        1,
        Duration::from_secs(2),
    )
    .await;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["kind"], "delete");
    assert!(events[0]["values"].is_null());
}

#[tokio::test]
async fn test_subscribe_multiple_events() {
    let h = Harness::start("multi", &["pk", "v"]).await;
    let sub_url = h.url("/tables/multi/subscribe");
    let base = h.base_url.clone();
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            for i in 1u64..=5 {
                client
                    .post(format!("{base}/tables/multi/rows/{i}"))
                    .json(&serde_json::json!({ "values": [i, i * 10] }))
                    .send()
                    .await
                    .unwrap();
            }
        },
        5,
        Duration::from_secs(3),
    )
    .await;

    assert_eq!(events.len(), 5, "expected 5 insert events");
    for ev in &events {
        assert_eq!(ev["kind"], "insert");
    }
}

#[tokio::test]
async fn test_subscribe_multiple_subscribers_each_receive() {
    let dir = TempDir::new().unwrap();
    let opts = EngineOptions::new(FLUSH);
    let reg = Arc::new(TableRegistry::open(dir.path(), opts).unwrap());
    let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
    let srv = HttpServer::bind(Arc::clone(&reg), cfg).await.unwrap();
    let addr = srv.local_addr().unwrap();
    tokio::spawn(async move { srv.serve().await.unwrap() });
    let base = format!("http://{addr}");
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    client
        .post(format!("{base}/tables/shared"))
        .json(&serde_json::json!({ "columns": ["pk", "val"] }))
        .send()
        .await
        .unwrap();

    use tokio::sync::mpsc;

    let (tx1, mut rx1) = mpsc::channel::<Value>(16);
    let (tx2, mut rx2) = mpsc::channel::<Value>(16);

    // Subscriber 1.
    let c1 = client.clone();
    let b1 = base.clone();
    tokio::spawn(async move {
        let resp = c1
            .get(format!("{b1}/tables/shared/subscribe"))
            .send()
            .await
            .unwrap();
        let mut stream = resp.bytes_stream();
        let mut buf = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();
            buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(nl) = buf.find('\n') {
                let line: String = buf.drain(..=nl).collect();
                if let Some(json) = line.trim().strip_prefix("data:")
                    && let Ok(v) = serde_json::from_str(json.trim())
                {
                    let _ = tx1.send(v).await;
                }
            }
        }
    });

    // Subscriber 2.
    let c2 = client.clone();
    let b2 = base.clone();
    tokio::spawn(async move {
        let resp = c2
            .get(format!("{b2}/tables/shared/subscribe"))
            .send()
            .await
            .unwrap();
        let mut stream = resp.bytes_stream();
        let mut buf = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();
            buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(nl) = buf.find('\n') {
                let line: String = buf.drain(..=nl).collect();
                if let Some(json) = line.trim().strip_prefix("data:")
                    && let Ok(v) = serde_json::from_str(json.trim())
                {
                    let _ = tx2.send(v).await;
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Insert a row.
    client
        .post(format!("{base}/tables/shared/rows/42"))
        .json(&serde_json::json!({ "values": [42, 99] }))
        .send()
        .await
        .unwrap();

    // Both subscribers should receive the event.
    let ev1 = tokio::time::timeout(Duration::from_secs(2), rx1.recv()).await;
    let ev2 = tokio::time::timeout(Duration::from_secs(2), rx2.recv()).await;
    assert!(
        ev1.is_ok() && ev1.unwrap().is_some(),
        "subscriber 1 did not receive event"
    );
    assert!(
        ev2.is_ok() && ev2.unwrap().is_some(),
        "subscriber 2 did not receive event"
    );
}

#[tokio::test]
async fn test_subscribe_two_tables_isolated() {
    let dir = TempDir::new().unwrap();
    let opts = EngineOptions::new(FLUSH);
    let reg = Arc::new(TableRegistry::open(dir.path(), opts).unwrap());
    let cfg = HttpServerConfig::new("127.0.0.1:0".parse().unwrap());
    let srv = HttpServer::bind(Arc::clone(&reg), cfg).await.unwrap();
    let addr = srv.local_addr().unwrap();
    tokio::spawn(async move { srv.serve().await.unwrap() });
    let base = format!("http://{addr}");
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    for t in &["tableA", "tableB"] {
        client
            .post(format!("{base}/tables/{t}"))
            .json(&serde_json::json!({ "columns": ["pk", "v"] }))
            .send()
            .await
            .unwrap();
    }

    use tokio::sync::mpsc;
    let (tx_a, mut rx_a) = mpsc::channel::<Value>(16);

    let c = client.clone();
    let b = base.clone();
    tokio::spawn(async move {
        let resp = c
            .get(format!("{b}/tables/tableA/subscribe"))
            .send()
            .await
            .unwrap();
        let mut stream = resp.bytes_stream();
        let mut buf = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();
            buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(nl) = buf.find('\n') {
                let line: String = buf.drain(..=nl).collect();
                if let Some(json) = line.trim().strip_prefix("data:")
                    && let Ok(v) = serde_json::from_str(json.trim())
                {
                    let _ = tx_a.send(v).await;
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Insert into tableB only — tableA subscriber should NOT see it.
    client
        .post(format!("{base}/tables/tableB/rows/1"))
        .json(&serde_json::json!({ "values": [1, 2] }))
        .send()
        .await
        .unwrap();

    // Insert into tableA — tableA subscriber SHOULD see it.
    client
        .post(format!("{base}/tables/tableA/rows/2"))
        .json(&serde_json::json!({ "values": [2, 3] }))
        .send()
        .await
        .unwrap();

    let ev = tokio::time::timeout(Duration::from_secs(2), rx_a.recv())
        .await
        .expect("timeout")
        .expect("channel closed");
    assert_eq!(ev["pk"], 2, "tableA subscriber received event from tableB");
}

#[tokio::test]
async fn test_subscribe_filter_range() {
    let h = Harness::start("range_test", &["pk", "age"]).await;
    let sub_url = format!(
        "{}?col=1&lo=20&hi=30",
        h.url("/tables/range_test/subscribe")
    );
    let base = h.base_url.clone();
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            for (pk, age) in [(1u64, 15u64), (2, 25), (3, 35), (4, 22)] {
                client
                    .post(format!("{base}/tables/range_test/rows/{pk}"))
                    .json(&serde_json::json!({ "values": [pk, age] }))
                    .send()
                    .await
                    .unwrap();
            }
        },
        2,
        Duration::from_secs(3),
    )
    .await;

    assert_eq!(
        events.len(),
        2,
        "expected exactly 2 events (age=25 and age=22)"
    );
    let ages: Vec<u64> = events
        .iter()
        .map(|e| e["values"][1].as_u64().unwrap())
        .collect();
    assert!(ages.contains(&25));
    assert!(ages.contains(&22));
}

#[tokio::test]
async fn test_subscribe_field_update_emits_event() {
    let h = Harness::start("fu_test", &["pk", "val"]).await;
    // Pre-insert.
    h.client
        .post(h.url("/tables/fu_test/rows/1"))
        .json(&serde_json::json!({ "values": [1, 10] }))
        .send()
        .await
        .unwrap();

    let sub_url = h.url("/tables/fu_test/subscribe");
    let patch_url = h.url("/tables/fu_test/rows/1/field");
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            client
                .patch(&patch_url)
                .json(&serde_json::json!({ "col": 1, "val": 99 }))
                .send()
                .await
                .unwrap();
        },
        1,
        Duration::from_secs(2),
    )
    .await;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["kind"], "insert"); // field update goes through insert path
    assert_eq!(events[0]["values"][1], 99);
}

#[tokio::test]
async fn test_subscribe_cas_success_emits_event() {
    let h = Harness::start("cas_test", &["pk", "v"]).await;
    h.client
        .post(h.url("/tables/cas_test/rows/5"))
        .json(&serde_json::json!({ "values": [5, 10] }))
        .send()
        .await
        .unwrap();

    let sub_url = h.url("/tables/cas_test/subscribe");
    let cas_url = h.url("/tables/cas_test/rows/5/cas");
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            client
                .post(&cas_url)
                .json(&serde_json::json!({ "expected": [5, 10], "new_values": [5, 20] }))
                .send()
                .await
                .unwrap();
        },
        1,
        Duration::from_secs(2),
    )
    .await;

    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["values"][1], 20);
}

#[tokio::test]
async fn test_subscribe_purge_expired_emits_delete_events() {
    let h = Harness::start("ttl_test", &["pk", "v"]).await;

    // Insert rows with TTL that expire at time 100.
    for pk in 1u64..=3 {
        h.client
            .post(h.url(&format!("/tables/ttl_test/rows/{pk}/ttl")))
            .json(&serde_json::json!({ "values": [pk, pk * 10], "expire_at": 100 }))
            .send()
            .await
            .unwrap();
    }

    let sub_url = h.url("/tables/ttl_test/subscribe");
    let purge_url = h.url("/tables/ttl_test/purge_expired");
    let client = h.client.clone();

    let events = collect_events(
        &h.client,
        &sub_url,
        async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            // Purge at now=200 — all 3 rows have expired.
            client
                .post(&purge_url)
                .query(&[("now", "200")])
                .send()
                .await
                .unwrap();
        },
        3,
        Duration::from_secs(3),
    )
    .await;

    assert_eq!(events.len(), 3, "expected 3 delete events from purge");
    for ev in &events {
        assert_eq!(ev["kind"], "delete");
    }
}
