# Source Code & KoStore Backup Strategy
## VectorVolt + EKOS

**Status:** Architecture Capture (implement at Phase 4 alongside Workflow Runtime)  
**Captured:** 2026-07-07  

---

## 1. The Key Insight — WAL Already Solves This

VectorVolt's WAL writes every `ko_put` before it hits the KoStore.  
Source files are KOs. Therefore: **source commits are already in the WAL.**

We do not need a separate backup system. We need to **ship the WAL** to encrypted off-site destinations continuously — and take periodic full snapshots for fast restore.

```
ko_put("ekos/crates/ekos-kernel/main", "src/store.rs", new_version)
         ↓
    WAL segment written  ← already happens today
         ↓
    WAL Shipper (new)  →  encrypted off-site backup
         ↓
    KoStore updated
```

RPO (how much can we lose if server crashes): **seconds** — last WAL segment.

---

## 2. Three-Layer Backup

```
Layer 1 — WAL Streaming (continuous, RPO = seconds)
  Every WAL segment shipped immediately after it closes.
  Destination: hot standby (local second disk or nearby server).
  Encrypted: AES-256-GCM with backup key.

Layer 2 — Source Snapshot (after every build/commit, RPO = last commit)
  After every successful build workflow:
    → snapshot only source namespace KOs
    → fast (source files are small)
    → shipped to off-site cold storage
  This is the "backup after every build" the user needs.

Layer 3 — Full KoStore Snapshot (daily, RPO = 24h for non-source data)
  Full snapshot of all KOs (source + knowledge + config + audit).
  Shipped to off-site cold storage.
  Older than 30 days → moved to archive (cheaper storage tier).
```

---

## 3. Backup Encryption — Separate From Session Keys

Source code encryption (Section 6 of AUTONOMOUS_RUNTIME_VISION.md) uses the **HSM master key**.  
Backups use a **backup key** derived from the HSM master key but stored separately:

```
HSM master key
     ↓  HKDF derivation
Backup key  →  stored in N key-holder shards (same 3-of-5 threshold as master)

Backup package:
  header: { snapshot_id, timestamp, ko_count, source_commit_id }
  body:   AES-256-GCM( encrypted KO blobs, backup_key )
  mac:    HMAC-SHA256( header || body, backup_key )
```

Why separate key:
- Backup can be decrypted without unlocking the live HSM
- Key holders can restore without being on-site
- Compromise of backup key does not expose live session keys

---

## 4. Backup Destinations — 3-2-1 Rule

```
3 copies:
  Copy 1 — Primary KoStore (production node, NVMe)
  Copy 2 — Hot standby (WAL streaming, local secondary disk or LAN node)
  Copy 3 — Cold off-site (encrypted, object storage or air-gapped drive)

2 different media:
  NVMe SSD (primary + hot standby)
  Object storage / USB archive drive (cold)

1 off-site:
  Cold backup physically or logically separated from production node
```

Off-site options (admin configures in ekosd.toml):

```toml
[backup]
wal_streaming   = "local:/dev/sdb1/ekos-wal"          # hot standby
source_snapshot = "s3://garvion-backup/ekos-source"   # or sftp://, or local path
full_snapshot   = "s3://garvion-backup/ekos-full"
snapshot_retention_days = 30
archive_after_days      = 90

# Or fully air-gapped:
# full_snapshot = "local:/mnt/usb-backup/ekos"
```

No cloud dependency required. Works with any destination that can receive files.

---

## 5. What Gets Backed Up After Every Build

After `BuildWorkflow` completes successfully:

```
Source snapshot contains:
  All KOs in namespace: "<product>/crates/*/main"
  All KOs in namespace: "<product>/crates/*/feature-*"  (active branches)
  All KOs of kind:      "source_file", "build_artifact", "change_proposal"
  
Does NOT include (covered by daily full snapshot):
  Knowledge objects (application data)
  Audit KOs
  Config KOs
  Session tokens
```

Source snapshot is small — typically < 50MB for an entire codebase.  
Ships to off-site in seconds over any reasonable connection.

---

## 6. Server Crash — Recovery Procedure

### Scenario A: Node restarts, disk intact

```
ekosd starts → reads WAL → replays uncommitted segments → KoStore consistent
Time: seconds to minutes (WAL replay)
Source code: 100% intact
```

VectorVolt WAL already handles this. No intervention needed.

### Scenario B: Node crashes, disk corrupted

```
1. Spin up new node
2. Restore latest full snapshot from cold backup
3. Decrypt with backup key (requires 3-of-5 key holders)
4. Replay WAL segments from hot standby (Layer 1)
   → brings state forward to last shipped WAL segment
5. ekos-security MFA setup on new node
6. Resume

RTO: < 2 hours with documented runbook
RPO: seconds (last WAL segment that was shipped)
```

### Scenario C: Both node and hot standby lost (catastrophic)

```
1. Restore from cold off-site snapshot (Layer 2 or 3)
2. Source code: intact up to last build (Layer 2 snapshot)
3. Application data: intact up to last daily snapshot (Layer 3)
4. Gap between last snapshot and crash: re-derivable from git archive (emergency fallback)

RTO: < 4 hours
RPO: last build (source), last daily snapshot (data)
```

---

## 7. Backup Verification (Automated)

Backups that are never tested are not backups.  
A `BackupVerifyWorkflow` runs weekly in Shadowbox("dev"):

```
1. Fetch latest cold snapshot
2. Restore into Shadowbox("dev") (isolated — cannot affect production)
3. Verify:
   - All source KOs present and decrypt successfully
   - ko_history() returns expected version count
   - Build workflow can compile from restored source
4. Result stored as KO: kind="backup_verify_result"
5. Admin panel shows last verified date + result
```

If verification fails → alert admin immediately, treat as critical.

---

## 8. Crate Addition

```
ekos/crates/ekos-backup/
  src/
    wal_shipper.rs    ← streams WAL segments to destinations
    snapshot.rs       ← source namespace snapshot + full snapshot
    encrypt.rs        ← backup key derivation, AES-256-GCM packaging
    verify.rs         ← restore + verify workflow
  Cargo.toml
```

Integrated into `ekosd` — runs as background tasks, not a separate process.

---

## 9. Phase Placement

```
Phase 4:   ekos-backup crate
           WAL streaming (Layer 1) — most critical, implement first
           Source snapshot after build (Layer 2)

Phase 5:   Full daily snapshot (Layer 3)
           BackupVerifyWorkflow
           Admin panel backup status section

Phase 6:   Air-gapped cold backup support
           Key shard ceremony for backup key (separate from master key ceremony)
```

---

*Backup is not optional for a system that holds its own source code.*  
*WAL streaming is continuous — not a scheduled job. Source snapshot is per-commit — not daily.*
