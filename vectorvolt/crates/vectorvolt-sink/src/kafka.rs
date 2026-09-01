//! [`KafkaSink`] — serializes sink events to JSON and sends them to a tokio
//! mpsc channel (VV-SNK-013).
//!
//! The channel acts as a stand-in for a real Kafka producer: in tests the
//! receiver captures every message, while production code would bridge the
//! channel to `rdkafka` or another client.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{
    error::{SinkError, SinkResult},
    sink::{Sink, SinkEvent, SinkEventKind},
};

/// A Kafka message: topic + JSON payload.
#[derive(Debug, Clone)]
pub struct KafkaMessage {
    pub topic: String,
    pub payload: String,
}

pub type KafkaSinkReceiver = mpsc::Receiver<KafkaMessage>;

/// Serializes each [`SinkEvent`] to a JSON [`KafkaMessage`] and sends it to
/// an internal `mpsc` channel.
pub struct KafkaSink {
    name: String,
    topic: String,
    tx: Arc<tokio::sync::Mutex<mpsc::Sender<KafkaMessage>>>,
}

impl KafkaSink {
    /// Create a new sink with a buffered channel of `capacity` messages.
    pub fn new(name: &str, topic: &str, capacity: usize) -> (Self, KafkaSinkReceiver) {
        let (tx, rx) = mpsc::channel(capacity);
        let sink = Self {
            name: name.to_string(),
            topic: topic.to_string(),
            tx: Arc::new(tokio::sync::Mutex::new(tx)),
        };
        (sink, rx)
    }

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
}

#[async_trait]
impl Sink for KafkaSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: SinkEvent) -> SinkResult<()> {
        let payload = Self::event_to_json(&event);
        let msg = KafkaMessage {
            topic: self.topic.clone(),
            payload,
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
        let (sink, mut rx) = KafkaSink::new("k", "events", 16);
        sink.send(SinkEvent::insert("orders", 1, vec![10, 20]))
            .await
            .unwrap();
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg.topic, "events");
        assert!(msg.payload.contains("insert"));
    }

    #[tokio::test]
    async fn send_delete_reaches_receiver() {
        let (sink, mut rx) = KafkaSink::new("k", "events", 16);
        sink.send(SinkEvent::delete("orders", 5)).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert!(msg.payload.contains("delete"));
    }

    #[tokio::test]
    async fn payload_contains_pk() {
        let (sink, mut rx) = KafkaSink::new("k", "events", 16);
        sink.send(SinkEvent::insert("t", 42, vec![])).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert!(msg.payload.contains("42"), "payload: {}", msg.payload);
    }

    #[tokio::test]
    async fn payload_contains_table() {
        let (sink, mut rx) = KafkaSink::new("k", "events", 16);
        sink.send(SinkEvent::insert("my_table", 1, vec![]))
            .await
            .unwrap();
        let msg = rx.recv().await.unwrap();
        assert!(msg.payload.contains("my_table"), "payload: {}", msg.payload);
    }

    #[tokio::test]
    async fn payload_is_valid_json() {
        let (sink, mut rx) = KafkaSink::new("k", "t", 16);
        sink.send(SinkEvent::insert("t", 1, vec![2, 3]))
            .await
            .unwrap();
        let msg = rx.recv().await.unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.payload).unwrap();
        assert_eq!(v["pk"], 1u64);
    }

    #[tokio::test]
    async fn multiple_events_ordered() {
        let (sink, mut rx) = KafkaSink::new("k", "t", 64);
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
        let (sink, rx) = KafkaSink::new("k", "t", 1);
        drop(rx);
        let result = sink.send(SinkEvent::insert("t", 1, vec![])).await;
        assert!(matches!(result, Err(SinkError::ChannelClosed)));
    }

    #[tokio::test]
    async fn topic_set_correctly() {
        let (sink, mut rx) = KafkaSink::new("k", "my-topic", 4);
        sink.send(SinkEvent::insert("t", 1, vec![])).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg.topic, "my-topic");
    }

    #[tokio::test]
    async fn flush_is_noop() {
        let (sink, _rx) = KafkaSink::new("k", "t", 4);
        sink.flush().await.unwrap();
    }

    #[test]
    fn name_matches_constructor() {
        let (sink, _) = KafkaSink::new("my-kafka", "t", 4);
        assert_eq!(sink.name(), "my-kafka");
    }

    #[tokio::test]
    async fn values_in_payload() {
        let (sink, mut rx) = KafkaSink::new("k", "t", 4);
        sink.send(SinkEvent::insert("t", 1, vec![11, 22, 33]))
            .await
            .unwrap();
        let msg = rx.recv().await.unwrap();
        assert!(msg.payload.contains("11"), "payload: {}", msg.payload);
    }

    #[tokio::test]
    async fn update_kind_in_payload() {
        let (sink, mut rx) = KafkaSink::new("k", "t", 4);
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
        let (sink, mut rx) = KafkaSink::new("k", "t", 4);
        sink.send(SinkEvent::delete("t", 7)).await.unwrap();
        let msg = rx.recv().await.unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.payload).unwrap();
        assert_eq!(v["values"].as_array().unwrap().len(), 0);
    }
}
