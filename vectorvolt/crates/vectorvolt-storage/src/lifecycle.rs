//! Partition lifecycle: compaction planning, retention, and verification
//! (VV-STO-015/016/020).
//!
//! Everything here operates on **partition metadata**, never on partition
//! contents. A plan is a pure function of a `&[PartitionMeta]` slice, so it can be
//! computed, logged, diffed, and tested without touching a byte of data — and a
//! caller can inspect what would happen before letting it happen. Executing a plan
//! is the engine's job.
//!
//! That split is deliberate. Compaction and retention are the two operations in a
//! storage engine that **destroy data**, and the failure mode is not a crash but a
//! quiet absence. Making the decision separable from the deletion means the
//! decision can be unit-tested exhaustively, which is what the tests at the bottom
//! of this file do.
//!
//! ## No hidden clock
//!
//! Retention takes `now_millis` as an argument rather than reading the system
//! clock. Age-based deletion whose input is ambient time cannot be tested at
//! boundaries, and a wrong answer there deletes live data.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{PartitionReader, StorageResult};

// ── Partition descriptor ──────────────────────────────────────────────────────

/// What the lifecycle planners know about one partition.
///
/// Deliberately not the partition itself: a plan over 10,000 partitions must not
/// require opening 10,000 files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionMeta {
    pub file_id: u16,
    pub partition_id: u16,
    pub path: PathBuf,
    /// LSM level. 0 is freshly flushed and may overlap other L0 partitions.
    pub level: u8,
    pub size_bytes: u64,
    pub row_count: u32,
    /// Creation time, milliseconds since the Unix epoch.
    pub created_at_millis: u64,
    /// Primary-key range, for overlap detection.
    pub min_key: u64,
    pub max_key: u64,
    /// Rows superseded by a later write or delete. Drives compaction urgency:
    /// a partition that is mostly tombstones is mostly wasted I/O on every scan.
    pub tombstones: u32,
    /// Under a compliance hold. Retention must never delete this, whatever the
    /// policy says.
    pub worm_locked: bool,
}

impl PartitionMeta {
    /// Fraction of rows that are dead. `0.0` for an empty partition.
    pub fn garbage_ratio(&self) -> f64 {
        if self.row_count == 0 {
            return 0.0;
        }
        self.tombstones as f64 / self.row_count as f64
    }

    /// True if this partition's key range overlaps `other`'s.
    pub fn overlaps(&self, other: &PartitionMeta) -> bool {
        self.min_key <= other.max_key && self.max_key >= other.min_key
    }
}

// ── Compaction planning (VV-STO-015) ──────────────────────────────────────────

/// One unit of compaction work: merge these partitions into `output_level`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionJob {
    pub inputs: Vec<PartitionMeta>,
    pub output_level: u8,
    pub reason: CompactionReason,
}

impl CompactionJob {
    pub fn input_bytes(&self) -> u64 {
        self.inputs.iter().map(|p| p.size_bytes).sum()
    }

    pub fn input_rows(&self) -> u64 {
        self.inputs.iter().map(|p| p.row_count as u64).sum()
    }
}

/// Why a job was scheduled. Carried through so an operator reading a compaction
/// log can tell a routine merge from a garbage emergency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionReason {
    /// Enough similarly-sized partitions accumulated (size-tiered).
    SizeRun,
    /// A level exceeded its byte budget (leveled).
    LevelOverflow,
    /// Dead rows crossed [`CompactionStrategy::garbage_threshold`].
    GarbageCollection,
}

/// How to choose what to compact.
///
/// ## The trade this enum exists to make explicit
///
/// | | write amplification | read amplification | space amplification |
/// |---|---|---|---|
/// | Size-tiered | **low** | high | high (up to 2x during merge) |
/// | Leveled | high | **low** | low |
///
/// Size-tiered rewrites each row roughly `log(N)` times but leaves many
/// overlapping partitions, so a point lookup may probe several. Leveled keeps
/// each level non-overlapping — one probe per level — but rewrites far more.
///
/// Neither is correct in general, which is exactly why this is a choice and not a
/// constant. A write-heavy ingest wants size-tiered; a read-heavy serving path
/// wants leveled. VectorVolt's stated target is read latency, so [`Default`] is
/// leveled — but the ingest path may legitimately want otherwise, and hard-coding
/// one would have silently picked a side.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompactionStrategy {
    SizeTiered {
        /// Fewest partitions in a size bucket before merging it.
        min_run: usize,
        /// Most partitions to merge at once. Bounds the peak disk overhead of a
        /// single job.
        max_run: usize,
        /// Partitions within this size factor of each other share a bucket.
        size_ratio: f64,
        /// Merge any single partition whose dead-row fraction exceeds this, even
        /// if no run has formed.
        garbage_threshold: f64,
    },
    Leveled {
        /// L0 partitions tolerated before they are merged into L1. L0 is the only
        /// level allowed to self-overlap, so its size directly costs read latency.
        l0_trigger: usize,
        /// Each level holds this many times more bytes than the one above.
        level_multiplier: u64,
        /// Byte budget for L1. Level `n` gets `l1_bytes * multiplier^(n-1)`.
        l1_bytes: u64,
        garbage_threshold: f64,
    },
}

impl Default for CompactionStrategy {
    fn default() -> Self {
        CompactionStrategy::Leveled {
            l0_trigger: 4,
            level_multiplier: 10,
            l1_bytes: 64 * 1024 * 1024,
            garbage_threshold: 0.30,
        }
    }
}

impl CompactionStrategy {
    /// A size-tiered default tuned for ingest.
    pub fn size_tiered() -> Self {
        CompactionStrategy::SizeTiered {
            min_run: 4,
            max_run: 16,
            size_ratio: 2.0,
            garbage_threshold: 0.30,
        }
    }

    fn garbage_threshold(&self) -> f64 {
        match self {
            CompactionStrategy::SizeTiered {
                garbage_threshold, ..
            }
            | CompactionStrategy::Leveled {
                garbage_threshold, ..
            } => *garbage_threshold,
        }
    }

    /// Byte budget for `level` under a leveled strategy.
    fn level_budget(&self, level: u8) -> Option<u64> {
        match self {
            CompactionStrategy::Leveled {
                level_multiplier,
                l1_bytes,
                ..
            } => {
                if level == 0 {
                    return None; // L0 is bounded by count, not bytes
                }
                Some(l1_bytes.saturating_mul(
                    level_multiplier.saturating_pow(u32::from(level).saturating_sub(1)),
                ))
            }
            CompactionStrategy::SizeTiered { .. } => None,
        }
    }

    /// Compute the jobs to run, most urgent first.
    ///
    /// Returns an empty vector when nothing needs doing — the common case, and one
    /// a caller should be able to detect without inspecting job contents.
    ///
    /// **A partition never appears in two jobs.** Compaction rewrites its inputs,
    /// so scheduling one twice in a round would either double-delete it or race.
    /// Enforced by tracking claimed ids, and asserted in the tests.
    pub fn plan(&self, partitions: &[PartitionMeta]) -> Vec<CompactionJob> {
        let mut jobs = Vec::new();
        let mut claimed: Vec<(u16, u16)> = Vec::new();

        // Garbage first: a partition that is mostly dead rows wastes I/O on every
        // scan that touches it, and rewriting it is cheap because most of what it
        // holds is discarded. Prioritised over routine merges for that reason.
        let gt = self.garbage_threshold();
        for p in partitions {
            if p.garbage_ratio() > gt {
                claimed.push((p.file_id, p.partition_id));
                jobs.push(CompactionJob {
                    inputs: vec![p.clone()],
                    output_level: p.level,
                    reason: CompactionReason::GarbageCollection,
                });
            }
        }

        let free: Vec<&PartitionMeta> = partitions
            .iter()
            .filter(|p| !claimed.contains(&(p.file_id, p.partition_id)))
            .collect();

        match self {
            CompactionStrategy::SizeTiered {
                min_run,
                max_run,
                size_ratio,
                ..
            } => jobs.extend(Self::plan_size_tiered(
                &free,
                *min_run,
                *max_run,
                *size_ratio,
            )),
            CompactionStrategy::Leveled { l0_trigger, .. } => {
                jobs.extend(self.plan_leveled(&free, *l0_trigger))
            }
        }
        jobs
    }

    fn plan_size_tiered(
        free: &[&PartitionMeta],
        min_run: usize,
        max_run: usize,
        size_ratio: f64,
    ) -> Vec<CompactionJob> {
        // Bucket by size: walk ascending and start a new bucket whenever a
        // partition is more than `size_ratio` times the bucket's smallest member.
        let mut sorted: Vec<&PartitionMeta> = free.to_vec();
        sorted.sort_by_key(|p| (p.size_bytes, p.file_id, p.partition_id));

        let mut jobs = Vec::new();
        let mut bucket: Vec<&PartitionMeta> = Vec::new();

        let flush = |bucket: &mut Vec<&PartitionMeta>, jobs: &mut Vec<CompactionJob>| {
            if bucket.len() >= min_run {
                for chunk in bucket.chunks(max_run) {
                    if chunk.len() >= min_run {
                        jobs.push(CompactionJob {
                            inputs: chunk.iter().map(|p| (*p).clone()).collect(),
                            // Size-tiered has no real level hierarchy; the merged
                            // output sits one below its inputs so repeated merges
                            // do not loop on the same level forever.
                            output_level: chunk.iter().map(|p| p.level).max().unwrap_or(0) + 1,
                            reason: CompactionReason::SizeRun,
                        });
                    }
                }
            }
            bucket.clear();
        };

        for p in sorted {
            match bucket.first() {
                Some(first) if (p.size_bytes as f64) <= first.size_bytes as f64 * size_ratio => {
                    bucket.push(p)
                }
                Some(_) => {
                    flush(&mut bucket, &mut jobs);
                    bucket.push(p);
                }
                None => bucket.push(p),
            }
        }
        flush(&mut bucket, &mut jobs);
        jobs
    }

    fn plan_leveled(&self, free: &[&PartitionMeta], l0_trigger: usize) -> Vec<CompactionJob> {
        let mut jobs = Vec::new();
        let mut by_level: HashMap<u8, Vec<&PartitionMeta>> = HashMap::new();
        for p in free {
            by_level.entry(p.level).or_default().push(p);
        }

        // L0 → L1 when too many L0 partitions accumulate. All of L0 goes at once
        // because L0 partitions may overlap each other, so merging a subset would
        // leave the rest still overlapping the new L1 output.
        if let Some(l0) = by_level.get(&0)
            && l0.len() >= l0_trigger
        {
            let mut inputs: Vec<PartitionMeta> = l0.iter().map(|p| (*p).clone()).collect();
            inputs.sort_by_key(|p| (p.file_id, p.partition_id));
            jobs.push(CompactionJob {
                inputs,
                output_level: 1,
                reason: CompactionReason::LevelOverflow,
            });
        }

        // Anything already pulled into a job this round. Tracked across levels,
        // not within one, because an Ln job absorbs overlapping Ln+1 partitions —
        // so by the time the loop reaches Ln+1, some of its members are already
        // spoken for. Without this a partition lands in two jobs of the same
        // round, and since compaction removes its inputs, the second job either
        // double-deletes it or races the first.
        let mut claimed: Vec<(u16, u16)> = jobs
            .iter()
            .flat_map(|j| j.inputs.iter().map(|p| (p.file_id, p.partition_id)))
            .collect();

        // Ln → Ln+1 when a level exceeds its byte budget. One partition is chosen
        // and merged with everything it overlaps in the next level down — the
        // defining move of leveled compaction, and what keeps each level
        // non-overlapping so a lookup probes it once.
        let mut levels: Vec<u8> = by_level.keys().copied().filter(|l| *l > 0).collect();
        levels.sort_unstable();
        for level in levels {
            let Some(budget) = self.level_budget(level) else {
                continue;
            };
            let members: Vec<&&PartitionMeta> = by_level[&level]
                .iter()
                .filter(|p| !claimed.contains(&(p.file_id, p.partition_id)))
                .collect();
            // Budget is measured over the WHOLE level, including partitions
            // already claimed — they still occupy bytes until their job runs.
            let used: u64 = by_level[&level].iter().map(|p| p.size_bytes).sum();
            if used <= budget || members.is_empty() {
                continue;
            }

            // Oldest first: it is the most likely to hold superseded rows, so
            // merging it reclaims the most space per byte rewritten.
            let Some(victim) = members
                .iter()
                .min_by_key(|p| (p.created_at_millis, p.file_id, p.partition_id))
            else {
                continue;
            };

            let mut inputs = vec![(**victim).clone()];
            if let Some(next) = by_level.get(&(level + 1)) {
                for candidate in next {
                    let key = (candidate.file_id, candidate.partition_id);
                    if candidate.overlaps(victim) && !claimed.contains(&key) {
                        inputs.push((*candidate).clone());
                    }
                }
            }
            inputs.sort_by_key(|p| (p.level, p.file_id, p.partition_id));
            claimed.extend(inputs.iter().map(|p| (p.file_id, p.partition_id)));
            jobs.push(CompactionJob {
                inputs,
                output_level: level + 1,
                reason: CompactionReason::LevelOverflow,
            });
        }
        jobs
    }
}

// ── Retention (VV-STO-020) ────────────────────────────────────────────────────

/// A single retention rule. Rules combine with OR: a partition is deletable if
/// **any** rule selects it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionRule {
    /// Delete partitions older than this many milliseconds.
    MaxAgeMillis(u64),
    /// Delete oldest partitions until total size fits.
    MaxTotalBytes(u64),
    /// Delete oldest partitions until at most this many remain.
    MaxPartitions(usize),
}

/// What to do with a partition that retention selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionAction {
    pub partition: PartitionMeta,
    pub rule: RetentionRule,
}

/// The outcome of evaluating a policy. Deletions and refusals are both reported,
/// because "nothing was deleted" and "deletion was blocked by a legal hold" are
/// very different operational facts and must not look alike in a log.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RetentionPlan {
    pub delete: Vec<RetentionAction>,
    /// Selected by a rule but retained anyway, with the reason.
    pub retained: Vec<(PartitionMeta, RetentionVeto)>,
}

/// Why a selected partition was not deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionVeto {
    /// Under a compliance / WORM lock.
    WormLocked,
    /// Deleting it would leave the table with no partitions at all.
    LastPartition,
}

/// A retention policy: a set of rules plus the safety rails.
#[derive(Debug, Clone, Default)]
pub struct RetentionPolicy {
    pub rules: Vec<RetentionRule>,
}

impl RetentionPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rule(mut self, rule: RetentionRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Decide what to delete, as of `now_millis`.
    ///
    /// Two rails are unconditional and cannot be configured away:
    ///
    /// 1. **A WORM-locked partition is never deleted.** Compliance holds exist to
    ///    survive misconfigured policies; a policy that could override one would
    ///    be worthless as a legal control.
    /// 2. **The last partition is never deleted.** A `MaxTotalBytes(0)` typo, or a
    ///    clock jump making everything look ancient, should not empty the table.
    ///
    /// Both are reported in `retained` rather than silently skipped, so an
    /// operator can see the policy is being clamped instead of wondering why disk
    /// use is not falling.
    pub fn plan(&self, partitions: &[PartitionMeta], now_millis: u64) -> RetentionPlan {
        let mut plan = RetentionPlan::default();
        if partitions.is_empty() {
            return plan;
        }

        // Oldest first — every rule deletes from the old end.
        let mut ordered: Vec<&PartitionMeta> = partitions.iter().collect();
        ordered.sort_by_key(|p| (p.created_at_millis, p.file_id, p.partition_id));

        // Selected id -> the rule that selected it (first rule wins, for reporting).
        let mut selected: Vec<((u16, u16), RetentionRule)> = Vec::new();
        let mark = |p: &PartitionMeta, rule: RetentionRule, sel: &mut Vec<_>| {
            let key = (p.file_id, p.partition_id);
            if !sel
                .iter()
                .any(|(k, _): &((u16, u16), RetentionRule)| *k == key)
            {
                sel.push((key, rule));
            }
        };

        for rule in &self.rules {
            match *rule {
                RetentionRule::MaxAgeMillis(max_age) => {
                    for p in &ordered {
                        // Saturating: a partition with a future timestamp (clock
                        // skew across a cluster) has age 0 and is never expired by
                        // age. Wrapping here would make it maximally old and delete
                        // the newest data first.
                        let age = now_millis.saturating_sub(p.created_at_millis);
                        if age > max_age {
                            mark(p, *rule, &mut selected);
                        }
                    }
                }
                RetentionRule::MaxTotalBytes(budget) => {
                    let total: u64 = ordered.iter().map(|p| p.size_bytes).sum();
                    let mut over = total.saturating_sub(budget);
                    for p in &ordered {
                        if over == 0 {
                            break;
                        }
                        mark(p, *rule, &mut selected);
                        over = over.saturating_sub(p.size_bytes);
                    }
                }
                RetentionRule::MaxPartitions(keep) => {
                    if ordered.len() > keep {
                        for p in &ordered[..ordered.len() - keep] {
                            mark(p, *rule, &mut selected);
                        }
                    }
                }
            }
        }

        // Apply the rails. `survivors` counts what would remain, so the
        // last-partition rail is evaluated against the real outcome rather than
        // against the original count.
        let mut survivors = ordered.len();
        for p in &ordered {
            let key = (p.file_id, p.partition_id);
            let Some((_, rule)) = selected.iter().find(|(k, _)| *k == key) else {
                continue;
            };
            if p.worm_locked {
                plan.retained
                    .push(((*p).clone(), RetentionVeto::WormLocked));
                continue;
            }
            if survivors <= 1 {
                plan.retained
                    .push(((*p).clone(), RetentionVeto::LastPartition));
                continue;
            }
            survivors -= 1;
            plan.delete.push(RetentionAction {
                partition: (*p).clone(),
                rule: *rule,
            });
        }
        plan
    }
}

// ── Verification (VV-STO-016) ─────────────────────────────────────────────────

/// One problem found by [`verify_partitions`].
#[derive(Debug, Clone, PartialEq)]
pub enum VerifyFinding {
    /// The file would not open, or failed its internal CRCs.
    Unreadable { path: PathBuf, error: String },
    /// Two partitions claim the same `(file_id, partition_id)`. Row descriptors
    /// address rows by that pair, so a duplicate makes lookups ambiguous.
    DuplicateId {
        file_id: u16,
        partition_id: u16,
        paths: Vec<PathBuf>,
    },
    /// A partition's on-disk row count disagrees with its recorded metadata.
    RowCountMismatch {
        path: PathBuf,
        recorded: u32,
        actual: u32,
    },
    /// Two partitions at the same level (> 0) have overlapping key ranges. Legal
    /// at L0, a broken invariant anywhere below it.
    LevelOverlap {
        level: u8,
        a: (u16, u16),
        b: (u16, u16),
    },
}

/// Result of a verification sweep.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VerifyReport {
    pub checked: usize,
    pub findings: Vec<VerifyFinding>,
}

impl VerifyReport {
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Verify a set of partitions, individually and against each other.
///
/// Per-file integrity is already enforced by [`PartitionReader::open`], which
/// checks the header and whole-file CRCs. What this adds is the **cross-partition**
/// half (VV-STO-016): invariants that no single file can violate on its own and
/// that therefore nothing else checks —
///
/// - duplicate `(file_id, partition_id)` pairs, which make a `RowDescriptor`
///   ambiguous about which file it points into;
/// - a recorded row count that disagrees with the file;
/// - key-range overlap within a level below L0, which silently breaks the leveled
///   compaction invariant that a lookup probes each level at most once.
///
/// Reports every finding rather than stopping at the first: an operator running
/// this after a crash wants the full damage list, not one symptom at a time.
pub fn verify_partitions(partitions: &[PartitionMeta]) -> StorageResult<VerifyReport> {
    let mut report = VerifyReport::default();

    // Per-file: open (which verifies both CRCs) and cross-check the row count.
    for p in partitions {
        report.checked += 1;
        match PartitionReader::open(Path::new(&p.path)) {
            Err(e) => report.findings.push(VerifyFinding::Unreadable {
                path: p.path.clone(),
                error: e.to_string(),
            }),
            Ok(r) => {
                if r.row_count() != p.row_count {
                    report.findings.push(VerifyFinding::RowCountMismatch {
                        path: p.path.clone(),
                        recorded: p.row_count,
                        actual: r.row_count(),
                    });
                }
            }
        }
    }

    // Duplicate ids.
    let mut by_id: HashMap<(u16, u16), Vec<PathBuf>> = HashMap::new();
    for p in partitions {
        by_id
            .entry((p.file_id, p.partition_id))
            .or_default()
            .push(p.path.clone());
    }
    let mut dupes: Vec<((u16, u16), Vec<PathBuf>)> = by_id
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect();
    // Sorted so the report is deterministic; HashMap order is not.
    dupes.sort_by_key(|(id, _)| *id);
    for ((file_id, partition_id), paths) in dupes {
        report.findings.push(VerifyFinding::DuplicateId {
            file_id,
            partition_id,
            paths,
        });
    }

    // Key-range overlap within a level. L0 is exempt: overlapping L0 partitions
    // are the normal result of flushing, and merging them away is precisely what
    // L0 compaction is for.
    let mut by_level: HashMap<u8, Vec<&PartitionMeta>> = HashMap::new();
    for p in partitions.iter().filter(|p| p.level > 0) {
        by_level.entry(p.level).or_default().push(p);
    }
    let mut levels: Vec<u8> = by_level.keys().copied().collect();
    levels.sort_unstable();
    for level in levels {
        let mut members = by_level[&level].clone();
        members.sort_by_key(|p| (p.min_key, p.max_key, p.file_id, p.partition_id));
        for pair in members.windows(2) {
            if pair[0].overlaps(pair[1]) {
                report.findings.push(VerifyFinding::LevelOverlap {
                    level,
                    a: (pair[0].file_id, pair[0].partition_id),
                    b: (pair[1].file_id, pair[1].partition_id),
                });
            }
        }
    }

    Ok(report)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(id: u16, level: u8, size: u64, created: u64, keys: (u64, u64)) -> PartitionMeta {
        PartitionMeta {
            file_id: 1,
            partition_id: id,
            path: PathBuf::from(format!("p{id}.vvsst")),
            level,
            size_bytes: size,
            row_count: 1000,
            created_at_millis: created,
            min_key: keys.0,
            max_key: keys.1,
            tombstones: 0,
            worm_locked: false,
        }
    }

    // ── compaction planning ───────────────────────────────────────────────────

    #[test]
    fn nothing_to_do_yields_no_jobs() {
        let s = CompactionStrategy::default();
        assert!(s.plan(&[]).is_empty(), "empty input");
        assert!(
            s.plan(&[meta(0, 0, 1000, 0, (0, 100))]).is_empty(),
            "a single L0 partition is below the trigger"
        );
    }

    #[test]
    fn leveled_merges_all_of_l0_at_the_trigger() {
        let s = CompactionStrategy::Leveled {
            l0_trigger: 3,
            level_multiplier: 10,
            l1_bytes: 1 << 30,
            garbage_threshold: 0.9,
        };
        let ps: Vec<PartitionMeta> = (0..3)
            .map(|i| meta(i, 0, 100, i as u64, (0, 100)))
            .collect();
        let jobs = s.plan(&ps);
        assert_eq!(jobs.len(), 1);
        // ALL of L0, not a subset: L0 partitions overlap each other, so leaving
        // some behind would leave them overlapping the new L1 output too.
        assert_eq!(jobs[0].inputs.len(), 3);
        assert_eq!(jobs[0].output_level, 1);
        assert_eq!(jobs[0].reason, CompactionReason::LevelOverflow);

        // One below the trigger does nothing.
        assert!(s.plan(&ps[..2]).is_empty());
    }

    #[test]
    fn leveled_merges_a_victim_with_its_overlapping_successors() {
        let s = CompactionStrategy::Leveled {
            l0_trigger: 100,
            level_multiplier: 10,
            l1_bytes: 1000,
            garbage_threshold: 0.9,
        };
        let ps = vec![
            // L1 is over its 1000-byte budget.
            meta(1, 1, 800, 10, (0, 50)),
            meta(2, 1, 800, 20, (60, 90)),
            // L2 candidates: one overlaps the victim, one does not.
            meta(3, 2, 100, 5, (40, 70)),
            meta(4, 2, 100, 6, (500, 600)),
        ];
        let jobs = s.plan(&ps);
        assert_eq!(jobs.len(), 1, "expected one L1 overflow job, got {jobs:?}");
        let ids: Vec<u16> = jobs[0].inputs.iter().map(|p| p.partition_id).collect();
        // Victim is the OLDEST L1 partition (most likely to hold superseded rows),
        // merged with exactly the L2 partitions its key range touches.
        assert!(ids.contains(&1), "oldest L1 partition should be the victim");
        assert!(
            ids.contains(&3),
            "overlapping L2 partition must be included"
        );
        assert!(
            !ids.contains(&4),
            "non-overlapping L2 partition must NOT be rewritten"
        );
        assert_eq!(jobs[0].output_level, 2);
    }

    #[test]
    fn leveled_respects_the_level_budget_curve() {
        let s = CompactionStrategy::Leveled {
            l0_trigger: 100,
            level_multiplier: 10,
            l1_bytes: 1000,
            garbage_threshold: 0.9,
        };
        assert_eq!(s.level_budget(0), None, "L0 is bounded by count, not bytes");
        assert_eq!(s.level_budget(1), Some(1_000));
        assert_eq!(s.level_budget(2), Some(10_000));
        assert_eq!(s.level_budget(3), Some(100_000));
        // A deep level must not overflow into a wrapped, tiny budget.
        assert!(s.level_budget(30).unwrap() >= s.level_budget(10).unwrap());
    }

    #[test]
    fn size_tiered_groups_similar_sizes_only() {
        let s = CompactionStrategy::SizeTiered {
            min_run: 3,
            max_run: 16,
            size_ratio: 2.0,
            garbage_threshold: 0.9,
        };
        let ps = vec![
            meta(1, 0, 100, 1, (0, 10)),
            meta(2, 0, 150, 2, (0, 10)),
            meta(3, 0, 190, 3, (0, 10)),
            // 10x larger — a different bucket, and alone, so no job.
            meta(4, 0, 2000, 4, (0, 10)),
        ];
        let jobs = s.plan(&ps);
        assert_eq!(jobs.len(), 1, "got {jobs:?}");
        let ids: Vec<u16> = jobs[0].inputs.iter().map(|p| p.partition_id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
        assert_eq!(jobs[0].reason, CompactionReason::SizeRun);
        assert!(
            !ids.contains(&4),
            "a 10x larger partition must not join a small run"
        );
    }

    #[test]
    fn size_tiered_caps_a_job_at_max_run() {
        let s = CompactionStrategy::SizeTiered {
            min_run: 2,
            max_run: 4,
            size_ratio: 2.0,
            garbage_threshold: 0.9,
        };
        let ps: Vec<PartitionMeta> = (0..10)
            .map(|i| meta(i, 0, 100, i as u64, (0, 10)))
            .collect();
        let jobs = s.plan(&ps);
        // 10 partitions, 4 per job: two full jobs plus a 2-partition remainder.
        assert_eq!(jobs.len(), 3, "got {jobs:?}");
        for j in &jobs {
            assert!(
                j.inputs.len() <= 4,
                "a job exceeded max_run: {} inputs",
                j.inputs.len()
            );
        }
        assert_eq!(jobs.iter().map(|j| j.inputs.len()).sum::<usize>(), 10);
    }

    #[test]
    fn garbage_heavy_partitions_are_compacted_alone_and_first() {
        let s = CompactionStrategy::Leveled {
            l0_trigger: 100,
            level_multiplier: 10,
            l1_bytes: 1 << 30,
            garbage_threshold: 0.30,
        };
        let mut dirty = meta(7, 2, 500, 1, (0, 10));
        dirty.tombstones = 900; // 90% dead
        let clean = meta(8, 2, 500, 2, (20, 30));
        let jobs = s.plan(&[clean.clone(), dirty.clone()]);

        assert_eq!(jobs.len(), 1, "got {jobs:?}");
        assert_eq!(jobs[0].reason, CompactionReason::GarbageCollection);
        assert_eq!(jobs[0].inputs.len(), 1);
        assert_eq!(jobs[0].inputs[0].partition_id, 7);
        // Rewritten in place: the point is to drop dead rows, not to move levels.
        assert_eq!(jobs[0].output_level, 2);
    }

    /// **A partition must never appear in two jobs of the same round.**
    ///
    /// Compaction rewrites and then removes its inputs, so scheduling one twice
    /// means either a double delete or two writers racing on the same output. The
    /// garbage pass and the level pass are separate loops over the same slice,
    /// which is exactly the shape that produces this bug.
    #[test]
    fn no_partition_is_scheduled_twice() {
        for strategy in [
            CompactionStrategy::default(),
            CompactionStrategy::size_tiered(),
            CompactionStrategy::Leveled {
                l0_trigger: 2,
                level_multiplier: 2,
                l1_bytes: 100,
                garbage_threshold: 0.2,
            },
            CompactionStrategy::SizeTiered {
                min_run: 2,
                max_run: 3,
                size_ratio: 4.0,
                garbage_threshold: 0.2,
            },
        ] {
            // A mix that triggers every pass at once: dirty partitions, a crowded
            // L0, and levels over budget.
            let mut ps = Vec::new();
            for i in 0..12u16 {
                let mut p = meta(
                    i,
                    (i % 4) as u8,
                    100 * (i as u64 + 1),
                    i as u64,
                    (
                        i as u64 * 10,
                        i as u64 * 10 + 15, // deliberately overlapping
                    ),
                );
                if i % 3 == 0 {
                    p.tombstones = 800;
                }
                ps.push(p);
            }
            let jobs = strategy.plan(&ps);
            let mut seen: Vec<(u16, u16)> = Vec::new();
            for j in &jobs {
                for input in &j.inputs {
                    let key = (input.file_id, input.partition_id);
                    assert!(
                        !seen.contains(&key),
                        "{strategy:?}: partition {key:?} scheduled twice"
                    );
                    seen.push(key);
                }
            }
            for j in &jobs {
                assert!(!j.inputs.is_empty(), "{strategy:?}: empty job");
            }
        }
    }

    // ── retention ─────────────────────────────────────────────────────────────

    #[test]
    fn max_age_deletes_only_what_is_older() {
        let policy = RetentionPolicy::new().with_rule(RetentionRule::MaxAgeMillis(1000));
        let ps = vec![
            meta(1, 0, 10, 0, (0, 1)),    // age 5000 — expired
            meta(2, 0, 10, 4500, (2, 3)), // age 500  — kept
            meta(3, 0, 10, 5000, (4, 5)), // age 0    — kept
        ];
        let plan = policy.plan(&ps, 5000);
        let ids: Vec<u16> = plan
            .delete
            .iter()
            .map(|a| a.partition.partition_id)
            .collect();
        assert_eq!(ids, vec![1]);
        assert!(plan.retained.is_empty());
    }

    #[test]
    fn max_age_boundary_is_strictly_greater() {
        let policy = RetentionPolicy::new().with_rule(RetentionRule::MaxAgeMillis(1000));
        // Exactly at the limit is retained; one past it is not.
        let at = vec![meta(1, 0, 10, 0, (0, 1)), meta(2, 0, 10, 500, (2, 3))];
        assert!(
            policy.plan(&at, 1000).delete.is_empty(),
            "age exactly == max_age must be retained"
        );
        assert_eq!(policy.plan(&at, 1001).delete.len(), 1);
    }

    /// A future-dated partition must not be treated as infinitely old.
    ///
    /// Clock skew across a cluster makes this real. An unsaturated
    /// `now - created` underflows to a huge age, which would delete the NEWEST
    /// data first — the worst possible inversion.
    #[test]
    fn future_timestamps_do_not_underflow_into_deletion() {
        let policy = RetentionPolicy::new().with_rule(RetentionRule::MaxAgeMillis(1000));
        let ps = vec![
            meta(1, 0, 10, 0, (0, 1)),
            meta(2, 0, 10, u64::MAX, (2, 3)), // far future
        ];
        let plan = policy.plan(&ps, 5000);
        let ids: Vec<u16> = plan
            .delete
            .iter()
            .map(|a| a.partition.partition_id)
            .collect();
        assert_eq!(ids, vec![1], "a future-dated partition must not be deleted");
    }

    #[test]
    fn max_total_bytes_deletes_oldest_until_it_fits() {
        let policy = RetentionPolicy::new().with_rule(RetentionRule::MaxTotalBytes(250));
        let ps = vec![
            meta(1, 0, 100, 1, (0, 1)),
            meta(2, 0, 100, 2, (2, 3)),
            meta(3, 0, 100, 3, (4, 5)),
            meta(4, 0, 100, 4, (6, 7)),
        ];
        let plan = policy.plan(&ps, 100);
        let ids: Vec<u16> = plan
            .delete
            .iter()
            .map(|a| a.partition.partition_id)
            .collect();
        // 400 total, budget 250, over by 150 → drop the two oldest (200).
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn max_partitions_keeps_the_newest_n() {
        let policy = RetentionPolicy::new().with_rule(RetentionRule::MaxPartitions(2));
        let ps: Vec<PartitionMeta> = (1..=5).map(|i| meta(i, 0, 10, i as u64, (0, 1))).collect();
        let plan = policy.plan(&ps, 100);
        let ids: Vec<u16> = plan
            .delete
            .iter()
            .map(|a| a.partition.partition_id)
            .collect();
        assert_eq!(ids, vec![1, 2, 3], "keep the 2 newest");
    }

    /// A WORM lock overrides every rule.
    ///
    /// Compliance holds exist to survive misconfigured retention. A policy that
    /// could override one would be worthless as a legal control, so this is a
    /// hard rail and not a configurable default.
    #[test]
    fn worm_lock_blocks_deletion_and_is_reported() {
        let policy = RetentionPolicy::new()
            .with_rule(RetentionRule::MaxAgeMillis(0))
            .with_rule(RetentionRule::MaxPartitions(1))
            .with_rule(RetentionRule::MaxTotalBytes(0));
        let mut locked = meta(1, 0, 100, 0, (0, 1));
        locked.worm_locked = true;
        let ps = vec![
            locked,
            meta(2, 0, 100, 1, (2, 3)),
            meta(3, 0, 100, 2, (4, 5)),
        ];

        let plan = policy.plan(&ps, 10_000);
        assert!(
            !plan.delete.iter().any(|a| a.partition.partition_id == 1),
            "a WORM-locked partition was scheduled for deletion"
        );
        assert!(
            plan.retained
                .iter()
                .any(|(p, v)| p.partition_id == 1 && *v == RetentionVeto::WormLocked),
            "the veto must be reported, not silently applied"
        );
    }

    /// Retention must never empty the table.
    ///
    /// `MaxTotalBytes(0)`, or a clock jump making everything look ancient, would
    /// otherwise delete every partition. The rail is reported so an operator sees
    /// the policy being clamped rather than wondering why disk use stopped falling.
    #[test]
    fn retention_never_deletes_the_last_partition() {
        for policy in [
            RetentionPolicy::new().with_rule(RetentionRule::MaxAgeMillis(0)),
            RetentionPolicy::new().with_rule(RetentionRule::MaxTotalBytes(0)),
            RetentionPolicy::new().with_rule(RetentionRule::MaxPartitions(0)),
        ] {
            let ps: Vec<PartitionMeta> =
                (1..=4).map(|i| meta(i, 0, 100, i as u64, (0, 1))).collect();
            let plan = policy.plan(&ps, u64::MAX / 2);
            assert_eq!(
                plan.delete.len(),
                3,
                "{:?}: expected 3 deletions leaving 1, got {}",
                policy.rules,
                plan.delete.len()
            );
            assert!(
                plan.retained
                    .iter()
                    .any(|(_, v)| *v == RetentionVeto::LastPartition),
                "{:?}: the last-partition rail must be reported",
                policy.rules
            );
            // And what survives is the NEWEST, not an arbitrary one.
            let deleted: Vec<u16> = plan
                .delete
                .iter()
                .map(|a| a.partition.partition_id)
                .collect();
            assert!(!deleted.contains(&4), "the newest partition must survive");
        }
    }

    #[test]
    fn no_rules_deletes_nothing() {
        let ps: Vec<PartitionMeta> = (1..=5).map(|i| meta(i, 0, 100, i as u64, (0, 1))).collect();
        let plan = RetentionPolicy::new().plan(&ps, u64::MAX);
        assert!(plan.delete.is_empty());
        assert!(plan.retained.is_empty());
        assert!(RetentionPolicy::new().plan(&[], 0).delete.is_empty());
    }

    #[test]
    fn overlapping_rules_do_not_double_select() {
        // Both rules select partition 1; it must appear exactly once.
        let policy = RetentionPolicy::new()
            .with_rule(RetentionRule::MaxAgeMillis(10))
            .with_rule(RetentionRule::MaxPartitions(2));
        let ps: Vec<PartitionMeta> = (1..=3).map(|i| meta(i, 0, 100, i as u64, (0, 1))).collect();
        let plan = policy.plan(&ps, 1000);
        let ids: Vec<u16> = plan
            .delete
            .iter()
            .map(|a| a.partition.partition_id)
            .collect();
        let mut uniq = ids.clone();
        uniq.sort_unstable();
        uniq.dedup();
        assert_eq!(
            ids.len(),
            uniq.len(),
            "a partition was selected twice: {ids:?}"
        );
    }

    // ── verification ──────────────────────────────────────────────────────────

    #[test]
    fn duplicate_partition_ids_are_reported() {
        // Paths do not exist, so every partition also reports Unreadable; this
        // test is about the cross-partition finding, which does not need files.
        let mut a = meta(1, 1, 10, 0, (0, 10));
        let mut b = meta(1, 1, 10, 0, (20, 30));
        a.path = PathBuf::from("a.vvsst");
        b.path = PathBuf::from("b.vvsst");
        let report = verify_partitions(&[a, b]).unwrap();
        assert_eq!(report.checked, 2);
        assert!(
            report.findings.iter().any(|f| matches!(
                f,
                VerifyFinding::DuplicateId {
                    file_id: 1,
                    partition_id: 1,
                    ..
                }
            )),
            "duplicate id not reported: {:?}",
            report.findings
        );
    }

    #[test]
    fn level_overlap_is_reported_below_l0_but_allowed_at_l0() {
        // L0 overlap is normal — it is what flushing produces.
        let l0 = vec![meta(1, 0, 10, 0, (0, 100)), meta(2, 0, 10, 0, (50, 150))];
        let r = verify_partitions(&l0).unwrap();
        assert!(
            !r.findings
                .iter()
                .any(|f| matches!(f, VerifyFinding::LevelOverlap { .. })),
            "L0 overlap must be allowed"
        );

        // The same overlap at L2 breaks the one-probe-per-level invariant.
        let l2 = vec![meta(1, 2, 10, 0, (0, 100)), meta(2, 2, 10, 0, (50, 150))];
        let r = verify_partitions(&l2).unwrap();
        assert!(
            r.findings
                .iter()
                .any(|f| matches!(f, VerifyFinding::LevelOverlap { level: 2, .. })),
            "L2 overlap not reported: {:?}",
            r.findings
        );

        // Adjacent-but-disjoint ranges are not an overlap.
        let ok = vec![meta(1, 2, 10, 0, (0, 100)), meta(2, 2, 10, 0, (101, 200))];
        let r = verify_partitions(&ok).unwrap();
        assert!(
            !r.findings
                .iter()
                .any(|f| matches!(f, VerifyFinding::LevelOverlap { .. })),
            "disjoint ranges reported as overlapping"
        );
    }

    #[test]
    fn unreadable_files_are_reported_not_fatal() {
        // The sweep must survive a missing file and keep checking the rest —
        // an operator running this after a crash wants the whole damage list.
        let ps = vec![
            meta(1, 0, 10, 0, (0, 10)),
            meta(2, 0, 10, 0, (20, 30)),
            meta(3, 0, 10, 0, (40, 50)),
        ];
        let report = verify_partitions(&ps).unwrap();
        assert_eq!(report.checked, 3, "the sweep stopped early");
        assert_eq!(
            report
                .findings
                .iter()
                .filter(|f| matches!(f, VerifyFinding::Unreadable { .. }))
                .count(),
            3
        );
        assert!(!report.is_clean());
    }

    #[test]
    fn verify_findings_are_deterministic() {
        // Duplicate detection walks a HashMap; the report must not vary between
        // runs or a diff-based monitor would alarm on noise.
        let ps = vec![
            meta(1, 1, 10, 0, (0, 10)),
            meta(1, 1, 10, 0, (0, 10)),
            meta(2, 1, 10, 0, (5, 15)),
            meta(2, 1, 10, 0, (5, 15)),
        ];
        let first = verify_partitions(&ps).unwrap();
        for _ in 0..8 {
            assert_eq!(verify_partitions(&ps).unwrap(), first, "report varied");
        }
    }
}
