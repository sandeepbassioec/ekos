//! Real Raft consensus core (paper, figure 2) with durable state.
//!
//! `RaftNode` is a synchronous, transport-agnostic state machine driven by
//! `tick()` (logical time), `step()` (inbound RPC) and `propose()`. Each call
//! returns the outbound RPCs to send. All state that Raft requires to be
//! persistent (term, vote, log, snapshot) is fsynced *before* the call
//! returns, so replies are never visible before they are durable.
//!
//! Beyond the core algorithm this implements:
//! * **Pre-vote** (dissertation 9.6) plus leader stickiness, so a partitioned
//!   or removed node that rejoins cannot depose a healthy leader.
//! * **Single-server membership changes** (dissertation 4.1): a change is a
//!   config entry in the log, effective as soon as it is appended, one
//!   uncommitted change at a time. New servers join as non-voting *learners*
//!   and are promoted to voters automatically once they have caught up.
//! * **Chunked, resumable InstallSnapshot** (offset/done chunks).

use super::config::{ClusterConfig, is_config_command};
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

/// Default InstallSnapshot chunk size.
pub const DEFAULT_SNAPSHOT_CHUNK: usize = 1024 * 1024;
/// Upper bound on a snapshot a follower will assemble in memory.
pub const MAX_SNAPSHOT_BYTES: usize = 1 << 30;
/// Heartbeats during which a removed server is still told about its removal.
const REMOVAL_NOTIFY_HEARTBEATS: u32 = 5;

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
    #[error("command uses the prefix reserved for membership entries")]
    ReservedCommand,
    #[error("membership change refused: {0}")]
    ConfigChange(String),
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
    /// Voting members as this node currently sees them (latest config entry
    /// in its log, committed or not).
    pub voters: Vec<NodeId>,
    pub learners: Vec<NodeId>,
    /// Log index of the latest membership entry (0 = initial configuration).
    pub config_index: u64,
}

pub type Outbound = Vec<(NodeId, Rpc)>;

/// Leader-side progress of a snapshot transfer to one follower.
struct SnapTx {
    /// Next byte to send (the follower's acknowledged resume point).
    offset: u64,
    /// Ticks since the last send or acknowledgement.
    stall: u64,
}

/// Follower-side partially received snapshot.
struct SnapRx {
    index: u64,
    term: u64,
    data: Vec<u8>,
    config: Option<ClusterConfig>,
}

pub struct RaftNode<M: StateMachine> {
    id: NodeId,
    /// Every other member (voters and learners) we replicate to / accept from.
    peers: Vec<NodeId>,
    cfg: RaftConfig,
    storage: FileStorage,
    sm: M,
    rng: StdRng,

    // membership
    config: ClusterConfig,
    /// Membership as of the snapshot (or the bootstrap configuration).
    snap_config: ClusterConfig,
    config_index: u64,
    /// Removed peers still to be told about their removal (leader only).
    leaving: HashMap<NodeId, u32>,

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
    pre_voting: bool,
    pre_votes: HashSet<NodeId>,
    next_index: HashMap<NodeId, u64>,
    match_index: HashMap<NodeId, u64>,
    snap_chunk: usize,
    snap_tx: HashMap<NodeId, SnapTx>,
    snap_rx: Option<SnapRx>,
}

impl<M: StateMachine> RaftNode<M> {
    /// Open (or create) a node backed by `dir`. `peers` excludes `id`; the
    /// initial configuration is `peers + id` unless the log or snapshot on
    /// disk already carries a (newer) configuration.
    pub fn open(
        id: NodeId,
        peers: Vec<NodeId>,
        cfg: RaftConfig,
        dir: impl AsRef<Path>,
        sm: M,
    ) -> Result<Self, RaftError> {
        let mut voters = peers;
        voters.push(id.clone());
        Self::open_with(id, ClusterConfig::new(voters, Vec::new()), cfg, dir, sm)
    }

    /// Open a node that is *joining* an existing cluster. `peers` are the
    /// current members (used to recognise and reach the leader); the node is
    /// not a voter, so it never campaigns, until the leader's configuration
    /// entries reach it.
    pub fn open_joining(
        id: NodeId,
        peers: Vec<NodeId>,
        cfg: RaftConfig,
        dir: impl AsRef<Path>,
        sm: M,
    ) -> Result<Self, RaftError> {
        let peers: Vec<NodeId> = peers.into_iter().filter(|p| *p != id).collect();
        Self::open_with(id, ClusterConfig::new(peers, Vec::new()), cfg, dir, sm)
    }

    fn open_with(
        id: NodeId,
        bootstrap: ClusterConfig,
        cfg: RaftConfig,
        dir: impl AsRef<Path>,
        mut sm: M,
    ) -> Result<Self, RaftError> {
        let (storage, loaded) = FileStorage::open(dir)?;
        let (snap_index, snap_term, snap_data, snap_config) = match loaded.snapshot {
            Some(s) => {
                sm.restore(&s.data);
                let c = s.config.unwrap_or_else(|| bootstrap.clone());
                (s.last_index, s.last_term, s.data, c)
            }
            None => (0, 0, Vec::new(), bootstrap),
        };
        let mut node = RaftNode {
            rng: StdRng::seed_from_u64(cfg.seed),
            id,
            peers: Vec::new(),
            cfg,
            storage,
            sm,
            config: snap_config.clone(),
            snap_config,
            config_index: 0,
            leaving: HashMap::new(),
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
            pre_voting: false,
            pre_votes: HashSet::new(),
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            snap_chunk: DEFAULT_SNAPSHOT_CHUNK,
            snap_tx: HashMap::new(),
            snap_rx: None,
        };
        node.recompute_config();
        node.reset_timer();
        Ok(node)
    }

    /// Set the InstallSnapshot chunk size (bytes, minimum 1).
    pub fn with_snapshot_chunk_size(mut self, bytes: usize) -> Self {
        self.snap_chunk = bytes.max(1);
        self
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
    /// Current membership (latest config entry in this node's log).
    pub fn config(&self) -> &ClusterConfig {
        &self.config
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
            voters: self.config.voters.clone(),
            learners: self.config.learners.clone(),
            config_index: self.config_index,
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

    fn is_voter(&self) -> bool {
        self.config.is_voter(&self.id)
    }

    fn quorum(&self) -> usize {
        self.config.voters.len() / 2 + 1
    }

    fn voters_other_than_self(&self) -> Vec<NodeId> {
        self.config
            .voters
            .iter()
            .filter(|v| **v != self.id)
            .cloned()
            .collect()
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
        self.pre_voting = false;
        self.snap_tx.clear();
        self.persist_hard_state()
    }

    /// True while this node is (or has recently heard from) a live leader and
    /// must therefore not help depose it (leader stickiness).
    fn in_lease(&self) -> bool {
        match self.role {
            RaftRole::Leader => true,
            RaftRole::Follower => {
                self.leader_id.is_some() && self.elapsed < self.cfg.election_ticks.max(1)
            }
            RaftRole::Candidate => false,
        }
    }

    fn log_up_to_date(&self, last_log_index: u64, last_log_term: u64) -> bool {
        last_log_term > self.last_term()
            || (last_log_term == self.last_term() && last_log_index >= self.last_index())
    }

    // ---- membership ------------------------------------------------------

    /// Install `cfg` as the effective configuration (effective-on-append).
    fn set_config(&mut self, cfg: ClusterConfig, index: u64) {
        let old_peers = std::mem::take(&mut self.peers);
        self.peers = cfg
            .voters
            .iter()
            .chain(cfg.learners.iter())
            .filter(|n| **n != self.id)
            .cloned()
            .collect();
        self.config = cfg;
        self.config_index = index;
        if self.role == RaftRole::Leader {
            let next = self.last_index() + 1;
            for p in self.peers.clone() {
                self.next_index.entry(p.clone()).or_insert(next);
                self.match_index.entry(p.clone()).or_insert(0);
                self.leaving.remove(&p);
            }
            for p in old_peers {
                if !self.peers.contains(&p) {
                    self.snap_tx.remove(&p);
                    self.leaving.insert(p, REMOVAL_NOTIFY_HEARTBEATS);
                }
            }
        }
    }

    /// Recompute the effective configuration from the snapshot + log.
    fn recompute_config(&mut self) {
        let found = self
            .log
            .iter()
            .rev()
            .find_map(|e| ClusterConfig::decode_entry(&e.command).map(|c| (c, e.index)));
        match found {
            Some((c, i)) => self.set_config(c, i),
            None => {
                let c = self.snap_config.clone();
                self.set_config(c, 0);
            }
        }
    }

    /// Membership as of log index `idx` (which must be within the log).
    fn config_at(&self, idx: u64) -> ClusterConfig {
        let n = idx.saturating_sub(self.snap_index) as usize;
        self.log
            .iter()
            .take(n)
            .rev()
            .find_map(|e| ClusterConfig::decode_entry(&e.command))
            .unwrap_or_else(|| self.snap_config.clone())
    }

    fn check_change_ready(&self) -> Result<(), RaftError> {
        if self.role != RaftRole::Leader {
            return Err(RaftError::NotLeader(self.leader_id.clone()));
        }
        if self.config_index > self.commit {
            return Err(RaftError::ConfigChange(
                "a previous membership change is not yet committed".into(),
            ));
        }
        if self.term_at(self.commit) != Some(self.term) {
            return Err(RaftError::ConfigChange(
                "leader has not yet committed an entry of its own term".into(),
            ));
        }
        Ok(())
    }

    fn append_config(&mut self, new: ClusterConfig) -> Result<(u64, Outbound), RaftError> {
        let idx = self.append_local(new.encode_entry())?;
        self.set_config(new, idx);
        self.advance_commit()?;
        Ok((idx, self.broadcast_append()))
    }

    /// A learner is "caught up" if it has acknowledged an entry within one
    /// append batch of the commit index.
    fn learner_caught_up(&self, id: &NodeId) -> bool {
        let m = self.match_index.get(id).copied().unwrap_or(0);
        m > 0 && m + self.cfg.max_entries_per_append as u64 >= self.commit
    }

    /// Leader-only: add `id` as a non-voting learner. The leader promotes it
    /// to a voter automatically once it has caught up.
    pub fn add_learner(&mut self, id: NodeId) -> Result<(u64, Outbound), RaftError> {
        self.check_change_ready()?;
        if self.config.contains(&id) {
            return Err(RaftError::ConfigChange(format!(
                "{id:?} is already a member"
            )));
        }
        let mut new = self.config.clone();
        new.learners.push(id);
        new.normalize();
        self.append_config(new)
    }

    /// Leader-only: promote a caught-up learner to voter.
    pub fn promote(&mut self, id: NodeId) -> Result<(u64, Outbound), RaftError> {
        self.check_change_ready()?;
        if !self.config.is_learner(&id) {
            return Err(RaftError::ConfigChange(format!("{id:?} is not a learner")));
        }
        if !self.learner_caught_up(&id) {
            return Err(RaftError::ConfigChange(format!("{id:?} has not caught up")));
        }
        let mut new = self.config.clone();
        new.learners.retain(|n| *n != id);
        new.voters.push(id);
        new.normalize();
        self.append_config(new)
    }

    /// Leader-only: remove a voter or learner. The last voter cannot be
    /// removed. If the leader removes itself it keeps leading until the change
    /// commits and then steps down.
    pub fn remove_server(&mut self, id: NodeId) -> Result<(u64, Outbound), RaftError> {
        self.check_change_ready()?;
        if !self.config.contains(&id) {
            return Err(RaftError::ConfigChange(format!("{id:?} is not a member")));
        }
        let mut new = self.config.clone();
        new.voters.retain(|n| *n != id);
        new.learners.retain(|n| *n != id);
        if new.voters.is_empty() {
            return Err(RaftError::ConfigChange(
                "cannot remove the last voter".into(),
            ));
        }
        self.append_config(new)
    }

    /// Promote one caught-up learner if no change is in flight.
    fn maybe_promote(&mut self, out: &mut Outbound) -> Result<(), RaftError> {
        if self.role != RaftRole::Leader
            || self.config.learners.is_empty()
            || self.check_change_ready().is_err()
        {
            return Ok(());
        }
        let cand = self
            .config
            .learners
            .iter()
            .find(|l| self.learner_caught_up(l))
            .cloned();
        if let Some(l) = cand {
            let (_, o) = self.promote(l)?;
            out.extend(o);
        }
        Ok(())
    }

    // ---- driving ---------------------------------------------------------

    pub fn tick(&mut self) -> Result<Outbound, RaftError> {
        if self.role == RaftRole::Leader {
            for st in self.snap_tx.values_mut() {
                st.stall += 1;
            }
            self.hb_elapsed += 1;
            if self.hb_elapsed >= self.cfg.heartbeat_ticks {
                self.hb_elapsed = 0;
                let mut out = self.broadcast_append();
                self.leaving.retain(|_, c| {
                    *c = c.saturating_sub(1);
                    *c > 0
                });
                self.maybe_promote(&mut out)?;
                return Ok(out);
            }
            return Ok(Vec::new());
        }
        self.elapsed += 1;
        if self.elapsed >= self.timeout {
            if !self.is_voter() {
                // Learners and removed servers never campaign.
                self.reset_timer();
                return Ok(Vec::new());
            }
            return self.start_pre_vote();
        }
        Ok(Vec::new())
    }

    /// Pre-vote (dissertation 9.6): probe whether an election could be won
    /// before touching term or vote.
    fn start_pre_vote(&mut self) -> Result<Outbound, RaftError> {
        self.reset_timer();
        self.pre_voting = true;
        self.pre_votes.clear();
        self.pre_votes.insert(self.id.clone());
        if self.pre_votes.len() >= self.quorum() {
            return self.start_election();
        }
        let proposed = self.term + 1;
        Ok(self
            .voters_other_than_self()
            .into_iter()
            .map(|p| {
                (
                    p,
                    Rpc::PreVote {
                        term: proposed,
                        last_log_index: self.last_index(),
                        last_log_term: self.last_term(),
                    },
                )
            })
            .collect())
    }

    fn start_election(&mut self) -> Result<Outbound, RaftError> {
        self.pre_voting = false;
        self.term += 1;
        self.voted_for = Some(self.id.clone());
        self.role = RaftRole::Candidate;
        self.leader_id = None;
        self.votes.clear();
        self.votes.insert(self.id.clone());
        self.persist_hard_state()?;
        self.reset_timer();
        let mut out = Vec::new();
        if self.votes_won() {
            self.become_leader(&mut out)?;
            return Ok(out);
        }
        for p in self.voters_other_than_self() {
            out.push((
                p,
                Rpc::RequestVote {
                    term: self.term,
                    last_log_index: self.last_index(),
                    last_log_term: self.last_term(),
                },
            ));
        }
        Ok(out)
    }

    fn votes_won(&self) -> bool {
        self.votes
            .iter()
            .filter(|v| self.config.is_voter(v))
            .count()
            >= self.quorum()
    }

    fn become_leader(&mut self, out: &mut Outbound) -> Result<(), RaftError> {
        self.role = RaftRole::Leader;
        self.pre_voting = false;
        self.leader_id = Some(self.id.clone());
        self.hb_elapsed = 0;
        self.snap_tx.clear();
        self.leaving.clear();
        self.next_index.clear();
        self.match_index.clear();
        let next = self.last_index() + 1;
        for p in self.peers.clone() {
            self.next_index.insert(p.clone(), next);
            self.match_index.insert(p, 0);
        }
        // No-op entry so entries of earlier terms can be committed (5.4.2).
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
        if is_config_command(&command) {
            return Err(RaftError::ReservedCommand);
        }
        let idx = self.append_local(command)?;
        self.advance_commit()?;
        Ok((idx, self.broadcast_append()))
    }

    fn broadcast_append(&mut self) -> Outbound {
        let mut targets = self.peers.clone();
        targets.extend(self.leaving.keys().cloned());
        let resend_after = self.cfg.election_ticks.max(1);
        targets
            .into_iter()
            .map(|p| {
                let in_transfer = self
                    .snap_tx
                    .get(&p)
                    .is_some_and(|st| st.stall < resend_after);
                let m = if in_transfer {
                    // A chunk is in flight: keep the follower's election timer
                    // quiet with an empty heartbeat instead of resending it.
                    Rpc::AppendEntries {
                        term: self.term,
                        prev_log_index: 0,
                        prev_log_term: 0,
                        entries: Vec::new(),
                        leader_commit: 0,
                    }
                } else {
                    self.append_for(&p)
                };
                (p, m)
            })
            .collect()
    }

    fn append_for(&mut self, peer: &NodeId) -> Rpc {
        let last = self.last_index();
        let next = (*self.next_index.get(peer).unwrap_or(&(last + 1))).clamp(1, last + 1);
        if next <= self.snap_index {
            return self.snapshot_chunk_for(peer);
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

    fn snapshot_chunk_for(&mut self, peer: &NodeId) -> Rpc {
        let total = self.snap_data.len();
        let st = self.snap_tx.entry(peer.clone()).or_insert(SnapTx {
            offset: 0,
            stall: 0,
        });
        st.stall = 0;
        if st.offset as usize > total {
            st.offset = 0;
        }
        let off = st.offset as usize;
        let end = (off + self.snap_chunk).min(total);
        Rpc::InstallSnapshot {
            term: self.term,
            last_included_index: self.snap_index,
            last_included_term: self.snap_term,
            offset: off as u64,
            data: self.snap_data[off..end].to_vec(),
            done: end == total,
            config: Some(self.snap_config.clone()),
        }
    }

    /// Handle an inbound RPC from `from`.
    pub fn step(&mut self, from: NodeId, rpc: Rpc) -> Result<Outbound, RaftError> {
        if !self.peers.contains(&from) {
            return Ok(Vec::new()); // not a member
        }
        // Pre-vote never changes term, vote or role of the receiver.
        if let Rpc::PreVote {
            term,
            last_log_index,
            last_log_term,
        } = &rpc
        {
            let granted = self.config.is_voter(&from)
                && *term > self.term
                && !self.in_lease()
                && self.log_up_to_date(*last_log_index, *last_log_term);
            return Ok(vec![(
                from,
                Rpc::PreVoteReply {
                    term: self.term,
                    proposed_term: *term,
                    granted,
                },
            )]);
        }
        // Leader stickiness: while a leader is alive, a higher-term vote
        // request (e.g. from a removed or rejoining server) is refused
        // without even adopting its term.
        if let Rpc::RequestVote { term, .. } = &rpc
            && *term > self.term
            && self.in_lease()
        {
            return Ok(vec![(
                from,
                Rpc::RequestVoteReply {
                    term: self.term,
                    granted: false,
                },
            )]);
        }
        if rpc.term() > self.term {
            self.advance_term(rpc.term())?;
        }
        let reply = match rpc {
            Rpc::PreVote { .. } => None,
            Rpc::PreVoteReply {
                proposed_term,
                granted,
                ..
            } => return self.on_pre_vote_reply(from, proposed_term, granted),
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
                offset,
                data,
                done,
                config,
            } => Some(self.on_install_snapshot(
                &from,
                term,
                (last_included_index, last_included_term),
                (offset, done),
                data,
                config,
            )?),
            Rpc::InstallSnapshotReply {
                term,
                last_included_index,
                next_offset,
            } => return self.on_snapshot_reply(from, term, last_included_index, next_offset),
        };
        Ok(reply.map(|r| vec![(from, r)]).unwrap_or_default())
    }

    // ---- elections -------------------------------------------------------

    fn on_pre_vote_reply(
        &mut self,
        from: NodeId,
        proposed_term: u64,
        granted: bool,
    ) -> Result<Outbound, RaftError> {
        if !self.pre_voting
            || !granted
            || proposed_term != self.term + 1
            || !self.config.is_voter(&from)
        {
            return Ok(Vec::new());
        }
        self.pre_votes.insert(from);
        let n = self
            .pre_votes
            .iter()
            .filter(|v| self.config.is_voter(v))
            .count();
        if n >= self.quorum() {
            return self.start_election();
        }
        Ok(Vec::new())
    }

    fn on_request_vote(
        &mut self,
        from: &NodeId,
        term: u64,
        last_log_index: u64,
        last_log_term: u64,
    ) -> Result<Rpc, RaftError> {
        if term < self.term || !self.config.is_voter(from) {
            return Ok(Rpc::RequestVoteReply {
                term: self.term,
                granted: false,
            });
        }
        // Election restriction (5.4.1): candidate's log must be at least as up to date.
        let up_to_date = self.log_up_to_date(last_log_index, last_log_term);
        let can_vote = self.voted_for.is_none() || self.voted_for.as_ref() == Some(from);
        if can_vote && up_to_date {
            if self.voted_for.is_none() {
                self.voted_for = Some(from.clone());
                self.persist_hard_state()?; // durable before the grant leaves
            }
            self.pre_voting = false;
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
        if self.votes_won() {
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
        self.pre_voting = false;
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
            // Membership entries take effect on append, not on commit.
            let newest_cfg = fresh
                .iter()
                .rev()
                .find_map(|e| ClusterConfig::decode_entry(&e.command).map(|c| (c, e.index)));
            self.log.extend(fresh);
            if let Some((c, i)) = newest_cfg {
                self.set_config(c, i);
            }
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
        if self.config_index >= index {
            self.recompute_config();
        }
        Ok(())
    }

    fn on_install_snapshot(
        &mut self,
        from: &NodeId,
        term: u64,
        (last_index, last_term): (u64, u64),
        (offset, done): (u64, bool),
        data: Vec<u8>,
        config: Option<ClusterConfig>,
    ) -> Result<Rpc, RaftError> {
        let reply = |s: &Self, installed: u64, next: u64| Rpc::InstallSnapshotReply {
            term: s.term,
            last_included_index: installed,
            next_offset: next,
        };
        if term < self.term {
            return Ok(reply(self, 0, 0));
        }
        self.role = RaftRole::Follower;
        self.leader_id = Some(from.clone());
        self.pre_voting = false;
        self.reset_timer();
        if last_index <= self.commit {
            // Already have everything this snapshot covers.
            self.snap_rx = None;
            return Ok(reply(self, self.commit, 0));
        }

        let same = self
            .snap_rx
            .as_ref()
            .is_some_and(|r| r.index == last_index && r.term == last_term);
        if offset == 0 {
            self.snap_rx = Some(SnapRx {
                index: last_index,
                term: last_term,
                data: Vec::new(),
                config: None,
            });
        } else {
            let have = match (&self.snap_rx, same) {
                (Some(r), true) => r.data.len() as u64,
                _ => {
                    self.snap_rx = None;
                    return Ok(reply(self, 0, 0));
                }
            };
            if have != offset {
                // Duplicate or gap: tell the leader where to resume.
                return Ok(reply(self, 0, have));
            }
        }
        let rx = self.snap_rx.as_mut().expect("snapshot receiver");
        if rx.data.len() + data.len() > MAX_SNAPSHOT_BYTES {
            self.snap_rx = None;
            return Ok(reply(self, 0, 0));
        }
        rx.data.extend_from_slice(&data);
        if config.is_some() {
            rx.config = config;
        }
        if !done {
            let next = rx.data.len() as u64;
            return Ok(reply(self, 0, next));
        }

        let rx = self.snap_rx.take().expect("snapshot receiver");
        let keep_suffix =
            last_index <= self.last_index() && self.term_at(last_index) == Some(last_term);
        let remaining: Vec<LogEntry> = if keep_suffix {
            self.log[(last_index - self.snap_index) as usize..].to_vec()
        } else {
            Vec::new()
        };
        let snap_config = rx.config.unwrap_or_else(|| self.config.clone());
        let snap = Snapshot {
            last_index,
            last_term,
            data: rx.data,
            config: Some(snap_config.clone()),
        };
        self.storage.save_snapshot(&snap, &remaining)?;
        self.sm.restore(&snap.data);
        self.snap_index = last_index;
        self.snap_term = last_term;
        self.snap_data = snap.data;
        self.snap_config = snap_config;
        self.log = remaining;
        self.commit = last_index;
        self.applied = last_index;
        self.recompute_config();
        Ok(reply(self, last_index, 0))
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
        if self.snap_tx.contains_key(&from) {
            // Mid snapshot transfer: replies to heartbeats carry no information.
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
        next_offset: u64,
    ) -> Result<Outbound, RaftError> {
        if self.role != RaftRole::Leader || term != self.term {
            return Ok(Vec::new());
        }
        if last_index == 0 {
            // Chunk acknowledged (or the follower asks us to resume elsewhere).
            let Some(st) = self.snap_tx.get_mut(&from) else {
                return Ok(Vec::new());
            };
            if st.offset == next_offset {
                return Ok(Vec::new()); // duplicate acknowledgement
            }
            st.offset = next_offset;
            st.stall = 0;
            let r = self.append_for(&from);
            return Ok(vec![(from, r)]);
        }
        self.snap_tx.remove(&from);
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

    /// Commit the highest index replicated on a majority of the *current*
    /// voters **and** belonging to the current term (5.4.2).
    fn advance_commit(&mut self) -> Result<(), RaftError> {
        if self.role != RaftRole::Leader {
            return Ok(());
        }
        let mut n = self.last_index();
        while n > self.commit {
            if self.term_at(n) != Some(self.term) {
                break;
            }
            let count = self
                .config
                .voters
                .iter()
                .filter(|v| **v == self.id || self.match_index.get(*v).copied().unwrap_or(0) >= n)
                .count();
            if count >= self.quorum() {
                self.commit = n;
                break;
            }
            n -= 1;
        }
        self.apply_committed()?;
        // A leader that is no longer a voter steps down once the change that
        // removed it has committed (dissertation 4.2.2).
        if self.role == RaftRole::Leader
            && !self.is_voter()
            && self.config_index > 0
            && self.config_index <= self.commit
        {
            self.role = RaftRole::Follower;
            self.leader_id = None;
            self.snap_tx.clear();
            self.reset_timer();
        }
        Ok(())
    }

    fn apply_committed(&mut self) -> Result<(), RaftError> {
        while self.applied < self.commit {
            self.applied += 1;
            let e = &self.log[(self.applied - self.snap_index - 1) as usize];
            if !e.command.is_empty() && !is_config_command(&e.command) {
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
        let config = self.config_at(idx);
        let cut = (idx - self.snap_index) as usize;
        let snap = Snapshot {
            last_index: idx,
            last_term: term,
            data,
            config: Some(config.clone()),
        };
        self.storage.save_snapshot(&snap, &self.log[cut..])?;
        self.log.drain(..cut);
        self.snap_index = idx;
        self.snap_term = term;
        self.snap_data = snap.data;
        self.snap_config = config;
        // In-flight transfers refer to the previous snapshot; restart them.
        self.snap_tx.clear();
        Ok(true)
    }
}
