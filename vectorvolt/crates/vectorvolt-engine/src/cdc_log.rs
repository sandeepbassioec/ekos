//! Persistent CDC event log — Layer 67 + compaction (Layer 68).
//!
//! Every [`ChangeEvent`] is appended to a per-table binary log file
//! (`cdc.log`) as it is published.  The log is append-only and survives
//! process restarts, enabling two capabilities:
//!
//! - **SSE replay**: `GET /tables/{t}/subscribe?from=<lsn>` replays stored
//!   events before switching to the live broadcast stream.
//! - **View backfill**: `POST /views` with `"backfill": true` replays the
//!   log into a newly created materialized view so its initial state reflects
//!   all committed mutations, not just future ones.
//!
//! ## Layer 68 additions
//!
//! - [`CdcLog::stats`] — count events, measure on-disk size, find min/max LSN
//!   without parsing JSON bodies (header-only scan).
//! - [`CdcLog::compact`] — rewrite the log keeping only events with
//!   `lsn >= retain_lsn`, atomically replacing the original file.
//!
//! ## Wire format
//!
//! ```text
//! ┌─────────────┬───────────┬──────────────────┐
//! │  lsn: u64LE │ len: u32LE│  json: [u8; len] │
//! └─────────────┴───────────┴──────────────────┘
//! ```
//!
//! The LSN is redundant (it is also embedded in the [`ChangeEvent`] JSON) but
//! placing it first lets a reader skip entries cheaply without parsing JSON.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::cdc::ChangeEvent;

// ── stats ─────────────────────────────────────────────────────────────────────

/// Summary statistics for a CDC log file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdcLogStats {
    /// Total number of events in the log.
    pub event_count: u64,
    /// On-disk byte size of the log file.
    pub byte_size: u64,
    /// Lowest LSN in the log, or `0` when empty.
    pub min_lsn: u64,
    /// Highest LSN in the log, or `0` when empty.
    pub max_lsn: u64,
}

// ── log ───────────────────────────────────────────────────────────────────────

/// Append-only log of [`ChangeEvent`]s for one table.
pub struct CdcLog {
    path: PathBuf,
    writer: Mutex<std::io::BufWriter<File>>,
}

impl CdcLog {
    /// Open or create the log at `path`.
    pub fn open(path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            path,
            writer: Mutex::new(std::io::BufWriter::new(file)),
        })
    }

    /// Append a [`ChangeEvent`] to the log.  The write is flushed immediately
    /// so the entry is visible to concurrent readers.
    pub fn append(&self, event: &ChangeEvent) -> std::io::Result<()> {
        let json = serde_json::to_vec(event).map_err(std::io::Error::other)?;
        let mut w = self.writer.lock();
        w.write_all(&event.lsn.to_le_bytes())?;
        w.write_all(&(json.len() as u32).to_le_bytes())?;
        w.write_all(&json)?;
        w.flush()
    }

    /// Return all events with `lsn >= from_lsn`, in order.
    pub fn read_from(&self, from_lsn: u64) -> std::io::Result<Vec<ChangeEvent>> {
        let mut file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e),
        };
        let mut out = Vec::new();
        let mut lsn_buf = [0u8; 8];
        let mut len_buf = [0u8; 4];
        loop {
            match file.read_exact(&mut lsn_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            if file.read_exact(&mut len_buf).is_err() {
                break;
            }
            let lsn = u64::from_le_bytes(lsn_buf);
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut json = vec![0u8; len];
            if file.read_exact(&mut json).is_err() {
                break;
            }
            if lsn >= from_lsn
                && let Ok(ev) = serde_json::from_slice::<ChangeEvent>(&json)
            {
                out.push(ev);
            }
        }
        Ok(out)
    }

    /// Scan the log and return the highest LSN seen, or `0` if the log is
    /// empty.  Used to seed [`TableEngine`]'s LSN counter on restart.
    pub fn max_lsn(&self) -> std::io::Result<u64> {
        let mut file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e),
        };
        let mut max = 0u64;
        let mut lsn_buf = [0u8; 8];
        let mut len_buf = [0u8; 4];
        loop {
            match file.read_exact(&mut lsn_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            if file.read_exact(&mut len_buf).is_err() {
                break;
            }
            let lsn = u64::from_le_bytes(lsn_buf);
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut skip = vec![0u8; len];
            if file.read_exact(&mut skip).is_err() {
                break;
            }
            if lsn > max {
                max = lsn;
            }
        }
        Ok(max)
    }

    // ── Layer 68: stats + compaction ──────────────────────────────────────────

    /// Scan the log headers (no JSON parsing) and return summary statistics.
    pub fn stats(&self) -> std::io::Result<CdcLogStats> {
        let byte_size = match std::fs::metadata(&self.path) {
            Ok(m) => m.len(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(CdcLogStats {
                    event_count: 0,
                    byte_size: 0,
                    min_lsn: 0,
                    max_lsn: 0,
                });
            }
            Err(e) => return Err(e),
        };
        let mut file = File::open(&self.path)?;
        let mut event_count = 0u64;
        let mut min_lsn = u64::MAX;
        let mut max_lsn = 0u64;
        let mut lsn_buf = [0u8; 8];
        let mut len_buf = [0u8; 4];
        loop {
            match file.read_exact(&mut lsn_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            if file.read_exact(&mut len_buf).is_err() {
                break;
            }
            let lsn = u64::from_le_bytes(lsn_buf);
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut skip = vec![0u8; len];
            if file.read_exact(&mut skip).is_err() {
                break;
            }
            event_count += 1;
            if lsn < min_lsn {
                min_lsn = lsn;
            }
            if lsn > max_lsn {
                max_lsn = lsn;
            }
        }
        if event_count == 0 {
            min_lsn = 0;
        }
        Ok(CdcLogStats {
            event_count,
            byte_size,
            min_lsn,
            max_lsn,
        })
    }

    /// Rewrite the log retaining only events with `lsn >= retain_lsn`.
    ///
    /// The rewrite is atomic: events are written to a temp file adjacent to the
    /// log, then the temp file is renamed over the original.  Returns the number
    /// of events **removed** (i.e. older than `retain_lsn`).
    ///
    /// The writer lock is held for the entire operation so no concurrent
    /// append can interleave with the rewrite.
    pub fn compact(&self, retain_lsn: u64) -> std::io::Result<usize> {
        let mut _guard = self.writer.lock();

        // Read all events from the existing log.
        let mut file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e),
        };

        let mut kept: Vec<Vec<u8>> = Vec::new();
        let mut dropped = 0usize;
        let mut lsn_buf = [0u8; 8];
        let mut len_buf = [0u8; 4];
        loop {
            match file.read_exact(&mut lsn_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            if file.read_exact(&mut len_buf).is_err() {
                break;
            }
            let lsn = u64::from_le_bytes(lsn_buf);
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut json = vec![0u8; len];
            if file.read_exact(&mut json).is_err() {
                break;
            }
            if lsn >= retain_lsn {
                // Reconstruct the full frame to write to the new file.
                let mut frame = Vec::with_capacity(12 + len);
                frame.extend_from_slice(&lsn_buf);
                frame.extend_from_slice(&len_buf);
                frame.extend_from_slice(&json);
                kept.push(frame);
            } else {
                dropped += 1;
            }
        }
        drop(file);

        // Write retained events to a temp file, then rename atomically.
        let tmp_path = self.path.with_extension("log.tmp");
        {
            let mut tmp = std::io::BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&tmp_path)?,
            );
            for frame in &kept {
                tmp.write_all(frame)?;
            }
            tmp.flush()?;
        }
        std::fs::rename(&tmp_path, &self.path)?;

        // Re-open the writer pointing at the new (compacted) file.
        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        *_guard = std::io::BufWriter::new(new_file);

        Ok(dropped)
    }
}
