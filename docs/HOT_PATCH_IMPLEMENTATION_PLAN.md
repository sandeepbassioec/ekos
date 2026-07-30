# Hot-Patching Implementation Plan
## EKOS, ESIP, VectorVolt (2026-Q3)

**Status**: Post-research planning  
**Priority**: Medium (enables continuous deployment later)  
**Owner**: Architecture team  
**Timeline**: 16 weeks (Phases 1-4)

---

## Quick Reference: Which Strategy for Each Project

### ESIP (React SPA)
- **Primary**: Feature flags (forward-only rollout) + Webpack HMR (dev only)
- **Secondary**: Blue-green deployment (if major UI overhaul)
- **Constraints**: Stateless; no session pinning needed
- **Adoption Timeline**: Phase 1 (weeks 1-4) + Phase 2 (weeks 5-8)

### EKOS (Enterprise Knowledge OS, React + Bedrock AI)
- **Primary**: Feature flags (control feature activation across tenants)
- **Secondary**: Widget hot-reloading (per-widget updates without app reload)
- **Tertiary**: Canary deployment (gradual rollout to customer segments)
- **Constraints**: Multi-tenant; session state per org; Bedrock API calls must degrade gracefully
- **Adoption Timeline**: Phase 1 (weeks 1-4) + Phase 2 (weeks 5-8) + Phase 4 (weeks 17-20 if DB changes needed)

### VectorVolt (Rust OLTP Engine)
- **Primary**: Per-worker code updates (Livepatch model) + Feature flags for optimizer paths
- **Secondary**: WAL format versioning (backward-compatible reads)
- **Tertiary**: Schema migrations with Vitess (if adopting for scale-out)
- **Constraints**: Lock-free data structures; workers must never see mixed code versions mid-transaction
- **Adoption Timeline**: Phase 1 (foundation) + Phase 3 (weeks 9-16) + Phase 4 (weeks 17-20)

---

## Phase 1: Foundation (Weeks 1-4)
*Applies to: All projects*

### 1.1 Feature Flag System

**Goal**: Decouple deployment from feature activation; enable instant rollback.

**Acceptance Criteria**:
- [ ] Feature flag service (in-process or remote) with <1ms latency
- [ ] Operators can toggle flags without redeployment
- [ ] Audit trail: who enabled/disabled flag and when
- [ ] Gradual rollout: `rollout_percentage` (0-100%)
- [ ] Deterministic: same user gets same result on reload

**Technology Stack**:
- **Option A (Simple)**: JSON config file + polling (suitable for <100K users)
  ```json
  {
    "features": {
      "ekos-new-dashboard": {
        "enabled": false,
        "rollout_percentage": 0,
        "description": "New AI-powered dashboard"
      }
    }
  }
  ```
- **Option B (Scalable)**: LaunchDarkly, Unleash, or custom Redis-backed service
  - Supports per-segment targeting (org, user, region)
  - Real-time updates (WebSocket push)
  - Built-in analytics

**Effort**: 1-2 weeks (Option A: 3 days; Option B: 10 days)

**Dependencies**: Monitoring setup (Phase 1.3).

### 1.2 Canary Traffic Routing

**Goal**: Route small percentage of traffic to new code; measure success.

**Acceptance Criteria**:
- [ ] Load balancer / ingress can split traffic (e.g., 1% vs 99%)
- [ ] Request can be tagged with version (header, log field)
- [ ] Metrics aggregated per version
- [ ] Manual switch to revert traffic split (< 30 seconds)

**Implementation**:
- **ESIP/EKOS**: Nginx/Traefik config:
  ```nginx
  location / {
    if ($request_time % 100 < 1) {
      proxy_pass http://backend_v2;
    }
    proxy_pass http://backend_v1;
  }
  ```
- **VectorVolt**: Per-worker flag check:
  ```rust
  if worker_id % 100 < canary_percentage {
    use_new_storage_engine = true;
  }
  ```

**Effort**: 1 week

**Dependencies**: Metrics collection (Phase 1.3).

### 1.3 Metrics Collection & Monitoring

**Goal**: Detect failures within 30 seconds; support auto-rollback decisions.

**Acceptance Criteria**:
- [ ] Error rate collected per version
- [ ] P50, P95, P99 latency per version
- [ ] Custom business metrics (e.g., conversion rate, indexing throughput)
- [ ] Dashboards showing before/after comparisons
- [ ] Alerts trigger on > 2σ deviation from baseline

**Metrics to Instrument**:

| Metric | Collection | Alert Threshold |
|--------|-----------|------------------|
| Request error rate | Per version | +2σ above baseline for 30s |
| Request latency (P95, P99) | Per version | +50% above baseline for 60s |
| Application exceptions | All | any uncaught exception |
| Business KPI (EKOS) | Per feature flag | -5% conversion |
| Throughput (queries/sec) | Per worker (VectorVolt) | -10% |

**Technology Stack**:
- Prometheus (metrics collection)
- Grafana (dashboards)
- AlertManager (auto-rollback trigger)
- Jaeger (distributed tracing, optional)

**Effort**: 2 weeks

**Dependencies**: None.

### 1.4 Rollback Procedures & Testing

**Goal**: Document and practice rollback for each project.

**Acceptance Criteria**:
- [ ] Runbook: "How to rollback feature X" (one-page per feature)
- [ ] Mock incident: simulate failure, execute rollback, measure time
- [ ] Chaos testing: randomly enable/disable feature flags, verify no crashes
- [ ] Rollback verification: after rollback, system returns to baseline (metrics, functionality)

**Runbook Template**:
```markdown
# Rollback: Feature X

## Detection
- Error rate > 10% for > 30 seconds
- OR P99 latency > 2 seconds for > 1 minute
- OR business metric drop > 5%

## Action
1. Page on-call engineer (if not already paged by AlertManager)
2. Disable feature flag: `feature_x.enabled = false`
3. Clear cache (if applicable): `redis FLUSHDB patterns:feature_x:*`
4. Verify metrics return to baseline within 2 minutes
5. Create incident log with timeline

## Post-Mortem
- [ ] Root cause identified
- [ ] Fix validated in canary before next deployment
- [ ] Metrics thresholds adjusted if false-positive
```

**Effort**: 1 week

**Dependencies**: Metrics, feature flags.

---

## Phase 2: ESIP & EKOS Widget/Feature Hot-Patching (Weeks 5-8)
*Applies to: ESIP, EKOS*

### 2.1 Widget Hot-Reloading Framework

**Goal**: Update EKOS widget UI without refreshing page (development experience only; not production).

**Acceptance Criteria**:
- [ ] Each EKOS widget is a React component with `module.hot.accept()` handler
- [ ] Widget state (form input, scroll position) persists during HMR
- [ ] New widget definition loads without page reload
- [ ] Errors in new widget don't crash the app (error boundary)

**Architecture** (see `HOT_PATCH_ARCHITECTURE.md` Part 6):
```typescript
// Widget registry (never reloaded)
const registry = new Map<string, WidgetDefinition>();

// On HMR:
if (module.hot) {
  module.hot.accept('./widget-v2.ts', () => {
    const newDef = require('./widget-v2.ts').default;
    registry.set('dashboard-widget', newDef);
    // Trigger re-render of affected instances
  });
}
```

**Implementation**:
1. Wrap each EKOS widget in `<WidgetHost>` component
2. Add `module.hot.accept()` to widget entry points
3. WidgetHost maintains component instance; on HMR, calls `setState({ dirty: true })` → re-renders
4. React Hooks state auto-persists (no cleanup needed)

**Effort**: 1 week

**Dependencies**: Webpack HMR already available in ESIP/EKOS.

**Note**: This is **dev-only**; production does not use HMR.

### 2.2 Feature Flag UI in EKOS

**Goal**: Allow EKOS operators to view and toggle feature flags for their organization.

**Acceptance Criteria**:
- [ ] Feature flag panel in EKOS admin UI
- [ ] Display: flag name, enabled (Y/N), rollout percentage
- [ ] Operators can toggle `enabled` without deploy
- [ ] Operators can adjust `rollout_percentage` (5%, 10%, 25%, 50%, 100%)
- [ ] Audit log: timestamp, operator, old value → new value

**UI Sketch**:
```
┌─ Feature Flags ─────────────────┐
│                                 │
│ Flag Name          Enabled   %  │
│ ─────────────────────────────────│
│ new-dashboard      [OFF]    0%   │
│ improved-search    [ON]     25%  │
│ ai-copilot         [ON]    100%  │
│                                 │
│ [Enable] [50%] [GA] [Disable]  │
└─────────────────────────────────┘
```

**Backend API**:
```
PATCH /api/v1/features/{name}
{
  "enabled": true,
  "rollout_percentage": 50
}
```

**Effort**: 2 weeks

**Dependencies**: Feature flag system (Phase 1.1).

### 2.3 A/B Testing Framework (Optional; ESIP Only)

**Goal**: Enable experimentation (not hot-patching per se, but related).

**Acceptance Criteria**:
- [ ] Route requests to A or B based on stable hash (user_id)
- [ ] Metrics collected separately for A and B cohorts
- [ ] Statistical significance calculation (p-value, confidence interval)
- [ ] Declare winner after N days or when significance reached

**Technology**: 
- Statsig, Optimizely, or custom framework
- Feature flag integration: flag name `ab_test_dashboard` with 50% rollout = A/B test

**Effort**: 2-3 weeks (if building custom; 1 day if using third-party)

**Dependencies**: Feature flags, metrics.

**Timeline**: Post Phase 2; include in Phase 3 if time permits.

---

## Phase 3: VectorVolt Worker Optimization Rollout (Weeks 9-16)
*Applies to: VectorVolt*

### 3.1 Per-Worker Version Tagging

**Goal**: Enable workers to transition to new code version at safe points (epoch boundaries).

**Acceptance Criteria**:
- [ ] Worker has `current_version` tag; GLOBAL_VERSION is source of truth
- [ ] At epoch boundary, worker checks: if GLOBAL_VERSION > current_version, transition
- [ ] Transition is atomic: old code path completes, new code path starts
- [ ] No in-flight transactions span version boundary

**Implementation**:
```rust
pub struct Worker {
  id: usize,
  epoch: u64,
  current_version: u32,
  next_version: u32,
  storage_engine: Arc<StorageEngine>,
}

impl Worker {
  pub fn process_batch(&mut self) -> Result<()> {
    // 1. Check for new version at epoch boundary (safe point)
    if let Ok(new_v) = GLOBAL_VERSION.load(Acquire) {
      if new_v > self.current_version {
        self.transition_to(new_v)?;
      }
    }
    
    // 2. Process requests with current version
    for txn in self.read_batch() {
      self.storage_engine.execute(txn)?;
    }
    
    // 3. Increment epoch
    self.epoch += 1;
  }
  
  fn transition_to(&mut self, version: u32) -> Result<()> {
    // Drain in-flight ops; ensure all use current_version
    self.storage_engine = load_storage_engine(version)?;
    self.current_version = version;
    Ok(())
  }
}
```

**Effort**: 1-2 weeks

**Dependencies**: Storage engine abstraction.

### 3.2 Shadow Traffic Collection

**Goal**: Route 1% of requests to new code; compare results; build confidence.

**Acceptance Criteria**:
- [ ] New code path executes in parallel (read-only; no side effects)
- [ ] Results compared: (old_result == new_result) counted
- [ ] Mismatch logged: input, old_result, new_result for investigation
- [ ] Metrics: match rate (target 99.99%), worst-case latency delta

**Implementation**:
```rust
pub async fn execute_with_shadow(txn: &Transaction) -> Result<Output> {
  let old_result = storage_engine_v1.execute(txn).await?;
  
  if CANARY_PERCENTAGE >= random(0..100) {
    let new_result = storage_engine_v2.execute(txn).await;
    match new_result {
      Ok(new) if old_result == new => {
        METRICS.increment("shadow.match");
      }
      Ok(new) => {
        METRICS.increment("shadow.mismatch");
        log::warn!("Results differ for txn {:?}: old={:?}, new={:?}",
                   txn.id, old_result, new);
      }
      Err(e) => {
        METRICS.increment("shadow.error");
        log::warn!("Shadow error: {:?}", e);
      }
    }
  }
  
  Ok(old_result)
}
```

**Metrics to Track**:
- Shadow match rate (target: 99.99%)
- Shadow latency delta (target: < 10%)
- Shadow error rate (target: 0%)
- Worst-case mismatches (investigate each)

**Effort**: 1 week

**Dependencies**: Version tagging, metrics collection.

### 3.3 Worker Migration Orchestration

**Goal**: Coordinate worker transitions; monitor; auto-rollback if needed.

**Acceptance Criteria**:
- [ ] Operator triggers transition: `set GLOBAL_VERSION=2`
- [ ] Workers transition over next 1 minute (batch-by-batch)
- [ ] Metrics monitored: error rate, latency, custom KPIs
- [ ] Auto-rollback triggered if metrics exceed thresholds
- [ ] Metrics dashboards show: workers transitioned %, current version per worker

**Orchestration Flow**:
```
[Operator]: set GLOBAL_VERSION=2
         ↓
[Global State]: GLOBAL_VERSION=2 (source of truth)
         ↓
[Workers]: At epoch boundary, each worker:
  if GLOBAL_VERSION > current_version:
    transition_to(GLOBAL_VERSION)
         ↓
[Metrics]: Error rate, P99 latency, custom KPIs
         ↓
[Auto-Rollback]: If error rate > threshold:
  set GLOBAL_VERSION=1
         ↓
[Workers]: Transition back to v1 at next epoch

Timeline: T+0 operator action
          T+1min all workers transitioned
          T+2min metrics stable or rollback triggered
```

**Effort**: 2 weeks

**Dependencies**: Version tagging, metrics, feature flags.

### 3.4 Metrics Dashboards & Alerts

**Goal**: Real-time visibility during worker transitions; auto-rollback triggers.

**Dashboards**:

1. **Worker Transition Dashboard**:
   - Gauge: Workers at v1 vs v2 (stacked bar)
   - Timeline: Transition progress (X-axis: time, Y-axis: version)
   - Current state: GLOBAL_VERSION, last update timestamp

2. **Health Dashboard**:
   - Error rate (v1 baseline vs v2 current)
   - P95, P99 latency (v1 baseline vs v2 current)
   - Throughput (txns/sec, v1 vs v2)
   - Custom KPI (e.g., index scan efficiency)

3. **Rollback Trigger Dashboard**:
   - Red zone: when metric exceeds threshold
   - Yellow zone: when metric approaches threshold
   - Action log: "Rollback triggered at T+5m, reason: error rate 12% > threshold 10%"

**Alerts**:
- Error rate > baseline + 2σ for > 30s → Page on-call + auto-rollback
- P99 latency > baseline * 1.5 for > 60s → Alert (manual review)
- Custom KPI drop > 5% → Alert (manual review)

**Effort**: 2 weeks (includes Prometheus/Grafana setup, alerting rules).

**Dependencies**: Metrics collection, orchestration.

---

## Phase 4: Database Migrations with Zero-Downtime (Weeks 17-20)
*Applies to: VectorVolt (if needed), EKOS/ESIP (if using persistent storage)*

### 4.1 Vitess Integration (Optional; Only if Scale-Out)

**Prerequisites**:
- Using MySQL as primary
- Scaling to > 1 billion rows
- Schema changes expected (new features)

**Goal**: Enable online schema migrations with automatic rollback.

**Acceptance Criteria**:
- [ ] Vitess cluster running (controller + vtctld)
- [ ] Database migrations via `ALTER TABLE ... ALGORITHM=VITESS`
- [ ] Rollback via `REVERT VITESS_MIGRATION`
- [ ] Runbook: "How to roll back a schema migration"

**Setup** (high-level):
1. Deploy Vitess controller
2. Migrate MySQL tables into Vitess keyspace
3. Update application: connect to Vitess VTProxy instead of MySQL directly
4. Schema changes: use Vitess DDL strategy

**Example Migration**:
```sql
-- Before: ALTER TABLE users ADD COLUMN shard_id INT;
-- This is long-running on large tables

-- With Vitess:
ALTER TABLE users ADD COLUMN shard_id INT ALGORITHM=VITESS;

-- If issues arise:
REVERT VITESS_MIGRATION;
```

**Effort**: 4-6 weeks (Vitess learning curve + deployment + testing)

**Dependencies**: MySQL database, DevOps bandwidth.

**Alternative** (if Vitess too complex):
- Use gh-ost + manual archive table management
- Less automated, but simpler to operate

### 4.2 Schema Versioning Strategy

**Goal**: Support old + new code simultaneously reading/writing different schema versions.

**Acceptance Criteria**:
- [ ] Entities have version field
- [ ] New code reads all versions; writes latest version
- [ ] Old code reads only its version (errors if schema incompatible)
- [ ] Backfill job migrates old→new schema
- [ ] Rollback: disable backfill, revert schema changes

**Implementation**:
```rust
pub struct IndexMetadata {
  id: u64,
  version: u8, // 1, 2, 3, ...
  name: String,
  // Version 2 additions:
  shard_id: Option<u32>,
  // Version 3 additions:
  compression: Option<CompressionType>,
}

// Old code (v1): reads, ignores shard_id and compression
// New code (v2): reads all, backfill process sets shard_id
// Future code (v3): all fields required

pub fn deserialize(bytes: &[u8]) -> Result<IndexMetadata> {
  let version = bytes[0];
  match version {
    1 => { /* parse v1 */ }
    2 => { /* parse v2 */ }
    3 => { /* parse v3 */ }
    _ => Err("unknown version"),
  }
}
```

**Effort**: 1-2 weeks (design + implementation)

**Dependencies**: None.

### 4.3 Backfill Job Framework

**Goal**: Migrate old-schema data to new schema without downtime.

**Acceptance Criteria**:
- [ ] Backfill job processes in batches (e.g., 1000 rows/sec)
- [ ] No table locks; reads from live table, writes updated rows
- [ ] Can be paused/resumed
- [ ] Progress tracked; ETA estimated
- [ ] Safe to rollback: re-run backfill, idempotent

**Implementation**:
```rust
pub async fn backfill_shard_ids(batch_size: usize) -> Result<()> {
  let mut offset = 0;
  loop {
    let indices = load_indices(offset, batch_size)
      .where_clause("shard_id IS NULL")
      .execute()
      .await?;
    
    if indices.is_empty() { break; }
    
    for mut index in indices {
      index.shard_id = Some(compute_shard(&index));
      store(&index).await?;
    }
    
    offset += batch_size;
    tokio::time::sleep(Duration::from_millis(100)).await; // Rate limit
    
    // Check for rollback signal
    if BACKFILL_ENABLED.load(Acquire) == false {
      return Ok(());
    }
  }
  
  Ok(())
}
```

**Effort**: 1 week

**Dependencies**: Schema versioning, database connection pooling.

### 4.4 Rollback Testing Procedures

**Goal**: Ensure rollback is tested before attempting in production.

**Acceptance Criteria**:
- [ ] For each schema migration, test rollback in staging
- [ ] Rollback test: apply migration, rollback, verify old code still works
- [ ] Document: what queries work before/after/rollback
- [ ] Runbook: exact steps to rollback in production (copy-paste ready)

**Rollback Test Checklist**:
```markdown
# Schema Migration Rollback Test: Add `shard_id` to `indices`

## Setup
1. Clone production data to staging
2. Verify data consistency

## Test
1. Apply migration: `ALTER TABLE indices ADD COLUMN shard_id INT;`
2. Start backfill: `run_backfill_shard_ids()`
3. Verify new code works (query `shard_id` field, gets values)
4. Rollback: `REVERT VITESS_MIGRATION;` (or `DROP COLUMN shard_id;`)
5. Verify old code works (old queries run without error)
6. Re-apply migration; let backfill run to completion
7. Verify data integrity (shard_id values correct for all rows)

## Metrics
- Migration time: X minutes
- Backfill rate: Y rows/sec
- Rollback time: Z seconds
- Data loss: 0 rows

## Runbook
- If production migration fails, execute rollback_procedures/shard_id_rollback.sh
```

**Effort**: 1-2 weeks (depends on migration complexity)

**Dependencies**: Staging environment, schema versioning.

---

## Cross-Project Dependencies & Sequencing

```
Phase 1: Foundation (Weeks 1-4)
  ├─ Feature flags (1.1) — all projects depend on this
  ├─ Canary routing (1.2)
  ├─ Metrics & monitoring (1.3)
  └─ Rollback procedures (1.4)
         ↓
Phase 2: ESIP/EKOS (Weeks 5-8)
  ├─ Widget HMR (2.1) — depends on Phase 1 metrics
  ├─ Feature flag UI (2.2) — depends on Phase 1.1
  └─ A/B testing (2.3) — optional, depends on Phase 1
         ↓
Phase 3: VectorVolt (Weeks 9-16)
  ├─ Worker versioning (3.1)
  ├─ Shadow traffic (3.2) — depends on Phase 1 metrics
  ├─ Worker migration (3.3) — depends on Phase 1.1 (feature flags)
  └─ Dashboards & alerts (3.4) — depends on Phase 1.3 metrics
         ↓
Phase 4: Database (Weeks 17-20) — optional, only if needed
  ├─ Vitess integration (4.1) — depends on Phase 1 metrics for safety
  ├─ Schema versioning (4.2)
  ├─ Backfill framework (4.3)
  └─ Rollback testing (4.4)
```

---

## Risk Mitigation

| Risk | Mitigation | Owner |
|------|-----------|-------|
| Metrics collection too slow (miss failures) | Instrument early; test alerting in canary | DevOps |
| Feature flag system becomes single point of failure | Design for redundancy; test failover | Backend |
| Workers stuck in old version (migration hangs) | Timeout + forced transition; runbook | SRE |
| Data corruption during schema migration | Test backfill with staging data; verify checksums | DBA |
| Rollback doesn't actually revert behavior | Test rollback in staging for each feature | QA |

---

## Success Criteria

- [ ] Phase 1 complete: All projects can deploy features with instant rollback
- [ ] Phase 2 complete: ESIP/EKOS support feature flags and widget updates
- [ ] Phase 3 complete: VectorVolt workers transition to new code without restart
- [ ] Phase 4 complete (optional): Database schema migrations run without downtime
- [ ] Runbooks documented and tested: incident response < 5 minutes
- [ ] Metrics dashboards live: visibility into all deployments
- [ ] Team trained: anyone on call can execute rollback in < 2 minutes

---

## Effort Estimate Summary

| Phase | Component | Effort (weeks) | Total |
|-------|-----------|----------------|-------|
| 1 | Feature flags | 1-2 | 6 |
| 1 | Canary routing | 1 | |
| 1 | Metrics & monitoring | 2 | |
| 1 | Rollback procedures | 1 | |
| 2 | Widget HMR | 1 | 5 |
| 2 | Feature flag UI | 2 | |
| 2 | A/B testing | 2-3 | |
| 3 | Worker versioning | 1-2 | 8 |
| 3 | Shadow traffic | 1 | |
| 3 | Worker migration | 2 | |
| 3 | Dashboards & alerts | 2 | |
| 4 | Vitess integration | 4-6 | 9-11 |
| 4 | Schema versioning | 1-2 | |
| 4 | Backfill framework | 1 | |
| 4 | Rollback testing | 1-2 | |
| **Total** | | | **28-32 weeks** |

**If prioritizing (MVP)**:
- Phase 1: 6 weeks (foundation)
- Phase 2a: 2 weeks (feature flag UI only; skip A/B testing)
- Phase 3: 8 weeks (VectorVolt)
- **Total MVP: 16 weeks**

---

## Next Steps (This Week)

1. [ ] Review `HOT_PATCH_ARCHITECTURE.md` with architecture team
2. [ ] Confirm technology stack choices (feature flag service, monitoring tools)
3. [ ] Assign Phase 1 work to teams (split feature flags, metrics, rollback procedures)
4. [ ] Schedule kickoff meeting (week 1, day 1 of Phase 1)
5. [ ] Create Jira epics and user stories for each phase

---

**Document Version**: 1.0  
**Created**: July 2026  
**Owner**: Architecture  
**Status**: Planning (awaiting approval)
