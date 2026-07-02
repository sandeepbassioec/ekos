# AMD Silicon Research Handover — VectorVolt Integration Guide

> **Purpose:** Companion document to `INTEL_SILICON_RESEARCH_HANDOVER.md`. Maps AMD EPYC architecture to VectorVolt's 6-phase architecture. Covers NUMA topology (Naples→Rome→Milan→Genoa→Turin), SME/SEV/SEV-SNP memory encryption, and Zen 5 core architecture.

> **Date:** 2026-04-14

---

## Table of Contents

1. [Document Inventory & Relevance](#1-inventory)
2. [AMD EPYC 9005 (Turin/Zen 5) Architecture](#2-zen5)
3. [AMD NUMA Topology — Evolution & VectorVolt Mapping](#3-numa)
4. [AMD Memory Encryption — SME/SEV/SEV-ES/SEV-SNP](#4-sme-sev)
5. [Intel vs AMD Feature Comparison Matrix](#5-comparison)
6. [VectorVolt HAL — Vendor-Agnostic Abstraction Design](#6-hal-abstraction)
7. [Phase-by-Phase AMD Integration](#7-phase-matrix)
8. [C#/.NET Integration Patterns (AMD-specific)](#8-csharp)

---

## 1. Document Inventory & Relevance <a name="1-inventory"></a>

### Directly Relevant (TIER 1-2)

| File | Document | Pages | Relevance |
|------|----------|-------|-----------|
| `58462_amd-epyc-9005-tg-architecture-overview.pdf` | **EPYC 9005 (Turin/Zen 5) Architecture** | 11 | TIER 1 — Core architecture, CCX/CCD/IOD layout, NUMA |
| `memory-encryption-white-paper.pdf` | **AMD Memory Encryption (SME/SEV)** | 12 | TIER 1 — SME C-bit, SEV per-VM keys, key management |
| `56795_1.00_NUMA_Milan_PUB.pdf` | **NUMA Topology — Milan (Zen 3)** | 18 | TIER 1 — NPS settings, SLIT distances, L3AsNUMA |
| `56308_0.70_NUMA_Naples.pdf` | **NUMA Topology — Naples (Zen 1)** | 7 | TIER 2 — Original NUMA layout |
| `dell-emc-dfd-numa-amd-epyc-2ndgen.pdf` | **Dell NUMA Guide — Rome (Zen 2)** | 3 | TIER 2 — NPS recommendations per workload |
| `2025_AMD_Whitepaper_SEV_Enablement_Guide.pdf` | **SEV Enablement Guide (all EPYC)** | 52 | TIER 1 — BIOS config, MSR addresses, ASID management |
| `58478_amd-epyc-9005-tg-containers-on-kubernetes.pdf` | **K8s Tuning Guide — EPYC 9005** | 20 | TIER 2 — CPU pinning, NUMA alignment, NRI plugin |
| `amd-manual.pdf` (APM Vol 2) | **AMD64 System Programming** | 782 | TIER 1 — MSRs, page tables, MCA, perf monitoring |
| `lp1894.pdf` | **Lenovo SME/SEV/SEV-ES Guide** | 16 | TIER 2 — Practical enablement |
| `lp1893.pdf` | **Lenovo SEV-SNP Guide** | 13 | TIER 2 — SEV-SNP enablement |

### Reference Only (TIER 3)

| File | Document | Notes |
|------|----------|-------|
| `24592_3.24.pdf` | APM Vol 1 — Application Programming | 394 pages, general ISA |
| `24594_3.37.pdf` | APM Vol 3 — Instructions | 712 pages, instruction reference |
| `26569_APM_v5-r3.13.pdf` | APM Vol 5 — x87/Media Instructions | 372 pages |
| `40332-r4.07.pdf` | APM Vols 1-5 Combined | 3336 pages, superset |
| `using-amd-sev-es-on-thinkystem.pdf` | Lenovo SEV-ES (vSphere) | Practical guide |

### Not Relevant to VectorVolt

| File | Why Not Relevant |
|------|-----------------|
| `AI_Engine_Kernel_Graph_Programming_Guide.pdf` | Xilinx/Versal AI Engine — FPGA, not CPU |
| `Alveo_X3522_User_Guide.pdf` | Xilinx Alveo SmartNIC — FPGA accelerator |
| `AOCLUserGuide.pdf` | Empty (0 bytes) |
| `DynamicFunctionExchange.pdf` | Empty (0 bytes) — Xilinx partial reconfiguration |
| `OnLoadUserGuide.pdf` | Empty (0 bytes) |
| `VivadoDesignSuiteUserGuide.pdf` | Empty (0 bytes) — Xilinx FPGA design tool |
| `uProf.pdf` | Empty (0 bytes) — AMD profiling tool |
| `blog-main.zip` | Not a PDF |

---

## 2. AMD EPYC 9005 (Turin/Zen 5) Architecture <a name="2-zen5"></a>

### 2.1 Chiplet Architecture

```
Package (Socket SP5)
├── I/O Die (IOD) — Central hub via AMD Infinity Fabric
│   ├── 12 Unified Memory Controllers (UMCs) — DDR5 @ 6000 MT/s
│   ├── 4 P-links (PCIe Gen5 / CXL 2.0)
│   ├── 4 G-links (xGMI for 2P interconnect, 32Gbps each)
│   └── Cache coherency management
│
├── Up to 16 CCDs (Core Complex Dies) — Zen 5 Classic
│   └── 1 CCX per CCD
│       ├── Up to 8 Zen 5 cores
│       ├── 32 MB shared L3 cache
│       └── 2 SMT threads per core (16 threads per CCX)
│
└── OR up to 12 CCDs — Zen 5c Dense
    └── 1 CCX per CCD
        ├── Up to 16 Zen 5c cores
        ├── 32 MB shared L3 cache
        └── 2 SMT threads per core (32 threads per CCX)
```

### 2.2 Key Specs — EPYC 9005

| Feature | Zen 5 Classic | Zen 5c Dense |
|---------|--------------|--------------|
| Max cores | 128 (256 threads) | 192 (384 threads) |
| Cores per CCX | Up to 8 | Up to 16 |
| L3 cache per CCX | 32 MB | 32 MB |
| CCDs | Up to 16 | Up to 12 |
| CCDs per quadrant | Up to 4 | Up to 3 |
| Memory channels | 12 DDR5 | 12 DDR5 |
| Max memory speed | 6000 MT/s | 6000 MT/s |
| Max memory capacity | 8 TB (6TB DDR5 + 2TB CXL) | Same |
| PCIe | 128 lanes Gen 5 | Same |
| CXL | 64 lanes CXL 2.0 | Same |
| Socket | SP5 | SP5 |
| CPUID Family | 1Ah | 1Ah |

### 2.3 Connectivity (Dual Socket)

- 4 xGMI links @ 32 Gbps each between sockets
- 64 PCIe lanes per socket in 2P (128 total)
- Optional: 3 xGMI links → 80 PCIe lanes per socket (160 total)

### 2.4 Key Difference from Intel for VectorVolt

**AMD uses a chiplet (MCM) architecture** — this means:
- **Each CCD connects to the IOD via a dedicated GMI link** — there's a fabric hop for every cross-CCD communication
- **Memory is behind the IOD** — even local memory goes through the fabric
- **L3 cache is per-CCX, not shared across CCDs** — unlike Intel where multiple cores can share a large LLC
- **NUMA effects are MORE pronounced** than on Intel monolithic dies

**VectorVolt implication:** Core pinning is EVEN MORE CRITICAL on AMD. A shard writer MUST have its memory allocated on the same NUMA node as the CCD it runs on. Cross-CCD access adds significant latency.

---

## 3. AMD NUMA Topology — Evolution & VectorVolt Mapping <a name="3-numa"></a>

### 3.1 NUMA Topology Evolution

| Generation | Codename | Dies/Socket | CCXs/Die | Cores/CCX | Memory Channels | NUMA Nodes (NPS4) |
|-----------|----------|-------------|----------|-----------|----------------|-------------------|
| EPYC 7001 | Naples | 4 | 2 | 4 | 8 (2/die) | 4 (1/die) |
| EPYC 7002 | Rome | 1 IOD + 8 CCD | 2/CCD | 4 | 8 (via IOD) | 4 (1/quadrant) |
| EPYC 7003 | Milan | 1 IOD + 8 CCD | 1/CCD | 8 | 8 (via IOD) | 4 (1/quadrant) |
| EPYC 9004 | Genoa | 1 IOD + 12 CCD | 1/CCD | 8 | 12 (via IOD) | 4 (1/quadrant) |
| EPYC 9005 | Turin | 1 IOD + 16 CCD | 1/CCD | 8 or 16 | 12 (via IOD) | 4 (1/quadrant) |

### 3.2 NPS (Nodes Per Socket) Settings

This is the **single most important BIOS setting** for VectorVolt on AMD.

| NPS Setting | NUMA Nodes/Socket | Memory Interleaving | VectorVolt Recommendation |
|------------|-------------------|--------------------|-----------------------------|
| **NPS4** | 4 (1 per quadrant) | 2-channel (per quadrant) | **RECOMMENDED for VectorVolt** — finest granularity, lowest cross-NUMA penalty |
| NPS2 | 2 (1 per half) | 4-channel (per half) | Acceptable if workload needs more interleaving |
| NPS1 | 1 (whole socket) | 8/12-channel (all) | NOT recommended — hides NUMA, increases average latency |
| NPS0 | 1 (whole system, 2P only) | 16/24-channel (all) | NEVER for VectorVolt |

### 3.3 SLIT Distances (System Locality Information Table)

**Milan NPS4 (2P system):**
```
                S0,BR  S0,TR  S0,TL  S0,BL  S1,BR  S1,TR  S1,TL  S1,BL
S0,BR             10     12     12     12   20/32  20/32  20/32  20/32
S0,TR             12     10     12     12   20/32  20/32  20/32  20/32
S0,TL             12     12     10     12   20/32  20/32  20/32  20/32
S0,BL             12     12     12     10   20/32  20/32  20/32  20/32
S1,BR           20/32  20/32  20/32  20/32    10     12     12     12
S1,TR           20/32  20/32  20/32  20/32    12     10     12     12
S1,TL           20/32  20/32  20/32  20/32    12     12     10     12
S1,BL           20/32  20/32  20/32  20/32    12     12     12     10
```

**Distances explained:**
- **10** = Local (same quadrant)
- **12** = Near-hop (same socket, different quadrant, 1-hop Infinity Fabric, +2)
- **20** = Far-hop (different socket, 1-hop xGMI, +10) — default
- **32** = Far-hop with PinNodesToSocket (forces OS to reclaim locally before remote)

**VectorVolt implication:**
- Cross-quadrant penalty = +20% (12 vs 10)
- Cross-socket penalty = +100-220% (20-32 vs 10)
- **ALWAYS allocate shard memory on the same NUMA node as the shard writer core**

### 3.4 L3AsNUMA — Per-CCX NUMA Nodes

When enabled, each CCX (= each L3 cache) becomes its own NUMA node.

**Milan 1P, NPS4, L3AsNUMA Enabled — 8 NUMA nodes:**
```
     0    1    2    3    4    5    6    7
0   10   11   12   12   12   12   12   12
1   11   10   12   12   12   12   12   12
2   12   12   10   11   12   12   12   12
3   12   12   11   10   12   12   12   12
4   12   12   12   12   10   11   12   12
5   12   12   12   12   11   10   12   12
6   12   12   12   12   12   12   10   11
7   12   12   12   12   12   12   11   10
```

- **11** = Same partition, different CCX (shares memory controllers but different L3)

**VectorVolt recommendation:**
- Enable L3AsNUMA for finest-grained control
- Pin each shard writer to a single CCX
- Allocate shard memory to the CCX's NUMA node
- Reader threads on the SAME CCX (distance 10-11) — they share L3

### 3.5 EPYC 9005 (Turin) NUMA

Turin with Zen 5 Classic (128 cores):
- 16 CCDs, 4 quadrants, 4 CCDs per quadrant
- Each quadrant has 3 UMCs (memory controllers)
- NPS4 → 4 NUMA nodes, each with 32 cores + 3 memory channels

Turin with Zen 5c Dense (192 cores):
- 12 CCDs, 4 quadrants, 3 CCDs per quadrant
- NPS4 → 4 NUMA nodes, each with 48 cores + 3 memory channels

**Wide GMI option:** Some OPNs connect each CCD to 2 GMI interfaces for double core-to-IOD bandwidth (trades CCD count for bandwidth).

---

## 4. AMD Memory Encryption — SME/SEV/SEV-ES/SEV-SNP <a name="4-sme-sev"></a>

### 4.1 Technology Stack

```
Layer 4: SEV 4.1 (Turin)       — Trusted I/O (TIO) — DMA from device directly into encrypted VM
Layer 3: SEV 3.0/3.1 (Milan+)  — SEV-SNP — Secure Nested Paging, memory integrity, attestation
Layer 2: SEV 2.0 (Rome)        — SEV-ES — Encrypted State (CPU registers encrypted on VMEXIT)
Layer 1: SEV 1.0 (Naples)      — SEV — Per-VM memory encryption keys
Layer 0: SME/TSME              — Secure Memory Encryption (single key, all memory)
```

### 4.2 SME (Secure Memory Encryption)

**What it is:** AES-128 encryption of DRAM, controlled per-page via the **C-bit** (bit 47 of physical address in page table entry).

**Key details:**
- Encryption key generated randomly by AMD Secure Processor (ARM Cortex-A5 embedded in SoC)
- Key NEVER exposed to software — stored in hardware registers inside SoC
- AES engine in each memory controller — encrypts on write, decrypts on read
- C-bit = 1 in PTE → page encrypted; C-bit = 0 → page unencrypted
- Physical address-based tweak protects against cipher-text block move attacks

**MSR: MSRC001_0010 (SYS_CFG)**
```
Bit 23 = SMEE (Secure Memory Encryption Enable) — Write-1-Only, cleared on reset
```

**CPUID Detection:**
```
CPUID Fn8000_001F:EAX[0] = 1  → SME supported
```

**TSME (Transparent SME):** Encrypts ALL memory regardless of C-bit — no OS support needed, enabled in BIOS only.

### 4.3 SEV (Secure Encrypted Virtualization)

**What it is:** Each VM gets its own AES encryption key. VM memory is encrypted with its unique key, providing cryptographic isolation between VMs and between VMs and hypervisor.

**Key details:**
- Each VM tagged with an ASID (Address Space ID) — used to select encryption key
- Keys managed by AMD Secure Processor firmware — hypervisor NEVER sees keys
- Guest can mark pages as private (encrypted) or shared (unencrypted for HV communication)
- Instruction pages and page tables MUST be private (hardware enforced)
- DMA into encrypted guest memory NOT allowed — must use shared pages or bounce buffers
- Multi-core guests supported — same ASID for all vCPUs of a guest

**ASID capacity by generation:**
```
EPYC 7001 (Naples): 16 keys (SEV only)
EPYC 7002 (Rome):   509 keys (SEV) or 253 keys (SEV+ES)
EPYC 7003 (Milan):  509 keys (SEV) or configurable ES/SNP split
EPYC 9004 (Genoa):  1006 keys
EPYC 9005 (Turin):  1006 keys
```

**Physical address impact of SEV:**
```
EPYC 9005: 52-bit PA without SEV → 46-bit with SEV (1006 keys)
EPYC 7003: 48-bit PA without SEV → 43-bit with SEV (509 keys)
```

### 4.4 SEV-ES (Encrypted State)

Extends SEV to encrypt CPU register state on VMEXIT:
- Hypervisor cannot read guest register contents
- Detects malicious register modifications
- Uses VMCB (Virtual Machine Control Block) for encrypted register save area

### 4.5 SEV-SNP (Secure Nested Paging)

**The strongest protection — recommended for VectorVolt multi-tenancy.**

Adds hardware memory integrity:
- **Reverse Map Table (RMP):** Hardware-enforced page ownership tracking
- **Prevents:** Replay attacks, data corruption, memory aliasing, memory re-mapping
- **Attestation:** Guest owner can verify the platform is genuine AMD with SEV-SNP
- **TCB rollback protection:** Cannot revert AMD-SP firmware to old version

**Threat mitigation:**
| Threat | Mitigated? |
|--------|-----------|
| HV reads private VM memory | Yes |
| HV reads VM register state | Yes |
| Device DMA to VM memory | Yes |
| Replay attack (replace with old copy) | Yes |
| Data corruption (replace with junk) | Yes |
| Memory aliasing (two guest pages → same DRAM) | Yes |
| Memory re-mapping (swap DRAM pages) | Yes |
| Cold boot / offline DRAM analysis | Yes |
| TCB rollback | Yes |
| Malicious interrupt injection | Optional |
| Branch predictor poisoning | Optional |

### 4.6 SEV 4.1 — Trusted I/O (TIO) — Turin Only

New in EPYC 9005: Allows devices to DMA directly into encrypted VM memory. Eliminates the bounce buffer requirement of earlier SEV generations.

### 4.7 BIOS Configuration for SEV-SNP

Required BIOS settings (AMD CRB, may vary by OEM):
```
Advanced → AMD CBS → CPU Common Options:
  SMEE: Enable
  SEV Control: Enable
  SEV-ES ASID Space Limit: <configure split>
  SNP Memory (RMP Table) Coverage: Enabled (or Custom)

Advanced → AMD CBS → UMC Common Options → DDR Security:
  TSME: Enable/Disable (separate from SEV)
```

### 4.8 Comparison: AMD SEV-SNP vs Intel MKTME

| Feature | AMD SEV-SNP | Intel MKTME |
|---------|-------------|------------|
| **Encryption** | AES-128 per VM key | AES-XTS 128/256 per KeyID |
| **Key management** | AMD Secure Processor (ARM Cortex-A5) | PCONFIG instruction |
| **Keys available** | 1006 (EPYC 9005) | Up to 32K (theoretical) |
| **Page ownership** | RMP (hardware-enforced) | Software-managed via PTE KeyIDs |
| **Integrity** | Hardware integrity (SNP) | Optional SHA-3 (MKTME 1.6) |
| **Attestation** | Full remote attestation | No built-in attestation |
| **Register protection** | SEV-ES encrypts registers | Not applicable (different model) |
| **DMA protection** | TIO (Turin) / bounce buffers | MKTME covers all DMA with encryption |
| **Maturity** | Production since 2017, widely deployed | Newer, fewer deployments |
| **VM migration** | Supported with encrypted transport | N/A |
| **C#/.NET access** | Via KVM/QEMU APIs | Via kernel MSR interface |

**VectorVolt recommendation:** Support BOTH. Use vendor-agnostic abstraction:
- On AMD: SEV-SNP for VM-level tenant isolation
- On Intel: MKTME for page-level tenant isolation
- Different mechanisms but same security goal

---

## 5. Intel vs AMD Feature Comparison Matrix <a name="5-comparison"></a>

| VectorVolt Need | Intel Technology | AMD Technology | Abstraction |
|----------------|-----------------|----------------|-------------|
| **Topology discovery** | CPUID 0x1F (Package/Die/Tile/Module/Core/SMT) | CPUID 0x1F (Package/CCD/CCX/Core/SMT) + NPS BIOS | `CpuTopology` with vendor branches |
| **Core pinning** | Per-core affinity, avoid HT siblings | Per-CCX affinity, respect NPS quadrants | `ShardAffinityManager` |
| **Memory encryption** | MKTME (per-page KeyID) | SEV-SNP (per-VM ASID) | `TenantIsolationProvider` |
| **Hardware memcpy** | Intel DSA (MOVDIR64B/ENQCMD) | **None equivalent** — use AVX-512 VMOVDQA64 | `DataMover` with DSA fast path |
| **Hardware CRC** | DSA Copy+CRC / SSE4.2 CRC32 | SSE4.2 CRC32 (same instruction) | `HardwareCrc` — identical |
| **PCLMULQDQ** | Supported | Supported (same instruction) | `CarrylessMultiply` — identical |
| **5-Level Paging** | CR4.LA57 | CR4.LA57 (same mechanism) | `VirtualMemoryConfig` — identical |
| **Spectre mitigations** | IBRS/STIBP/SSBD/IBPB | IBRS/STIBP/SSBD/IBPB (similar but different MSRs) | `SecurityMitigations` with vendor MSR map |
| **Performance monitoring** | PMU (IA32_PERFEVTSELx) | PMU + IBS (Instruction-Based Sampling) | `PerformanceCounters` with vendor events |
| **MCA** | Enhanced MCA Logging | MCA (standard x86) | `MemoryHealthMonitor` |
| **Interrupt delivery** | FRED (new) | Standard IDT/APIC | Runtime detection |
| **Hardware compression** | IAA (deflate accelerator) | **None equivalent** | `ColdDataCompressor` with software fallback |
| **I/O virtualization** | Intel Scalable IOV | AMD-Vi (IOMMU) | `IoVirtualization` |
| **NIC acceleration** | Scalable IOV ADIs | Standard SR-IOV | `NetworkQueueManager` |

### Critical Gap: No AMD DSA Equivalent

AMD does not have a hardware data streaming accelerator equivalent to Intel DSA. This is the **biggest architectural difference** for VectorVolt.

**Mitigation strategy on AMD:**
1. Use AVX-512 `VMOVDQA64` for optimized memory copies (Zen 5 has full 512-bit path)
2. Use `REP MOVSB` with ERMS (Enhanced REP MOVSB) — hardware-optimized on Zen
3. Software CRC computation via PCLMULQDQ (same performance as Intel)
4. No hardware delta records — implement in software using SIMD compare + gather
5. No hardware dualcast — implement as two sequential writes (or use `REP STOSB` for fill)

---

## 6. VectorVolt HAL — Vendor-Agnostic Abstraction Design <a name="6-hal-abstraction"></a>

### 6.1 Vendor Detection

```csharp
public enum CpuVendor { Intel, Amd, Unknown }

public static CpuVendor DetectVendor() {
    // CPUID leaf 0 returns vendor string in EBX-EDX-ECX
    var (_, ebx, ecx, edx) = X86Base.CpuId(0, 0);
    // "GenuineIntel" or "AuthenticAMD"
    if (ebx == 0x756E6547 && edx == 0x49656E69 && ecx == 0x6C65746E)
        return CpuVendor.Intel;
    if (ebx == 0x68747541 && edx == 0x69746E65 && ecx == 0x444D4163)
        return CpuVendor.Amd;
    return CpuVendor.Unknown;
}
```

### 6.2 Key Abstractions

```csharp
// Vendor-agnostic data mover
public interface IDataMover {
    void Copy(ReadOnlySpan<byte> src, Span<byte> dst);
    void CopyWithCrc(ReadOnlySpan<byte> src, Span<byte> dst, out uint crc);
    void Fill(Span<byte> dst, ulong pattern);
    void Compare(ReadOnlySpan<byte> a, ReadOnlySpan<byte> b, out bool equal, out int firstDiff);
    void Dualcast(ReadOnlySpan<byte> src, Span<byte> dst1, Span<byte> dst2);
}

// Intel implementation uses DSA hardware
public class DsaDataMover : IDataMover { /* MOVDIR64B/ENQCMD */ }

// AMD implementation uses AVX-512 + software
public class AvxDataMover : IDataMover { /* VMOVDQA64 + software CRC */ }

// Vendor-agnostic tenant isolation
public interface ITenantIsolation {
    int MaxTenants { get; }
    void AssignTenantKey(int tenantId);
    void EncryptPage(IntPtr page, int tenantId);
}

// Intel: MKTME
public class MktmeIsolation : ITenantIsolation { /* PCONFIG + PTE KeyIDs */ }

// AMD: SEV-SNP (via KVM/QEMU APIs from host)
public class SevSnpIsolation : ITenantIsolation { /* /dev/sev + KVM ioctl */ }

// Vendor-agnostic NUMA topology
public interface INumaTopology {
    NumaNode[] EnumerateNodes();
    int GetDistance(int node1, int node2);
    int[] GetCoresOnNode(int nodeId);
    int GetNodeForCore(int coreId);
    int GetL3CacheId(int coreId);
}
```

---

## 7. Phase-by-Phase AMD Integration <a name="7-phase-matrix"></a>

### Phase 1 — Silicon/HAL

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| **Vendor detection** | CPUID leaf 0: "AuthenticAMD" | P0 |
| **Topology** | CPUID 0x1F + NPS awareness | P0 |
| **NUMA detection** | Parse ACPI SRAT/SLIT tables, detect NPS setting | P0 |
| **L3 cache topology** | 32MB per CCX, 1 CCX per CCD | P0 |
| **Memory encryption capability** | CPUID Fn8000_001F | P1 |
| **AVX-512 detection** | CPUID leaf 7: AVX512F/BW/DQ/VL | P1 |

### Phase 2 — Single-Writer Engine

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| **Core pinning** | Pin to CCX, respect NPS quadrant | P0 |
| **Memory allocation** | `numactl` / `mbind()` to same NUMA node | P0 |
| **SMT avoidance** | Detect HT sibling via CPUID 0x1F SMT level | P0 |
| **AVX-512 memcpy** | VMOVDQA64 for bulk data movement | P1 |

### Phase 3 — WAL/Durability

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| **CRC32C** | SSE4.2 CRC32 instruction (same as Intel) | P0 |
| **PCLMULQDQ** | Same instruction, same performance | P0 |
| **Software delta records** | AVX-512 VPCMPB + VPCOMPRESSB for diff generation | P1 |
| **Software dualcast** | Two sequential AVX-512 stores | P2 |

### Phase 4 — Kernel Bypass Networking

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| **PCIe Gen5** | 128 lanes, same as Intel | P1 |
| **CXL 2.0** | 64 lanes via 4 P-links — pooled memory for shared state | P2 |
| **AMD-Vi IOMMU** | DMA remapping for device assignment | P1 |

### Phase 5 — Read Fabric/SIMD

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| **AVX-512** | Full 512-bit execution path on Zen 5 (no frequency penalty!) | P0 |
| **VP2INTERSECT** | New instruction for set intersection — useful for join operations | P1 |

### Phase 6 — Multi-Tenancy

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| **SEV-SNP** | Per-VM encryption + integrity + attestation | P0 |
| **TSME** | Full memory encryption (single key, no per-tenant) | P1 (fallback) |
| **ASID management** | Up to 1006 keys on EPYC 9005 | P0 |

---

## 8. C#/.NET Integration Patterns (AMD-specific) <a name="8-csharp"></a>

### 8.1 AMD NUMA Detection via OS APIs

```csharp
// On Linux, read /sys/devices/system/node/
// Or use numactl --hardware

// On .NET, use P/Invoke to libnuma
[DllImport("libnuma.so.1")]
static extern int numa_max_node();

[DllImport("libnuma.so.1")]
static extern int numa_node_of_cpu(int cpu);

[DllImport("libnuma.so.1")]
static extern void numa_set_membind(IntPtr nodemask);
```

### 8.2 AMD CPUID for SME/SEV Detection

```csharp
// AMD-specific CPUID leaf for memory encryption
var (eax, ebx, ecx, edx) = X86Base.CpuId(unchecked((int)0x8000001F), 0);

bool smeSupported     = (eax & (1 << 0)) != 0;
bool sevSupported     = (eax & (1 << 1)) != 0;
bool sevEsSupported   = (eax & (1 << 3)) != 0;
bool sevSnpSupported  = (eax & (1 << 4)) != 0;

int cBitPosition      = ebx & 0x3F;        // Bit position of C-bit in PTE
int physAddrReduction = (ebx >> 6) & 0x3F;  // PA bits used for encryption
int maxEncryptedGuests = ecx;               // Number of encrypted guests supported
int minSevAsid        = edx;               // Minimum ASID for SEV-enabled guest
```

### 8.3 AVX-512 Optimized Operations (AMD Zen 5)

```csharp
// Zen 5 has full 512-bit ALU — NO frequency downclocking penalty (unlike Zen 4)
if (Avx512F.IsSupported) {
    // Bulk memory compare for delta detection
    Vector512<byte> a = Avx512BW.LoadVector512(srcA);
    Vector512<byte> b = Avx512BW.LoadVector512(srcB);
    ulong mask = Avx512BW.CompareEqual(a, b).ExtractMostSignificantBits();
    // mask bits where 0 = difference found
}

// VP2INTERSECT for join operations (Zen 5)
// Not yet in .NET intrinsics — may need native shim
```

### 8.4 NPS Detection at Runtime

NPS setting is a BIOS option, not directly readable from user space. Detection strategies:
```csharp
// Strategy 1: Parse ACPI SLIT table (requires root)
// Distance 10 = local, 12 = near-hop, 20/32 = far-hop

// Strategy 2: Count NUMA nodes and match to expected patterns
int numaNodes = numa_max_node() + 1;
int coresPerSocket = Environment.ProcessorCount / socketCount;
// NPS4: 4 nodes/socket; NPS2: 2 nodes/socket; NPS1: 1 node/socket

// Strategy 3: Measure actual latency between NUMA nodes
// Allocate memory on each node, measure cross-node access time
```

---

## Source PDFs Reference

| File | Document | Pages | Key Content |
|------|----------|-------|-------------|
| 58462 | EPYC 9005 Architecture Overview | 11 | Zen 5 specs, CCX/CCD/IOD, NUMA, CPUID |
| memory-encryption-white-paper | AMD Memory Encryption | 12 | SME C-bit, SEV per-VM keys, architecture |
| 56795 | NUMA Topology Milan | 18 | NPS settings, SLIT distances, L3AsNUMA |
| 56308 | NUMA Topology Naples | 7 | Original 4-die architecture |
| 2025_SEV_Guide | SEV Enablement Guide | 52 | BIOS config, MSR, ASID, SEV 4.1 TIO |
| dell-emc-dfd-numa | Dell NUMA Guide Rome | 3 | NPS recommendations per workload |
| 58478 | K8s Tuning EPYC 9005 | 20 | CPU pinning, NUMA alignment, NRI plugin |
| amd-manual (APM Vol 2) | System Programming | 782 | MSRs, paging, MCA, perf monitoring |
| lp1894 | Lenovo SME/SEV/SEV-ES | 16 | Practical enablement steps |
| lp1893 | Lenovo SEV-SNP | 13 | SEV-SNP enablement steps |

---

---

# PART 2: BEYOND-CPU HARDWARE OPTIMIZATIONS (AMD Platform)

> **Added:** 2026-04-14 (Session 2). Covers NVMe storage, networking/NIC, and platform/BIOS tuning — AMD-specific differences and recommendations. Vendor-neutral technologies (NVMe, RDMA, etc.) are documented in detail in the Intel handover; this section focuses on **AMD-specific configuration, MSRs, and BIOS settings**.

---

## 9. NVMe Storage Optimizations (AMD Platform) <a name="9-nvme-amd"></a>

> **Note:** NVMe technologies (CMB, PMR, ZNS, FDP, NVMe-oF) are vendor-neutral — they work identically on AMD and Intel platforms. See the Intel handover Sections 20.1-20.6 for full register-level details. This section covers AMD-specific considerations.

### 9.1 AMD-Specific NVMe Considerations

**IOMMU (AMD-Vi) for NVMe:**
- AMD uses AMD-Vi (not VT-d) for DMA remapping
- BIOS: "IOMMU" → Enable (required for VFIO passthrough with DPDK/SPDK)
- Kernel: `amd_iommu=on iommu=pt` (passthrough mode for best performance)
- AMD-Vi supports PASID for per-process NVMe access (needed for shared queues)

**PCIe Lane Allocation on EPYC 9005:**
- 128 PCIe Gen5 lanes per socket
- 4 P-links (configurable as PCIe or CXL)
- NVMe SSDs connect via P-links through IOD
- **VectorVolt consideration:** All NVMe I/O traverses the IOD fabric — allocate NVMe devices on the same quadrant (NPS4 NUMA node) as the shard writer to minimize fabric hops

**CXL on AMD EPYC 9005:**
- CXL 2.0 supported via P-links (up to 64 CXL lanes)
- CXL Type 3 memory expanders work identically to Intel
- CXL memory appears as additional NUMA node
- Same `cxl-cli` and `/sys/bus/cxl/devices/` tooling
- **Key difference:** AMD CXL goes through IOD → slightly different latency profile (~130-180ns vs Intel's ~120-170ns)

### 9.2 AMD-Specific NVMe Tiered WAL Path

Same tiering strategy as Intel (see Intel handover Section 20.6), with identical priority order:
```
Tier 0: CXL PMem or NVMe PMR → ~200ns commit
Tier 1: NVMe CMB + ZNS Zone Append → ~8-12μs
Tier 2: NVMe + FDP → ~10-15μs
Tier 3: Conventional NVMe + SPDK → ~10-15μs
```

No AMD-specific changes needed — NVMe is vendor-neutral.

---

## 10. Network / NIC Optimizations (AMD Platform) <a name="10-network-amd"></a>

> **Note:** RDMA, DPDK, AF_XDP, PTP, SmartNICs are all vendor-neutral. See Intel handover Sections 21.1-21.8 for full details. This section covers AMD-specific networking differences.

### 10.1 AMD Pensando DPU (AMD-Owned)

AMD acquired Pensando Systems in 2022. The **AMD Pensando DSC-200** is AMD's SmartNIC/DPU:

| Feature | AMD Pensando DSC-200 | NVIDIA BlueField-3 |
|---------|---------------------|-------------------|
| Speed | 2×100G (200Gb/s) | 400Gb/s |
| Pipeline | **P4-programmable** (custom packet logic in hardware) | Fixed-function + ARM cores |
| Firewall | Stateful, line-rate | Via ARM cores |
| Encryption | Inline IPsec/TLS | Inline IPsec/TLS |
| Programmability | P4 → hardware match-action tables | DOCA SDK on ARM cores |

**AMD Pensando Pollara 400:** Next-gen AI NIC with advanced RDMA and congestion control. 400Gb/s. Available since H1 2025.

**VectorVolt advantage on AMD platform:** Using AMD Pensando DPU on an AMD EPYC server gives a vertically-integrated stack where AMD controls CPU + NIC + fabric.

### 10.2 RDMA on AMD Platforms

RDMA NICs are vendor-neutral (ConnectX-7, E810/E830, Broadcom P2200G all work on AMD). No AMD-specific differences in RDMA verbs or performance.

**AMD-specific consideration:** RDMA NIC interrupt affinity must respect NPS settings. Pin RDMA CQ processing threads to cores on the same NUMA quadrant as the NIC's PCIe attachment point.

### 10.3 DPDK on AMD

DPDK works identically on AMD. The mlx5 bifurcated driver, VFIO-PCI binding, and AF_XDP zero-copy all function the same.

**AMD-specific BIOS for DPDK:**
- "IOMMU" → Enable (for VFIO-PCI)
- "ACS Control" → depends on isolation needs (same as Intel)
- "SR-IOV Support" → Enable if using VF-based DPDK
- "NPC" (NUMA Per CCD) affects which NUMA node the NIC is associated with

### 10.4 Intel ADQ Equivalent on AMD

AMD does **NOT** have an equivalent to Intel ADQ (Application Device Queues). On AMD platforms:
- Use **RSS + Flow Director** on ConnectX-7/E810 for flow steering (vendor-neutral NIC features)
- Use **AMD Pensando DSC-200** with P4 rules for per-application traffic classes
- Use **ethtool ntuple filters** for n-tuple flow pinning

---

## 11. Platform / BIOS / Memory Tuning (AMD-Specific) <a name="11-platform-amd"></a>

### 11.1 C-State Tuning (AMD)

AMD C-states differ from Intel in naming and MSR addresses:

| C-State | AMD Name | Exit Latency | What Powers Down |
|---------|----------|-------------|-----------------|
| C0 | Active | 0 | Nothing |
| C1 | Halt | ~1μs | Core clock gate |
| C2 | — | ~5-10μs | Core clock off |
| CC6 | Core C6 | ~50-100μs | Core power off, state saved to DRAM |
| PC6 | Package C6 | ~100-300μs | Package power reduced |

**Key MSRs (AMD):**
```
MSRC001_0292 (CSTATE_CONFIG):
  Controls C-state behavior and limits

MSRC001_0015 (HWCR):
  Bit 25: SMM_Lock
  Bit 9:  Ignne_Em (related to power management)
```

**BIOS settings (AMD CBS):**
```
Advanced → AMD CBS → CPU Common Options:
  "Global C-State Control" → Disabled (disables all C-states)
  "Power Supply Idle Control" → Typical Current Idle
  "CCD/Core SEV-SNP" → (separate from C-states, don't confuse)

Advanced → AMD CBS → NBIO Common Options:
  "IOMMU" → Enabled
  "ACS" → as needed
```

**VectorVolt recommendation:** Disable "Global C-State Control" in AMD CBS. Same rationale as Intel — eliminates 50-200μs wakeup latency.

### 11.2 P-State / Frequency Control (AMD)

**AMD uses CPPC2 (Collaborative Processor Performance Control v2)** — equivalent to Intel HWP.

**Key MSRs:**
```
MSRC001_0061 (P-State Current Limit):
  Bits [6:4] = PstateMaxVal (highest allowed P-state)
  Bits [2:0] = CurPstateLimit

MSRC001_0062 (P-State Control):
  Bits [2:0] = PstateCmd (write desired P-state index)

MSRC001_0063 (P-State Status):
  Bits [2:0] = CurPstate (current P-state)

MSRC001_0064-0069 (P-State definitions 0-5):
  Frequency, voltage, current for each P-state
```

**BIOS settings:**
```
Advanced → AMD CBS → CPU Common Options:
  "Core Performance Boost" → Enabled (= AMD turbo)
  "Global C-State Control" → Disabled
  "CPPC" → Enabled (for OS frequency control)
  "CPPC Preferred Cores" → Enabled (AMD's equivalent to TBMT 3.0)
```

**Linux:** `echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor`
**AMD-specific driver:** `amd-pstate` (preferred over `acpi-cpufreq` on Zen 4/5).

### 11.3 AMD Preferred Cores (Equivalent to Intel TBMT 3.0)

**What it is:** AMD CPPC2 + "Preferred Cores" identifies the fastest cores per CCD (silicon lottery). The OS can prioritize scheduling on these cores.

**Detection:**
```
CPUID Fn8000_0008:EBX[2] (PreferedCoreCnt) — indicates preferred core support
/sys/devices/system/cpu/cpu*/cpufreq/amd_pstate_prefcore_ranking — per-core ranking (higher = faster)
```

**BIOS:** "CPPC Preferred Cores" → Enable (under AMD CBS → CPU Common Options).

**VectorVolt recommendation:** Enable preferred cores. Pin WAL writer to highest-ranked core per CCD. On Zen 5 with 1 CCX per CCD, each CCD has 1-2 preferred cores.

### 11.4 NPS vs Intel SNC Comparison

| Feature | AMD NPS | Intel SNC |
|---------|---------|-----------|
| Mechanism | Partitions IOD quadrants into NUMA nodes | Partitions LLC slices + memory controllers |
| Options | NPS1 / NPS2 / NPS4 | Disabled / SNC2 / SNC4 |
| VectorVolt recommendation | **NPS4** (finest granularity) | **SNC2** (good balance) |
| Effect on LLC | No change (L3 is per-CCX always) | Partitions shared LLC |
| Effect on memory | Restricts interleaving to quadrant | Restricts to SNC domain |

**Key difference:** On AMD, L3 is ALWAYS per-CCX regardless of NPS setting. NPS only affects which memory controllers each NUMA node owns. On Intel, SNC actually partitions the shared LLC.

### 11.5 Power Limit Tuning (AMD PPT/TDC/EDC)

AMD uses different power limit terminology than Intel PL1/PL2:

| AMD Term | Intel Equivalent | What It Controls |
|----------|-----------------|-----------------|
| PPT (Package Power Tracking) | PL2 (burst) | Max socket power draw |
| TDC (Thermal Design Current) | N/A | Max sustained current draw |
| EDC (Electrical Design Current) | N/A | Max peak current draw |

**BIOS settings:**
```
Advanced → AMD CBS → CPU Common Options:
  "Package Power Limit (PPT)" → Max (server boards: typically at TDP)
  
Advanced → AMD Overclocking:
  "PPT Limit" → platform max
  "TDC Limit" → platform max
  "EDC Limit" → platform max
```

**VectorVolt recommendation:** Set PPT to max supported by platform cooling. No AMD MSR equivalent that's easily writable from OS like Intel's MSR 0x610 — must be set in BIOS.

### 11.6 AMD Prefetcher Control

**MSR MSRC001_1020 (Load-Store Configuration):**
```
Bit 13: Disable Stream Prefetcher
Bit 15: Disable Stride Prefetcher
```

**MSR MSRC001_1022 (Data Cache Configuration):**
```
Bit 13: Disable Hardware Prefetch (L1)
```

**BIOS (AMD CBS):**
```
Advanced → AMD CBS → CPU Common Options:
  "L1 Stream HW Prefetcher" → Disable/Enable
  "L2 Stream HW Prefetcher" → Disable/Enable
```

**VectorVolt recommendation:** Same as Intel — disable adjacent-line / stride prefetch for random B-tree/hash workloads. Keep L1 stream prefetcher for sequential WAL writes.

### 11.7 Hyper-Threading / SMT on AMD

AMD calls it **SMT (Simultaneous Multi-Threading)**, same concept as Intel HT.

**Impact on Zen 5:** Each Zen 5 core has 2 threads. Disabling SMT gives each thread:
- Full L1D: 48KB (vs shared)
- Full L2: 1MB (vs shared)
- Full execution units: 6-wide issue, 4 ALU, 2 AGU
- Full store buffer, load buffer, reorder buffer

**BIOS:** "SMT Control" → Disable (under AMD CBS → CPU Common Options).
**Kernel:** `nosmt` boot parameter.

**VectorVolt recommendation:** Disable SMT. Same rationale as Intel — latency > throughput for OLTP. Each Zen 5 core at full resources.

### 11.8 DDR5 on AMD EPYC 9005

**12 DDR5 channels per socket** (vs Intel SPR's 8 channels):
- 3 UMCs per quadrant × 4 quadrants = 12 channels
- 1DPC: 12 × 64GB = 768GB per socket at DDR5-6000 MT/s
- 2DPC: 12 × 128GB = 1.5TB per socket at DDR5-5600 MT/s (derating)

**Max memory with CXL:** 6TB DDR5 + 2TB CXL = 8TB per socket.

| Config | Speed | Bandwidth |
|--------|-------|-----------|
| 1DPC (12 × 64GB = 768GB) | DDR5-6000 | ~576 GB/s per socket |
| 2DPC (12 × 128GB = 1.5TB) | DDR5-5600 | ~537 GB/s per socket |

**VectorVolt recommendation:** 1DPC with 12 × 64GB RDIMMs = 768GB at full DDR5-6000 speed. AMD's 12-channel advantage gives ~50% more bandwidth than Intel's 8-channel SPR.

### 11.9 Huge Pages on AMD

Identical to Intel — uses same Linux huge page infrastructure:
```bash
# Disable THP
echo never > /sys/kernel/mm/transparent_hugepage/enabled
echo never > /sys/kernel/mm/transparent_hugepage/defrag

# Reserve per-NUMA-node (NPS4 = 4 nodes per socket)
for node in 0 1 2 3; do
  echo 16384 > /sys/devices/system/node/node$node/hugepages/hugepages-2048kB/nr_hugepages
done

# Or 1GB pages at boot
hugepagesz=1G hugepages=128 default_hugepagesz=1G
```

**AMD-specific:** With NPS4, ensure huge pages are distributed evenly across all 4 NUMA nodes per socket. Uneven distribution causes cross-node allocation fallback (+20-100% latency).

### 11.10 Patrol Scrub on AMD

**BIOS (AMD CBS):**
```
Advanced → AMD CBS → UMC Common Options → DDR Memory Features:
  "Patrol Scrub" → Disable
  "Patrol Scrub Interval" → if not disabling, set to max
```

**VectorVolt recommendation:** Disable. DDR5 on-die ECC provides base protection. Same rationale as Intel.

### 11.11 PCIe Gen5 on AMD EPYC 9005

- 128 PCIe Gen5 lanes per socket
- 4 P-links, each configurable as x16 PCIe Gen5 or CXL 2.0
- Bifurcation supported (vendor-dependent BIOS options)
- **AMD-Vi IOMMU** for device isolation (vs Intel VT-d)

**BIOS:**
```
Advanced → AMD CBS → NBIO Common Options:
  "IOMMU" → Enabled
  "ACS Enable" → as needed
  "PCIe ARI Support" → Enabled (for SR-IOV)
  "Enable AER Cap" → Enabled (Advanced Error Reporting)
```

---

## 12. Updated Intel vs AMD Comparison Matrix (Full Stack) <a name="12-full-comparison"></a>

| VectorVolt Need | Intel | AMD | Abstraction |
|----------------|-------|-----|-------------|
| **CPU Topology** | CPUID 0x1F (Package/Die/Tile/Module/Core/SMT) | CPUID 0x1F + NPS BIOS | `CpuTopology` |
| **NUMA Partitioning** | SNC2/SNC4 | NPS1/NPS2/NPS4 + L3AsNUMA | `INumaTopology` |
| **Memory Encryption** | MKTME (per-page KeyID) | SEV-SNP (per-VM ASID) | `ITenantIsolation` |
| **Hardware Memcpy** | DSA (MOVDIR64B/ENQCMD) | AVX-512 VMOVDQA64 (software) | `IDataMover` |
| **Hardware CRC** | DSA Copy+CRC / SSE4.2 | SSE4.2 CRC32 (same) | `HardwareCrc` |
| **Hardware Compression** | IAA (deflate) | None — software fallback | `ColdDataCompressor` |
| **Interrupt Delivery** | FRED (new, faster) | Standard IDT/APIC | Runtime detect |
| **Preferred Cores** | TBMT 3.0 (MSR 0x771) | CPPC2 Preferred Cores | `FavoredCoreSelector` |
| **C-State Control** | MSR 0xE2, intel_idle | MSRC001_0292, amd CBS | `PlatformTuningValidator` |
| **P-State Control** | HWP (MSR 0x774, EPP) | CPPC2 (amd-pstate driver) | `PlatformTuningValidator` |
| **Prefetcher Tuning** | MSR 0x1A4 | MSRC001_1020/1022 | `PlatformTuningValidator` |
| **DPU/SmartNIC** | Intel IPU | AMD Pensando DSC-200/Pollara 400 | `DpuOffloadManager` |
| **DDR5 Channels** | 8 per socket (SPR) | **12 per socket (EPYC 9005)** | Platform config |
| **PCIe Lanes** | 80 (SPR) / 128 (GNR) | **128** (EPYC 9005) | Platform config |
| **CXL** | CXL 2.0 (1.1 on SPR) | CXL 2.0 (64 lanes via P-links) | `CxlMemoryManager` |
| **NVMe (CMB/PMR/ZNS/FDP)** | Vendor-neutral (identical) | Vendor-neutral (identical) | `StorageCapabilities` |
| **RDMA** | Vendor-neutral (identical) | Vendor-neutral (identical) | `RdmaCapabilities` |
| **PTP** | Vendor-neutral (identical) | Vendor-neutral (identical) | `DistributedClock` |
| **Huge Pages** | Identical Linux API | Identical Linux API | `HugePageAllocator` |

---

## 13. Updated Phase-by-Phase AMD Integration (All Hardware) <a name="13-full-phase-matrix"></a>

### Phase 1 — Silicon/HAL (Updated)

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| All items from Section 7, Phase 1 | — | — |
| **NVMe capability detection** | Same registers as Intel (vendor-neutral) | P0 |
| **CXL device enumeration** | Via P-links, slightly higher latency (~130-180ns) | P1 |
| **RDMA NIC detection** | Vendor-neutral | P1 |
| **AMD-Vi IOMMU config** | Required for VFIO/SPDK/DPDK | P0 |

### Phase 2 — Single-Writer Engine (Updated)

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| All items from Section 7, Phase 2 | — | — |
| **Huge page buffer pool** | 1GB pages, distributed across NPS4 NUMA nodes | P0 |
| **CPPC2 Preferred Cores** | Pin WAL writer to highest-ranked core per CCD | P1 |
| **C-state disable** | AMD CBS: "Global C-State Control" → Disabled | P0 |
| **SMT disable** | AMD CBS: "SMT Control" → Disabled | P1 |

### Phase 3 — WAL/Durability (Updated)

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| All items from Section 7, Phase 3 | — | — |
| **NVMe PMR WAL commit** | Same as Intel (~200ns) | P0 |
| **ZNS Zone Append** | Same as Intel — concurrent lock-free WAL | P0 |
| **FDP data placement** | Same as Intel — WAL vs data separation | P1 |

### Phase 4 — Kernel Bypass Networking (Updated)

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| All items from Section 7, Phase 4 | — | — |
| **RDMA replication** | One-sided writes, sub-2μs on RoCEv2 | P0 |
| **AF_XDP / DPDK** | Vendor-neutral; IOMMU=AMD-Vi | P0 |
| **AMD Pensando DPU** | P4-programmable, TLS/IPsec offload | P1 |
| **PTP timestamps** | Vendor-neutral | P1 |
| **Flow steering** | No AMD ADQ equivalent — use NIC-native (ConnectX-7 flow rules) or Pensando P4 | P1 |

### Phase 5 — Read Fabric/SIMD (Updated)

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| All items from Section 7, Phase 5 | — | — |
| **CXL memory tier** | CXL via P-links, ~130-180ns | P1 |
| **12-channel DDR5 bandwidth** | 50% more bandwidth than Intel 8-channel | P0 (architecture advantage) |

### Phase 6 — Multi-Tenancy (Updated)

| Technology | AMD Detail | Priority |
|-----------|-----------|----------|
| All items from Section 7, Phase 6 | — | — |
| **NPS4 NUMA isolation** | Finest granularity, shard-per-quadrant | P0 |

---

## 14. New Classes Added (AMD Beyond-CPU) <a name="14-new-classes-amd"></a>

> Most new classes are **vendor-neutral** (shared with Intel). AMD-specific additions:

### AMD-Specific
```
AmdCppcPreferredCores       — CPPC2 preferred core ranking detection
AmdNpsDetector              — Detect NPS setting from SLIT distances / NUMA node count
AmdPensandoOffloader        — AMD Pensando DSC-200 P4-based offload management
AmdViIommuConfig            — AMD-Vi IOMMU configuration validation
```

### Vendor-Neutral (same as Intel handover, Section 24)
```
StorageCapabilities         — NVMe CMB/PMR/ZNS/FDP detection
PmrWalWriter                — PMR-based WAL commit (~200ns)
CxlPmemWalWriter            — CXL PMem WAL commit (~200ns)
ZnsWalWriter                — Zone Append concurrent WAL
FdpPlacementManager         — Data lifetime separation
CmbQueueManager             — SQ in Controller Memory Buffer
CxlMemoryManager            — CXL Type 3 NUMA management
RdmaReplicator              — One-sided RDMA Write for replication
KernelBypassTransport       — AF_XDP / DPDK packet I/O
FlowSteeringManager         — RSS + n-tuple flow rules
DistributedClock            — PTP hardware timestamps
PacketFilterEngine          — XDP/eBPF firewall
PlatformTuningValidator     — C-state/P-state/SMT/THP validation
HugePageAllocator           — 1GB/2MB pages with NUMA affinity
CxlBufferPoolTier           — Warm-page CXL memory tier
```

---

## 15. AMD Platform Tuning Checklist <a name="15-checklist"></a>

Quick-reference BIOS checklist for VectorVolt on AMD EPYC 9005:

```
Advanced → AMD CBS → CPU Common Options:
  ☑ SMEE: Enabled (for SEV-SNP)
  ☑ SEV Control: Enabled
  ☑ SMT Control: Disabled
  ☑ Global C-State Control: Disabled
  ☑ Core Performance Boost: Enabled
  ☑ CPPC: Enabled
  ☑ CPPC Preferred Cores: Enabled
  ☑ L1 Stream HW Prefetcher: Enabled
  ☑ L2 Stream HW Prefetcher: Disabled (for random access)

Advanced → AMD CBS → UMC Common Options → DDR:
  ☑ Patrol Scrub: Disabled
  ☑ TSME: Disabled (use SEV-SNP per-tenant instead)

Advanced → AMD CBS → NBIO Common Options:
  ☑ IOMMU: Enabled
  ☑ ACS Enable: As needed
  ☑ PCIe ARI Support: Enabled

Advanced → AMD CBS → DF Common Options:
  ☑ NUMA Nodes Per Socket (NPS): NPS4
  ☑ Memory Interleaving: Channel
  ☑ L3 As NUMA Domain: Enabled (optional, for per-CCX granularity)

Advanced → AMD Overclocking:
  ☑ PPT Limit: Platform max
  ☑ TDC Limit: Platform max
  ☑ EDC Limit: Platform max

Boot / OS:
  ☑ Kernel: amd_iommu=on iommu=pt nosmt
  ☑ Kernel: hugepagesz=1G hugepages=128 default_hugepagesz=1G
  ☑ Kernel: transparent_hugepage=never
  ☑ Governor: echo performance > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
  ☑ Driver: amd-pstate=active
```

---

---

# PART 3: AMD-SPECIFIC RUNTIME & DEPLOYMENT NOTES

> **Added:** 2026-04-14 (Session 3). AMD-specific additions to the runtime probe-and-dispatch architecture. The full architecture (SiliconManifest, ProviderFactory, interfaces, graceful degradation, Kubernetes, on-prem) is defined in the **Intel handover document Part 3 (Sections 25-35)**. This section covers only **AMD differences**.

> **READ FIRST:** Intel handover Sections 25-35 for the complete architecture. This document is a **companion**, not standalone.

---

## 16. AMD-Specific Probing Differences <a name="16-amd-probing"></a>

### 16.1 CpuProber — AMD Path

```csharp
// AMD-specific CPUID detection (inside CpuProber.Probe)
if (vendor == CpuVendor.Amd)
{
    // ── SEV/SNP capability (CPUID Fn8000_001F) ──
    var (eaxSev, ebxSev, ecxSev, edxSev) = X86Base.CpuId(unchecked((int)0x8000001F), 0);
    b.HasSevSnp        = (eaxSev & (1 << 4)) != 0;
    b.SevMaxAsids      = ecxSev;
    int cBitPosition   = ebxSev & 0x3F;
    int paReduction    = (ebxSev >> 6) & 0x3F;

    // ── AVX-512 without throttle (Zen 5+) ──
    // Zen 5 has full 512-bit ALU with NO frequency downclocking
    // Detect via CPUID Family 1Ah (EPYC 9005)
    var (eax1, _, _, _) = X86Base.CpuId(1, 0);
    int family = ((eax1 >> 8) & 0xF) + ((eax1 >> 20) & 0xFF);
    b.HasAvx512NoThrottle = (family >= 0x1A) && Avx512F.IsSupported;

    // ── VP2INTERSECT (Zen 5 set intersection) ──
    // Check CPUID leaf 7, ECX=0, EDX bit 8
    var (_, _, _, edx7) = X86Base.CpuId(7, 0);
    b.HasVp2Intersect = (edx7 & (1 << 8)) != 0;

    // ── No DSA on AMD ──
    b.HasDsa = false;
    b.HasIaa = false;

    // ── No MKTME on AMD ──
    b.HasMktme = false;
    b.MktmeMaxKeys = 0;

    // ── No FRED on AMD ──
    b.HasFred = false;

    // ── No Key Locker on AMD ──
    b.HasKeyLocker = false;

    // ── AMD-specific Spectre: similar MSRs but different addresses ──
    // IBRS/STIBP/SSBD via CPUID Fn8000_0008:EBX
    var (_, ebx8, _, _) = X86Base.CpuId(unchecked((int)0x80000008), 0);
    b.HasEnhancedIbrs = (ebx8 & (1 << 2)) != 0;  // IBRS always-on
}
```

### 16.2 TopologyProber — AMD Differences

```csharp
// AMD topology uses same CPUID 0x1F but different hierarchy:
// Intel: Package → Die → Tile → Module → Core → SMT
// AMD:   Package → CCD → CCX → Core → SMT
//
// Also: NPS setting is NOT readable via CPUID — must be inferred

if (vendor == CpuVendor.Amd)
{
    // ── NPS detection strategy ──
    // Strategy 1: Count NUMA nodes and match to pattern
    int numaNodes = /* from /sys/devices/system/node/ */;
    int sockets   = /* count unique package IDs */;
    int nodesPerSocket = numaNodes / sockets;

    b.AmdNps = nodesPerSocket switch {
        >= 4 => NpsLevel.Nps4,   // 4+ nodes per socket = NPS4
        >= 2 => NpsLevel.Nps2,   // 2 nodes per socket = NPS2
        _    => NpsLevel.Nps1    // 1 node per socket = NPS1
    };

    // Strategy 2: Parse SLIT distances for confirmation
    // Distance 10 = local, 12 = near-hop (same socket), 20/32 = far-hop (cross socket)
    // NPS4 shows 4 groups of distance-10 within each socket

    // Strategy 3 (most accurate): Measure actual cross-node latency
    // Allocate memory on each node, time cross-node reads

    // ── L3AsNUMA detection ──
    // If NUMA nodes > (sockets × NPS_expected_nodes), L3AsNUMA is on
    // e.g., 8 nodes on 1-socket NPS4 = L3AsNUMA (should be 4 without it)

    // ── Preferred cores ──
    // Read /sys/devices/system/cpu/cpu*/cpufreq/amd_pstate_prefcore_ranking
    // Higher value = faster core (silicon lottery)
    var rankings = new Dictionary<int, int>();
    foreach (var cpu in Directory.GetDirectories("/sys/devices/system/cpu/", "cpu[0-9]*")) {
        string rankFile = Path.Combine(cpu, "cpufreq/amd_pstate_prefcore_ranking");
        if (File.Exists(rankFile)) {
            int cpuId = int.Parse(Path.GetFileName(cpu).Substring(3));
            int rank = int.Parse(File.ReadAllText(rankFile).Trim());
            rankings[cpuId] = rank;
        }
    }
    b.FavoredCoreIds = rankings.OrderByDescending(kv => kv.Value)
                               .Take(4)
                               .Select(kv => kv.Key)
                               .ToArray();
}
```

### 16.3 PlatformProber — AMD Differences

```csharp
if (vendor == CpuVendor.Amd)
{
    // ── CPU governor: prefer amd-pstate driver ──
    string driver = File.ReadAllText("/sys/devices/system/cpu/cpu0/cpufreq/scaling_driver").Trim();
    if (driver != "amd-pstate" && driver != "amd-pstate-epp") {
        b.AddWarning(WarningSeverity.Warning, "AMD_PSTATE_MISSING",
            $"CPU scaling driver is '{driver}'. AMD platforms should use 'amd-pstate' for best frequency management.",
            "Boot with kernel param: amd_pstate=active");
    }

    // ── NPS validation ──
    if (b.AmdNps == NpsLevel.Nps1) {
        b.AddWarning(WarningSeverity.Warning, "NPS1_DETECTED",
            "NPS1 detected. VectorVolt recommends NPS4 for finest NUMA granularity and lowest cross-node latency.",
            "BIOS: AMD CBS → DF Common Options → NUMA Nodes Per Socket → NPS4");
    }

    // ── IOMMU check ──
    bool iommuEnabled = File.Exists("/sys/class/iommu/");
    if (!iommuEnabled) {
        b.AddWarning(WarningSeverity.Warning, "AMD_IOMMU_OFF",
            "AMD-Vi IOMMU not detected. Required for VFIO (DPDK/SPDK) and DSA PASID-like features.",
            "BIOS: AMD CBS → NBIO → IOMMU → Enabled. Kernel: amd_iommu=on iommu=pt");
    }
}
```

---

## 17. AMD-Specific Provider Factory Differences <a name="17-amd-factory"></a>

The `ProviderFactory` (defined in Intel handover Section 29) handles AMD automatically via the `SiliconManifest`. Key AMD-specific dispatch paths:

```csharp
// ── DataMover: AMD has NO DSA — always AVX-512 or AVX2 ──
// On AMD Zen 5: Avx512DataMover (full 512-bit, no throttle)
// On AMD Zen 4: Avx512DataMover (but may throttle in some configs)
// On AMD Zen 3-: Avx2DataMover

// ── WAL Writer: AMD has NO DSA CRC — use software CRC ──
// PMR/CXL/ZNS/FDP paths are vendor-neutral (same on AMD and Intel)
// Only difference: no DSA CRC inline — software CRC via SSE4.2 instead
// This means PmrWalWriter (not PmrDsaWalWriter) on AMD

// ── Tenant Isolation: SEV-SNP instead of MKTME ──
// SevSnpIsolation uses /dev/sev + KVM ioctls (host-level)
// Different isolation model: per-VM ASID vs per-page KeyID
// Max tenants: 1006 ASIDs on EPYC 9005 (vs 32K KeyIDs on Intel MKTME)

// ── Compression: AMD has NO IAA — always software ──
// SoftwareDeflateCompressor on AMD (zstd or lz4)

// ── Core Selector: uses CPPC2 preferred cores instead of TBMT 3.0 ──
// Same interface (FavoredCoreSelector), different detection source
// CPPC2 ranking from /sys/devices/system/cpu/cpu*/cpufreq/amd_pstate_prefcore_ranking
// vs Intel TBMT 3.0 from MSR 0x771 (IA32_HWP_CAPABILITIES)
```

### AMD Degradation Chain Differences

Where AMD diverges from Intel in the degradation chains (see Intel handover Section 30):

```
Data Movement:
  Intel: DSA → AVX-512 → AVX2 → REP MOVSB → Buffer.MemoryCopy
  AMD:   AVX-512 (no throttle on Zen 5) → AVX2 → REP MOVSB → Buffer.MemoryCopy
         ^^^^^^^^ starts here (no DSA)

WAL CRC:
  Intel: DSA Copy+CRC (inline) → SSE4.2 + PCLMULQDQ → SSE4.2 only → software
  AMD:   SSE4.2 + PCLMULQDQ → SSE4.2 only → software
         ^^^^^^^^^^^^^^^^^^^^^^ starts here (no DSA CRC)

Tenant Isolation:
  Intel: MKTME → Key Locker + SW crypto → AES-NI GCM → Process isolation
  AMD:   SEV-SNP → AES-NI GCM → Process isolation
         ^^^^^^^ starts here (different but equally strong)

Compression:
  Intel: IAA hardware deflate → software zstd → software lz4 → none
  AMD:   Software zstd → software lz4 → none
         ^^^^^^^^^^^^^^ starts here (no IAA)
```

---

## 18. AMD Kubernetes Deployment Differences <a name="18-amd-k8s"></a>

> **Full K8s architecture:** See Intel handover Sections 33.1-33.8. This section covers only AMD differences.

### 18.1 K8s Device Plugin Differences

| Plugin | Intel | AMD | Notes |
|--------|-------|-----|-------|
| DSA | `intel.com/dsa` ✓ | ✗ Not available | AMD has no DSA equivalent |
| IAA | `intel.com/iaa` ✓ | ✗ Not available | AMD has no IAA equivalent |
| RDMA | Same | Same | Vendor-neutral (ConnectX-7 works on both) |
| SR-IOV | Same | Same | Vendor-neutral |
| AF_XDP | Same | Same | Vendor-neutral |
| AMD SEV | N/A | `amd.com/sev` | Confidential Containers (Kata + SEV-SNP) |

### 18.2 AMD Worker Node Kernel Command Line

```bash
# AMD-specific kernel params (differs from Intel)
GRUB_CMDLINE_LINUX="amd_iommu=on iommu=pt nosmt \
  hugepagesz=1G hugepages=128 default_hugepagesz=1G \
  transparent_hugepage=never \
  isolcpus=4-63 nohz_full=4-63 rcu_nocbs=4-63 \
  amd_pstate=active \
  skew_tick=1 tsc=reliable clocksource=tsc"
```

**Key differences from Intel kernel params:**
- `amd_iommu=on` instead of `intel_iommu=on`
- `amd_pstate=active` — AMD P-state driver (instead of intel_pstate)
- No `intel_idle.max_cstate=1` — AMD uses different idle driver; C-state control via BIOS "Global C-State Control"

### 18.3 AMD Pod Spec Differences (Tier 1)

```yaml
# Differences from Intel pod spec (Section 33.6):
resources:
  requests:
    cpu: "16"
    memory: "8Gi"
    hugepages-1Gi: "64Gi"
    # NO intel.com/dsa — AMD has no DSA
    # NO intel.com/iaa — AMD has no IAA
    rdma/rdma_shared_device_a: "1"    # Same as Intel
  limits:
    cpu: "16"
    memory: "8Gi"
    hugepages-1Gi: "64Gi"
    rdma/rdma_shared_device_a: "1"

# For SEV-SNP confidential containers:
# Use Kata Containers runtime with AMD SEV-SNP support
# runtimeClassName: kata-cc-sev-snp
```

### 18.4 AMD Helm Chart Value Overrides

```yaml
# values-amd.yaml — overlay on top of base values.yaml

deployment:
  tier: performance

cpu:
  cores: 16                        # physical cores (SMT disabled)
  vendor: amd                       # triggers AMD-specific probing
  preferredCoresEnabled: true       # CPPC2 preferred cores

accelerators:
  dsa:
    enabled: false                  # Not available on AMD
  iaa:
    enabled: false                  # Not available on AMD

network:
  rdma:
    enabled: auto                   # Same as Intel
  pensandoDpu:
    enabled: auto                   # AMD Pensando DSC-200 detection

security:
  tenantIsolation: sevsnp           # SEV-SNP instead of MKTME
  confidentialCompute: false         # Kata + SEV-SNP (optional)
```

---

## 19. AMD On-Prem Reference Architecture <a name="19-amd-onprem"></a>

### 19.1 Recommended AMD Server Configuration

```
┌────────────────────────────────────────────────────┐
│  VectorVolt Node — AMD EPYC 9554 (Zen 5)           │
│                                                      │
│  CPU:        64 cores (SMT disabled = 64 threads)    │
│  BIOS:       NPS4 (4 NUMA nodes per socket)          │
│              C-states disabled, CPB enabled           │
│              CPPC + Preferred Cores enabled           │
│              SEV-SNP enabled, TSME disabled           │
│              Patrol Scrub disabled                    │
│                                                      │
│  Memory:     768 GB DDR5-6000 (12 × 64GB, 1DPC)     │
│              + 1GB HugePages (128 reserved)           │
│              3 UMCs per quadrant                      │
│              ~576 GB/s aggregate bandwidth             │
│                                                      │
│  Storage:    1× Samsung PM9D3 3.84TB (WAL, FDP)      │
│              4× Samsung PM9D3 7.68TB (Data, FDP)     │
│              PCIe Gen5 × 4 each                       │
│              Bifurcated x16 slot for 4× data SSDs    │
│                                                      │
│  Network:    1× NVIDIA ConnectX-7 (2×100GbE RoCEv2)  │
│              Port 1: Client traffic (AF_XDP)          │
│              Port 2: Replication (RDMA one-sided)     │
│              PTP hardware clock (/dev/ptp0)           │
│                                                      │
│  Optional:   1× AMD Pensando DSC-200 (DPU)           │
│              TLS termination + packet filtering       │
│                                                      │
│  CXL:        Optional 1× Samsung CMM-D 128GB         │
│              Warm buffer pool tier (~150ns)            │
│              Connected via P-link (CXL 2.0)           │
└────────────────────────────────────────────────────┘
```

### 19.2 AMD vs Intel: Bandwidth Advantage

| Metric | Intel SPR (Xeon 4th Gen) | AMD EPYC 9005 (Turin) |
|--------|------------------------|----------------------|
| DDR5 Channels | 8 | **12** (+50%) |
| Peak DDR5 Bandwidth | ~384 GB/s | **~576 GB/s** (+50%) |
| PCIe Gen5 Lanes | 80 | **128** (+60%) |
| CXL Lanes | 16 (dedicated) | **64** (via P-links) |
| Cores (max) | 60 (Xeon w9-3595X) | **128** (EPYC 9754) |
| Threads (max) | 120 | **256** |

**VectorVolt on AMD gets 50% more memory bandwidth and 60% more PCIe lanes** — critical for a memory-bound OLTP engine with many NVMe devices.

### 19.3 AMD NUMA-Aware Shard Layout (NPS4)

```
Socket (EPYC 9554, NPS4)
├── NUMA Node 0 (Quadrant 0): Cores 0-15, 3 UMCs, 192 GB DDR5
│   ├── Shard Writer 0 (pinned to favored core in this quadrant)
│   ├── Shard Writer 1
│   ├── Reader threads (same quadrant, distance = 10)
│   └── Memory: All shard 0+1 pages allocated here (numactl --membind=0)
│
├── NUMA Node 1 (Quadrant 1): Cores 16-31, 3 UMCs, 192 GB DDR5
│   ├── Shard Writer 2
│   ├── Shard Writer 3
│   ├── Reader threads
│   └── Memory: All shard 2+3 pages
│
├── NUMA Node 2 (Quadrant 2): Cores 32-47, 3 UMCs, 192 GB DDR5
│   ├── Shard Writer 4
│   ├── Shard Writer 5
│   ├── Reader threads
│   └── Memory: All shard 4+5 pages
│
└── NUMA Node 3 (Quadrant 3): Cores 48-63, 3 UMCs, 192 GB DDR5
    ├── Shard Writer 6
    ├── Shard Writer 7
    ├── Replication thread (RDMA CQ polling)
    └── Memory: All shard 6+7 pages + RDMA registered buffers

NVMe Assignment:
  WAL SSD → connect to quadrant 0's P-link (closest to shard writers 0+1)
  Data SSD 0-3 → spread across quadrants (bifurcated x16 slot)

Network:
  ConnectX-7 → quadrant 3's P-link (dedicated replication quadrant)
```

---

## 20. AMD-Specific Startup Warning Examples <a name="20-amd-warnings"></a>

```
╔══════════════════════════════════════════════════════════════════════╗
║  VectorVolt v0.1.0 — Platform Capability Report                     ║
╠══════════════════════════════════════════════════════════════════════╣
║                                                                      ║
║  ── CPU ──                                                           ║
║  Vendor:     AMD EPYC 9554 (Zen 5, Family 1Ah)                      ║
║  Cores:      64 physical (SMT disabled ✓)                            ║
║  SIMD:       AVX-512 (no throttle) ✓                                 ║
║  DSA:        ✗ Not available (AMD — using AVX-512 DataMover)         ║
║  IAA:        ✗ Not available (AMD — using software compression)      ║
║  SEV-SNP:    Supported, 1006 ASIDs ✓                                 ║
║  VP2INTERSECT: Supported ✓                                           ║
║  Spectre:    Enhanced IBRS ✓                                         ║
║  Paging:     5-Level (LA57) ✓                                        ║
║                                                                      ║
║  ── Topology ──                                                      ║
║  NUMA:       4 nodes (NPS4 ✓)                                        ║
║  Distances:  [10, 12, 12, 12; 12, 10, 12, 12; ...]                  ║
║  Favored:    Cores 2, 18, 35, 50 (CPPC2 preferred)                  ║
║  L3 Cache:   32 MB per CCX (4 CCDs per quadrant)                     ║
║  DDR5:       12 channels @ 6000 MT/s (576 GB/s) ✓                   ║
║                                                                      ║
║  ── Storage ──                                                       ║
║  WAL Device: Samsung PM9D3 (/dev/nvme0n1)                            ║
║    PMR:      ✗ Not available                                         ║
║    ZNS:      ✗ Not available                                         ║
║    FDP:      ✓ 8 Placement Handles                                   ║
║    CMB:      ✗ Not available                                         ║
║  CXL:        ✓ Samsung CMM-D 128GB on NUMA node 4 (~150ns)          ║
║                                                                      ║
║  ── Network ──                                                       ║
║  NIC:        ConnectX-7 (mlx5_0) — 2×100GbE                         ║
║  RDMA:       RoCEv2 ✓ (200 Gbps)                                    ║
║  AF_XDP:     ✓ (mlx5 zero-copy)                                     ║
║  PTP:        ✓ (/dev/ptp0)                                           ║
║  Pensando:   ✗ Not detected                                          ║
║                                                                      ║
║  ── Platform ──                                                      ║
║  HugePages:  128 × 1GB (128 GB, 32 per NUMA node) ✓                 ║
║  THP:        Disabled ✓                                              ║
║  C-States:   Disabled (Global C-State Control) ✓                     ║
║  Governor:   performance (amd-pstate driver) ✓                       ║
║  Deploy:     Bare Metal                                              ║
║                                                                      ║
╠══════════════════════════════════════════════════════════════════════╣
║  SELECTED PROVIDERS (AMD path)                                       ║
╠══════════════════════════════════════════════════════════════════════╣
║  DataMover:      Avx512DataMover (Zen 5, no throttle)                ║
║  WAL Writer:     FdpWalWriter (FDP + software CRC)                   ║
║  Tenant Isolation: SevSnpIsolation (1006 ASIDs)                      ║
║  Client I/O:     AfXdpTransport (mlx5 zero-copy)                     ║
║  Replication:    RdmaReplicator (RoCEv2, one-sided writes)           ║
║  Buffer Pool:    TieredHugePageBufferPool (DDR5 hot + CXL warm)      ║
║  Clock:          PtpHardwareClock (/dev/ptp0)                        ║
║  Checksum:       HardwareCrcProvider (SSE4.2 + PCLMULQDQ)           ║
║  Encryption:     AesNiGcmProvider (AES-NI + PCLMULQDQ)              ║
║  Compression:    SoftwareDeflateCompressor (zstd)                    ║
║  Core Selector:  FavoredCoreSelector (CPPC2 rankings)                ║
║                                                                      ║
╠══════════════════════════════════════════════════════════════════════╣
║  ⚠ WARNINGS (0 critical, 2 info)                                   ║
╠══════════════════════════════════════════════════════════════════════╣
║  [INFO]  NO_DSA — AMD platform: no Intel DSA. Using AVX-512.        ║
║  [INFO]  NO_IAA — AMD platform: no Intel IAA. Using software zstd.  ║
║                                                                      ║
╠══════════════════════════════════════════════════════════════════════╣
║  ESTIMATED LATENCY BUDGET                                            ║
╠══════════════════════════════════════════════════════════════════════╣
║  Client packet receive (AF_XDP):        ~5 μs                        ║
║  Transaction processing (engine):       ~1-5 μs                      ║
║  WAL commit (FDP + software CRC):       ~10-12 μs                    ║
║  Replication (RDMA one-sided write):    ~2-4 μs                      ║
║  Client response (AF_XDP):              ~5 μs                        ║
║  ─────────────────────────────────────────────                       ║
║  TOTAL estimated replicated commit:     ~23-31 μs                    ║
║                                                                      ║
║  NOTE: AMD matches Intel on this workload because the dominant       ║
║  latency is storage (NVMe) + network (RDMA), not CPU compute.        ║
║  DSA absence costs ~0-2μs on CRC; compensated by AMD's 50% more     ║
║  memory bandwidth for buffer pool operations.                        ║
╚══════════════════════════════════════════════════════════════════════╝
```

---

## 21. AMD-Specific New Classes <a name="21-amd-new-classes"></a>

### AMD-Only Classes (not in Intel handover)
```
AmdCppcPreferredCores       — Read /sys/devices/system/cpu/cpu*/cpufreq/amd_pstate_prefcore_ranking
AmdNpsDetector              — Infer NPS setting from NUMA node count + SLIT distances
AmdPensandoOffloader        — AMD Pensando DSC-200 P4-based DPU offload management
AmdViIommuConfig            — AMD-Vi IOMMU configuration validation + PASID setup
AmdSevSnpManager            — SEV-SNP ASID lifecycle management via /dev/sev + KVM ioctls
AmdPstateConfigurator       — amd-pstate driver validation + EPP tuning
```

### Shared Classes (vendor-neutral, same implementation on Intel and AMD)
```
StorageCapabilities, PmrWalWriter, ZnsWalWriter, FdpWalWriter, CmbQueueManager
CxlMemoryManager, CxlPmemWalWriter, CxlBufferPoolTier
RdmaReplicator, KernelBypassTransport, FlowSteeringManager
DistributedClock, PacketFilterEngine
PlatformTuningValidator, HugePageAllocator
SiliconManifest, ProviderFactory, StartupReporter (all probers)
```

**Total class count for AMD path: ~50 classes** (6 AMD-specific + ~44 vendor-neutral shared with Intel)

---

## Source References (All Parts)

| Category | Key Sources |
|----------|------------|
| **Part 1 — AMD CPU Silicon** | |
| EPYC 9005 Architecture | 58462_amd-epyc-9005-tg-architecture-overview.pdf (11pp) |
| SME/SEV | memory-encryption-white-paper.pdf (12pp) |
| NUMA Milan | 56795_1.00_NUMA_Milan_PUB.pdf (18pp) |
| NUMA Naples | 56308_0.70_NUMA_Naples.pdf (7pp) |
| SEV Enablement | 2025_AMD_Whitepaper_SEV_Enablement_Guide.pdf (52pp) |
| K8s Tuning | 58478_amd-epyc-9005-tg-containers-on-kubernetes.pdf (20pp) |
| AMD APM Vol 2 | amd-manual.pdf (782pp) |
| Lenovo SEV-SNP | lp1893.pdf (13pp), lp1894.pdf (16pp) |
| Dell NUMA | dell-emc-dfd-numa-amd-epyc-2ndgen.pdf (3pp) |
| **Part 2 — Beyond CPU** | |
| NVMe | NVMe Base Specification 2.0, TP4053 (ZNS), TP4146 (FDP) |
| CXL | CXL Specification 2.0/3.0 |
| RDMA | rdma-core, NVIDIA RDMA Programming Guide |
| AMD Pensando | AMD Pensando DSC-200 datasheet, Pollara 400 brief |
| Platform Tuning | AMD EPYC 9005 Tuning Guide, AMD CBS Reference |
| DDR5 | JEDEC DDR5 Spec, AMD UMC documentation |
| **Part 3 — Runtime & Deployment** | |
| K8s CPU Manager | kubernetes.io/docs/tasks/administer-cluster/cpu-management-policies |
| K8s Topology Manager | kubernetes.io/docs/tasks/administer-cluster/topology-manager |
| Confidential Containers | github.com/confidential-containers (Kata + SEV-SNP) |
| AMD CPPC2 | AMD PPR for EPYC 9005, ACPI 6.5 CPPC specification |
| amd-pstate driver | kernel.org/doc/html/latest/admin-guide/pm/amd-pstate.html |

---

*Complete AMD handover generated across 3 research sessions on 2026-04-14. For the full runtime architecture (SiliconManifest, ProviderFactory, interfaces, degradation chains, K8s pod specs, Helm chart), see the Intel handover document Sections 25-35 — this AMD document is a companion that covers AMD-specific differences only.*
