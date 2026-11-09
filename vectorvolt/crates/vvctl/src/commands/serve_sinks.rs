//! Sink wiring for `vvctl serve --sinks-config` (kafka / postgres / redis-stream).
//!
//! The config file (YAML, or TOML when the extension is `.toml`) declares sinks:
//!
//! ```yaml
//! poll_interval_ms: 200
//! sinks:
//!   - name: cdc-kafka
//!     type: kafka
//!     brokers: ["kafka:9092"]
//!     topic: vv-cdc
//!     compression: gzip        # none | gzip
//!     max_retries: 3
//!   - name: warehouse
//!     type: postgres
//!     conn_str: "host=db user=vv dbname=wh"
//!     table: public.events
//!     tls: { mode: verify-full, ca_file: /etc/ssl/db-ca.pem }
//!   - name: stream
//!     type: redis_stream
//!     url: "redis://cache:6379/0"
//!     stream: vv-events
//! ```
//!
//! Every sink is connected at startup; if any cannot connect, startup fails.
//! A [`WalSinkBridge`] per table then tails the WAL (decrypting it when
//! `--encrypt-at-rest` is on) into a durable outbox; an [`OutboxDispatcher`]
//! drains that to the sinks at-least-once, acking after a successful flush,
//! retrying with backoff and dead-lettering after `max_delivery_attempts`:
//!
//! ```yaml
//! max_delivery_attempts: 5     # default
//! retry_backoff_ms: 200        # default; doubles up to max_retry_backoff_ms
//! max_retry_backoff_ms: 30000
//! dispatch_batch_size: 100
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::Deserialize;
use vectorvolt_crypto::KeyStore;
use vectorvolt_events::{
    DomainEventEnvelope, EventDlq, EventOutbox, EventSink, RedisStreamSink, config::RedisSinkConfig,
};
use vectorvolt_sink::{
    KafkaCompression, KafkaSink, KafkaSinkConfig, PgTlsMode, PostgresSink, PostgresSinkConfig,
    PostgresTlsConfig, RetryPolicy, Sink, SinkError, SinkEvent, SinkEventKind, SinkRegistry,
    SinkResult, WalSinkBridge,
};
use vectorvolt_wal::WalConfig;

/// Parsed `--sinks-config` file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SinksFile {
    #[serde(default)]
    pub sinks: Vec<SinkSpec>,
    /// How often each table's WAL is polled for new mutations.
    #[serde(default = "default_poll_ms")]
    pub poll_interval_ms: u64,
    /// Failed delivery attempts before an event moves to the DLQ.
    #[serde(default = "default_max_attempts")]
    pub max_delivery_attempts: u32,
    /// First retry delay; doubles per consecutive failure up to
    /// `max_retry_backoff_ms`.
    #[serde(default = "default_retry_backoff_ms")]
    pub retry_backoff_ms: u64,
    #[serde(default = "default_max_retry_backoff_ms")]
    pub max_retry_backoff_ms: u64,
    /// Events the dispatcher takes from the outbox per pass.
    #[serde(default = "default_dispatch_batch")]
    pub dispatch_batch_size: usize,
}

fn default_max_attempts() -> u32 {
    5
}
fn default_retry_backoff_ms() -> u64 {
    200
}
fn default_max_retry_backoff_ms() -> u64 {
    30_000
}
fn default_dispatch_batch() -> usize {
    100
}

fn default_poll_ms() -> u64 {
    200
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SinkSpec {
    Kafka {
        name: String,
        brokers: Vec<String>,
        topic: String,
        /// Pin to one partition; default hashes keys across all partitions.
        partition: Option<i32>,
        #[serde(default)]
        compression: Option<String>,
        batch_size: Option<usize>,
        max_retries: Option<u32>,
        retry_backoff_ms: Option<u64>,
        connect_timeout_ms: Option<u64>,
    },
    Postgres {
        name: String,
        conn_str: String,
        table: String,
        batch_size: Option<usize>,
        connect_timeout_ms: Option<u64>,
        #[serde(default)]
        tls: Option<PgTlsSpec>,
    },
    #[serde(alias = "redis-stream")]
    RedisStream {
        name: String,
        url: String,
        stream: String,
        maxlen: Option<u64>,
        connect_timeout_ms: Option<u64>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgTlsSpec {
    pub mode: String,
    pub ca_file: Option<PathBuf>,
}

impl SinkSpec {
    pub fn name(&self) -> &str {
        match self {
            Self::Kafka { name, .. }
            | Self::Postgres { name, .. }
            | Self::RedisStream { name, .. } => name,
        }
    }
}

/// Parse a sinks config from text (`toml` selects TOML, anything else YAML).
pub fn parse_sinks(text: &str, toml_format: bool) -> Result<SinksFile> {
    let file: SinksFile = if toml_format {
        toml::from_str(text).context("parse sinks config (TOML)")?
    } else {
        serde_yaml::from_str(text).context("parse sinks config (YAML)")?
    };
    let mut seen = std::collections::HashSet::new();
    for s in &file.sinks {
        if s.name().is_empty() {
            bail!("sinks config: a sink has an empty name");
        }
        if !seen.insert(s.name().to_string()) {
            bail!("sinks config: duplicate sink name {:?}", s.name());
        }
    }
    Ok(file)
}

pub fn load_sinks(path: &Path) -> Result<SinksFile> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read sinks config '{}'", path.display()))?;
    let is_toml = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("toml"));
    parse_sinks(&text, is_toml).with_context(|| format!("sinks config '{}'", path.display()))
}

/// Translate a Kafka spec into a [`KafkaSinkConfig`].
pub fn kafka_config(spec: &SinkSpec) -> Result<KafkaSinkConfig> {
    let SinkSpec::Kafka {
        brokers,
        topic,
        partition,
        compression,
        batch_size,
        max_retries,
        retry_backoff_ms,
        connect_timeout_ms,
        ..
    } = spec
    else {
        bail!("not a kafka sink");
    };
    let mut cfg = KafkaSinkConfig::new(brokers.clone(), topic.clone());
    cfg.partition = *partition;
    if let Some(c) = compression {
        cfg.compression = KafkaCompression::parse(c)?;
    }
    if let Some(b) = batch_size {
        cfg.batch_size = *b;
    }
    let mut retry = RetryPolicy::default();
    if let Some(r) = max_retries {
        retry.max_retries = *r;
    }
    if let Some(ms) = retry_backoff_ms {
        retry.initial_backoff = Duration::from_millis(*ms);
    }
    cfg.retry = retry;
    if let Some(ms) = connect_timeout_ms {
        cfg.connect_timeout = Duration::from_millis(*ms);
    }
    Ok(cfg)
}

/// Translate a Postgres spec into a [`PostgresSinkConfig`].
pub fn postgres_config(spec: &SinkSpec) -> Result<PostgresSinkConfig> {
    let SinkSpec::Postgres {
        conn_str,
        table,
        batch_size,
        connect_timeout_ms,
        tls,
        ..
    } = spec
    else {
        bail!("not a postgres sink");
    };
    let mut cfg = PostgresSinkConfig::new(conn_str.clone(), table.clone());
    if let Some(b) = batch_size {
        cfg.batch_size = *b;
    }
    if let Some(ms) = connect_timeout_ms {
        cfg.connect_timeout = Duration::from_millis(*ms);
    }
    if let Some(t) = tls {
        cfg.tls = PostgresTlsConfig {
            mode: PgTlsMode::parse(&t.mode)?,
            ca_file: t.ca_file.clone(),
        };
    }
    Ok(cfg)
}

/// Adapts the synchronous events-crate [`RedisStreamSink`] to the async
/// [`Sink`] trait used by the WAL bridge.
struct RedisAdapter {
    name: String,
    inner: Arc<RedisStreamSink>,
}

fn to_envelope(ev: &SinkEvent) -> DomainEventEnvelope {
    let kind = match ev.kind {
        SinkEventKind::Insert => "insert",
        SinkEventKind::Update => "update",
        SinkEventKind::Delete => "delete",
    };
    DomainEventEnvelope::new(
        format!("vectorvolt.{}.{kind}", ev.table),
        "1",
        ev.table.clone(),
        ev.pk,
        serde_json::json!({ "kind": kind, "values": ev.values }),
    )
}

#[async_trait]
impl Sink for RedisAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, event: SinkEvent) -> SinkResult<()> {
        let inner = Arc::clone(&self.inner);
        let env = to_envelope(&event);
        tokio::task::spawn_blocking(move || inner.send(&env))
            .await
            .map_err(|e| SinkError::Io(format!("redis send join: {e}")))?
            .map_err(|e| SinkError::Io(format!("redis send: {e}")))
    }

    async fn flush(&self) -> SinkResult<()> {
        Ok(())
    }
}

/// Connect one sink. Errors name the sink and its kind.
pub async fn build_sink(spec: &SinkSpec) -> Result<Arc<dyn Sink>> {
    match spec {
        SinkSpec::Kafka { name, .. } => {
            let cfg = kafka_config(spec)?;
            let s = KafkaSink::connect(name, cfg)
                .await
                .with_context(|| format!("sink {name:?} (kafka): cannot connect"))?;
            Ok(Arc::new(s))
        }
        SinkSpec::Postgres { name, .. } => {
            let cfg = postgres_config(spec)?;
            let s = PostgresSink::connect(name, cfg)
                .await
                .with_context(|| format!("sink {name:?} (postgres): cannot connect"))?;
            Ok(Arc::new(s))
        }
        SinkSpec::RedisStream {
            name,
            url,
            stream,
            maxlen,
            connect_timeout_ms,
        } => {
            let rcfg = RedisSinkConfig {
                url: url.clone(),
                connect_timeout_ms: connect_timeout_ms.unwrap_or(3000),
            };
            let (stream, maxlen) = (stream.clone(), *maxlen);
            // The redis client is blocking; keep it off the async workers.
            let inner = tokio::task::spawn_blocking(move || {
                RedisStreamSink::from_config(&rcfg, stream, maxlen)
            })
            .await
            .context("redis connect task")?
            .map_err(|e| anyhow::anyhow!("{e}"))
            .with_context(|| format!("sink {name:?} (redis_stream): cannot connect"))?;
            Ok(Arc::new(RedisAdapter {
                name: name.clone(),
                inner: Arc::new(inner),
            }))
        }
    }
}

/// Connect every configured sink into a registry (fails on the first sink
/// that cannot connect).
pub async fn build_registry(file: &SinksFile) -> Result<Arc<SinkRegistry>> {
    let registry = Arc::new(SinkRegistry::new());
    for spec in &file.sinks {
        registry.register(build_sink(spec).await?);
    }
    Ok(registry)
}

/// WAL directories to tail: `(table name, wal dir)`.
pub type TableWal = (String, PathBuf);

/// Delivery policy for the durable outbox dispatcher.
#[derive(Debug, Clone)]
pub struct OutboxSettings {
    /// A failing event is moved to the DLQ after this many failed attempts.
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub batch_size: usize,
}

impl OutboxSettings {
    pub fn from_file(f: &SinksFile) -> Self {
        Self {
            max_attempts: f.max_delivery_attempts.max(1),
            initial_backoff: Duration::from_millis(f.retry_backoff_ms.max(1)),
            max_backoff: Duration::from_millis(f.max_retry_backoff_ms.max(f.retry_backoff_ms)),
            batch_size: f.dispatch_batch_size.max(1),
        }
    }
}

/// Where the durable outbox / DLQ journals live under the data dir.
pub fn outbox_paths(data_dir: &Path) -> (PathBuf, PathBuf) {
    let dir = data_dir.join("events");
    (dir.join("outbox"), dir.join("dlq"))
}

/// Where the per-table bridge cursors live (`<data_dir>/events/cursor`).
pub fn cursor_dir_path(data_dir: &Path) -> PathBuf {
    data_dir.join("events").join("cursor")
}

/// Registry-side sink that the WAL bridges fan out to: it only appends the
/// change event to the durable outbox (fsynced) and never talks to a
/// downstream system. Delivery is the [`OutboxDispatcher`]'s job.
struct EnqueueSink {
    outbox: Arc<EventOutbox>,
}

#[async_trait]
impl Sink for EnqueueSink {
    fn name(&self) -> &str {
        "outbox"
    }

    async fn send(&self, event: SinkEvent) -> SinkResult<()> {
        let outbox = Arc::clone(&self.outbox);
        let env = to_envelope(&event);
        tokio::task::spawn_blocking(move || outbox.try_push(env))
            .await
            .map_err(|e| SinkError::Io(format!("outbox enqueue join: {e}")))?
            .map_err(|e| SinkError::Io(format!("outbox enqueue: {e}")))
    }

    async fn flush(&self) -> SinkResult<()> {
        Ok(())
    }
}

/// Inverse of [`to_envelope`]; `None` for an envelope this module did not write.
fn from_envelope(env: &DomainEventEnvelope) -> Option<SinkEvent> {
    let kind = match env.payload.get("kind")?.as_str()? {
        "insert" => SinkEventKind::Insert,
        "update" => SinkEventKind::Update,
        "delete" => SinkEventKind::Delete,
        _ => return None,
    };
    let values = env
        .payload
        .get("values")?
        .as_array()?
        .iter()
        .map(|v| v.as_u64())
        .collect::<Option<Vec<u64>>>()?;
    Some(SinkEvent {
        table: env.entity.clone(),
        kind,
        pk: env.pk,
        values,
    })
}

/// What one [`OutboxDispatcher::dispatch_once`] pass did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DispatchReport {
    /// Events delivered to every sink, flushed and acked.
    pub delivered: usize,
    /// Events moved to the DLQ after exhausting their attempts.
    pub dead_lettered: usize,
    /// A send or flush failed in this pass (caller should back off).
    pub failed: bool,
}

/// Drains the durable outbox to the configured sinks (at-least-once).
///
/// An event is acked only after every sink accepted it AND the sinks' `flush`
/// succeeded. Failures are retried (the entry stays in the outbox, so a crash
/// redelivers it too); after `max_attempts` failed attempts it moves to the DLQ.
pub struct OutboxDispatcher {
    outbox: Arc<EventOutbox>,
    dlq: Arc<EventDlq>,
    registry: Arc<SinkRegistry>,
    settings: OutboxSettings,
    consecutive_failures: u32,
}

impl OutboxDispatcher {
    pub fn new(
        outbox: Arc<EventOutbox>,
        dlq: Arc<EventDlq>,
        registry: Arc<SinkRegistry>,
        settings: OutboxSettings,
    ) -> Self {
        Self {
            outbox,
            dlq,
            registry,
            settings,
            consecutive_failures: 0,
        }
    }

    /// Record one failed attempt for `seq`; dead-letters (and acks) it once the
    /// limit is reached. Returns whether it was dead-lettered.
    fn record_failure(&mut self, seq: u64, env: &DomainEventEnvelope, err: &str) -> bool {
        let n = self.outbox.record_attempt(seq);
        if n >= self.settings.max_attempts {
            // The DLQ entry is durable before the outbox entry is acked.
            self.dlq
                .push(env.clone(), format!("{err} (after {n} attempts)"));
            self.outbox.ack(&[seq]);
            true
        } else {
            false
        }
    }

    /// Backoff to wait after the latest failing pass (exponential, capped).
    pub fn backoff(&self) -> Duration {
        let exp = self.consecutive_failures.saturating_sub(1).min(20);
        self.settings
            .initial_backoff
            .saturating_mul(1u32 << exp)
            .min(self.settings.max_backoff)
    }

    /// Deliver up to one batch from the outbox head, preserving order: the
    /// pass stops at the first event that fails to send.
    pub async fn dispatch_once(&mut self) -> DispatchReport {
        let batch = self.outbox.peek(self.settings.batch_size);
        let mut report = DispatchReport::default();
        if batch.is_empty() {
            self.consecutive_failures = 0;
            return report;
        }
        let mut sent: Vec<u64> = Vec::new();
        let mut send_err: Option<(u64, DomainEventEnvelope, String)> = None;
        for (seq, env) in &batch {
            let Some(ev) = from_envelope(env) else {
                // Not decodable: retrying cannot help.
                self.dlq
                    .push(env.clone(), "undecodable outbox envelope".into());
                self.outbox.ack(&[*seq]);
                report.dead_lettered += 1;
                continue;
            };
            match self.registry.fan_out(ev).await {
                Ok(()) => sent.push(*seq),
                Err(e) => {
                    send_err = Some((*seq, env.clone(), e.to_string()));
                    break;
                }
            }
        }
        let mut failed = false;
        if !sent.is_empty() {
            match self.registry.flush_all().await {
                Ok(()) => {
                    self.outbox.ack(&sent);
                    report.delivered += sent.len();
                }
                Err(e) => {
                    failed = true;
                    let msg = format!("flush: {e}");
                    for (seq, env) in batch.iter().filter(|(s, _)| sent.contains(s)) {
                        if self.record_failure(*seq, env, &msg) {
                            report.dead_lettered += 1;
                        }
                    }
                }
            }
        }
        if let Some((seq, env, msg)) = send_err {
            failed = true;
            if self.record_failure(seq, &env, &msg) {
                report.dead_lettered += 1;
            }
        }
        report.failed = failed;
        if failed {
            self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        } else {
            self.consecutive_failures = 0;
        }
        report
    }
}

/// Build the per-table bridge that tails `wal_dir` (decrypting with `key`
/// when the WAL is encrypted) and enqueues into `enqueue`.
fn make_bridge(
    wal_dir: &Path,
    enqueue: &Arc<SinkRegistry>,
    table: &str,
    key: &Option<Arc<KeyStore>>,
    cursor_dir: Option<&Path>,
) -> WalSinkBridge {
    let cfg = WalConfig::default().with_dir(wal_dir).with_fsync(false);
    let mut b = WalSinkBridge::new(cfg, Arc::clone(enqueue), table).with_key(key.clone());
    if let Some(dir) = cursor_dir {
        b = b.with_cursor_file(dir.join(table));
    }
    b
}

/// Connect the sinks, then start a background thread that tails each table's
/// WAL into the durable outbox (`<data_dir>/events/outbox`, DLQ at
/// `<data_dir>/events/dlq`) and a dispatcher that drains it to the sinks.
/// `tables` is re-evaluated every poll so tables created after startup are
/// picked up (multi-table mode). `key` is the at-rest key, if any.
///
/// Returns once every sink has connected; returns an error otherwise.
pub fn start(
    file: &SinksFile,
    tables: impl Fn() -> Vec<TableWal> + Send + 'static,
    data_dir: &Path,
    key: Option<Arc<KeyStore>>,
) -> Result<Arc<SinkRegistry>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("vv-sinks")
        .build()
        .context("sink runtime")?;
    let registry = rt.block_on(build_registry(file))?;
    let (outbox_path, dlq_path) = outbox_paths(data_dir);
    let outbox = Arc::new(
        EventOutbox::open(&outbox_path)
            .with_context(|| format!("open event outbox {}", outbox_path.display()))?,
    );
    let dlq = Arc::new(
        EventDlq::open(&dlq_path)
            .with_context(|| format!("open event DLQ {}", dlq_path.display()))?,
    );
    let cursor_dir = cursor_dir_path(data_dir);
    let enqueue = Arc::new(SinkRegistry::new());
    enqueue.register(Arc::new(EnqueueSink {
        outbox: Arc::clone(&outbox),
    }));
    let mut dispatcher = OutboxDispatcher::new(
        outbox,
        dlq,
        Arc::clone(&registry),
        OutboxSettings::from_file(file),
    );
    let poll = Duration::from_millis(file.poll_interval_ms.max(10));
    std::thread::Builder::new()
        .name("vv-sink-bridge".into())
        .spawn(move || {
            rt.block_on(async move {
                tokio::spawn(async move {
                    loop {
                        let r = dispatcher.dispatch_once().await;
                        if r.failed {
                            eprintln!(
                                "sink dispatch failed (delivered {}, dead-lettered {}); backing off",
                                r.delivered, r.dead_lettered
                            );
                            tokio::time::sleep(dispatcher.backoff()).await;
                        } else if r.delivered == 0 && r.dead_lettered == 0 {
                            tokio::time::sleep(poll).await;
                        }
                    }
                });
                let mut bridges: HashMap<String, WalSinkBridge> = HashMap::new();
                loop {
                    for (table, dir) in tables() {
                        bridges
                            .entry(table.clone())
                            .or_insert_with(|| make_bridge(&dir, &enqueue, &table, &key, Some(&cursor_dir)));
                    }
                    for (table, bridge) in &bridges {
                        if let Err(e) = bridge.poll_once().await {
                            eprintln!("sink bridge '{table}': {e} (will retry)");
                        }
                    }
                    tokio::time::sleep(poll).await;
                }
            });
        })
        .context("spawn sink bridge thread")?;
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    const YAML: &str = r#"
poll_interval_ms: 50
sinks:
  - name: k
    type: kafka
    brokers: ["127.0.0.1:1"]
    topic: t
    compression: gzip
    max_retries: 5
    retry_backoff_ms: 10
    partition: 2
  - name: p
    type: postgres
    conn_str: "host=127.0.0.1 port=1 user=x"
    table: public.ev
    tls: { mode: verify-full, ca_file: /tmp/ca.pem }
  - name: r
    type: redis_stream
    url: "redis://127.0.0.1:1/"
    stream: s
    maxlen: 100
"#;

    #[test]
    fn parses_all_three_kinds() {
        let f = parse_sinks(YAML, false).unwrap();
        assert_eq!(f.poll_interval_ms, 50);
        assert_eq!(f.sinks.len(), 3);
        let k = kafka_config(&f.sinks[0]).unwrap();
        assert_eq!(k.partition, Some(2));
        assert_eq!(k.retry.max_retries, 5);
        assert_eq!(k.retry.initial_backoff, Duration::from_millis(10));
        let p = postgres_config(&f.sinks[1]).unwrap();
        assert_eq!(p.tls.mode, PgTlsMode::VerifyFull);
        assert_eq!(p.tls.ca_file.as_deref(), Some(Path::new("/tmp/ca.pem")));
        assert!(matches!(f.sinks[2], SinkSpec::RedisStream { .. }));
    }

    #[test]
    fn parses_toml() {
        let f = parse_sinks(
            "[[sinks]]\nname='r'\ntype='redis_stream'\nurl='redis://h/'\nstream='s'\n",
            true,
        )
        .unwrap();
        assert_eq!(f.poll_interval_ms, 200);
        assert_eq!(f.sinks[0].name(), "r");
    }

    #[test]
    fn rejects_duplicates_unknown_fields_and_bad_tls() {
        let dup = "sinks:\n - {name: a, type: redis_stream, url: u, stream: s}\n - {name: a, type: redis_stream, url: u, stream: s}\n";
        assert!(parse_sinks(dup, false).is_err());
        let unk = "sinks:\n - {name: a, type: redis_stream, url: u, stream: s, bogus: 1}\n";
        assert!(parse_sinks(unk, false).is_err());
        let bad =
            "sinks:\n - {name: a, type: postgres, conn_str: c, table: t, tls: {mode: maybe}}\n";
        let f = parse_sinks(bad, false).unwrap();
        assert!(postgres_config(&f.sinks[0]).is_err());
        let badc =
            "sinks:\n - {name: a, type: kafka, brokers: [b], topic: t, compression: brotli}\n";
        let f = parse_sinks(badc, false).unwrap();
        assert!(kafka_config(&f.sinks[0]).is_err());
    }

    #[test]
    fn unreachable_sinks_fail_startup_with_named_error() {
        let f = parse_sinks(
            "sinks:\n - {name: pg1, type: postgres, conn_str: 'host=127.0.0.1 port=1 user=x connect_timeout=2', table: t, connect_timeout_ms: 2000}\n",
            false,
        )
        .unwrap();
        let err = start(&f, Vec::new, std::env::temp_dir().as_path(), None)
            .err()
            .expect("must fail");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("pg1") && msg.contains("cannot connect"),
            "{msg}"
        );

        let f = parse_sinks(
            "sinks:\n - {name: rd, type: redis_stream, url: 'redis://127.0.0.1:1/', stream: s, connect_timeout_ms: 500}\n",
            false,
        )
        .unwrap();
        let err = start(&f, Vec::new, std::env::temp_dir().as_path(), None)
            .err()
            .expect("must fail");
        assert!(format!("{err:#}").contains("rd"), "{err:#}");

        let f = parse_sinks(
            "sinks:\n - {name: kf, type: kafka, brokers: ['127.0.0.1:1'], topic: t, connect_timeout_ms: 2000}\n",
            false,
        )
        .unwrap();
        let err = start(&f, Vec::new, std::env::temp_dir().as_path(), None)
            .err()
            .expect("must fail");
        assert!(format!("{err:#}").contains("kf"), "{err:#}");
    }

    #[test]
    fn envelope_mapping() {
        let e = to_envelope(&SinkEvent::insert("t", 9, vec![1, 2]));
        assert_eq!(e.event_type, "vectorvolt.t.insert");
        assert_eq!(e.pk, 9);
        assert_eq!(e.payload["values"], serde_json::json!([1, 2]));
    }

    // ── outbox dispatcher ────────────────────────────────────────────────────

    use std::sync::Mutex as StdMutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    struct TestSink {
        got: StdMutex<Vec<(u64, SinkEventKind)>>,
        fail_send: AtomicBool,
        fail_flush: AtomicBool,
        flushes_ok: AtomicUsize,
    }

    impl TestSink {
        fn new(fail_send: bool, fail_flush: bool) -> Arc<Self> {
            Arc::new(Self {
                got: StdMutex::new(Vec::new()),
                fail_send: AtomicBool::new(fail_send),
                fail_flush: AtomicBool::new(fail_flush),
                flushes_ok: AtomicUsize::new(0),
            })
        }
    }

    #[async_trait]
    impl Sink for TestSink {
        fn name(&self) -> &str {
            "test"
        }
        async fn send(&self, e: SinkEvent) -> SinkResult<()> {
            if self.fail_send.load(Ordering::SeqCst) {
                return Err(SinkError::Io("boom".into()));
            }
            self.got.lock().unwrap().push((e.pk, e.kind));
            Ok(())
        }
        async fn flush(&self) -> SinkResult<()> {
            if self.fail_flush.load(Ordering::SeqCst) {
                return Err(SinkError::Io("flush boom".into()));
            }
            self.flushes_ok.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn settings(max_attempts: u32) -> OutboxSettings {
        OutboxSettings {
            max_attempts,
            initial_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(4),
            batch_size: 10,
        }
    }

    fn dispatcher(
        dir: &Path,
        sink: Arc<TestSink>,
        max_attempts: u32,
    ) -> (OutboxDispatcher, Arc<EventOutbox>, Arc<EventDlq>) {
        let (op, dp) = outbox_paths(dir);
        let outbox = Arc::new(EventOutbox::open(op).unwrap());
        let dlq = Arc::new(EventDlq::open(dp).unwrap());
        let reg = Arc::new(SinkRegistry::new());
        reg.register(sink);
        let d = OutboxDispatcher::new(
            Arc::clone(&outbox),
            Arc::clone(&dlq),
            reg,
            settings(max_attempts),
        );
        (d, outbox, dlq)
    }

    fn enqueue(outbox: &EventOutbox, n: u64) {
        for pk in 0..n {
            outbox
                .try_push(to_envelope(&SinkEvent::insert("t", pk, vec![pk, pk + 1])))
                .unwrap();
        }
    }

    #[test]
    fn envelope_roundtrip() {
        let ev = SinkEvent::delete("t", 4);
        let back = from_envelope(&to_envelope(&ev)).unwrap();
        assert_eq!(
            (back.table.as_str(), back.pk, back.kind),
            ("t", 4, SinkEventKind::Delete)
        );
    }

    #[tokio::test]
    async fn acks_only_after_successful_flush() {
        let dir = tempfile::tempdir().unwrap();
        let sink = TestSink::new(false, true);
        let (mut d, outbox, dlq) = dispatcher(dir.path(), sink.clone(), 5);
        enqueue(&outbox, 3);

        let r = d.dispatch_once().await;
        assert!(r.failed && r.delivered == 0);
        assert_eq!(outbox.pending_count(), 3, "flush failed: nothing acked");
        assert!(dlq.is_empty());

        sink.fail_flush.store(false, Ordering::SeqCst);
        let r = d.dispatch_once().await;
        assert_eq!(r.delivered, 3);
        assert!(!r.failed);
        assert_eq!(outbox.pending_count(), 0);
        assert!(sink.flushes_ok.load(Ordering::SeqCst) >= 1);
    }

    #[tokio::test]
    async fn dead_letters_after_max_attempts() {
        let dir = tempfile::tempdir().unwrap();
        let sink = TestSink::new(true, false);
        let (mut d, outbox, dlq) = dispatcher(dir.path(), sink, 3);
        enqueue(&outbox, 1);

        for i in 1..=2 {
            let r = d.dispatch_once().await;
            assert!(r.failed && r.dead_lettered == 0, "attempt {i}");
            assert_eq!(outbox.pending_count(), 1);
            assert!(d.backoff() >= Duration::from_millis(1));
        }
        let r = d.dispatch_once().await;
        assert_eq!(r.dead_lettered, 1);
        assert_eq!(outbox.pending_count(), 0);
        assert_eq!(dlq.len(), 1);
        assert!(dlq.list()[0].1.contains("after 3 attempts"));
        assert!(d.backoff() <= Duration::from_millis(4));
    }

    #[tokio::test]
    async fn attempt_counts_survive_restart() {
        let dir = tempfile::tempdir().unwrap();
        {
            let (mut d, outbox, _dlq) = dispatcher(dir.path(), TestSink::new(true, false), 3);
            enqueue(&outbox, 1);
            for _ in 0..2 {
                assert!(d.dispatch_once().await.failed);
            }
            assert_eq!(outbox.attempts(0), 2);
        }
        let (mut d, outbox, dlq) = dispatcher(dir.path(), TestSink::new(true, false), 3);
        assert_eq!(outbox.attempts(0), 2, "carried across restart");
        let r = d.dispatch_once().await;
        assert_eq!(r.dead_lettered, 1, "third attempt hits the DLQ threshold");
        assert_eq!(dlq.len(), 1);
        assert_eq!(outbox.pending_count(), 0);
    }

    #[tokio::test]
    async fn unacked_events_are_redelivered_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        {
            let sink = TestSink::new(true, false); // never delivers
            let (mut d, outbox, _dlq) = dispatcher(dir.path(), sink, 100);
            enqueue(&outbox, 3);
            assert!(d.dispatch_once().await.failed);
            assert_eq!(outbox.pending_count(), 3);
        } // "crash": everything dropped without acks

        let sink = TestSink::new(false, false);
        let (mut d, outbox, dlq) = dispatcher(dir.path(), sink.clone(), 5);
        assert_eq!(outbox.pending_count(), 3, "journal replayed");
        let r = d.dispatch_once().await;
        assert_eq!(r.delivered, 3);
        let pks: Vec<u64> = sink.got.lock().unwrap().iter().map(|(p, _)| *p).collect();
        assert_eq!(pks, vec![0, 1, 2]);
        assert!(dlq.is_empty());

        // Acked events are not redelivered on the next restart.
        drop((d, outbox, dlq));
        let (_d, outbox, _dlq) = dispatcher(dir.path(), TestSink::new(false, false), 5);
        assert_eq!(outbox.pending_count(), 0);
    }

    #[tokio::test]
    async fn enqueue_sink_persists_to_outbox() {
        let dir = tempfile::tempdir().unwrap();
        let (op, _) = outbox_paths(dir.path());
        let outbox = Arc::new(EventOutbox::open(&op).unwrap());
        let reg = SinkRegistry::new();
        reg.register(Arc::new(EnqueueSink {
            outbox: Arc::clone(&outbox),
        }));
        reg.fan_out(SinkEvent::insert("t", 7, vec![1]))
            .await
            .unwrap();
        drop(reg);
        drop(outbox);
        let reopened = EventOutbox::open(&op).unwrap();
        assert_eq!(reopened.pending_count(), 1);
    }

    #[test]
    fn delivery_settings_parse_with_defaults() {
        let f = parse_sinks("sinks: []\n", false).unwrap();
        let s = OutboxSettings::from_file(&f);
        assert_eq!(s.max_attempts, 5);
        let f = parse_sinks("max_delivery_attempts: 2\nretry_backoff_ms: 50\n", false).unwrap();
        let s = OutboxSettings::from_file(&f);
        assert_eq!(
            (s.max_attempts, s.initial_backoff),
            (2, Duration::from_millis(50))
        );
    }
}
