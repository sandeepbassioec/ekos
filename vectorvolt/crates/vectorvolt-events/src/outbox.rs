//! EventOutbox (at-least-once) + EventDlq (dead-letter queue). VV-EVT-008, VV-EVT-012.
//!
//! Both can run purely in memory (`new()`) or be backed by a durable
//! append-only journal (`open(path)`): one JSON record per line, each append
//! `fsync`ed before the call returns. On open the journal is replayed, so
//! pending outbox entries and DLQ items survive a restart, and then compacted
//! (atomically rewritten with live records only). Unparseable lines (e.g. a
//! torn final line from a crash mid-append) are skipped.

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::envelope::DomainEventEnvelope;
use crate::sink::EventSink;

// ── Journal ──────────────────────────────────────────────────────────────────

/// fsynced JSON-lines append log.
struct Journal {
    path: PathBuf,
    file: File,
}

impl Journal {
    /// Read every parseable record from `path` (missing file = empty).
    fn read_all<R: DeserializeOwned>(path: &Path) -> io::Result<Vec<R>> {
        let f = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        let mut out = Vec::new();
        for line in BufReader::new(f).split(b'\n') {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            match serde_json::from_slice::<R>(&line) {
                Ok(r) => out.push(r),
                Err(e) => warn!("event journal {}: skipping bad record: {e}", path.display()),
            }
        }
        Ok(out)
    }

    /// Atomically replace the journal with `records` and open it for append.
    fn rewrite<R: Serialize>(path: &Path, records: &[R]) -> io::Result<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("compact.tmp");
        {
            let mut f = File::create(&tmp)?;
            for r in records {
                let mut line = serde_json::to_vec(r).map_err(io::Error::other)?;
                line.push(b'\n');
                f.write_all(&line)?;
            }
            f.sync_all()?;
        }
        std::fs::rename(&tmp, path)?;
        let file = OpenOptions::new().append(true).open(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
        })
    }

    /// Append one record and fsync it.
    fn append<R: Serialize>(&mut self, rec: &R) -> io::Result<()> {
        let mut line = serde_json::to_vec(rec).map_err(io::Error::other)?;
        line.push(b'\n');
        self.file.write_all(&line)?;
        self.file.sync_data()
    }
}

// ── EventOutbox ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum OutboxRec {
    Push { seq: u64, env: DomainEventEnvelope },
    Ack { seq: u64 },
}

struct OutboxInner {
    pending: VecDeque<(u64, DomainEventEnvelope)>,
    next_seq: u64,
    journal: Option<Journal>,
}

/// Transactional outbox — pending envelopes written before broker delivery (VV-EVT-008).
///
/// With [`EventOutbox::open`] every push is fsynced to a journal before it is
/// queued, and every delivery (or hand-off to the DLQ) is recorded, so
/// undelivered entries are redelivered after a restart (at-least-once).
pub struct EventOutbox {
    inner: Mutex<OutboxInner>,
}

impl EventOutbox {
    /// In-memory outbox (lost on restart).
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(OutboxInner {
                pending: VecDeque::new(),
                next_seq: 0,
                journal: None,
            }),
        }
    }

    /// Durable outbox journaled at `path`; replays pending entries from it.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let mut pending: VecDeque<(u64, DomainEventEnvelope)> = VecDeque::new();
        let mut next_seq = 0u64;
        for rec in Journal::read_all::<OutboxRec>(path)? {
            match rec {
                OutboxRec::Push { seq, env } => {
                    next_seq = next_seq.max(seq + 1);
                    pending.push_back((seq, env));
                }
                OutboxRec::Ack { seq } => {
                    next_seq = next_seq.max(seq + 1);
                    pending.retain(|(s, _)| *s != seq);
                }
            }
        }
        let live: Vec<OutboxRec> = pending
            .iter()
            .map(|(seq, env)| OutboxRec::Push {
                seq: *seq,
                env: env.clone(),
            })
            .collect();
        let journal = Journal::rewrite(path, &live)?;
        Ok(Self {
            inner: Mutex::new(OutboxInner {
                pending,
                next_seq,
                journal: Some(journal),
            }),
        })
    }

    /// Append an envelope to the outbox, durably if journaled. On a journal
    /// I/O error the envelope is NOT queued and the error is returned.
    pub fn try_push(&self, envelope: DomainEventEnvelope) -> io::Result<()> {
        let mut g = self.inner.lock().unwrap();
        let seq = g.next_seq;
        if let Some(j) = g.journal.as_mut() {
            j.append(&OutboxRec::Push {
                seq,
                env: envelope.clone(),
            })?;
        }
        g.next_seq += 1;
        g.pending.push_back((seq, envelope));
        Ok(())
    }

    /// Append an envelope to the outbox (called in the commit path).
    ///
    /// If the journal write fails the envelope is still queued in memory (so
    /// it is delivered this run) and the failure is logged; use
    /// [`EventOutbox::try_push`] to observe it.
    pub fn push(&self, envelope: DomainEventEnvelope) {
        let mut g = self.inner.lock().unwrap();
        let seq = g.next_seq;
        g.next_seq += 1;
        if let Some(j) = g.journal.as_mut()
            && let Err(e) = j.append(&OutboxRec::Push {
                seq,
                env: envelope.clone(),
            })
        {
            warn!("outbox journal append failed, queuing in memory only: {e}");
        }
        g.pending.push_back((seq, envelope));
    }

    /// Flush pending envelopes to the given sink, moving failures to DLQ.
    pub fn flush(&self, sink: &dyn EventSink, dlq: &EventDlq) {
        let mut g = self.inner.lock().unwrap();
        while let Some((seq, env)) = g.pending.pop_front() {
            if let Err(e) = sink.send(&env) {
                warn!("outbox flush failed: {e}");
                // The DLQ entry is made durable before the outbox entry is acked.
                dlq.push(env, e.to_string());
            }
            if let Some(j) = g.journal.as_mut()
                && let Err(e) = j.append(&OutboxRec::Ack { seq })
            {
                warn!("outbox journal ack failed (entry may be redelivered): {e}");
            }
        }
    }

    pub fn pending_count(&self) -> usize {
        self.inner.lock().unwrap().pending.len()
    }
}

impl Default for EventOutbox {
    fn default() -> Self {
        Self::new()
    }
}

// ── EventDlq ─────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum DlqRec {
    Add {
        seq: u64,
        env: DomainEventEnvelope,
        error: String,
    },
    Del {
        seq: u64,
    },
    Clear,
}

struct DlqInner {
    entries: Vec<(u64, DlqEntry)>,
    next_seq: u64,
    journal: Option<Journal>,
}

/// Dead-letter queue for failed deliveries (VV-EVT-012).
///
/// [`EventDlq::open`] makes it durable (same journal format as the outbox).
pub struct EventDlq {
    inner: Mutex<DlqInner>,
}

pub struct DlqEntry {
    pub envelope: DomainEventEnvelope,
    pub error: String,
}

impl EventDlq {
    /// In-memory DLQ (lost on restart).
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(DlqInner {
                entries: Vec::new(),
                next_seq: 0,
                journal: None,
            }),
        }
    }

    /// Durable DLQ journaled at `path`; replays surviving entries from it.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let mut entries: Vec<(u64, DlqEntry)> = Vec::new();
        let mut next_seq = 0u64;
        for rec in Journal::read_all::<DlqRec>(path)? {
            match rec {
                DlqRec::Add { seq, env, error } => {
                    next_seq = next_seq.max(seq + 1);
                    entries.push((
                        seq,
                        DlqEntry {
                            envelope: env,
                            error,
                        },
                    ));
                }
                DlqRec::Del { seq } => {
                    next_seq = next_seq.max(seq + 1);
                    entries.retain(|(s, _)| *s != seq);
                }
                DlqRec::Clear => entries.clear(),
            }
        }
        let live: Vec<DlqRec> = entries
            .iter()
            .map(|(seq, e)| DlqRec::Add {
                seq: *seq,
                env: e.envelope.clone(),
                error: e.error.clone(),
            })
            .collect();
        let journal = Journal::rewrite(path, &live)?;
        Ok(Self {
            inner: Mutex::new(DlqInner {
                entries,
                next_seq,
                journal: Some(journal),
            }),
        })
    }

    pub fn push(&self, envelope: DomainEventEnvelope, error: String) {
        let mut g = self.inner.lock().unwrap();
        let seq = g.next_seq;
        g.next_seq += 1;
        if let Some(j) = g.journal.as_mut()
            && let Err(e) = j.append(&DlqRec::Add {
                seq,
                env: envelope.clone(),
                error: error.clone(),
            })
        {
            warn!("DLQ journal append failed (entry not durable): {e}");
        }
        g.entries.push((seq, DlqEntry { envelope, error }));
    }

    /// Returns (event_type, error) pairs for `vvctl events dlq list`.
    pub fn list(&self) -> Vec<(String, String)> {
        self.inner
            .lock()
            .unwrap()
            .entries
            .iter()
            .map(|(_, e)| (e.envelope.event_type.clone(), e.error.clone()))
            .collect()
    }

    /// Replay all DLQ entries back through a sink, removing successfully delivered ones.
    pub fn replay(&self, sink: &dyn EventSink) {
        let mut g = self.inner.lock().unwrap();
        let mut delivered = Vec::new();
        g.entries.retain(|(seq, e)| {
            if sink.send(&e.envelope).is_err() {
                true
            } else {
                delivered.push(*seq);
                false
            }
        });
        if let Some(j) = g.journal.as_mut() {
            for seq in delivered {
                if let Err(e) = j.append(&DlqRec::Del { seq }) {
                    warn!("DLQ journal delete failed (entry may reappear): {e}");
                }
            }
        }
    }

    /// Drop all DLQ entries.
    pub fn drop_all(&self) {
        let mut g = self.inner.lock().unwrap();
        g.entries.clear();
        if let Some(j) = g.journal.as_mut()
            && let Err(e) = j.append(&DlqRec::Clear)
        {
            warn!("DLQ journal clear failed ({}): {e}", j.path.display());
        }
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().entries.is_empty()
    }
}

impl Default for EventDlq {
    fn default() -> Self {
        Self::new()
    }
}
