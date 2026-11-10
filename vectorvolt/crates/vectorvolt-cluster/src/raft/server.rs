//! Async driver binding a [`RaftNode`] to a [`RaftTransport`].

use super::node::{Outbound, RaftError, RaftNode, StateMachine, Status};
use crate::node::NodeId;
use crate::transport::RaftTransport;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

enum Cmd {
    Propose(Vec<u8>, oneshot::Sender<Result<u64, RaftError>>),
    Status(oneshot::Sender<Status>),
    Snapshot(oneshot::Sender<Result<bool, RaftError>>),
    Member(MemberOp, NodeId, oneshot::Sender<Result<u64, RaftError>>),
}

#[derive(Clone, Copy)]
enum MemberOp {
    AddLearner,
    Promote,
    Remove,
}

/// Handle to a running Raft node task.
pub struct RaftHandle {
    tx: mpsc::Sender<Cmd>,
    join: JoinHandle<()>,
}

impl RaftHandle {
    /// Propose a command; returns its log index once appended on the leader
    /// (not yet committed — observe `Status::commit_index`).
    pub async fn propose(&self, command: Vec<u8>) -> Result<u64, RaftError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Cmd::Propose(command, tx))
            .await
            .map_err(|_| closed())?;
        rx.await.map_err(|_| closed())?
    }

    pub async fn status(&self) -> Option<Status> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Cmd::Status(tx)).await.ok()?;
        rx.await.ok()
    }

    async fn member(&self, op: MemberOp, id: NodeId) -> Result<u64, RaftError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Cmd::Member(op, id, tx))
            .await
            .map_err(|_| closed())?;
        rx.await.map_err(|_| closed())?
    }

    /// Leader-only: add `id` as a non-voting learner; the leader promotes it to
    /// voter automatically once it has caught up. Returns the config entry index.
    pub async fn add_server(&self, id: NodeId) -> Result<u64, RaftError> {
        self.member(MemberOp::AddLearner, id).await
    }

    /// Leader-only: explicitly promote a caught-up learner to voter.
    pub async fn promote_server(&self, id: NodeId) -> Result<u64, RaftError> {
        self.member(MemberOp::Promote, id).await
    }

    /// Leader-only: remove a server (possibly the leader itself, which then
    /// steps down once the change commits).
    pub async fn remove_server(&self, id: NodeId) -> Result<u64, RaftError> {
        self.member(MemberOp::Remove, id).await
    }

    pub async fn snapshot(&self) -> Result<bool, RaftError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Cmd::Snapshot(tx))
            .await
            .map_err(|_| closed())?;
        rx.await.map_err(|_| closed())?
    }

    /// Stop the node task abruptly (models a crash: nothing is flushed that
    /// was not already fsynced).
    pub fn crash(&self) {
        self.join.abort();
    }

    pub fn is_finished(&self) -> bool {
        self.join.is_finished()
    }

    /// Wait for the task to end after `crash`, releasing its files.
    pub async fn join(self) {
        let _ = self.join.await;
    }
}

fn closed() -> RaftError {
    RaftError::Io(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "raft task stopped",
    ))
}

/// Spawn the driver loop. `tick` is the wall-clock length of one logical tick.
pub fn spawn<M, T>(mut node: RaftNode<M>, transport: Arc<T>, tick: Duration) -> RaftHandle
where
    M: StateMachine + 'static,
    T: RaftTransport,
{
    let (tx, mut rx) = mpsc::channel::<Cmd>(256);
    let join = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tick);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            let out: Result<Outbound, RaftError> = tokio::select! {
                biased;
                cmd = rx.recv() => match cmd {
                    None => return,
                    Some(Cmd::Propose(c, reply)) => match node.propose(c) {
                        Ok((idx, out)) => { let _ = reply.send(Ok(idx)); Ok(out) }
                        Err(e @ RaftError::Io(_)) => { let _ = reply.send(Err(closed())); Err(e) }
                        Err(e) => { let _ = reply.send(Err(e)); Ok(Vec::new()) }
                    },
                    Some(Cmd::Member(op, id, reply)) => {
                        let r = match op {
                            MemberOp::AddLearner => node.add_learner(id),
                            MemberOp::Promote => node.promote(id),
                            MemberOp::Remove => node.remove_server(id),
                        };
                        match r {
                            Ok((idx, out)) => { let _ = reply.send(Ok(idx)); Ok(out) }
                            Err(e @ RaftError::Io(_)) => { let _ = reply.send(Err(closed())); Err(e) }
                            Err(e) => { let _ = reply.send(Err(e)); Ok(Vec::new()) }
                        }
                    }
                    Some(Cmd::Status(reply)) => { let _ = reply.send(node.status()); Ok(Vec::new()) }
                    Some(Cmd::Snapshot(reply)) => match node.take_snapshot() {
                        Ok(b) => { let _ = reply.send(Ok(b)); Ok(Vec::new()) }
                        Err(e) => { let _ = reply.send(Err(closed())); Err(e) }
                    },
                },
                env = transport.recv() => match env {
                    None => return,
                    Some(env) => node.step(env.from, env.msg),
                },
                _ = interval.tick() => node.tick(),
            };
            match out {
                Ok(out) => {
                    for (to, msg) in out {
                        let _ = transport.send(&to, msg).await;
                    }
                }
                Err(e) => {
                    eprintln!("raft node {:?} stopping on fatal error: {e}", node.id());
                    return;
                }
            }
        }
    });
    RaftHandle { tx, join }
}
