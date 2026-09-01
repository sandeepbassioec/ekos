//! [`WalSinkBridge`] — tails the WAL and fans decoded mutations to sinks (VV-SNK-041).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use vectorvolt_txn::Mutation;
use vectorvolt_wal::{WalConfig, WalReader};

use crate::{
    error::SinkResult,
    registry::SinkRegistry,
    sink::{SinkEvent, SinkEventKind},
};

/// Reads new WAL frames since the last poll and fans decoded mutations to the
/// attached [`SinkRegistry`].
pub struct WalSinkBridge {
    wal_config: WalConfig,
    registry: Arc<SinkRegistry>,
    last_lsn: AtomicU64,
    table_name: String,
}

impl WalSinkBridge {
    pub fn new(wal_config: WalConfig, registry: Arc<SinkRegistry>, table_name: &str) -> Self {
        Self {
            wal_config,
            registry,
            last_lsn: AtomicU64::new(0),
            table_name: table_name.to_string(),
        }
    }

    /// Scan new frames, decode mutations, and fan events to the registry.
    pub async fn poll_once(&self) -> SinkResult<usize> {
        let reader = WalReader::open(self.wal_config.clone());
        let last = self.last_lsn.load(Ordering::Acquire);

        let mut dispatched = 0usize;
        let mut max_seq = last;

        let frames = reader
            .read_all()
            .map_err(|e| crate::error::SinkError::Io(e.to_string()))?;

        for frame in frames {
            if frame.sequence > last {
                if frame.sequence > max_seq {
                    max_seq = frame.sequence;
                }
                // Try to decode as a Mutation; skip frames that aren't mutations
                // (e.g. snapshot markers, checkpoint frames).
                if let Ok(mutation) = Mutation::decode(&frame.payload) {
                    let events = mutation_to_events(&self.table_name, &mutation);
                    for event in events {
                        self.registry.fan_out(event).await?;
                        dispatched += 1;
                    }
                }
            }
        }

        self.last_lsn.store(max_seq, Ordering::Release);
        Ok(dispatched)
    }

    pub fn last_lsn(&self) -> u64 {
        self.last_lsn.load(Ordering::Acquire)
    }
}

fn mutation_to_events(table: &str, mutation: &Mutation) -> Vec<SinkEvent> {
    match mutation {
        Mutation::Insert { pk, values } => {
            vec![SinkEvent {
                table: table.to_string(),
                kind: SinkEventKind::Insert,
                pk: *pk,
                values: values.clone(),
            }]
        }
        Mutation::Upsert { pk, values } => {
            vec![SinkEvent {
                table: table.to_string(),
                kind: SinkEventKind::Update,
                pk: *pk,
                values: values.clone(),
            }]
        }
        Mutation::Delete { pk } => {
            vec![SinkEvent {
                table: table.to_string(),
                kind: SinkEventKind::Delete,
                pk: *pk,
                values: vec![],
            }]
        }
        Mutation::InsertBatch { rows } => rows
            .iter()
            .map(|(pk, values)| SinkEvent {
                table: table.to_string(),
                kind: SinkEventKind::Insert,
                pk: *pk,
                values: values.clone(),
            })
            .collect(),
        Mutation::DeleteBatch { pks } => pks
            .iter()
            .map(|&pk| SinkEvent {
                table: table.to_string(),
                kind: SinkEventKind::Delete,
                pk,
                values: vec![],
            })
            .collect(),
        Mutation::UpsertBatch { rows } => rows
            .iter()
            .map(|(pk, values)| SinkEvent {
                table: table.to_string(),
                kind: SinkEventKind::Update,
                pk: *pk,
                values: values.clone(),
            })
            .collect(),
        Mutation::DdlCreate { .. } | Mutation::DdlDrop { .. } | Mutation::DdlAlter { .. } => vec![],
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SinkResult;
    use crate::sink::{Sink, SinkEvent};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tempfile::TempDir;
    use vectorvolt_txn::Mutation;
    use vectorvolt_wal::{WalConfig, WalWriter};

    struct CountSink {
        name: String,
        count: Arc<AtomicU64>,
    }

    impl CountSink {
        fn new(name: &str) -> (Arc<Self>, Arc<AtomicU64>) {
            let c = Arc::new(AtomicU64::new(0));
            (
                Arc::new(Self {
                    name: name.to_string(),
                    count: c.clone(),
                }),
                c,
            )
        }
    }

    #[async_trait]
    impl Sink for CountSink {
        fn name(&self) -> &str {
            &self.name
        }

        async fn send(&self, _: SinkEvent) -> SinkResult<()> {
            self.count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        async fn flush(&self) -> SinkResult<()> {
            Ok(())
        }
    }

    fn setup(dir: &TempDir) -> (WalWriter, WalConfig) {
        let cfg = WalConfig::default().with_dir(dir.path()).with_fsync(false);
        let w = WalWriter::open(cfg.clone()).unwrap();
        (w, cfg)
    }

    #[tokio::test]
    async fn poll_empty_wal_zero_dispatched() {
        let dir = TempDir::new().unwrap();
        let (_, cfg) = setup(&dir);
        let reg = Arc::new(SinkRegistry::new());
        let bridge = WalSinkBridge::new(cfg, reg, "t");
        assert_eq!(bridge.poll_once().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn poll_insert_dispatched() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        let mutation = Mutation::Insert {
            pk: 1,
            values: vec![10, 20],
        };
        wal.append(mutation.encode()).unwrap();
        wal.flush().unwrap();
        drop(wal);

        let reg = Arc::new(SinkRegistry::new());
        let (sink, count) = CountSink::new("s");
        reg.register(sink);
        let bridge = WalSinkBridge::new(cfg, reg, "t");
        let dispatched = bridge.poll_once().await.unwrap();
        assert_eq!(dispatched, 1);
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn poll_skips_already_seen() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        let mutation = Mutation::Insert {
            pk: 1,
            values: vec![10],
        };
        wal.append(mutation.encode()).unwrap();
        wal.flush().unwrap();
        drop(wal);

        let reg = Arc::new(SinkRegistry::new());
        let (sink, count) = CountSink::new("s");
        reg.register(sink);
        let bridge = WalSinkBridge::new(cfg.clone(), reg.clone(), "t");
        bridge.poll_once().await.unwrap();

        // Second poll: WAL unchanged, no new events
        let wal2 = WalWriter::open(cfg.clone()).unwrap();
        drop(wal2);
        let n = bridge.poll_once().await.unwrap();
        assert_eq!(n, 0);
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn poll_multiple_inserts() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        for i in 0..5u64 {
            wal.append(
                Mutation::Insert {
                    pk: i,
                    values: vec![i],
                }
                .encode(),
            )
            .unwrap();
        }
        wal.flush().unwrap();
        drop(wal);

        let reg = Arc::new(SinkRegistry::new());
        let (sink, count) = CountSink::new("s");
        reg.register(sink);
        let bridge = WalSinkBridge::new(cfg, reg, "t");
        assert_eq!(bridge.poll_once().await.unwrap(), 5);
        assert_eq!(count.load(Ordering::Relaxed), 5);
    }

    #[tokio::test]
    async fn poll_delete_dispatched() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        wal.append(Mutation::Delete { pk: 99 }.encode()).unwrap();
        wal.flush().unwrap();
        drop(wal);

        let reg = Arc::new(SinkRegistry::new());
        let (sink, count) = CountSink::new("s");
        reg.register(sink);
        let bridge = WalSinkBridge::new(cfg, reg, "t");
        bridge.poll_once().await.unwrap();
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn last_lsn_advances() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        wal.append(
            Mutation::Insert {
                pk: 1,
                values: vec![1],
            }
            .encode(),
        )
        .unwrap();
        wal.flush().unwrap();
        drop(wal);

        let reg = Arc::new(SinkRegistry::new());
        let bridge = WalSinkBridge::new(cfg, reg, "t");
        assert_eq!(bridge.last_lsn(), 0);
        bridge.poll_once().await.unwrap();
        assert!(bridge.last_lsn() > 0);
    }

    #[tokio::test]
    async fn batch_insert_dispatches_all() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        let rows: Vec<(u64, Vec<u64>)> = (0..3u64).map(|i| (i, vec![i])).collect();
        wal.append(Mutation::InsertBatch { rows }.encode()).unwrap();
        wal.flush().unwrap();
        drop(wal);

        let reg = Arc::new(SinkRegistry::new());
        let (sink, count) = CountSink::new("s");
        reg.register(sink);
        let bridge = WalSinkBridge::new(cfg, reg, "t");
        assert_eq!(bridge.poll_once().await.unwrap(), 3);
        assert_eq!(count.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn ddl_frames_not_dispatched() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        wal.append(
            Mutation::DdlCreate {
                table_name: "foo".into(),
            }
            .encode(),
        )
        .unwrap();
        wal.flush().unwrap();
        drop(wal);

        let reg = Arc::new(SinkRegistry::new());
        let (sink, count) = CountSink::new("s");
        reg.register(sink);
        let bridge = WalSinkBridge::new(cfg, reg, "t");
        bridge.poll_once().await.unwrap();
        assert_eq!(count.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn no_sinks_poll_ok() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        wal.append(
            Mutation::Insert {
                pk: 1,
                values: vec![1],
            }
            .encode(),
        )
        .unwrap();
        wal.flush().unwrap();
        drop(wal);

        let reg = Arc::new(SinkRegistry::new()); // no sinks
        let bridge = WalSinkBridge::new(cfg, reg, "t");
        // Should succeed even with no sinks registered
        bridge.poll_once().await.unwrap();
    }

    #[tokio::test]
    async fn upsert_dispatched_as_update() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        wal.append(
            Mutation::Upsert {
                pk: 5,
                values: vec![50],
            }
            .encode(),
        )
        .unwrap();
        wal.flush().unwrap();
        drop(wal);

        let reg = Arc::new(SinkRegistry::new());
        let (sink, count) = CountSink::new("s");
        reg.register(sink);
        let bridge = WalSinkBridge::new(cfg, reg, "t");
        assert_eq!(bridge.poll_once().await.unwrap(), 1);
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn delete_batch_dispatches_all() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        wal.append(Mutation::DeleteBatch { pks: vec![1, 2, 3] }.encode())
            .unwrap();
        wal.flush().unwrap();
        drop(wal);

        let reg = Arc::new(SinkRegistry::new());
        let (sink, count) = CountSink::new("s");
        reg.register(sink);
        let bridge = WalSinkBridge::new(cfg, reg, "t");
        assert_eq!(bridge.poll_once().await.unwrap(), 3);
        assert_eq!(count.load(Ordering::Relaxed), 3);
    }
}
