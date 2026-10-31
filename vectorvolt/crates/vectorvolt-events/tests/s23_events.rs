//! S23 integration tests — VV-EVT-001..016

use std::collections::HashMap;
use std::sync::Arc;

use vectorvolt_events::config::{
    EventFieldDef, EventRouteConfig, SinkKind, StructuredField, TriggerKind,
};
use vectorvolt_events::sink::SinkError;
use vectorvolt_events::{
    DomainEventEmitter, DomainEventEnvelope, DomainEventSchema, EventDlq, EventOutbox, EventSink,
    InProcessChannelSink, RedisStreamSink, SchemaVersionRegistry, YamlEventRouteConfig,
};
use vectorvolt_txn::Mutation;

fn no_lookup(_table: &str, _pk: u64) -> Option<Vec<u64>> {
    None
}

// ── VV-EVT-003: DomainEventEnvelope ─────────────────────────────────────────

#[test]
fn envelope_json_round_trip() {
    let env = DomainEventEnvelope::new(
        "OrderPlaced",
        "1.2.0",
        "orders",
        42,
        serde_json::json!({"order_id": 42}),
    );
    let json = env.to_json().unwrap();
    assert!(json.contains("OrderPlaced"));
    assert!(json.contains("1.2.0"));
}

#[test]
fn envelope_msgpack_round_trip() {
    let env = DomainEventEnvelope::new(
        "OrderCancelled",
        "1.0.0",
        "orders",
        7,
        serde_json::json!({}),
    );
    let bytes = env.to_msgpack().unwrap();
    let decoded = DomainEventEnvelope::from_msgpack(&bytes).unwrap();
    assert_eq!(decoded.event_type, "OrderCancelled");
    assert_eq!(decoded.pk, 7);
}

// ── VV-EVT-014: ComputedEventField ──────────────────────────────────────────

#[test]
fn computed_field_total_dollars() {
    let mut fields = HashMap::new();
    fields.insert(
        "total_dollars".into(),
        EventFieldDef::Structured(StructuredField {
            expr: Some("values[1] / 100".into()),
            object: None,
            lookup: None,
        }),
    );
    let payload = vectorvolt_events::field::resolve_fields(&fields, 1, &[0, 25000], &no_lookup);
    assert_eq!(payload["total_dollars"], serde_json::json!(250));
}

// ── VV-EVT-015: NestedEventField ─────────────────────────────────────────────

#[test]
fn nested_field_address_object() {
    let mut sub = HashMap::new();
    sub.insert("city".into(), "values[3]".into());
    sub.insert("country".into(), "values[4]".into());
    let mut fields = HashMap::new();
    fields.insert(
        "address".into(),
        EventFieldDef::Structured(StructuredField {
            expr: None,
            object: Some(sub),
            lookup: None,
        }),
    );
    let payload =
        vectorvolt_events::field::resolve_fields(&fields, 1, &[0, 0, 0, 99, 44], &no_lookup);
    let addr = &payload["address"];
    assert_eq!(addr["city"], serde_json::json!(99u64));
    assert_eq!(addr["country"], serde_json::json!(44u64));
}

// ── VV-EVT-016: LookupEnrichmentField ────────────────────────────────────────

#[test]
fn lookup_field_resolves_foreign_value() {
    use vectorvolt_events::config::LookupDef;
    let mut fields = HashMap::new();
    fields.insert(
        "customer_name_id".into(),
        EventFieldDef::Structured(StructuredField {
            expr: None,
            object: None,
            lookup: Some(LookupDef {
                table: "customers".into(),
                pk: "values[0]".into(),
                column: 1,
            }),
        }),
    );
    let lookup = |table: &str, pk: u64| -> Option<Vec<u64>> {
        if table == "customers" && pk == 5 {
            Some(vec![5, 99])
        } else {
            None
        }
    };
    let payload = vectorvolt_events::field::resolve_fields(&fields, 1, &[5], &lookup);
    assert_eq!(payload["customer_name_id"], serde_json::json!(99u64));
}

// ── VV-EVT-002: DomainEventEmitter + when: filter ───────────────────────────

#[test]
fn emitter_when_filter_suppresses_small_orders() {
    let schema = DomainEventSchema {
        name: "LargeOrderPlaced".into(),
        version: "1.0.0".into(),
        entity: "orders".into(),
        trigger: TriggerKind::Insert,
        when: Some("values[1] > 100000".into()),
        fields: HashMap::new(),
        routes: vec![EventRouteConfig {
            sink: SinkKind::InProcess,
            topic: None,
            partition_key: None,
            stream: None,
            maxlen: None,
            url: None,
            hmac_secret: None,
        }],
    };
    let mut emitter = DomainEventEmitter::new("orders", vec![schema]);
    let (sink, mut rx) = InProcessChannelSink::new(16);
    emitter.register_sink("in_process", Arc::new(sink));

    // Small order — filtered out
    emitter.on_commit(
        &Mutation::Insert {
            pk: 1,
            values: vec![0, 500],
        },
        &no_lookup,
    );
    assert!(rx.try_recv().is_err(), "small order should be filtered");

    // Large order — passes filter
    emitter.on_commit(
        &Mutation::Insert {
            pk: 2,
            values: vec![0, 200_000],
        },
        &no_lookup,
    );
    let received = rx.try_recv().expect("large order should pass filter");
    assert_eq!(received.event_type, "LargeOrderPlaced");
}

#[test]
fn emitter_delete_trigger() {
    let schema = DomainEventSchema {
        name: "OrderCancelled".into(),
        version: "1.0.0".into(),
        entity: "orders".into(),
        trigger: TriggerKind::Delete,
        when: None,
        fields: {
            let mut m = HashMap::new();
            m.insert("order_id".into(), EventFieldDef::Direct("pk".into()));
            m
        },
        routes: vec![EventRouteConfig {
            sink: SinkKind::InProcess,
            topic: None,
            partition_key: None,
            stream: None,
            maxlen: None,
            url: None,
            hmac_secret: None,
        }],
    };
    let mut emitter = DomainEventEmitter::new("orders", vec![schema]);
    let (sink, mut rx) = InProcessChannelSink::new(16);
    emitter.register_sink("in_process", Arc::new(sink));

    emitter.on_commit(&Mutation::Delete { pk: 99 }, &no_lookup);
    let env = rx.try_recv().expect("delete should emit OrderCancelled");
    assert_eq!(env.event_type, "OrderCancelled");
    assert_eq!(env.pk, 99);
}

// ── VV-EVT-007: InProcessChannelSink ─────────────────────────────────────────

#[test]
fn in_process_sink_broadcasts() {
    let (sink, mut rx1) = InProcessChannelSink::new(8);
    let mut rx2 = sink.subscribe();
    let env = DomainEventEnvelope::new("Test", "1.0.0", "t", 1, serde_json::json!({}));
    sink.send(&env).unwrap();
    assert!(rx1.try_recv().is_ok());
    assert!(rx2.try_recv().is_ok());
}

// ── VV-EVT-005: RedisStreamSink (noop) ───────────────────────────────────────

#[test]
fn redis_stream_sink_noop_succeeds() {
    let sink = RedisStreamSink::noop("vv:events:orders");
    let env = DomainEventEnvelope::new("OrderPlaced", "1.0.0", "orders", 1, serde_json::json!({}));
    assert!(sink.send(&env).is_ok());
}

// ── VV-EVT-008: EventOutbox ──────────────────────────────────────────────────

#[test]
fn outbox_accumulates_and_flushes() {
    let outbox = EventOutbox::new();
    let dlq = EventDlq::new();
    let (sink, mut rx) = InProcessChannelSink::new(16);

    for i in 0..3u64 {
        outbox.push(DomainEventEnvelope::new(
            "Ev",
            "1.0.0",
            "t",
            i,
            serde_json::json!({}),
        ));
    }
    assert_eq!(outbox.pending_count(), 3);

    outbox.flush(&sink, &dlq);
    assert_eq!(outbox.pending_count(), 0);
    assert_eq!(dlq.len(), 0);
    // 3 messages in channel
    for _ in 0..3 {
        rx.try_recv().unwrap();
    }
}

#[test]
fn outbox_failed_delivery_goes_to_dlq() {
    struct FailSink;
    impl EventSink for FailSink {
        fn send(&self, _: &DomainEventEnvelope) -> Result<(), SinkError> {
            Err(SinkError::ChannelClosed)
        }
    }
    let outbox = EventOutbox::new();
    let dlq = EventDlq::new();
    outbox.push(DomainEventEnvelope::new(
        "Ev",
        "1.0.0",
        "t",
        1,
        serde_json::json!({}),
    ));
    outbox.flush(&FailSink, &dlq);
    assert_eq!(outbox.pending_count(), 0);
    assert_eq!(dlq.len(), 1);
}

// ── VV-EVT-012: EventDlq ────────────────────────────────────────────────────

#[test]
fn dlq_list_shows_entries() {
    let dlq = EventDlq::new();
    dlq.push(
        DomainEventEnvelope::new("Ev", "1.0.0", "t", 1, serde_json::json!({})),
        "timeout".into(),
    );
    let list = dlq.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].0, "Ev");
    assert_eq!(list[0].1, "timeout");
}

#[test]
fn dlq_drop_all_clears() {
    let dlq = EventDlq::new();
    dlq.push(
        DomainEventEnvelope::new("E", "1.0.0", "t", 1, serde_json::json!({})),
        "err".into(),
    );
    dlq.drop_all();
    assert!(dlq.is_empty());
}

// ── VV-EVT-013: SchemaVersionRegistry ────────────────────────────────────────

#[test]
fn registry_register_and_list_versions() {
    let tmp = tempfile::tempdir().unwrap();
    let reg = SchemaVersionRegistry::new(tmp.path()).unwrap();
    let schema = DomainEventSchema {
        name: "OrderPlaced".into(),
        version: "1.0.0".into(),
        entity: "orders".into(),
        trigger: TriggerKind::Insert,
        when: None,
        fields: HashMap::new(),
        routes: vec![],
    };
    reg.register(&schema).unwrap();
    let versions = reg.list_versions("orders", "OrderPlaced").unwrap();
    assert_eq!(versions, vec!["1.0.0"]);
}

#[test]
fn registry_list_domains_and_events() {
    let tmp = tempfile::tempdir().unwrap();
    let reg = SchemaVersionRegistry::new(tmp.path()).unwrap();
    for (entity, name) in [
        ("orders", "OrderPlaced"),
        ("orders", "OrderCancelled"),
        ("inventory", "StockUpdated"),
    ] {
        reg.register(&DomainEventSchema {
            name: name.into(),
            version: "1.0.0".into(),
            entity: entity.into(),
            trigger: TriggerKind::Insert,
            when: None,
            fields: HashMap::new(),
            routes: vec![],
        })
        .unwrap();
    }
    let mut domains = reg.list_domains().unwrap();
    domains.sort();
    assert_eq!(domains, vec!["inventory", "orders"]);
    let mut order_events = reg.list_events("orders").unwrap();
    order_events.sort();
    assert_eq!(order_events, vec!["OrderCancelled", "OrderPlaced"]);
}

#[test]
fn registry_show_schema() {
    let tmp = tempfile::tempdir().unwrap();
    let reg = SchemaVersionRegistry::new(tmp.path()).unwrap();
    let schema = DomainEventSchema {
        name: "OrderPlaced".into(),
        version: "2.0.0".into(),
        entity: "orders".into(),
        trigger: TriggerKind::Insert,
        when: None,
        fields: HashMap::new(),
        routes: vec![],
    };
    reg.register(&schema).unwrap();
    let loaded = reg.show("orders", "OrderPlaced", "2.0.0").unwrap();
    assert_eq!(loaded.version, "2.0.0");
}

#[test]
fn registry_diff_two_versions() {
    let tmp = tempfile::tempdir().unwrap();
    let reg = SchemaVersionRegistry::new(tmp.path()).unwrap();
    let mut fields_v1 = HashMap::new();
    fields_v1.insert("order_id".into(), EventFieldDef::Direct("pk".into()));
    let v1 = DomainEventSchema {
        name: "Ev".into(),
        version: "1.0.0".into(),
        entity: "orders".into(),
        trigger: TriggerKind::Insert,
        when: None,
        fields: fields_v1,
        routes: vec![],
    };
    let mut fields_v2 = HashMap::new();
    fields_v2.insert("order_id".into(), EventFieldDef::Direct("pk".into()));
    fields_v2.insert("total".into(), EventFieldDef::Direct("values[1]".into()));
    let v2 = DomainEventSchema {
        name: "Ev".into(),
        version: "2.0.0".into(),
        entity: "orders".into(),
        trigger: TriggerKind::Insert,
        when: None,
        fields: fields_v2,
        routes: vec![],
    };
    reg.register(&v1).unwrap();
    reg.register(&v2).unwrap();
    let diffs = reg.diff("orders", "Ev", "1.0.0", "2.0.0").unwrap();
    assert!(
        diffs
            .iter()
            .any(|d| d.contains("fields") || d.contains("+"))
    );
}

// ── VV-EVT-009: YamlEventRouteConfig ─────────────────────────────────────────

#[test]
fn yaml_config_round_trip() {
    let yaml = r#"
domain_events:
  - entity: orders
    events:
      - name: OrderPlaced
        version: "1.2.0"
        trigger: insert
        when: "values[1] > 0"
        fields:
          order_id: "pk"
          total_cents: "values[1]"
        routes:
          - sink: kafka
            topic: orders.placed
sinks:
  kafka:
    brokers: ["kafka:9092"]
"#;
    let cfg: YamlEventRouteConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.domain_events[0].entity, "orders");
    assert_eq!(cfg.domain_events[0].events[0].name, "OrderPlaced");
    assert_eq!(cfg.domain_events[0].events[0].version, "1.2.0");
    assert!(cfg.sinks.kafka.is_some());
}

// ── PP-66 / B28: HttpWebhookSink reports real delivery results ──────────────

/// Minimal HTTP/1.1 server answering every request with `status`. Returns the
/// URL and a counter of requests received.
fn http_server(status: &'static str) -> (String, Arc<std::sync::atomic::AtomicUsize>) {
    use std::io::{Read, Write};
    use std::sync::atomic::{AtomicUsize, Ordering};

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/hook", listener.local_addr().unwrap());
    let hits = Arc::new(AtomicUsize::new(0));
    let hits2 = Arc::clone(&hits);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            // Read headers, then Content-Length bytes of body.
            let mut buf = Vec::new();
            let mut chunk = [0u8; 1024];
            let header_end = loop {
                let n = match stream.read(&mut chunk) {
                    Ok(0) | Err(_) => break None,
                    Ok(n) => n,
                };
                buf.extend_from_slice(&chunk[..n]);
                if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    break Some(i + 4);
                }
            };
            let Some(header_end) = header_end else {
                continue;
            };
            let headers = String::from_utf8_lossy(&buf[..header_end]).to_ascii_lowercase();
            let len: usize = headers
                .lines()
                .find_map(|l| l.strip_prefix("content-length:"))
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0);
            while buf.len() < header_end + len {
                match stream.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => buf.extend_from_slice(&chunk[..n]),
                }
            }
            hits2.fetch_add(1, Ordering::SeqCst);
            let resp =
                format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            let _ = stream.write_all(resp.as_bytes());
        }
    });
    (url, hits)
}

fn webhook_env() -> DomainEventEnvelope {
    DomainEventEnvelope::new(
        "OrderPlaced",
        "1.0.0",
        "orders",
        7,
        serde_json::json!({"id": 7}),
    )
}

#[test]
fn webhook_500_returns_err_after_retries_and_lands_in_dlq() {
    use std::sync::atomic::Ordering;
    use vectorvolt_events::HttpWebhookSink;

    let (url, hits) = http_server("500 Internal Server Error");
    let sink = HttpWebhookSink::new(url, Some("s3cret".into()), 5, 3, 1);

    let err = sink
        .send(&webhook_env())
        .expect_err("a 500 must not be reported as Ok");
    assert!(
        matches!(err, SinkError::Http(ref m) if m.contains("500")),
        "{err}"
    );
    assert_eq!(hits.load(Ordering::SeqCst), 3, "retry_max attempts");

    // The outbox must now route the failure to the DLQ.
    let outbox = EventOutbox::new();
    let dlq = EventDlq::new();
    outbox.push(webhook_env());
    outbox.flush(&sink, &dlq);
    assert_eq!(dlq.len(), 1);
    assert_eq!(outbox.pending_count(), 0);
}

#[test]
fn webhook_2xx_returns_ok() {
    use std::sync::atomic::Ordering;
    use vectorvolt_events::HttpWebhookSink;

    let (url, hits) = http_server("200 OK");
    let sink = HttpWebhookSink::new(url, None, 5, 3, 1);
    sink.send(&webhook_env()).unwrap();
    sink.send(&webhook_env()).unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 2);
}

#[tokio::test(flavor = "current_thread")]
async fn webhook_send_inside_runtime_reports_failure() {
    use vectorvolt_events::HttpWebhookSink;

    let (url, _hits) = http_server("500 Internal Server Error");
    let sink = HttpWebhookSink::new(url, None, 5, 2, 1);
    // Previously this spawned and returned Ok immediately.
    assert!(sink.send(&webhook_env()).is_err());
    // The explicit fire-and-forget path still exists and returns at once.
    assert!(sink.send_fire_and_forget(&webhook_env()).is_ok());
}
