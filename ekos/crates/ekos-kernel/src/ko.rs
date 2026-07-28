//! Knowledge Object — the atomic unit of data in EKOS.
//!
//! Every piece of knowledge (requirement, decision, component, diagram,
//! metric, etc.) is a KO. The kernel stores KOs; applications own the
//! payload schema.
//!
//! ## Bitemporal model
//!
//! Each KO carries two time axes:
//!   - valid_from / valid_until  — when the fact is true in the real world
//!   - tx_time                   — when the kernel recorded the fact (immutable)
//!
//! This lets you answer "what did we know about X on date D?" as well as
//! "what was true about X on date D?" independently.

use std::fmt;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --------------------------------------------------------------------------
// KoId
// --------------------------------------------------------------------------

/// Opaque, globally unique identifier for a Knowledge Object.
/// Uses UUID v7 (time-ordered) so IDs sort chronologically.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KoId(Uuid);

impl KoId {
    /// Generate a new time-ordered KO identifier.
    pub fn new() -> Self {
        KoId(Uuid::now_v7())
    }

    /// Parse from a string (hyphenated UUID format).
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(KoId(Uuid::parse_str(s)?))
    }

    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for KoId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for KoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --------------------------------------------------------------------------
// KoKind
// --------------------------------------------------------------------------

/// The semantic type of a Knowledge Object.  Open-ended string so applications
/// can introduce their own kinds without a kernel change.
///
/// Built-in kinds (used by ESIP):
///   Requirement, Decision, Component, Diagram, Metric, Risk, Stakeholder
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KoKind(pub String);

impl KoKind {
    pub fn new(s: impl Into<String>) -> Self {
        KoKind(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for KoKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --------------------------------------------------------------------------
// KoStatus
// --------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KoStatus {
    Active,
    Deleted,
}

// --------------------------------------------------------------------------
// KnowledgeObject
// --------------------------------------------------------------------------

/// A single versioned, bitemporal knowledge object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeObject {
    /// Stable identifier — never changes across versions.
    pub id: KoId,

    /// Logical namespace (e.g. "ekos", "esip", "vectorvolt").
    pub namespace: String,

    /// Semantic type (open-ended string).
    pub kind: KoKind,

    /// Application-owned payload.  The kernel treats this as opaque bytes;
    /// callers are responsible for their own serialization format.
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,

    // ── Bitemporal ──────────────────────────────────────────────────────────
    /// When this version of the fact became true in the real world.
    pub valid_from: DateTime<Utc>,

    /// When this version of the fact stopped being true (None = still valid).
    pub valid_until: Option<DateTime<Utc>>,

    /// When the kernel recorded this version.  Immutable after first write.
    pub tx_time: DateTime<Utc>,

    // ── Metadata ────────────────────────────────────────────────────────────
    /// Free-form tags for secondary index lookups.
    pub tags: Vec<String>,

    /// Monotonically increasing within a single KO (starts at 1).
    pub version: u64,

    /// Active or soft-deleted.
    pub status: KoStatus,
}

impl KnowledgeObject {
    /// Create a new version-1 KO.
    pub fn new(namespace: impl Into<String>, kind: KoKind, payload: impl Into<Vec<u8>>) -> Self {
        let now = Utc::now();
        KnowledgeObject {
            id: KoId::new(),
            namespace: namespace.into(),
            kind,
            payload: payload.into(),
            valid_from: now,
            valid_until: None,
            tx_time: now,
            tags: Vec::new(),
            version: 1,
            status: KoStatus::Active,
        }
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }

    pub fn with_valid_from(mut self, t: DateTime<Utc>) -> Self {
        self.valid_from = t;
        self
    }

    pub fn with_valid_until(mut self, t: DateTime<Utc>) -> Self {
        self.valid_until = Some(t);
        self
    }

    pub fn is_deleted(&self) -> bool {
        self.status == KoStatus::Deleted
    }

    /// Produce the next version of this KO with an updated payload.
    /// tx_time is reset to now; all other fields are inherited.
    pub fn next_version(&self, payload: impl Into<Vec<u8>>) -> Self {
        KnowledgeObject {
            id: self.id.clone(),
            namespace: self.namespace.clone(),
            kind: self.kind.clone(),
            payload: payload.into(),
            valid_from: self.valid_from,
            valid_until: self.valid_until,
            tx_time: Utc::now(),
            tags: self.tags.clone(),
            version: self.version + 1,
            status: KoStatus::Active,
        }
    }

    /// Serialize to MessagePack bytes for storage.
    pub fn to_bytes(&self) -> Result<Bytes, rmp_serde::encode::Error> {
        Ok(Bytes::from(rmp_serde::to_vec_named(self)?))
    }

    /// Deserialize from MessagePack bytes.
    pub fn from_bytes(b: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(b)
    }
}
