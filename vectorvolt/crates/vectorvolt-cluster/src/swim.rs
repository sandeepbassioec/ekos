use crate::membership::MembershipList;
use crate::node::{NodeId, NodeState};
use std::collections::HashMap;
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
