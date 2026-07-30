#!/usr/bin/env bash
# vv-probe-linux.sh — read-only capability probe for a VectorVolt test host.
#
# WHAT THIS DOES: reports what the machine can support. Nothing else.
# WHAT THIS DOES NOT DO: it makes no changes, installs nothing, writes no files,
# opens no ports, and needs no root. Every command below is a read. You can read
# the whole thing in a minute before running it.
#
# USAGE
#   bash vv-probe-linux.sh              # human-readable
#   bash vv-probe-linux.sh > box1.txt   # capture to paste back
#
# Run it on BOTH boxes. The differences between them matter as much as the
# absolute values.

set -u  # deliberately NOT -e: a missing optional tool should not abort the probe

say()  { printf '%s\n' "$*"; }
hdr()  { printf '\n== %s %s\n' "$*" "$(printf '=%.0s' $(seq 1 $((60 - ${#1} - 4))))"; }
kv()   { printf '  %-28s %s\n' "$1" "$2"; }
have() { command -v "$1" >/dev/null 2>&1; }

say "VectorVolt host capability probe"
say "generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ') (UTC)"

# ── Identity ──────────────────────────────────────────────────────────────────
hdr "HOST IDENTITY"
kv "hostname"        "$(hostname 2>/dev/null || echo '?')"
kv "kernel"          "$(uname -r)"
kv "arch"            "$(uname -m)"
if [ -r /etc/os-release ]; then
  # shellcheck disable=SC1091
  . /etc/os-release
  kv "distro" "${PRETTY_NAME:-$NAME $VERSION_ID}"
fi

# EC2 instance metadata. IMDSv2 needs a token; try it, fall back to IMDSv1.
# This is a link-local address (169.254.169.254) — not an external network call.
if have curl; then
  TOK=$(curl -s -X PUT --max-time 2 \
        -H "X-aws-ec2-metadata-token-ttl-seconds: 60" \
        http://169.254.169.254/latest/api/token 2>/dev/null)
  MD="http://169.254.169.254/latest/meta-data"
  if [ -n "${TOK}" ]; then
    ITYPE=$(curl -s --max-time 2 -H "X-aws-ec2-metadata-token: ${TOK}" "$MD/instance-type" 2>/dev/null)
    AZ=$(curl -s --max-time 2 -H "X-aws-ec2-metadata-token: ${TOK}" "$MD/placement/availability-zone" 2>/dev/null)
    PRIV=$(curl -s --max-time 2 -H "X-aws-ec2-metadata-token: ${TOK}" "$MD/local-ipv4" 2>/dev/null)
  else
    ITYPE=$(curl -s --max-time 2 "$MD/instance-type" 2>/dev/null)
    AZ=$(curl -s --max-time 2 "$MD/placement/availability-zone" 2>/dev/null)
    PRIV=$(curl -s --max-time 2 "$MD/local-ipv4" 2>/dev/null)
  fi
  kv "ec2 instance type"  "${ITYPE:-not on EC2 / metadata blocked}"
  kv "ec2 az"             "${AZ:-?}"
  kv "ec2 private ip"     "${PRIV:-?}"

  case "${ITYPE:-}" in
    t2.*|t3.*|t3a.*|t4g.*)
      say ""
      say "  !! BURSTABLE INSTANCE. CPU-credit throttling and hypervisor steal"
      say "     time make nanosecond latency numbers from this host unreliable."
      say "     Fine for correctness and cluster testing; do NOT publish"
      say "     benchmark figures measured here." ;;
    *.metal)
      say ""
      say "  ++ BARE METAL. No hypervisor steal, real PMU access, huge pages and"
      say "     isolcpus fully effective. This is the good case for benchmarks." ;;
  esac
fi

# ── Privilege ─────────────────────────────────────────────────────────────────
hdr "PRIVILEGE (needed for huge pages, isolcpus, XDP, DPDK)"
kv "uid" "$(id -u) ($(id -un))"
if [ "$(id -u)" -eq 0 ]; then
  kv "root" "yes — running as root"
elif sudo -n true 2>/dev/null; then
  kv "root" "yes — passwordless sudo available"
else
  kv "root" "unknown — 'sudo -n' failed (may still work interactively)"
fi

# ── CPU topology ──────────────────────────────────────────────────────────────
hdr "CPU TOPOLOGY (VV-HAL-001)"
if have lscpu; then
  lscpu 2>/dev/null | grep -E \
    '^(Architecture|Model name|CPU\(s\)|Thread\(s\) per core|Core\(s\) per socket|Socket\(s\)|NUMA node\(s\)|CPU max MHz|CPU MHz|L1d|L1i|L2|L3|Flags)' \
    | sed 's/^/  /' | cut -c1-160
else
  kv "lscpu" "absent — install util-linux for full detail"
  kv "logical cpus" "$(nproc 2>/dev/null || echo '?')"
fi

hdr "SIMD FEATURE FLAGS (VV-HAL-004..008)"
for f in sse4_2 avx avx2 avx512f avx512dq avx512bw bmi2 popcnt; do
  if grep -qm1 " $f" /proc/cpuinfo 2>/dev/null; then
    kv "$f" "present"
  else
    kv "$f" "-"
  fi
done
say "  (avx2 + sse4_2 are what our current kernels use. avx512f would let us"
say "   enable the VV-HAL-007 feature flag; absence is fine.)"

# ── Invariant TSC — required for our cycle timer to be trustworthy ────────────
hdr "TIMESTAMP COUNTER (VV-HAL-037)"
for f in constant_tsc nonstop_tsc tsc_reliable rdtscp; do
  if grep -qm1 " $f" /proc/cpuinfo 2>/dev/null; then kv "$f" "present"; else kv "$f" "-"; fi
done
kv "clocksource" "$(cat /sys/devices/system/clocksource/clocksource0/current_clocksource 2>/dev/null || echo '?')"
say "  (constant_tsc + nonstop_tsc mean RDTSC ticks at a fixed rate regardless"
say "   of turbo/idle — without them our ns conversion drifts.)"

# ── NUMA ──────────────────────────────────────────────────────────────────────
hdr "NUMA (VV-HAL-028..030)"
if [ -d /sys/devices/system/node ]; then
  NODES=$(ls -d /sys/devices/system/node/node[0-9]* 2>/dev/null | wc -l)
  kv "numa nodes" "$NODES"
  if have numactl; then
    numactl --hardware 2>/dev/null | sed 's/^/  /'
  else
    kv "numactl" "absent (optional — we bind via mbind() directly)"
    for n in /sys/devices/system/node/node[0-9]*; do
      [ -r "$n/cpulist" ] && kv "$(basename "$n") cpus" "$(cat "$n/cpulist")"
    done
  fi
  [ "$NODES" -le 1 ] && say "  Single NUMA node: node-binding work is untestable here (not a blocker —
  most single-socket cloud instances look like this)."
else
  kv "numa" "not exposed by kernel"
fi

# ── Huge pages ────────────────────────────────────────────────────────────────
hdr "HUGE PAGES (VV-HAL-031)"
grep -E '^(HugePages_Total|HugePages_Free|HugePages_Rsvd|Hugepagesize|AnonHugePages)' \
  /proc/meminfo 2>/dev/null | sed 's/^/  /'
kv "THP setting" "$(cat /sys/kernel/mm/transparent_hugepage/enabled 2>/dev/null || echo '?')"
if have ulimit; then kv "memlock limit (KB)" "$(ulimit -l 2>/dev/null || echo '?')"; fi
say "  If HugePages_Total is 0 we can still use transparent huge pages, but"
say "  explicit MAP_HUGETLB needs: sudo sysctl vm.nr_hugepages=N  (root, no reboot)."
say "  mlock() needs memlock limit raised — 'unlimited' is ideal."

# ── Core isolation ────────────────────────────────────────────────────────────
hdr "CORE ISOLATION (VV-HAL-035 — needs boot params + reboot)"
kv "cmdline" "$(tr -s ' ' <  /proc/cmdline 2>/dev/null | cut -c1-140)"
for p in isolcpus nohz_full rcu_nocbs; do
  if grep -qm1 "$p" /proc/cmdline 2>/dev/null; then kv "$p" "SET"; else kv "$p" "not set"; fi
done
kv "isolated cpus" "$(cat /sys/devices/system/cpu/isolated 2>/dev/null || echo 'none')"

# ── Async / bypass I/O ────────────────────────────────────────────────────────
hdr "ASYNC + KERNEL-BYPASS I/O (VV-WAL-017, VV-NET-011..014)"
KMAJ=$(uname -r | cut -d. -f1); KMIN=$(uname -r | cut -d. -f2)
kv "kernel version" "${KMAJ}.${KMIN}"
if [ "$KMAJ" -gt 5 ] || { [ "$KMAJ" -eq 5 ] && [ "$KMIN" -ge 1 ]; }; then
  kv "io_uring (>=5.1)" "supported"
  if [ "$KMAJ" -gt 5 ] || { [ "$KMAJ" -eq 5 ] && [ "$KMIN" -ge 10 ]; }; then
    kv "io_uring maturity" "5.10+ — good"
  else
    kv "io_uring maturity" "5.1-5.9 — usable, early API"
  fi
else
  kv "io_uring" "NOT supported (needs >=5.1)"
fi
if [ "$KMAJ" -gt 4 ] || { [ "$KMAJ" -eq 4 ] && [ "$KMIN" -ge 18 ]; }; then
  kv "AF_XDP/eBPF (>=4.18)" "supported"
else
  kv "AF_XDP/eBPF" "NOT supported (needs >=4.18)"
fi
kv "bpf syscall" "$([ -e /proc/sys/kernel/unprivileged_bpf_disabled ] && echo present || echo '?')"
kv "hugetlb for DPDK" "$([ -d /dev/hugepages ] && echo '/dev/hugepages present' || echo 'absent')"
kv "vfio-pci module" "$(lsmod 2>/dev/null | grep -cm1 vfio_pci | sed 's/^0$/not loaded/;s/^1$/loaded/' || echo '?')"

# ── NIC ───────────────────────────────────────────────────────────────────────
hdr "NETWORK INTERFACES (DPDK/XDP viability)"
if have ip; then
  ip -br link 2>/dev/null | sed 's/^/  /'
  for i in $(ls /sys/class/net 2>/dev/null); do
    [ "$i" = "lo" ] && continue
    DRV=$(basename "$(readlink -f "/sys/class/net/$i/device/driver" 2>/dev/null)" 2>/dev/null)
    kv "$i driver" "${DRV:-?}"
    [ "$DRV" = "ena" ] && say "    ena = AWS Elastic Network Adapter. DPDK-capable via the ENA PMD, and
    XDP works in generic (skb) mode. Native XDP needs driver support."
  done
else
  kv "ip" "absent"
fi

# ── Memory + storage ──────────────────────────────────────────────────────────
hdr "MEMORY / STORAGE"
if have free; then free -h 2>/dev/null | sed 's/^/  /'; fi
if have lsblk; then lsblk -d -o NAME,SIZE,ROTA,MODEL 2>/dev/null | sed 's/^/  /'; fi
say "  ROTA=0 means SSD/NVMe. WAL fsync latency depends heavily on this."

# ── Toolchain ─────────────────────────────────────────────────────────────────
hdr "BUILD TOOLCHAIN"
for t in rustc cargo gcc cc make pkg-config git perf; do
  if have "$t"; then
    kv "$t" "$("$t" --version 2>/dev/null | head -1 | cut -c1-60)"
  else
    kv "$t" "ABSENT"
  fi
done
say "  rustc >= 1.85 needed (we use edition 2024). perf is optional but is how"
say "  we would profile cache misses (VV-OBS-010)."

# ── Peer reachability (for 2-node cluster work) ───────────────────────────────
hdr "PEER REACHABILITY (VV-CLU-*, needs both boxes)"
say "  To measure inter-node RTT — which sets the floor on replication latency"
say "  and Raft election time — run this on box A with box B's PRIVATE ip:"
say ""
say "      ping -c 20 -i 0.2 <PEER_PRIVATE_IP> | tail -3"
say ""
say "  Same-AZ private RTT should be ~0.1-0.5 ms. Cross-AZ is ~1-2 ms, which"
say "  would dominate our <2s failover budget's election phase."

hdr "END OF PROBE"
say "Nothing on this host was modified. Paste the output back to continue."
