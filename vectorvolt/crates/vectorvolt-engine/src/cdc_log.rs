//! Persistent CDC event log — Layer 67.
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

use crate::cdc::ChangeEvent;

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
        let json = serde_json::to_vec(event)
            .map_err(std::io::Error::other)?;
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
            // Skip JSON body.
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
}
