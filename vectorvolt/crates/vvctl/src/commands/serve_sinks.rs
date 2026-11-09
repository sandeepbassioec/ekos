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
//! A [`WalSinkBridge`] per table then tails the WAL and fans mutations out.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::Deserialize;
use vectorvolt_events::{DomainEventEnvelope, EventSink, RedisStreamSink, config::RedisSinkConfig};
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

/// Connect the sinks, then start a background thread that tails each table's
/// WAL and fans mutations out. `tables` is re-evaluated every poll so tables
/// created after startup are picked up (multi-table mode).
///
/// Returns once every sink has connected; returns an error otherwise.
pub fn start(
    file: &SinksFile,
    tables: impl Fn() -> Vec<TableWal> + Send + 'static,
) -> Result<Arc<SinkRegistry>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("vv-sinks")
        .build()
        .context("sink runtime")?;
    let registry = rt.block_on(build_registry(file))?;
    let poll = Duration::from_millis(file.poll_interval_ms.max(10));
    let reg = Arc::clone(&registry);
    std::thread::Builder::new()
        .name("vv-sink-bridge".into())
        .spawn(move || {
            rt.block_on(async move {
                let mut bridges: std::collections::HashMap<String, WalSinkBridge> =
                    std::collections::HashMap::new();
                loop {
                    for (table, dir) in tables() {
                        bridges.entry(table.clone()).or_insert_with(|| {
                            let cfg = WalConfig::default().with_dir(&dir).with_fsync(false);
                            WalSinkBridge::new(cfg, Arc::clone(&reg), &table)
                        });
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
        let badc = "sinks:\n - {name: a, type: kafka, brokers: [b], topic: t, compression: lz4}\n";
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
        let err = start(&f, Vec::new).err().expect("must fail");
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
        let err = start(&f, Vec::new).err().expect("must fail");
        assert!(format!("{err:#}").contains("rd"), "{err:#}");

        let f = parse_sinks(
            "sinks:\n - {name: kf, type: kafka, brokers: ['127.0.0.1:1'], topic: t, connect_timeout_ms: 2000}\n",
            false,
        )
        .unwrap();
        let err = start(&f, Vec::new).err().expect("must fail");
        assert!(format!("{err:#}").contains("kf"), "{err:#}");
    }

    #[test]
    fn envelope_mapping() {
        let e = to_envelope(&SinkEvent::insert("t", 9, vec![1, 2]));
        assert_eq!(e.event_type, "vectorvolt.t.insert");
        assert_eq!(e.pk, 9);
        assert_eq!(e.payload["values"], serde_json::json!([1, 2]));
    }
}
