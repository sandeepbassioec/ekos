//! Key scheme for the VectorVolt KV store.
//!
//! All keys are UTF-8 strings.  The scheme is designed so that prefix scans
//! naturally return the right set of records without secondary lookups.
//!
//! Layout:
//!
//!   ko:{ns}/{id}                     → latest active version (MessagePack KO)
//!   koh:{ns}/{id}/{tx_ts_ns}         → historical version log (MessagePack KO)
//!   idx:kind:{ns}/{kind}/{id}        → kind index  (value = empty)
//!   idx:tag:{ns}/{tag}/{id}          → tag index   (value = empty)
//!
//! All timestamps in the history key are zero-padded 19-digit nanoseconds
//! since Unix epoch for lexicographic ordering.

use bytes::Bytes;

// --------------------------------------------------------------------------
// Primary keys
// --------------------------------------------------------------------------

/// Key for the latest version of a KO.
pub fn ko_key(namespace: &str, id: &str) -> Bytes {
    Bytes::from(format!("ko:{namespace}/{id}"))
}

/// Key for a specific historical version (tx_time in nanoseconds).
pub fn ko_history_key(namespace: &str, id: &str, tx_ns: i64) -> Bytes {
    Bytes::from(format!("koh:{namespace}/{id}/{tx_ns:019}"))
}

/// Prefix that covers all history entries for a single KO.
pub fn ko_history_prefix(namespace: &str, id: &str) -> Bytes {
    Bytes::from(format!("koh:{namespace}/{id}/"))
}

// --------------------------------------------------------------------------
// Index keys (value is always empty — presence is the signal)
// --------------------------------------------------------------------------

pub fn idx_kind_key(namespace: &str, kind: &str, id: &str) -> Bytes {
    Bytes::from(format!("idx:kind:{namespace}/{kind}/{id}"))
}

pub fn idx_kind_prefix(namespace: &str, kind: &str) -> Bytes {
    Bytes::from(format!("idx:kind:{namespace}/{kind}/"))
}

pub fn idx_tag_key(namespace: &str, tag: &str, id: &str) -> Bytes {
    Bytes::from(format!("idx:tag:{namespace}/{tag}/{id}"))
}

pub fn idx_tag_prefix(namespace: &str, tag: &str) -> Bytes {
    Bytes::from(format!("idx:tag:{namespace}/{tag}/"))
}

/// Prefix covering all KOs in a namespace.
pub fn ko_namespace_prefix(namespace: &str) -> Bytes {
    Bytes::from(format!("ko:{namespace}/"))
}

/// Extract the KO id from the trailing segment of an index key.
/// e.g. "idx:kind:ekos/Requirement/abc123" → "abc123"
pub fn id_from_index_key(key: &Bytes) -> Option<&str> {
    let s = std::str::from_utf8(key).ok()?;
    s.rsplit('/').next()
}
