#!/usr/bin/env python3
"""
VectorVolt Requirements Traceability Matrix Generator
-----------------------------------------------------
Reads catalog.yaml, verifies that referenced code files exist, computes
coverage metrics, and writes TRACEABILITY.md alongside itself.

Usage:
    python docs/traceability/generate.py [--root <repo-root>] [--strict]

Options:
    --root      Path to the repository root (default: two directories up
                from this script, i.e. the VectorVolt repo root)
    --strict    Exit 1 if any *implemented* requirement references a code
                file that does not exist on disk (used in CI)

Exit codes:
    0  All checks passed
    1  One or more verification failures (only in --strict mode)
"""

import argparse
import sys
from collections import defaultdict
from pathlib import Path

try:
    import yaml
except ImportError:
    print("ERROR: pyyaml is required.  Install with:  pip install pyyaml", file=sys.stderr)
    sys.exit(2)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

STATUS_EMOJI = {
    "draft":        "⬜",
    "in_progress":  "🔵",
    "implemented":  "✅",
    "verified":     "🏆",
}

PHASE_ORDER = [
    "vectorvolt-1",
    "vectorvolt-2",
    "vectorvolt-3",
    "vectorvolt-packaging",
    "ekos-4",
    "ekos-5",
    "ekos-6",
]


def load_catalog(path: Path) -> list[dict]:
    with path.open("r", encoding="utf-8") as f:
        return yaml.safe_load(f)


def check_files(root: Path, req: dict) -> list[str]:
    """Return missing code-file paths, but only for implemented/verified requirements.

    draft and in_progress requirements are expected to have missing files —
    flagging them would produce environment-dependent output (different branches
    have different files checked out) and break the up-to-date diff check in CI.
    """
    if req.get("status") not in ("implemented", "verified"):
        return []
    missing = []
    for rel in req.get("code") or []:
        if not (root / rel).exists():
            missing.append(rel)
    return missing


def coverage_pct(reqs: list[dict], statuses: tuple[str, ...]) -> float:
    total = len(reqs)
    if total == 0:
        return 0.0
    matched = sum(1 for r in reqs if r.get("status") in statuses)
    return (matched / total) * 100.0


def group_by_phase(reqs: list[dict]) -> dict[str, list[dict]]:
    groups: dict[str, list[dict]] = defaultdict(list)
    for r in reqs:
        groups[r.get("phase", "unknown")].append(r)
    return groups


# ---------------------------------------------------------------------------
# Markdown builders
# ---------------------------------------------------------------------------

def badge(pct: float) -> str:
    if pct >= 80:
        colour = "brightgreen"
    elif pct >= 50:
        colour = "yellow"
    else:
        colour = "red"
    return f"![{pct:.0f}%](https://img.shields.io/badge/coverage-{pct:.0f}%25-{colour})"


def render_dashboard(reqs: list[dict]) -> str:
    total     = len(reqs)
    draft     = sum(1 for r in reqs if r["status"] == "draft")
    in_prog   = sum(1 for r in reqs if r["status"] == "in_progress")
    impl      = sum(1 for r in reqs if r["status"] == "implemented")
    verified  = sum(1 for r in reqs if r["status"] == "verified")
    done_pct  = coverage_pct(reqs, ("implemented", "verified"))
    vest_pct  = coverage_pct(reqs, ("verified",))

    lines = [
        "## Dashboard",
        "",
        f"| Metric | Value |",
        f"|--------|-------|",
        f"| Total requirements | {total} |",
        f"| Draft | {draft} |",
        f"| In Progress | {in_prog} |",
        f"| Implemented | {impl} |",
        f"| Verified | {verified} |",
        f"| Implementation coverage | **{done_pct:.1f}%** |",
        f"| Verification coverage | **{vest_pct:.1f}%** |",
        "",
    ]
    return "\n".join(lines)


def render_phase_section(phase: str, reqs: list[dict], root: Path, missing_map: dict) -> str:
    lines = [
        f"### Phase: `{phase}`",
        "",
        "| ID | Title | Status | Tests | Code |",
        "|----|-------|--------|-------|------|",
    ]
    for r in sorted(reqs, key=lambda x: x["id"]):
        rid     = r["id"]
        title   = r["title"]
        status  = r.get("status", "draft")
        emoji   = STATUS_EMOJI.get(status, "❓")
        tests   = r.get("tests") or []
        code    = r.get("code") or []

        missing = missing_map.get(rid, [])
        code_cell = ", ".join(
            f"`{c}` ⚠️" if c in missing else f"`{c}`"
            for c in code
        ) or "—"
        test_count = f"{len(tests)} test(s)" if tests else "—"
        lines.append(f"| `{rid}` | {title} | {emoji} {status} | {test_count} | {code_cell} |")

    lines.append("")
    return "\n".join(lines)


def render_gaps(missing_map: dict, reqs: list[dict]) -> str:
    gap_reqs = [r for r in reqs if missing_map.get(r["id"])]
    if not gap_reqs:
        return "## Gaps\n\nNo gaps detected — all referenced code files exist.\n"

    lines = [
        "## Gaps",
        "",
        "The following requirements reference source files that were **not found** on disk.",
        "This may indicate the file has been renamed, the path in catalog.yaml is wrong,",
        "or the code is not yet written.",
        "",
        "| ID | Status | Missing File |",
        "|----|--------|--------------|",
    ]
    for r in gap_reqs:
        for m in missing_map[r["id"]]:
            lines.append(f"| `{r['id']}` | {r.get('status','?')} | `{m}` |")
    lines.append("")
    return "\n".join(lines)


def render_draft_backlog(reqs: list[dict]) -> str:
    drafts = [r for r in reqs if r.get("status") in ("draft", "in_progress")]
    if not drafts:
        return "## Backlog\n\nNo draft or in-progress requirements.\n"

    lines = [
        "## Backlog (draft / in-progress)",
        "",
        "| ID | Phase | Title |",
        "|----|-------|-------|",
    ]
    for r in sorted(drafts, key=lambda x: (PHASE_ORDER.index(x.get("phase","")) if x.get("phase","") in PHASE_ORDER else 99, x["id"])):
        lines.append(f"| `{r['id']}` | `{r.get('phase','?')}` | {r['title']} |")
    lines.append("")
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    script_dir = Path(__file__).resolve().parent
    default_root = script_dir.parent.parent   # docs/traceability → docs → repo root

    parser = argparse.ArgumentParser(description="Generate TRACEABILITY.md")
    parser.add_argument("--root",   default=str(default_root),
                        help="Repository root (default: %(default)s)")
    parser.add_argument("--strict", action="store_true",
                        help="Exit 1 on missing code files for implemented requirements")
    args = parser.parse_args()

    root    = Path(args.root).resolve()
    catalog = script_dir / "catalog.yaml"
    output  = script_dir / "TRACEABILITY.md"

    reqs = load_catalog(catalog)

    # ── File verification ────────────────────────────────────────────────────
    missing_map: dict[str, list[str]] = {}
    strict_failures = []
    for r in reqs:
        missing = check_files(root, r)
        if missing:
            missing_map[r["id"]] = missing
            if args.strict and r.get("status") in ("implemented", "verified"):
                strict_failures.extend(missing)

    # ── Build groups ─────────────────────────────────────────────────────────
    groups = group_by_phase(reqs)

    # ── Render ───────────────────────────────────────────────────────────────
    sections = [
        "# Requirements Traceability Matrix\n",
        "_Auto-generated by `docs/traceability/generate.py`._",
        "_Do not edit this file manually — edit `catalog.yaml` instead._\n",
        render_dashboard(reqs),
    ]

    sections.append("## Requirements by Phase\n")
    for phase in PHASE_ORDER:
        if phase in groups:
            sections.append(render_phase_section(phase, groups[phase], root, missing_map))

    # Phases not in our ordering (future-proofing)
    for phase, phase_reqs in sorted(groups.items()):
        if phase not in PHASE_ORDER:
            sections.append(render_phase_section(phase, phase_reqs, root, missing_map))

    sections.append(render_gaps(missing_map, reqs))
    sections.append(render_draft_backlog(reqs))

    md = "\n".join(sections)
    output.write_text(md, encoding="utf-8")
    print(f"Wrote {output}")

    # ── Coverage summary ─────────────────────────────────────────────────────
    impl_pct = coverage_pct(reqs, ("implemented", "verified"))
    print(f"Implementation coverage: {impl_pct:.1f}%  ({len(reqs)} requirements total)")
    if missing_map:
        print(f"WARNING: {len(missing_map)} requirement(s) reference missing files", file=sys.stderr)
    if strict_failures:
        print(f"STRICT FAIL: {len(strict_failures)} missing code file(s) for implemented requirements", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
