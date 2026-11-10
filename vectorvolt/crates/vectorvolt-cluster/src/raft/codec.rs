//! Compact binary wire encoding for Raft RPCs (replaces the JSON payloads).
//!
//! Integers are LEB128 varints, byte strings are length-prefixed, node ids are
//! their 16 raw UUID bytes. Decoding is fully bounds-checked: lengths are
//! validated against the remaining input before any allocation so a hostile
//! frame cannot trigger a huge allocation.

use super::config::ClusterConfig;
use super::log::LogEntry;
use super::rpc::Rpc;
use crate::node::NodeId;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
#[error("wire decode error: {0}")]
pub struct CodecError(pub &'static str);

/// Types that travel inside a transport frame.
pub trait Wire: Sized {
    fn to_wire(&self) -> Vec<u8>;
    fn from_wire(buf: &[u8]) -> Result<Self, CodecError>;
}

#[derive(Default)]
pub struct Writer(pub Vec<u8>);

impl Writer {
    pub fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    pub fn bool(&mut self, v: bool) {
        self.0.push(v as u8);
    }
    pub fn varint(&mut self, mut v: u64) {
        while v >= 0x80 {
            self.0.push((v as u8) | 0x80);
            v >>= 7;
        }
        self.0.push(v as u8);
    }
    pub fn bytes(&mut self, b: &[u8]) {
        self.varint(b.len() as u64);
        self.0.extend_from_slice(b);
    }
    pub fn node(&mut self, n: &NodeId) {
        self.0.extend_from_slice(n.0.as_bytes());
    }
    pub fn nodes(&mut self, ns: &[NodeId]) {
        self.varint(ns.len() as u64);
        for n in ns {
            self.node(n);
        }
    }
}

pub struct Reader<'a> {
    buf: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Reader { buf }
    }
    pub fn remaining(&self) -> usize {
        self.buf.len()
    }
    pub fn u8(&mut self) -> Result<u8, CodecError> {
        let (&b, rest) = self.buf.split_first().ok_or(CodecError("eof"))?;
        self.buf = rest;
        Ok(b)
    }
    pub fn bool(&mut self) -> Result<bool, CodecError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(CodecError("bad bool")),
        }
    }
    pub fn varint(&mut self) -> Result<u64, CodecError> {
        let mut v = 0u64;
        for shift in (0..70).step_by(7) {
            let b = self.u8()?;
            if shift == 63 && b > 1 {
                return Err(CodecError("varint overflow"));
            }
            v |= ((b & 0x7f) as u64) << shift;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(CodecError("varint too long"))
    }
    pub fn bytes(&mut self) -> Result<Vec<u8>, CodecError> {
        let n = self.varint()?;
        if n > self.buf.len() as u64 {
            return Err(CodecError("length exceeds input"));
        }
        let (head, rest) = self.buf.split_at(n as usize);
        self.buf = rest;
        Ok(head.to_vec())
    }
    pub fn node(&mut self) -> Result<NodeId, CodecError> {
        if self.buf.len() < 16 {
            return Err(CodecError("eof in node id"));
        }
        let (head, rest) = self.buf.split_at(16);
        self.buf = rest;
        Ok(NodeId(
            Uuid::from_slice(head).map_err(|_| CodecError("uuid"))?,
        ))
    }
    pub fn nodes(&mut self) -> Result<Vec<NodeId>, CodecError> {
        let n = self.varint()?;
        if n.saturating_mul(16) > self.buf.len() as u64 {
            return Err(CodecError("node list exceeds input"));
        }
        (0..n).map(|_| self.node()).collect()
    }
    pub fn finish(&self) -> Result<(), CodecError> {
        if self.buf.is_empty() {
            Ok(())
        } else {
            Err(CodecError("trailing bytes"))
        }
    }
}

fn put_entry(w: &mut Writer, e: &LogEntry) {
    w.varint(e.term);
    w.varint(e.index);
    w.bytes(&e.command);
}

fn get_entry(r: &mut Reader) -> Result<LogEntry, CodecError> {
    Ok(LogEntry {
        term: r.varint()?,
        index: r.varint()?,
        command: r.bytes()?,
    })
}

const T_RV: u8 = 1;
const T_RVR: u8 = 2;
const T_AE: u8 = 3;
const T_AER: u8 = 4;
const T_IS: u8 = 5;
const T_ISR: u8 = 6;
const T_PV: u8 = 7;
const T_PVR: u8 = 8;

impl Wire for Rpc {
    fn to_wire(&self) -> Vec<u8> {
        let mut w = Writer::default();
        match self {
            Rpc::RequestVote {
                term,
                last_log_index,
                last_log_term,
            } => {
                w.u8(T_RV);
                w.varint(*term);
                w.varint(*last_log_index);
                w.varint(*last_log_term);
            }
            Rpc::RequestVoteReply { term, granted } => {
                w.u8(T_RVR);
                w.varint(*term);
                w.bool(*granted);
            }
            Rpc::AppendEntries {
                term,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
            } => {
                w.u8(T_AE);
                w.varint(*term);
                w.varint(*prev_log_index);
                w.varint(*prev_log_term);
                w.varint(*leader_commit);
                w.varint(entries.len() as u64);
                for e in entries {
                    put_entry(&mut w, e);
                }
            }
            Rpc::AppendEntriesReply {
                term,
                success,
                match_index,
                conflict_index,
            } => {
                w.u8(T_AER);
                w.varint(*term);
                w.bool(*success);
                w.varint(*match_index);
                w.varint(*conflict_index);
            }
            Rpc::InstallSnapshot {
                term,
                last_included_index,
                last_included_term,
                offset,
                data,
                done,
                config,
            } => {
                w.u8(T_IS);
                w.varint(*term);
                w.varint(*last_included_index);
                w.varint(*last_included_term);
                w.varint(*offset);
                w.bool(*done);
                match config {
                    Some(c) => {
                        w.bool(true);
                        w.nodes(&c.voters);
                        w.nodes(&c.learners);
                    }
                    None => w.bool(false),
                }
                w.bytes(data);
            }
            Rpc::InstallSnapshotReply {
                term,
                last_included_index,
                next_offset,
            } => {
                w.u8(T_ISR);
                w.varint(*term);
                w.varint(*last_included_index);
                w.varint(*next_offset);
            }
            Rpc::PreVote {
                term,
                last_log_index,
                last_log_term,
            } => {
                w.u8(T_PV);
                w.varint(*term);
                w.varint(*last_log_index);
                w.varint(*last_log_term);
            }
            Rpc::PreVoteReply {
                term,
                proposed_term,
                granted,
            } => {
                w.u8(T_PVR);
                w.varint(*term);
                w.varint(*proposed_term);
                w.bool(*granted);
            }
        }
        w.0
    }

    fn from_wire(buf: &[u8]) -> Result<Self, CodecError> {
        let mut r = Reader::new(buf);
        let rpc = match r.u8()? {
            T_RV => Rpc::RequestVote {
                term: r.varint()?,
                last_log_index: r.varint()?,
                last_log_term: r.varint()?,
            },
            T_RVR => Rpc::RequestVoteReply {
                term: r.varint()?,
                granted: r.bool()?,
            },
            T_AE => {
                let term = r.varint()?;
                let prev_log_index = r.varint()?;
                let prev_log_term = r.varint()?;
                let leader_commit = r.varint()?;
                let n = r.varint()?;
                // Every entry needs at least 3 bytes (term, index, empty command).
                if n.saturating_mul(3) > r.remaining() as u64 {
                    return Err(CodecError("entry count exceeds input"));
                }
                let entries = (0..n)
                    .map(|_| get_entry(&mut r))
                    .collect::<Result<Vec<_>, _>>()?;
                Rpc::AppendEntries {
                    term,
                    prev_log_index,
                    prev_log_term,
                    entries,
                    leader_commit,
                }
            }
            T_AER => Rpc::AppendEntriesReply {
                term: r.varint()?,
                success: r.bool()?,
                match_index: r.varint()?,
                conflict_index: r.varint()?,
            },
            T_IS => {
                let term = r.varint()?;
                let last_included_index = r.varint()?;
                let last_included_term = r.varint()?;
                let offset = r.varint()?;
                let done = r.bool()?;
                let config = if r.bool()? {
                    Some(ClusterConfig {
                        voters: r.nodes()?,
                        learners: r.nodes()?,
                    })
                } else {
                    None
                };
                let data = r.bytes()?;
                Rpc::InstallSnapshot {
                    term,
                    last_included_index,
                    last_included_term,
                    offset,
                    data,
                    done,
                    config,
                }
            }
            T_ISR => Rpc::InstallSnapshotReply {
                term: r.varint()?,
                last_included_index: r.varint()?,
                next_offset: r.varint()?,
            },
            T_PV => Rpc::PreVote {
                term: r.varint()?,
                last_log_index: r.varint()?,
                last_log_term: r.varint()?,
            },
            T_PVR => Rpc::PreVoteReply {
                term: r.varint()?,
                proposed_term: r.varint()?,
                granted: r.bool()?,
            },
            _ => return Err(CodecError("unknown rpc tag")),
        };
        r.finish()?;
        Ok(rpc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples() -> Vec<Rpc> {
        let e = |i: u64| LogEntry {
            term: 3,
            index: i,
            command: vec![i as u8; 5],
        };
        vec![
            Rpc::RequestVote {
                term: 300,
                last_log_index: 1 << 40,
                last_log_term: 2,
            },
            Rpc::RequestVoteReply {
                term: 1,
                granted: true,
            },
            Rpc::AppendEntries {
                term: 9,
                prev_log_index: 4,
                prev_log_term: 3,
                entries: vec![e(5), e(6)],
                leader_commit: u64::MAX,
            },
            Rpc::AppendEntriesReply {
                term: 9,
                success: false,
                match_index: 0,
                conflict_index: 7,
            },
            Rpc::InstallSnapshot {
                term: 2,
                last_included_index: 10,
                last_included_term: 2,
                offset: 1024,
                data: vec![7; 100],
                done: true,
                config: Some(ClusterConfig::new(vec![NodeId::new()], vec![NodeId::new()])),
            },
            Rpc::InstallSnapshot {
                term: 2,
                last_included_index: 10,
                last_included_term: 2,
                offset: 0,
                data: vec![],
                done: false,
                config: None,
            },
            Rpc::InstallSnapshotReply {
                term: 2,
                last_included_index: 0,
                next_offset: 99,
            },
            Rpc::PreVote {
                term: 5,
                last_log_index: 1,
                last_log_term: 1,
            },
            Rpc::PreVoteReply {
                term: 4,
                proposed_term: 5,
                granted: false,
            },
        ]
    }

    #[test]
    fn roundtrip_all_variants() {
        for m in samples() {
            let b = m.to_wire();
            assert_eq!(Rpc::from_wire(&b).unwrap(), m);
        }
    }

    #[test]
    fn smaller_than_json() {
        for m in samples() {
            assert!(m.to_wire().len() < serde_json::to_vec(&m).unwrap().len());
        }
    }

    #[test]
    fn truncations_and_garbage_never_panic() {
        for m in samples() {
            let b = m.to_wire();
            for cut in 0..b.len() {
                assert!(Rpc::from_wire(&b[..cut]).is_err(), "cut {cut}");
            }
            let mut extra = b.clone();
            extra.push(0);
            assert!(Rpc::from_wire(&extra).is_err());
        }
        // Hostile entry count / length must not allocate or panic.
        assert!(Rpc::from_wire(&[T_AE, 1, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0x0f]).is_err());
        assert!(Rpc::from_wire(&[0xee]).is_err());
        assert!(Rpc::from_wire(&[T_IS, 1, 1, 1, 0, 0, 0, 0xff, 0xff, 0xff, 0x7f]).is_err());
    }
}
