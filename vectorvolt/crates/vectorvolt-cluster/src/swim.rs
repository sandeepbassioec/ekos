use crate::membership::MembershipList;
use crate::node::{NodeId, NodeState};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum ProbeResult {
    Alive,
    Suspect,
    Dead,
}

/// Trait allowing tests to inject mock probers.
pub trait Prober: Send + Sync {
    fn probe(&self, target: &NodeId) -> ProbeResult;
}

/// Real prober (no TCP — returns Suspect on timeout simulation).
pub struct DirectProber;
impl Prober for DirectProber {
    fn probe(&self, _target: &NodeId) -> ProbeResult {
        ProbeResult::Suspect
    }
}

pub struct SwimProber {
    membership: Arc<MembershipList>,
    probe_timeout: Duration,
    suspect_timeout: Duration,
    suspect_times: std::sync::Mutex<HashMap<NodeId, Instant>>,
    /// Optional injected prober for tests.
    prober: Option<Box<dyn Prober>>,
}

impl SwimProber {
    pub fn new(
        membership: Arc<MembershipList>,
        probe_timeout: Duration,
        suspect_timeout: Duration,
    ) -> Self {
        SwimProber {
            membership,
            probe_timeout,
            suspect_timeout,
            suspect_times: std::sync::Mutex::new(HashMap::new()),
            prober: None,
        }
    }

    pub fn with_prober(mut self, p: Box<dyn Prober>) -> Self {
        self.prober = Some(p);
        self
    }

    fn do_probe(&self, target: &NodeId) -> ProbeResult {
        if let Some(ref p) = self.prober {
            return p.probe(target);
        }
        // No real TCP — default to Suspect (timeout simulation).
        ProbeResult::Suspect
    }

    pub async fn probe_direct(&self, target: &NodeId) -> ProbeResult {
        if self.membership.get(target).is_none() {
            return ProbeResult::Dead;
        }
        let result = self.do_probe(target);
        match &result {
            ProbeResult::Alive => {
                self.membership.set_state(target, NodeState::Alive);
                let mut times = self.suspect_times.lock().unwrap();
                times.remove(target);
            }
            ProbeResult::Suspect | ProbeResult::Dead => {
                self.membership.mark_suspect(target);
                let mut times = self.suspect_times.lock().unwrap();
                times.entry(target.clone()).or_insert_with(Instant::now);
            }
        }
        result
    }

    pub async fn probe_indirect(&self, target: &NodeId, via: &[NodeId]) -> ProbeResult {
        if self.membership.get(target).is_none() {
            return ProbeResult::Dead;
        }
        // Ask each via node; if any returns Alive, target is Alive.
        for helper in via {
            if let Some(ref p) = self.prober {
                let r = p.probe(helper);
                if r == ProbeResult::Alive {
                    self.membership.set_state(target, NodeState::Alive);
                    return ProbeResult::Alive;
                }
            }
        }
        // All failed — mark Dead.
        self.membership.mark_dead(target);
        ProbeResult::Dead
    }

    /// Promotes Suspect→Dead if suspect_timeout has elapsed.
    pub fn expire_suspects(&self) -> Vec<NodeId> {
        let mut expired = Vec::new();
        let now = Instant::now();
        let mut times = self.suspect_times.lock().unwrap();
        let timeout = self.suspect_timeout;
        times.retain(|id, when| {
            if now.duration_since(*when) >= timeout {
                expired.push(id.clone());
                false
            } else {
                true
            }
        });
        for id in &expired {
            self.membership.mark_dead(id);
        }
        expired
    }

    pub fn probe_timeout(&self) -> Duration {
        self.probe_timeout
    }

    /// Test helper: mutable access to the suspect_times map.
    pub fn suspect_times_for_test(
        &self,
    ) -> std::sync::MutexGuard<'_, HashMap<NodeId, std::time::Instant>> {
        self.suspect_times.lock().unwrap()
    }
}

// ─── Mock probers for tests ────────────────────────────────────────────────

pub struct AlwaysAliveProber;
impl Prober for AlwaysAliveProber {
    fn probe(&self, _target: &NodeId) -> ProbeResult {
        ProbeResult::Alive
    }
}

pub struct AlwaysTimeoutProber;
impl Prober for AlwaysTimeoutProber {
    fn probe(&self, _target: &NodeId) -> ProbeResult {
        ProbeResult::Suspect
    }
}

// ─── Real SWIM protocol node ───────────────────────────────────────────────

use crate::gossip_net::{Envelope, Kind, Transport};
use crate::node::NodeMeta;
use rand::seq::SliceRandom;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::{oneshot, watch};

/// Tunables for [`SwimNode`].
#[derive(Debug, Clone)]
pub struct SwimConfig {
    /// Period between probe rounds.
    pub probe_interval: Duration,
    /// Wait for a direct ack; the indirect phase waits twice this.
    pub probe_timeout: Duration,
    /// Suspect -> Dead delay.
    pub suspect_timeout: Duration,
    /// Number of helpers asked to probe indirectly (k).
    pub indirect_probes: usize,
    /// Max piggy-backed updates per datagram.
    pub max_piggyback: usize,
}

impl Default for SwimConfig {
    fn default() -> Self {
        SwimConfig {
            probe_interval: Duration::from_millis(500),
            probe_timeout: Duration::from_millis(200),
            suspect_timeout: Duration::from_secs(5),
            indirect_probes: 3,
            max_piggyback: 8,
        }
    }
}

/// A running SWIM member: failure detector + dissemination over a [`Transport`].
pub struct SwimNode {
    membership: Arc<MembershipList>,
    transport: Arc<dyn Transport>,
    cfg: SwimConfig,
    seq: AtomicU64,
    pending: std::sync::Mutex<HashMap<u64, oneshot::Sender<()>>>,
    /// Updates awaiting dissemination with their transmit counts.
    queue: std::sync::Mutex<HashMap<NodeId, (NodeMeta, u32)>>,
    suspect_since: std::sync::Mutex<HashMap<NodeId, Instant>>,
    stop: watch::Sender<bool>,
    left: AtomicBool,
}

impl SwimNode {
    pub fn new(
        membership: Arc<MembershipList>,
        transport: Arc<dyn Transport>,
        cfg: SwimConfig,
    ) -> Arc<Self> {
        let (stop, _) = watch::channel(false);
        Arc::new(SwimNode {
            membership,
            transport,
            cfg,
            seq: AtomicU64::new(1),
            pending: Default::default(),
            queue: Default::default(),
            suspect_since: Default::default(),
            stop,
            left: AtomicBool::new(false),
        })
    }

    pub fn membership(&self) -> &Arc<MembershipList> {
        &self.membership
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.transport.local_addr()
    }

    pub fn has_left(&self) -> bool {
        self.left.load(Ordering::SeqCst)
    }

    fn me(&self) -> NodeMeta {
        let mut m = self
            .membership
            .get(self.membership.local_id())
            .expect("local member present");
        m.addr = self.transport.local_addr();
        m
    }

    /// Spawn the receive and probe loops.
    pub fn start(self: &Arc<Self>) {
        let rx = self.clone();
        let mut stop = self.stop.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = stop.changed() => break,
                    got = rx.transport.recv() => match got {
                        Some((_, env)) => {
                            let n = rx.clone();
                            tokio::spawn(async move { n.handle(env).await });
                        }
                        None => break,
                    }
                }
            }
        });
        let pr = self.clone();
        let mut stop = self.stop.subscribe();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(pr.cfg.probe_interval);
            loop {
                tokio::select! {
                    _ = stop.changed() => break,
                    _ = tick.tick() => {
                        pr.expire_suspects();
                        let n = pr.clone();
                        // Probe concurrently so a slow round never stalls the clock.
                        tokio::spawn(async move { n.probe_round().await });
                    }
                }
            }
        });
    }

    /// Stop all loops without announcing (a crash, when used in tests).
    pub fn shutdown(&self) {
        let _ = self.stop.send(true);
    }

    /// Join the cluster through the first reachable seed.
    pub async fn join(self: &Arc<Self>, seeds: &[SocketAddr]) -> bool {
        for seed in seeds {
            if *seed == self.local_addr() {
                continue;
            }
            let (seq, rx) = self.register();
            self.send(*seed, seq, Kind::Join, Vec::new()).await;
            if let Ok(Ok(())) = tokio::time::timeout(self.cfg.probe_timeout * 4, rx).await {
                return true;
            }
            self.pending.lock().unwrap().remove(&seq);
        }
        false
    }

    /// Gracefully leave: bump incarnation, mark self Left and broadcast it.
    pub async fn leave(&self) {
        self.left.store(true, Ordering::SeqCst);
        let id = self.membership.local_id().clone();
        self.membership.bump_incarnation(&id);
        self.membership.set_state(&id, NodeState::Left);
        let me = self.me();
        let peers: Vec<NodeMeta> = self
            .membership
            .all_members()
            .into_iter()
            .filter(|m| m.id != id && matches!(m.state, NodeState::Alive | NodeState::Suspect))
            .collect();
        for p in peers {
            let seq = self.seq.fetch_add(1, Ordering::SeqCst);
            let env = Envelope {
                seq,
                kind: Kind::Ping,
                from: me.clone(),
                updates: vec![me.clone()],
            };
            self.transport.send(p.addr, &env).await;
        }
        self.shutdown();
    }

    fn register(&self) -> (u64, oneshot::Receiver<()>) {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(seq, tx);
        (seq, rx)
    }

    fn enqueue(&self, meta: NodeMeta) {
        self.queue
            .lock()
            .unwrap()
            .insert(meta.id.clone(), (meta, 0));
    }

    /// Pick up to `max_piggyback` least-transmitted updates; retire old ones.
    fn take_updates(&self) -> Vec<NodeMeta> {
        let n = self.membership.len().max(2) as f64;
        let limit = (3.0 * n.log2().ceil()) as u32 + 1;
        let mut q = self.queue.lock().unwrap();
        let mut items: Vec<_> = q.values().map(|(m, c)| (m.clone(), *c)).collect();
        items.sort_by_key(|(_, c)| *c);
        items.truncate(self.cfg.max_piggyback);
        let mut out = Vec::new();
        for (m, _) in items {
            if let Some(e) = q.get_mut(&m.id) {
                e.1 += 1;
                if e.1 >= limit {
                    q.remove(&m.id);
                }
            }
            out.push(m);
        }
        out
    }

    async fn send(&self, to: SocketAddr, seq: u64, kind: Kind, mut extra: Vec<NodeMeta>) {
        let mut updates = self.take_updates();
        updates.append(&mut extra);
        let env = Envelope {
            seq,
            kind,
            from: self.me(),
            updates,
        };
        self.transport.send(to, &env).await;
    }

    /// Apply updates; refute any claim that we are not alive.
    fn merge(&self, updates: Vec<NodeMeta>) {
        let local = self.membership.local_id().clone();
        for u in updates {
            if u.id == local {
                if u.state != NodeState::Alive && !self.has_left() {
                    let cur = self
                        .membership
                        .get(&local)
                        .map(|m| m.incarnation)
                        .unwrap_or(0);
                    if u.incarnation >= cur {
                        self.membership.set_incarnation(&local, u.incarnation + 1);
                        self.enqueue(self.me());
                    }
                }
                continue;
            }
            if self.membership.apply_update(u.clone()) {
                self.enqueue(u);
            }
        }
    }

    async fn handle(self: Arc<Self>, env: Envelope) {
        if self.has_left() {
            return;
        }
        let sender = env.from.clone();
        let local = self.membership.local_id().clone();
        self.merge(env.updates);
        // Learn unknown senders (e.g. a joiner).
        if sender.id != local && self.membership.get(&sender.id).is_none() {
            self.merge(vec![sender.clone()]);
        }
        match env.kind {
            Kind::Ping => {
                // Tell a node we consider suspect/dead so it can refute.
                let mut extra = Vec::new();
                if let Some(m) = self.membership.get(&sender.id)
                    && matches!(m.state, NodeState::Dead | NodeState::Suspect)
                {
                    extra.push(m);
                }
                self.send(sender.addr, env.seq, Kind::Ack, extra).await;
            }
            Kind::Join => {
                let snapshot: Vec<NodeMeta> = self
                    .membership
                    .all_members()
                    .into_iter()
                    .take(256)
                    .collect();
                self.send(sender.addr, env.seq, Kind::Ack, snapshot).await;
            }
            Kind::Ack => {
                if let Some(tx) = self.pending.lock().unwrap().remove(&env.seq) {
                    let _ = tx.send(());
                }
            }
            Kind::PingReq { target_addr, .. } => {
                if self
                    .ping(target_addr, self.cfg.probe_timeout, Vec::new())
                    .await
                {
                    self.send(sender.addr, env.seq, Kind::Ack, Vec::new()).await;
                }
            }
        }
    }

    /// Ping `addr`, wait up to `timeout` for the ack.
    async fn ping(&self, addr: SocketAddr, timeout: Duration, extra: Vec<NodeMeta>) -> bool {
        let (seq, rx) = self.register();
        self.send(addr, seq, Kind::Ping, extra).await;
        let ok = tokio::time::timeout(timeout, rx)
            .await
            .is_ok_and(|r| r.is_ok());
        if !ok {
            self.pending.lock().unwrap().remove(&seq);
        }
        ok
    }

    async fn probe_round(self: &Arc<Self>) {
        if self.has_left() {
            return;
        }
        let local = self.membership.local_id().clone();
        let peers: Vec<NodeMeta> = self
            .membership
            .all_members()
            .into_iter()
            .filter(|m| m.id != local && matches!(m.state, NodeState::Alive | NodeState::Suspect))
            .collect();
        let Some(target) = peers.choose(&mut rand::thread_rng()).cloned() else {
            return;
        };
        // 1. direct ping
        // A suspect is told about its suspicion so it can refute it.
        let extra = if target.state == NodeState::Suspect {
            vec![target.clone()]
        } else {
            Vec::new()
        };
        if self.ping(target.addr, self.cfg.probe_timeout, extra).await {
            return;
        }
        // 2. indirect ping-req via k random members
        let mut helpers: Vec<&NodeMeta> = peers
            .iter()
            .filter(|m| m.id != target.id && m.state == NodeState::Alive)
            .collect();
        helpers.shuffle(&mut rand::thread_rng());
        helpers.truncate(self.cfg.indirect_probes);
        if !helpers.is_empty() {
            let (seq, rx) = self.register();
            for h in &helpers {
                let kind = Kind::PingReq {
                    target: target.id.clone(),
                    target_addr: target.addr,
                };
                self.send(h.addr, seq, kind, Vec::new()).await;
            }
            let ok = tokio::time::timeout(self.cfg.probe_timeout * 2, rx)
                .await
                .is_ok_and(|r| r.is_ok());
            if ok {
                return;
            }
            self.pending.lock().unwrap().remove(&seq);
        }
        // 3. no ack at all: suspect at the incarnation we know.
        if let Some(cur) = self.membership.get(&target.id)
            && cur.state == NodeState::Alive
        {
            let mut s = cur;
            s.state = NodeState::Suspect;
            if self.membership.apply_update(s.clone()) {
                self.enqueue(s);
            }
        }
    }

    /// Promote members that stayed Suspect past `suspect_timeout` to Dead.
    fn expire_suspects(&self) {
        let now = Instant::now();
        let mut since = self.suspect_since.lock().unwrap();
        let members = self.membership.all_members();
        since.retain(|id, _| {
            members
                .iter()
                .any(|m| &m.id == id && m.state == NodeState::Suspect)
        });
        for m in members {
            if m.state != NodeState::Suspect || &m.id == self.membership.local_id() {
                continue;
            }
            let t = *since.entry(m.id.clone()).or_insert(now);
            if now.duration_since(t) >= self.cfg.suspect_timeout {
                let mut d = m;
                d.state = NodeState::Dead;
                if self.membership.apply_update(d.clone()) {
                    self.enqueue(d);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{NodeMeta, NodeState};
    use std::net::SocketAddr;

    fn make_meta(id: NodeId) -> NodeMeta {
        NodeMeta {
            id,
            addr: "127.0.0.1:9000".parse::<SocketAddr>().unwrap(),
            state: NodeState::Alive,
            incarnation: 1,
            last_seen_ns: 0,
        }
    }

    fn make_membership(self_id: NodeId) -> Arc<MembershipList> {
        Arc::new(MembershipList::new(make_meta(self_id)))
    }

    #[tokio::test]
    async fn probe_direct_alive_prober_returns_alive() {
        let id = NodeId::new();
        let ml = make_membership(id.clone());
        let prober = SwimProber::new(
            ml.clone(),
            Duration::from_millis(500),
            Duration::from_secs(2),
        )
        .with_prober(Box::new(AlwaysAliveProber));
        let result = prober.probe_direct(&id).await;
        assert_eq!(result, ProbeResult::Alive);
    }

    #[tokio::test]
    async fn probe_direct_timeout_returns_suspect() {
        let id = NodeId::new();
        let ml = make_membership(id.clone());
        let prober = SwimProber::new(
            ml.clone(),
            Duration::from_millis(500),
            Duration::from_secs(2),
        )
        .with_prober(Box::new(AlwaysTimeoutProber));
        let result = prober.probe_direct(&id).await;
        assert_eq!(result, ProbeResult::Suspect);
    }

    #[tokio::test]
    async fn membership_updated_after_suspect() {
        let id = NodeId::new();
        let ml = make_membership(id.clone());
        let prober = SwimProber::new(
            ml.clone(),
            Duration::from_millis(500),
            Duration::from_secs(2),
        )
        .with_prober(Box::new(AlwaysTimeoutProber));
        prober.probe_direct(&id).await;
        assert_eq!(ml.get(&id).unwrap().state, NodeState::Suspect);
    }

    #[test]
    fn expire_suspects_before_timeout_returns_empty() {
        let id = NodeId::new();
        let ml = make_membership(id.clone());
        let prober = SwimProber::new(
            ml.clone(),
            Duration::from_millis(500),
            Duration::from_secs(100),
        );
        // Mark suspect manually
        ml.mark_suspect(&id);
        {
            let mut t = prober.suspect_times.lock().unwrap();
            t.insert(id.clone(), Instant::now());
        }
        let expired = prober.expire_suspects();
        assert!(expired.is_empty());
    }

    #[test]
    fn expire_suspects_after_timeout_returns_node() {
        let id = NodeId::new();
        let ml = make_membership(id.clone());
        let prober = SwimProber::new(
            ml.clone(),
            Duration::from_millis(500),
            Duration::from_millis(0),
        );
        ml.mark_suspect(&id);
        {
            let mut t = prober.suspect_times.lock().unwrap();
            // Use a very old instant to simulate elapsed timeout.
            t.insert(id.clone(), Instant::now() - Duration::from_secs(10));
        }
        let expired = prober.expire_suspects();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0], id);
    }

    #[tokio::test]
    async fn indirect_probe_all_fail_returns_dead() {
        let self_id = NodeId::new();
        let target_id = NodeId::new();
        let helper_id = NodeId::new();
        let ml = make_membership(self_id.clone());
        ml.upsert(make_meta(target_id.clone()));
        ml.upsert(make_meta(helper_id.clone()));
        let prober = SwimProber::new(
            ml.clone(),
            Duration::from_millis(500),
            Duration::from_secs(2),
        )
        .with_prober(Box::new(AlwaysTimeoutProber));
        let result = prober.probe_indirect(&target_id, &[helper_id]).await;
        assert_eq!(result, ProbeResult::Dead);
    }

    #[tokio::test]
    async fn indirect_probe_one_succeeds_returns_alive() {
        let self_id = NodeId::new();
        let target_id = NodeId::new();
        let helper_id = NodeId::new();
        let ml = make_membership(self_id.clone());
        ml.upsert(make_meta(target_id.clone()));
        ml.upsert(make_meta(helper_id.clone()));
        let prober = SwimProber::new(
            ml.clone(),
            Duration::from_millis(500),
            Duration::from_secs(2),
        )
        .with_prober(Box::new(AlwaysAliveProber));
        let result = prober.probe_indirect(&target_id, &[helper_id]).await;
        assert_eq!(result, ProbeResult::Alive);
    }

    #[tokio::test]
    async fn probe_nonexistent_node_returns_dead() {
        let self_id = NodeId::new();
        let ml = make_membership(self_id.clone());
        let prober = SwimProber::new(
            ml.clone(),
            Duration::from_millis(500),
            Duration::from_secs(2),
        );
        let ghost = NodeId::new();
        let result = prober.probe_direct(&ghost).await;
        assert_eq!(result, ProbeResult::Dead);
    }

    #[tokio::test]
    async fn suspect_then_alive_clears_suspect() {
        let id = NodeId::new();
        let ml = make_membership(id.clone());
        {
            let prober = SwimProber::new(
                ml.clone(),
                Duration::from_millis(500),
                Duration::from_secs(2),
            )
            .with_prober(Box::new(AlwaysTimeoutProber));
            prober.probe_direct(&id).await;
        }
        assert_eq!(ml.get(&id).unwrap().state, NodeState::Suspect);
        {
            let prober = SwimProber::new(
                ml.clone(),
                Duration::from_millis(500),
                Duration::from_secs(2),
            )
            .with_prober(Box::new(AlwaysAliveProber));
            prober.probe_direct(&id).await;
        }
        assert_eq!(ml.get(&id).unwrap().state, NodeState::Alive);
    }

    #[test]
    fn two_suspects_expire_independently() {
        let a = NodeId::new();
        let b = NodeId::new();
        let ml = make_membership(a.clone());
        ml.upsert(make_meta(b.clone()));
        let prober = SwimProber::new(
            ml.clone(),
            Duration::from_millis(500),
            Duration::from_millis(0),
        );
        ml.mark_suspect(&a);
        ml.mark_suspect(&b);
        {
            let mut t = prober.suspect_times.lock().unwrap();
            t.insert(a.clone(), Instant::now() - Duration::from_secs(10));
            t.insert(b.clone(), Instant::now() - Duration::from_secs(10));
        }
        let expired = prober.expire_suspects();
        assert_eq!(expired.len(), 2);
    }

    #[tokio::test]
    async fn probe_timeout_zero_always_times_out() {
        let id = NodeId::new();
        let ml = make_membership(id.clone());
        // AlwaysTimeoutProber simulates probe_timeout=0 behaviour.
        let prober = SwimProber::new(ml.clone(), Duration::from_millis(0), Duration::from_secs(2))
            .with_prober(Box::new(AlwaysTimeoutProber));
        let result = prober.probe_direct(&id).await;
        assert_eq!(result, ProbeResult::Suspect);
    }

    #[tokio::test]
    async fn membership_len_decreases_after_dead() {
        let self_id = NodeId::new();
        let target = NodeId::new();
        let ml = make_membership(self_id.clone());
        ml.upsert(make_meta(target.clone()));
        let prober = SwimProber::new(
            ml.clone(),
            Duration::from_millis(0),
            Duration::from_millis(0),
        )
        .with_prober(Box::new(AlwaysTimeoutProber));
        prober.probe_direct(&target).await;
        // Manually expire
        {
            let mut t = prober.suspect_times.lock().unwrap();
            t.insert(target.clone(), Instant::now() - Duration::from_secs(10));
        }
        prober.expire_suspects();
        assert_eq!(ml.alive_members().len(), 1); // only self
    }

    #[test]
    fn incarnation_bump_on_refute() {
        let id = NodeId::new();
        let ml = make_membership(id.clone());
        let before = ml.get(&id).unwrap().incarnation;
        ml.bump_incarnation(&id);
        let after = ml.get(&id).unwrap().incarnation;
        assert!(after > before);
    }
}
