# Intel Silicon Research Handover — VectorVolt Integration Guide

> **Purpose:** This document captures all findings from analyzing 16 Intel specification PDFs stored in `C:\VectorVolt\intel\`. It maps each technology to VectorVolt's 6-phase architecture and provides implementation-ready details (CPUID leaves, MSR addresses, descriptor formats, C#/.NET integration patterns). Hand this to the VectorVolt development chat window for incorporation into requirements and implementation plan.

> **Date:** 2026-04-14

---

## Table of Contents

1. [Executive Summary — What to Build](#1-executive-summary)
2. [TIER 1: Intel DSA (Data Streaming Accelerator)](#2-intel-dsa)
3. [TIER 1: PCLMULQDQ / Carry-Less Multiplication](#3-pclmulqdq)
4. [TIER 1: Processor Topology Enumeration](#4-topology)
5. [TIER 2: MKTME (Multi-Key Total Memory Encryption)](#5-mktme)
6. [TIER 2: 5-Level Paging (LA57)](#6-5level-paging)
7. [TIER 2: FRED (Flexible Return and Event Delivery)](#7-fred)
8. [TIER 2: Speculative Execution Side-Channel Mitigations](#8-spectre)
9. [TIER 3: IAA (In-Memory Analytics Accelerator)](#9-iaa)
10. [TIER 3: Intel Scalable I/O Virtualization](#10-scalable-iov)
11. [TIER 3: BFloat16 Numerics](#11-bf16)
12. [TIER 3: PMU Sharing Guidelines](#12-pmu)
13. [TIER 3: MCA (Machine Check Architecture) Enhancements](#13-mca)
14. [TIER 3: Intel Key Locker](#14-key-locker)
15. [TIER 3: Runtime Microcode Update](#15-microcode)
16. [TIER 3: CPUID Passthrough / AVX10 / AMX](#16-cpuid-passthrough)
17. [Phase-by-Phase Integration Matrix](#17-matrix)
18. [New Classes / Interfaces to Add](#18-new-classes)
19. [C#/.NET Integration Patterns](#19-csharp-patterns)

---

## 1. Executive Summary — What to Build <a name="1-executive-summary"></a>

### Top 3 Game-Changers for VectorVolt

| # | Technology | Phase | Impact |
|---|-----------|-------|--------|
| 1 | **Intel DSA** — Hardware-accelerated memcpy, CRC, delta records, dualcast | Phase 3 (WAL) + Phase 2 (Engine) | Zero-CPU-cost WAL writes with inline CRC; hardware delta records reduce write amplification by 10-100x; dualcast for synchronous replication |
| 2 | **MKTME** — Per-page hardware memory encryption with per-tenant keys | Phase 6 (Multi-Tenancy) | Zero-overhead tenant isolation at silicon level; up to 32K unique keys; no software crypto overhead |
| 3 | **Topology Enumeration** — CPUID 0x1F for Package/Die/Tile/Module/Core/SMT | Phase 1 (HAL) | Correct NUMA-aware core pinning; cache-sharing detection; foundation for single-writer-per-core |

### Supporting Technologies

| Technology | Phase | Value |
|-----------|-------|-------|
| PCLMULQDQ | Phase 3 + Phase 5 | Hardware CRC32C, AES-GCM for encryption-at-rest |
| FRED | Phase 4 | 2-4x faster interrupt delivery for kernel bypass |
| 5-Level Paging | Phase 1 + Phase 6 | 128 PB virtual address space for memory-mapped everything |
| Spectre Mitigations | Phase 2 + Phase 6 | Know the performance tax; Enhanced IBRS detection |
| IAA Compression | Phase 5 | Hardware-accelerated cold-data compression (2.5-10x faster than SW) |
| Scalable IOV | Phase 4 | Software-composable NIC queue assignment |
| BFloat16 | Phase 5 | Hardware FMA for vectorized analytics |

---

## 2. Intel DSA (Data Streaming Accelerator) <a name="2-intel-dsa"></a>

**Source:** `341204-006-intel-data-streaming-accelerator-spec.pdf` (288 pages, Rev 3.0, June 2025)

### 2.1 What It Is

Intel DSA is a PCIe Root Complex Integrated Endpoint that performs DMA data movement and transformation operations. It offloads memcpy, memset, compare, CRC, DIF (Data Integrity Field), delta records, dualcast, scatter-gather, and type conversion from the CPU.

### 2.2 Why VectorVolt Needs This

| VectorVolt Operation | DSA Operation | Benefit |
|---------------------|---------------|---------|
| WAL entry write | Copy with CRC (0x11) | Single HW op: copy data + generate CRC checksum |
| WAL delta journaling | Create Delta Record (0x07) | HW generates minimal diff between old/new page (10-byte deltas) |
| WAL delta apply (recovery) | Apply Delta Record (0x08) | HW applies delta to reconstruct page |
| Synchronous replication | Memory Copy with Dualcast (0x09) | Single HW op writes to TWO destinations |
| Page zero on alloc | Fill (0x04) | HW fills memory with pattern, zero CPU |
| Shard comparison (dedup) | Compare (0x05) | HW compares two memory regions |
| Integrity verification | DIF Check (0x12) / DIX Generate (0x17) | HW data integrity on every I/O |
| Scatter-gather I/O | Gather Copy (0x1C) / Scatter Copy (0x1D) | HW scatter-gather for non-contiguous buffers |
| Analytics reduction | Reduce (0x19) / Gather Reduce (0x1B) | HW add/min/max/and/or/xor with type conversion |

### 2.3 Device Discovery

- DSA is a PCIe device — enumerate via standard PCI config space
- Each SoC may have multiple DSA instances
- Each instance exposed as Root Complex Integrated Endpoint
- Behind a DMA Remapping hardware unit (IOMMU)
- Supports PASID for per-process virtual address spaces

### 2.4 Architecture Overview

```
Software → Descriptor (64 bytes) → Portal (MOVDIR64B/ENQCMD) → Work Queue → Engine → Completion Record
```

Key components:
- **Work Queues (WQ):** On-device storage for submitted descriptors
  - **Dedicated WQ (DWQ):** Single owner, submitted via MOVDIR64B (64-byte non-torn write)
  - **Shared WQ (SWQ):** Multiple submitters, submitted via ENQCMD/ENQCMDS (Deferrable Memory Write — returns success/retry)
- **Engines:** Process descriptors dispatched from WQs
- **Groups:** Associate WQs with engines for QoS isolation
- **Portals (BAR2):** 64-byte submission windows, one per 4KB page, 4 per WQ

### 2.5 Descriptor Format (64 bytes)

```
Offset  Size   Field
------  ----   -----
0       4      Trusted Fields (PASID[19:0], Priv[bit 30], Reserved)
4       3      Flags (24 bits — see below)
7       1      Operation (opcode byte)
8       8      Completion Record Address (32-byte aligned)
16      8      Source Address
24      8      Destination Address
32      4      Transfer Size / Element Count
36      2      Completion Interrupt Handle
38      2      Reserved / Operation-specific
40      8      Operation-specific field 1
48      8      Operation-specific field 2
56      8      Operation-specific field 3
```

### 2.6 Key Operation Opcodes

```
0x00  No-op                    0x10  CRC Generation
0x01  Batch                    0x11  Copy with CRC Generation
0x02  Drain                    0x12  DIF Check
0x03  Memory Move              0x13  DIF Insert
0x04  Fill                     0x14  DIF Strip
0x05  Compare                  0x15  DIF Update
0x06  Compare Pattern          0x17  DIX Generate
0x07  Create Delta Record      0x18  Type Conversion
0x08  Apply Delta Record       0x19  Reduce
0x09  Memory Copy w/ Dualcast  0x1A  Reduce with Dualcast
0x0A  Translation Fetch        0x1B  Gather Reduce
0x20  Cache Flush              0x1C  Gather Copy
0x21  Update Window            0x1D  Scatter Copy
0x23  Inter-Domain Copy        0x1E  Scatter Fill
```

### 2.7 Flags Field (24 bits)

```
Bit  Description
---  -----------
0    Block On Fault (1 = wait for page fault resolution)
1    Completion Record Address Valid
2    Request Completion Record
3    Request Completion Interrupt
4    Fence (ordering within batch)
5    Reserved
6    Check Result (affects Status based on operation result)
7    Expected Result (for Check Result)
8    Cache Control 1
9    Address 1 TC Selector
10   Address 2 TC Selector
11   Address 3 TC Selector
12   Completion Record TC Selector
13   Strict Ordering (force write ordering)
14   Cache Control 2
15   Reserved
16   Cache Control 3 / Operation-Specific
17+  Operation-Specific Flags
```

### 2.8 Completion Record (32 bytes)

```
Offset  Size  Field
------  ----  -----
0       1     Status (never written as 0; init to 0 to detect completion)
1       1     Result (operation-specific)
2       1     Fault Info (operand identifier, fault address masked)
3       1     Reserved
4       4     Bytes Completed (for partial completion on page fault)
8       8     Fault Address
16      8     Operation-specific (Invalid Flags / Delta Record Size / etc.)
24      8     Operation-specific / Unused
```

**Status codes:**
- 0x01 = Success
- 0x02 = Success with false predicate
- 0x03 = Partial completion (page fault, non-PRS)
- 0x04 = Partial completion (page fault, PRS)
- 0x05 = Success with predicate
- 0x06 = Partial completion (DIF error)
- 0x10+ = Various errors

### 2.9 Portal Submission

**For DWQ (Dedicated Work Queue):**
```csharp
// MOVDIR64B: 64-byte non-torn write to portal address
// Portal at BAR2 + (wq_index * 4 * 4096) + (portal_index * 4096)
// portal_index 0-3 per WQ
Movdir64b(portalAddress, descriptorAddress);
```

**For SWQ (Shared Work Queue):**
```csharp
// ENQCMD: Deferrable memory write — returns success/retry via ZF
// retry if ZF=0 (WQ full)
while (!Enqcmd(portalAddress, descriptorAddress)) {
    // spin or yield
}
```

### 2.10 Delta Record Format (for WAL)

Each delta is 10 bytes:
```
Bytes 0-1: Offset (16-bit, little-endian, represents multiple of 8 bytes)
Bytes 2-9: 8-byte data block from Source2 that differs from Source1
```

- Maximum Transfer Size: 512 KB (0x80000 bytes) — limited by 16-bit offset
- Maximum Delta Record Size: configurable per descriptor
- Source1/Source2 addresses and Transfer Size must be 8-byte aligned

### 2.11 QoS Controls

- **WQ Priority:** 1-15, relative within a group, weighted round-robin dispatch
- **Traffic Classes:** Two TC values per group (TC-A, TC-B), selectable per address in descriptor
- **Read Buffer Allocation:** Reserved + Allowed per group — controls read bandwidth
- **Bandwidth Control:** Read/Write bandwidth limits per group (fraction of max)
- **Latency Control:** Descriptors in Progress Limit per engine — lower = lower latency, lower throughput

### 2.12 Key Capability Registers (BAR0)

```
Offset  Register        Description
------  --------        -----------
0x000   GENCAP          General Capabilities (max transfer size, max batch size, etc.)
0x008   GENCFG          General Configuration
0x010   GENCTRL         General Control
0x018   GENSTATUS       General Status
0x020   INTCAUSE        Interrupt Cause
0x028   CMD             Command Register (Enable/Disable/Drain/Abort)
0x030   CMDSTATUS       Command Status
0x038   CMDCAP          Command Capabilities
0x040   SWERROR         Software Error Register
0x048   EVLCFG          Event Log Configuration
0x050   EVLSTATUS       Event Log Status
0x060   IDCAP           Inter-Domain Capabilities
0x068   IDBR            Inter-Domain Bitmap
0x080   OPCAP           Operation Capabilities (bitmask of supported operations)
0x180   DSACAP0         DSA Capabilities 0 (SGL formats, inter-domain support)
0x188   DSACAP1         DSA Capabilities 1 (compute ops, data types, FP conversions)
0x190   DSACAP2         DSA Capabilities 2 (rounding, saturation, DAZ, FTZ)
```

Group Configuration Table (per group):
```
GRPCAP   — Group Capabilities
GRPCFG   — Group Configuration (engines, WQs, TC values, read buffers)
GRPFLAGS — Group Flags (bandwidth limits, descriptors in progress limits)
```

WQ Configuration Table (per WQ):
```
WQCFG    — WQ size, mode (dedicated/shared), priority, PASID, max transfer/batch size
```

### 2.13 VectorVolt Integration Design

**New class: `DsaEngine`** (in Phase 1 HAL layer)
- Discover and initialize DSA devices
- Configure WQs: one DWQ per shard writer thread, SWQs for reader threads
- Map portals into process address space

**New class: `DsaWalWriter`** (in Phase 3 WAL layer)
- Use `Copy with CRC` (0x11) for WAL entry writes with inline integrity
- Use `Create Delta Record` (0x07) for delta-based WAL journaling
- Use `Apply Delta Record` (0x08) for recovery/replay
- Batch multiple WAL entries using Batch descriptor (0x01) with Fence flags

**New class: `DsaReplicator`** (in Phase 3/4)
- Use `Dualcast` (0x09) for synchronous write to primary + replica
- Use `Inter-Domain Copy` (0x23) for cross-VM replication

---

## 3. PCLMULQDQ / Carry-Less Multiplication <a name="3-pclmulqdq"></a>

**Source:** `clmul-wp-rev-2-02-2014-04-20.pdf` (84 pages, Rev 2.02)

### 3.1 What It Is

PCLMULQDQ performs carry-less (polynomial) multiplication of two 64-bit operands, producing a 128-bit result. This is the core operation for:
- **GCM (Galois Counter Mode)** — authenticated encryption
- **CRC computation** — for any polynomial (not just iSCSI)
- **Elliptic Curve Cryptography** over binary fields

### 3.2 Instruction Format

```
PCLMULQDQ xmm1, xmm2/m128, imm8

Opcode: 66 0f 3a 44
CPUID detection: Leaf 1, ECX[1] = 1

imm8[0] selects Temp1: 0 = xmm1[63:0], 1 = xmm1[127:64]
imm8[4] selects Temp2: 0 = xmm2[63:0], 1 = xmm2[127:64]

Result: 127-bit carry-less product in xmm1
```

### 3.3 GCM/GHASH Algorithm

The Galois Hash operates in GF(2^128) with irreducible polynomial:
```
P(x) = x^128 + x^7 + x^2 + x + 1
```

GHASH computation:
```
Xi = (Xi-1 XOR Ci) * H    (where * is GF(2^128) multiplication)
```

Each GF multiplication = PCLMULQDQ + reduction modulo P(x).

**Aggregated reduction:** Pipeline multiple PCLMULQDQ operations, defer reduction to the end — achieves near 1 cycle/byte throughput with AES-NI.

### 3.4 CRC Computation

PCLMULQDQ can accelerate CRC with ANY polynomial (not just CRC32C which has a dedicated instruction). For VectorVolt, this means:
- CRC32 (for compatibility) — use dedicated CRC32 instruction
- CRC64 — use PCLMULQDQ
- Custom polynomials for page checksums — use PCLMULQDQ

### 3.5 C#/.NET Integration

```csharp
using System.Runtime.Intrinsics;
using System.Runtime.Intrinsics.X86;

// Check support
if (Pclmulqdq.IsSupported) { ... }

// Carry-less multiply
Vector128<long> a = Vector128.Create(dataLow, 0L);
Vector128<long> b = Vector128.Create(hashKey, 0L);
Vector128<long> result = Pclmulqdq.CarrylessMultiply(a, b, 0x00);

// For AES-GCM, combine with Aes intrinsics:
if (Aes.IsSupported) {
    Vector128<byte> encrypted = Aes.Encrypt(data, roundKey);
}
```

### 3.6 VectorVolt Use Cases

1. **WAL page checksums:** CRC64 via PCLMULQDQ for page integrity
2. **AES-GCM encryption-at-rest:** AES-NI + PCLMULQDQ for authenticated encryption
3. **Network packet checksums:** CRC for kernel-bypass networking
4. **Hash computation:** Polynomial hashing for hash indexes

---

## 4. Processor Topology Enumeration <a name="4-topology"></a>

**Source:** `intel-64-architecture-processor-topology-enumeration.pdf` (30 pages, Rev 2.0)

### 4.1 What It Is

Intel processors expose topology via CPUID. The topology is hierarchical:
```
Package → Die → DieGrp → Tile → Module → Core → SMT (Logical Processor)
```

### 4.2 CPUID Leaf Selection Algorithm

```
if CPUID.0.EAX >= 0x1F:
    Use Leaf 0x1F (V2 Extended Topology — includes Die, Tile, Module)
elif CPUID.0.EAX >= 0x0B:
    Use Leaf 0x0B (V1 Extended Topology — Package, Core, SMT only)
else:
    Use Leaf 0x01 + Leaf 0x04 (Legacy — limited)
```

### 4.3 CPUID Leaf 0x1F — V2 Extended Topology

Enumerate sub-leaves (ECX=0, 1, 2, ...):

```
Input:  EAX = 0x1F, ECX = sub-leaf index
Output:
  EAX[4:0]  = Number of bits to shift right on x2APIC ID to get unique ID at this level
  EBX[15:0] = Number of logical processors at this level
  ECX[7:0]  = Sub-leaf index
  ECX[15:8] = Level Type:
               0 = Invalid (stop enumeration)
               1 = SMT
               2 = Core
               3 = Module
               4 = Tile
               5 = Die
               6 = DieGrp (reserved in some docs)
  EDX[31:0] = x2APIC ID of current logical processor
```

### 4.4 APIC ID Bit Field Decomposition

```
x2APIC ID = | PACKAGE_ID | DIE_ID | TILE_ID | MODULE_ID | CORE_ID | SMT_ID |
             ← high bits                                    low bits →

Shift amounts from CPUID.1F sub-leaves give boundary positions.
```

**Algorithm to extract IDs:**
```
SMT_ID     = x2APIC_ID & ((1 << shift_smt) - 1)
CORE_ID    = (x2APIC_ID >> shift_smt) & ((1 << (shift_core - shift_smt)) - 1)
MODULE_ID  = (x2APIC_ID >> shift_core) & ((1 << (shift_module - shift_core)) - 1)
TILE_ID    = (x2APIC_ID >> shift_module) & ((1 << (shift_tile - shift_module)) - 1)
DIE_ID     = (x2APIC_ID >> shift_tile) & ((1 << (shift_die - shift_tile)) - 1)
PACKAGE_ID = x2APIC_ID >> shift_die
```

### 4.5 Cache Topology via CPUID Leaf 0x04

```
Input:  EAX = 0x04, ECX = cache level index (0, 1, 2, ...)
Output:
  EAX[4:0]   = Cache type (1=Data, 2=Instruction, 3=Unified)
  EAX[7:5]   = Cache level (1, 2, 3)
  EAX[25:14] = Max logical processors sharing this cache - 1
  EAX[31:26] = Max cores sharing this cache - 1
  EBX          = Line size, partitions, associativity
  ECX          = Number of sets - 1
  EDX          = Inclusiveness, complex indexing
```

### 4.6 VectorVolt HAL Implementation

**Required class: `CpuTopology`**

```csharp
public sealed class CpuTopology {
    // Per-logical-processor info
    public record LogicalProcessor(
        uint ApicId, uint PackageId, uint DieId, uint TileId,
        uint ModuleId, uint CoreId, uint SmtId);

    // Cache sharing info
    public record CacheLevel(
        int Level, CacheType Type, int LineSize, int TotalSize,
        int Associativity, uint[] SharingProcessors);

    // Core affinity for shard pinning
    public LogicalProcessor[] EnumerateTopology();
    public CacheLevel[] EnumerateCaches();

    // Find isolated core for single-writer shard
    public uint FindIsolatedCore(uint avoidCoresSharing L2With);

    // Check if two logical processors share L3
    public bool SharesL3(uint apicId1, uint apicId2);
}
```

**Key decisions for VectorVolt:**
- Pin each shard writer to a **physical core** (not HT sibling)
- Place reader threads on **same die** as shard writer (shared L3)
- Avoid placing two hot shards on cores sharing L2
- Use Die-level affinity for NUMA memory allocation

---

## 5. MKTME (Multi-Key Total Memory Encryption) <a name="5-mktme"></a>

**Source:** `336907-006-multi-key-total-memory-encryption-spec.pdf` (24 pages, Rev 1.6, April 2025)

### 5.1 What It Is

TME (Total Memory Encryption) encrypts ALL physical memory with a single ephemeral AES-XTS key generated by the CPU. TME-MK (Multi-Key) adds **page-granular encryption** with up to 32K different keys. Each memory page can be encrypted with a different key by encoding a KeyID in the upper bits of the physical address.

### 5.2 Hardware Details

**CPUID Detection:**
```
CPUID.(EAX=7, ECX=0):ECX[13] = 1  → TME supported
```

**MSR: IA32_TME_CAPABILITY (0x981)** — Read-only
```
Bit 0     = AES-XTS 128-bit supported
Bit 1     = AES-XTS 128-bit with SHA-3 integrity supported
Bit 2     = AES-XTS 256-bit supported
Bit 3     = AES-XTS 256-bit with SHA-3 integrity supported
Bits 30:4 = Reserved
Bit 31    = TME encryption bypass supported
Bits 35:32 = MK_TME_MAX_KEYID_BITS (max 15 → 32K keys)
Bits 50:36 = MK_TME_MAX_KEYS (max 32K-1, KeyID 0 reserved for TME)
Bits 63:51 = Reserved
```

**MSR: IA32_TME_ACTIVATE (0x982)** — Read/Write (locked after first write)
```
Bit 0     = Lock (RO, set on successful WRMSR)
Bit 1     = Hardware Encryption Enable (enables TME and TME-MK)
Bit 2     = Key Select (0=new key, 1=restore from standby)
Bit 3     = Save TME key for Standby
Bits 7:4  = TME Policy/Encryption Algorithm (must match capability)
Bits 30:8 = Reserved
Bit 31    = TME Encryption Bypass Enable (0=encrypt KeyID0, 1=bypass)
Bits 35:32 = MK_TME_KEYID_BITS (number of KeyID bits to allocate)
Bits 47:36 = Reserved
Bits 63:48 = Reserved
```

### 5.3 How KeyIDs Work

Physical addresses are modified to include a KeyID in the upper bits:
```
Physical Address = | KeyID (N bits) | Actual Physical Address (PA_WIDTH - N bits) |
```

- KeyID 0 = TME default key (CPU-generated ephemeral)
- KeyID 1..MAX = TME-MK keys (software-provisioned or CPU-generated)
- KeyID is set in page table entries (PTE bits above the physical address)

### 5.4 Key Programming via PCONFIG

```
PCONFIG instruction with leaf MKTME_KEY_PROGRAM:
  - Programs a key for a specific KeyID
  - Input: KeyID, key type (create new, wrap with CPU key, provide directly)
  - Key data structure in memory (encrypted key material)
```

### 5.5 VectorVolt Multi-Tenancy Design

```
Tenant A → KeyID 1 → All pages for Tenant A's shards encrypted with Key 1
Tenant B → KeyID 2 → All pages for Tenant B's shards encrypted with Key 2
...
Tenant N → KeyID N → Hardware isolation, zero CPU overhead
```

**Even if a bug leaks memory across tenants, the data is encrypted with a different key.**

### 5.6 MSR: IA32_TME_EXCLUDE_MASK (0x983) and IA32_TME_EXCLUDE_BASE (0x984)

Allow excluding a physical address range from TME encryption (for MMIO, firmware, etc.).

---

## 6. 5-Level Paging (LA57) <a name="6-5level-paging"></a>

**Source:** `5-level-paging-white-paper.pdf` (31 pages, Rev 1.1)

### 6.1 What It Is

4-level paging: 48-bit linear addresses → 256 TB virtual address space
5-level paging: 57-bit linear addresses → 128 PB virtual address space

5-level adds a PML5 table above PML4, using 9 additional address bits.

### 6.2 Detection and Enabling

```
CPUID.(EAX=7, ECX=0):ECX[16] = 1  → LA57 (5-level paging) supported
CR4.LA57 (bit 12) = 1  → Enable 5-level paging
```

### 6.3 Address Decomposition

**4-level (current):**
```
63:48  Sign extension (canonical check)
47:39  PML4 index (9 bits) → PML4 Table
38:30  PDPT index (9 bits) → Page Directory Pointer Table
29:21  PD index (9 bits)   → Page Directory
20:12  PT index (9 bits)   → Page Table
11:0   Page offset (12 bits)
```

**5-level (new):**
```
63:57  Sign extension (canonical check)
56:48  PML5 index (9 bits) → PML5 Table (NEW)
47:39  PML4 index (9 bits) → PML4 Table
38:30  PDPT index (9 bits) → PDPT
29:21  PD index (9 bits)   → PD
20:12  PT index (9 bits)   → PT
11:0   Page offset (12 bits)
```

### 6.4 Canonicality

- 4-level: bits 63:48 must match bit 47 (sign extension)
- 5-level: bits 63:57 must match bit 56 (sign extension)

### 6.5 VectorVolt Relevance

- Design virtual address space layout to be 57-bit aware from day one
- Memory-mapped data files can address 128 PB instead of 256 TB
- Structure pointers, offsets, and address calculations to work with both 4-level and 5-level
- Detect at runtime via CPUID and adapt

---

## 7. FRED (Flexible Return and Event Delivery) <a name="7-fred"></a>

**Source:** `346446-009-flexible-return-and-event-delivery.pdf` (62 pages, Rev 9.0, June 2025)

### 7.1 What It Is

FRED replaces the legacy IDT-based interrupt/exception delivery with a deterministic, MSR-configured mechanism. It eliminates TSS-based stack switching and provides separate entry points for ring-0 and ring-3 events.

### 7.2 Detection and Enabling

```
CPUID.(EAX=7, ECX=1):EAX[17] = 1  → FRED supported
CPUID.(EAX=7, ECX=1):EAX[18] = 1  → LKGS supported
CPUID.(EAX=7, ECX=1):EAX[20] = 1  → NMI-source reporting supported
CR4.FRED (bit 32) = 1  → Enable FRED transitions
```

### 7.3 New MSRs

```
MSR 0x1D4  IA32_FRED_CONFIG
  Bits 1:0   = Current Stack Level (CSL)
  Bit 3      = Decrement SSP when not changing stacks
  Bits 8:6   = RSP decrement amount (in 64-byte cache lines) when not changing stacks
  Bits 10:9  = Stack level for maskable interrupts in ring 0
  Bits 63:12 = Upper bits of event delivery entry point (4KB-aligned)

MSR 0x1CC  IA32_FRED_RSP0  (Ring-0 stack pointer for stack level 0)
MSR 0x1CD  IA32_FRED_RSP1  (Ring-0 stack pointer for stack level 1)
MSR 0x1CE  IA32_FRED_RSP2  (Ring-0 stack pointer for stack level 2)
MSR 0x1CF  IA32_FRED_RSP3  (Ring-0 stack pointer for stack level 3)

MSR 0x1D0  IA32_FRED_STKLVLS  (Stack level assignment for each event vector)

MSR 0x1D1  IA32_FRED_SSP0-3   (Shadow stack pointers per level)
```

### 7.4 Event Delivery

Events categorized into 4 types × 256 vectors:
- **Type 0:** External interrupts (vectors 0-255)
- **Type 1:** NMI, SMI
- **Type 2:** Exceptions (#DE, #DB, #BP, #OF, #UD, #NM, #DF, #GP, #PF, #MF, #AC, #MC, #XM, #VE, #CP, #HV, #VC, #SX)
- **Type 3:** SYSCALL, SYSENTER

Entry points for ring-3 events at base address; ring-0 events at base + 256 bytes.

### 7.5 New Return Instructions

- **ERETS** — Event Return to Supervisor (ring 0 → ring 0)
- **ERETU** — Event Return to User (ring 0 → ring 3, updates GS base like SWAPGS)

### 7.6 VectorVolt Relevance

- FRED reduces interrupt delivery latency — important for Phase 4 kernel bypass
- Deterministic stack switching = no surprise stack faults during hot path
- LKGS instruction simplifies GS segment management for per-CPU data
- When running on FRED-enabled kernels, VectorVolt's interrupt coalescing gets more predictable timing

---

## 8. Speculative Execution Side-Channel Mitigations <a name="8-spectre"></a>

**Source:** `336996-speculative-execution-side-channel-mitigations.pdf` (23 pages, Rev 3.0)

### 8.1 Mitigation Mechanisms

| Mechanism | MSR | Bit | Purpose | Performance Cost |
|-----------|-----|-----|---------|-----------------|
| IBRS | IA32_SPEC_CTRL (0x48) | 0 | Restrict indirect branch speculation | Moderate |
| Enhanced IBRS | IA32_ARCH_CAPABILITIES (0x10A) | 1 | Always-on IBRS, lower overhead | Low |
| STIBP | IA32_SPEC_CTRL (0x48) | 1 | Single-thread indirect branch predictors | Low-Moderate |
| SSBD | IA32_SPEC_CTRL (0x48) | 2 | Speculative Store Bypass Disable | Moderate |
| IBPB | IA32_PRED_CMD (0x49) | 0 | Indirect Branch Predictor Barrier | One-shot |
| L1D_FLUSH | IA32_FLUSH_CMD (0x10B) | 0 | Flush L1 Data Cache | High (one-shot) |

### 8.2 CPUID Detection

```
CPUID.(EAX=7, ECX=0):EDX[26] = 1  → IBRS/IBPB supported
CPUID.(EAX=7, ECX=0):EDX[27] = 1  → STIBP supported
CPUID.(EAX=7, ECX=0):EDX[29] = 1  → IA32_ARCH_CAPABILITIES present
CPUID.(EAX=7, ECX=0):EDX[31] = 1  → SSBD supported
```

### 8.3 IA32_ARCH_CAPABILITIES MSR (0x10A)

```
Bit 0  = RDCL_NO (not susceptible to Meltdown)
Bit 1  = IBRS_ALL (Enhanced IBRS — always active, low overhead)
Bit 2  = RSBA (Return Stack Buffer Alternate predictions possible)
Bit 3  = SKIP_L1DFL_VMENTRY (no L1D flush needed on VM entry)
Bit 4  = SSB_NO (not susceptible to Speculative Store Bypass)
Bit 5  = MDS_NO (not susceptible to Microarchitectural Data Sampling)
Bit 6  = IF_PSCHANGE_MC_NO (page size change doesn't cause MC)
Bit 7  = TSX_CTRL (TSX can be controlled)
Bit 8  = TAA_NO (not susceptible to TSX Async Abort)
```

### 8.4 VectorVolt Design Implications

1. **Detect Enhanced IBRS (bit 1 of IA32_ARCH_CAPABILITIES):** If present, IBRS overhead is minimal — no need to toggle per context switch
2. **Multi-tenancy (Phase 6):** Need L1D_FLUSH between tenant context switches unless RDCL_NO is set
3. **Performance budget:** Measure and account for mitigation overhead in latency budgets
4. **HyperThreading:** STIBP needed if two tenants share a physical core (VectorVolt should avoid this via topology-aware scheduling)

---

## 9. IAA (In-Memory Analytics Accelerator) <a name="9-iaa"></a>

**Source:** `ZSWAPwithIAA_1.0.pdf` (10 pages)

### 9.1 What It Is

Intel IAA is a hardware accelerator for compression/decompression (deflate algorithm). The zswap paper demonstrates:
- **Swap-out (compression) latency:** 1.38x-2.53x faster than lz4, 2.87x-6.37x faster than zstd
- **Swap-in (decompression) latency:** 2.13x-2.26x faster than zstd
- **With batching (8 engines):** 3.37x-7.42x faster than lz4/zstd for swap-out
- **With 64KB folios + batching:** 4.17x-10.25x faster for swap-out

### 9.2 Compression Modes

| Mode | Description |
|------|-------------|
| `deflate-iaa` | Default IAA compression mode |
| `deflate-iaa-canned` | Pre-built Huffman tables, faster but lower ratio |
| `deflate-iaa-dynamic` | Dynamic Huffman tables, slower but better ratio |

### 9.3 VectorVolt Cold-Data Tiering

For Phase 5 (Read Fabric), VectorVolt could implement a buffer pool with hot/warm/cold tiers:
- **Hot:** Uncompressed in DRAM, served at full speed
- **Warm:** IAA-compressed in DRAM, hardware decompression on access
- **Cold:** Evicted to NVMe

IAA compression of cold pages saves DRAM without the CPU cost of software compression.

---

## 10. Intel Scalable I/O Virtualization <a name="10-scalable-iov"></a>

**Source:** `intel-scalable-io-virtualization-technical-specification.pdf` (29 pages, Rev 1.1)

### 10.1 What It Is

Scalable IOV goes beyond SR-IOV by introducing:
- **ADI (Assignable Device Interface):** Smallest unit of device assignment — can be a single work queue
- **VDCM (Virtual Device Composition Module):** Software component that composes virtual devices from ADIs
- **Software-composable:** VMM/host driver dynamically maps ADIs to VMs

### 10.2 Key Differences from SR-IOV

| Aspect | SR-IOV | Scalable IOV |
|--------|--------|-------------|
| Granularity | Virtual Function (VF) | ADI (work queue level) |
| Scalability | ~256 VFs typical | 1000s of ADIs |
| Configuration | Hardware-defined | Software-composable |
| Hardware cost | VF requires full config space | ADI is lightweight |
| Dynamic | Static allocation | Dynamic mapping |

### 10.3 VectorVolt Relevance (Phase 4)

For kernel-bypass networking in cloud/VM environments:
- Each VectorVolt worker thread gets its own ADI (NIC queue)
- Direct submission without VMM intervention
- PASID-based isolation between tenants
- Works with DSA for direct hardware DMA between NIC and DSA

---

## 11. BFloat16 Numerics <a name="11-bf16"></a>

**Source:** `bf16-hardware-numerics-definition-white-paper.pdf` (7 pages, Rev 1.0)

### 11.1 Format

```
BF16: 1 sign + 8 exponent + 7 mantissa = 16 bits
FP16: 1 sign + 5 exponent + 10 mantissa = 16 bits
FP32: 1 sign + 8 exponent + 23 mantissa = 32 bits
```

BF16 is a truncated FP32 — same exponent range, less precision. No denormal support needed.

### 11.2 FMA Unit

- Two BF16 inputs multiplied exactly (no precision loss in multiply)
- Result accumulated into FP32 accumulator
- DAZ=On, FTZ=On, RNE rounding (all hardwired)

### 11.3 VectorVolt Relevance (Phase 5)

- Vectorized aggregate computations (SUM, AVG) on numeric columns
- DSA Rev 3.0 supports BF16 as a data type for Type Conversion and Reduce operations
- Can compute reductions in BF16 on DSA hardware, zero CPU

---

## 12. PMU Sharing Guidelines <a name="12-pmu"></a>

**Source:** `pmu-sharing-guidelines.pdf` (13 pages)

### 12.1 Key Points for VectorVolt

**CPUID detection:**
```
CPUID.0AH:EAX[7:0]  = PerfMon version (>0 = architectural PerfMon supported)
CPUID.0AH:EAX[15:8] = Number of programmable counters per LP
CPUID.0AH:EBX       = Unavailable events bitmask
CPUID.0AH:EDX[4:0]  = Number of fixed-function counters per thread
```

**Sharing rules:**
- Check if counter is "in-use" before programming (IA32_PERFEVTSELx event field != 0)
- Relinquish counters on exit (zero the control register)
- Don't use "Freeze PerfMon on PMI" if other agents use counters
- Use software freeze (read-modify-write IA32_PERF_GLOBAL_CTRL) instead

### 12.2 VectorVolt Observability

Use PMU counters for:
- L1/L2/L3 cache miss rates per shard
- Branch misprediction rate on query processing hot path
- Instructions per cycle (IPC) for performance regression detection
- Memory bandwidth utilization

---

## 13. MCA (Machine Check Architecture) Enhancements <a name="13-mca"></a>

**Source:** `329176-mca-enhancements-in-intel-xeon-processors.pdf` (14 pages, Rev 1.1)

### 13.1 What It Is

Enhanced MCA Logging provides additional error information beyond basic machine check bank registers. Key use: **Predictive Failure Analysis (PFA)** for memory.

### 13.2 Detection

```
IA32_MCG_CAP MSR bit 26 (MCG_ELOG_P) = 1  → Enhanced MCA Logging supported
ACPI DSM GUID: 663E35AF-CC10-41a4-88EA-5470AF055295  → Platform support
```

### 13.3 VectorVolt Relevance

- Monitor correctable memory errors (CE) on DIMM serving hot shards
- Proactively migrate shards away from failing DIMMs before uncorrectable errors (UE)
- Enhanced MCA logs provide DIMM-level granularity for error attribution

---

## 14. Intel Key Locker <a name="14-key-locker"></a>

**Source:** `343965-intel-key-locker-specification.pdf` (65 pages)

### 14.1 What It Is

Key Locker wraps AES keys into opaque "key handles" so the actual key material never exists in software-accessible memory.

### 14.2 Instructions

```
LOADIWKEY      — Load Internal Wrapping Key
ENCODEKEY128   — Encode AES-128 key into handle
ENCODEKEY256   — Encode AES-256 key into handle
AESENC128KL    — AES encrypt with 128-bit key handle
AESDEC128KL    — AES decrypt with 128-bit key handle
AESENC256KL    — AES encrypt with 256-bit key handle
AESDEC256KL    — AES decrypt with 256-bit key handle
AESENCWIDE128KL — Wide (8-block) AES encrypt with 128-bit key handle
AESDECWIDE128KL — Wide (8-block) AES decrypt with 128-bit key handle
AESENCWIDE256KL — Wide (8-block) AES encrypt with 256-bit key handle
AESDECWIDE256KL — Wide (8-block) AES decrypt with 256-bit key handle
```

### 14.3 CPUID Detection

```
CPUID.(EAX=7, ECX=0):ECX[23] = 1  → Key Locker supported
CPUID.(EAX=19H):EAX            → Key Locker details
```

### 14.4 VectorVolt Relevance

- Encryption-at-rest with key material never exposed in memory
- Even if an attacker dumps VectorVolt's process memory, they can't extract the AES key
- Combine with MKTME: MKTME for memory encryption, Key Locker for data-at-rest encryption

---

## 15. Runtime Microcode Update <a name="15-microcode"></a>

**Source:** `356111-003-runtime-update-tech-paper.pdf` (37 pages, Rev 1.2, June 2025)

### 15.1 Key Points

- Microcode updates can change CPUID bits and MSR definitions at runtime
- **IA32_MCU_ENUMERATION (0x7B):** Enumerates update capabilities
  - Bit 0: UNIFORM_MCU_AVAIL (uniform update scope available)
  - Bits 15:8: UNIFORM_MCU_SCOPE (core/module/tile/die/package/platform)
- **IA32_MCU_STATUS (0x7C):** Status of last update attempt
- Runtime updates require coordination across all logical processors

### 15.2 VectorVolt Relevance

- Be aware that CPU capabilities can change during runtime
- Cache CPUID results at startup but re-check after microcode update events
- Minimum Runtime Update Revision field in MCU header ensures compatibility

---

## 16. CPUID Passthrough / AVX10 / AMX <a name="16-cpuid-passthrough"></a>

**Source:** `356709-003-intel-cpuid-passthrough-virtualization-considerations-tech-paper.pdf` (10 pages, Rev 3.0)

### 16.1 Key Points

- VMMs can safely pass through AVX10 version numbers without understanding each feature
- AMX (Advanced Matrix Extensions) CPUID subleaf 1 can be passed through
- Required states for AVX10: XCR0 + IA32_XSS state components must be enabled

### 16.2 VectorVolt Relevance

- When VectorVolt runs in VMs, ensure the VMM passes through required CPUID leaves
- Document minimum VMM requirements for DSA, MKTME, and topology features
- AVX10 convergence means SIMD code works uniformly across Intel architectures

---

## 17. Phase-by-Phase Integration Matrix <a name="17-matrix"></a>

### Phase 1 — Silicon/HAL

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| **Topology Enumeration** (CPUID 0x1F) | `CpuTopology` | P0 — Foundation |
| **5-Level Paging** (CPUID/CR4.LA57) | `VirtualMemoryConfig` | P1 — Detect and adapt |
| **CPUID Feature Detection** | `SiliconCapabilities` | P0 — Foundation |
| **DSA Device Discovery** | `DsaDevice` | P1 — Enumerate |
| **PMU Setup** | `PerformanceCounters` | P2 — Observability |

### Phase 2 — Single-Writer Engine

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| **Spectre Mitigations** | `SecurityMitigations` | P1 — Know the overhead |
| **Topology-Aware Pinning** | `ShardAffinityManager` | P0 — Core pinning |
| **DSA Fill** (0x04) | `PageAllocator` | P2 — Zero-fill optimization |

### Phase 3 — WAL/Durability

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| **DSA Copy+CRC** (0x11) | `DsaWalWriter` | P0 — Core WAL path |
| **DSA Delta Records** (0x07/0x08) | `DsaDeltaJournal` | P0 — Write amplification |
| **DSA Dualcast** (0x09) | `DsaReplicator` | P1 — Sync replication |
| **PCLMULQDQ CRC** | `HardwareCrc` | P0 — Checksum |
| **DSA DIF** (0x12-0x17) | `DataIntegrityManager` | P1 — End-to-end integrity |

### Phase 4 — Kernel Bypass Networking

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| **FRED** | Runtime detection | P2 — Awareness |
| **Scalable IOV** | `NetworkQueueManager` | P1 — Direct NIC access |
| **DSA Inter-Domain** (0x23-0x26) | `CrossVmDataMover` | P2 — VM-to-VM copy |

### Phase 5 — Read Fabric/SIMD

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| **IAA Compression** | `ColdDataCompressor` | P1 — Memory tiering |
| **BFloat16 + DSA Reduce** | `VectorizedAggregator` | P2 — Analytics |
| **DSA Scatter-Gather** (0x1B-0x1E) | `ScatterGatherEngine` | P1 — Embedding ops |
| **DSA Type Conversion** (0x18) | `TypeConverter` | P2 — Format conversion |

### Phase 6 — Multi-Tenancy

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| **MKTME** | `TenantMemoryEncryption` | P0 — Core isolation |
| **Key Locker** | `KeyHandleManager` | P1 — Key protection |
| **Spectre L1D_FLUSH** | `TenantContextSwitch` | P1 — Side-channel |
| **MCA/PFA** | `MemoryHealthMonitor` | P2 — Reliability |

---

## 18. New Classes / Interfaces to Add <a name="18-new-classes"></a>

Based on this research, VectorVolt needs the following new classes/interfaces added to the architecture:

### HAL Layer (Phase 1)
```
SiliconCapabilities         — CPUID feature detection for all Intel technologies
CpuTopology                 — CPUID 0x1F topology enumeration
DsaDevice                   — DSA PCI device discovery and configuration
DsaWorkQueue                — Work queue management (DWQ/SWQ)
DsaDescriptor               — 64-byte descriptor builder (all operation types)
DsaCompletionRecord         — Completion record parser
DsaPortal                   — Portal mapping and submission (MOVDIR64B/ENQCMD)
PerformanceCounters          — PMU counter management
VirtualMemoryConfig          — 4-level vs 5-level paging detection
```

### Engine Layer (Phase 2)
```
ShardAffinityManager        — Topology-aware core pinning
SecurityMitigations          — Spectre detection and overhead accounting
```

### WAL Layer (Phase 3)
```
DsaWalWriter                — DSA Copy+CRC for WAL entries
DsaDeltaJournal             — DSA delta record creation/application
DsaReplicator               — DSA dualcast for synchronous replication
HardwareCrc                 — PCLMULQDQ-based CRC64
DataIntegrityManager        — DSA DIF/DIX for end-to-end integrity
AesGcmEncryptor             — AES-NI + PCLMULQDQ for encryption-at-rest
```

### Network Layer (Phase 4)
```
NetworkQueueManager          — Scalable IOV ADI management
```

### Read Fabric (Phase 5)
```
ColdDataCompressor          — IAA-accelerated compression
VectorizedAggregator        — DSA Reduce + BF16 for analytics
ScatterGatherEngine         — DSA scatter-gather for embeddings
```

### Multi-Tenancy (Phase 6)
```
TenantMemoryEncryption      — MKTME KeyID management
KeyHandleManager            — Key Locker handle lifecycle
TenantContextSwitch         — L1D_FLUSH + IBPB between tenants
MemoryHealthMonitor         — MCA-based predictive failure analysis
```

---

## 19. C#/.NET Integration Patterns <a name="19-csharp-patterns"></a>

### 19.1 Intrinsics Access (.NET 10)

```csharp
// Available in System.Runtime.Intrinsics.X86
Sse2.IsSupported          // SSE2
Sse42.IsSupported         // CRC32C instruction
Aes.IsSupported           // AES-NI
Pclmulqdq.IsSupported    // Carry-less multiply
Avx2.IsSupported          // AVX2
Avx512F.IsSupported       // AVX-512

// CRC32C (built-in instruction)
uint crc = Sse42.Crc32(seed, data);

// PCLMULQDQ
var result = Pclmulqdq.CarrylessMultiply(a, b, 0x00);

// AES
var encrypted = Aes.Encrypt(block, roundKey);
```

### 19.2 DSA Access from User Space

DSA requires kernel driver support but can be accessed from user space via:

**Option A: Linux IDXD driver** (via /dev/dsa/wqX.Y)
```csharp
// Memory-map the portal
var portal = MemoryMappedFile.CreateFromFile("/dev/dsa/wq0.0", ...);
var portalSpan = portal.CreateViewAccessor();

// Build descriptor in pinned memory
var desc = new DsaDescriptor {
    Operation = 0x11, // Copy with CRC
    Flags = 0x07,     // Block on fault + CR valid + Request CR
    SourceAddress = srcPtr,
    DestinationAddress = dstPtr,
    TransferSize = length,
    CompletionRecordAddress = crPtr,
};

// Submit via MOVDIR64B (DWQ) or ENQCMD (SWQ)
// Requires platform invoke to assembly stub
SubmitDescriptor(portalAddress, &desc);
```

**Option B: Custom kernel module** with ioctl interface

**Option C: DSA library** (libaccel-config on Linux)

### 19.3 MOVDIR64B / ENQCMD from C#

These are new x86 instructions that .NET doesn't expose as intrinsics yet. Requires:
```csharp
// Option 1: P/Invoke to a small native shim
[DllImport("vectorvolt_hal.so")]
static extern int dsa_submit_movdir64b(IntPtr portal, IntPtr descriptor);

[DllImport("vectorvolt_hal.so")]
static extern int dsa_submit_enqcmd(IntPtr portal, IntPtr descriptor);

// Option 2: NativeAOT with inline assembly
// Option 3: Use idxd user-space library
```

### 19.4 MSR Access (Kernel Level)

For MKTME, PMU, and MCA, reading MSRs requires kernel privilege:
```csharp
// Via /dev/msr on Linux
[DllImport("vectorvolt_hal.so")]
static extern long read_msr(int cpu, uint msrAddress);

[DllImport("vectorvolt_hal.so")]
static extern void write_msr(int cpu, uint msrAddress, long value);
```

### 19.5 CPUID from C#

```csharp
// .NET provides X86Base.CpuId
(int eax, int ebx, int ecx, int edx) = X86Base.CpuId(leaf, subleaf);

// Example: Topology enumeration
var (eax, ebx, ecx, edx) = X86Base.CpuId(0x1F, 0); // V2 topology, sub-leaf 0
int shiftAmount = eax & 0x1F;
int levelType = (ecx >> 8) & 0xFF;
int logicalProcessors = ebx & 0xFFFF;
uint apicId = (uint)edx;
```

---

## Source PDFs Reference

| File | Document | Pages | Revision |
|------|----------|-------|----------|
| 253669-091-sdm-vol-3b.pdf | Intel SDM Vol 3B — System Programming Guide Part 2 | 488 | March 2026 |
| 329176-mca-enhancements.pdf | MCA Enhancements in Xeon | 14 | Rev 1.1, Jan 2018 |
| 336907-006-multi-key-total-memory-encryption-spec.pdf | TME/TME-MK Architecture | 24 | Rev 1.6, Apr 2025 |
| 336996-speculative-execution-side-channel-mitigations.pdf | Spectre/Meltdown Mitigations | 23 | Rev 3.0, Jul 2018 |
| 341204-006-intel-data-streaming-accelerator-spec.pdf | Intel DSA Architecture | 288 | Rev 3.0, Jun 2025 |
| 343965-intel-key-locker-specification.pdf | Intel Key Locker | 65 | Sep 2020 |
| 346446-009-flexible-return-and-event-delivery.pdf | FRED Specification | 62 | Rev 9.0, Jun 2025 |
| 356111-003-runtime-update-tech-paper.pdf | Runtime Microcode Update | 37 | Rev 1.2, Jun 2025 |
| 356709-003-intel-cpuid-passthrough.pdf | CPUID Passthrough / AVX10 / AMX | 10 | Rev 3.0, Aug 2025 |
| 5-level-paging-white-paper.pdf | 5-Level Paging and 5-Level EPT | 31 | Rev 1.1, May 2017 |
| ZSWAPwithIAA_1.0.pdf | Memory Tiering with IAA | 10 | v1.0 |
| bf16-hardware-numerics-definition-white-paper.pdf | BFloat16 Hardware Numerics | 7 | Rev 1.0, Nov 2018 |
| clmul-wp-rev-2-02-2014-04-20.pdf | PCLMULQDQ and GCM Mode | 84 | Rev 2.02, Apr 2014 |
| intel-64-architecture-processor-topology-enumeration.pdf | Processor Topology Enumeration | 30 | Rev 2.0, Apr 2023 |
| intel-scalable-io-virtualization-technical-specification.pdf | Scalable I/O Virtualization | 29 | Rev 1.1, Sep 2020 |
| pmu-sharing-guidelines.pdf | PMU Sharing Guide | 13 | - |

---

---

# PART 2: BEYOND-CPU HARDWARE OPTIMIZATIONS (Intel Platform)

> **Added:** 2026-04-14 (Session 2). Covers NVMe storage, networking/NIC, and platform/BIOS tuning — all Intel-specific or vendor-neutral technologies that complement the CPU-silicon research above.

---

## 20. NVMe Storage Optimizations <a name="20-nvme"></a>

### 20.1 NVMe CMB (Controller Memory Buffer)

**What it is:** On-SSD DRAM exposed to host via PCIe BAR. Host can place NVMe submission queues in CMB, eliminating one PCIe DMA read per I/O command.

**Detection registers (BAR0):**
```
Offset 0x58  CMBSZ (CMB Size) — non-zero = CMB supported
  Bit 0 (SQS):  Submission Queues supported in CMB
  Bit 1 (CQS):  Completion Queues supported
  Bit 2 (LISTS): PRP/SGL Lists supported
  Bit 3 (RDS):  Read Data supported
  Bit 4 (WDS):  Write Data supported

Offset 0x5C  CMBLOC (CMB Location)
  Bits 2:0  = BIR (BAR Indicator Register — which PCIe BAR)
  Bits 31:12 = Offset within BAR (4KB units)

Offset 0x50  CMBMSC (CMB Memory Space Control, 8 bytes) — NVMe 1.4+
```

**User-space access:**
```csharp
// Memory-map the CMB BAR (e.g., BAR2)
// Linux: mmap /sys/bus/pci/devices/<BDF>/resource2
// SPDK: spdk_nvme_ctrlr_map_cmb()
// Place SQ entries directly in mapped region — zero DMA for queue ops
```

**Performance:** Saves ~0.5-1.5μs per I/O; 5-15% IOPS improvement for small random I/O.
**CMB sizes:** Typically 256KB-64MB (enough for queues, not bulk data).
**Products:** Samsung PM1733/PM1735 (enterprise, firmware-dependent). QEMU can emulate for dev.

### 20.2 NVMe PMR (Persistent Memory Region) ⭐

**What it is:** Battery/supercap-backed DRAM on SSD, memory-mapped by host. Byte-addressable persistent storage at PCIe BAR write speed (~200ns).

**Detection registers (BAR0):**
```
Offset 0xE00  PMRCAP (PMR Capabilities)
  Bit 0 (RDS):   Read Data Support
  Bit 1 (WDS):   Write Data Support
  Bits 7:4:      PMRWBM (Write Barrier Mechanisms)
  Bits 11:8:     PMRTO (Timeout, 500ms units)

Offset 0xE04  PMRCTL (PMR Control)
  Bit 0 (EN):    Enable PMR

Offset 0xE08  PMRSTS (PMR Status)
  Bit 0 (ERR):   Error
  Bit 8 (NRDY):  Not Ready — poll until cleared

Offset 0xE0C  PMREBS (Elasticity Buffer Size)
Offset 0xE10  PMRSWTP (Sustained Write Throughput)
Offset 0xE14  PMRMSCL/0xE18 PMRMSCU (Memory Space Control)
```

**Enable sequence:**
1. Check PMRCAP at 0xE00 — non-zero = supported
2. Write 1 to PMRCTL.EN (0xE04, bit 0)
3. Poll PMRSTS.NRDY (0xE08, bit 8) until cleared
4. mmap the PMR BAR
5. Write data using `mov` + `clwb` + `sfence` (same as persistent memory)

**Performance:** ~200-300ns write latency (PCIe posted write). PMR sizes: 16MB-512MB.
**Products:** Samsung PM1733/PM1735, Kioxia CM6/CD6 (firmware-dependent).

**VectorVolt WAL impact:** PMR reduces WAL commit from ~10μs (NVMe command) to ~200ns (MMIO write). This is the **single most impactful storage technology** for VectorVolt's commit latency.

### 20.3 NVMe ZNS (Zoned Namespaces)

**What it is:** Exposes flash zone structure to host. Sequential-write-only zones eliminate drive-side garbage collection → zero GC-induced latency spikes.

**Detection:** Identify Namespace CSI (Command Set Identifier) = 0x02 → ZNS namespace.

**Key parameters:**
- Zone Size (ZSZE): typically 256MB-2GB (commonly 1GB)
- Max Open Resources (MAR): 14-128 concurrent open zones
- Max Active Resources (MOR): active zone limit
- Zone Append Size Limit (ZASL): max per-append

**Zone Append command (opcode 0x7D):**
- Specifies only Zone Start LBA; controller returns actual written LBA
- Multiple writers can append concurrently — controller serializes
- **Perfect for concurrent WAL writers without locking**

**Zone states:** Empty → Open → Full → (Reset) → Empty

**Performance:** Write latency: 5-15μs with p99 ≈ p50 (no GC spikes). Conventional SSDs: p99 can spike to milliseconds during GC.
**Products:** Western Digital ZN540 (Gen4), Samsung PM1731a, FADU FC4121.

### 20.4 NVMe FDP (Flexible Data Placement)

**What it is:** Host directs different data streams to different flash Reclaim Units via Placement Handles in each Write command (CDW13 bits 15:0). Prevents lifetime-mixing of hot/cold data → 50% WAF reduction.

**Detection:** `Get Features` FID=0x1D; `nvme fdp status /dev/nvme0n1`.

**VectorVolt placement strategy:**
```
Placement Handle 0 → WAL data (short-lived, frequently reset)
Placement Handle 1 → B-tree/index data (medium-lived)
Placement Handle 2 → Table data pages (long-lived)
Placement Handle 3 → Temp/sort spill (very short-lived)
```

**Products:** Samsung PM9D3/PM9E1, Kioxia CD8P, Solidigm D7-PS1010.

### 20.5 NVMe-oF (NVMe over Fabrics)

| Transport | Additional Latency | Bandwidth | Infrastructure |
|-----------|-------------------|-----------|---------------|
| **NVMe/RDMA** | +5-15μs (total ~15-30μs for 4KB) | Line-rate: 100-400 Gbps | Requires RDMA NIC + lossless Ethernet |
| **NVMe/TCP** | +30-100μs (total ~40-120μs) | Can saturate 100GbE | Standard Ethernet |

**User-space:** `nvme connect -t rdma -a <ip> -s 4420 -n <nqn>` — remote appears as `/dev/nvmeXnY`.
**VectorVolt use:** NVMe-oF/RDMA for remote WAL replication (~15μs to commit on remote storage).

### 20.6 CXL Persistent Memory (Optane Successor)

**Intel Optane is discontinued (July 2022).** CXL Type 3 devices are the replacement.

**CXL Type 3 memory expanders:**
- Byte-addressable memory on PCIe/CXL bus
- Latency: ~120-170ns (adds 40-90ns over local DDR5)
- Appears as additional NUMA node to OS
- Programming model: DAX (`/dev/dax0.0`) + `clwb`/`sfence` — identical to Optane PMem

**Detection:** PCIe DVSEC (Vendor ID 0x1E98), `lspci | grep cxl`, `/sys/bus/cxl/devices/`.
**Products:** Samsung CMM-D (128GB), Micron CZ120 (256GB), SK Hynix CMB.

**VectorVolt tiered WAL path (auto-detect at startup):**
```
Tier 0: CXL PMem or NVMe PMR → ~200ns commit (byte-addressable)
Tier 1: NVMe CMB + ZNS Zone Append → ~8-12μs (zero GC, concurrent writers)
Tier 2: NVMe + FDP → ~10-15μs (reduced GC, data separation)
Tier 3: Conventional NVMe + SPDK → ~10-15μs (baseline, subject to GC spikes)
```

---

## 21. Network / NIC Optimizations (Intel-Specific + Vendor-Neutral) <a name="21-network"></a>

### 21.1 RDMA (Remote Direct Memory Access)

**Transport comparison for VectorVolt:**

| Property | InfiniBand | RoCEv2 | iWARP |
|----------|-----------|--------|-------|
| Latency (small msg) | ~0.6μs | 2-4μs | Higher (TCP overhead) |
| Switch latency | ~100ns | ~230ns | ~230ns |
| Infrastructure | Dedicated IB fabric | Standard Ethernet (DCB/PFC/ECN) | Standard Ethernet |
| **Recommendation** | Best latency | **Best balance for VectorVolt** | Not recommended |

**RDMA Verbs for VectorVolt:**
```
One-sided (no remote CPU involvement):
  IBV_WR_RDMA_WRITE        — WAL log shipping to remote buffer
  IBV_WR_RDMA_WRITE_WITH_IMM — write + notify remote CQ
  IBV_WR_RDMA_READ         — remote shard read
  IBV_WR_ATOMIC_CMP_AND_SWP — distributed lock acquisition

Two-sided:
  IBV_WR_SEND / ibv_post_recv — request/response for client connections
```

**Key optimization:** `IBV_SEND_INLINE` for messages ≤64-256 bytes eliminates PCIe read by NIC. Critical for small WAL entries.

**Queue Pairs (QP):** RC (Reliable Connected) for replication. **Completion Queues (CQ):** Polled via `ibv_poll_cq()` in busy-loop — zero syscalls. **Memory Regions (MR):** Register once at startup via `ibv_reg_mr()`.

**Latency numbers:**
- One-sided RDMA Write: 1.0-1.5μs (IB NDR), 2-4μs (RoCEv2 100GbE)
- Round-trip (write + poll CQ): 2-3μs (IB), 4-8μs (RoCEv2)

**Products:**
| NIC | Speed | RDMA | Notes |
|-----|-------|------|-------|
| NVIDIA ConnectX-7 | 400Gb/s | IB NDR + RoCEv2 | Gold standard, bifurcated DPDK |
| Intel E810-CQDA2 | 2×100G | iWARP + RoCEv2 | ADQ support, XDP/AF_XDP |
| Intel E830 | 200GbE | RoCEv2 | Succeeds E810 |
| Broadcom P2200G | 400Gb/s | RoCEv2 | PCIe 5.0 x16 |

**User-space API:** `rdma-core` library (libibverbs + librdmacm).

### 21.2 DPDK (Data Plane Development Kit)

**Poll-mode driver:** Zero syscalls, zero context switches in data path. 3-7μs packet latency (vs 20-50μs kernel).

**Requirements:**
- Hugepages (2MB or 1GB) for mbuf pools
- Bind NIC to VFIO-PCI: `dpdk-devbind.py -b vfio-pci <BDF>`
- **Exception:** NVIDIA mlx5 uses bifurcated driver — NIC stays in kernel, DPDK accesses directly. DPDK + RDMA coexist on same port.

**Intel-specific:** Intel E810/E830 support DPDK PMD natively. Requires `ice` driver unbind → `vfio-pci`.

### 21.3 AF_XDP (Alternative to DPDK)

**What it is:** Zero-copy path from NIC RX queue to userspace via `AF_XDP` socket + shared UMEM.

| Metric | AF_XDP (zero-copy) | DPDK |
|--------|-------------------|------|
| Rx throughput | ~39 Mpps | ~45-60 Mpps |
| Latency | Sub-10μs | 3-7μs |
| Kernel integration | Full — coexists with kernel | None — exclusive NIC |

**Advantage for VectorVolt:** AF_XDP doesn't take NIC from kernel. Management/SSH traffic still works. Intel ice driver has native AF_XDP zero-copy support.

### 21.4 Intel ADQ (Application Device Queues)

**Intel E810/E830 exclusive.** Creates dedicated hardware queue sets per application with guaranteed bandwidth, isolated from other traffic.

**Configuration:** `tc` (traffic control) + ADQ-aware ice driver.
**VectorVolt use:** Separate hardware queues for replication traffic vs client queries vs management — zero interference.

### 21.5 PTP Hardware Timestamping

**What it is:** NIC-embedded hardware clock (PHC) timestamps packets at PHY layer with nanosecond precision.

**Accuracy:**
| Deployment | Accuracy |
|-----------|---------|
| Same switch, HW timestamp | Sub-10 nanoseconds |
| Same datacenter, PTP grandmaster | 10-50 nanoseconds |
| Cross-datacenter | Sub-microsecond |

**Socket API:**
```c
int flags = SOF_TIMESTAMPING_TX_HARDWARE | SOF_TIMESTAMPING_RX_HARDWARE |
            SOF_TIMESTAMPING_RAW_HARDWARE;
setsockopt(fd, SOL_SOCKET, SO_TIMESTAMPING, &flags, sizeof(flags));
```

**Detection:** `ethtool -T eth0` shows HW timestamping caps. `ls /dev/ptp*` lists PHCs.

**VectorVolt use:** Distributed transaction ordering without centralized timestamp oracle. With 50ns accuracy, clock-based MVCC works globally (like Google Spanner's TrueTime).

### 21.6 RSS / Flow Director

**RSS:** NIC hashes packet headers → distributes across RX queues → each queue pinned to a CPU core.
```bash
ethtool -L eth0 combined 16            # 16 queues
ethtool --set-rxfh-indir eth0 equal 8  # spread across 8 queues
```

**Flow Director (Intel):** Hardware n-tuple filters pin specific flows to specific cores:
```bash
ethtool --config-ntuple eth0 flow-type tcp4 src-ip 10.0.0.1 dst-port 5432 action 3
# All TCP from 10.0.0.1:* → queue 3 → core 3
```

### 21.7 SmartNIC / DPU

| DPU | Speed | Key Offloads |
|-----|-------|-------------|
| NVIDIA BlueField-3 | 400Gb/s | TLS, IPsec, compression, NVMe-oF, OVS (~300 CPU cores equivalent) |
| AMD Pensando DSC-200 | 200Gb/s | P4-programmable pipeline, firewall, telemetry |
| Intel IPU (Mount Evans) | 200Gb/s | vSwitch, storage virtualization |

**VectorVolt offloads:** TLS termination (free host CPU from client encryption), IPsec for replication, NVMe-oF for remote storage access.

### 21.8 XDP (eXpress Data Path)

eBPF program at NIC driver level — earliest packet processing point in Linux.

| Action | Use for VectorVolt |
|--------|-------------------|
| XDP_DROP | Firewall: drop unauthorized packets at driver level |
| XDP_REDIRECT | Route to AF_XDP socket for zero-copy receive |
| XDP_TX | Bounce packets for load balancing |

---

## 22. Platform / BIOS / Memory Tuning (Intel-Specific) <a name="22-platform"></a>

### 22.1 C-State Tuning

| C-State | Exit Latency | What Powers Down |
|---------|-------------|-----------------|
| C0 | 0 (active) | Nothing |
| C1/C1E | ~1-2μs | Core clock halt |
| C3 | ~30-50μs | L1 flush, voltage reduced |
| C6 | ~80-200μs | Core voltage removed, state saved to LLC |
| C7 | ~100-250μs | LLC portion powered down |
| C8/C10 | ~300-500+μs | Package deep sleep |

**Key MSRs:**
```
MSR 0xE2 (IA32_PKG_CST_CONFIG_CONTROL):
  Bits [2:0] = Package C-State Limit (000b = C0/C1 only)

MSR 0x770 (IA32_PM_ENABLE):
  Bit 0 = HWP enable
```

**BIOS settings:** "CPU C-States" → Disable; "Package C-State Limit" → C0/C1; "C6 Report" → Disable.
**Kernel:** `intel_idle.max_cstate=1` or `processor.max_cstate=1`.

**VectorVolt recommendation:** Disable all C-states beyond C1. Eliminates 50-200μs wakeup tail latency.

### 22.2 P-State / Frequency Locking

**Key MSRs:**
```
MSR 0x199 (IA32_PERF_CTL): Bits [15:8] = desired frequency ratio
MSR 0x198 (IA32_PERF_STATUS): current frequency
MSR 0xCE  (MSR_PLATFORM_INFO): Bits [15:8] = max non-turbo, [47:40] = max efficiency
MSR 0x774 (IA32_HWP_REQUEST):
  Bits [7:0] = min perf, [15:8] = max perf, [23:16] = desired perf
  Bits [31:24] = EPP (Energy Performance Preference: 0=performance, 255=power-save)
```

**Configuration:**
```bash
echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
echo 100 > /sys/devices/system/cpu/intel_pstate/min_perf_pct
```

**VectorVolt recommendation:** `performance` governor + EPP=0. Constant max frequency.

### 22.3 Intel Turbo Boost Max 3.0

**What it is:** Factory-tested favored cores run 100-300 MHz faster. Pin hottest threads to them.

**Detection:** MSR 0x771 (IA32_HWP_CAPABILITIES) bits [7:0] = Highest Performance per core. Compare across cores — higher = faster.

**OS:** `intel-speed-select perf-profile info`. Linux `itmt` driver exposes core priority.
**BIOS:** "Intel Turbo Boost Max Technology 3.0" → Enable.

**VectorVolt recommendation:** Pin WAL writer + transaction coordinator to favored cores for 5-8% latency gain.

### 22.4 Intel Sub-NUMA Clustering (SNC)

**What it is:** Splits socket into 2 (SNC2) or 4 (SNC4) NUMA nodes by partitioning LLC slices and memory controllers.

**Sapphire Rapids (4 tiles):**
- SNC2: 2 tiles per node → half LLC + half memory channels each
- SNC4: 1 tile per node → quarter LLC + quarter memory channels each

**Performance:** SNC2 reduces average LLC access by 5-10ns (from ~25ns to ~15-18ns) due to fewer mesh hops.

**Detection:** `numactl --hardware` — SNC2 on 2-socket shows 4 NUMA nodes instead of 2.
**BIOS:** "Sub NUMA Clustering" → SNC2.

**VectorVolt recommendation:** Enable SNC2. Matches AMD NPS2 — same abstraction via `INumaTopology`.

### 22.5 Power Limit Tuning (PL1/PL2/RAPL)

**MSR 0x610 (MSR_PKG_POWER_LIMIT):**
```
Bits [14:0]  = PL1 (sustained, in power units from MSR 0x606)
Bit 15       = PL1 enable
Bits [23:17] = PL1 time window
Bits [46:32] = PL2 (burst)
Bit 47       = PL2 enable
Bit 63       = Lock
```

**VectorVolt recommendation:** Set PL1 = PL2 = max cooling allows. Eliminates throttling transitions.

### 22.6 LLC Prefetcher Tuning

**MSR 0x1A4 (prefetcher control, Intel Xeon):**
```
Bit 0: L2 Hardware Prefetcher (disable = set bit)
Bit 1: L2 Adjacent Cache Line Prefetch (disable = set bit)
Bit 2: DCU (L1) Prefetcher (disable = set bit)
Bit 3: DCU IP Prefetcher (disable = set bit)
```

**VectorVolt recommendation:** Disable "Adjacent Cache Line Prefetch" (Bit 1) — doubles bandwidth waste for random access. Keep L2 HW and DCU IP prefetchers enabled.

### 22.7 Patrol Scrub

**What it is:** Memory controller periodically scrubs all DRAM for ECC error detection. Can cause 1-5μs latency blips.
**BIOS:** "Patrol Scrub" → Disable (DDR5 on-die ECC provides base protection).

### 22.8 PCIe Gen5 Tuning

**Bifurcation:** Split x16 → 4×x4 for 4 NVMe SSDs on passive riser (zero latency overhead vs active PCIe switch which adds ~100-200ns).
**BIOS:** "PCIe Slot N Bifurcation" → x4x4x4x4.

**ACS (Access Control Services):** Enable for VFIO/IOMMU isolation. Disable for peer-to-peer DMA (e.g., NIC ↔ DSA direct).
**ATS (Address Translation Services):** Required for DSA PASID + Shared Virtual Memory.
**Max Payload Size:** 256B default (optimal for random OLTP I/O).
**Relaxed Ordering:** Enable for RDMA NIC (10-15% throughput gain). Keep strict for NVMe.

### 22.9 Hyper-Threading / SMT

**Impact of disabling SMT (per thread gains):**
- L1D: full 48KB (vs shared), L2: full 1.25-2MB, TLB: full DTLB/STLB entries
- All execution ports uncontested, full store buffer (56 entries on SPR)

**OLTP benchmarks:** SMT off = 5-15% lower p99 latency for compute-bound in-memory workloads. Redis/Memcached consistently better with SMT off.

**VectorVolt recommendation:** Disable SMT. Each shard writer gets full core resources. Deterministic, no sibling interference.

### 22.10 DDR5 Features

| Feature | Detail | Impact |
|---------|--------|--------|
| On-Die ECC (ODECC) | Mandatory in DDR5, corrects single-bit errors within DRAM chip | Transparent; complements system ECC |
| Same-Bank Refresh (SBR) | Only 1 bank refreshed at a time, others accessible | 30-40% less observable refresh latency vs DDR4 |
| Sub-Channel Architecture | 2 independent 32-bit sub-channels per DIMM | Doubles random-access transaction rate vs DDR4 |

**Optimal population (Sapphire Rapids, 8 channels/socket):**
| Config | Speed | Notes |
|--------|-------|-------|
| 1 DPC (8 × 64GB = 512GB) | DDR5-4800 | Full speed — **recommended** |
| 2 DPC (16 × 64GB = 1TB) | DDR5-4400 | ~8% derating |

### 22.11 Huge Pages ⭐

**TLP miss overhead:**
| Pages | TLB entries for 128GB | Miss cost |
|-------|----------------------|-----------|
| 4KB | 33.5M | ~30-50ns (4-level walk) |
| 2MB | 65,536 | ~20-30ns (3-level walk) |
| 1GB | 128 | ~10-15ns (2-level walk) |

**Transparent Huge Pages (THP) — DISABLE for databases:**
- Causes 1-100ms allocation stalls during compaction
- `khugepaged` background CPU (1-5%)
- 10-30% memory bloat from promotion
- Oracle, PostgreSQL, MongoDB, Redis, SAP HANA all recommend disabling THP

**Configuration:**
```bash
# Disable THP
echo never > /sys/kernel/mm/transparent_hugepage/enabled
echo never > /sys/kernel/mm/transparent_hugepage/defrag

# Reserve 1GB pages at boot (kernel cmdline)
hugepagesz=1G hugepages=128 default_hugepagesz=1G

# Per-NUMA node 2MB pages at runtime
echo 32768 > /sys/devices/system/node/node0/hugepages/hugepages-2048kB/nr_hugepages
```

**C#/.NET usage:**
```csharp
// P/Invoke to mmap with MAP_HUGETLB | MAP_HUGE_1GB | MAP_POPULATE
// MAP_POPULATE pre-faults all pages — no page fault latency on first access
[DllImport("libc.so.6")]
static extern IntPtr mmap(IntPtr addr, ulong length, int prot, int flags, int fd, long offset);

const int MAP_HUGETLB = 0x40000;
const int MAP_HUGE_1GB = 30 << 26; // MAP_HUGE_SHIFT
const int MAP_POPULATE = 0x08000;
const int MAP_PRIVATE  = 0x02;
const int MAP_ANONYMOUS = 0x20;
const int PROT_READ_WRITE = 0x03;

IntPtr bufferPool = mmap(IntPtr.Zero, bufferPoolSize,
    PROT_READ_WRITE,
    MAP_PRIVATE | MAP_ANONYMOUS | MAP_HUGETLB | MAP_HUGE_1GB | MAP_POPULATE,
    -1, 0);
```

**VectorVolt recommendation:** 1GB huge pages for buffer pool; 2MB for metadata/hash tables. Disable THP system-wide.

### 22.12 CXL Memory Tiering

| Version | Year | Key Addition |
|---------|------|-------------|
| CXL 1.1 | 2019 | Type 1/2/3 devices on PCIe Gen5 |
| CXL 2.0 | 2020 | Switching, static memory pooling, hotplug |
| CXL 3.0 | 2022 | Multi-level switching, dynamic shared memory, PCIe Gen6 |

**Type 3 latency:** ~120-170ns (adds 40-90ns over local DDR5's ~80ns).

**VectorVolt buffer pool tiers:**
```
Tier 0: L1/L2/LLC (hardware managed, ~1-25ns)
Tier 1: Local DDR5 (~80ns) — hot pages
Tier 2: CXL memory (~150ns) — warm pages (10x faster than NVMe!)
Tier 3: NVMe (~10-15μs) — cold pages
```

---

## 23. Updated Phase-by-Phase Matrix (All Hardware) <a name="23-full-matrix"></a>

### Phase 1 — Silicon/HAL (Updated)

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| All Phase 1 items from Section 17 | — | — |
| **NVMe capability detection** (CMB, PMR, ZNS, FDP) | `StorageCapabilities` | P0 |
| **CXL device enumeration** | `CxlMemoryManager` | P1 |
| **RDMA NIC detection** | `RdmaCapabilities` | P1 |
| **PTP clock detection** | `PrecisionClock` | P2 |

### Phase 2 — Single-Writer Engine (Updated)

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| All Phase 2 items from Section 17 | — | — |
| **Huge page buffer pool** (1GB pages) | `HugePageAllocator` | P0 |
| **C-state / P-state / SMT config validation** | `PlatformTuningValidator` | P1 |
| **TBMT 3.0 favored-core detection** | `FavoredCoreSelector` | P1 |

### Phase 3 — WAL/Durability (Updated)

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| All Phase 3 items from Section 17 | — | — |
| **NVMe PMR WAL commit** (~200ns) | `PmrWalWriter` | P0 |
| **ZNS Zone Append** (concurrent lock-free WAL) | `ZnsWalWriter` | P0 |
| **FDP data placement** (WAL vs data separation) | `FdpPlacementManager` | P1 |
| **NVMe CMB** (SQ in controller memory) | `CmbQueueManager` | P1 |
| **CXL PMem WAL** (byte-addressable persistent) | `CxlPmemWalWriter` | P1 |

### Phase 4 — Kernel Bypass Networking (Updated)

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| All Phase 4 items from Section 17 | — | — |
| **RDMA replication** (one-sided writes) | `RdmaReplicator` | P0 |
| **AF_XDP / DPDK client I/O** | `KernelBypassTransport` | P0 |
| **Intel ADQ traffic isolation** | `TrafficClassManager` | P1 |
| **PTP hardware timestamps** | `DistributedClock` | P1 |
| **RSS / Flow Director** | `FlowSteeringManager` | P1 |
| **NVMe-oF/RDMA remote storage** | `RemoteStorageClient` | P2 |
| **XDP firewall** | `PacketFilterEngine` | P2 |
| **SmartNIC/DPU offload** | `DpuOffloadManager` | P2 |

### Phase 5 — Read Fabric/SIMD (Updated)

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| All Phase 5 items from Section 17 | — | — |
| **CXL memory tier** (warm pages) | `CxlBufferPoolTier` | P1 |

### Phase 6 — Multi-Tenancy (Updated)

| Technology | Class/Component | Priority |
|-----------|----------------|----------|
| All Phase 6 items from Section 17 | — | — |
| **SNC2 NUMA partitioning** | Part of `INumaTopology` | P1 |

---

## 24. New Classes Added (Beyond-CPU) <a name="24-new-classes-beyond-cpu"></a>

### Storage Layer
```
StorageCapabilities         — Detect NVMe CMB/PMR/ZNS/FDP at startup
PmrWalWriter                — Byte-addressable WAL commit via PMR (~200ns)
CxlPmemWalWriter            — Byte-addressable WAL commit via CXL PMem (~200ns)
ZnsWalWriter                — Zone Append for concurrent lock-free WAL
FdpPlacementManager         — Per-I/O placement handle for data lifetime separation
CmbQueueManager             — Place NVMe SQs in Controller Memory Buffer
CxlMemoryManager            — CXL Type 3 device enumeration + NUMA node management
RemoteStorageClient         — NVMe-oF/RDMA for remote storage access
```

### Network Layer
```
RdmaCapabilities            — Detect RDMA NIC, verbs, transport type
RdmaReplicator              — One-sided RDMA Write for WAL log shipping
KernelBypassTransport       — AF_XDP or DPDK for client packet I/O
TrafficClassManager         — Intel ADQ / Flow Director queue isolation
FlowSteeringManager         — RSS indirection + n-tuple flow rules
DistributedClock            — PTP hardware timestamp for global ordering
PacketFilterEngine          — XDP/eBPF firewall at driver level
DpuOffloadManager           — SmartNIC/DPU task offloading
```

### Platform Layer
```
PlatformTuningValidator     — Verify C-state/P-state/SMT/THP/SNC config at startup
HugePageAllocator           — 1GB/2MB explicit huge page allocation with NUMA affinity
FavoredCoreSelector         — TBMT 3.0 detection for priority thread pinning
CxlBufferPoolTier           — Warm-page tier using CXL memory (~150ns)
```

---

---

# PART 3: RUNTIME ARCHITECTURE — PROBE ONCE, DISPATCH FOREVER

> **Added:** 2026-04-14 (Session 3). Defines the boot-time capability detection architecture, provider factory pattern, graceful degradation chains, Kubernetes deployment, and on-prem bare metal deployment. This is the **glue** that connects all hardware research (Parts 1 & 2) into a working runtime system.

> **Core Principle:** VectorVolt MUST run on a developer laptop with ZERO hardware features (all software fallbacks) AND scale to a fully-tuned EPYC/Xeon server with every hardware accelerator — **the same binary, same code paths, just different providers resolved at boot.** Zero capability checks on the hot path. Ever.

---

## 25. The Problem: No Per-Operation Hardware Checks <a name="25-problem"></a>

```csharp
// ❌ WRONG — checking capabilities on every WAL write adds microseconds
void WriteWal(ReadOnlySpan<byte> data) {
    if (HasDsa()) DsaCopyWithCrc(data);           // branch prediction miss + CPUID overhead
    else if (HasAvx512()) AvxCopy(data);
    else SoftwareCopy(data);
}

// ✅ CORRECT — resolved once at boot, zero branches on hot path
void WriteWal(ReadOnlySpan<byte> data) {
    _walWriter.Append(data);  // concrete type decided at startup, never changes
}
```

**The solution:** Probe all hardware capabilities ONCE during startup (~50-100ms), build an immutable capability manifest, use a factory to create concrete implementations of every interface, then freeze. The hot path never touches CPUID, MSR, PCI config space, or sysfs again.

---

## 26. Boot Sequence: Capability Detection Flow <a name="26-boot-sequence"></a>

```
VectorVolt Process Starts
        │
        ▼
┌──────────────────────┐
│  Phase 0: PROBE      │  (~50-100ms, runs on main thread before anything else)
│                      │
│  ┌────────────────┐  │
│  │ CpuProber      │  │  CPUID leaves, MSR reads, vendor detection
│  │ TopologyProber │  │  CPUID 0x1F, NUMA nodes, SLIT distances, cache hierarchy
│  │ StorageProber  │  │  NVMe BAR0 registers (CMB/PMR/ZNS/FDP), CXL DVSEC
│  │ NetworkProber  │  │  ibv_devices, ethtool, /dev/ptp*, DPDK bind status
│  │ PlatformProber │  │  HugePages sysfs, THP status, C-state, governor, cgroups
│  │ K8sProber      │  │  KUBERNETES_SERVICE_HOST env, cgroup limits, device mounts
│  └───────┬────────┘  │
│          │           │
│  ┌───────▼────────┐  │
│  │ SiliconManifest│  │  ← IMMUTABLE after this point. Sealed. Read-only.
│  │ (frozen)       │  │     Stored as static singleton. JIT can inline.
│  └───────┬────────┘  │
│          │           │
│  ┌───────▼────────┐  │
│  │ ProviderFactory│  │  Reads manifest → creates ONE concrete impl per interface
│  └───────┬────────┘  │
│          │           │
│  ┌───────▼────────┐  │
│  │ StartupReport  │  │  Logs selected providers + warnings to console/file
│  └────────────────┘  │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  Phase 1-6: RUN      │  All interfaces resolved. ZERO hardware checks.
│  (normal operation)  │  Every call goes through pre-resolved concrete type.
└──────────────────────┘
```

---

## 27. SiliconManifest — The Immutable Capability Struct <a name="27-manifest"></a>

```csharp
/// <summary>
/// Populated ONCE during boot by probers. Frozen after construction.
/// Every subsystem reads from this. Nobody writes after boot.
/// Stored as a static readonly singleton — JIT can devirtualize and inline.
/// </summary>
public sealed record SiliconManifest
{
    // ════════════════════════════════════════════════════════════
    //  CPU — detected via CPUID + MSR reads
    // ════════════════════════════════════════════════════════════
    public CpuVendor Vendor { get; init; }              // Intel | Amd | Unknown
    public SimdLevel Simd { get; init; }                // Avx512 | Avx2 | Sse42 | Scalar
    public bool HasCrc32 { get; init; }                 // SSE4.2 CRC32C instruction
    public bool HasPclmulqdq { get; init; }             // Carry-less multiply (GCM/CRC64)
    public bool HasAesNi { get; init; }                 // AES-NI instructions
    public bool HasDsa { get; init; }                   // Intel DSA (Data Streaming Accelerator)
    public bool HasIaa { get; init; }                   // Intel IAA (compression accelerator)
    public bool HasFred { get; init; }                  // FRED interrupt delivery
    public bool HasMktme { get; init; }                 // Intel MKTME (per-page encryption)
    public int MktmeMaxKeys { get; init; }              // 0 if not available, up to 32K
    public bool HasSevSnp { get; init; }                // AMD SEV-SNP (per-VM encryption)
    public int SevMaxAsids { get; init; }               // 0 if not available, up to 1006
    public bool HasKeyLocker { get; init; }             // Intel Key Locker (opaque key handles)
    public bool HasAvx512NoThrottle { get; init; }      // Zen 5: no frequency downclocking
    public bool HasVp2Intersect { get; init; }          // Zen 5: set intersection instruction
    public bool Has5LevelPaging { get; init; }          // CR4.LA57 (57-bit VA)
    public bool HasEnhancedIbrs { get; init; }          // Spectre: always-on, low overhead
    public bool HasRdclNo { get; init; }                // Meltdown: no L1D flush needed
    public bool HasSsbNo { get; init; }                 // SSB: no mitigation needed

    // ════════════════════════════════════════════════════════════
    //  TOPOLOGY — detected via CPUID 0x1F + ACPI SRAT/SLIT
    // ════════════════════════════════════════════════════════════
    public int PhysicalCoreCount { get; init; }
    public int LogicalProcessorCount { get; init; }
    public bool SmtEnabled { get; init; }               // true = WARNING at startup
    public int NumaNodeCount { get; init; }
    public NumaDistanceMatrix NumaDistances { get; init; }
    public int[] FavoredCoreIds { get; init; }          // TBMT 3.0 or CPPC2 preferred
    public NpsLevel AmdNps { get; init; }               // Nps1 | Nps2 | Nps4 | NotAmd
    public SncLevel IntelSnc { get; init; }             // Disabled | Snc2 | Snc4 | NotIntel
    public int L3CacheBytesPerNode { get; init; }
    public int L2CacheBytesPerCore { get; init; }
    public int CacheLineSize { get; init; }             // 64 bytes (validated, not assumed)
    public TopologyNode[] FullTopology { get; init; }   // Per-LP: APIC ID, package, die, core, SMT

    // ════════════════════════════════════════════════════════════
    //  STORAGE — detected via NVMe register reads + sysfs/CXL enum
    // ════════════════════════════════════════════════════════════
    public bool NvmePmrAvailable { get; init; }         // Byte-addressable persistent (~200ns)
    public long NvmePmrSizeBytes { get; init; }         // 0 if not available
    public bool NvmeZnsAvailable { get; init; }         // Zoned Namespaces (no GC)
    public int NvmeZnsMaxOpenZones { get; init; }       // Concurrent zone writers
    public long NvmeZnsZoneSizeBytes { get; init; }     // Zone size (typically 1GB)
    public bool NvmeFdpAvailable { get; init; }         // Flexible Data Placement
    public int NvmeFdpMaxPlacementHandles { get; init; }
    public bool NvmeCmbAvailable { get; init; }         // Controller Memory Buffer
    public long NvmeCmbSizeBytes { get; init; }
    public bool CxlMemoryAvailable { get; init; }       // CXL Type 3 memory expander
    public int CxlNumaNodeId { get; init; }             // Which NUMA node is CXL, -1 if none
    public long CxlMemorySizeBytes { get; init; }
    public long CxlLatencyNs { get; init; }             // Measured or estimated

    // ════════════════════════════════════════════════════════════
    //  NETWORK — detected via ibv_devices, ethtool, sysfs
    // ════════════════════════════════════════════════════════════
    public bool RdmaAvailable { get; init; }
    public RdmaTransportType RdmaType { get; init; }    // RoCEv2 | InfiniBand | iWarp | None
    public string RdmaDeviceName { get; init; }         // e.g., "mlx5_0"
    public int RdmaPortSpeed { get; init; }             // Gbps (100, 200, 400)
    public bool DpdkAvailable { get; init; }            // VFIO-PCI bound NIC detected
    public bool AfXdpAvailable { get; init; }           // AF_XDP zero-copy capable driver
    public bool PtpHardwareAvailable { get; init; }     // PTP hardware clock
    public string PtpDevicePath { get; init; }          // e.g., "/dev/ptp0"
    public bool AdqAvailable { get; init; }             // Intel ADQ (E810/E830)
    public bool SmartNicAvailable { get; init; }        // DPU detected (BlueField/Pensando)
    public string SmartNicType { get; init; }           // "BlueField-3" | "Pensando" | null

    // ════════════════════════════════════════════════════════════
    //  PLATFORM — detected via sysfs, kernel params, cgroups
    // ════════════════════════════════════════════════════════════
    public bool HugePages1GAvailable { get; init; }
    public bool HugePages2MAvailable { get; init; }
    public long HugePages1GTotal { get; init; }         // Count of 1GB pages
    public long HugePages2MTotal { get; init; }         // Count of 2MB pages
    public bool ThpDisabled { get; init; }              // false = WARNING
    public bool CStatesDisabled { get; init; }          // false = WARNING
    public string CpuGovernor { get; init; }            // "performance" expected
    public bool PatrolScrubDisabled { get; init; }      // false = WARNING

    // ════════════════════════════════════════════════════════════
    //  DEPLOYMENT CONTEXT — K8s vs bare metal vs VM
    // ════════════════════════════════════════════════════════════
    public DeploymentMode Deployment { get; init; }     // BareMetal | Kubernetes | Vm | Container
    public bool IsKubernetes { get; init; }             // KUBERNETES_SERVICE_HOST detected
    public bool IsVirtualized { get; init; }            // Hypervisor detected via CPUID
    public int CpuBudget { get; init; }                 // cgroup cpu.max or physical cores
    public long MemoryBudgetBytes { get; init; }        // cgroup memory.max or physical

    // ════════════════════════════════════════════════════════════
    //  STARTUP WARNINGS — non-fatal, logged prominently
    // ════════════════════════════════════════════════════════════
    public IReadOnlyList<StartupWarning> Warnings { get; init; }
}

public record StartupWarning(WarningSeverity Severity, string Code, string Message, string Recommendation);
public enum WarningSeverity { Info, Warning, Critical }
public enum CpuVendor { Intel, Amd, Unknown }
public enum SimdLevel { Avx512, Avx2, Sse42, Scalar }
public enum NpsLevel { Nps1, Nps2, Nps4, NotAmd }
public enum SncLevel { Disabled, Snc2, Snc4, NotIntel }
public enum RdmaTransportType { RoCEv2, InfiniBand, iWarp, None }
public enum DeploymentMode { BareMetal, Kubernetes, Vm, Container }
```

---

## 28. Probers — How Each Category Is Detected <a name="28-probers"></a>

### 28.1 CpuProber

```csharp
public static partial class CpuProber
{
    public static void Probe(SiliconManifestBuilder b)
    {
        // ── Vendor detection (CPUID leaf 0) ──
        var (_, ebx, ecx, edx) = X86Base.CpuId(0, 0);
        b.Vendor = (ebx, edx, ecx) switch {
            (0x756E6547, 0x49656E69, 0x6C65746E) => CpuVendor.Intel,
            (0x68747541, 0x69746E65, 0x444D4163) => CpuVendor.Amd,
            _ => CpuVendor.Unknown
        };

        // ── SIMD level ──
        b.Simd = Avx512F.IsSupported  ? SimdLevel.Avx512
               : Avx2.IsSupported     ? SimdLevel.Avx2
               : Sse42.IsSupported    ? SimdLevel.Sse42
               :                        SimdLevel.Scalar;

        // ── Instruction support ──
        b.HasCrc32     = Sse42.IsSupported;
        b.HasPclmulqdq = Pclmulqdq.IsSupported;
        b.HasAesNi     = Aes.IsSupported;

        // ── Intel-specific features (CPUID leaf 7) ──
        if (b.Vendor == CpuVendor.Intel) {
            var (eax7, ebx7, ecx7, edx7) = X86Base.CpuId(7, 0);
            b.HasMktme     = (ecx7 & (1 << 13)) != 0;  // TME
            b.HasKeyLocker = (ecx7 & (1 << 23)) != 0;

            var (eax7_1, _, _, _) = X86Base.CpuId(7, 1);
            b.HasFred      = (eax7_1 & (1 << 17)) != 0;

            // DSA: detected via PCI enumeration (not CPUID)
            // IAA: detected via PCI enumeration
            // MKTME keys: read from MSR 0x981 bits [50:36]

            // Enhanced IBRS
            if ((edx7 & (1 << 29)) != 0) { // IA32_ARCH_CAPABILITIES present
                // Read MSR 0x10A
                long archCaps = ReadMsr(0x10A);
                b.HasEnhancedIbrs = (archCaps & (1 << 1)) != 0;
                b.HasRdclNo       = (archCaps & (1 << 0)) != 0;
                b.HasSsbNo        = (archCaps & (1 << 4)) != 0;
            }

            // MKTME max keys: MSR 0x981 bits [50:36]
            if (b.HasMktme) {
                long tmeCap = ReadMsr(0x981);
                b.MktmeMaxKeys = (int)((tmeCap >> 36) & 0x7FFF);
            }
        }

        // ── AMD-specific features ──
        if (b.Vendor == CpuVendor.Amd) {
            var (eaxSev, ebxSev, ecxSev, edxSev) =
                X86Base.CpuId(unchecked((int)0x8000001F), 0);
            b.HasSevSnp        = (eaxSev & (1 << 4)) != 0;
            b.SevMaxAsids      = ecxSev;  // max encrypted guests

            // Zen 5 AVX-512 without throttle
            b.HasAvx512NoThrottle = (b.Simd == SimdLevel.Avx512
                                     && b.Vendor == CpuVendor.Amd);
        }

        // ── 5-Level Paging ──
        var (eax7b, _, ecx7b, _) = X86Base.CpuId(7, 0);
        b.Has5LevelPaging = (ecx7b & (1 << 16)) != 0;
    }
}
```

### 28.2 TopologyProber

```csharp
public static partial class TopologyProber
{
    public static void Probe(SiliconManifestBuilder b)
    {
        // ── CPUID 0x1F enumeration (V2 Extended Topology) ──
        var topology = new List<TopologyNode>();
        int maxSubleaf = 0;
        int shiftSmt = 0, shiftCore = 0, shiftModule = 0, shiftDie = 0;

        for (int subleaf = 0; subleaf < 16; subleaf++) {
            var (eax, ebx, ecx, edx) = X86Base.CpuId(0x1F, subleaf);
            int levelType = (ecx >> 8) & 0xFF;
            if (levelType == 0) break;  // Invalid = stop

            int shift = eax & 0x1F;
            switch (levelType) {
                case 1: shiftSmt = shift; break;     // SMT
                case 2: shiftCore = shift; break;    // Core
                case 3: shiftModule = shift; break;  // Module
                case 5: shiftDie = shift; break;     // Die
            }
        }

        // ── Enumerate all logical processors ──
        // Run CPUID on each LP via thread affinity
        // Extract APIC ID decomposition per LP
        // Build TopologyNode[] array

        // ── NUMA from OS ──
        // Linux: parse /sys/devices/system/node/
        // Read SLIT distances from /sys/devices/system/node/node*/distance

        // ── Detect NUMA partitioning mode ──
        b.NumaNodeCount = /* from OS */;
        if (b.Vendor == CpuVendor.Amd) {
            // Infer NPS from node count + cores per node
            b.AmdNps = b.NumaNodeCount switch {
                >= 4 => NpsLevel.Nps4,
                >= 2 => NpsLevel.Nps2,
                _    => NpsLevel.Nps1
            };
        }
        if (b.Vendor == CpuVendor.Intel) {
            // SNC detection: if NUMA nodes > sockets, SNC is enabled
            int sockets = /* count unique package IDs */;
            b.IntelSnc = (b.NumaNodeCount / sockets) switch {
                >= 4 => SncLevel.Snc4,
                >= 2 => SncLevel.Snc2,
                _    => SncLevel.Disabled
            };
        }

        // ── Favored cores ──
        // Intel: read MSR 0x771 (IA32_HWP_CAPABILITIES) per core, compare bits [7:0]
        // AMD: read /sys/devices/system/cpu/cpu*/cpufreq/amd_pstate_prefcore_ranking
        // Sort by ranking, top N = favored

        // ── SMT detection ──
        b.SmtEnabled = b.LogicalProcessorCount > b.PhysicalCoreCount;
        if (b.SmtEnabled) {
            b.AddWarning(WarningSeverity.Warning, "SMT_ENABLED",
                "SMT/Hyper-Threading is enabled. VectorVolt recommends disabling for lowest p99 latency.",
                "BIOS: Disable 'Hyper-Threading' (Intel) or 'SMT Control' (AMD)");
        }
    }
}
```

### 28.3 StorageProber

```csharp
public static partial class StorageProber
{
    public static void Probe(SiliconManifestBuilder b)
    {
        // ── NVMe device enumeration ──
        // On Linux: iterate /sys/class/nvme/nvme*/
        // For each NVMe controller, mmap BAR0 and read capability registers

        // PMR detection: BAR0 + 0xE00 (PMRCAP) — non-zero = PMR supported
        // CMB detection: BAR0 + 0x58  (CMBSZ)  — non-zero = CMB supported
        // ZNS detection: Identify Namespace CSI = 0x02
        // FDP detection: Get Features FID=0x1D

        // ── CXL enumeration ──
        // Linux: iterate /sys/bus/cxl/devices/
        // Or: lspci for DVSEC with Vendor ID 0x1E98
        // If CXL Type 3 found, identify which NUMA node it maps to

        // ── Tiered storage path selection (logged at startup) ──
        if (b.NvmePmrAvailable)      Log("WAL path: PMR (~200ns byte-addressable commit)");
        else if (b.NvmeZnsAvailable)  Log("WAL path: ZNS Zone Append (~8-12μs, zero GC)");
        else if (b.NvmeFdpAvailable)  Log("WAL path: FDP (~10-15μs, reduced GC)");
        else                          Log("WAL path: Standard NVMe (~10-15μs, baseline)");
    }
}
```

### 28.4 NetworkProber

```csharp
public static partial class NetworkProber
{
    public static void Probe(SiliconManifestBuilder b)
    {
        // ── RDMA detection ──
        // Linux: check /sys/class/infiniband/ for devices
        // Or: P/Invoke ibv_get_device_list() from libibverbs
        // Read device attributes: port speed, transport type (IB/RoCE/iWARP)

        // ── DPDK detection ──
        // Check if any NIC is bound to vfio-pci: parse /sys/bus/pci/drivers/vfio-pci/
        // Or: check if mlx5 bifurcated driver is loaded (coexists with kernel)

        // ── AF_XDP detection ──
        // Check NIC driver capabilities: ethtool -i <iface> → driver name
        // Known AF_XDP zero-copy drivers: ice, mlx5, bnxt, i40e

        // ── PTP detection ──
        // Check /dev/ptp* existence
        // Or: ethtool -T <iface> for hardware timestamping capabilities

        // ── Intel ADQ detection ──
        // Check driver (ice) + ethtool hw-tc-offload capability

        // ── SmartNIC/DPU detection ──
        // lspci for BlueField (Mellanox vendor ID + BF device ID)
        // Or: check /opt/mellanox/doca/ existence
        // lspci for Pensando (AMD vendor ID + Elba/Capri device ID)
    }
}
```

### 28.5 PlatformProber

```csharp
public static partial class PlatformProber
{
    public static void Probe(SiliconManifestBuilder b)
    {
        // ── Huge Pages ──
        // Linux: read /sys/kernel/mm/hugepages/hugepages-1048576kB/nr_hugepages (1GB)
        // Linux: read /sys/kernel/mm/hugepages/hugepages-2048kB/nr_hugepages (2MB)

        if (!b.HugePages1GAvailable && !b.HugePages2MAvailable) {
            b.AddWarning(WarningSeverity.Critical, "NO_HUGEPAGES",
                "No huge pages available. Buffer pool will use 4KB pages with severe TLB pressure.",
                "Reserve: echo 128 > /proc/sys/vm/nr_hugepages or boot with hugepagesz=1G hugepages=128");
        }

        // ── THP check ──
        // Read /sys/kernel/mm/transparent_hugepage/enabled
        // If not "never" or "[never]", warn
        if (!b.ThpDisabled) {
            b.AddWarning(WarningSeverity.Critical, "THP_ENABLED",
                "Transparent Huge Pages enabled. Will cause 1-100ms allocation stalls.",
                "echo never > /sys/kernel/mm/transparent_hugepage/enabled");
        }

        // ── C-state check ──
        // Read /sys/devices/system/cpu/cpu0/cpuidle/state*/disable
        // Or check intel_idle.max_cstate kernel param

        // ── CPU Governor ──
        // Read /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
        if (b.CpuGovernor != "performance") {
            b.AddWarning(WarningSeverity.Warning, "GOVERNOR_NOT_PERF",
                $"CPU governor is '{b.CpuGovernor}', not 'performance'. May cause frequency ramp latency.",
                "echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor");
        }

        // ── Kubernetes detection ──
        b.IsKubernetes = Environment.GetEnvironmentVariable("KUBERNETES_SERVICE_HOST") != null;
        if (b.IsKubernetes) {
            // Read cgroup cpu.max for CPU budget
            // Read cgroup memory.max for memory budget
            // Detect if Guaranteed QoS (cpu requests == limits)
        }

        // ── Virtualization detection ──
        // CPUID leaf 1, ECX bit 31 = hypervisor present
        var (_, _, ecx1, _) = X86Base.CpuId(1, 0);
        b.IsVirtualized = (ecx1 & (1 << 31)) != 0;
        if (b.IsVirtualized) {
            b.AddWarning(WarningSeverity.Info, "VIRTUALIZED",
                "Running in a VM/hypervisor. Some hardware features (DSA, MKTME, PMR) may be unavailable.",
                "For full performance, deploy on bare metal or with PCIe passthrough.");
        }
    }
}
```

---

## 29. ProviderFactory — One-Time Dispatch Resolution <a name="29-factory"></a>

```csharp
/// <summary>
/// Creates concrete implementations of every hardware-abstracted interface.
/// Called ONCE during boot. After this, the engine NEVER checks capabilities again.
/// All returned types are sealed — JIT can devirtualize and inline.
/// </summary>
public static class ProviderFactory
{
    public static VectorVoltProviders CreateAll(SiliconManifest m)
    {
        return new VectorVoltProviders
        {
            DataMover           = CreateDataMover(m),
            WalWriter           = CreateWalWriter(m),
            TenantIsolation     = CreateTenantIsolation(m),
            ClientTransport     = CreateClientTransport(m),
            ReplicationTransport = CreateReplicationTransport(m),
            BufferPool          = CreateBufferPool(m),
            Clock               = CreateClock(m),
            ChecksumProvider    = CreateChecksum(m),
            EncryptionProvider  = CreateEncryption(m),
            CompressionProvider = CreateCompression(m),
            CoreSelector        = CreateCoreSelector(m),
        };
    }

    // ── Data Movement (memcpy, fill, compare, dualcast) ──
    static IDataMover CreateDataMover(SiliconManifest m) => m switch
    {
        { HasDsa: true }                         => new DsaDataMover(m),
        { Simd: SimdLevel.Avx512 }               => new Avx512DataMover(),
        { Simd: SimdLevel.Avx2 }                 => new Avx2DataMover(),
        _                                        => new ScalarDataMover()
    };

    // ── WAL Writer (commit path — most latency-critical) ──
    static IWalWriter CreateWalWriter(SiliconManifest m) => m switch
    {
        { NvmePmrAvailable: true, HasDsa: true }  => new PmrDsaWalWriter(m),    // Best: PMR commit + DSA CRC
        { NvmePmrAvailable: true }                 => new PmrWalWriter(m),       // PMR commit + software CRC
        { CxlMemoryAvailable: true }               => new CxlPmemWalWriter(m),  // CXL byte-addressable
        { NvmeZnsAvailable: true, HasDsa: true }   => new ZnsDsaWalWriter(m),   // ZNS + DSA CRC
        { NvmeZnsAvailable: true }                 => new ZnsWalWriter(m),       // ZNS + software CRC
        { NvmeFdpAvailable: true }                 => new FdpWalWriter(m),       // FDP placement
        { HasDsa: true }                           => new DsaWalWriter(m),       // DSA Copy+CRC on std NVMe
        _                                          => new StandardWalWriter(m)   // mmap + software CRC
    };

    // ── Tenant Isolation (encryption) ──
    static ITenantIsolation CreateTenantIsolation(SiliconManifest m) => m switch
    {
        { HasMktme: true, MktmeMaxKeys: > 0 }     => new MktmeIsolation(m),
        { HasSevSnp: true, SevMaxAsids: > 0 }     => new SevSnpIsolation(m),
        _                                          => new ProcessIsolation()     // OS-level fallback
    };

    // ── Client Network Transport ──
    static INetworkTransport CreateClientTransport(SiliconManifest m) => m switch
    {
        { DpdkAvailable: true }                    => new DpdkTransport(m),
        { AfXdpAvailable: true }                   => new AfXdpTransport(m),
        _                                          => new SocketTransport()
    };

    // ── Replication Transport ──
    static IReplicationTransport CreateReplicationTransport(SiliconManifest m) => m switch
    {
        { RdmaAvailable: true }                    => new RdmaReplicator(m),
        _                                          => new TcpReplicator()
    };

    // ── Buffer Pool (memory allocation) ──
    static IBufferPool CreateBufferPool(SiliconManifest m) => m switch
    {
        { CxlMemoryAvailable: true, HugePages1GAvailable: true }
                                                   => new TieredHugePageBufferPool(m),
        { HugePages1GAvailable: true }             => new HugePageBufferPool(m, HugePageSize.OneGb),
        { HugePages2MAvailable: true }             => new HugePageBufferPool(m, HugePageSize.TwoMb),
        _                                          => new StandardBufferPool()
    };

    // ── Clock (distributed ordering) ──
    static IClock CreateClock(SiliconManifest m) => m switch
    {
        { PtpHardwareAvailable: true }             => new PtpHardwareClock(m.PtpDevicePath),
        _                                          => new MonotonicSystemClock()
    };

    // ── Checksum ──
    static IChecksumProvider CreateChecksum(SiliconManifest m) => m switch
    {
        { HasDsa: true }                           => new DsaCrcProvider(m),
        { HasCrc32: true, HasPclmulqdq: true }     => new HardwareCrcProvider(),
        { HasCrc32: true }                         => new Sse42CrcProvider(),
        _                                          => new SoftwareCrcProvider()
    };

    // ── Encryption (data at rest) ──
    static IEncryptionProvider CreateEncryption(SiliconManifest m) => m switch
    {
        { HasKeyLocker: true, HasAesNi: true }     => new KeyLockerAesGcm(),
        { HasAesNi: true, HasPclmulqdq: true }     => new AesNiGcmProvider(),
        _                                          => new SoftwareAesProvider()
    };

    // ── Compression (cold data tiering) ──
    static ICompressionProvider CreateCompression(SiliconManifest m) => m switch
    {
        { HasIaa: true }                           => new IaaCompressor(m),
        _                                          => new SoftwareDeflateCompressor()
    };

    // ── Core Selection (thread pinning) ──
    static ICoreSelector CreateCoreSelector(SiliconManifest m) => m switch
    {
        { FavoredCoreIds.Length: > 0 }             => new FavoredCoreSelector(m),
        { NumaNodeCount: > 1 }                     => new NumaAwareCoreSelector(m),
        _                                          => new SimpleCoreSelector(m)
    };
}
```

---

## 30. Graceful Degradation Chains <a name="30-degradation"></a>

Every capability has a **full chain** from best to baseline:

### 30.1 Data Movement Chain
```
Intel DSA (hardware, zero CPU)
  └→ AVX-512 VMOVDQA64 (SIMD, ~1 cycle/64 bytes)
       └→ AVX2 VMOVDQA (SIMD, ~1 cycle/32 bytes)
            └→ REP MOVSB with ERMS (scalar, hardware-optimized)
                 └→ Buffer.MemoryCopy (managed, baseline)
```

### 30.2 WAL Commit Chain
```
NVMe PMR + DSA CRC (~200ns, hardware CRC, byte-addressable)
  └→ NVMe PMR + software CRC (~200ns commit, ~50ns CRC)
       └→ CXL PMem + software CRC (~200ns, byte-addressable)
            └→ NVMe ZNS Zone Append + DSA CRC (~8-12μs, no GC, hardware CRC)
                 └→ NVMe ZNS Zone Append + software CRC (~8-12μs)
                      └→ NVMe + FDP + software CRC (~10-15μs, reduced GC)
                           └→ NVMe + DSA Copy+CRC (~10-15μs, hardware CRC)
                                └→ NVMe + mmap + software CRC (~10-15μs, baseline)
```

### 30.3 Replication Chain
```
RDMA one-sided write (sub-2μs, zero remote CPU)
  └→ DPDK + custom protocol (~5-10μs)
       └→ AF_XDP zero-copy (~10μs)
            └→ TCP kernel networking (~50-100μs)
```

### 30.4 Client I/O Chain
```
DPDK poll-mode (3-7μs, dedicated core)
  └→ AF_XDP zero-copy (~10μs, kernel coexistence)
       └→ io_uring (~15-25μs, modern kernel)
            └→ epoll + TCP (~30-50μs, baseline)
```

### 30.5 Tenant Isolation Chain
```
Intel MKTME per-page (hardware AES-XTS, per-tenant KeyID, 32K tenants)
  └→ AMD SEV-SNP per-VM (hardware AES-128, per-VM ASID, 1006 tenants)
       └→ Intel Key Locker + software page crypto (opaque key handles)
            └→ AES-NI + PCLMULQDQ AES-GCM (software, wire-speed with HW instructions)
                 └→ Process-level isolation (OS boundaries, no encryption)
```

### 30.6 Checksum Chain
```
DSA Copy+CRC (hardware, inline with memcpy, zero extra CPU)
  └→ SSE4.2 CRC32C + PCLMULQDQ CRC64 (hardware instructions)
       └→ SSE4.2 CRC32C only (single instruction)
            └→ Software CRC lookup table (baseline)
```

### 30.7 Compression Chain
```
Intel IAA (hardware deflate, 2.5-10x faster than software)
  └→ Software zstd (good ratio, moderate CPU)
       └→ Software LZ4 (fast, lower ratio)
            └→ No compression (baseline)
```

### 30.8 Clock / Ordering Chain
```
PTP hardware timestamp (sub-50ns accuracy, NIC PHC)
  └→ PTP software timestamp (~1μs accuracy)
       └→ clock_gettime(CLOCK_MONOTONIC_RAW) (~25ns, local only)
            └→ DateTime.UtcNow (.NET, ~100ns, local only)
```

### 30.9 Buffer Pool Chain
```
1GB HugePages + CXL warm tier (hot DDR5 + warm CXL + cold NVMe)
  └→ 1GB HugePages single tier (near-zero TLB misses)
       └→ 2MB HugePages single tier (low TLB misses)
            └→ 4KB standard pages (high TLB pressure, baseline)
```

### 30.10 Core Selection Chain
```
TBMT 3.0 / CPPC2 Preferred Cores (pin to fastest silicon)
  └→ NUMA-aware core pinning (pin to same NUMA node as memory)
       └→ Simple core pinning (sched_setaffinity, avoid siblings)
            └→ OS scheduler (no pinning, baseline)
```

---

## 31. Startup Report Format <a name="31-startup-report"></a>

```
╔══════════════════════════════════════════════════════════════════════╗
║  VectorVolt v0.1.0 — Platform Capability Report                     ║
╠══════════════════════════════════════════════════════════════════════╣
║                                                                      ║
║  ── CPU ──                                                           ║
║  Vendor:     Intel Xeon w5-3435X (Sapphire Rapids)                   ║
║  Cores:      16 physical (SMT disabled ✓)                            ║
║  SIMD:       AVX-512 ✓                                               ║
║  DSA:        1 device, 4 engines, 8 WQs ✓                           ║
║  IAA:        1 device ✓                                              ║
║  MKTME:      Supported, 63 KeyIDs available ✓                       ║
║  FRED:       Supported ✓                                             ║
║  Key Locker: Supported ✓                                             ║
║  Spectre:    Enhanced IBRS ✓  RDCL_NO ✓  SSB_NO ✓                   ║
║  Paging:     5-Level (LA57) ✓                                        ║
║                                                                      ║
║  ── Topology ──                                                      ║
║  NUMA:       2 nodes (SNC2 enabled ✓)                                ║
║  Distances:  [10, 21; 21, 10]                                        ║
║  Favored:    Cores 3, 7 (TBMT 3.0 detected)                         ║
║  L3 Cache:   22.5 MB per SNC node                                    ║
║                                                                      ║
║  ── Storage ──                                                       ║
║  WAL Device: Samsung PM9D3 (/dev/nvme0n1)                            ║
║    PMR:      ✗ Not available                                         ║
║    ZNS:      ✗ Not available                                         ║
║    FDP:      ✓ 8 Placement Handles                                   ║
║    CMB:      ✓ 256KB (SQ placement enabled)                          ║
║  CXL:        ✗ No CXL memory devices detected                       ║
║                                                                      ║
║  ── Network ──                                                       ║
║  NIC:        ConnectX-7 (mlx5_0) — 2×100GbE                         ║
║  RDMA:       RoCEv2 ✓ (200 Gbps)                                    ║
║  DPDK:       ✗ (NIC not bound to vfio-pci, using bifurcated)        ║
║  AF_XDP:     ✓ (mlx5 zero-copy)                                     ║
║  PTP:        ✓ (/dev/ptp0, hardware timestamping)                    ║
║  ADQ:        ✗ (Intel NIC required)                                  ║
║  DPU:        ✗ Not detected                                          ║
║                                                                      ║
║  ── Platform ──                                                      ║
║  HugePages:  128 × 1GB reserved (128 GB) ✓                          ║
║  THP:        Disabled ✓                                              ║
║  C-States:   C1 only ✓                                               ║
║  Governor:   performance ✓                                           ║
║  Patrol:     Disabled ✓                                              ║
║  Deploy:     Bare Metal                                              ║
║                                                                      ║
╠══════════════════════════════════════════════════════════════════════╣
║  SELECTED PROVIDERS                                                  ║
╠══════════════════════════════════════════════════════════════════════╣
║  DataMover:      DsaDataMover (Intel DSA, 4 engines)                 ║
║  WAL Writer:     FdpDsaWalWriter (FDP placement + DSA Copy+CRC)      ║
║  Tenant Isolation: MktmeIsolation (63 KeyIDs)                        ║
║  Client I/O:     AfXdpTransport (mlx5 zero-copy)                     ║
║  Replication:    RdmaReplicator (RoCEv2, one-sided writes)           ║
║  Buffer Pool:    HugePageBufferPool (128 × 1GB)                      ║
║  Clock:          PtpHardwareClock (/dev/ptp0)                        ║
║  Checksum:       DsaCrcProvider (inline Copy+CRC)                    ║
║  Encryption:     KeyLockerAesGcm (opaque handles)                    ║
║  Compression:    IaaCompressor (hardware deflate)                    ║
║  Core Selector:  FavoredCoreSelector (cores 3, 7 preferred)         ║
║                                                                      ║
╠══════════════════════════════════════════════════════════════════════╣
║  ⚠ WARNINGS (0 critical, 1 warning, 1 info)                        ║
╠══════════════════════════════════════════════════════════════════════╣
║  [WARN]  NO_PMR — NVMe PMR not available. Using FDP WAL path.       ║
║          → For ~200ns commits, use Samsung PM1733 or Kioxia CM6.     ║
║  [INFO]  VIRTUALIZED_NO — Running on bare metal. Full HW access.     ║
║                                                                      ║
╠══════════════════════════════════════════════════════════════════════╣
║  ESTIMATED LATENCY BUDGET                                            ║
╠══════════════════════════════════════════════════════════════════════╣
║  Client packet receive (AF_XDP):        ~5 μs                        ║
║  Transaction processing (engine):       ~1-5 μs                      ║
║  WAL commit (FDP + DSA CRC):            ~10-12 μs                    ║
║  Replication (RDMA one-sided write):    ~2-4 μs                      ║
║  Client response (AF_XDP):              ~5 μs                        ║
║  ─────────────────────────────────────────────                       ║
║  TOTAL estimated replicated commit:     ~23-31 μs                    ║
╚══════════════════════════════════════════════════════════════════════╝
```

---

## 32. Interface Definitions (Complete Set) <a name="32-interfaces"></a>

```csharp
// ═══════════════════════════════════════════════════════════════
//  Every interface below has 2+ implementations:
//    1. Hardware-accelerated (Intel-specific or AMD-specific)
//    2. Software fallback (works everywhere)
//  The ProviderFactory selects ONE at boot. Hot path = just call.
// ═══════════════════════════════════════════════════════════════

/// <summary>Bulk data movement: copy, fill, compare, dualcast, delta</summary>
public interface IDataMover
{
    void Copy(ReadOnlySpan<byte> src, Span<byte> dst);
    void CopyWithCrc(ReadOnlySpan<byte> src, Span<byte> dst, out uint crc32c);
    void Fill(Span<byte> dst, ulong pattern);
    void Compare(ReadOnlySpan<byte> a, ReadOnlySpan<byte> b, out bool equal, out int firstDiff);
    void Dualcast(ReadOnlySpan<byte> src, Span<byte> dst1, Span<byte> dst2);
    void CreateDelta(ReadOnlySpan<byte> oldPage, ReadOnlySpan<byte> newPage, Span<byte> deltaOut, out int deltaSize);
    void ApplyDelta(ReadOnlySpan<byte> delta, Span<byte> targetPage);
}

/// <summary>WAL commit — the most latency-critical path in the entire engine</summary>
public interface IWalWriter
{
    /// <summary>Append a WAL record. Returns when durable (persisted).</summary>
    long Append(ReadOnlySpan<byte> record, out uint checksum);
    /// <summary>Append multiple records as a batch (for throughput).</summary>
    void AppendBatch(ReadOnlySpan<WalEntry> entries);
    /// <summary>Sync/flush if needed (no-op for PMR, fdatasync for NVMe).</summary>
    void Sync();
    /// <summary>Report current WAL position (for replication lag tracking).</summary>
    long CurrentPosition { get; }
}

/// <summary>Hardware-enforced tenant memory isolation</summary>
public interface ITenantIsolation
{
    int MaxTenants { get; }
    void AssignTenantKey(int tenantId);
    void RevokeTenantKey(int tenantId);
    void OnTenantContextSwitch(int fromTenantId, int toTenantId); // L1D flush if needed
    bool RequiresL1DFlush { get; } // false if RDCL_NO
}

/// <summary>Client-facing network transport (request/response)</summary>
public interface INetworkTransport : IDisposable
{
    ValueTask<int> ReceiveAsync(Memory<byte> buffer, CancellationToken ct);
    ValueTask SendAsync(ReadOnlyMemory<byte> buffer, CancellationToken ct);
    void SetFlowAffinity(int coreId); // pin this connection's traffic to a core
}

/// <summary>Replication transport (WAL log shipping)</summary>
public interface IReplicationTransport : IDisposable
{
    /// <summary>Ship WAL bytes to replica. Returns when remote-durable.</summary>
    ValueTask<bool> ReplicateAsync(ReadOnlyMemory<byte> walData, long walPosition);
    /// <summary>Latency of last replication round-trip.</summary>
    TimeSpan LastReplicationLatency { get; }
}

/// <summary>Buffer pool for data pages</summary>
public interface IBufferPool
{
    IntPtr AllocatePage(int numaNode);         // NUMA-aware allocation
    void FreePage(IntPtr page);
    long TotalBytes { get; }
    long UsedBytes { get; }
    BufferPoolTier GetPageTier(IntPtr page);   // Hot (DDR5), Warm (CXL), Cold (NVMe)
    void PromotePage(IntPtr page);             // Move cold → warm → hot
    void DemotePage(IntPtr page);              // Move hot → warm → cold
}

/// <summary>Distributed time source</summary>
public interface IClock
{
    long NowNanoseconds();                     // Monotonic, high-resolution
    long NowWithUncertainty(out long errorBoundNs); // For Spanner-style TrueTime
}

/// <summary>Checksum computation</summary>
public interface IChecksumProvider
{
    uint Crc32C(ReadOnlySpan<byte> data);
    ulong Crc64(ReadOnlySpan<byte> data);      // PCLMULQDQ path
}

/// <summary>Encryption at rest</summary>
public interface IEncryptionProvider
{
    void Encrypt(ReadOnlySpan<byte> plaintext, Span<byte> ciphertext, ReadOnlySpan<byte> key, ReadOnlySpan<byte> nonce);
    void Decrypt(ReadOnlySpan<byte> ciphertext, Span<byte> plaintext, ReadOnlySpan<byte> key, ReadOnlySpan<byte> nonce);
    bool KeysAreOpaque { get; }                // true for Key Locker (key never in software memory)
}

/// <summary>Cold data compression</summary>
public interface ICompressionProvider
{
    int Compress(ReadOnlySpan<byte> input, Span<byte> output);
    int Decompress(ReadOnlySpan<byte> compressed, Span<byte> output);
    string AlgorithmName { get; }              // "IAA-deflate", "zstd", "lz4"
}

/// <summary>Thread-to-core assignment</summary>
public interface ICoreSelector
{
    int SelectCoreForShardWriter(int shardId);
    int[] SelectCoresForReaders(int shardId, int readerCount);
    int GetNumaNodeForCore(int coreId);
    bool IsFavoredCore(int coreId);
}
```

---

## 33. Kubernetes Deployment Architecture <a name="33-kubernetes"></a>

### 33.1 Two-Tier Deployment Model

```
┌──────────────────────────────────────────────────────────────┐
│ TIER 1: "Performance" (Bare Metal K8s / Dedicated Nodes)      │
│                                                                │
│ • Dedicated worker nodes with VectorVolt-tuned BIOS + kernel  │
│ • Node labels: vectorvolt.io/role=dataplane                    │
│ • Node taints: vectorvolt.io/dedicated=true:NoSchedule         │
│ • BIOS: SMT off, C-states off, NPS4/SNC2, HugePages           │
│ • Kernel: isolcpus, nosmt, hugepages, max_cstate=1             │
│ • Device plugins: DSA, IAA, RDMA, SR-IOV, AF_XDP              │
│ • Full hardware: PMR, ZNS, DPDK, PTP                          │
│ • Pod: privileged caps, hostNetwork, Guaranteed QoS            │
│                                                                │
│ → Sub-10μs commit (full acceleration)                          │
│ → This is "bare metal performance with K8s orchestration"      │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ TIER 2: "Compatible" (Shared K8s / Cloud VMs / Dev/Test)      │
│                                                                │
│ • Standard K8s nodes, possibly VMs (EKS, GKE, AKS)            │
│ • CPU pinning via CPU Manager (if Guaranteed QoS)              │
│ • 2MB HugePages (if node supports)                             │
│ • Standard networking (kernel TCP, no RDMA/DPDK)               │
│ • All software fallbacks — still correct, just slower          │
│ • Pod: non-privileged, standard networking                     │
│                                                                │
│ → ~50-100μs commit (software path, still faster than most DBs) │
│ → This is "dev laptop experience" in a K8s pod                 │
└──────────────────────────────────────────────────────────────┘
```

### 33.2 Required K8s Device Plugins

| Plugin | Repo | What It Exposes | Resource Name |
|--------|------|----------------|---------------|
| Intel DSA | `github.com/intel/intel-device-plugins-for-kubernetes` | `/dev/dsa/wqX.Y` | `intel.com/dsa` |
| Intel IAA | Same repo | `/dev/iaa/wqX.Y` | `intel.com/iaa` |
| RDMA Shared | `github.com/Mellanox/k8s-rdma-shared-dev-plugin` | RDMA device contexts | `rdma/rdma_shared_device_a` |
| SR-IOV Network | `github.com/k8snetworkplumbingwg/sriov-network-device-plugin` | NIC VFs for DPDK | `intel.com/sriov_netdevice` |
| AF_XDP | `github.com/intel/afxdp-plugins-for-kubernetes` | AF_XDP sockets | `afxdp/myPool` |

### 33.3 K8s Kubelet Configuration

```yaml
# /var/lib/kubelet/config.yaml (on VectorVolt worker nodes)
apiVersion: kubelet.config.k8s.io/v1beta1
kind: KubeletConfiguration
cpuManagerPolicy: static              # REQUIRED — enables per-pod CPU pinning
cpuManagerPolicyOptions:
  full-pcpus-only: "true"             # Only allocate full physical cores
topologyManagerPolicy: single-numa-node  # Force all pod resources on 1 NUMA node
memoryManagerPolicy: Static           # NUMA-aware memory allocation
reservedSystemCPUs: "0-3"             # Reserve cores 0-3 for kubelet/system
featureGates:
  CPUManagerPolicyAlphaOptions: true
  MemoryManager: true
  TopologyManager: true
```

### 33.4 Worker Node Preparation

```bash
#!/bin/bash
# vectorvolt-node-prep.sh — run via DaemonSet or Ansible on VectorVolt worker nodes

# ── Kernel command line (requires reboot) ──
# Add to GRUB_CMDLINE_LINUX in /etc/default/grub:
# Intel:
#   intel_idle.max_cstate=1 nosmt hugepagesz=1G hugepages=128
#   default_hugepagesz=1G transparent_hugepage=never
#   isolcpus=4-31 nohz_full=4-31 rcu_nocbs=4-31
#   intel_iommu=on iommu=pt
#
# AMD:
#   amd_iommu=on iommu=pt nosmt hugepagesz=1G hugepages=128
#   default_hugepagesz=1G transparent_hugepage=never
#   isolcpus=4-31 nohz_full=4-31 rcu_nocbs=4-31

# ── Runtime tuning (no reboot needed) ──

# Disable THP
echo never > /sys/kernel/mm/transparent_hugepage/enabled
echo never > /sys/kernel/mm/transparent_hugepage/defrag

# CPU governor
echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Disable NUMA balancing (VectorVolt handles its own)
echo 0 > /proc/sys/kernel/numa_balancing

# Disable swappiness
echo 0 > /proc/sys/vm/swappiness

# Reserve 2MB huge pages per NUMA node (if 1GB not available)
# for node in /sys/devices/system/node/node*; do
#   echo 16384 > $node/hugepages/hugepages-2048kB/nr_hugepages
# done

# Label the node
kubectl label node $(hostname) vectorvolt.io/role=dataplane
kubectl taint node $(hostname) vectorvolt.io/dedicated=true:NoSchedule
```

### 33.5 Helm Chart Values (Full)

```yaml
# values.yaml — VectorVolt Helm Chart
replicaCount: 3                     # 3-node cluster for replication

image:
  repository: vectorvolt/engine
  tag: latest
  pullPolicy: IfNotPresent

deployment:
  tier: performance                 # "performance" or "compatible"

nodeSelector:
  vectorvolt.io/role: dataplane

tolerations:
  - key: vectorvolt.io/dedicated
    operator: Exists
    effect: NoSchedule

cpu:
  cores: 16                         # dedicated physical cores
  isolatedCores: true               # prefer isolcpus cores
  pinToNuma: true                   # single-NUMA-node topology

memory:
  request: "8Gi"
  limit: "8Gi"                      # must equal request for Guaranteed QoS
  hugepages:
    enabled: true
    size: "1Gi"                     # "1Gi" or "2Mi"
    count: 64                       # 64 × 1GB = 64GB buffer pool
  cxlTier:
    enabled: false                  # future: CXL warm page tier
    numaNode: 2                     # CXL NUMA node ID

storage:
  wal:
    device: "/dev/nvme0n1"          # dedicated NVMe for WAL
    pmr: auto                       # auto-detect PMR
    zns: auto                       # auto-detect ZNS
    fdp: auto                       # auto-detect FDP
    placementHandles:
      wal: 0
      index: 1
      data: 2
      temp: 3
  data:
    device: "/dev/nvme1n1"          # dedicated NVMe for data

network:
  mode: hostNetwork                 # "hostNetwork" | "sriov" | "standard"
  rdma:
    enabled: auto
    transport: rocev2
    device: mlx5_0
  dpdk:
    enabled: false                  # explicit opt-in
  afxdp:
    enabled: auto
  ptp:
    enabled: auto
    device: /dev/ptp0

security:
  tenantIsolation: auto             # "auto" | "mktme" | "sevsnp" | "process"
  confidentialCompute: false        # Kata + SEV-SNP/TDX
  tlsOffload: false                 # DPU-based TLS termination

accelerators:
  dsa:
    enabled: auto
    workQueues: 4                   # WQs per engine
  iaa:
    enabled: auto

observability:
  pmu:
    enabled: true                   # Performance counter monitoring
  mcaMonitoring:
    enabled: true                   # Memory health / DIMM failure prediction
  startupReport:
    enabled: true                   # Print full capability report at boot
    logLevel: info                  # "info" | "debug" (debug prints all CPUID/MSR values)
```

### 33.6 Pod Spec (Generated by Helm — Tier 1)

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: vectorvolt-engine-0
  labels:
    app: vectorvolt
    vectorvolt.io/tier: performance
  annotations:
    k8s.v1.cni.cncf.io/networks: sriov-rdma  # Multus secondary network for RDMA
spec:
  hostNetwork: true
  dnsPolicy: ClusterFirstWithHostNet
  serviceAccountName: vectorvolt

  nodeSelector:
    vectorvolt.io/role: dataplane

  tolerations:
    - key: vectorvolt.io/dedicated
      operator: Exists
      effect: NoSchedule

  containers:
    - name: vectorvolt
      image: vectorvolt/engine:latest
      command: ["./vectorvolt", "--config", "/etc/vectorvolt/config.yaml"]

      securityContext:
        privileged: false
        capabilities:
          add:
            - IPC_LOCK              # RDMA memory registration (ibv_reg_mr)
            - SYS_NICE              # sched_setaffinity for core pinning
            - NET_ADMIN             # PTP, AF_XDP, ethtool
            - SYS_RAWIO             # MSR reads for MKTME/PMU (optional)
            - NET_RAW               # Raw sockets for DPDK fallback

      resources:
        requests:
          cpu: "16"                 # MUST equal limits → Guaranteed QoS → CPU Manager
          memory: "8Gi"
          hugepages-1Gi: "64Gi"
          intel.com/dsa: "1"        # Intel DSA device
          intel.com/iaa: "1"        # Intel IAA device
          rdma/rdma_shared_device_a: "1"  # RDMA NIC context
        limits:
          cpu: "16"
          memory: "8Gi"
          hugepages-1Gi: "64Gi"
          intel.com/dsa: "1"
          intel.com/iaa: "1"
          rdma/rdma_shared_device_a: "1"

      env:
        - name: VECTORVOLT_DEPLOYMENT
          value: "kubernetes"
        - name: VECTORVOLT_TIER
          value: "performance"
        - name: VECTORVOLT_NODE_NAME
          valueFrom:
            fieldRef:
              fieldPath: spec.nodeName

      volumeMounts:
        - name: hugepages
          mountPath: /dev/hugepages
        - name: wal-device
          mountPath: /dev/vectorvolt-wal
        - name: data-device
          mountPath: /dev/vectorvolt-data
        - name: ptp-clock
          mountPath: /dev/ptp0
        - name: config
          mountPath: /etc/vectorvolt
        - name: sys-cpu
          mountPath: /sys/devices/system/cpu
          readOnly: true
        - name: sys-node
          mountPath: /sys/devices/system/node
          readOnly: true

  volumes:
    - name: hugepages
      emptyDir:
        medium: HugePages-1Gi
    - name: wal-device
      hostPath:
        path: /dev/nvme0n1
        type: BlockDevice
    - name: data-device
      hostPath:
        path: /dev/nvme1n1
        type: BlockDevice
    - name: ptp-clock
      hostPath:
        path: /dev/ptp0
        type: CharDevice
    - name: config
      configMap:
        name: vectorvolt-config
    - name: sys-cpu
      hostPath:
        path: /sys/devices/system/cpu
    - name: sys-node
      hostPath:
        path: /sys/devices/system/node
```

### 33.7 Pod Spec (Tier 2 — Compatible / Dev)

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: vectorvolt-engine-0
  labels:
    app: vectorvolt
    vectorvolt.io/tier: compatible
spec:
  # NO hostNetwork — uses standard CNI
  # NO privileged — non-privileged container
  # NO device plugins — all software fallbacks

  containers:
    - name: vectorvolt
      image: vectorvolt/engine:latest

      resources:
        requests:
          cpu: "4"
          memory: "4Gi"
          hugepages-2Mi: "2Gi"      # 2MB pages if available, otherwise engine uses 4KB
        limits:
          cpu: "4"                   # triggers CPU pinning if CPU Manager is static
          memory: "4Gi"
          hugepages-2Mi: "2Gi"

      volumeMounts:
        - name: hugepages
          mountPath: /dev/hugepages
        - name: wal-pvc
          mountPath: /var/vectorvolt/wal
        - name: data-pvc
          mountPath: /var/vectorvolt/data

  volumes:
    - name: hugepages
      emptyDir:
        medium: HugePages-2Mi
    - name: wal-pvc
      persistentVolumeClaim:
        claimName: vectorvolt-wal
    - name: data-pvc
      persistentVolumeClaim:
        claimName: vectorvolt-data
```

### 33.8 K8s Auto-Detection Logic in PlatformProber

```csharp
// VectorVolt automatically detects K8s constraints and adapts
public static class K8sProber
{
    public static void Probe(SiliconManifestBuilder b)
    {
        b.IsKubernetes = Environment.GetEnvironmentVariable("KUBERNETES_SERVICE_HOST") != null;

        if (!b.IsKubernetes) return;

        // ── CPU budget from cgroup ──
        // cgroup v2: /sys/fs/cgroup/cpu.max → "quota period"
        // cgroup v1: /sys/fs/cgroup/cpu/cpu.cfs_quota_us + period_us
        string cpuMax = File.ReadAllText("/sys/fs/cgroup/cpu.max").Trim();
        if (cpuMax != "max") {
            var parts = cpuMax.Split(' ');
            b.CpuBudget = (int)(long.Parse(parts[0]) / long.Parse(parts[1]));
        }

        // ── Memory budget from cgroup ──
        string memMax = File.ReadAllText("/sys/fs/cgroup/memory.max").Trim();
        if (memMax != "max") {
            b.MemoryBudgetBytes = long.Parse(memMax);
        }

        // ── Detect which devices are mounted (K8s device plugins) ──
        b.HasDsa  = Directory.Exists("/dev/dsa")  || File.Exists("/dev/dsa/wq0.0");
        b.HasIaa  = Directory.Exists("/dev/iaa")  || File.Exists("/dev/iaa/wq0.0");
        // RDMA: check /dev/infiniband/uverbs0
        b.RdmaAvailable = File.Exists("/dev/infiniband/uverbs0");
        // PTP: check if /dev/ptp0 is mounted
        b.PtpHardwareAvailable = File.Exists("/dev/ptp0");

        // ── Detect Guaranteed QoS (cpu requests == limits → CPU pinning active) ──
        // If cpuset is restricted (not all CPUs), CPU Manager is pinning us
        string cpuset = File.ReadAllText("/sys/fs/cgroup/cpuset.cpus.effective").Trim();
        // Parse cpuset to know exactly which cores we're pinned to
    }
}
```

---

## 34. On-Premises Bare Metal Deployment <a name="34-onprem"></a>

### 34.1 Reference Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    VectorVolt On-Prem Cluster                    │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  Node 1     │  │  Node 2     │  │  Node 3     │             │
│  │  (Primary)  │  │  (Replica)  │  │  (Replica)  │             │
│  │             │  │             │  │             │             │
│  │ EPYC 9554   │  │ EPYC 9554   │  │ EPYC 9554   │             │
│  │ 64 cores    │  │ 64 cores    │  │ 64 cores    │             │
│  │ 512GB DDR5  │  │ 512GB DDR5  │  │ 512GB DDR5  │             │
│  │ NPS4 (4 NU) │  │ NPS4 (4 NU) │  │ NPS4 (4 NU) │             │
│  │             │  │             │  │             │             │
│  │ NVMe WAL:   │  │ NVMe WAL:   │  │ NVMe WAL:   │             │
│  │  PM9D3 FDP  │  │  PM9D3 FDP  │  │  PM9D3 FDP  │             │
│  │ NVMe Data:  │  │ NVMe Data:  │  │ NVMe Data:  │             │
│  │  PM9D3 ×4   │  │  PM9D3 ×4   │  │  PM9D3 ×4   │             │
│  │             │  │             │  │             │             │
│  │ ConnectX-7  │──│ ConnectX-7  │──│ ConnectX-7  │             │
│  │ 2×100G RDMA │  │ 2×100G RDMA │  │ 2×100G RDMA │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│         │                │                │                      │
│         └────── RoCEv2 Fabric (PFC + ECN enabled) ──────┘       │
│                          │                                       │
│                    ┌─────┴─────┐                                │
│                    │ PTP Grand │  GNSS-referenced               │
│                    │ Master    │  sub-50ns accuracy              │
│                    └───────────┘                                │
└─────────────────────────────────────────────────────────────────┘
```

### 34.2 BIOS Configuration Checklist

#### Intel Xeon (Sapphire Rapids / Emerald Rapids / Granite Rapids)

```
Processor Configuration:
  ☑ Hyper-Threading:           Disabled
  ☑ Hardware Prefetcher:       Enabled
  ☑ Adjacent Cache Prefetch:   Disabled
  ☑ DCU Streamer Prefetcher:   Enabled
  ☑ DCU IP Prefetcher:         Enabled
  ☑ LLC Prefetch:              Disabled
  ☑ Intel Turbo Boost:         Enabled
  ☑ TBMT 3.0:                  Enabled
  ☑ Energy Efficient Turbo:    Disabled
  ☑ Sub-NUMA Clustering:       SNC2

Power Management:
  ☑ Power Technology:          Custom
  ☑ C-States:                  C1 only (disable C6/C7/C8/C10)
  ☑ Package C-State Limit:     C0/C1
  ☑ Hardware P-States:         Native Mode
  ☑ Energy Performance Bias:   Performance

Memory Configuration:
  ☑ Patrol Scrub:              Disabled
  ☑ Memory Frequency:          Auto (max DDR5 speed)
  ☑ Populate all 8 channels:   1DPC for max frequency

PCIe / Devices:
  ☑ Intel VT-d:                Enabled (for DSA PASID)
  ☑ ATS:                       Enabled (for DSA SVM)
  ☑ SR-IOV:                    Enabled (for NIC)
  ☑ PCIe Bifurcation:          As needed (x4x4x4x4 for NVMe riser)

Security:
  ☑ TME:                       Enabled
  ☑ TME-MK:                    Enabled
  ☑ Key Locker:                Enabled
  ☑ SGX:                       Disabled (VectorVolt uses MKTME, not SGX)
```

#### AMD EPYC 9005 (Turin)

```
Advanced → AMD CBS → CPU Common Options:
  ☑ SMEE:                      Enabled
  ☑ SEV Control:               Enabled
  ☑ SEV-ES ASID Space Limit:   Configure per tenant count
  ☑ SNP Memory Coverage:       Enabled
  ☑ SMT Control:               Disabled
  ☑ Global C-State Control:    Disabled
  ☑ Core Performance Boost:    Enabled
  ☑ CPPC:                      Enabled
  ☑ CPPC Preferred Cores:      Enabled
  ☑ L1 Stream HW Prefetcher:   Enabled
  ☑ L2 Stream HW Prefetcher:   Disabled

Advanced → AMD CBS → UMC Common Options:
  ☑ Patrol Scrub:              Disabled
  ☑ TSME:                      Disabled (use SEV-SNP instead)
  ☑ Memory Interleaving:       Channel

Advanced → AMD CBS → DF Common Options:
  ☑ NUMA Nodes Per Socket:     NPS4
  ☑ L3 As NUMA Domain:         Enabled (optional)

Advanced → AMD CBS → NBIO Common Options:
  ☑ IOMMU:                     Enabled
  ☑ ACS Enable:                As needed
  ☑ PCIe ARI Support:          Enabled

Advanced → AMD Overclocking:
  ☑ PPT Limit:                 Platform max
  ☑ TDC Limit:                 Platform max
  ☑ EDC Limit:                 Platform max
```

### 34.3 OS / Kernel Configuration

```bash
# /etc/default/grub — GRUB_CMDLINE_LINUX (reboot required)

# Intel:
GRUB_CMDLINE_LINUX="intel_idle.max_cstate=1 nosmt \
  hugepagesz=1G hugepages=128 default_hugepagesz=1G \
  transparent_hugepage=never \
  isolcpus=4-63 nohz_full=4-63 rcu_nocbs=4-63 \
  intel_iommu=on iommu=pt \
  skew_tick=1 tsc=reliable clocksource=tsc"

# AMD:
GRUB_CMDLINE_LINUX="amd_iommu=on iommu=pt nosmt \
  hugepagesz=1G hugepages=128 default_hugepagesz=1G \
  transparent_hugepage=never \
  isolcpus=4-63 nohz_full=4-63 rcu_nocbs=4-63 \
  skew_tick=1 tsc=reliable clocksource=tsc"

# ── Sysctl (/etc/sysctl.d/99-vectorvolt.conf) ──
vm.swappiness = 0
vm.zone_reclaim_mode = 0
vm.dirty_ratio = 40
vm.dirty_background_ratio = 10
kernel.numa_balancing = 0
kernel.sched_rt_runtime_us = -1
net.core.busy_read = 50
net.core.busy_poll = 50
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
```

### 34.4 Systemd Service

```ini
# /etc/systemd/system/vectorvolt.service
[Unit]
Description=VectorVolt Data Engine
After=network.target
Requires=sys-devices-system-node.mount

[Service]
Type=simple
ExecStartPre=/usr/local/bin/vectorvolt-preflight-check
ExecStart=/usr/local/bin/vectorvolt --config /etc/vectorvolt/config.yaml
Restart=on-failure
RestartSec=5

# Resource isolation
CPUAffinity=4-63
LimitMEMLOCK=infinity
LimitNOFILE=1048576
LimitNPROC=65536

# Capabilities (instead of running as root)
AmbientCapabilities=CAP_IPC_LOCK CAP_SYS_NICE CAP_NET_ADMIN CAP_SYS_RAWIO CAP_NET_RAW
NoNewPrivileges=true

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=vectorvolt

[Install]
WantedBy=multi-user.target
```

---

## 35. Complete New Classes Summary (All Parts) <a name="35-all-classes"></a>

### Runtime Architecture (Part 3 — NEW)
```
SiliconManifest             — Immutable capability struct, frozen after boot
SiliconManifestBuilder      — Mutable builder used during probe phase only
CpuProber                   — CPUID + MSR reads for CPU features
TopologyProber              — CPUID 0x1F + SLIT distances + NUMA enumeration
StorageProber               — NVMe BAR0 register reads + CXL DVSEC enumeration
NetworkProber               — ibv_devices + ethtool + sysfs for NIC capabilities
PlatformProber              — HugePages + THP + C-state + governor + cgroup checks
K8sProber                   — Kubernetes-specific: cgroup limits, device mounts
ProviderFactory             — Reads manifest → creates concrete impl per interface
StartupReporter             — Prints formatted capability report + warnings
PlatformTuningValidator     — Checks BIOS/OS config, emits warnings for suboptimal
```

### Interfaces (Part 3 — NEW)
```
IDataMover                  — Copy, CopyWithCrc, Fill, Compare, Dualcast, Delta
IWalWriter                  — Append, AppendBatch, Sync, CurrentPosition
ITenantIsolation            — AssignKey, RevokeKey, OnContextSwitch
INetworkTransport           — Receive, Send, SetFlowAffinity
IReplicationTransport       — Replicate, LastReplicationLatency
IBufferPool                 — AllocatePage, FreePage, Promote/Demote tier
IClock                      — NowNanoseconds, NowWithUncertainty
IChecksumProvider           — Crc32C, Crc64
IEncryptionProvider         — Encrypt, Decrypt, KeysAreOpaque
ICompressionProvider        — Compress, Decompress, AlgorithmName
ICoreSelector               — SelectCoreForShardWriter, SelectCoresForReaders
```

### Concrete Implementations Per Interface
```
IDataMover:          DsaDataMover → Avx512DataMover → Avx2DataMover → ScalarDataMover
IWalWriter:          PmrDsaWalWriter → PmrWalWriter → CxlPmemWalWriter → ZnsDsaWalWriter
                     → ZnsWalWriter → FdpWalWriter → DsaWalWriter → StandardWalWriter
ITenantIsolation:    MktmeIsolation → SevSnpIsolation → ProcessIsolation
INetworkTransport:   DpdkTransport → AfXdpTransport → SocketTransport
IReplicationTransport: RdmaReplicator → TcpReplicator
IBufferPool:         TieredHugePageBufferPool → HugePageBufferPool → StandardBufferPool
IClock:              PtpHardwareClock → MonotonicSystemClock
IChecksumProvider:   DsaCrcProvider → HardwareCrcProvider → Sse42CrcProvider → SoftwareCrcProvider
IEncryptionProvider: KeyLockerAesGcm → AesNiGcmProvider → SoftwareAesProvider
ICompressionProvider: IaaCompressor → SoftwareDeflateCompressor
ICoreSelector:       FavoredCoreSelector → NumaAwareCoreSelector → SimpleCoreSelector
```

**Total new class count (all 3 parts combined): ~80+ classes**
- Part 1 (CPU silicon): ~30 classes
- Part 2 (NVMe + Network + Platform): ~23 classes
- Part 3 (Runtime architecture): ~30 classes (probers + interfaces + implementations)

---

## Source References (All Parts)

| Category | Key Sources |
|----------|------------|
| **Part 1 — CPU Silicon** | |
| Intel DSA | 341204-006-intel-data-streaming-accelerator-spec.pdf (288pp) |
| MKTME | 336907-006-multi-key-total-memory-encryption-spec.pdf (24pp) |
| Topology | intel-64-architecture-processor-topology-enumeration.pdf (30pp) |
| PCLMULQDQ | clmul-wp-rev-2-02-2014-04-20.pdf (84pp) |
| FRED | 346446-009-flexible-return-and-event-delivery.pdf (62pp) |
| Spectre | 336996-speculative-execution-side-channel-mitigations.pdf (23pp) |
| 5-Level Paging | 5-level-paging-white-paper.pdf (31pp) |
| Key Locker | 343965-intel-key-locker-specification.pdf (65pp) |
| IAA | ZSWAPwithIAA_1.0.pdf (10pp) |
| Scalable IOV | intel-scalable-io-virtualization-technical-specification.pdf (29pp) |
| BFloat16 | bf16-hardware-numerics-definition-white-paper.pdf (7pp) |
| PMU | pmu-sharing-guidelines.pdf (13pp) |
| MCA | 329176-mca-enhancements-in-intel-xeon-processors.pdf (14pp) |
| Microcode | 356111-003-runtime-update-tech-paper.pdf (37pp) |
| CPUID Passthrough | 356709-003-intel-cpuid-passthrough.pdf (10pp) |
| Intel SDM | 253669-091-sdm-vol-3b.pdf (488pp) |
| **Part 1 — AMD Silicon** | |
| EPYC 9005 | 58462_amd-epyc-9005-tg-architecture-overview.pdf (11pp) |
| SME/SEV | memory-encryption-white-paper.pdf (12pp) |
| NUMA Milan | 56795_1.00_NUMA_Milan_PUB.pdf (18pp) |
| SEV Guide | 2025_AMD_Whitepaper_SEV_Enablement_Guide.pdf (52pp) |
| AMD APM Vol 2 | amd-manual.pdf (782pp) |
| **Part 2 — Beyond CPU** | |
| NVMe | NVMe Base Specification 2.0, TP4053 (ZNS), TP4146 (FDP) |
| CXL | CXL Specification 2.0/3.0 |
| RDMA | rdma-core, NVIDIA RDMA Programming Guide |
| DPDK | dpdk.org, Intel E810/E830 driver guides |
| PTP | IEEE 1588-2019, Linux kernel timestamping docs |
| Platform | Intel Xeon Tuning Guide, AMD EPYC Tuning Guide |
| **Part 3 — Runtime Architecture** | |
| K8s CPU Manager | kubernetes.io/docs/tasks/administer-cluster/cpu-management-policies |
| K8s Topology Manager | kubernetes.io/docs/tasks/administer-cluster/topology-manager |
| K8s Device Plugins | github.com/intel/intel-device-plugins-for-kubernetes |
| K8s RDMA Plugin | github.com/Mellanox/k8s-rdma-shared-dev-plugin |
| K8s SR-IOV Plugin | github.com/k8snetworkplumbingwg/sriov-network-device-plugin |

---

*Complete handover generated across 3 research sessions on 2026-04-14. Covers CPU silicon (Intel + AMD), NVMe storage, networking/NIC, platform tuning, runtime probe-and-dispatch architecture, Kubernetes deployment, and on-prem bare metal deployment.*
