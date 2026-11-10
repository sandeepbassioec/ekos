//! Real Raft consensus core (paper, figure 2) with durable state.
//!
//! `RaftNode` is a synchronous, transport-agnostic state machine driven by
//! `tick()` (logical time), `step()` (inbound RPC) and `propose()`. Each call
//! returns the outbound RPCs to send. All state that Raft requires to be
//! persistent (term, vote, log, snapshot) is fsynced *before* the call
//! returns, so replies are never visible before they are durable.

use super::log::LogEntry;
use super::rpc::Rpc;
use super::state::RaftRole;
use super::storage::{FileStorage, HardState, Snapshot};
use crate::node::NodeId;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::Path;

/// Application state machine driven by committed log entries.
pub trait StateMachine: Send {
    /// Apply a committed, non-empty command. Called exactly once per index
    /// (per process lifetime), in order.
    fn apply(&mut self, index: u64, command: &[u8]);
    /// Serialize the full state (state after the last applied entry).
    fn snapshot(&self) -> Vec<u8>;
    /// Replace the state with a previously produced snapshot.
    fn restore(&mut self, data: &[u8]);
}

#[derive(Debug, thiserror::Error)]
pub enum RaftError {
    #[error("storage error: {0}")]
    Io(#[from] io::Error),
    #[error("not leader (leader hint: {0:?})")]
    NotLeader(Option<NodeId>),
    #[error("empty commands are reserved for leader no-ops")]
    EmptyCommand,
}

#[derive(Debug, Clone)]
pub struct RaftConfig {
    /// Minimum election timeout in ticks; actual timeout is uniform in
    /// `[election_ticks, 2*election_ticks)`.
    pub election_ticks: u64,
    pub heartbeat_ticks: u64,
    /// Auto-snapshot once this many applied entries follow the last snapshot
    /// (0 disables).
    pub snapshot_threshold: u64,
    pub max_entries_per_append: usize,
    /// Seed for the randomized election timeouts (deterministic tests).
    pub seed: u64,
}

impl Default for RaftConfig {
    fn default() -> Self {
        RaftConfig {
            election_ticks: 10,
            heartbeat_ticks: 2,
            snapshot_threshold: 0,
            max_entries_per_append: 64,
            seed: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Status {
    pub id: NodeId,
    pub role: RaftRole,
    pub term: u64,
    pub leader_id: Option<NodeId>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub last_index: u64,
    pub snapshot_index: u64,
}

pub type Outbound = Vec<(NodeId, Rpc)>;

pub struct RaftNode<M: StateMachine> {
    id: NodeId,
    peers: Vec<NodeId>,
    cfg: RaftConfig,
    storage: FileStorage,
    sm: M,
    rng: StdRng,

    // persistent
    term: u64,
    voted_for: Option<NodeId>,
    snap_index: u64,
    snap_term: u64,
    snap_data: Vec<u8>,
    /// Entries with index `snap_index + 1 ..`.
    log: Vec<LogEntry>,

    // volatile
    role: RaftRole,
    leader_id: Option<NodeId>,
    commit: u64,
    applied: u64,
    elapsed: u64,
    timeout: u64,
    hb_elapsed: u64,
    votes: HashSet<NodeId>,
    next_index: HashMap<NodeId, u64>,
    match_index: HashMap<NodeId, u64>,
}

impl<M: StateMachine> RaftNode<M> {
    /// Open (or create) a node backed by `dir`. `peers` excludes `id`.
    pub fn open(
        id: NodeId,
        peers: Vec<NodeId>,
        cfg: RaftConfig,
        dir: impl AsRef<Path>,
        mut sm: M,
    ) -> Result<Self, RaftError> {
        let (storage, loaded) = FileStorage::open(dir)?;
        let (snap_index, snap_term, snap_data) = match loaded.snapshot {
            Some(s) => {
                sm.restore(&s.data);
                (s.last_index, s.last_term, s.data)
            }
            None => (0, 0, Vec::new()),
        };
        let mut node = RaftNode {
            rng: StdRng::seed_from_u64(cfg.seed),
            id,
            peers,
            cfg,
            storage,
            sm,
            term: loaded.hard_state.term,
            voted_for: loaded.hard_state.voted_for,
            snap_index,
            snap_term,
            snap_data,
            log: loaded.entries,
            role: RaftRole::Follower,
            leader_id: None,
            commit: snap_index,
            applied: snap_index,
            elapsed: 0,
            timeout: 0,
            hb_elapsed: 0,
            votes: HashSet::new(),
            next_index: HashMap::new(),
            match_index: HashMap::new(),
        };
        node.reset_timer();
        Ok(node)
    }

    // ---- accessors -------------------------------------------------------

    pub fn id(&self) -> &NodeId {
        &self.id
    }
    pub fn role(&self) -> RaftRole {
        self.role.clone()
    }
    pub fn term(&self) -> u64 {
        self.term
    }
    pub fn voted_for(&self) -> Option<&NodeId> {
        self.voted_for.as_ref()
    }
    pub fn leader_id(&self) -> Option<&NodeId> {
        self.leader_id.as_ref()
    }
    pub fn commit_index(&self) -> u64 {
        self.commit
    }
    pub fn last_applied(&self) -> u64 {
        self.applied
    }
    pub fn last_index(&self) -> u64 {
        self.snap_index + self.log.len() as u64
    }
    pub fn snapshot_index(&self) -> u64 {
        self.snap_index
    }
    pub fn state_machine(&self) -> &M {
        &self.sm
    }
    pub fn status(&self) -> Status {
        Status {
            id: self.id.clone(),
            role: self.role.clone(),
            term: self.term,
            leader_id: self.leader_id.clone(),
            commit_index: self.commit,
            last_applied: self.applied,
            last_index: self.last_index(),
            snapshot_index: self.snap_index,
        }
    }

    fn last_term(&self) -> u64 {
        self.log.last().map_or(self.snap_term, |e| e.term)
    }

    fn term_at(&self, index: u64) -> Option<u64> {
        if index == self.snap_index {
            Some(self.snap_term)
        } else if index < self.snap_index || index > self.last_index() {
            None
        } else {
            Some(self.log[(index - self.snap_index - 1) as usize].term)
        }
    }

    fn quorum(&self) -> usize {
        let cluster_size = self.peers.len() + 1;
        cluster_size / 2 + 1
    }

    fn reset_timer(&mut self) {
        self.elapsed = 0;
        let base = self.cfg.election_ticks.max(1);
        self.timeout = base + self.rng.gen_range(0..base);
    }

    fn persist_hard_state(&mut self) -> Result<(), RaftError> {
        self.storage.save_hard_state(&HardState {
            term: self.term,
            voted_for: self.voted_for.clone(),
        })?;
        Ok(())
    }

    /// Adopt a (strictly greater) term: clears the vote, persists, becomes follower.
    fn advance_term(&mut self, term: u64) -> Result<(), RaftError> {
        debug_assert!(term > self.term);
        self.term = term;
        self.voted_for = None;
        self.role = RaftRole::Follower;
        self.leader_id = None;
        self.votes.clear();
        self.persist_hard_state()
    }

    // ---- driving ---------------------------------------------------------

    pub fn tick(&mut self) -> Result<Outbound, RaftError> {
        if self.role == RaftRole::Leader {
            self.hb_elapsed += 1;
            if self.hb_elapsed >= self.cfg.heartbeat_ticks {
                self.hb_elapsed = 0;
                return Ok(self.broadcast_append());
            }
            return Ok(Vec::new());
        }
        self.elapsed += 1;
        if self.elapsed >= self.timeout {
            return self.start_election();
        }
        Ok(Vec::new())
    }

    fn start_election(&mut self) -> Result<Outbound, RaftError> {
        self.term += 1;
        self.voted_for = Some(self.id.clone());
        self.role = RaftRole::Candidate;
        self.leader_id = None;
        self.votes.clear();
        self.votes.insert(self.id.clone());
        self.persist_hard_state()?;
        self.reset_timer();
        let mut out = Vec::new();
        if self.votes.len() >= self.quorum() {
            self.become_leader(&mut out)?;
            return Ok(out);
        }
        for p in &self.peers {
            out.push((
                p.clone(),
                Rpc::RequestVote {
                    term: self.term,
                    last_log_index: self.last_index(),
                    last_log_term: self.last_term(),
                },
            ));
        }
        Ok(out)
    }

    fn become_leader(&mut self, out: &mut Outbound) -> Result<(), RaftError> {
        self.role = RaftRole::Leader;
        self.leader_id = Some(self.id.clone());
        self.hb_elapsed = 0;
        let next = self.last_index() + 1;
        for p in self.peers.clone() {
            self.next_index.insert(p.clone(), next);
            self.match_index.insert(p, 0);
        }
        // No-op entry so entries of earlier terms can be committed (§5.4.2).
        self.append_local(Vec::new())?;
        self.advance_commit()?;
        out.extend(self.broadcast_append());
        Ok(())
    }

    fn append_local(&mut self, command: Vec<u8>) -> Result<u64, RaftError> {
        let entry = LogEntry {
            term: self.term,
            index: self.last_index() + 1,
            command,
        };
        self.storage.append(std::slice::from_ref(&entry))?;
        let idx = entry.index;
        self.log.push(entry);
        Ok(idx)
    }

    /// Leader-only: append a client command. Returns its log index.
    pub fn propose(&mut self, command: Vec<u8>) -> Result<(u64, Outbound), RaftError> {
        if self.role != RaftRole::Leader {
            return Err(RaftError::NotLeader(self.leader_id.clone()));
        }
        if command.is_empty() {
            return Err(RaftError::EmptyCommand);
        }
        let idx = self.append_local(command)?;
        self.advance_commit()?;
        Ok((idx, self.broadcast_append()))
    }

    fn broadcast_append(&mut self) -> Outbound {
        let peers = self.peers.clone();
        peers
            .into_iter()
            .map(|p| {
                let m = self.append_for(&p);
                (p, m)
            })
            .collect()
    }

    fn append_for(&mut self, peer: &NodeId) -> Rpc {
        let last = self.last_index();
        let next = (*self.next_index.get(peer).unwrap_or(&(last + 1))).clamp(1, last + 1);
        if next <= self.snap_index {
            return Rpc::InstallSnapshot {
                term: self.term,
                last_included_index: self.snap_index,
                last_included_term: self.snap_term,
                data: self.snap_data.clone(),
            };
        }
        let prev = next - 1;
        let prev_term = self.term_at(prev).unwrap_or(0);
        let start = (next - self.snap_index - 1) as usize;
        let end = (start + self.cfg.max_entries_per_append).min(self.log.len());
        Rpc::AppendEntries {
            term: self.term,
            prev_log_index: prev,
            prev_log_term: prev_term,
            entries: self.log[start..end].to_vec(),
            leader_commit: self.commit,
        }
    }

    /// Handle an inbound RPC from `from`.
    pub fn step(&mut self, from: NodeId, rpc: Rpc) -> Result<Outbound, RaftError> {
        if !self.peers.contains(&from) {
            return Ok(Vec::new()); // not a member
        }
        if rpc.term() > self.term {
            self.advance_term(rpc.term())?;
        }
        let reply = match rpc {
            Rpc::RequestVote {
                term,
                last_log_index,
                last_log_term,
            } => Some(self.on_request_vote(&from, term, last_log_index, last_log_term)?),
            Rpc::RequestVoteReply { term, granted } => {
                return self.on_vote_reply(from, term, granted);
            }
            Rpc::AppendEntries {
                term,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
            } => Some(self.on_append(
                &from,
                term,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit,
            )?),
            Rpc::AppendEntriesReply {
                term,
                success,
                match_index,
                conflict_index,
            } => return self.on_append_reply(from, term, success, match_index, conflict_index),
            Rpc::InstallSnapshot {
                term,
                last_included_index,
                last_included_term,
                data,
            } => Some(self.on_install_snapshot(
                &from,
                term,
                last_included_index,
                last_included_term,
                data,
            )?),
            Rpc::InstallSnapshotReply {
                term,
                last_included_index,
            } => return self.on_snapshot_reply(from, term, last_included_index),
        };
        Ok(reply.map(|r| vec![(from, r)]).unwrap_or_default())
    }

    // ---- elections -------------------------------------------------------

    fn on_request_vote(
        &mut self,
        from: &NodeId,
        term: u64,
        last_log_index: u64,
        last_log_term: u64,
    ) -> Result<Rpc, RaftError> {
        if term < self.term {
            return Ok(Rpc::RequestVoteReply {
                term: self.term,
                granted: false,
            });
        }
        // Election restriction (§5.4.1): candidate's log must be at least as up to date.
        let up_to_date = last_log_term > self.last_term()
            || (last_log_term == self.last_term() && last_log_index >= self.last_index());
        let can_vote = self.voted_for.is_none() || self.voted_for.as_ref() == Some(from);
        if can_vote && up_to_date {
            if self.voted_for.is_none() {
                self.voted_for = Some(from.clone());
                self.persist_hard_state()?; // durable before the grant leaves
            }
            self.reset_timer();
            return Ok(Rpc::RequestVoteReply {
                term: self.term,
                granted: true,
            });
        }
        Ok(Rpc::RequestVoteReply {
            term: self.term,
            granted: false,
        })
    }

    fn on_vote_reply(
        &mut self,
        from: NodeId,
        term: u64,
        granted: bool,
    ) -> Result<Outbound, RaftError> {
        if self.role != RaftRole::Candidate || term != self.term || !granted {
            return Ok(Vec::new());
        }
        self.votes.insert(from);
        let mut out = Vec::new();
        if self.votes.len() >= self.quorum() {
            self.become_leader(&mut out)?;
        }
        Ok(out)
    }

    // ---- replication: follower side -------------------------------------

    fn on_append(
        &mut self,
        from: &NodeId,
        term: u64,
        mut prev_index: u64,
        prev_term: u64,
        mut entries: Vec<LogEntry>,
        leader_commit: u64,
    ) -> Result<Rpc, RaftError> {
        let fail = |s: &Self, conflict: u64| Rpc::AppendEntriesReply {
            term: s.term,
            success: false,
            match_index: 0,
            conflict_index: conflict,
        };
        if term < self.term {
            return Ok(fail(self, 0));
        }
        // Same term: a valid leader exists; candidates step down.
        self.role = RaftRole::Follower;
        self.leader_id = Some(from.clone());
        self.reset_timer();

        if prev_index < self.snap_index {
            // Everything up to snap_index is committed and therefore identical.
            entries.retain(|e| e.index > self.snap_index);
            prev_index = self.snap_index;
        } else if prev_index > self.last_index() {
            return Ok(fail(self, self.last_index() + 1));
        } else if self.term_at(prev_index) != Some(prev_term) {
            let ct = self.term_at(prev_index).unwrap_or(0);
            let mut first = prev_index;
            while first > self.snap_index + 1 && self.term_at(first - 1) == Some(ct) {
                first -= 1;
            }
            return Ok(fail(self, first));
        }

        let ack = entries.last().map_or(prev_index, |e| e.index);
        let mut fresh: Vec<LogEntry> = Vec::new();
        for e in entries {
            if fresh.is_empty() {
                if e.index <= self.last_index() {
                    if self.term_at(e.index) == Some(e.term) {
                        continue; // already have it
                    }
                    if e.index <= self.commit {
                        // Must never happen for a correct leader; refuse.
                        return Ok(fail(self, self.commit + 1));
                    }
                    self.truncate_from(e.index)?;
                } else if e.index != self.last_index() + 1 {
                    return Ok(fail(self, self.last_index() + 1));
                }
            }
            fresh.push(e);
        }
        if !fresh.is_empty() {
            self.storage.append(&fresh)?; // fsync before ack
            self.log.extend(fresh);
        }
        if leader_commit > self.commit {
            let c = leader_commit.min(ack);
            if c > self.commit {
                self.commit = c;
                self.apply_committed()?;
            }
        }
        Ok(Rpc::AppendEntriesReply {
            term: self.term,
            success: true,
            match_index: ack,
            conflict_index: 0,
        })
    }

    fn truncate_from(&mut self, index: u64) -> Result<(), RaftError> {
        self.storage.truncate_from(index)?;
        self.log.truncate((index - self.snap_index - 1) as usize);
        Ok(())
    }

    fn on_install_snapshot(
        &mut self,
        from: &NodeId,
        term: u64,
        last_index: u64,
        last_term: u64,
        data: Vec<u8>,
    ) -> Result<Rpc, RaftError> {
        if term < self.term {
            return Ok(Rpc::InstallSnapshotReply {
                term: self.term,
                last_included_index: 0,
            });
        }
        self.role = RaftRole::Follower;
        self.leader_id = Some(from.clone());
        self.reset_timer();
        if last_index <= self.commit {
            // Already have everything this snapshot covers.
            return Ok(Rpc::InstallSnapshotReply {
                term: self.term,
                last_included_index: self.commit,
            });
        }
        let keep_suffix =
            last_index <= self.last_index() && self.term_at(last_index) == Some(last_term);
        let remaining: Vec<LogEntry> = if keep_suffix {
            self.log[(last_index - self.snap_index) as usize..].to_vec()
        } else {
            Vec::new()
        };
        let snap = Snapshot {
            last_index,
            last_term,
            data,
        };
        self.storage.save_snapshot(&snap, &remaining)?;
        self.sm.restore(&snap.data);
        self.snap_index = last_index;
        self.snap_term = last_term;
        self.snap_data = snap.data;
        self.log = remaining;
        self.commit = last_index;
        self.applied = last_index;
        Ok(Rpc::InstallSnapshotReply {
            term: self.term,
            last_included_index: last_index,
        })
    }

    // ---- replication: leader side ---------------------------------------

    fn on_append_reply(
        &mut self,
        from: NodeId,
        term: u64,
        success: bool,
        match_index: u64,
        conflict_index: u64,
    ) -> Result<Outbound, RaftError> {
        if self.role != RaftRole::Leader || term != self.term {
            return Ok(Vec::new());
        }
        let cur_match = *self.match_index.get(&from).unwrap_or(&0);
        if success {
            let m = match_index.max(cur_match).min(self.last_index());
            self.match_index.insert(from.clone(), m);
            self.next_index.insert(from.clone(), m + 1);
            self.advance_commit()?;
            if m < self.last_index() {
                let r = self.append_for(&from);
                return Ok(vec![(from, r)]);
            }
            return Ok(Vec::new());
        }
        let next = *self.next_index.get(&from).unwrap_or(&1);
        let new_next = conflict_index
            .max(1)
            .min(next.saturating_sub(1).max(1))
            .max(cur_match + 1);
        self.next_index.insert(from.clone(), new_next);
        let r = self.append_for(&from);
        Ok(vec![(from, r)])
    }

    fn on_snapshot_reply(
        &mut self,
        from: NodeId,
        term: u64,
        last_index: u64,
    ) -> Result<Outbound, RaftError> {
        if self.role != RaftRole::Leader || term != self.term || last_index == 0 {
            return Ok(Vec::new());
        }
        let m = last_index
            .max(*self.match_index.get(&from).unwrap_or(&0))
            .min(self.last_index());
        self.match_index.insert(from.clone(), m);
        self.next_index.insert(from.clone(), m + 1);
        self.advance_commit()?;
        if m < self.last_index() {
            let r = self.append_for(&from);
            return Ok(vec![(from, r)]);
        }
        Ok(Vec::new())
    }

    /// Commit the highest index replicated on a majority **and** belonging
    /// to the current term (§5.4.2).
    fn advance_commit(&mut self) -> Result<(), RaftError> {
        if self.role != RaftRole::Leader {
            return Ok(());
        }
        let mut n = self.last_index();
        while n > self.commit {
            if self.term_at(n) != Some(self.term) {
                break;
            }
            let count = 1 + self
                .peers
                .iter()
                .filter(|p| self.match_index.get(*p).copied().unwrap_or(0) >= n)
                .count();
            if count >= self.quorum() {
                self.commit = n;
                break;
            }
            n -= 1;
        }
        self.apply_committed()
    }

    fn apply_committed(&mut self) -> Result<(), RaftError> {
        while self.applied < self.commit {
            self.applied += 1;
            let e = &self.log[(self.applied - self.snap_index - 1) as usize];
            if !e.command.is_empty() {
                self.sm.apply(e.index, &e.command);
            }
        }
        if self.cfg.snapshot_threshold > 0
            && self.applied - self.snap_index >= self.cfg.snapshot_threshold
        {
            self.take_snapshot()?;
        }
        Ok(())
    }

    /// Snapshot the state machine at `last_applied` and compact the log.
    /// Returns false if there is nothing new to snapshot.
    pub fn take_snapshot(&mut self) -> Result<bool, RaftError> {
        if self.applied <= self.snap_index {
            return Ok(false);
        }
        let idx = self.applied;
        let term = self.term_at(idx).unwrap_or(self.snap_term);
        let data = self.sm.snapshot();
        let cut = (idx - self.snap_index) as usize;
        let snap = Snapshot {
            last_index: idx,
            last_term: term,
            data,
        };
        self.storage.save_snapshot(&snap, &self.log[cut..])?;
        self.log.drain(..cut);
        self.snap_index = idx;
        self.snap_term = term;
        self.snap_data = snap.data;
        Ok(true)
    }
}
