//! Allocation-count guards for the read paths (VV-TST-008).
//!
//! ## Why this exists instead of only a timing gate
//!
//! The bug these guard against shipped and survived two commits: `ok_or` with an
//! allocating error argument, evaluated eagerly, so decoding N rows performed N
//! heap allocations for an error that never fires. It made frame decode look 15x
//! slower than it is and the wrong figure was quoted as a real cost.
//!
//! A timing gate on CI would probably not have caught it. Shared runners have
//! noisy neighbours, and a threshold loose enough not to flake is loose enough to
//! miss a 2x regression on one code path among many.
//!
//! **Allocation counts are deterministic.** They do not vary with load, core
//! count, or thermal state. And the property that actually matters here is not
//! "fast" — it is *algorithmic*: decoding a column must allocate a number of times
//! that does not depend on how many rows it decodes. That is exactly what was
//! violated, and it can be asserted exactly.
//!
//! ## The shape of the assertion
//!
//! Every test measures at two row counts and compares the DIFFERENCE, rather than
//! asserting an absolute number. An absolute count is brittle — it changes when
//! `Vec` growth strategy changes, or when an unrelated helper starts using a
//! `String` — and a brittle test gets its threshold raised until it means nothing.
//!
//! A difference near zero means O(1). A difference near the row delta means O(N),
//! which is the bug. There is no plausible middle, so the threshold can be tight
//! without being fragile.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use tempfile::TempDir;
use vectorvolt_storage::{
    ColumnValues, MicroPartition, PartitionReader, WriteOptions, write_partition_with,
};

// ── counting allocator ────────────────────────────────────────────────────────

// PER-THREAD, not global. `cargo test` runs tests in parallel, so a shared
// counter measures every other test's allocations too. All four guards below
// failed on first run for exactly that reason — the measurement was picking up
// concurrent work and reporting it as per-row allocation in the code under test.
//
// A thread-local `Cell` is also cheaper than an atomic on the allocator hot path,
// which matters because this allocator wraps every allocation in the binary.
thread_local! {
    static ALLOCS: Cell<usize> = const { Cell::new(0) };
}

fn bump() {
    // `try_with`: during thread-local destruction the counter may already be gone,
    // and a panic inside the allocator would abort the process. A dropped count at
    // teardown is harmless — nothing is being measured then.
    let _ = ALLOCS.try_with(|c| c.set(c.get() + 1));
}

fn count() -> usize {
    ALLOCS.try_with(|c| c.get()).unwrap_or(0)
}

struct Counting;

// SAFETY: every method forwards to `System`, which is a valid allocator. The
// counter is bumped before delegating and never affects the returned pointer, so
// the allocator contract is unchanged.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        bump();
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // Counted: a `Vec` that grows repeatedly instead of reserving once is the
        // same class of problem as allocating per row.
        bump();
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Allocations performed by `f` **on this thread**.
fn allocations<T>(f: impl FnOnce() -> T) -> usize {
    let before = count();
    let out = f();
    let after = count();
    // Keep the result alive until after the second read, so its allocation is
    // counted and its drop is not.
    std::hint::black_box(&out);
    after - before
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn shapes(n: usize) -> Vec<(&'static str, Vec<u64>)> {
    let distinct = [0u64, u64::MAX / 4, u64::MAX / 2, u64::MAX];
    vec![
        ("sequential", (0..n as u64).collect()),
        (
            "narrow",
            (0..n as u64).map(|i| 10_000_000_000 + i % 1000).collect(),
        ),
        (
            "runs",
            (0..n as u64).map(|i| (i / 256) * 9_999_991).collect(),
        ),
        (
            "wide_low_card",
            (0..n).map(|i| distinct[(i * 7) % 4]).collect(),
        ),
        (
            "incompressible",
            (0..n as u64)
                .map(|i| {
                    let mut x = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
                    x ^= x >> 29;
                    x ^ (x >> 32)
                })
                .collect(),
        ),
        ("all_identical", vec![7u64; n]),
    ]
}

fn open(dir: &TempDir, tag: &str, values: &[u64], lz4: bool) -> PartitionReader {
    let mut p = MicroPartition::new(0, 1);
    p.push_column("v", ColumnValues::auto_encode(values.to_vec()))
        .unwrap();
    let path = dir.path().join(format!("{tag}.vvsst"));
    let mut f = std::fs::File::create(&path).unwrap();
    let opts = if lz4 {
        WriteOptions::new().with_lz4()
    } else {
        WriteOptions::default()
    };
    write_partition_with(&mut f, &p, opts).unwrap();
    drop(f);
    PartitionReader::open(&path).unwrap()
}

/// Rows used for the small and large measurements. The gap is large enough that
/// a per-row allocation is unmissable and small enough to stay fast.
const SMALL: usize = 1_000;
const LARGE: usize = 64_000;

/// Slack for allocations that legitimately differ between the two sizes —
/// a `Vec` reaching one more growth step, for instance.
const SLACK: usize = 16;

// ── tests ─────────────────────────────────────────────────────────────────────

/// **Decoding a column allocates O(1) times, not O(rows).**
///
/// The direct regression guard for the `ok_or` bug. With a pre-reserved buffer,
/// decoding 64x as many rows must not allocate meaningfully more times.
#[test]
fn column_decode_does_not_allocate_per_row() {
    let dir = TempDir::new().unwrap();

    for (name, _) in shapes(SMALL) {
        let small_vals = shapes(SMALL)
            .into_iter()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1;
        let large_vals = shapes(LARGE)
            .into_iter()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1;

        let rs = open(&dir, &format!("{name}_s"), &small_vals, false);
        let rl = open(&dir, &format!("{name}_l"), &large_vals, false);

        let mut bs = Vec::with_capacity(SMALL);
        let mut bl = Vec::with_capacity(LARGE);
        // Warm: first call may populate lazily-initialised state.
        rs.decode_column_into(0, &mut bs).unwrap();
        rl.decode_column_into(0, &mut bl).unwrap();

        let a_small = allocations(|| rs.decode_column_into(0, &mut bs).unwrap());
        let a_large = allocations(|| rl.decode_column_into(0, &mut bl).unwrap());

        let extra = a_large.saturating_sub(a_small);
        assert!(
            extra <= SLACK,
            "{name}: decoding {LARGE} rows allocated {a_large} times vs {a_small} for \
             {SMALL} rows ({extra} extra). That scales with row count — something in \
             the decode loop is allocating per row.",
        );
    }
}

/// The same property for point reads: reading a value allocates nothing.
///
/// `read_value` is the O(1) path the whole read architecture rests on. An
/// allocation there is worse than in a scan — it is paid per lookup, and lookups
/// are the operation the project optimises for.
#[test]
fn point_reads_do_not_allocate() {
    let dir = TempDir::new().unwrap();
    for (name, values) in shapes(LARGE) {
        let r = open(&dir, &format!("pt_{name}"), &values, false);
        let _ = r.read_value(0, 0).unwrap();

        // Strided, so every branch of the decoder is exercised.
        let a = allocations(|| {
            let mut acc = 0u64;
            for i in (0..LARGE).step_by(97) {
                acc = acc.wrapping_add(r.read_value(0, i).unwrap());
            }
            acc
        });
        assert_eq!(
            a,
            0,
            "{name}: {} point reads allocated {a} times; read_value must be \
             allocation-free",
            LARGE / 97
        );
    }
}

/// A caller-supplied buffer is genuinely reused across partitions.
///
/// The whole reason `decode_column_into` exists rather than returning a `Vec`. If
/// the buffer were reallocated each time, a scan over N partitions would pay N
/// allocations and the API would be pointless.
#[test]
fn a_reused_buffer_stops_allocating_after_the_first_partition() {
    let dir = TempDir::new().unwrap();
    let values: Vec<u64> = (0..LARGE as u64).collect();
    let readers: Vec<PartitionReader> = (0..5)
        .map(|i| open(&dir, &format!("reuse_{i}"), &values, false))
        .collect();

    let mut buf = Vec::new();
    // First decode is allowed to allocate — the buffer starts empty.
    readers[0].decode_column_into(0, &mut buf).unwrap();

    let a = allocations(|| {
        for r in &readers[1..] {
            r.decode_column_into(0, &mut buf).unwrap();
        }
    });
    assert!(
        a <= SLACK,
        "decoding 4 further partitions into the same buffer allocated {a} times; \
         the buffer is not being reused"
    );
}

/// A compressed column allocates a bounded number of times, not per row.
///
/// LZ4 must allocate once for the decompressed payload — that is inherent and is
/// why compression is opt-in. What it must not do is allocate per value on top.
#[test]
fn compressed_decode_allocates_for_the_payload_but_not_per_row() {
    let dir = TempDir::new().unwrap();
    // Long runs, which LZ4 genuinely compresses. `i % 500` does not: the frame
    // encoding already packs it to 9 bits, and LZ4 over that is not smaller, so
    // the writer correctly falls back to verbatim and the test would assert
    // nothing.
    let small: Vec<u64> = (0..SMALL as u64).map(|i| (i / 64) * 7_777_777).collect();
    let large: Vec<u64> = (0..LARGE as u64).map(|i| (i / 64) * 7_777_777).collect();

    let rs = open(&dir, "lz4_s", &small, true);
    let rl = open(&dir, "lz4_l", &large, true);
    assert!(rs.is_column_compressed(0), "test needs a compressed column");

    let mut bs = Vec::with_capacity(SMALL);
    let mut bl = Vec::with_capacity(LARGE);
    rs.decode_column_into(0, &mut bs).unwrap();
    rl.decode_column_into(0, &mut bl).unwrap();

    let a_small = allocations(|| rs.decode_column_into(0, &mut bs).unwrap());
    let a_large = allocations(|| rl.decode_column_into(0, &mut bl).unwrap());

    assert!(
        a_large.saturating_sub(a_small) <= SLACK,
        "compressed decode: {a_large} allocations at {LARGE} rows vs {a_small} at \
         {SMALL}; the decompression buffer should be one allocation regardless of size"
    );
}

/// `filter_offsets_into` reuses its scratch buffer too.
#[test]
fn range_filtering_reuses_its_scratch_buffer() {
    let dir = TempDir::new().unwrap();
    let values: Vec<u64> = (0..LARGE as u64).collect();
    let r = open(&dir, "filter", &values, false);

    let mut scratch = Vec::new();
    // Warm both the mapping and the scratch buffer.
    let _ = r.filter_offsets_into(0, 0, 10, &mut scratch).unwrap();

    // A narrow range, so the returned Vec is small and the scratch dominates.
    let a = allocations(|| {
        for i in 0..8u64 {
            let _ = r
                .filter_offsets_into(0, i * 10, i * 10 + 5, &mut scratch)
                .unwrap();
        }
    });
    // 8 calls, each returning a small Vec: a handful of allocations, not 8 x LARGE.
    assert!(
        a <= 8 * 4 + SLACK,
        "8 filtered scans allocated {a} times; the scratch buffer is being \
         reallocated per call"
    );
}

/// The guard itself works.
///
/// A test that measures allocations is worthless if the measurement is broken, and
/// a broken counter fails OPEN — every assertion above would pass trivially. This
/// asserts the counter actually observes a known allocation.
#[test]
fn the_allocation_counter_is_not_silently_broken() {
    let none = allocations(|| 1u64 + 1);
    assert_eq!(none, 0, "counted an allocation for arithmetic");

    let one = allocations(|| Vec::<u64>::with_capacity(1024));
    assert!(one >= 1, "failed to count a Vec allocation");

    let many = allocations(|| {
        let mut v: Vec<Vec<u64>> = Vec::with_capacity(100);
        for i in 0..100 {
            v.push(vec![i; 4]);
        }
        v
    });
    assert!(
        many >= 100,
        "counted only {many} allocations for 100 Vec pushes"
    );
}
