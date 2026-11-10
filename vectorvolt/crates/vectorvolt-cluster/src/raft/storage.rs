//! Durable Raft storage: hard state (term, vote), append-only log (WAL) and
//! snapshot. Every mutating call fsyncs before returning, so callers may ack
//! a vote / AppendEntries only after the call succeeds.
//!
//! Layout inside the directory:
//! - `hard_state`  : `[crc32 u32 LE][json]`, replaced atomically (tmp+rename)
//! - `log.wal`     : records `[len u32 LE][crc32 u32 LE][json WalRecord]`;
//!   a torn / corrupt tail is detected on load and truncated away
//! - `snapshot`    : `[crc32 u32 LE][json]`, replaced atomically

use super::log::LogEntry;
use crate::node::NodeId;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const MAX_RECORD: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HardState {
    pub term: u64,
    pub voted_for: Option<NodeId>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub last_index: u64,
    pub last_term: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
enum WalRecord {
    Entry(LogEntry),
    /// Remove every entry with `index >= from`.
    Truncate {
        from: u64,
    },
}

/// State recovered from disk.
#[derive(Debug, Default)]
pub struct Loaded {
    pub hard_state: HardState,
    pub snapshot: Option<Snapshot>,
    /// Entries strictly after the snapshot, contiguous.
    pub entries: Vec<LogEntry>,
}

pub struct FileStorage {
    dir: PathBuf,
    wal: Option<File>,
}

fn crc(bytes: &[u8]) -> u32 {
    crc32fast::hash(bytes)
}

fn bad(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.to_string())
}

fn sync_dir(dir: &Path) {
    // Best effort: directory fsync is meaningful on unix, unsupported on Windows.
    #[cfg(unix)]
    if let Ok(d) = File::open(dir) {
        let _ = d.sync_all();
    }
    #[cfg(not(unix))]
    let _ = dir;
}

fn write_atomic(dir: &Path, name: &str, payload: &[u8]) -> io::Result<()> {
    let tmp = dir.join(format!("{name}.tmp"));
    let mut f = File::create(&tmp)?;
    f.write_all(&crc(payload).to_le_bytes())?;
    f.write_all(payload)?;
    f.sync_all()?;
    drop(f);
    fs::rename(&tmp, dir.join(name))?;
    sync_dir(dir);
    Ok(())
}

fn read_checked(path: &Path) -> io::Result<Option<Vec<u8>>> {
    let mut buf = Vec::new();
    match File::open(path) {
        Ok(mut f) => f.read_to_end(&mut buf)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    if buf.len() < 4 {
        return Err(bad("file too short"));
    }
    let want = u32::from_le_bytes(buf[..4].try_into().expect("4 bytes"));
    if crc(&buf[4..]) != want {
        return Err(bad("checksum mismatch"));
    }
    Ok(Some(buf[4..].to_vec()))
}

fn encode_record(rec: &WalRecord) -> Vec<u8> {
    let payload = serde_json::to_vec(rec).expect("wal record serializes");
    let mut out = Vec::with_capacity(8 + payload.len());
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&crc(&payload).to_le_bytes());
    out.extend_from_slice(&payload);
    out
}

impl FileStorage {
    /// Open (creating if needed) the storage directory and recover state.
    pub fn open(dir: impl AsRef<Path>) -> io::Result<(Self, Loaded)> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;
        // Leftover temp files from a crash mid-replace are garbage.
        for n in ["hard_state.tmp", "snapshot.tmp", "log.wal.tmp"] {
            let _ = fs::remove_file(dir.join(n));
        }

        let hard_state = match read_checked(&dir.join("hard_state"))? {
            Some(p) => serde_json::from_slice(&p).map_err(|e| bad(&e.to_string()))?,
            None => HardState::default(),
        };
        let snapshot: Option<Snapshot> = match read_checked(&dir.join("snapshot"))? {
            Some(p) => Some(serde_json::from_slice(&p).map_err(|e| bad(&e.to_string()))?),
            None => None,
        };
        let snap_index = snapshot.as_ref().map_or(0, |s| s.last_index);

        let wal_path = dir.join("log.wal");
        let mut raw = Vec::new();
        if let Ok(mut f) = File::open(&wal_path) {
            f.read_to_end(&mut raw)?;
        }
        let mut entries: Vec<LogEntry> = Vec::new();
        let mut pos = 0usize;
        let mut good = 0usize;
        while pos + 8 <= raw.len() {
            let len = u32::from_le_bytes(raw[pos..pos + 4].try_into().expect("4")) as usize;
            let want = u32::from_le_bytes(raw[pos + 4..pos + 8].try_into().expect("4"));
            if len > MAX_RECORD || pos + 8 + len > raw.len() {
                break;
            }
            let payload = &raw[pos + 8..pos + 8 + len];
            if crc(payload) != want {
                break;
            }
            let Ok(rec) = serde_json::from_slice::<WalRecord>(payload) else {
                break;
            };
            match rec {
                WalRecord::Entry(e) => entries.push(e),
                WalRecord::Truncate { from } => entries.retain(|e| e.index < from),
            }
            pos += 8 + len;
            good = pos;
        }

        let mut wal = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&wal_path)?;
        if (good as u64) < raw.len() as u64 {
            // Torn tail: cut it off so later appends follow valid data.
            wal.set_len(good as u64)?;
            wal.sync_all()?;
        }
        wal.seek(SeekFrom::End(0))?;

        // Drop entries covered by the snapshot; keep only a contiguous run.
        entries.retain(|e| e.index > snap_index);
        let mut contiguous = Vec::with_capacity(entries.len());
        for (e, expect) in entries.into_iter().zip(snap_index + 1..) {
            if e.index != expect {
                break;
            }
            contiguous.push(e);
        }

        Ok((
            FileStorage {
                dir,
                wal: Some(wal),
            },
            Loaded {
                hard_state,
                snapshot,
                entries: contiguous,
            },
        ))
    }

    fn wal(&mut self) -> io::Result<&mut File> {
        self.wal
            .as_mut()
            .ok_or_else(|| io::Error::other("wal handle unavailable after failed snapshot"))
    }

    pub fn save_hard_state(&mut self, hs: &HardState) -> io::Result<()> {
        let payload = serde_json::to_vec(hs).map_err(|e| bad(&e.to_string()))?;
        write_atomic(&self.dir, "hard_state", &payload)
    }

    pub fn append(&mut self, entries: &[LogEntry]) -> io::Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        let mut buf = Vec::new();
        for e in entries {
            buf.extend(encode_record(&WalRecord::Entry(e.clone())));
        }
        let wal = self.wal()?;
        wal.write_all(&buf)?;
        wal.sync_data()
    }

    pub fn truncate_from(&mut self, from: u64) -> io::Result<()> {
        let wal = self.wal()?;
        wal.write_all(&encode_record(&WalRecord::Truncate { from }))?;
        wal.sync_data()
    }

    /// Persist `snap` and rewrite the WAL to contain only `remaining`.
    /// The snapshot is made durable first; a crash in between is recovered by
    /// discarding WAL entries covered by the snapshot on load.
    pub fn save_snapshot(&mut self, snap: &Snapshot, remaining: &[LogEntry]) -> io::Result<()> {
        let payload = serde_json::to_vec(snap).map_err(|e| bad(&e.to_string()))?;
        write_atomic(&self.dir, "snapshot", &payload)?;

        let tmp = self.dir.join("log.wal.tmp");
        let mut f = File::create(&tmp)?;
        for e in remaining {
            f.write_all(&encode_record(&WalRecord::Entry(e.clone())))?;
        }
        f.sync_all()?;
        drop(f);
        // Release our handle before replacing the file (required on Windows).
        self.wal = None;
        fs::rename(&tmp, self.dir.join("log.wal"))?;
        sync_dir(&self.dir);
        let mut wal = OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.dir.join("log.wal"))?;
        wal.seek(SeekFrom::End(0))?;
        self.wal = Some(wal);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(term: u64, index: u64) -> LogEntry {
        LogEntry {
            term,
            index,
            command: vec![index as u8],
        }
    }

    #[test]
    fn roundtrip_hard_state_and_log() {
        let d = tempfile::tempdir().unwrap();
        let (mut s, l) = FileStorage::open(d.path()).unwrap();
        assert!(l.entries.is_empty());
        let id = NodeId::new();
        s.save_hard_state(&HardState {
            term: 7,
            voted_for: Some(id.clone()),
        })
        .unwrap();
        s.append(&[e(1, 1), e(1, 2), e(2, 3)]).unwrap();
        s.truncate_from(3).unwrap();
        s.append(&[e(3, 3)]).unwrap();
        drop(s);
        let (_s, l) = FileStorage::open(d.path()).unwrap();
        assert_eq!(l.hard_state.term, 7);
        assert_eq!(l.hard_state.voted_for, Some(id));
        assert_eq!(l.entries.len(), 3);
        assert_eq!(l.entries[2].term, 3);
    }

    #[test]
    fn torn_tail_is_ignored_and_repaired() {
        let d = tempfile::tempdir().unwrap();
        let (mut s, _) = FileStorage::open(d.path()).unwrap();
        s.append(&[e(1, 1), e(1, 2)]).unwrap();
        drop(s);
        let p = d.path().join("log.wal");
        let mut f = OpenOptions::new().append(true).open(&p).unwrap();
        f.write_all(&[200, 0, 0, 0, 1, 2, 3, 4, 9, 9]).unwrap(); // torn record
        drop(f);
        let (mut s, l) = FileStorage::open(d.path()).unwrap();
        assert_eq!(l.entries.len(), 2);
        s.append(&[e(1, 3)]).unwrap();
        drop(s);
        let (_s, l) = FileStorage::open(d.path()).unwrap();
        assert_eq!(l.entries.len(), 3);
    }

    #[test]
    fn snapshot_compacts_log() {
        let d = tempfile::tempdir().unwrap();
        let (mut s, _) = FileStorage::open(d.path()).unwrap();
        s.append(&[e(1, 1), e(1, 2), e(1, 3)]).unwrap();
        s.save_snapshot(
            &Snapshot {
                last_index: 2,
                last_term: 1,
                data: vec![9],
            },
            &[e(1, 3)],
        )
        .unwrap();
        s.append(&[e(1, 4)]).unwrap();
        drop(s);
        let (_s, l) = FileStorage::open(d.path()).unwrap();
        assert_eq!(l.snapshot.unwrap().last_index, 2);
        assert_eq!(l.entries.len(), 2);
        assert_eq!(l.entries[0].index, 3);
    }
}
