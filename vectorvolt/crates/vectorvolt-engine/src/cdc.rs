//! Change Data Capture (CDC) types.
//!
//! Every successful mutation published to the per-engine broadcast channel
//! carries a [`ChangeEvent`].  HTTP consumers receive events over SSE at
//! `GET /tables/{t}/subscribe`.

use serde::{Deserialize, Serialize};

/// The kind of change that occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Insert,
    Delete,
}

/// A single mutation broadcast to all active subscribers and persisted to the
/// CDC log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    /// Monotonically increasing log sequence number assigned at publish time.
    /// Starts at 1; a value of 0 means "no LSN assigned" (never produced by
    /// a live engine).
    pub lsn: u64,
    pub kind: ChangeKind,
    pub pk: u64,
    /// New row values — present on [`ChangeKind::Insert`], absent on
    /// [`ChangeKind::Delete`].
    pub values: Option<Vec<u64>>,
    /// Commit versionstamp (12 bytes, hex on the wire) when a commit stamper is
    /// installed ([`TableEngine::set_commit_stamper`](crate::TableEngine::set_commit_stamper)).
    #[serde(default, skip_serializing_if = "Option::is_none", with = "stamp_hex")]
    pub versionstamp: Option<[u8; 12]>,
}

impl ChangeEvent {
    /// Attach a commit versionstamp.
    pub fn with_stamp(mut self, stamp: Option<[u8; 12]>) -> Self {
        self.versionstamp = stamp;
        self
    }

    pub fn insert(lsn: u64, pk: u64, values: Vec<u64>) -> Self {
        Self {
            lsn,
            kind: ChangeKind::Insert,
            pk,
            values: Some(values),
            versionstamp: None,
        }
    }

    pub fn delete(lsn: u64, pk: u64) -> Self {
        Self {
            lsn,
            kind: ChangeKind::Delete,
            pk,
            values: None,
            versionstamp: None,
        }
    }
}

/// Hex (de)serialisation of the optional 12-byte versionstamp.
mod stamp_hex {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &Option<[u8; 12]>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(b) => {
                let mut out = String::with_capacity(24);
                for x in b {
                    out.push_str(&format!("{x:02x}"));
                }
                s.serialize_str(&out)
            }
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<[u8; 12]>, D::Error> {
        let Some(h) = Option::<String>::deserialize(d)? else {
            return Ok(None);
        };
        if h.len() != 24 {
            return Err(serde::de::Error::custom(
                "versionstamp must be 24 hex chars",
            ));
        }
        let mut b = [0u8; 12];
        for (i, slot) in b.iter_mut().enumerate() {
            *slot =
                u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).map_err(serde::de::Error::custom)?;
        }
        Ok(Some(b))
    }
}
