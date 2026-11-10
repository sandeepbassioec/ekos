//! Auto-scale rebalancer (gap F7): from per-shard load metrics, propose
//! splits and moves that keep node load within an imbalance threshold.
//!
//! Off by default. Rate-limited: at most one migration in flight per node
//! (as source or target) and at most `max_actions_per_round` actions a round.

use crate::migration::{
    MapControl, MigrationConfig, MigrationReport, Nodes, ShardLoad, migrate_shard, split_shard,
};
use crate::node::NodeId;
use crate::shard::ShardTable;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceConfig {
    pub enabled: bool,
    /// A node is hot when its load exceeds `avg * (1 + imbalance_threshold)`.
    pub imbalance_threshold: f64,
    pub keys_weight: f64,
    pub ops_weight: f64,
    /// Never split a shard with fewer keys than this.
    pub min_split_keys: u64,
    pub max_actions_per_round: usize,
}

impl Default for RebalanceConfig {
    fn default() -> Self {
        RebalanceConfig {
            enabled: false,
            imbalance_threshold: 0.25,
            keys_weight: 1.0,
            ops_weight: 1.0,
            min_split_keys: 2,
            max_actions_per_round: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShardMetric {
    pub shard_id: u64,
    pub owner: NodeId,
    pub keys: u64,
    pub ops_per_sec: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Split { shard_id: u64 },
    Move { shard_id: u64, to: NodeId },
}

fn load_of(cfg: &RebalanceConfig, m: &ShardMetric) -> f64 {
    m.keys as f64 * cfg.keys_weight + m.ops_per_sec * cfg.ops_weight
}

/// Pure planner. `busy` nodes (migration in flight) are neither sources nor
/// targets.
pub fn plan(
    cfg: &RebalanceConfig,
    table: &ShardTable,
    metrics: &[ShardMetric],
    candidates: &[NodeId],
    busy: &HashSet<NodeId>,
) -> Vec<Action> {
    let eligible: Vec<NodeId> = candidates
        .iter()
        .filter(|n| !table.draining.contains(n) && !table.drained.contains(n))
        .cloned()
        .collect();
    if eligible.len() < 2 {
        return vec![];
    }
    let mut node_load: HashMap<NodeId, f64> = eligible.iter().map(|n| (n.clone(), 0.0)).collect();
    for m in metrics {
        if let Some(l) = node_load.get_mut(&m.owner) {
            *l += load_of(cfg, m);
        }
    }
    let total: f64 = node_load.values().sum();
    let avg = total / eligible.len() as f64;
    if avg <= 0.0 {
        return vec![];
    }
    let limit = avg * (1.0 + cfg.imbalance_threshold);

    let mut used: HashSet<NodeId> = busy.clone();
    let mut actions = Vec::new();
    let mut order = eligible.clone();
    order.sort_by(|a, b| {
        node_load[b]
            .partial_cmp(&node_load[a])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    for hot in order {
        if actions.len() >= cfg.max_actions_per_round {
            break;
        }
        let hot_load = node_load[&hot];
        if hot_load <= limit || used.contains(&hot) {
            continue;
        }
        let cold = eligible
            .iter()
            .filter(|n| **n != hot && !used.contains(*n))
            .min_by(|a, b| {
                node_load[*a]
                    .partial_cmp(&node_load[*b])
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.0.cmp(&b.0))
            });
        let Some(cold) = cold.cloned() else { continue };
        let gap = hot_load - node_load[&cold];
        let mine: Vec<&ShardMetric> = metrics
            .iter()
            .filter(|m| m.owner == hot && !table.migrating.contains_key(&m.shard_id))
            .collect();
        // Best improving move: strictly lowers the hot node below its old
        // load without making the target the new hot spot.
        let best_move = mine
            .iter()
            .map(|m| (*m, load_of(cfg, m)))
            .filter(|(_, l)| *l > 0.0 && *l < gap)
            .min_by(|a, b| {
                (a.1 - gap / 2.0)
                    .abs()
                    .partial_cmp(&(b.1 - gap / 2.0).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        if let Some((m, _)) = best_move {
            actions.push(Action::Move {
                shard_id: m.shard_id,
                to: cold.clone(),
            });
            used.insert(hot);
            used.insert(cold);
            continue;
        }
        // No single shard fits: split the heaviest splittable one.
        let split = mine
            .iter()
            .filter(|m| m.keys >= cfg.min_split_keys.max(2))
            .max_by(|a, b| {
                load_of(cfg, a)
                    .partial_cmp(&load_of(cfg, b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        if let Some(m) = split {
            actions.push(Action::Split {
                shard_id: m.shard_id,
            });
            used.insert(hot);
        }
    }
    actions
}

/// Converts node-reported cumulative op counters into per-shard ops/s.
#[derive(Default)]
pub struct MetricsCollector {
    last: Mutex<Option<(Instant, HashMap<u64, u64>)>>,
}

impl MetricsCollector {
    pub async fn collect(&self, table: &ShardTable, nodes: &Nodes) -> Vec<ShardMetric> {
        let mut out = Vec::new();
        let now = Instant::now();
        let mut totals: HashMap<u64, u64> = HashMap::new();
        let mut loads: Vec<(NodeId, ShardLoad)> = Vec::new();
        for id in nodes.ids() {
            let Ok(n) = nodes.get(&id) else { continue };
            let Ok(ls) = n.loads().await else { continue };
            for l in ls {
                // Only the map's owner is authoritative for a shard.
                if table.shard(l.shard_id).is_some_and(|s| s.owner == id) {
                    totals.insert(l.shard_id, l.ops_total);
                    loads.push((id.clone(), l));
                }
            }
        }
        let prev = self.last.lock().unwrap().replace((now, totals));
        for (owner, l) in loads {
            let ops = prev
                .as_ref()
                .and_then(|(t, m)| {
                    let dt = now.duration_since(*t).as_secs_f64();
                    m.get(&l.shard_id)
                        .filter(|_| dt > 0.0)
                        .map(|p| l.ops_total.saturating_sub(*p) as f64 / dt)
                })
                .unwrap_or(0.0);
            out.push(ShardMetric {
                shard_id: l.shard_id,
                owner,
                keys: l.keys,
                ops_per_sec: ops,
            });
        }
        out
    }
}

#[derive(Debug, Default)]
pub struct RoundReport {
    pub disabled: bool,
    pub splits: usize,
    pub moves: Vec<MigrationReport>,
    pub failures: Vec<String>,
    /// Actions skipped because a node was already migrating.
    pub rate_limited: usize,
}

/// Executes planned actions with per-node rate limiting.
pub struct Rebalancer {
    pub cfg: RebalanceConfig,
    /// node -> migrations in flight (as source or target).
    active: Mutex<HashMap<NodeId, usize>>,
    peak_per_node: Mutex<usize>,
}

impl Rebalancer {
    pub fn new(cfg: RebalanceConfig) -> Arc<Self> {
        Arc::new(Rebalancer {
            cfg,
            active: Mutex::new(HashMap::new()),
            peak_per_node: Mutex::new(0),
        })
    }

    /// Highest number of simultaneous migrations ever seen on one node.
    pub fn peak_migrations_per_node(&self) -> usize {
        *self.peak_per_node.lock().unwrap()
    }

    fn busy_set(&self) -> HashSet<NodeId> {
        self.active
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, c)| **c > 0)
            .map(|(n, _)| n.clone())
            .collect()
    }

    /// Atomically claim both nodes; `false` when either is already busy.
    fn try_claim(&self, a: &NodeId, b: &NodeId) -> bool {
        let mut g = self.active.lock().unwrap();
        if g.get(a).copied().unwrap_or(0) > 0 || g.get(b).copied().unwrap_or(0) > 0 {
            return false;
        }
        let mut peak = self.peak_per_node.lock().unwrap();
        for n in [a, b] {
            let c = g.entry(n.clone()).or_insert(0);
            *c += 1;
            *peak = (*peak).max(*c);
        }
        true
    }

    fn release(&self, a: &NodeId, b: &NodeId) {
        let mut g = self.active.lock().unwrap();
        for n in [a, b] {
            if let Some(c) = g.get_mut(n) {
                *c = c.saturating_sub(1);
            }
        }
    }

    /// Plan and execute one round.
    pub async fn run_round(
        self: &Arc<Self>,
        control: Arc<dyn MapControl>,
        nodes: Nodes,
        metrics: Vec<ShardMetric>,
        mcfg: MigrationConfig,
    ) -> RoundReport {
        let mut report = RoundReport::default();
        if !self.cfg.enabled {
            report.disabled = true;
            return report;
        }
        let table = control.table();
        let actions = plan(&self.cfg, &table, &metrics, &nodes.ids(), &self.busy_set());
        let mut tasks = tokio::task::JoinSet::new();
        for a in actions {
            match a {
                Action::Split { shard_id } => {
                    match split_shard(control.as_ref(), &nodes, shard_id, None).await {
                        Ok(_) => report.splits += 1,
                        Err(e) => report.failures.push(format!("split {shard_id}: {e}")),
                    }
                }
                Action::Move { shard_id, to } => {
                    let Some(from) = table.shard(shard_id).map(|s| s.owner.clone()) else {
                        continue;
                    };
                    if !self.try_claim(&from, &to) {
                        report.rate_limited += 1;
                        continue;
                    }
                    let (me, control, nodes, mcfg) =
                        (self.clone(), control.clone(), nodes.clone(), mcfg.clone());
                    tasks.spawn(async move {
                        let r = migrate_shard(control.as_ref(), &nodes, shard_id, &to, &mcfg).await;
                        me.release(&from, &to);
                        r.map_err(|e| format!("move {shard_id}: {e}"))
                    });
                }
            }
        }
        while let Some(j) = tasks.join_next().await {
            match j {
                Ok(Ok(rep)) => report.moves.push(rep),
                Ok(Err(e)) => report.failures.push(e),
                Err(e) => report.failures.push(e.to_string()),
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shard::ShardCmd;
    use uuid::Uuid;

    fn n(i: u128) -> NodeId {
        NodeId(Uuid::from_u128(i))
    }

    fn table_with(owner: &NodeId) -> ShardTable {
        let mut t = ShardTable::new();
        t.apply(&ShardCmd::Init {
            owner: owner.clone(),
        })
        .unwrap();
        t
    }

    #[test]
    fn plan_splits_a_single_hot_shard_then_moves() {
        let (a, b) = (n(1), n(2));
        let t = table_with(&a);
        let cfg = RebalanceConfig {
            enabled: true,
            ..Default::default()
        };
        let m = vec![ShardMetric {
            shard_id: 1,
            owner: a.clone(),
            keys: 100,
            ops_per_sec: 0.0,
        }];
        let p = plan(&cfg, &t, &m, &[a.clone(), b.clone()], &HashSet::new());
        // gap == load, so no strictly-improving whole-shard move: split.
        assert_eq!(p, vec![Action::Split { shard_id: 1 }]);
    }

    #[test]
    fn plan_respects_busy_and_balanced() {
        let (a, b) = (n(1), n(2));
        let mut t = table_with(&a);
        t.apply(&ShardCmd::Split {
            shard_id: 1,
            at: 10,
        })
        .unwrap();
        let cfg = RebalanceConfig::default();
        let m = vec![
            ShardMetric {
                shard_id: 1,
                owner: a.clone(),
                keys: 50,
                ops_per_sec: 0.0,
            },
            ShardMetric {
                shard_id: 2,
                owner: a.clone(),
                keys: 50,
                ops_per_sec: 0.0,
            },
        ];
        let nodes = [a.clone(), b.clone()];
        let p = plan(&cfg, &t, &m, &nodes, &HashSet::new());
        assert_eq!(p.len(), 1);
        assert!(matches!(p[0], Action::Move { .. }));
        let busy: HashSet<NodeId> = [b.clone()].into();
        assert!(plan(&cfg, &t, &m, &nodes, &busy).is_empty());
        let balanced = vec![
            ShardMetric {
                shard_id: 1,
                owner: a,
                keys: 50,
                ops_per_sec: 0.0,
            },
            ShardMetric {
                shard_id: 2,
                owner: b,
                keys: 50,
                ops_per_sec: 0.0,
            },
        ];
        assert!(plan(&cfg, &t, &balanced, &nodes, &HashSet::new()).is_empty());
    }
}
