# Hot-Patching & Zero-Downtime Deployment Architecture
## Practical Guide for EKOS Widgets, Features, and VectorVolt Optimizations

**Research Synthesis**: July 2026  
**Status**: Reference Architecture (not yet implemented)  
**Scope**: OS-level patching, application runtime injection, database migrations, distributed deployment strategies

---

## Executive Summary

This document synthesizes research across 5 domains (kernel, application, database, distributed systems, state management) into a practical hot-patching strategy for EKOS and VectorVolt. The key insight is that **consistency models from kernel live patching are directly transferable to application-level updates**, yielding three strategies:

| Strategy | Model | Best For | Tradeoff |
|----------|-------|----------|----------|
| **Quiesce** | kpatch (stop-the-world) | Rare critical patches, simple rollback | Brief service pause (~100ms) |
| **Per-Component** | Livepatch hybrid (syscall exit migration) | Widget/feature updates | Complex state coordination |
| **Forward-Only** | Feature flags + gradual activation | Normal feature rollouts | No backward compatibility |

---

## Part 1: OS-Level Kernel Hot-Patching (Transferable Patterns)

### Three Consistency Models

Research across Linux kernel implementations reveals three distinct approaches. **For EKOS, the Livepatch hybrid model is most applicable.**

#### 1. kpatch (Stop-the-World)
- **Method**: Examine all task stacks; only patch if target function not executing; pause briefly during swap
- **Consistency**: Total order—old code never coexists with new code
- **Latency**: ~100ms pause during atomic swap window
- **Pros**: Simplest implementation, easiest rollback, minimal memory overhead
- **Cons**: Service unavailable during patch window
- **Use Case**: EKOS critical bug fixes, VectorVolt storage format incompatibilities

#### 2. kGraft (Per-Task Migration)
- **Method**: Maintain both code versions; route each task to old/new version via trampolines; transition at kernel exit
- **Consistency**: Per-task consistency model—each task sees coherent view of old or new code, never mixed
- **Latency**: Zero downtime; tasks continue executing while patching
- **Pros**: True zero-downtime; never pauses
- **Cons**: Complex; both code versions resident in memory; harder to verify correctness
- **Use Case**: VectorVolt worker thread optimizations, where migration must be task-aware

#### 3. Mainline Livepatch (Hybrid—**RECOMMENDED**)
- **Method**: Combine kGraft's per-task migration with kpatch's stack-trace switching
  1. Check sleeping tasks' stacks → if safe, patch and mark ready
  2. At syscall exit → transition active tasks to new code
  3. Special case: idle "swapper" task via kernel hooks
- **Consistency**: Per-task; transitions guaranteed at safe points (syscall boundaries)
- **Latency**: ~1-10ms per task (no global pause); achieves near-zero downtime
- **Pros**: Practical balance of simplicity + availability; safe migration points well-defined
- **Cons**: Requires identifying safe transition points (syscall/context-switch boundaries)
- **Use Case**: **Primary model for EKOS feature updates and VectorVolt component reloads**

### Application to EKOS/VectorVolt

**For EKOS widgets:**
- Treat each widget instance as a "task" in the kernel model
- Widget updates apply safely only when widget is not actively rendering (= syscall boundary analog)
- Pending render events queue until updated code is ready
- Each widget maintains internal version tag; new events target matching version

**For VectorVolt worker threads:**
- Each worker is a "task"; new optimization code ships disabled
- At next epoch boundary (=syscall boundary), worker transitions to optimized path
- Fallback path remains active; rollback is immediate flag toggle
- No memory reallocation needed during transition

---

## Part 2: Application-Level Runtime Code Injection

### Language-Specific Capabilities and Limitations

#### Java HotSwap

**Capability**: Debuggers can redefined class bytecode in place using same class identity.

**Hard Constraints**:
- ❌ Cannot add methods
- ❌ Cannot add fields (except synthetic)
- ✅ Can change method bodies only

**Why**: JVM architecture makes field addition dangerous:
- Adding fields requires resizing every object instance in memory
- Requires moving objects (garbage collector coordination)
- Inlining optimizations in JIT make method addition costly

**For VectorVolt**: Not applicable (Rust, not JVM). But lesson: **avoid structural changes; use wrapper/adapter pattern instead**.

**Workaround (JRebel/DCEVM)**:
- DCEVM (Dynamic Code Evolution VM) is modified JVM supporting more changes
- JRebel (commercial) monitors `.class` files and reloads with fewer restrictions
- Still limited; not production-grade

#### Python importlib.reload()

**Capability**: Reload module at runtime with `importlib.reload(module)`

**Constraints**:
- Works only if single reference exists to module
- Cannot reload C extensions
- Module-level state persists unless explicitly cleared
- High risk of memory leaks in long-running services (cached references, circular imports)

**For EKOS**: Viable for dev-time widget hot-reloading; **not production-safe** without careful isolation.

**Production Pattern**:
```python
# Isolate hot-reloadable logic in separate module
# Main app imports once, calls exported interface
# Reloadable module can be reloaded without affecting main

# main.py (never reloaded)
from widget_interface import WidgetRegistry
registry = WidgetRegistry()
# ... uses registry

# widget_registry.py (reloadable)
import importlib
import widget_v1
current_impl = widget_v1

def reload_widgets():
    global current_impl
    importlib.reload(widget_v1)
    # Clear any cached state
    current_impl = widget_v1
```

#### Webpack/Next.js Hot Module Replacement (HMR)

**Capability**: Update JavaScript modules at runtime without page refresh.

**Mechanism**:
1. Webpack dev server watches source files
2. On change, bundles delta and sends via WebSocket
3. Module accepts update via `module.hot.accept(path, callback)`
4. Callback re-initializes state with new code

**Critical Limitation**: HMR requires **explicit opt-in per module**. Default behavior is page reload.

**State Management Gotcha** (from research):
```javascript
// ❌ WRONG: old handler persists
export function printMe() { console.log('v1'); }
document.body.addEventListener('click', printMe);

// ✅ CORRECT: re-bind after HMR
let printMe = () => console.log('v1');
let button = document.createElement('button');
button.addEventListener('click', printMe);
document.body.appendChild(button);

if (module.hot) {
  module.hot.accept('./print.js', () => {
    // Re-render entire component to rebind event handlers
    document.body.innerHTML = '';
    setupComponent();
  })
}
```

**For EKOS widgets**: 
- Each widget is a module with `module.hot.accept()` handler
- Handler receives new component definition
- Handler re-initializes widget DOM without page reload
- React component state can be preserved if using Hooks + dependency management

#### Go Plugin/dlopen Pattern

**Capability**: Load `.so` (shared object) at runtime via `plugin.Open()`

**Constraints**:
- No symbol versioning → breaking changes require app restart
- Limited to function pointers; no struct field changes
- Requires cgo; no pure-Go symbol reloading

**For VectorVolt**: Not applicable (Rust, not Go), but pattern is instructive: **design interfaces as function tables, not structs**.

---

## Part 3: Database Zero-Downtime Migrations

### Three Approaches

#### 1. gh-ost (GitHub Online Schema Migrations)

**Architecture**:
1. Create "ghost" table with new schema
2. Copy rows from original table → ghost table
3. Connect to MySQL binlog stream; replay changes to ghost table
4. When caught up, perform atomic **two-table rename** (milliseconds)

**Consistency Model**:
- Asynchronous copy; ghost table lags during migration
- Atomic cutover: `RENAME TABLE original TO archive, ghost TO original`
- Minimal metadata lock (~milliseconds)

**Pros**:
- No triggers (unlike pt-osc); uses binlog, lower overhead
- Widely tested at scale (GitHub origin)
- Can pause/resume migration

**Cons**:
- No native rollback (archive table must be manually swapped back)
- Cutover complexity; research found edge cases where monitoring queries can trigger data loss
- Must manage ghost table cleanup

**Recommended**: Data schema migrations with manual archive table retention for rollback.

#### 2. Vitess Online DDL

**Architecture** (built on gh-ost):
1. VReplication engine manages ghost table synchronization
2. Binlog stream replicated to ghost table
3. Buffered cutover: read/write queries paused briefly
4. Atomic cutover with connection rejection threshold

**Key Difference**: **Native revertibility via `REVERT VITESS_MIGRATION`**
- Stored in topology metadata
- Can restore original schema without manual table swaps
- Rollback fast (~seconds)

**Pros**:
- Vitess handles orchestration (no manual scripts)
- Native rollback integrated into deployment
- Better consistency guarantees than gh-ost

**Cons**:
- Requires Vitess infrastructure (not just MySQL)
- Learning curve for Vitess operators

**Recommended**: For VectorVolt schema changes; Vitess provides rollback guarantees.

#### 3. pt-online-schema-change (pt-osc)

**Architecture**:
1. Create ghost table
2. Add triggers to original table
3. Triggers capture changes; ghost table updated within same transaction
4. Atomic: `RENAME TABLE original TO archive, ghost TO original`

**Pros**:
- Single atomic rename (simpler than gh-ost cutover logic)
- Synchronous change capture (no lag)

**Cons**:
- Trigger overhead under load
- Metadata lock extends to triggers during cutover → becomes contention point
- No rollback capability
- Slower than gh-ost on large tables

**Not Recommended**: For VectorVolt; gh-ost or Vitess superior.

### Migration Checklist for VectorVolt

| Change Type | Tool | Rollback | Notes |
|-------------|------|----------|-------|
| Add column (nullable) | gh-ost | Manual archive swap | Safe; low risk |
| Add column (non-null, requires fill) | Vitess | REVERT VITESS_MIGRATION | Built-in rollback |
| Add index | Online DDL | Schema revert | Consider deferred index creation |
| Drop column | Vitess | REVERT before cutover | Irreversible post-cutover |
| Rename column | Feature flag + dual-write | Disable flag | Keep old column, dual-write, cutover in phases |
| Change type (int→bigint) | Vitess + shadow table | REVERT | Test type casting extensively |

---

## Part 4: Distributed Systems Deployment Strategies

### Three Patterns and When to Use Each

#### 1. Blue-Green Deployment

**Architecture**:
- Two identical production environments: **Blue** (live) and **Green** (staging)
- Deploy new version to **Green**; test; atomic switch via load balancer/DNS
- Old version (**Blue**) remains ready for instant rollback

**Consistency**: Atomic from user perspective; all requests route to one version at any moment.

**Pros**:
- Simple rollback: flip switch back to Blue
- Zero user requests see mixed versions
- Easy to test Green before switching

**Cons**:
- Requires 2x infrastructure cost
- Database schema changes incompatible with rollback (Blue expects old schema)
- For stateful services, session affinity required (users may land on wrong version mid-session)

**Use Case**: Stateless services, non-breaking DB changes, high-confidence releases.

**For ESIP**: Viable; React SPA doesn't maintain server state.

#### 2. Canary Deployment

**Architecture**:
- New version deployed alongside old
- Percentage-based traffic split (1% → 5% → 25% → 100%)
- Monitor metrics (error rate, latency, business KPIs); auto-rollback if threshold exceeded

**Consistency**: Mixed versions coexist; requests routed by percentage.

**Pros**:
- Limits blast radius (only 1% of users affected by bugs)
- Real-world validation before full rollout
- Gradual resource consumption ramp

**Cons**:
- Complex observability (must correlate failures to version)
- Longer rollout time (minutes to hours)
- Database schema changes problematic (mixing versions accessing different schemas)
- Sticky sessions complicate version mixing

**Use Case**: High-traffic services, new ML models, A/B testing.

**For EKOS**: Useful for gradual feature validation.

#### 3. Feature Flags (Forward-Only Rollout)

**Architecture**:
- New code deployed disabled by default (feature flag = false)
- Gradual activation via configuration: enable for 1% → 10% → 100% of users
- Users experience no deployment interruption; features activate asynchronously
- Rollback = disable flag (no code rollback needed)

**Consistency Model**: All users see same code; feature availability per user.

**Pros**:
- Decouples deployment from feature activation
- Instant rollback (flip flag, no redeployment)
- Forward-only: fixes deployed and activated forward, never rolled back
- Database schema changes can be handled incrementally (new column deployed, activation later)

**Cons**:
- Feature flag bloat if not cleaned up
- Dead code paths in production
- Requires flag evaluation logic in hot paths

**Use Case**: Continuous deployment, database-incompatible changes, risk-tolerant orgs.

**For EKOS/ESIP**: Recommended for normal feature rollouts.

### Combining Strategies

**Pattern: Blue-Green + Canary + Feature Flags**

```
Week 1: Blue-green deploy to Green (all new features flagged OFF)
        Tests pass; switch live traffic 1% → Green (Canary)
        
Week 2: If metrics green, increase → 25% of traffic canary
        Simultaneously, enable feature flag for 5% of users
        
Week 3: Fully switch to Green (100% traffic)
        Feature flags enable for 100% of users
        Archive Blue environment (no longer needed)
        
Rollback at any stage:
  - Canary fails → revert traffic split (instant)
  - Feature flag issue → disable flag (instant, no redeployment)
```

---

## Part 5: State Management and Rollback Architecture

### State Machine During Updates

```
[STEADY]
   ↓
[PATCH_READY]
  - New code loaded but not active
  - Old code handling all requests/transactions
  - Rollback: discard new code (no cleanup needed)
   ↓
[TRANSITION]
  - Per-component/task migration underway
  - Some requests use old code, some new
  - New transactions may be incompatible (see DB version check)
  - Rollback: reject new transactions, wait for old to drain
   ↓
[ACTIVE]
  - All requests/transactions use new code
  - Old code deallocated
  - Rollback: expensive; may require state transformation

OPTIONAL: [SHADOW]
  - New code receives copy of requests (no side effects)
  - Metrics collected; compared to old code
  - Builds confidence before TRANSITION
```

### In-Flight Request Migration

**Challenge**: A request arrives when old code is active, completes with new code (or vice versa).

**Solutions**:

#### 1. Idempotency Tokens (Safe Regardless)
```
Request = (idempotency_key, operation)
Store = { key → result }

On retry (new code):
  - Check if key in Store
  - If yes: return cached result (safe, even if semantics differ)
  - If no: execute new code operation
```

**Works for**: E-commerce (orders), payments, idempotent updates.  
**Doesn't work for**: Queries, read-side updates.

#### 2. Version Tagging (Explicit State Versioning)
```
Entity = { data, version }

Old code: increment version on write
New code: read version, apply new logic only if compatible

Example:
  User entity v1: {name, email}
  User entity v2: {name, email, profile_picture_url}
  
  Old code writing to new DB:
    user = {name, email, version: 1}
  
  New code reading:
    if user.version >= 2:
      load profile_picture_url
    else:
      use default
```

**Works for**: Schema migrations, optional fields, backward-compatible changes.

#### 3. Graceful Degradation (Feature Detection)
```
New code queries: "Does this component support new behavior?"
  - Yes → use new code path
  - No → use fallback path (compatible with old state)

Example:
  if (entity.hasOwnProperty('profile_picture_url')) {
    displayProfilePicture(entity.profile_picture_url);
  } else {
    displayPlaceholder();
  }
```

**Works for**: UI components, optional enhancements.

#### 4. Deferred Application (Batch Processing)
```
In-flight requests:
  - Old code: stage changes in queue with version tag
  - Transition: drain queue, apply in version order
  - New code: continue from drained queue
```

**Works for**: Batch jobs, analytics, non-critical updates.  
**Risk**: Explosion in queue size if transition delayed.

### Session State Pinning (Stateful Services)

**Problem**: User session binds to a worker; worker updates; session sees inconsistent state.

**Solution: Session Migration Lease**

```
Session = { user_id, worker_id, version, lease_expiry }

During update:
  1. New version deployed; old version leases expire at T+5min
  2. New workers handle only version=NEW
  3. Old sessions continue on old workers until lease expires
  4. Old workers reject new sessions (route to new workers)
  5. At T+5min, migrate remaining sessions:
     - Pause session
     - Copy state to new version format
     - Resume on new worker
```

**Lease duration**: balance between availability and cleanup time.  
**For EKOS**: 5-10 min reasonable (user session lifetime is hours).

---

## Part 6: Practical Architecture for EKOS

### EKOS Widget Hot-Reloading

**Goal**: Update EKOS widget UI component without refreshing entire app.

**Architecture**:

```typescript
// widget-registry.ts (never reloaded)
const registry = new Map<string, WidgetDefinition>();

export interface WidgetDefinition {
  id: string;
  version: number;
  render: (props: any) => React.ReactElement;
  cleanup: (state: any) => void;
}

export async function loadWidget(id: string, code: string) {
  const newDef = await compileAndLoad(code);
  
  if (newDef.version <= registry.get(id)?.version) {
    throw new Error('Version mismatch');
  }
  
  // Transition: quiesce existing instances
  const instances = findWidgetInstances(id);
  for (const instance of instances) {
    instance.setState({ loading: true });
  }
  
  // Update registry
  registry.set(id, newDef);
  
  // Resume instances with new code
  for (const instance of instances) {
    instance.setState({ loading: false });
    instance.forceUpdate();
  }
}
```

**State Preservation**:
```typescript
// Within React component
const [state, setState] = useState({ data: null });
const widgetDef = registry.get(widgetId);

useEffect(() => {
  // On HMR: if component re-renders, state persists
  // (React Hook state survives component re-render)
}, [widgetDef]);

// On widget definition change: re-render only this widget, not page
```

**Rollback**:
- Keep version history in registry
- Hot-load previous version on error
- Automatic rollback if new widget throws during render

### EKOS Feature Hot-Activation

**Goal**: Ship new feature disabled; activate gradually via flag.

**Pattern: Feature Flag + Gradual Rollout**

```typescript
// features.ts
const flags = {
  'new-dashboard': {
    enabled: false,
    rollout: 0, // 0-100 percentage
  }
};

export function isFeatureEnabled(name: string, userId: string): boolean {
  const flag = flags[name];
  if (!flag.enabled) return false;
  
  // Deterministic rollout: same user always gets same result
  const hash = hashUserId(userId) % 100;
  return hash < flag.rollout;
}

// In EKOS view:
{isFeatureEnabled('new-dashboard', currentUserId) && <NewDashboard />}
{!isFeatureEnabled('new-dashboard', currentUserId) && <OldDashboard />}
```

**Activation Timeline**:
```
Day 0: Deploy code (flag OFF, rollout 0%)
Day 1: Enable for QA (flag ON, rollout 5%)
Day 2: Expand (rollout 25%)
Day 3: GA (rollout 100%)
Week 1: Remove old code and flag
```

**Rollback**: Set `enabled: false` → instant revert.

---

## Part 7: Practical Architecture for VectorVolt

### VectorVolt Worker Optimization Rollout

**Goal**: Deploy new storage engine optimization; migrate worker threads safely; instant rollback if needed.

**Consistency Model: Per-Worker Livepatch Hybrid**

```rust
// worker.rs
pub struct Worker {
  id: usize,
  version: AtomicU32,
  epoch: AtomicU64,
  storage_engine: Arc<RwLock<Box<dyn StorageEngine>>>,
}

impl Worker {
  pub fn next_epoch(&mut self) {
    // Safe migration point (syscall boundary analog)
    // In reality: after processing batch, before next acquire
    
    let new_version = self.version.load(Acquire);
    if new_version > self.current_version {
      // Transition to new code
      self.storage_engine = load_storage_engine(new_version);
      self.current_version = new_version;
    }
    
    self.epoch.fetch_add(1, Release);
  }
}

// During update:
pub async fn hot_patch_storage_engine(new_impl: Box<dyn StorageEngine>) {
  // 1. New version shipped (disabled)
  let new_version = 2;
  
  // 2. Shadow traffic: route 1% of requests to new_impl, compare results
  // ... telemetry collection ...
  
  // 3. Transition: workers migrate at epoch boundary
  GLOBAL_VERSION.store(new_version, Release);
  
  // 4. Workers transition when they reach next_epoch()
  // Old code path remains resident until all workers transition
  
  // 5. Rollback available until last worker transitions
  if error_detected() {
    GLOBAL_VERSION.store(1, Release);
  }
}
```

**Rollback Window**: From patch ready until all workers transition (~10ms per worker).

### VectorVolt WAL Compatibility

**Goal**: Upgrade WAL format; old code must read old + new format; new code reads only new.

**Pattern: Dual-Version Reading**

```rust
pub enum WalEntry {
  V1 {
    timestamp: u64,
    txn_id: u64,
    data: Vec<u8>,
  },
  V2 {
    timestamp: u64,
    txn_id: u64,
    epoch: u32,
    data: Vec<u8>,
    compression: CompressionType,
  },
}

// Old code (v1): read and deserialize
// New code (v2): read, check version, deserialize
pub fn deserialize_wal_entry(bytes: &[u8]) -> Result<WalEntry> {
  let version = bytes[0];
  match version {
    1 => deserialize_v1(&bytes[1..]),
    2 => deserialize_v2(&bytes[1..]),
    _ => Err("unknown version"),
  }
}

// Writing: always new format
pub fn serialize_wal_entry(entry: &WalEntry) -> Vec<u8> {
  match entry {
    V2 { ... } => {
      let mut result = vec![2]; // version byte
      result.extend(serialize_v2_data(...));
      result
    }
  }
}
```

**Migration**:
1. Deploy v2 code (new code reads V1 + V2, writes V2)
2. All workers transition to v2 over time
3. WAL compaction step: rewrite old V1 entries as V2
4. Remove V1 deserialization code in next release

### VectorVolt Schema Migration (Hypothetical)

**Scenario**: Add `shard_id` field to index metadata; new query optimization requires it.

**Phase 1: Dual-Write (0 downtime)**
```rust
// Index struct
pub struct IndexMetadata {
  id: u64,
  name: String,
  shard_id: Option<u32>, // New field
}

// Writer (works with both old index files and new)
pub fn write_index_metadata(meta: &IndexMetadata) {
  let mut data = encode(meta);
  data.push(meta.shard_id.unwrap_or(0) as u8);
  store(data);
}

// Reader
pub fn read_index_metadata(data: &[u8]) -> IndexMetadata {
  let mut meta = decode::<IndexMetadata>(&data[..data.len()-1]);
  meta.shard_id = Some(data[data.len()-1] as u32);
  meta
}
```

**Phase 2: Data Migration (background job)**
```rust
async fn backfill_shard_ids() {
  // Scan all indices without shard_id
  for index in scan_indices_where(|m| m.shard_id.is_none()) {
    let shard = compute_shard(&index);
    index.shard_id = Some(shard);
    write_index_metadata(&index);
  }
}
```

**Phase 3: Enforce Non-Null (one more release cycle)**
```rust
pub struct IndexMetadata {
  id: u64,
  shard_id: u32, // Now required; all old records have it
}
```

**Rollback**: Always possible by disabling backfill job; shard_id defaults safely.

---

## Part 8: Rollback Guarantees and Verification

### Rollback Classification

| Type | Trigger | Latency | Data Risk | Use Case |
|------|---------|---------|-----------|----------|
| **Instant** | Human flag toggle | < 100ms | None | Feature flags |
| **Fast** | Worker transition pause | < 1s | Low | Worker code updates |
| **Safe Async** | Archive table swap | 1-10s | Low | DB migrations (gh-ost) |
| **Session Drain** | Session lease expiry | 1-10min | Medium | Stateful services |
| **Full Rebuild** | Restart from backup | 1-1hr | High | Catastrophic failure |

**For EKOS**: Instant rollback (feature flags); Safe Async (DB migrations if any).

**For VectorVolt**: Instant rollback (worker optimizations); Safe Async (schema migrations via Vitess).

### Verification Checklist Before Hot-Patch

- [ ] **Backward Compatibility**: New code can read old state format
- [ ] **State Migration**: Plan for in-flight requests during transition
- [ ] **Idempotency**: Can patches be applied twice without side effects?
- [ ] **Rollback Tested**: Manually test rollback path at least once
- [ ] **Metrics Instrumented**: Can detect failure within 30 seconds
- [ ] **Canary Window**: Route 1% traffic to new code for 5 minutes before GA
- [ ] **Feature Flag**: New feature controllable without redeployment
- [ ] **Documentation**: Post-mortem template ready if rollback triggered

### Monitoring During Transitions

**Red Line (Auto-Rollback)**:
- Error rate > baseline + 2σ for > 30 seconds
- P99 latency > baseline * 1.5 for > 1 minute
- Custom business metrics (e.g., conversion rate drop > 5%)

**Yellow Line (Alert, Manual Review)**:
- P50 latency increased > 10%
- Errorbudget consumed > 50% in 1 hour

---

## Part 9: Implementation Roadmap

### Phase 1: Foundation (Weeks 1-4)
- [ ] Feature flag system (on/off, rollout %)
- [ ] Metrics collection pipeline (pre/post update)
- [ ] Canary traffic routing (1% split)
- [ ] Rollback procedures documented + tested

### Phase 2: ESIP/EKOS (Weeks 5-8)
- [ ] Widget hot-reloading (module.hot.accept per widget)
- [ ] Feature flag UI in EKOS
- [ ] A/B testing framework (for dashboard experiment)
- [ ] Session state versioning

### Phase 3: VectorVolt Engine (Weeks 9-16)
- [ ] Per-worker version tagging
- [ ] Shadow traffic collection
- [ ] Worker migration at epoch boundaries
- [ ] Metrics dashboards (latency, error rates, custom KPIs)

### Phase 4: Database (Weeks 17-20)
- [ ] Vitess integration (if using MySQL)
- [ ] Schema versioning strategy
- [ ] Backfill job framework
- [ ] Rollback testing procedures

---

## Sources and Further Reading

- [Kernel Live Patching (Matthew Ruffell)](https://ruffell.nz/programming/writeups/2020/04/20/everything-you-wanted-to-know-about-kernel-livepatch-in-ubuntu.html)
- [Java HotSwap Guide (JRebel by Perforce)](https://www.jrebel.com/blog/java-hotswap-guide)
- [Webpack Hot Module Replacement](https://webpack.js.org/guides/hot-module-replacement/)
- [gh-ost vs Vitess Online DDL (PlanetScale)](https://planetscale.com/docs/vitess/schema-changes/online-schema-change-tools-comparison)
- [Feature Flags vs Blue-Green Deployments (Unleash)](https://www.getunleash.io/blog/blue-green-deployments-vs-feature-flags)
- [Blue-Green vs Canary Deployments (Octopus Deploy)](https://octopus.com/devops/software-deployments/blue-green-vs-canary-deployments/)

---

## FAQ

**Q: Can we hot-patch VectorVolt's Rust core without restarting workers?**  
A: Yes, using the per-worker Livepatch model. Deploy new binary; workers transition at epoch boundaries; instant rollback via feature flags.

**Q: What about in-flight transactions that span old + new code?**  
A: Use version tags on entities + idempotency tokens. New code ignores unknown fields; old code ignores new optional fields. Transactional consistency maintained via WAL.

**Q: How do we rollback a database schema migration?**  
A: Vitess provides `REVERT VITESS_MIGRATION`. For gh-ost, retain archive table and manually swap back (not automatic; requires runbook).

**Q: Isn't feature flagging code debt?**  
A: Yes. Set expiration dates (e.g., "remove by Sept 1"). Automate flag cleanup in your CI (lint for unused flags).

**Q: How long should we keep old code in binary?**  
A: ~10x the longest expected worker lifetime. For EKOS, 24 hours should suffice.

---

**Document Version**: 1.0  
**Last Updated**: July 2026  
**Status**: Reference Architecture (not implemented; review before adoption)
