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
    /// At-rest key of an encrypted WAL (`--encrypt-at-rest`); frames are opened
    /// with the engine's own replay helper, so both sides decode identically.
    key: Option<Arc<vectorvolt_crypto::KeyStore>>,
    /// Durable cursor file (`<dir>/events/cursor/<table>`), if persisted.
    cursor_file: Option<std::path::PathBuf>,
}

impl WalSinkBridge {
    pub fn new(wal_config: WalConfig, registry: Arc<SinkRegistry>, table_name: &str) -> Self {
        Self {
            wal_config,
            registry,
            last_lsn: AtomicU64::new(0),
            cursor: Mutex::new(WalCursor::beginning()),
            table_name: table_name.to_string(),
            key: None,
            cursor_file: None,
        }
    }

    /// Persist the cursor at `path` (write-temp + fsync + rename) after each
    /// poll whose events were all dispatched, and resume from it if present.
    pub fn with_cursor_file(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        let path = path.into();
        if let Some(c) = load_cursor(&path) {
            *self.cursor.lock() = c;
        }
        self.cursor_file = Some(path);
        self
    }

    /// Decrypt frames of an encrypted WAL with `key` (same key the engine uses).
    pub fn with_key(mut self, key: Option<Arc<vectorvolt_crypto::KeyStore>>) -> Self {
        self.key = key;
        self
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
            while let Some(mut frame) = reader.next_frame().map_err(io)? {
                frame.payload = vectorvolt_txn::manager::open_wal_payload(
                    self.key.as_deref(),
                    &reader.header,
                    frame.sequence,
                    std::mem::take(&mut frame.payload),
                )
                .map_err(|e| crate::error::SinkError::Io(format!("wal frame decrypt: {e}")))?;
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
        if let Some(p) = &self.cursor_file
            && next_cursor != from
        {
            save_cursor(p, next_cursor)
                .map_err(|e| crate::error::SinkError::Io(format!("save cursor: {e}")))?;
        }
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

fn load_cursor(path: &std::path::Path) -> Option<WalCursor> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut it = text.split_whitespace();
    Some(WalCursor {
        segment_id: it.next()?.parse().ok()?,
        offset: it.next()?.parse().ok()?,
    })
}

fn save_cursor(path: &std::path::Path, c: WalCursor) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    {
        let mut f = std::fs::File::create(&tmp)?;
        write!(f, "{} {}", c.segment_id, c.offset)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)
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

    #[tokio::test]
    async fn persisted_cursor_prevents_duplicates_after_restart() {
        let dir = TempDir::new().unwrap();
        let (wal, cfg) = setup(&dir);
        let cur = dir.path().join("cursor").join("t");
        let append = |from: u64, n: u64| {
            for i in from..from + n {
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
        };
        append(0, 3);
        let mk = |cfg: WalConfig| {
            let reg = Arc::new(SinkRegistry::new());
            let (sink, count) = CountSink::new("s");
            reg.register(sink);
            (
                WalSinkBridge::new(cfg, reg, "t").with_cursor_file(cur.clone()),
                count,
            )
        };
        let (b1, c1) = mk(cfg.clone());
        assert_eq!(b1.poll_once().await.unwrap(), 3);
        assert_eq!(c1.load(Ordering::Relaxed), 3);
        drop(b1);

        // "Restart": fresh bridge resumes from the persisted cursor.
        let (b2, c2) = mk(cfg.clone());
        assert_eq!(b2.poll_once().await.unwrap(), 0, "no duplicates");
        append(3, 2);
        assert_eq!(b2.poll_once().await.unwrap(), 2);
        assert_eq!(c2.load(Ordering::Relaxed), 2);
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

    /// Records every event it receives (in-process test double).
    struct RecSink(parking_lot::Mutex<Vec<(u64, SinkEventKind, Vec<u64>)>>);

    #[async_trait]
    impl Sink for RecSink {
        fn name(&self) -> &str {
            "rec"
        }
        async fn send(&self, e: SinkEvent) -> SinkResult<()> {
            self.0.lock().push((e.pk, e.kind, e.values));
            Ok(())
        }
        async fn flush(&self) -> SinkResult<()> {
            Ok(())
        }
    }

    /// Write the same mutations to a plaintext WAL and to an encrypted one
    /// (sealed exactly as `TxnManager::commit` does) and tail both.
    #[tokio::test]
    async fn encrypted_wal_yields_same_events_as_plain() {
        use vectorvolt_crypto::KeyStore;

        let key = Arc::new(KeyStore::from_bytes([9u8; 32]).unwrap());
        let muts = [
            Mutation::Insert {
                pk: 1,
                values: vec![10, 20],
            },
            Mutation::Delete { pk: 1 },
            Mutation::InsertBatch {
                rows: vec![(2, vec![3]), (3, vec![4])],
            },
        ];

        let plain_dir = TempDir::new().unwrap();
        let (pw, pcfg) = setup(&plain_dir);
        for m in &muts {
            pw.append(m.encode()).unwrap();
        }
        pw.flush().unwrap();

        let enc_dir = TempDir::new().unwrap();
        let ecfg = WalConfig::default()
            .with_dir(enc_dir.path())
            .with_fsync(false)
            .with_encryption_key_id(Some(key.key_id()));
        let ew = WalWriter::open(ecfg.clone()).unwrap();
        for m in &muts {
            let payload = m.encode();
            ew.append_with(|seq| {
                let aad = vectorvolt_wal::segment::frame_aad(&key.key_id(), seq);
                vectorvolt_crypto::encryptor::encrypt_with_aad(&key, &payload, &aad)
                    .map_err(|e| vectorvolt_wal::WalError::Io(std::io::Error::other(e.to_string())))
            })
            .unwrap();
        }
        ew.flush().unwrap();

        let tail = |cfg: WalConfig, k: Option<Arc<vectorvolt_crypto::KeyStore>>| async move {
            let reg = Arc::new(SinkRegistry::new());
            let rec = Arc::new(RecSink(parking_lot::Mutex::new(Vec::new())));
            reg.register(rec.clone());
            let bridge = WalSinkBridge::new(cfg, reg, "t").with_key(k);
            bridge.poll_once().await.unwrap();
            rec.0.lock().clone()
        };
        let plain = tail(pcfg, None).await;
        let enc = tail(ecfg.clone(), Some(key.clone())).await;
        assert_eq!(plain.len(), 4);
        assert_eq!(plain, enc);

        // No key / wrong key must error, not silently skip.
        let reg = Arc::new(SinkRegistry::new());
        assert!(
            WalSinkBridge::new(ecfg.clone(), reg.clone(), "t")
                .poll_once()
                .await
                .is_err()
        );
        let wrong = Arc::new(KeyStore::from_bytes([1u8; 32]).unwrap());
        assert!(
            WalSinkBridge::new(ecfg, reg, "t")
                .with_key(Some(wrong))
                .poll_once()
                .await
                .is_err()
        );
    }
}
