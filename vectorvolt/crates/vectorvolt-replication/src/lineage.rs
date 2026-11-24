//! Epoch lineage: how a node's data maps onto the primaries it has followed.
//!
//! Every node numbers its WAL itself, and a follower's WAL numbering is *not*
//! the primary's (it applies one batch per `Frames` message). After a failover
//! the new primary therefore cannot name a catch-up point in the old primary's
//! LSNs directly. The lineage is the missing translation table, persisted next
//! to the replication position:
//!
//! * `Applied { epoch, lsn, local }` — after applying the primary-of-`epoch`'s
//!   frames up to `lsn`, this node's own WAL tip was `local`.
//! * `PrimaryStart { epoch, tip }` / `PrimaryEnd { epoch, tip }` — this node was
//!   the primary of `epoch` for local WAL sequences `tip_start ..= tip_end`
//!   (identity mapping: its LSN *is* its local sequence).
//!
//! Two nodes agree on a catch-up point like Raft log matching: the follower
//! sends `(epoch, highest lsn)` for its newest epochs, the primary picks the
//! newest epoch it also knows, takes the smaller of the two end positions as
//! the common point and maps it to its own WAL sequence ([`Lineage::map`]).
//! Anything the follower applied beyond the common point is its divergent
//! (never acknowledged to the new primary) tail.

use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use parking_lot::Mutex;

/// File (inside the data dir) holding the lineage.
pub const LINEAGE_FILE: &str = "replication.lineage";

/// Shared handle: the client appends to it, a promoted node's server reads it.
pub type LineageStore = Mutex<Lineage>;

const REC_LEN: usize = 8 * 4 + 4;
/// Compact once the lineage holds this many records.
const MAX_RECS: usize = 65_536;
/// Epochs advertised in a hello.
const HISTORY_EPOCHS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Applied = 0,
    PrimaryStart = 1,
    PrimaryEnd = 2,
}

#[derive(Debug, Clone, Copy)]
struct Rec {
    kind: Kind,
    epoch: u64,
    lsn: u64,
    local: u64,
}

impl Rec {
    fn encode(&self) -> [u8; REC_LEN] {
        let mut b = [0u8; REC_LEN];
        b[0..8].copy_from_slice(&(self.kind as u64).to_le_bytes());
        b[8..16].copy_from_slice(&self.epoch.to_le_bytes());
        b[16..24].copy_from_slice(&self.lsn.to_le_bytes());
        b[24..32].copy_from_slice(&self.local.to_le_bytes());
        let crc = crc32fast::hash(&b[..32]);
        b[32..].copy_from_slice(&crc.to_le_bytes());
        b
    }

    fn decode(b: &[u8]) -> Option<Rec> {
        if b.len() != REC_LEN {
            return None;
        }
        let crc = u32::from_le_bytes(b[32..].try_into().ok()?);
        if crc32fast::hash(&b[..32]) != crc {
            return None;
        }
        let u = |i: usize| u64::from_le_bytes(b[i..i + 8].try_into().expect("8 bytes"));
        let kind = match u(0) {
            0 => Kind::Applied,
            1 => Kind::PrimaryStart,
            2 => Kind::PrimaryEnd,
            _ => return None,
        };
        Some(Rec {
            kind,
            epoch: u(8),
            lsn: u(16),
            local: u(24),
        })
    }
}

/// Result of [`Lineage::map`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapped {
    /// The common point, in the lineage epoch's own LSN space.
    pub common_lsn: u64,
    /// This node's WAL sequence to stream from (at or before the common point).
    pub local: u64,
}

pub struct Lineage {
    path: PathBuf,
    recs: Vec<Rec>,
    file: File,
}

impl Lineage {
    /// Load `<dir>/replication.lineage`, dropping any torn tail.
    pub fn open(dir: &Path) -> std::io::Result<Self> {
        let path = dir.join(LINEAGE_FILE);
        let raw = std::fs::read(&path).unwrap_or_default();
        let mut recs = Vec::new();
        for chunk in raw.as_chunks::<REC_LEN>().0.iter().map(|c| c.as_slice()) {
            match Rec::decode(chunk) {
                Some(r) => recs.push(r),
                None => break,
            }
        }
        let valid = recs.len() * REC_LEN;
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        if valid != raw.len() {
            let mut me = Lineage { path, recs, file };
            me.rewrite()?;
            return Ok(me);
        }
        Ok(Lineage { path, recs, file })
    }

    /// Wrap in the shared handle the client and server take.
    pub fn shared(self) -> std::sync::Arc<LineageStore> {
        std::sync::Arc::new(Mutex::new(self))
    }

    fn append(&mut self, rec: Rec, sync: bool) -> std::io::Result<()> {
        self.file.write_all(&rec.encode())?;
        if sync {
            self.file.sync_data()?;
        }
        self.recs.push(rec);
        if self.recs.len() > MAX_RECS {
            self.compact()?;
        }
        Ok(())
    }

    fn rewrite(&mut self) -> std::io::Result<()> {
        let tmp = self.path.with_extension("tmp");
        {
            let mut f = File::create(&tmp)?;
            for r in &self.recs {
                f.write_all(&r.encode())?;
            }
            f.sync_all()?;
        }
        std::fs::rename(&tmp, &self.path)?;
        self.file = OpenOptions::new().append(true).open(&self.path)?;
        Ok(())
    }

    /// Thin old `Applied` records (keep every 32nd, plus the newest 8192).
    fn compact(&mut self) -> std::io::Result<()> {
        let n = self.recs.len();
        let keep_from = n.saturating_sub(8192);
        let mut i = 0usize;
        let old = std::mem::take(&mut self.recs);
        self.recs = old
            .into_iter()
            .enumerate()
            .filter(|(idx, r)| {
                if r.kind != Kind::Applied || *idx >= keep_from {
                    return true;
                }
                i += 1;
                i.is_multiple_of(32)
            })
            .map(|(_, r)| r)
            .collect();
        self.rewrite()
    }

    /// After applying `epoch`'s frames up to `lsn`, the local WAL tip was `local`.
    pub fn record_applied(&mut self, epoch: u64, lsn: u64, local: u64) -> std::io::Result<()> {
        if let Some(l) = self.recs.last() {
            if l.kind == Kind::Applied && l.epoch == epoch && l.lsn == lsn {
                return Ok(());
            }
        }
        // An epoch change is rare and must survive a crash; batches need not.
        let sync = self.recs.last().is_none_or(|l| l.epoch != epoch);
        self.append(
            Rec {
                kind: Kind::Applied,
                epoch,
                lsn,
                local,
            },
            sync,
        )
    }

    /// This node is the primary of `epoch`; its WAL tip is `tip`.
    pub fn start_primary(&mut self, epoch: u64, tip: u64) -> std::io::Result<()> {
        self.append(
            Rec {
                kind: Kind::PrimaryStart,
                epoch,
                lsn: tip,
                local: tip,
            },
            true,
        )
    }

    /// This node stopped being the primary of `epoch` at WAL tip `tip`.
    pub fn end_primary(&mut self, epoch: u64, tip: u64) -> std::io::Result<()> {
        let started = self
            .recs
            .iter()
            .any(|r| r.kind == Kind::PrimaryStart && r.epoch == epoch);
        let ended = self
            .recs
            .iter()
            .any(|r| r.kind == Kind::PrimaryEnd && r.epoch == epoch);
        if !started || ended {
            return Ok(());
        }
        self.append(
            Rec {
                kind: Kind::PrimaryEnd,
                epoch,
                lsn: tip,
                local: tip,
            },
            true,
        )
    }

    /// Epoch this node last led and has not yet recorded the end of.
    pub fn open_primary_epoch(&self) -> Option<u64> {
        self.recs
            .iter()
            .rev()
            .find(|r| r.kind != Kind::Applied)
            .filter(|r| r.kind == Kind::PrimaryStart)
            .map(|r| r.epoch)
    }

    fn start_of(&self, epoch: u64) -> Option<u64> {
        self.recs
            .iter()
            .find(|r| r.kind == Kind::PrimaryStart && r.epoch == epoch)
            .map(|r| r.lsn)
    }

    /// Highest LSN this node holds for `epoch`; `open_tip` is the current WAL
    /// tip, used when this node leads `epoch` right now.
    pub fn end_of(&self, epoch: u64, open_tip: u64) -> Option<u64> {
        if self.start_of(epoch).is_some() {
            return Some(
                self.recs
                    .iter()
                    .find(|r| r.kind == Kind::PrimaryEnd && r.epoch == epoch)
                    .map_or(open_tip, |r| r.lsn),
            );
        }
        self.recs
            .iter()
            .filter(|r| r.kind == Kind::Applied && r.epoch == epoch)
            .map(|r| r.lsn)
            .max()
    }

    /// Translate "the primary of `epoch` at `lsn`" to this node's WAL.
    /// `None` when this node has no usable record of that point.
    pub fn map(&self, epoch: u64, lsn: u64, open_tip: u64) -> Option<Mapped> {
        let end = self.end_of(epoch, open_tip)?;
        if let Some(start) = self.start_of(epoch) {
            if lsn < start {
                return None;
            }
            let c = lsn.min(end);
            return Some(Mapped {
                common_lsn: c,
                local: c,
            });
        }
        let best = self
            .recs
            .iter()
            .filter(|r| r.kind == Kind::Applied && r.epoch == epoch && r.lsn <= lsn)
            .max_by_key(|r| r.lsn)?;
        Some(Mapped {
            common_lsn: lsn.min(end),
            local: best.local,
        })
    }

    /// `(epoch, highest lsn)` for the newest epochs, newest first. The caller's
    /// current `position` always heads the list (the lineage file may lag it).
    pub fn history(&self, position: (u64, u64), open_tip: u64) -> Vec<(u64, u64)> {
        let mut epochs: Vec<u64> = self.recs.iter().map(|r| r.epoch).collect();
        epochs.push(position.0);
        epochs.sort_unstable_by(|a, b| b.cmp(a));
        epochs.dedup();
        epochs
            .into_iter()
            .take(HISTORY_EPOCHS)
            .map(|e| {
                if e == position.0 {
                    (e, position.1)
                } else {
                    (e, self.end_of(e, open_tip).unwrap_or(0))
                }
            })
            .collect()
    }

    /// Forget everything after `(epoch, lsn)`: that tail was truncated away.
    pub fn truncate_after(&mut self, epoch: u64, lsn: u64) -> std::io::Result<()> {
        let was_primary = self.start_of(epoch).is_some();
        self.recs.retain(|r| match r.epoch.cmp(&epoch) {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
            std::cmp::Ordering::Equal => match r.kind {
                Kind::Applied => r.lsn <= lsn,
                Kind::PrimaryStart => true,
                Kind::PrimaryEnd => false,
            },
        });
        if was_primary {
            self.recs.push(Rec {
                kind: Kind::PrimaryEnd,
                epoch,
                lsn,
                local: lsn,
            });
        }
        self.rewrite()
    }

    /// Replace the lineage with a single point (a full snapshot was installed).
    pub fn reset_to(&mut self, epoch: u64, lsn: u64, local: u64) -> std::io::Result<()> {
        self.recs.clear();
        self.recs.push(Rec {
            kind: Kind::Applied,
            epoch,
            lsn,
            local,
        });
        self.rewrite()
    }

    /// Number of records (diagnostics / tests).
    pub fn len(&self) -> usize {
        self.recs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.recs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_follower_points_and_primary_identity() {
        let d = tempfile::tempdir().unwrap();
        let mut l = Lineage::open(d.path()).unwrap();
        // Followed epoch 1: leader lsn 10 -> local 4, lsn 25 -> local 9.
        l.record_applied(1, 10, 4).unwrap();
        l.record_applied(1, 25, 9).unwrap();
        // Led epoch 2 from local tip 9.
        l.start_primary(2, 9).unwrap();
        assert_eq!(l.end_of(1, 50), Some(25));
        assert_eq!(
            l.map(1, 30, 50),
            Some(Mapped {
                common_lsn: 25,
                local: 9
            })
        );
        assert_eq!(
            l.map(1, 12, 50),
            Some(Mapped {
                common_lsn: 12,
                local: 4
            })
        );
        assert_eq!(l.map(1, 5, 50), None, "before the first record: unknown");
        assert_eq!(
            l.map(2, 20, 30),
            Some(Mapped {
                common_lsn: 20,
                local: 20
            })
        );
        assert_eq!(l.end_of(2, 30), Some(30));
        l.end_primary(2, 33).unwrap();
        assert_eq!(l.end_of(2, 99), Some(33));
        assert_eq!(l.history((1, 25), 99)[0], (2, 33));
        // Reload from disk.
        let l2 = Lineage::open(d.path()).unwrap();
        assert_eq!(l2.len(), l.len());
        assert_eq!(l2.map(1, 25, 0).unwrap().local, 9);
    }

    #[test]
    fn truncate_drops_the_divergent_tail_and_later_epochs() {
        let d = tempfile::tempdir().unwrap();
        let mut l = Lineage::open(d.path()).unwrap();
        l.record_applied(1, 10, 4).unwrap();
        l.record_applied(1, 25, 9).unwrap();
        l.record_applied(2, 3, 11).unwrap();
        l.truncate_after(1, 10).unwrap();
        assert_eq!(l.end_of(1, 0), Some(10));
        assert_eq!(l.end_of(2, 0), None);
    }

    #[test]
    fn torn_tail_is_discarded() {
        let d = tempfile::tempdir().unwrap();
        {
            let mut l = Lineage::open(d.path()).unwrap();
            l.record_applied(1, 10, 4).unwrap();
            l.record_applied(1, 20, 8).unwrap();
        }
        let p = d.path().join(LINEAGE_FILE);
        let mut raw = std::fs::read(&p).unwrap();
        raw.truncate(raw.len() - 5);
        std::fs::write(&p, raw).unwrap();
        let l = Lineage::open(d.path()).unwrap();
        assert_eq!(l.len(), 1);
    }
}
