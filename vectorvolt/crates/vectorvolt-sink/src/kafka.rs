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

/// Record compression codec. Each codec is behind a default-on feature
/// (`compression-gzip`, `compression-snappy`, `compression-lz4`,
/// `compression-zstd`) mapped to the matching rskafka feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KafkaCompression {
    #[default]
    None,
    #[cfg(feature = "compression-gzip")]
    Gzip,
    #[cfg(feature = "compression-snappy")]
    Snappy,
    #[cfg(feature = "compression-lz4")]
    Lz4,
    #[cfg(feature = "compression-zstd")]
    Zstd,
}

impl KafkaCompression {
    /// Parse `none` / `gzip` / `snappy` / `lz4` / `zstd` (case-insensitive).
    pub fn parse(s: &str) -> SinkResult<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "" | "none" => Ok(Self::None),
            #[cfg(feature = "compression-gzip")]
            "gzip" => Ok(Self::Gzip),
            #[cfg(feature = "compression-snappy")]
            "snappy" => Ok(Self::Snappy),
            #[cfg(feature = "compression-lz4")]
            "lz4" => Ok(Self::Lz4),
            #[cfg(feature = "compression-zstd")]
            "zstd" => Ok(Self::Zstd),
            other => Err(SinkError::Other(format!(
                "kafka: unsupported compression {other:?} (supported: none{}{}{}{})",
                if cfg!(feature = "compression-gzip") {
                    ", gzip"
                } else {
                    ""
                },
                if cfg!(feature = "compression-snappy") {
                    ", snappy"
                } else {
                    ""
                },
                if cfg!(feature = "compression-lz4") {
                    ", lz4"
                } else {
                    ""
                },
                if cfg!(feature = "compression-zstd") {
                    ", zstd"
                } else {
                    ""
                },
            ))),
        }
    }

    fn to_rskafka(self) -> Compression {
        match self {
            Self::None => Compression::NoCompression,
            #[cfg(feature = "compression-gzip")]
            Self::Gzip => Compression::Gzip,
            #[cfg(feature = "compression-snappy")]
            Self::Snappy => Compression::Snappy,
            #[cfg(feature = "compression-lz4")]
            Self::Lz4 => Compression::Lz4,
            #[cfg(feature = "compression-zstd")]
            Self::Zstd => Compression::Zstd,
        }
    }
}
/// Bounded exponential-backoff retry policy for retriable produce errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Retries after the first attempt (0 = try once).
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
        }
    }
}

impl RetryPolicy {
    /// Delay before retry number `attempt` (0-based): `initial * 2^attempt`,
    /// capped at `max_backoff`.
    pub fn backoff(&self, attempt: u32) -> Duration {
        let factor = 1u32.checked_shl(attempt).unwrap_or(u32::MAX);
        self.initial_backoff
            .saturating_mul(factor)
            .min(self.max_backoff)
    }
}

/// Run `op`, retrying while `retriable(&err)` holds, at most
/// `policy.max_retries` times with exponential backoff.
pub async fn retry_with<T, E, Fut>(
    policy: &RetryPolicy,
    mut op: impl FnMut() -> Fut,
    retriable: impl Fn(&E) -> bool,
) -> Result<T, E>
where
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut attempt = 0u32;
    loop {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if attempt < policy.max_retries && retriable(&e) => {
                tokio::time::sleep(policy.backoff(attempt)).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Broker error codes that are transient and safe to retry.
pub fn is_retriable_protocol_error(e: &rskafka::client::error::ProtocolError) -> bool {
    use rskafka::client::error::ProtocolError as P;
    matches!(
        e,
        P::LeaderNotAvailable
            | P::NotLeaderOrFollower
            | P::RequestTimedOut
            | P::BrokerNotAvailable
            | P::NetworkException
            | P::NotEnoughReplicas
            | P::NotEnoughReplicasAfterAppend
            | P::CorruptMessage
    )
}

/// Whether an rskafka client error is worth retrying (transient broker /
/// connection failures; never request-shape or authorization errors).
pub fn is_retriable_error(e: &rskafka::client::error::Error) -> bool {
    use rskafka::client::error::Error as E;
    match e {
        E::ServerError { protocol_error, .. } => is_retriable_protocol_error(protocol_error),
        E::Connection(_) => true,
        _ => false,
    }
}

fn murmur2(data: &[u8]) -> u32 {
    const M: u32 = 0x5bd1_e995;
    const R: u32 = 24;
    let mut h: u32 = 0x9747_b28c ^ (data.len() as u32);
    let (chunks, rem) = data.as_chunks::<4>();
    for c in chunks {
        let mut k = u32::from_le_bytes(*c);
        k = k.wrapping_mul(M);
        k ^= k >> R;
        k = k.wrapping_mul(M);
        h = h.wrapping_mul(M) ^ k;
    }
    if !rem.is_empty() {
        for (i, b) in rem.iter().enumerate().rev() {
            h ^= (*b as u32) << (8 * i);
        }
        h = h.wrapping_mul(M);
    }
    h ^= h >> 13;
    h = h.wrapping_mul(M);
    h ^= h >> 15;
    h
}

/// Index into a topic's partition list for `key` (Kafka's default
/// partitioner: `(murmur2(key) & 0x7fffffff) % n`). Same key, same partition.
pub fn partition_index(key: &[u8], n_partitions: usize) -> usize {
    if n_partitions == 0 {
        return 0;
    }
    ((murmur2(key) & 0x7fff_ffff) as usize) % n_partitions
}

/// Connection settings for [`KafkaSink`].
#[derive(Debug, Clone)]
pub struct KafkaSinkConfig {
    pub brokers: Vec<String>,
    pub topic: String,
    /// `Some(p)` pins every record to partition `p`; `None` (default) hashes
    /// the record key across all partitions discovered from topic metadata.
    pub partition: Option<i32>,
    /// Auto-flush once this many records are buffered (min 1).
    pub batch_size: usize,
    pub connect_timeout: Duration,
    pub compression: KafkaCompression,
    pub retry: RetryPolicy,
}

impl KafkaSinkConfig {
    pub fn new(brokers: Vec<String>, topic: impl Into<String>) -> Self {
        Self {
            brokers,
            topic: topic.into(),
            partition: None,
            batch_size: 100,
            connect_timeout: Duration::from_secs(5),
            compression: KafkaCompression::None,
            retry: RetryPolicy::default(),
        }
    }
}

/// Real Kafka producer sink (pure-Rust `rskafka`).
pub struct KafkaSink {
    name: String,
    batch_size: usize,
    /// One client per topic partition (a single entry when pinned).
    clients: Vec<PartitionClient>,
    compression: KafkaCompression,
    retry: RetryPolicy,
    /// Buffered `(client index, record)` pairs.
    buffer: tokio::sync::Mutex<Vec<(usize, Record)>>,
}

impl KafkaSink {
    /// Connect to the cluster and resolve the target partition(s). Fails if no
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
            let partitions: Vec<i32> = match cfg.partition {
                Some(p) => vec![p],
                None => {
                    let topics = client
                        .list_topics()
                        .await
                        .map_err(|e| SinkError::Io(format!("kafka metadata: {e}")))?;
                    let t = topics
                        .into_iter()
                        .find(|t| t.name == cfg.topic)
                        .ok_or_else(|| {
                            SinkError::Other(format!("kafka: topic {:?} not found", cfg.topic))
                        })?;
                    if t.partitions.is_empty() {
                        return Err(SinkError::Other(format!(
                            "kafka: topic {:?} has no partitions",
                            cfg.topic
                        )));
                    }
                    t.partitions.into_iter().collect()
                }
            };
            let mut clients = Vec::with_capacity(partitions.len());
            for p in partitions {
                clients.push(
                    client
                        .partition_client(cfg.topic.clone(), p, UnknownTopicHandling::Error)
                        .await
                        .map_err(|e| SinkError::Io(format!("kafka partition client: {e}")))?,
                );
            }
            Ok::<_, SinkError>(clients)
        };
        let clients = tokio::time::timeout(cfg.connect_timeout, fut)
            .await
            .map_err(|_| SinkError::Io("kafka connect: timed out".into()))??;
        Ok(Self {
            name: name.to_string(),
            batch_size: cfg.batch_size.max(1),
            clients,
            compression: cfg.compression,
            retry: cfg.retry,
            buffer: tokio::sync::Mutex::new(Vec::new()),
        })
    }

    /// Number of partitions records are spread across.
    pub fn partition_count(&self) -> usize {
        self.clients.len()
    }

    /// Produce everything buffered, per partition, retrying transient errors.
    /// Records for a partition stay buffered until that partition's batch is
    /// acknowledged; the first failure is returned after all partitions tried.
    async fn produce_locked(&self, buf: &mut Vec<(usize, Record)>) -> SinkResult<()> {
        if buf.is_empty() {
            return Ok(());
        }
        let mut groups: Vec<Vec<Record>> = vec![Vec::new(); self.clients.len()];
        for (idx, rec) in buf.drain(..) {
            groups[idx].push(rec);
        }
        let codec = self.compression.to_rskafka();
        let mut first_err: Option<SinkError> = None;
        for (idx, records) in groups.into_iter().enumerate() {
            if records.is_empty() {
                continue;
            }
            let client = &self.clients[idx];
            let res = retry_with(
                &self.retry,
                || client.produce(records.clone(), codec),
                is_retriable_error,
            )
            .await;
            if let Err(e) = res {
                first_err.get_or_insert(SinkError::Io(format!("kafka produce: {e}")));
                buf.extend(records.into_iter().map(|r| (idx, r)));
            }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

#[async_trait]
impl Sink for KafkaSink {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: SinkEvent) -> SinkResult<()> {
        let mut buf = self.buffer.lock().await;
        let rec = encode_record(&event);
        let idx = match rec.key.as_deref() {
            Some(k) => partition_index(k, self.clients.len()),
            None => 0,
        };
        buf.push((idx, rec));
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
        assert_eq!(c.partition, None);
        assert_eq!(c.batch_size, 100);
        assert_eq!(c.compression, KafkaCompression::None);
        assert_eq!(c.retry.max_retries, 3);
    }

    #[test]
    fn partition_index_is_stable_and_in_range() {
        for pk in 0u64..500 {
            let k = pk.to_be_bytes();
            let a = partition_index(&k, 6);
            assert!(a < 6);
            assert_eq!(a, partition_index(&k, 6), "same key, same partition");
        }
        assert_eq!(partition_index(b"x", 0), 0);
        assert_eq!(partition_index(b"x", 1), 0);
    }

    #[test]
    fn partition_index_spreads_keys() {
        let mut hit = [0usize; 4];
        for pk in 0u64..400 {
            hit[partition_index(&pk.to_be_bytes(), 4)] += 1;
        }
        assert!(hit.iter().all(|&n| n > 40), "uneven spread: {hit:?}");
    }

    #[test]
    fn murmur2_matches_kafka_reference_vectors() {
        // Values from Kafka's Utils.murmur2 test suite.
        assert_eq!(murmur2(b"21") as i32, -973932308);
        assert_eq!(murmur2(b"foobar") as i32, -790332482);
        assert_eq!(murmur2(b"a-little-bit-long-string") as i32, -985981536);
    }

    #[test]
    fn backoff_is_exponential_and_capped() {
        let p = RetryPolicy {
            max_retries: 5,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_millis(500),
        };
        assert_eq!(p.backoff(0), Duration::from_millis(100));
        assert_eq!(p.backoff(1), Duration::from_millis(200));
        assert_eq!(p.backoff(2), Duration::from_millis(400));
        assert_eq!(p.backoff(3), Duration::from_millis(500));
        assert_eq!(p.backoff(40), Duration::from_millis(500));
    }

    fn fast_policy(max_retries: u32) -> RetryPolicy {
        RetryPolicy {
            max_retries,
            initial_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(2),
        }
    }

    #[tokio::test]
    async fn retry_succeeds_after_transient_failures() {
        let calls = std::sync::atomic::AtomicU32::new(0);
        let r: Result<u32, &str> = retry_with(
            &fast_policy(3),
            || async {
                let n = calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if n < 2 { Err("transient") } else { Ok(n) }
            },
            |_| true,
        )
        .await;
        assert_eq!(r, Ok(2));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_is_bounded() {
        let calls = std::sync::atomic::AtomicU32::new(0);
        let r: Result<(), &str> = retry_with(
            &fast_policy(2),
            || async {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err("down")
            },
            |_| true,
        )
        .await;
        assert_eq!(r, Err("down"));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_skips_non_retriable() {
        let calls = std::sync::atomic::AtomicU32::new(0);
        let r: Result<(), &str> = retry_with(
            &fast_policy(5),
            || async {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err("fatal")
            },
            |_| false,
        )
        .await;
        assert!(r.is_err());
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn retriable_protocol_errors() {
        use rskafka::client::error::ProtocolError as P;
        assert!(is_retriable_protocol_error(&P::NotLeaderOrFollower));
        assert!(is_retriable_protocol_error(&P::RequestTimedOut));
        assert!(!is_retriable_protocol_error(&P::TopicAuthorizationFailed));
        assert!(!is_retriable_protocol_error(&P::MessageTooLarge));
    }

    #[test]
    fn compression_parsing() {
        assert_eq!(
            KafkaCompression::parse("none").unwrap(),
            KafkaCompression::None
        );
        assert_eq!(KafkaCompression::parse("").unwrap(), KafkaCompression::None);
        assert!(KafkaCompression::parse("brotli").is_err());
        assert!(KafkaCompression::parse("bogus").is_err());
        #[cfg(feature = "compression-snappy")]
        assert_eq!(
            KafkaCompression::parse("Snappy").unwrap(),
            KafkaCompression::Snappy
        );
        #[cfg(feature = "compression-lz4")]
        assert_eq!(
            KafkaCompression::parse("lz4").unwrap(),
            KafkaCompression::Lz4
        );
        #[cfg(feature = "compression-zstd")]
        assert_eq!(
            KafkaCompression::parse(" zstd ").unwrap(),
            KafkaCompression::Zstd
        );
        #[cfg(feature = "compression-gzip")]
        assert_eq!(
            KafkaCompression::parse("GZIP").unwrap(),
            KafkaCompression::Gzip
        );
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
