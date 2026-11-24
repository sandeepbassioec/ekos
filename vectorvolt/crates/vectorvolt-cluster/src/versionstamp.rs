//! Cluster-wide versionstamps (gap F8).
//!
//! A [`Versionstamp`] is 12 bytes: `epoch:u16 | physical_ms:u48 | logical:u16 | node:u16`
//! (big-endian), so byte order equals logical order.  Stamps come from a
//! per-node hybrid logical clock ([`VersionstampClock`]) that is strictly
//! increasing per node across backwards clock steps and restarts (a persisted
//! high-water reservation), and merges remote stamps via the HLC receive rule.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

pub const STAMP_LEN: usize = 12;
const PHYS_MAX: u64 = (1u64 << 48) - 1;
/// How far ahead of issued time the persisted reservation runs.
const RESERVE_MS: u64 = 1_000;

#[derive(Debug, thiserror::Error)]
pub enum StampError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Field order is significant: derived `Ord` == byte order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Versionstamp {
    pub epoch: u16,
    pub physical_ms: u64,
    pub logical: u16,
    pub node: u16,
}

impl Versionstamp {
    pub const ZERO: Versionstamp = Versionstamp {
        epoch: 0,
        physical_ms: 0,
        logical: 0,
        node: 0,
    };

    pub fn to_bytes(&self) -> [u8; STAMP_LEN] {
        let mut b = [0u8; STAMP_LEN];
        b[0..2].copy_from_slice(&self.epoch.to_be_bytes());
        b[2..8].copy_from_slice(&self.physical_ms.to_be_bytes()[2..8]);
        b[8..10].copy_from_slice(&self.logical.to_be_bytes());
        b[10..12].copy_from_slice(&self.node.to_be_bytes());
        b
    }

    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != STAMP_LEN {
            return None;
        }
        let mut p = [0u8; 8];
        p[2..8].copy_from_slice(&b[2..8]);
        Some(Self {
            epoch: u16::from_be_bytes([b[0], b[1]]),
            physical_ms: u64::from_be_bytes(p),
            logical: u16::from_be_bytes([b[8], b[9]]),
            node: u16::from_be_bytes([b[10], b[11]]),
        })
    }
}

pub type WallClock = Arc<dyn Fn() -> u64 + Send + Sync>;

pub fn system_wall_clock() -> WallClock {
    Arc::new(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    })
}

struct State {
    epoch: u16,
    phys: u64,
    logical: u16,
    /// Persisted reservation: every issued physical is < `limit`.
    limit: u64,
}

pub struct VersionstampClock {
    node: u16,
    wall: WallClock,
    path: Option<PathBuf>,
    st: Mutex<State>,
}

impl VersionstampClock {
    /// Open (or create) a clock. With `path`, the high-water mark and epoch persist.
    pub fn open(
        node: u16,
        epoch: u16,
        path: Option<PathBuf>,
        wall: WallClock,
    ) -> Result<Self, StampError> {
        let (mut ep, mut limit) = (epoch, 0u64);
        if let Some(p) = &path
            && let Ok(buf) = fs::read(p)
            && buf.len() == 14
            && crc32fast::hash(&buf[..10]).to_be_bytes() == buf[10..14]
        {
            ep = ep.max(u16::from_be_bytes([buf[0], buf[1]]));
            limit = u64::from_be_bytes(buf[2..10].try_into().unwrap());
        }
        // Anything issued before a crash had phys < limit, so starting at
        // `limit` with logical 0 is strictly greater.
        let phys = (wall)().max(limit).min(PHYS_MAX);
        let c = Self {
            node,
            wall,
            path,
            st: Mutex::new(State {
                epoch: ep,
                phys,
                logical: 0,
                limit: 0,
            }),
        };
        {
            let mut st = c.st.lock().unwrap();
            c.reserve(&mut st)?;
        }
        Ok(c)
    }

    pub fn node(&self) -> u16 {
        self.node
    }

    pub fn epoch(&self) -> u16 {
        self.st.lock().unwrap().epoch
    }

    /// Advance the cluster epoch / Raft term (never decreases).
    pub fn set_epoch(&self, epoch: u16) -> Result<(), StampError> {
        let mut st = self.st.lock().unwrap();
        if epoch > st.epoch {
            st.epoch = epoch;
            self.persist(&st, st.limit)?;
        }
        Ok(())
    }

    fn persist(&self, st: &State, limit: u64) -> Result<(), StampError> {
        let Some(p) = &self.path else { return Ok(()) };
        let mut buf = Vec::with_capacity(14);
        buf.extend_from_slice(&st.epoch.to_be_bytes());
        buf.extend_from_slice(&limit.to_be_bytes());
        let crc = crc32fast::hash(&buf);
        buf.extend_from_slice(&crc.to_be_bytes());
        let tmp = p.with_extension("tmp");
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&buf)?;
        f.sync_all()?;
        fs::rename(&tmp, p)?;
        Ok(())
    }

    fn reserve(&self, st: &mut State) -> Result<(), StampError> {
        if st.phys + 1 >= st.limit {
            let new_limit = st.phys + RESERVE_MS;
            self.persist(st, new_limit)?;
            st.limit = new_limit;
        }
        Ok(())
    }

    fn issue(
        &self,
        st: &mut State,
        remote: Option<Versionstamp>,
    ) -> Result<Versionstamp, StampError> {
        let now = (self.wall)();
        let mut p = st.phys.max(now);
        // Smallest logical that keeps us strictly above our previous stamp.
        let mut l: u32 = if p == st.phys {
            st.logical as u32 + 1
        } else {
            0
        };
        if let Some(r) = remote {
            if r.epoch > st.epoch {
                st.epoch = r.epoch;
            }
            if r.physical_ms > p {
                p = r.physical_ms;
                l = r.logical as u32 + 1;
            } else if r.physical_ms == p {
                l = l.max(r.logical as u32 + 1);
            }
        }
        if l > u16::MAX as u32 {
            p += 1;
            l = 0;
        }
        st.phys = p;
        st.logical = l as u16;
        self.reserve(st)?;
        Ok(Versionstamp {
            epoch: st.epoch,
            physical_ms: p,
            logical: st.logical,
            node: self.node,
        })
    }

    /// Assign the stamp for a local commit.
    pub fn next(&self) -> Result<Versionstamp, StampError> {
        let mut st = self.st.lock().unwrap();
        self.issue(&mut st, None)
    }

    /// HLC receive rule: merge a remote stamp; the returned stamp is greater
    /// than both the remote stamp and every earlier local stamp.
    pub fn observe(&self, remote: Versionstamp) -> Result<Versionstamp, StampError> {
        let mut st = self.st.lock().unwrap();
        self.issue(&mut st, Some(remote))
    }
}
