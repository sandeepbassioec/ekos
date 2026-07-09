# Evolution Engine as a Service
## Taking EKOS's Self-Healing Outside VectorVolt

**Status:** Architecture Capture (commercial vision — implement after Phase 5 proves the concept internally)
**Captured:** 2026-07-09

---

## 1. The Core Observation

The self-healing pipeline built for VectorVolt and EKOS (ARP → Evidence Correlation →
Shadowbox → PromotionEngine) is not specific to a knowledge runtime. Every software system
in the world has the same underlying problem:

```
Software changes → Every change is risky → Humans manage the risk manually
```

PagerDuty, Dynatrace, Datadog, GitHub Copilot — each solves one slice:
- Observability tools answer: *"What happened?"*
- AI coding tools answer: *"Write this new code"*
- CI/CD tools answer: *"Is this safe to deploy?"*

**Nobody answers:** *"I observed this failure in production, here is the evidence, I diagnosed
the root cause, I synthesized a fix, I proved it correct against production traffic, and I
recommend approval."*

That full pipeline is what EKOS does internally. It can be offered as a service.

---

## 2. What "Evolution Engine as a Service" Is NOT

- **Not another AI coding assistant.** Copilot, Cursor, Devin write code from prompts.
  The Evolution Engine finds failures, diagnoses root causes, and generates fixes — without
  a human providing the prompt.

- **Not an observability platform.** Dynatrace and Datadog are the *eyes*. The Evolution
  Engine is the *reasoning layer* that sits above them.

- **Not a replacement for the developer.** The Evolution Engine cannot handle architecture
  changes, feature development, or requirements-level decisions. It handles *operational
  correctness* — the system doing what it was already designed to do, reliably.

---

## 3. The Customer's System (What We're Plugging Into)

The Evolution Engine needs three things from any customer system:

### 3.1 Evidence Access

```
What we need from the customer:
  Metrics          → existing Datadog / Prometheus / CloudWatch (we read, not replace)
  Error events     → existing Sentry / Splunk / application logs
  Deployments      → git history + CI/CD deployment events
  Business metrics → optional but makes correlation dramatically better
```

All existing observability infrastructure becomes input. We don't replace it —
we add a reasoning layer on top.

### 3.2 Source Access

The Evolution Engine synthesizes patches. To synthesize a correct patch, it needs the
source code. Three models depending on customer security posture:

```
Model A — On-Premises Appliance (recommended for enterprise):
  Customer deploys Evolution Engine inside their own infrastructure.
  Source never leaves their perimeter.
  No cloud dependency.

Model B — Source Escrow + Cloud Processing:
  Customer sends AES-256-GCM encrypted source (customer holds the key).
  Evolution Engine decrypts in an isolated environment, synthesizes patch, re-encrypts.
  Customer key never leaves their HSM.
  Legal precedent exists: code escrow has operated for decades.

Model C — Telemetry-Only (limited capability):
  Customer provides only metrics and error events, no source.
  Evolution Engine can diagnose and classify issues.
  Cannot synthesize patches (no source = no synthesis).
  Can recommend known fixes from the Evolution Knowledge Graph (see Section 6).
```

### 3.3 Sandbox Access

```
Evolution Engine needs to:
  1. Compile a candidate patch against the customer's codebase
  2. Run the customer's test suite
  3. Replay production traffic against the patched binary

Options:
  Option A: Deploy sandbox inside customer's cloud account (Evolution Engine manages it)
  Option B: Customer provides a staging environment endpoint
  Option C: Evolution Engine runs sandboxes in its own isolated multi-tenant environment
            (Model B/C only — source was shared)
```

---

## 4. The Service Pipeline

For any participating system:

```
1. Evidence Collection
   Customer's Datadog/Prometheus → Evidence Correlation Engine
   Evidence classified: Infrastructure / Runtime / Application / Business / Evolution

2. Correlation
   Temporal clustering + causal graph walk
   Output: "Root cause hypothesis: deployment 14 min ago, confidence 94%"

3. Routing
   Infrastructure issue? → Self-heal recommendation (scale, restart, migrate)
   Deployment regression? → Rollback recommendation (NOT patch synthesis)
   Code bug?             → Proceed to ARP

4. ARP — Payload Capture
   Capture exact gRPC/HTTP payload that triggered the error
   Capture preceding 50 transactions (sliding window)
   Capture store state at time of failure

5. Diagnosis
   LLM + source analysis → root cause identification
   Source read in isolated sandbox (never persisted outside sandbox)

6. Synthesis
   LLM generates candidate patch (Rust / Go / Java / Python — language plugins)
   Compiled in customer's sandbox or hosted sandbox

7. Validation (3-layer)
   Layer 1: Original payload replay → same error must NOT reproduce
   Layer 2: PayloadFuzzer variants → adjacent cases must also pass
   Layer 3: Regression corpus → no previously fixed bugs re-introduced
   + Customer's own test suite runs in full

8. Evidence Report
   Full diff (before/after)
   Confidence score breakdown
   Affected components
   Risk assessment
   Recommended deployment strategy (canary / blue-green / immediate)

9. Human Approval Gate
   Always present — cannot be bypassed by Evolution Engine default config
   Customer's team reviews, approves, rejects, or requests changes

10. Deployment (if approved)
    Customer's existing CI/CD pipeline executes the deployment
    Evolution Engine monitors post-deploy for regression

11. Learning
    Outcome (success / regression / rollback) stored in Evolution Knowledge Graph
    Future fixes for similar patterns benefit from this outcome
```

---

## 5. Digital Twin (for Behavioral Validation)

Replaying the WAL (production transactions) validates correctness: does the patch produce
the right output? But it does not validate **behavioral realism**: does the patch hold up
under the actual load patterns, memory pressure, and contention that occur in production?

A **Digital Twin** goes further than Shadowbox:

```
Shadowbox tests:    Does the code produce correct outputs?
Digital Twin tests: Does the code behave correctly under production load patterns?
```

The Digital Twin is a full replica of the customer's production environment state:
- Same memory profile (buffer pool occupancy, cache hit rates)
- Same concurrent connection count
- Same background job timing
- Same data distribution

When a patch is validated in the Digital Twin, it has been tested under the exact
conditions that caused the original failure — not just a clean replay.

**Phase placement for Digital Twin:**
- Shadowbox is Phase 4 (already designed).
- Digital Twin is Phase 6+ for VectorVolt's own use.
- Digital Twin-as-a-Service is a late-stage Evolution Engine capability.

---

## 6. Evolution Knowledge Graph

The most powerful long-term differentiator: the Evolution Engine learns from every
fix across every customer.

```
Data stored per fix outcome:
  - Error pattern (NOT customer's source or data — the pattern only)
  - Root cause classification
  - Fix strategy (which kind of patch solved it)
  - Whether the fix held (no regression in 30 days)
  - System topology that had this failure (language, framework, volume)
```

When a new customer system encounters a failure, the Evolution Engine asks:

*"Have any of the 500 other systems using this service encountered a similar pattern?
What fix strategy worked there?"*

This is fundamentally different from "AI writes code." It's **operational collective memory**:
patterns learned across thousands of production failures, used to inform diagnosis and
synthesis for every new failure.

Customer data is never shared. The *pattern abstraction* (failure signature) is learned and
shared with appropriate anonymization and tenant isolation.

---

## 7. Languages Supported

ARP internally targets Rust (VectorVolt's language). The Evolution Engine as a Service
must support the enterprise software landscape:

| Language | Phase | Notes |
|---|---|---|
| Rust | Phase 5 internal | ARP's native target |
| Java / Kotlin | External Phase 1 | Largest enterprise target |
| C# / .NET | External Phase 1 | Second-largest enterprise target |
| Go | External Phase 2 | Cloud-native services |
| Python | External Phase 2 | Data engineering, ML pipelines |
| TypeScript/Node | External Phase 3 | API layers |

Each language requires a separate **synthesis plugin** (LLM prompt chain tuned for that
language's error patterns, type system, and idioms) and a separate sandbox environment.

---

## 8. Commercial Model

```
Tier 1 — Evidence & Correlation (free / open source):
  Read existing observability, correlate events, classify root causes.
  No source access required. No patch synthesis.
  Designed to show value without commitment.

Tier 2 — Diagnosis & Recommendation (SaaS subscription):
  Model C: telemetry-only service.
  Hypothesis generation, risk assessment, deployment recommendations.
  No patch synthesis (no source). Recommendations are human-actionable descriptions.

Tier 3 — Full Evolution Engine (SaaS or on-prem):
  Model A or B: source access + sandbox.
  Complete pipeline: evidence → patch → validation → human approval → deploy.
  On-prem appliance available for regulated industries (banking, healthcare, defence).

Tier 4 — Evolution Platform (enterprise):
  Digital Twin.
  Evolution Knowledge Graph participation.
  SLA guarantees.
  Dedicated sandbox infrastructure.
  Custom language plugins.
```

---

## 9. Go-To-Market Sequence

```
Now (Phase 1–4):
  Build and prove Evolution Engine internally for VectorVolt/EKOS.
  The first customer is Garvion itself.

Phase 5 (after internal ARP is live):
  Package as on-prem appliance (Tier 3).
  Target: 2-5 design partners in VectorVolt's first enterprise customers.
  Revenue: license fee per node under observation.

After design partner validation:
  Open Tier 1 (evidence correlation) as open source.
  This drives adoption and provides telemetry for the Knowledge Graph.

After open-source traction:
  Launch Tier 2 SaaS (diagnosis without source).
  This is the volume tier — many customers, low friction.

After Tier 2 traction:
  Full Tier 3 commercial launch.
  The Knowledge Graph is now trained on real data.
  Differentiation is proven, not claimed.
```

---

## 10. What This Is NOT Competing With

| Tool | What It Does | Evolution Engine Difference |
|---|---|---|
| Datadog | Observability, alerting | We reason above Datadog's data |
| GitHub Copilot | Write code from prompts | We find bugs in running code, no prompt needed |
| Devin / Cursor | Autonomous developer | They build features; we maintain operational correctness |
| PagerDuty | Alert routing + on-call | We resolve before the page |
| Harness / ArgoCD | Deployment automation | We validate what's safe to deploy |

The competitive moat is not any single feature. It is the **full pipeline** from observability
input to production-proven patch — with human governance baked in, not bolted on.

---

*The first customer is always yourself.*
*Ship this for VectorVolt. Prove it works. Then let others use it.*
