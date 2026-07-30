# EKOS Native POD Architecture: Complete Technical Specification

**Version:** 1.0  
**Date:** 2026-07-12  
**Purpose:** Custom process-based isolation architecture for EKOS features, preventing blast radius

> **Vision:** Each EKOS feature runs in its own lightweight process (POD). If one crashes, parent restarts it. Other features unaffected. Communication via ultra-fast shared memory RPC, not network.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Overview](#architecture-overview)
3. [POD Execution Model](#pod-execution-model)
4. [POD Manager Specification](#pod-manager-specification)
5. [POD Lifecycle](#pod-lifecycle)
6. [Shared Data Plane](#shared-data-plane)
7. [RPC Communication Protocol](#rpc-communication-protocol)
8. [Resource Limits & Enforcement](#resource-limits--enforcement)
9. [Health Monitoring & Auto-Restart](#health-monitoring--auto-restart)
10. [Integration with EKOS](#integration-with-ekos)
11. [Integration with Self-Healing](#integration-with-self-healing)
12. [Integration with VectorVolt](#integration-with-vectorvolt)
13. [Security Model](#security-model)
14. [Performance Characteristics](#performance-characteristics)
15. [Implementation Guide](#implementation-guide)
16. [Example Workflows](#example-workflows)
17. [Monitoring & Observability](#monitoring--observability)
18. [Appendix](#appendix)

---

## Executive Summary

### The Problem

**EKOS as single runtime:**
- One feature crashes → entire platform down
- One feature memory leak → starves other features
- One feature slow → slows everyone down
- Self-healing requires coordinating across all features
- Deployment requires coordinating with all features

### The Solution: EKOS Native POD Architecture

**Key Insight:** Process-based isolation (not container), with ultra-fast shared-memory RPC (not network).

```
Benefits:
  ✅ One POD crashes → parent restarts it (<100ms)
  ✅ Memory limit enforced → runaway POD can't starve others
  ✅ Independent scaling → scale bottleneck feature only
  ✅ Fast communication → 1-10µs latency (not 3-5ms)
  ✅ Self-healing per-POD → no multi-feature coordination
  ✅ Deep integration → not bolted-on like K8s

Costs:
  ⚠️ Custom implementation (vs. using K8s)
  ⚠️ Operational complexity (but less than K8s)
  ⚠️ Process model (not container model)
```

### Architecture at a Glance

```
EKOS Core Runtime (PID: 1000)
├── POD Manager
│   ├── Spawner (fork/exec)
│   ├── Monitor (heartbeat, resource usage)
│   ├── Scheduler (allocate CPU/memory)
│   ├── Router (route RPC calls)
│   └── Reaper (clean up dead PODs)
│
├── Feature POD 1 (PID: 2001) - Widget-A
│   ├── Memory: 256MB limit (cgroup enforced)
│   ├── CPU: 1 core limit
│   ├── Status: ALIVE
│   └── Code: widget-a.so (dynamically loaded)
│
├── Feature POD 2 (PID: 2002) - Widget-B
│   ├── Memory: 256MB limit
│   ├── CPU: 1 core limit
│   ├── Status: ALIVE
│   └── Code: widget-b.so
│
├── Feature POD 3 (PID: 2003) - Self-Healing
│   ├── Memory: 512MB limit
│   ├── CPU: 2 cores limit
│   ├── Status: ALIVE
│   └── Code: self-healing.so
│
└── Shared Data Plane
    ├── RPC Router (protobuf + eventfd)
    ├── Cache Layer (read-only shared memory)
    ├── Config Store (versioned)
    ├── Event Bus (pub/sub)
    ├── Metrics Collector (per-POD)
    └── Heartbeat Manager (ping interval: 1s)
```

**Key Difference from Kubernetes:**
- Kubernetes: Containers with network communication (~3-5ms latency)
- EKOS Native POD: Processes with shared-memory RPC (~1-10µs latency)

---

## Architecture Overview

### POD Execution Model

**What is a POD?**

A POD is a **lightweight execution context** for a single EKOS feature:
- Separate OS process (own memory space)
- Own file descriptors, thread pool
- Own resource limits (enforced by cgroup)
- Own crash isolation (parent restarts it)
- Communication via shared-memory RPC

**POD Lifecycle:**

```
CREATION (fork + cgroup setup)
    ↓
STARTING (load .so, initialize)
    ↓
ALIVE (process requests, send heartbeats)
    ├→ (normal) → TERMINATING → DEAD (clean shutdown)
    ├→ (crash detected) → DEAD → RESTARTING (spawn new)
    ├→ (resource exceeded) → KILLED → RESTARTING
    └→ (too many restarts) → DISABLED (manual intervention)
```

### Memory Topology

**Parent Process (EKOS Core):**
```
┌─────────────────────────────────┐
│ EKOS Core Runtime (PID: 1000)   │
│                                 │
│ Code: ekos-core.so              │
│ Heap: 2GB (for core logic)      │
│ Stack: 8MB                      │
│ Shared memory: 10GB (mmap)      │
│                                 │
├─ POD Manager                    │
├─ RPC Router                     │
├─ Shared Data Plane              │
└─ (references to POD processes)  │
└─────────────────────────────────┘
         │
         │ fork()
         ▼
┌─────────────────────────────────┐
│ Feature POD 1 (PID: 2001)       │
│                                 │
│ Code: widget-a.so               │
│ Heap: 256MB (isolated)          │
│ Stack: 2MB                      │
│ Shared memory: 10GB (mmap same) │
│                                 │
└─────────────────────────────────┘
         │
         │ fork()
         ▼
┌─────────────────────────────────┐
│ Feature POD 2 (PID: 2002)       │
│                                 │
│ Code: widget-b.so               │
│ Heap: 256MB (isolated)          │
│ Stack: 2MB                      │
│ Shared memory: 10GB (mmap same) │
│                                 │
└─────────────────────────────────┘
```

**Key Insight:**
- Parent + all PODs: share same **shared memory region** (mmap)
- Each POD: isolated **heap** (private memory)
- Communication: via shared memory, not network

---

## POD Manager Specification

### Data Structures

```rust
/// POD state information
pub struct PODMetadata {
    pub id: String,                    // "widget-a", "feature-collab", etc.
    pub pid: u32,                      // OS process ID
    pub created_at: Instant,
    pub last_heartbeat: Instant,
    pub status: PODStatus,
    pub restarts: u32,                 // count in last hour
    pub memory_limit_mb: u32,          // cgroup limit
    pub cpu_limit_cores: u32,          // cgroup limit
    pub priority: PODPriority,         // 1-10, higher = more important
}

pub enum PODStatus {
    Starting,                          // cgroup setup, .so loading
    Alive,                            // running, accepting requests
    Unhealthy,                        // missed heartbeats, may crash soon
    Dead,                             // crashed or killed
    Restarting,                       // parent respawning
    Disabled,                         // too many restarts, manual fix needed
}

pub enum PODPriority {
    Critical(u8),      // Self-healing: 10
    High(u8),          // EKOS features: 7-9
    Medium(u8),        // Tier-2 features: 4-6
    Low(u8),           // Tier-1 widgets: 1-3
}

pub struct PODConfig {
    pub id: String,
    pub binary_path: String,           // path to .so file
    pub entry_point: String,           // symbol to call in .so
    pub memory_limit_mb: u32,
    pub cpu_limit_cores: u32,
    pub heartbeat_interval_ms: u32,   // default: 1000ms
    pub restart_limit_per_hour: u32,  // default: 10
    pub environment: HashMap<String, String>,
}

pub struct PODManager {
    pods: HashMap<String, PODMetadata>,
    shared_data: Arc<SharedDataPlane>,
    router: Arc<RpcRouter>,
    cgroup_manager: Arc<CgroupManager>,
    heartbeat_manager: Arc<HeartbeatManager>,
    stats_collector: Arc<StatsCollector>,
}
```

### Core Operations

#### 1. Spawn POD

```rust
pub fn spawn_pod(&mut self, config: PODConfig) -> Result<String> {
    // 1. Validate configuration
    if config.memory_limit_mb < 128 {
        return Err("Minimum memory is 128MB");
    }
    
    // 2. Fork child process
    let pid = unsafe {
        let pid = libc::fork();
        if pid == -1 {
            return Err("fork() failed");
        }
        pid
    };
    
    if pid == 0 {
        // Child process: execute feature code
        self.execute_pod_code(&config)?;
        unreachable!("should not return from execute_pod_code");
    } else {
        // Parent process: setup POD metadata
        let pod = PODMetadata {
            id: config.id.clone(),
            pid: pid as u32,
            created_at: Instant::now(),
            last_heartbeat: Instant::now(),
            status: PODStatus::Starting,
            restarts: 0,
            memory_limit_mb: config.memory_limit_mb,
            cpu_limit_cores: config.cpu_limit_cores,
            priority: PODPriority::Medium(5),
        };
        
        self.pods.insert(config.id.clone(), pod);
        
        // Setup cgroups for resource limits
        self.cgroup_manager.create_cgroup(&config.id)?;
        self.cgroup_manager.set_memory_limit(
            &config.id,
            config.memory_limit_mb
        )?;
        self.cgroup_manager.set_cpu_limit(
            &config.id,
            config.cpu_limit_cores
        )?;
        
        // Add PID to cgroup
        self.cgroup_manager.add_pid_to_cgroup(&config.id, pid as u32)?;
        
        // Register in heartbeat manager
        self.heartbeat_manager.register(
            config.id.clone(),
            pid as u32,
            config.heartbeat_interval_ms
        );
        
        Ok(config.id)
    }
}

// Child process: execute feature code
fn execute_pod_code(&self, config: &PODConfig) -> Result<()> {
    // 1. Map shared data plane
    let shared_data = Arc::clone(&self.shared_data);
    
    // 2. Load feature .so file
    let lib = unsafe {
        libloading::Library::new(&config.binary_path)?
    };
    
    // 3. Get entry point symbol
    let entry_fn: libloading::Symbol<unsafe extern "C" fn(Arc<SharedDataPlane>) -> i32> = 
        unsafe { lib.get(config.entry_point.as_bytes())? };
    
    // 4. Call feature code
    let exit_code = unsafe {
        entry_fn(shared_data)
    };
    
    // 5. Exit with code
    std::process::exit(exit_code);
}
```

#### 2. Monitor PODs (Health Check)

```rust
pub fn monitor_pods(&self) {
    loop {
        let mut pods_to_restart = Vec::new();
        
        for (pod_id, metadata) in &self.pods {
            match metadata.status {
                PODStatus::Dead | PODStatus::Disabled => {
                    // Nothing to do
                    continue;
                }
                
                PODStatus::Alive => {
                    // Check heartbeat
                    if metadata.last_heartbeat.elapsed() > Duration::from_secs(5) {
                        println!("POD {} missed heartbeat, marking unhealthy", pod_id);
                        pods_to_restart.push(pod_id.clone());
                    }
                    
                    // Check resource usage
                    let (memory_used_mb, cpu_used_pct) = 
                        self.get_pod_resource_usage(&metadata.pid);
                    
                    if memory_used_mb > (metadata.memory_limit_mb * 95 / 100) {
                        println!("POD {} memory pressure: {}%", 
                            pod_id, 
                            (memory_used_mb * 100) / metadata.memory_limit_mb);
                        // Alert but don't kill (let cgroup handle it)
                    }
                    
                    if cpu_used_pct > 95 {
                        println!("POD {} CPU maxed out: {}%", pod_id, cpu_used_pct);
                        // Alert (might indicate infinite loop)
                    }
                }
                
                _ => {}
            }
        }
        
        // Restart dead PODs
        for pod_id in pods_to_restart {
            if let Err(e) = self.restart_pod(&pod_id) {
                eprintln!("Failed to restart POD {}: {}", pod_id, e);
            }
        }
        
        // Sleep before next check
        std::thread::sleep(Duration::from_millis(100));
    }
}
```

#### 3. Restart POD on Crash

```rust
pub fn restart_pod(&mut self, pod_id: &str) -> Result<()> {
    let metadata = self.pods.get_mut(pod_id)?;
    
    // Check restart limit
    let restarts_this_hour = metadata.restarts;
    let age = metadata.created_at.elapsed();
    
    if restarts_this_hour > 10 && age < Duration::from_secs(3600) {
        // Too many restarts in last hour, disable POD
        metadata.status = PODStatus::Disabled;
        self.alert_operator(&format!(
            "POD {} disabled: {} restarts in {}s",
            pod_id,
            restarts_this_hour,
            age.as_secs()
        ))?;
        return Ok(());
    }
    
    // Kill old process (if still alive)
    unsafe {
        libc::kill(metadata.pid as i32, libc::SIGKILL);
    }
    
    // Get original config
    let config = self.get_pod_config(pod_id)?;
    
    // Spawn new process
    let new_pid = self.spawn_pod(config)?;
    
    metadata.restarts += 1;
    metadata.last_heartbeat = Instant::now();
    
    println!("Restarted POD {} (attempt {})", pod_id, metadata.restarts);
    
    Ok(())
}
```

#### 4. Route RPC Call to POD

```rust
pub fn route_rpc_call(
    &self,
    pod_id: String,
    method: String,
    args: Vec<u8>,
) -> Result<Vec<u8>> {
    // Get POD metadata
    let pod = self.pods.get(&pod_id)
        .ok_or_else(|| "POD not found")?;
    
    // Check if POD is alive
    match pod.status {
        PODStatus::Alive => {},
        PODStatus::Dead => {
            // POD is dead, try to restart
            if let Err(e) = self.restart_pod(&pod_id) {
                return Err(format!("POD {} is dead and restart failed: {}", pod_id, e));
            }
            // Retry RPC call
            return self.route_rpc_call(pod_id, method, args);
        }
        other => {
            return Err(format!("POD {} is {:?}, cannot route", pod_id, other));
        }
    }
    
    // Route via RPC
    self.router.call_rpc(
        pod.pid,
        &method,
        &args,
        Duration::from_secs(30)  // 30-second timeout
    )
}
```

---

## POD Lifecycle

### Detailed State Diagram

```
┌─────────────┐
│  CREATION   │ (fork() called)
└──────┬──────┘
       │
       ▼
┌──────────────┐
│  STARTING    │ (cgroup setup, .so loading, initialization)
│              │ Duration: 1-5 seconds
│ Actions:     │
│ - dlopen()   │
│ - init code  │
│ - register   │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│    ALIVE     │ (running, accepting requests)
│              │
│ Actions:     │
│ - Process    │
│   requests   │
│ - Send       │
│   heartbeat  │
│ - Monitor    │
│   resources  │
└──────┬───────┘
       │
       ├─────────────┬─────────────┬──────────────┐
       │             │             │              │
       ▼             ▼             ▼              ▼
   CRASH      RESOURCE     MANUAL        NORMAL
   (missed    (memory/     (admin        (clean
    beat)     cpu limit)   shutdown)     shutdown)
   │             │             │            │
   │             │             │            ▼
   │             │             │         TERMINATING
   │             │             │            │
   └─────┬───────┴─────────────┴────────────┘
         │
         ▼
    ┌─────────┐
    │  DEAD   │ (process no longer running)
    │         │
    │ Decision:
    │ - Restart?
    │ - Disable?
    │ - Alert?
    └────┬────┘
         │
         ├─────────────────────┐
         │                     │
         ▼                     ▼
    RESTARTING          DISABLED
    (spawn new)      (manual fix)
    │                     │
    └─────┬───────────────┘
          │
          ▼
      STARTING
```

### Timeline Examples

**Normal Operation:**
```
T=0s:     Spawn POD (fork)
T=0-1s:   STARTING (cgroup setup, .so load)
T=1s:     Enter ALIVE
T=1-3600s: Process requests, send heartbeats every 1s
T=3600s:  Shutdown signal (clean), enter TERMINATING
T=3601s:  DEAD
```

**Crash & Restart:**
```
T=0s:     Spawn POD
T=1s:     ALIVE (processing requests)
T=45s:    Crash (segfault, out of memory, assertion)
T=45s:    Process dies, POD no longer responsive
T=46s:    Monitor detects heartbeat timeout
T=46s:    POD status: Dead → restart triggered
T=46-47s: RESTARTING (spawn new process)
T=47s:    New POD: STARTING
T=48s:    New POD: ALIVE
T=48-90s: Processing requests again (other PODs unaffected)
```

**Resource Limit Exceeded:**
```
T=100s:   POD memory usage: 240MB (of 256MB limit)
T=101s:   Allocate 20MB more → 260MB
T=101s:   Memory exceeds cgroup limit (256MB)
T=101s:   Linux OOM killer activates
T=101s:   Process killed by kernel
T=102s:   Monitor detects crash
T=102s:   Auto-restart triggered
T=103s:   New POD spawned with fresh 256MB
```

---

## Shared Data Plane

### Components

**1. Cache Layer (Read-Only for PODs)**
```
┌─────────────────────────────────┐
│ Shared Cache (mmap, 1GB)        │
│                                 │
│ ┌──────────────────────────────┐│
│ │ Design System Tokens         ││
│ │ (colors, spacing, fonts)     ││
│ │ Size: 10MB                   ││
│ └──────────────────────────────┘│
│                                 │
│ ┌──────────────────────────────┐│
│ │ Feature Configuration         ││
│ │ (versioned, immutable)        ││
│ │ Size: 50MB                    ││
│ └──────────────────────────────┘│
│                                 │
│ ┌──────────────────────────────┐│
│ │ Component Library             ││
│ │ (Button, Input, Modal, etc)  ││
│ │ Size: 100MB                   ││
│ └──────────────────────────────┘│
│                                 │
│ ┌──────────────────────────────┐│
│ │ User-Specific Data Cache      ││
│ │ (session, preferences)        ││
│ │ Size: 500MB (grows)           ││
│ └──────────────────────────────┘│
│                                 │
└─────────────────────────────────┘
```

**2. RPC Router**
```
┌─────────────────────────────────┐
│ RPC Router (event-driven)       │
│                                 │
│ For each POD:                   │
│ ┌──────────────────────────────┐│
│ │ Request Queue (mmap buffer) │ │
│ │ POD 1 → Router → POD 2      │ │
│ │ Max request: 1MB             │ │
│ └──────────────────────────────┘│
│                                 │
│ Mechanism: eventfd (epoll)      │
│ Notification: <100ns (IPC)      │
│                                 │
│ Size: 10MB (100 PODs × 100KB)   │
│                                 │
└─────────────────────────────────┘
```

**3. Event Bus (Pub/Sub)**
```
┌─────────────────────────────────┐
│ Event Bus (broadcast)           │
│                                 │
│ Event topics:                   │
│ - cache_invalidated             │
│ - feature_deployed              │
│ - feature_rollback              │
│ - anomaly_detected              │
│ - health_check                  │
│                                 │
│ Subscribers per topic: 1-50     │
│ Broadcast: <1µs per subscriber  │
│                                 │
│ Size: 100MB (ring buffer)       │
│                                 │
└─────────────────────────────────┘
```

**4. Metrics Collector**
```
┌─────────────────────────────────┐
│ Metrics Store (time-series)     │
│                                 │
│ Per-POD metrics:                │
│ - Requests/sec                  │
│ - Latency (p50, p95, p99)       │
│ - Errors/sec                    │
│ - Memory usage                  │
│ - CPU usage                     │
│ - GC pauses                     │
│                                 │
│ Retention: 1 hour (in-memory)   │
│ Persistence: off-loaded to disk │
│                                 │
│ Size: 500MB (1000 PODs × 500KB) │
│                                 │
└─────────────────────────────────┘
```

### Memory Layout

```
Virtual Memory (Linux mmap)

0x00000000 ┌─────────────────────────────┐
           │ POD 1 Heap (256MB)          │
           │ Private to POD 1            │
           │ Read/Write                  │
           │ OOM killer enforces limit   │
0x10000000 └─────────────────────────────┘
           ┌─────────────────────────────┐
           │ POD 2 Heap (256MB)          │
           │ Private to POD 2            │
           │ Read/Write                  │
0x20000000 └─────────────────────────────┘
           
           [... more PODs ...]
           
0xA0000000 ┌─────────────────────────────┐
           │ SHARED DATA PLANE (10GB)    │
           │ All PODs + Parent can access│
           │ mmap'd file: /dev/shm/ekos │
           │ Read/Write (protected by    │
           │ mutexes/atomics)            │
           │                             │
           │ ┌─────────────────────────┐ │
           │ │ Cache Layer (1GB)       │ │
           │ │ Read-only from PODs     │ │
           │ └─────────────────────────┘ │
           │                             │
           │ ┌─────────────────────────┐ │
           │ │ RPC Buffers (10MB)      │ │
           │ │ One per POD pair        │ │
           │ └─────────────────────────┘ │
           │                             │
           │ ┌─────────────────────────┐ │
           │ │ Event Bus (100MB)       │ │
           │ │ Ring buffer             │ │
           │ └─────────────────────────┘ │
           │                             │
           │ ┌─────────────────────────┐ │
           │ │ Metrics (500MB)         │ │
           │ │ Time-series per POD     │ │
           │ └─────────────────────────┘ │
           │                             │
0xF0000000 └─────────────────────────────┘
           ┌─────────────────────────────┐
           │ Parent (EKOS Core) Heap (2GB)
           │ POD Manager, RPC Router     │
           │ Shared Data Plane mgmt      │
0x100000000└─────────────────────────────┘
```

---

## RPC Communication Protocol

### Design Goals

1. **Ultra-low latency** (<10µs round-trip)
2. **Zero-copy where possible** (shared memory references)
3. **Efficient serialization** (Protobuf, custom binary)
4. **Simple, predictable** (no network stack complexity)

### Protocol Specification

**Message Format (Binary):**

```
[Header (16 bytes)] [Payload (variable)] [Checksum (4 bytes)]

Header:
  Byte 0-1:    Magic (0xABCD)
  Byte 2:      Version (0x01)
  Byte 3:      Flags (bit 0: async, bit 1: retry, ...)
  Byte 4-7:    Payload size (u32, little-endian)
  Byte 8-11:   Request ID (u32, for correlation)
  Byte 12-13:  Target POD ID (u16)
  Byte 14-15:  Reserved

Payload:
  Variable length, serialized request/response
  Format: Protobuf or custom binary

Checksum:
  CRC32 of entire message
```

### RPC Flow

```
POD A wants to call POD B:

1. POD A: Serialize request to protobuf
   request = FilterRequest {
     region: "APAC",
     date_range: (2026-01-01, 2026-07-12),
     data: SharedMemoryRef { addr: 0xA0000000, len: 1024 }
   }

2. POD A: Write to shared buffer
   shared_data.rpc_buffers[POD_A][POD_B].write(request)

3. POD A: Notify POD B (eventfd)
   eventfd_write(pod_b_eventfd, 1)
   Latency: < 100 nanoseconds

4. POD B: Event handler wakes up (epoll)
   on_rpc_event() {
     request = shared_data.rpc_buffers[POD_A][POD_B].read()
     result = handle_filter_request(request)
     shared_data.rpc_buffers[POD_B][POD_A].write(result)
     eventfd_write(pod_a_eventfd, 1)
   }

5. POD A: Response arrives
   response = shared_data.rpc_buffers[POD_B][POD_A].read()
   
Total latency: 1-10µs
Breakdown:
  - Protobuf serialization: 1-3µs
  - Memory copy: <1µs
  - eventfd notification: <100ns (×2)
  - Function execution: 1-100µs (depends on operation)
```

### RPC Error Handling

```rust
pub enum RpcError {
    Timeout,              // No response within 30s
    PODDead,             // Target POD not alive
    SerializationError,  // Protobuf error
    ChecksumMismatch,    // Data corruption
    CapacityExceeded,    // Message queue full
}

pub fn rpc_call_with_retry(
    &self,
    target_pod: String,
    method: String,
    args: Vec<u8>,
) -> Result<Vec<u8>> {
    const MAX_RETRIES: u32 = 3;
    
    for attempt in 0..MAX_RETRIES {
        match self.rpc_call(&target_pod, &method, &args) {
            Ok(response) => return Ok(response),
            Err(RpcError::PODDead) => {
                // POD is dead, try to restart
                if attempt < MAX_RETRIES - 1 {
                    self.pod_manager.restart_pod(&target_pod)?;
                    // Retry
                    continue;
                } else {
                    return Err(RpcError::PODDead);
                }
            }
            Err(RpcError::Timeout) => {
                if attempt < MAX_RETRIES - 1 {
                    // Retry with exponential backoff
                    std::thread::sleep(
                        Duration::from_millis(100 * 2_u64.pow(attempt))
                    );
                    continue;
                } else {
                    return Err(RpcError::Timeout);
                }
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(RpcError::Timeout)
}
```

---

## Resource Limits & Enforcement

### cgroup Configuration

**Setup (Linux cgroup v2):**

```bash
# Create cgroup for EKOS
mkdir -p /sys/fs/cgroup/ekos

# Set limits for Widget-A POD (256MB memory, 1 CPU)
echo "268435456" > /sys/fs/cgroup/ekos/widget-a/memory.max

# CPU limit: 1 core
echo "100000" > /sys/fs/cgroup/ekos/widget-a/cpu.max

# Add POD process to cgroup
echo $POD_PID > /sys/fs/cgroup/ekos/widget-a/cgroup.procs
```

**Memory Management:**

```
POD memory limit: 256MB

┌──────────────────────────────────┐
│ POD Process Heap (256MB)         │
│                                  │
│ Allocating...                    │
│ malloc(100MB) ✓                  │
│ malloc(100MB) ✓                  │
│ malloc(50MB) ✓ (total: 250MB)   │
│ malloc(20MB) ✗ (exceeds limit)   │
│                                  │
│ Kernel OOM killer activates:     │
│ 1. Find highest-memory process   │
│ 2. Send SIGKILL                  │
│ 3. Process dies                  │
│                                  │
│ → Parent detects crash           │
│ → Parent restarts POD            │
│ → New POD gets fresh 256MB       │
└──────────────────────────────────┘
```

**CPU Management:**

```
POD CPU limit: 1 core (100% of 1 core)

Timeline:
T=0:   POD starts using 50% CPU
T=1:   POD uses 80% CPU
T=2:   POD uses 100% CPU
T=3:   POD tries to use 150% CPU
       → Kernel throttles POD
       → POD can only use 100% of 1 core
       → Other cores available for other PODs

Monitoring:
  If CPU_USED > 95% for >30 seconds:
  → Alert operator ("POD xyz may be in infinite loop")
  → Option 1: Increase CPU limit
  → Option 2: Kill POD and investigate
```

### Memory Pressure Handling

```
EKOS cluster-wide memory: 32GB
Parent process: 2GB
PODs: 30 × 256MB = 7.5GB
Total used: 9.5GB
Available: 22.5GB

Normal operation:
  All PODs: healthy
  No OOM killer

Scenario: Widget-A memory leak
  Widget-A allocates more and more
  Widget-A reaches 256MB limit
  Kernel: OOM kill Widget-A process
  Parent: Detects crash
  Parent: Restart Widget-A
  Result: Only Widget-A affected (1 minute downtime)

Compare to monolithic:
  Widget-A memory leak
  Leaks into parent process
  Parent memory grows
  Parent reaches memory limit
  Kernel: OOM kill parent
  Result: ENTIRE EKOS platform DOWN (1 hour recovery)
```

---

## Health Monitoring & Auto-Restart

### Heartbeat Mechanism

**Design:**

```
Every POD sends heartbeat every 1 second

POD code:
  while true {
    process_requests()
    
    // Every 1 second, ping parent
    heartbeat_manager.ping()
    
    sleep(1 second)
  }

Parent code:
  monitor_loop {
    for each POD:
      if now - pod.last_heartbeat > 5 seconds:
        mark_pod_dead(pod)
        restart_pod(pod)
      
    sleep(100ms)
  }
```

**Metrics:**

```
Heartbeat telemetry:

┌─────────────────────────────────┐
│ Widget-A POD Health             │
│                                 │
│ Heartbeats sent: 3600/3600      │
│ Heartbeats missed: 0            │
│ Uptime: 1 hour                  │
│ Restarts (this hour): 0         │
│ Crash count (total): 0          │
│ Latest crash: never             │
│ Status: HEALTHY                 │
└─────────────────────────────────┘

┌─────────────────────────────────┐
│ Feature-Collab POD Health       │
│                                 │
│ Heartbeats sent: 3600/3600      │
│ Heartbeats missed: 2            │
│ Uptime: 1 hour                  │
│ Restarts (this hour): 1         │
│ Crash count (total): 5          │
│ Latest crash: 5 minutes ago     │
│ Status: UNHEALTHY               │
│ Alert: "POD restarted 5x/hour" │
└─────────────────────────────────┘
```

### Restart Limits

```
Policy: Max 10 restarts per hour per POD

Timeline:

T=0:    Widget crashes → restart 1/10 ✓
T=30:   Widget crashes → restart 2/10 ✓
T=60:   Widget crashes → restart 3/10 ✓
T=90:   Widget crashes → restart 4/10 ✓
T=120:  Widget crashes → restart 5/10 ✓
T=150:  Widget crashes → restart 6/10 ✓
T=180:  Widget crashes → restart 7/10 ✓
T=210:  Widget crashes → restart 8/10 ✓
T=240:  Widget crashes → restart 9/10 ✓
T=270:  Widget crashes → restart 10/10 ✓
T=300:  Widget crashes → DISABLED
        "POD has exceeded restart limit"
        "Manual intervention required"
        Operator receives alert
        Must fix underlying issue and manually re-enable

Hourly reset:
  After 1 hour, restart counter resets
  T=3601: New hour begins, restart counter = 0
```

---

## Integration with EKOS

### Feature Deployment via POD

**Traditional Flow (Monolithic):**
```
User request: "Generate widget for product filter"
  ↓
LLM generates code (widget.js)
  ↓
Verify code (types, security, performance)
  ↓
Deploy to EKOS runtime (hot-reload)
  ↓
Widget live, available to all users
```

**New Flow (POD-Based):**
```
User request: "Generate widget for product filter"
  ↓
LLM generates code (widget.js)
  ↓
Compile to binary (widget.so)
  ↓
Verify binary (types, security, performance)
  ↓
POD Manager: Spawn POD
  - fork() new process
  - dlopen(widget.so)
  - Register with RPC router
  ↓
Widget live, available to all users
  ↓
Feature updates:
  - Generate new widget.so (v2)
  - Spawn new POD (parallel to v1)
  - Route 10% traffic to new POD
  - Monitor metrics
  - Gradual rollout: 10% → 50% → 100%
  - Kill old POD when 100% on new
```

**Benefits:**
```
Old way (monolithic):
  - Deploy new widget
  - Affects entire EKOS if issue
  - Rollback requires restart
  - Downtime if something breaks

New way (POD-based):
  - Spawn new POD (parallel to old)
  - Only affects that feature
  - Rollback: delete new POD
  - Zero downtime
```

### Feature Communication

**Widget-A calls Widget-B:**

```rust
// Widget-A code (running in POD)
pub fn widget_a_handler(request: FilterRequest) -> Result<String> {
    // Call Widget-B to fetch data
    let data = rpc_call(
        pod_id: "widget-b",
        method: "fetch_regions",
        args: request.encode()
    )?;
    
    // Process data
    let result = process_regions(&data);
    Ok(result)
}

// Routing:
// Request → RPC Router → Widget-B POD (via shared memory)
// Latency: 1-10µs
```

---

## Integration with Self-Healing

### Self-Healing as POD

**Architecture:**

```
EKOS Core (parent)
├── Widget-A POD
├── Widget-B POD
├── Self-Healing POD (special POD, higher priority)
│   ├── Anomaly Detector (runs continuously)
│   ├── Fix Generator (calls LLM)
│   ├── Verifier (formal verification)
│   └── Patcher (applies fixes)
└── Other PODs
```

**Self-Healing Workflow:**

```
1. Self-Healing POD: Detect Anomaly
   "Widget-A latency increased 3x"
   
2. Self-Healing POD: Generate Fix
   LLM: "Widget-A has inefficient allocator"
   Generate: widget-a-optimized.so (v2)
   
3. Self-Healing POD: Verify
   Formal verification: ✓ Correctness proven
   Performance estimate: +40% latency improvement
   
4. POD Manager: Load New Code
   dlopen(widget-a-optimized.so) into Widget-A POD
   (same POD, new code)
   
5. Widget-A POD: Code Swap
   Old code: ~
   New code: ↓ (active)
   
6. Test: Send shadow traffic
   Compare old vs new: results identical ✓
   
7. Activate: All traffic → new code
   Latency: 120ms → 72ms ✓
   
Timeline: 5-10 minutes
Other PODs: unaffected during entire process
```

---

## Integration with VectorVolt

### Data Plane Access

**POD → VectorVolt Communication:**

```
POD (EKOS feature) wants to query VectorVolt:

1. POD: Serialize query
   query = "SELECT * FROM users WHERE region='APAC'"

2. POD: Write to VectorVolt request buffer (shared memory)
   shared_data.vectorvolt_requests.write(query)

3. POD: Notify VectorVolt (eventfd)
   eventfd_write(vectorvolt_eventfd, 1)
   Latency: <100ns

4. VectorVolt: Process query
   Execute in-process (no network overhead!)
   Return result to shared buffer

5. POD: Read result
   response = shared_data.vectorvolt_responses.read()
   Latency: 1-100µs total (depending on query)

Latency breakdown:
  - Query serialization: 1µs
  - Memory copy: <1µs
  - Notification: <100ns
  - VectorVolt execution: 1-100µs (query dependent)
  - Total: 2-102µs (vs. 3-5ms with network)
```

**Shared Memory Layout:**

```
EKOS Shared Data Plane (10GB)

┌──────────────────────────────────┐
│ VectorVolt Connection (500MB)    │
│                                  │
│ ┌────────────────────────────┐   │
│ │ Query Requests (100MB)     │   │
│ │ One request queue from all │   │
│ │ PODs → VectorVolt          │   │
│ └────────────────────────────┘   │
│                                  │
│ ┌────────────────────────────┐   │
│ │ Query Responses (100MB)    │   │
│ │ Results back to PODs       │   │
│ └────────────────────────────┘   │
│                                  │
│ ┌────────────────────────────┐   │
│ │ Read Cache (300MB)         │   │
│ │ PODs can read directly     │   │
│ │ (hot results, no query)    │   │
│ └────────────────────────────┘   │
│                                  │
└──────────────────────────────────┘

VectorVolt Process (separate)
└── Accesses shared data via mmap
    (same 10GB region)
```

---

## Security Model

### POD Isolation

**Memory Isolation:**
```
POD 1 Heap: 0x10000000 - 0x20000000 (isolated from others)
POD 2 Heap: 0x20000000 - 0x30000000 (isolated from others)

Can POD 1 read POD 2 memory?
  No (separate address spaces)

Can POD 1 crash POD 2?
  No (separate processes, signals limited)

Can POD 1 escape to parent?
  No (only via RPC calls, checked)
```

**Capability-Based Access:**
```
Each POD declares what it can access:

Widget-A capabilities:
  - Read: [users_table, regions_table]
  - Write: []
  - RPC: [widget-b, cache-layer]
  - Denied: [admin_functions, sensitive_audit_log]

POD tries to: call admin function
  → RPC router checks: "admin_functions" in Widget-A capabilities?
  → No → AccessDenied error

POD tries to: write to users_table
  → Memory access check: "Write: users_table" allowed?
  → No → SegFault or Access Denied (depending on implementation)
```

**Data Isolation:**
```
Org A's data: isolated from Org B's data
Org A POD can only read: {org_a_users, org_a_orders}
Org A POD cannot read: {org_b_users, org_b_orders}

Enforcement:
  - RPC calls filtered by capability
  - Cache access filtered by org_id
  - Database queries tagged with org_id
```

---

## Performance Characteristics

### Latency

**RPC Call Latency:**

```
Scenario: Widget-A calls Widget-B

Components:
  - Serialization (Protobuf): 1-3µs
  - Memory copy: <1µs
  - eventfd notification: <100ns
  - Context switch + function execution: 1-10µs
  - Deserialization: 1-3µs

Total: 3-17µs (p95)

Comparison:
  - Kubernetes network RPC: 3-5ms (1000x slower)
  - Direct function call (monolithic): 0.1-1µs (10x faster)
  - EKOS POD: 3-17µs (good compromise)
```

**POD Spawn Latency:**

```
New feature requested: "Generate Widget-C"

T=0s:  Code generation complete
T=1s:  Compilation complete
T=2s:  Spawn POD (fork)
T=2s:  cgroup setup: <100ms
T=2.1s: Load .so (dlopen): <500ms
T=2.6s: POD enters ALIVE state
T=2.6s: First request processed

Feature available: <3 seconds from generation complete
```

**Crash Recovery Latency:**

```
Widget crashes (e.g., segfault):

T=0s:   Widget executing
T=0.5s: Process crashes
T=1s:   Monitor detects (heartbeat timeout)
T=1.1s: POD Manager kills old process
T=1.2s: New POD spawned
T=2.2s: New POD ready (cgroup + .so loading)

Total recovery time: 2 seconds
User experience: "Widget temporarily unavailable, retrying..."
                (retry succeeds after 2-3 seconds)
```

### Throughput

**Request Throughput (POD-based):**

```
100 PODs, each handling 1000 req/sec
Total EKOS throughput: 100,000 requests/sec

Memory usage:
  100 PODs × 256MB = 25.6GB
  Shared data: 10GB
  Parent process: 2GB
  Total: ~38GB

This supports:
  10,000 concurrent users (per feature)
  1M+ features across all orgs
  Scales horizontally by adding more machines
```

### Memory Efficiency

```
Memory per POD: 256MB
  - Heap: ~200MB (actual usage)
  - Stack: 2MB
  - Process overhead: 54MB

Memory efficiency:
  Widget-A: 256MB → serves 10,000 users
  Per-user memory: 25.6KB

Comparison to monolithic:
  Monolithic EKOS: 2GB
  Monolithic per-user: 200KB (all users share)
  POD-based: 25.6KB per feature per users
  
Winner: Monolithic slightly better
But POD provides isolation benefits worth the trade-off
```

---

## Implementation Guide

### Tech Stack

```
Language: Rust (for POD Manager)
  Reason: Memory safety, performance, systems programming

Libraries:
  - nix: low-level Unix syscalls (fork, execve)
  - libc: C library bindings (cgroup, eventfd)
  - libloading: dynamic loading (.so files)
  - prost: Protobuf serialization
  - parking_lot: synchronization primitives
  - metrics: prometheus metrics

Build System: Cargo + custom build scripts

OS: Linux (cgroup support required)
  - cgroup v2 for resource limits
  - epoll for event notification
  - mmap for shared memory
```

### Development Phases

**Phase 1: POD Manager Core (4 weeks)**
```
1. Spawner: fork, execve, cgroup setup
2. Monitor: heartbeat detection, resource monitoring
3. Reaper: cleanup dead processes
4. Basic routing: send requests to PODs
5. Tests: spawn/crash/restart cycles
```

**Phase 2: RPC Layer (3 weeks)**
```
1. Shared memory setup (mmap)
2. Message serialization (Protobuf)
3. eventfd-based notification
4. Routing logic (POD to POD)
5. Error handling and retries
```

**Phase 3: Integration (2 weeks)**
```
1. Integrate with code generation (produce .so)
2. Integrate with deployment (spawn PODs)
3. Integrate with rollback (canary per-POD)
4. Integrate with metrics collection
```

**Phase 4: Self-Healing POD (2 weeks)**
```
1. Self-healing as POD (special priority)
2. Code loading (dlopen new .so)
3. Rollback mechanism
4. Monitoring and alerts
```

Total: 11 weeks (3 months)

---

## Example Workflows

### Workflow 1: Feature Deployment

```
Timeline: "Deploy new Widget-C"

T=0min:  User requests: "I need widget to filter by department"
T=1min:  LLM generates widget code (JavaScript)
T=1:30:  Compilation to WASM/binary (widget-c.so)
T=2min:  Verification (types, security, performance)
T=2:30:  POD Manager: spawn_pod("widget-c", config)
         ├─ fork() new process
         ├─ cgroup setup (256MB memory, 1 CPU)
         └─ dlopen(widget-c.so)
T=3min:  Widget-C enters ALIVE
         ├─ Registered with RPC router
         ├─ Heartbeat: ✓
         └─ Ready for requests
T=3:30:  Feature flag: widget-c.rollout = 0% (disabled)
T=4min:  Canary: widget-c.rollout = 1% (1% of users)
T=5min:  Monitor: error_rate=0.0%, latency_ms=45, memory_mb=80
T=10min: Increase: widget-c.rollout = 10%
T=15min: Increase: widget-c.rollout = 100% (full rollout)

Result:
  - Widget live
  - Performance: p99 latency 45ms (within budget)
  - Zero errors
  - Other widgets: unaffected
```

### Workflow 2: POD Crash & Recovery

```
Timeline: "Widget-A has memory leak"

T=0s:     Widget-A: Normal operation (60MB memory)
T=1min:   Widget-A: Memory growing (100MB)
T=5min:   Widget-A: Memory growing (200MB)
T=10min:  Widget-A: Memory growing (240MB)
T=15min:  Widget-A: Memory growing (260MB → exceeds 256MB limit)
T=15:01s: Kernel OOM killer: SIGKILL Widget-A process
T=15:02s: Monitor: detects heartbeat timeout
T=15:03s: POD Manager: spawn_pod("widget-a", config) [restart 1]
          ├─ fork() new process
          ├─ cgroup setup
          └─ dlopen(widget-a.so)
T=15:04s: Widget-A: ALIVE again, fresh 256MB memory

Result:
  - Widget-A: 1-minute downtime (auto-restart)
  - Other widgets: unaffected during entire incident
  - No manual intervention
  - Auto-restart recorded in metrics

Compare to monolithic:
  - Memory leak → kills entire EKOS
  - All 10,000 users affected
  - 1-2 hour recovery
  - Manual restart required
```

### Workflow 3: Self-Healing

```
Timeline: "Self-healing detects latency issue"

T=0min:   Widget-A: baseline p99 latency = 120ms
T=60min:  Widget-A: p99 latency = 350ms (3x spike)
T=60:05:  Self-Healing POD: anomaly detected
T=60:10:  Self-Healing POD: analyze Widget-A code
          Diagnosis: "Memory allocator is inefficient"
T=60:15:  Self-Healing POD: generate optimized code
          LLM: "Use arena-based allocator instead"
T=60:30:  Self-Healing POD: formal verification
          Prove: "Correctness maintained, latency improves 40%"
T=60:45:  Self-Healing POD: shadow execution
          Run optimized code on 1% shadow traffic
          Compare results: identical ✓
T=61:00:  Self-Healing POD: canary rollout
          10% of traffic → new code
T=61:20:  Monitor: p99 latency = 150ms ✓
T=61:30:  Increase rollout: 100%
T=62:00:  Widget-A: p99 latency = 72ms (down from 350ms!)

Result:
  - Widget auto-healed
  - 50% latency improvement
  - No restart required
  - Zero downtime
  - Zero manual intervention
```

---

## Monitoring & Observability

### POD Metrics

**Per-POD Metrics (collected every 10s):**

```
widget-a:
  requests_per_sec: 150
  latency_p50_ms: 25
  latency_p95_ms: 45
  latency_p99_ms: 72
  errors_per_sec: 0.05
  memory_used_mb: 180
  memory_limit_mb: 256
  cpu_used_percent: 45
  cpu_limit_percent: 100
  gc_pause_ms: 2
  uptime_seconds: 3600
  restarts_this_hour: 0
  status: ALIVE

widget-b:
  requests_per_sec: 200
  latency_p50_ms: 18
  latency_p95_ms: 38
  latency_p99_ms: 85
  errors_per_sec: 0.10
  memory_used_mb: 220
  memory_limit_mb: 256
  cpu_used_percent: 65
  cpu_limit_percent: 100
  gc_pause_ms: 5
  uptime_seconds: 7200
  restarts_this_hour: 0
  status: ALIVE

feature-collab:
  requests_per_sec: 50
  latency_p50_ms: 35
  latency_p95_ms: 120
  latency_p99_ms: 250
  errors_per_sec: 0.50
  memory_used_mb: 380  ← memory pressure
  memory_limit_mb: 512
  cpu_used_percent: 85
  cpu_limit_percent: 200
  gc_pause_ms: 10
  uptime_seconds: 600
  restarts_this_hour: 3  ← too many restarts
  status: UNHEALTHY
  alerts:
    - "High error rate: 0.5/sec"
    - "Memory pressure: 74% of limit"
    - "Frequent restarts: 3 in last hour"
```

### Dashboards

**EKOS Platform Dashboard:**

```
┌───────────────────────────────────────────┐
│ EKOS Platform Health                      │
├───────────────────────────────────────────┤
│ Total PODs: 150                           │
│ PODs Healthy: 147 (98%)                  │
│ PODs Unhealthy: 3                        │
│ Total Throughput: 75,000 req/sec         │
│ P99 Latency: 120ms                       │
│ Error Rate: 0.08%                        │
│ Memory Used: 32GB / 40GB (80%)           │
├───────────────────────────────────────────┤
│ Top Problematic PODs:                    │
│ 1. feature-collab: 3 restarts/hour       │
│ 2. widget-analytics: high latency (250ms)│
│ 3. widget-export: high memory (380MB)    │
└───────────────────────────────────────────┘
```

### Alerting

```
Alert Rules:

1. POD Restarting Too Often
   if: restarts_per_hour > 5
   action: "Investigate POD code/resources"
   severity: MEDIUM

2. High Error Rate
   if: error_rate > 1%
   action: "Check POD logs, may need rollback"
   severity: HIGH

3. Memory Pressure
   if: memory_used > 90% of limit
   action: "Increase memory limit or optimize POD"
   severity: MEDIUM

4. Unresponsive POD
   if: heartbeat_missed > 3
   action: "POD is dead, auto-restart triggered"
   severity: HIGH

5. CPU Maxed Out
   if: cpu_used > 95% for > 5 minutes
   action: "POD may be in infinite loop, kill and investigate"
   severity: MEDIUM
```

---

## Appendix

### A. Protobuf RPC Message Definition

```protobuf
syntax = "proto3";

package ekos.rpc;

message RpcRequest {
  string method = 1;
  bytes args = 2;
  uint32 timeout_sec = 3;
}

message RpcResponse {
  bytes result = 1;
  string error = 2;
  int32 exit_code = 3;
}

message PODMetrics {
  string pod_id = 1;
  uint64 requests_per_sec = 2;
  uint32 latency_p50_ms = 3;
  uint32 latency_p95_ms = 4;
  uint32 latency_p99_ms = 5;
  uint32 errors_per_sec = 6;
  uint32 memory_used_mb = 7;
  uint32 memory_limit_mb = 8;
  uint32 cpu_used_percent = 9;
}
```

### B. cgroup Configuration

```bash
#!/bin/bash
# setup-pod-cgroup.sh

POD_ID=$1
MEMORY_MB=$2
CPU_CORES=$3

# Create cgroup v2
mkdir -p /sys/fs/cgroup/ekos/$POD_ID

# Set memory limit
echo "$((MEMORY_MB * 1024 * 1024))" > \
  /sys/fs/cgroup/ekos/$POD_ID/memory.max

# Set CPU limit
echo "100000" > \
  /sys/fs/cgroup/ekos/$POD_ID/cpu.max

# Note: CPU core affinity (optional)
# echo "0-$((CPU_CORES-1))" > \
#   /sys/fs/cgroup/ekos/$POD_ID/cpuset.cpus

echo "POD $POD_ID cgroup configured"
```

### C. Heartbeat Implementation

```rust
pub struct HeartbeatManager {
    registered_pods: HashMap<String, (u32, Instant)>,
}

impl HeartbeatManager {
    pub fn register(&mut self, pod_id: String, pid: u32, interval_ms: u32) {
        self.registered_pods.insert(pod_id, (pid, Instant::now()));
    }
    
    pub fn ping(&mut self, pod_id: &str) {
        if let Some((_, last_ping)) = self.registered_pods.get_mut(pod_id) {
            *last_ping = Instant::now();
        }
    }
    
    pub fn check_health(&self) -> Vec<String> {
        let mut dead_pods = Vec::new();
        let now = Instant::now();
        
        for (pod_id, (_, last_ping)) in &self.registered_pods {
            if now.duration_since(*last_ping) > Duration::from_secs(5) {
                dead_pods.push(pod_id.clone());
            }
        }
        
        dead_pods
    }
}
```

---

## Document Control

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-07-12 | Initial design, complete specification |

**Approval Status:**
- [ ] Architecture Team
- [ ] Engineering Team
- [ ] Infrastructure Team

---

**END OF SPECIFICATION**
