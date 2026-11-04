//! Kafka sinks (VV-SNK-013).
//!
//! * [`KafkaSink`] - real producer built on the pure-Rust `rskafka` client.
//!   Events are JSON-encoded, buffered, and produced in batches; `flush`
//!   awaits broker acknowledgement.
//! * [`ChannelKafkaSink`] - in-memory **test double** that forwards messages
//!   to a tokio mpsc channel. It never talks to a broker.

use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use rskafka::{
    client::{
        ClientBuilder,
        partition::{Compression, PartitionClient, UnknownTopicHandling},
    },
    record::Record,
};
use tokio::sync::mpsc;

use crate::{
    error::{SinkError, SinkResult},
    sink::{Sink, SinkEvent, SinkEventKind},
};

fn event_to_json(event: &SinkEvent) -> String {
    let kind_str = match event.kind {
        SinkEventKind::Insert => "insert",
        SinkEventKind::Update => "update",
        SinkEventKind::Delete => "delete",
    };
    serde_json::json!({
        "table": event.table,
        "kind": kind_str,
        "pk": event.pk,
        "values": event.values,
    })
    .to_string()
}

/// Encode one event as a Kafka record: key = pk (big-endian), value = JSON.
pub fn encode_record(event: &SinkEvent) -> Record {
    Record {
        key: Some(event.pk.to_be_bytes().to_vec()),
        value: Some(event_to_json(event).into_bytes()),
        headers: Default::default(),
        timestamp: Utc::now(),
    }
}

/// Connection settings for [`KafkaSink`].
#[derive(Debug, Clone)]
pub struct KafkaSinkConfig {
    pub brokers: Vec<String>,
    pub topic: String,
    pub partition: i32,
    /// Auto-flush once this many records are buffered (min 1).
    pub batch_size: usize,
    pub connect_timeout: Duration,
}

impl KafkaSinkConfig {
    pub fn new(brokers: Vec<String>, topic: impl Into<String>) -> Self {
        Self {
            brokers,
            topic: topic.into(),
            partition: 0,
            batch_size: 100,
            connect_timeout: Duration::from_secs(5),
        }
    }
}

/// Real Kafka producer sink (pure-Rust `rskafka`).
pub struct KafkaSink {
    name: String,
    batch_size: usize,
    client: PartitionClient,
    buffer: tokio::sync::Mutex<Vec<Record>>,
}

impl KafkaSink {
    /// Connect to the cluster and resolve the target partition. Fails if no
    /// broker is reachable or the topic does not exist.
    pub async fn connect(name: &str, cfg: KafkaSinkConfig) -> SinkResult<Self> {
        if cfg.brokers.is_empty() {
            return Err(SinkError::Other("kafka: no brokers configured".into()));
        }
        let fut = async {
            let client = ClientBuilder::new(cfg.brokers.clone())
                .build()
                .await
                .map_err(|e| SinkError::Io(format!("kafka connect: {e}")))?;
            client
                .partition_client(
                    cfg.topic.clone(),
                    cfg.partition,
                    UnknownTopicHandling::Error,
                )
                .await
                .map_err(|e| SinkError::Io(format!("kafka partition client: {e}")))
        };
        let client = tokio::time::timeout(cfg.connect_timeout, fut)
            .await
            .map_err(|_| SinkError::Io("kafka connect: timed out".into()))??;
        Ok(Self {
            name: name.to_string(),
            batch_size: cfg.batch_size.max(1),
            client,
            buffer: tokio::sync::Mutex::new(Vec::new()),
        })
    }

    async fn produce_locked(&self, buf: &mut Vec<Record>) -> SinkResult<()> {
        if buf.is_empty() {
            return Ok(());
        }
        // Records stay buffered until the broker acknowledges them.
        self.client
            .produce(buf.clone(), Compression::NoCompression)
            .await
            .map_err(|e| SinkError::Io(format!("kafka produce: {e}")))?;
        buf.clear();
        Ok(())
    }
}

#[async_trait]
impl Sink for KafkaSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: SinkEvent) -> SinkResult<()> {
        let mut buf = self.buffer.lock().await;
        buf.push(encode_record(&event));
        if buf.len() >= self.batch_size {
            self.produce_locked(&mut buf).await?;
        }
        Ok(())
    }

    /// Produce all buffered records and await broker acknowledgement.
    async fn flush(&self) -> SinkResult<()> {
        let mut buf = self.buffer.lock().await;
        self.produce_locked(&mut buf).await
    }
}

// ── in-memory test double ────────────────────────────────────────────────────

/// A Kafka message: topic + JSON payload.
#[derive(Debug, Clone)]
pub struct KafkaMessage {
    pub topic: String,
    pub payload: String,
}

pub type KafkaSinkReceiver = mpsc::Receiver<KafkaMessage>;

/// TEST DOUBLE: serializes each [`SinkEvent`] to a JSON [`KafkaMessage`] and
/// sends it to an in-memory `mpsc` channel. Never contacts a broker.
pub struct ChannelKafkaSink {
    name: String,
    topic: String,
    tx: tokio::sync::Mutex<mpsc::Sender<KafkaMessage>>,
}

impl ChannelKafkaSink {
    /// Create a new sink with a buffered channel of `capacity` messages.
    pub fn new(name: &str, topic: &str, capacity: usize) -> (Self, KafkaSinkReceiver) {
        let (tx, rx) = mpsc::channel(capacity);
        let sink = Self {
            name: name.to_string(),
            topic: topic.to_string(),
            tx: tokio::sync::Mutex::new(tx),
        };
        (sink, rx)
    }
}

#[async_trait]
impl Sink for ChannelKafkaSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: SinkEvent) -> SinkResult<()> {
        let msg = KafkaMessage {
            topic: self.topic.clone(),
            payload: event_to_json(&event),
        };
        self.tx
            .lock()
            .await
            .send(msg)
            .await
            .map_err(|_| SinkError::ChannelClosed)
    }

    async fn flush(&self) -> SinkResult<()> {
        Ok(())
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::SinkEvent;

    #[tokio::test]
    async fn send_insert_reaches_receiver() {
        let (sink, mut rx) = ChannelKafkaSink::new("k", "events", 16);
        sink.send(SinkEvent::insert("orders", 1, vec![10, 20]))
            .await
            .unwrap();
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg.topic, "events");
        assert!(msg.payload.contains("insert"));
    }

    #[tokio::test]
    async fn send_delete_reaches_receiver() {
        let (sink, mut rx) = ChannelKafkaSink::new("k", "events", 16);
        sink.send(SinkEvent::delete("orders", 5)).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert!(msg.payload.contains("delete"));
    }

    #[tokio::test]
    async fn payload_contains_pk() {
        let (sink, mut rx) = ChannelKafkaSink::new("k", "events", 16);
        sink.send(SinkEvent::insert("t", 42, vec![])).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert!(msg.payload.contains("42"), "payload: {}", msg.payload);
    }

    #[tokio::test]
    async fn payload_contains_table() {
        let (sink, mut rx) = ChannelKafkaSink::new("k", "events", 16);
        sink.send(SinkEvent::insert("my_table", 1, vec![]))
            .await
            .unwrap();
        let msg = rx.recv().await.unwrap();
        assert!(msg.payload.contains("my_table"), "payload: {}", msg.payload);
    }

    #[tokio::test]
    async fn payload_is_valid_json() {
        let (sink, mut rx) = ChannelKafkaSink::new("k", "t", 16);
        sink.send(SinkEvent::insert("t", 1, vec![2, 3]))
            .await
            .unwrap();
        let msg = rx.recv().await.unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.payload).unwrap();
        assert_eq!(v["pk"], 1u64);
    }

    #[tokio::test]
    async fn multiple_events_ordered() {
        let (sink, mut rx) = ChannelKafkaSink::new("k", "t", 64);
        for i in 0..5u64 {
            sink.send(SinkEvent::insert("t", i, vec![i])).await.unwrap();
        }
        for i in 0..5u64 {
            let msg = rx.recv().await.unwrap();
            let v: serde_json::Value = serde_json::from_str(&msg.payload).unwrap();
            assert_eq!(v["pk"], i);
        }
    }

    #[tokio::test]
    async fn channel_closed_returns_error() {
        let (sink, rx) = ChannelKafkaSink::new("k", "t", 1);
        drop(rx);
        let result = sink.send(SinkEvent::insert("t", 1, vec![])).await;
        assert!(matches!(result, Err(SinkError::ChannelClosed)));
    }

    #[tokio::test]
    async fn topic_set_correctly() {
        let (sink, mut rx) = ChannelKafkaSink::new("k", "my-topic", 4);
        sink.send(SinkEvent::insert("t", 1, vec![])).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg.topic, "my-topic");
    }

    #[tokio::test]
    async fn flush_is_noop() {
        let (sink, _rx) = ChannelKafkaSink::new("k", "t", 4);
        sink.flush().await.unwrap();
    }

    #[test]
    fn name_matches_constructor() {
        let (sink, _) = ChannelKafkaSink::new("my-kafka", "t", 4);
        assert_eq!(sink.name(), "my-kafka");
    }

    #[tokio::test]
    async fn values_in_payload() {
        let (sink, mut rx) = ChannelKafkaSink::new("k", "t", 4);
        sink.send(SinkEvent::insert("t", 1, vec![11, 22, 33]))
            .await
            .unwrap();
        let msg = rx.recv().await.unwrap();
        assert!(msg.payload.contains("11"), "payload: {}", msg.payload);
    }

    #[tokio::test]
    async fn update_kind_in_payload() {
        let (sink, mut rx) = ChannelKafkaSink::new("k", "t", 4);
        let ev = SinkEvent {
            table: "t".into(),
            kind: SinkEventKind::Update,
            pk: 1,
            values: vec![5],
        };
        sink.send(ev).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert!(msg.payload.contains("update"), "payload: {}", msg.payload);
    }

    #[tokio::test]
    async fn delete_has_empty_values() {
        let (sink, mut rx) = ChannelKafkaSink::new("k", "t", 4);
        sink.send(SinkEvent::delete("t", 7)).await.unwrap();
        let msg = rx.recv().await.unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.payload).unwrap();
        assert_eq!(v["values"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn encode_record_key_and_value() {
        let rec = encode_record(&SinkEvent::insert("t", 7, vec![1, 2]));
        assert_eq!(rec.key.unwrap(), 7u64.to_be_bytes().to_vec());
        let v: serde_json::Value = serde_json::from_slice(&rec.value.unwrap()).unwrap();
        assert_eq!(v["kind"], "insert");
        assert_eq!(v["values"], serde_json::json!([1, 2]));
    }

    #[test]
    fn config_defaults() {
        let c = KafkaSinkConfig::new(vec!["b:9092".into()], "t");
        assert_eq!(c.partition, 0);
        assert_eq!(c.batch_size, 100);
    }

    #[tokio::test]
    async fn connect_without_brokers_errors() {
        let r = KafkaSink::connect("k", KafkaSinkConfig::new(vec![], "t")).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn connect_unreachable_errors() {
        let mut cfg = KafkaSinkConfig::new(vec!["127.0.0.1:1".into()], "t");
        cfg.connect_timeout = Duration::from_secs(10);
        let r = KafkaSink::connect("k", cfg).await;
        assert!(r.is_err(), "unreachable broker must be an error");
    }

    /// Needs a live broker: `VV_TEST_KAFKA=host:9092` and an existing topic
    /// named by `VV_TEST_KAFKA_TOPIC` (default `vv-test`).
    #[tokio::test]
    #[ignore = "requires VV_TEST_KAFKA"]
    async fn live_send_and_flush() {
        let broker = std::env::var("VV_TEST_KAFKA").expect("VV_TEST_KAFKA");
        let topic = std::env::var("VV_TEST_KAFKA_TOPIC").unwrap_or_else(|_| "vv-test".into());
        let sink = KafkaSink::connect("k", KafkaSinkConfig::new(vec![broker], topic))
            .await
            .unwrap();
        sink.send(SinkEvent::insert("t", 1, vec![1])).await.unwrap();
        sink.flush().await.unwrap();
    }
}
