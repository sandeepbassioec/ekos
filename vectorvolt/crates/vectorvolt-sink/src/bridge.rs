//! [`WalSinkBridge`] — tails the WAL and fans decoded mutations to sinks (VV-SNK-041).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::Mutex;
use vectorvolt_txn::Mutation;
use vectorvolt_wal::{Frame, SegmentReader, WalConfig, WalCursor, list_segment_ids, segment_path};

use crate::{
    error::SinkResult,
    registry::SinkRegistry,
    sink::{SinkEvent, SinkEventKind},
};

/// Reads new WAL frames since the last poll and fans decoded mutations to the
/// attached [`SinkRegistry`].
///
/// The bridge keeps a byte-level [`WalCursor`] (segment + offset just past the
/// last frame it has dispatched), so each poll reads only frames appended since
/// the previous one instead of re-reading the whole WAL (B29). `last_lsn` is
/// still checked per frame as a second guard against re-emission.
pub struct WalSinkBridge {
    wal_config: WalConfig,
    registry: Arc<SinkRegistry>,
    last_lsn: AtomicU64,
    /// Where the next poll resumes reading. Advanced only after every frame
    /// up to it was dispatched successfully (at-least-once on sink errors).
    cursor: Mutex<WalCursor>,
    table_name: String,
}

impl WalSinkBridge {
    pub fn new(wal_config: WalConfig, registry: Arc<SinkRegistry>, table_name: &str) -> Self {
        Self {
            wal_config,
            registry,
            last_lsn: AtomicU64::new(0),
            cursor: Mutex::new(WalCursor::beginning()),
            table_name: table_name.to_string(),
        }
    }

    /// Read every complete frame at or after `from`, returning them and the
    /// cursor just past the last one read.
    ///
    /// Unlike `WalReader::scan`, whose end-of-WAL cursor points *past* the
    /// last segment (so resuming from it would skip frames later appended to
    /// that still-active segment), the returned cursor stays inside the last
    /// segment, at its current end.
    fn read_from(&self, from: WalCursor) -> SinkResult<(Vec<Frame>, WalCursor)> {
        let io = |e: vectorvolt_wal::WalError| crate::error::SinkError::Io(e.to_string());
        let ids = list_segment_ids(&self.wal_config.dir).map_err(io)?;
        let start_idx = ids.partition_point(|&id| id < from.segment_id);

        let mut frames = Vec::new();
        let mut cur = from;
        for &seg_id in &ids[start_idx..] {
            let mut reader =
                SegmentReader::open(&segment_path(&self.wal_config.dir, seg_id)).map_err(io)?;
            if seg_id == from.segment_id {
                reader.seek_to(from.offset).map_err(io)?;
            } else {
                reader.rewind().map_err(io)?;
            }
            while let Some(frame) = reader.next_frame().map_err(io)? {
                frames.push(frame);
            }
            cur = WalCursor {
                segment_id: seg_id,
                offset: reader.file_position().map_err(io)?,
            };
        }
        Ok((frames, cur))
    }

    /// Scan frames appended since the last poll, decode mutations, and fan
    /// events to the registry.
    pub async fn poll_once(&self) -> SinkResult<usize> {
        let last = self.last_lsn.load(Ordering::Acquire);
        let from = *self.cursor.lock();

        let mut dispatched = 0usize;
        let mut max_seq = last;

        let (frames, next_cursor) = self.read_from(from)?;

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
        *self.cursor.lock() = next_cursor;
        Ok(dispatched)
    }

    /// Position the next poll resumes reading from.
    pub fn cursor(&self) -> WalCursor {
        *self.cursor.lock()
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
        Mutation::TxnBatch { ops } => ops
            .iter()
            .map(|(pk, op)| match op {
                Some(values) => SinkEvent {
                    table: table.to_string(),
                    kind: SinkEventKind::Update,
                    pk: *pk,
                    values: values.clone(),
                },
                None => SinkEvent {
                    table: table.to_string(),
                    kind: SinkEventKind::Delete,
                    pk: *pk,
                    values: vec![],
                },
            })
            .collect(),
        Mutation::DdlCreate { .. }
        | Mutation::DdlDrop { .. }
        | Mutation::DdlAlter { .. }
        | Mutation::Checkpoint { .. }
        | Mutation::Update { .. } => vec![],
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

    /// B29: a second poll must neither re-emit old events nor re-read old
    /// frames. The already-consumed frames are overwritten with garbage after
    /// the first poll: a full re-read would fail to decode them, a resumed
    /// read never touches them.
    #[tokio::test]
    async fn second_poll_resumes_from_cursor() {
        use std::io::{Seek, SeekFrom, Write};

        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        for i in 0..3u64 {
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

        let reg = Arc::new(SinkRegistry::new());
        let (sink, count) = CountSink::new("s");
        reg.register(sink);
        let bridge = WalSinkBridge::new(cfg.clone(), reg, "t");
        assert_eq!(bridge.poll_once().await.unwrap(), 3);
        let after_first = bridge.cursor();

        // Nothing new: nothing re-emitted, cursor unchanged.
        assert_eq!(bridge.poll_once().await.unwrap(), 0);
        assert_eq!(count.load(Ordering::Relaxed), 3);
        assert_eq!(bridge.cursor(), after_first);

        // Corrupt the consumed frames in place.
        let seg = vectorvolt_wal::segment_path(dir.path(), after_first.segment_id);
        let header = vectorvolt_wal::WalCursor::beginning().offset;
        let mut f = std::fs::OpenOptions::new().write(true).open(&seg).unwrap();
        f.seek(SeekFrom::Start(header)).unwrap();
        f.write_all(&vec![0xAB; (after_first.offset - header) as usize])
            .unwrap();
        f.sync_all().unwrap();
        drop(f);
        assert!(
            vectorvolt_wal::WalReader::open(cfg.clone())
                .read_all()
                .is_err(),
            "precondition: a full re-read must now fail"
        );

        // New frames are picked up from the cursor, old ones never re-read.
        for i in 10..12u64 {
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
        assert_eq!(bridge.poll_once().await.unwrap(), 2);
        assert_eq!(count.load(Ordering::Relaxed), 5);
        assert_eq!(bridge.last_lsn(), 5);
        assert!(bridge.cursor().offset > after_first.offset);
    }

    #[tokio::test]
    async fn cursor_follows_segment_rotation() {
        let dir = TempDir::new().unwrap();
        let cfg = WalConfig::default()
            .with_dir(dir.path())
            .with_fsync(false)
            .with_segment_max_bytes(256);
        let wal = WalWriter::open(cfg.clone()).unwrap();
        let reg = Arc::new(SinkRegistry::new());
        let (sink, count) = CountSink::new("s");
        reg.register(sink);
        let bridge = WalSinkBridge::new(cfg, reg, "t");

        let mut expected = 0u64;
        for round in 0..4u64 {
            for i in 0..5u64 {
                let pk = round * 100 + i;
                wal.append(
                    Mutation::Insert {
                        pk,
                        values: vec![pk; 4],
                    }
                    .encode(),
                )
                .unwrap();
            }
            wal.flush().unwrap();
            expected += 5;
            assert_eq!(bridge.poll_once().await.unwrap(), 5, "round {round}");
            assert_eq!(count.load(Ordering::Relaxed), expected);
        }
        assert!(bridge.cursor().segment_id > 0, "test must cross a rotation");
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
