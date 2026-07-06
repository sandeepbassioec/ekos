//! Conversions between the Protobuf wire types and the ekos-kernel domain types.

use chrono::{DateTime, TimeZone, Utc};
use prost_types::Timestamp;

use ekos_kernel::{KnowledgeObject, KoKind, KoStatus};

use crate::proto::ekos::v1 as pb;

// --------------------------------------------------------------------------
// Domain → Proto
// --------------------------------------------------------------------------

pub fn ko_to_proto(ko: KnowledgeObject) -> pb::KnowledgeObject {
    pb::KnowledgeObject {
        id:          ko.id.to_string(),
        namespace:   ko.namespace,
        kind:        ko.kind.0,
        payload:     ko.payload.into(),
        valid_from:  Some(dt_to_ts(ko.valid_from)),
        valid_until: ko.valid_until.map(dt_to_ts),
        tx_time:     Some(dt_to_ts(ko.tx_time)),
        tags:        ko.tags,
        version:     ko.version,
        status:      match ko.status {
            KoStatus::Active  => pb::KoStatus::Active as i32,
            KoStatus::Deleted => pb::KoStatus::Deleted as i32,
        },
    }
}

// --------------------------------------------------------------------------
// Proto → Domain
// --------------------------------------------------------------------------

pub fn proto_to_ko(p: &pb::KoPutRequest) -> anyhow::Result<KnowledgeObject> {
    use ekos_kernel::KoId;

    let id = if p.id.is_empty() {
        KoId::new()
    } else {
        KoId::parse(&p.id).map_err(|e| anyhow::anyhow!("invalid id: {e}"))?
    };

    let valid_from = p.valid_from
        .as_ref()
        .map(ts_to_dt)
        .unwrap_or_else(Utc::now);

    let valid_until = p.valid_until.as_ref().map(ts_to_dt);

    let now = Utc::now();
    Ok(KnowledgeObject {
        id,
        namespace:   p.namespace.clone(),
        kind:        KoKind::new(&p.kind),
        payload:     p.payload.to_vec(),
        valid_from,
        valid_until,
        tx_time:     now,
        tags:        p.tags.clone(),
        version:     1,
        status:      KoStatus::Active,
    })
}

// --------------------------------------------------------------------------
// Timestamp helpers
// --------------------------------------------------------------------------

pub fn dt_to_ts(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos:   dt.timestamp_subsec_nanos() as i32,
    }
}

pub fn ts_to_dt(ts: &Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as u32)
        .single()
        .unwrap_or_else(Utc::now)
}
