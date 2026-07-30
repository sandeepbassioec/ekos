#!/usr/bin/env bash
# vv-setup-linux.sh — install the toolchain, build VectorVolt, capture a baseline.
#
# WHAT THIS CHANGES (unlike vv-probe-linux.sh, this one does write):
#   - installs rustup + the stable Rust toolchain into $HOME/.cargo  (per-user, not system)
#   - builds the VectorVolt workspace in release mode (writes to vectorvolt/target/)
#   - writes benchmark output to ./vv-baseline-<hostname>.txt
# It installs no system packages, touches no kernel settings, and needs no root.
#
# It does NOT: change sysctls, set huge pages, alter boot params, or open ports.
# Those are separate and deliberate (see the OPTIONAL section printed at the end).
#
# USAGE
#   # 1. Get the repo onto the box first, using your own git credentials:
#   #      git clone https://github.com/sandeepbassioec/ekos.git
#   #    (or scp it over — this script does not handle auth)
#   # 2. Then, from anywhere:
#   bash vv-setup-linux.sh /path/to/ekos
#
#   Defaults to ./ekos if no path is given.

set -euo pipefail

REPO="${1:-./ekos}"
OUT="vv-baseline-$(hostname).txt"

die() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }
step() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }

[ -d "$REPO" ] || die "repo not found at '$REPO'. Clone it first, then pass its path."
[ -f "$REPO/vectorvolt/Cargo.toml" ] || die "'$REPO' does not look like the ekos repo (no vectorvolt/Cargo.toml)."
REPO="$(cd "$REPO" && pwd)"

# ── 1. Rust toolchain ─────────────────────────────────────────────────────────
step "Rust toolchain"
if command -v cargo >/dev/null 2>&1; then
  echo "cargo already present: $(cargo --version)"
else
  echo "Installing rustup (per-user, into \$HOME/.cargo)..."
  # -y: non-interactive. --profile minimal: no docs/IDE extras we don't need.
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable
fi
# shellcheck disable=SC1091
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
command -v cargo >/dev/null 2>&1 || die "cargo still not on PATH; open a new shell and re-run."

RUSTV=$(rustc --version | awk '{print $2}')
echo "rustc $RUSTV"
# Edition 2024 needs >= 1.85.
case "$RUSTV" in
  1.8[5-9]*|1.9*|1.[1-9][0-9][0-9]*|2.*) : ;;
  *) die "rustc $RUSTV is too old — edition 2024 needs >= 1.85. Run: rustup update stable" ;;
esac

# ── 2. Build ──────────────────────────────────────────────────────────────────
# NOTE: cd into vectorvolt/ rather than using --manifest-path. Cargo reads
# .cargo/config.toml from the cwd upward, and building from the repo root would
# silently skip it. (On Windows that breaks linking outright; here it is merely
# inconsistent, but same command shape everywhere is one less thing to debug.)
step "Building release binaries"
cd "$REPO/vectorvolt"
cargo build --release 2>&1 | tail -5

VVCTL="$REPO/vectorvolt/target/release/vvctl"
[ -x "$VVCTL" ] || die "build produced no vvctl at $VVCTL"

# ── 3. Test suite ─────────────────────────────────────────────────────────────
step "Test suite"
cargo test --workspace 2>&1 | grep -E "test result:" | \
  awk -F'[ ;]' '{p+=$4; f+=$7} END {printf "  %d passed, %d failed\n", p, f}'

# ── 4. Baseline capture ───────────────────────────────────────────────────────
step "Capturing baseline to $OUT"
cd - >/dev/null
{
  echo "VectorVolt Linux baseline"
  echo "host:    $(hostname)"
  echo "kernel:  $(uname -r)"
  echo "cpu:     $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs)"
  echo "rustc:   $RUSTV"
  echo "date:    $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  echo

  echo "########## HAL capabilities + SIMD speedup ##########"
  # Sweep working-set sizes. This box has a 35.8 MiB L3, so unlike a typical
  # desktop the 1M-element (8 MB) case is still cache-resident. The 8M-element
  # (64 MB) case is what actually exceeds L3 here — that is the one that shows
  # the true DRAM-bound speedup.
  for n in 1024 8192 65536 1000000 8000000; do
    kb=$(( n * 8 / 1024 ))
    echo
    echo "--- rows=$n (${kb} KB) ---"
    "$VVCTL" hal --rows "$n" --iters 50 2>&1
  done

  echo
  echo "########## End-to-end CRUD latency ##########"
  # Default temp dir (EBS root volume). See the note about instance-store NVMe
  # below — WAL fsync numbers from EBS are not the best this box can do.
  "$VVCTL" bench --rows 100000 --warmup 1000 2>&1

  echo
  echo "########## Storage context ##########"
  echo "TMPDIR=${TMPDIR:-/tmp}  (where the bench wrote its WAL)"
  df -h "${TMPDIR:-/tmp}" 2>/dev/null | tail -1
  lsblk -d -o NAME,SIZE,ROTA,MODEL 2>/dev/null | grep -E 'nvme|NAME'
} | tee "$OUT"

# ── 5. What to do next ────────────────────────────────────────────────────────
cat <<'NEXT'

============================================================
Baseline captured. Paste the file contents back.

OPTIONAL — only if you want to, each is a deliberate change:

1) Benchmark on the LOCAL instance-store NVMe instead of EBS.
   g4dn ships a 209 GB instance-store NVMe (nvme1n1) that is much faster and
   far more consistent for fsync than the EBS root volume. WAL latency is
   dominated by fsync, so EBS numbers understate what this box can do.
   NOTE: instance-store data is LOST on stop/terminate. Scratch only.

     sudo mkfs.ext4 -F /dev/nvme1n1
     sudo mkdir -p /mnt/nvme && sudo mount /dev/nvme1n1 /mnt/nvme
     sudo chown "$USER" /mnt/nvme
     TMPDIR=/mnt/nvme <path>/vvctl bench --rows 100000 --warmup 1000

2) Reserve 2 MB huge pages (root, no reboot, reversible).
   Needed for VV-HAL-031. 2048 pages = 4 GB of the 62 GB on this box.

     sudo sysctl vm.nr_hugepages=2048
     grep HugePages /proc/meminfo
     # revert:  sudo sysctl vm.nr_hugepages=0

3) Raise the memlock limit for mlock() work (VV-HAL-033).
   Currently ~7.8 GB, which is workable; 'unlimited' is ideal.

     ulimit -l unlimited     # this shell only, needs root

DO NOT bother with isolcpus yet — it needs a boot param plus a reboot, and
nothing in the codebase pins threads yet (VV-HAL-024 is unimplemented). It
would be ceremony with no measurable effect. Worth doing in S1b, not now.

NOT POSSIBLE on this instance, for the record:
  - NUMA node binding  — 1 socket, 1 node. Needs a 2-socket box.
  - NEON / ARM PRFM    — needs Graviton.
  - DPDK               — needs vfio-pci plus real setup; ENA PMD is capable
                         but this is not a quick win.
============================================================
NEXT
