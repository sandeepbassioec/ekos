//! AVX2 kernels — 256-bit vectors, 4 × `u64` per instruction.
//!
//! AVX2 is VectorVolt's **baseline** SIMD tier: universally available since
//! Haswell (2013) and Excavator/Zen (2015), with no downclocking penalty. Every
//! kernel here processes 4 `u64` lanes per iteration and falls back to a scalar
//! tail loop for the remainder.
//!
//! ## The unsigned-comparison problem
//!
//! AVX2 provides only **signed** 64-bit compare (`_mm256_cmpgt_epi64`). There is
//! no `_mm256_cmpgt_epu64` — that arrived with AVX-512. Comparing `u64` values
//! naively as `i64` breaks for anything above `i64::MAX`: `u64::MAX` reads as
//! `-1` and sorts below `0`.
//!
//! The fix is a range shift. XOR-ing with `i64::MIN` flips the top bit, mapping
//! the unsigned order `[0, u64::MAX]` onto the signed order `[i64::MIN, i64::MAX]`
//! monotonically. Signed compare in that space is exactly unsigned compare in the
//! original. The XOR is one instruction on a value we already loaded, so it costs
//! essentially nothing — and it is why `count_range` over PKs near `u64::MAX`
//! gives the same answer as the scalar oracle. `tests/simd_equivalence.rs` pins
//! this behaviour explicitly.
//!
//! ## Safety contract
//!
//! Every function is `unsafe fn` with `#[target_feature(enable = "avx2")]`.
//! Callers must have verified AVX2 support at runtime — [`crate::simd`] does this
//! once at startup via `cpuid` and dispatches accordingly. Calling these directly
//! on a CPU without AVX2 executes an illegal instruction.
//!
//! Within each body, the operations needing `unsafe` are: the intrinsics
//! themselves (require the target feature, guaranteed by the caller's contract);
//! pointer arithmetic bounded by `chunks * LANES <= values.len()` by construction;
//! unaligned vector loads/stores (`loadu`/`storeu` have no alignment requirement);
//! and `get_unchecked` in tail loops whose index is `< values.len()` by the loop
//! bound.

#![cfg(target_arch = "x86_64")]

use core::arch::x86_64::*;

/// Bit pattern that maps unsigned order onto signed order under XOR.
const FLIP: i64 = i64::MIN;

/// Lanes processed per vector iteration.
const LANES: usize = 4;

/// Independent accumulators used by reduction kernels (`min_max`, `sum`).
///
/// A naive reduction loop carries a serial dependency: iteration N cannot start
/// its `blendv`/`add` until iteration N−1 produced the accumulator. At ~1–3
/// cycles of latency per step that caps throughput far below the CPU's issue
/// width, and it is why a single-accumulator AVX2 `min_max` measured *slower*
/// than the scalar loop (which LLVM auto-vectorizes with several accumulators of
/// its own).
///
/// Four independent accumulator chains let the out-of-order engine keep four
/// operations in flight, then reduce them once at the end. 4 × [`LANES`] = 16
/// `u64` per iteration.
const UNROLL: usize = 4;

/// Elements consumed per unrolled block.
const BLOCK: usize = LANES * UNROLL;

/// AVX2 [`crate::simd::scalar::filter_range`].
///
/// # Safety
/// Requires runtime AVX2 support. See the module-level safety contract.
#[target_feature(enable = "avx2")]
pub unsafe fn filter_range(values: &[u64], lo: u64, hi: u64, out: &mut Vec<u32>) {
    // SAFETY: see module-level safety contract. Indices are bounded by
    // `chunks * LANES <= values.len()`; loads are unaligned-safe.
    unsafe {
        let flip = _mm256_set1_epi64x(FLIP);
        let lo_v = _mm256_xor_si256(_mm256_set1_epi64x(lo as i64), flip);
        let hi_v = _mm256_xor_si256(_mm256_set1_epi64x(hi as i64), flip);

        let n = values.len();
        let chunks = n / LANES;
        let base_ptr = values.as_ptr();

        for c in 0..chunks {
            let base = c * LANES;
            let raw = _mm256_loadu_si256(base_ptr.add(base) as *const __m256i);
            let v = _mm256_xor_si256(raw, flip);

            // out_of_range = (v < lo) | (v > hi)
            let below = _mm256_cmpgt_epi64(lo_v, v);
            let above = _mm256_cmpgt_epi64(v, hi_v);
            let bad = _mm256_or_si256(below, above);

            // One bit per 64-bit lane, taken from each lane's sign bit.
            let bad_bits = _mm256_movemask_pd(_mm256_castsi256_pd(bad)) as u32;
            let mut good = !bad_bits & 0x0F;

            while good != 0 {
                let lane = good.trailing_zeros() as usize;
                out.push((base + lane) as u32);
                good &= good - 1; // clear lowest set bit
            }
        }

        for i in (chunks * LANES)..n {
            let v = *values.get_unchecked(i);
            if v >= lo && v <= hi {
                out.push(i as u32);
            }
        }
    }
}

/// AVX2 [`crate::simd::scalar::count_range`].
///
/// # Safety
/// Requires runtime AVX2 support. See the module-level safety contract.
#[target_feature(enable = "avx2")]
pub unsafe fn count_range(values: &[u64], lo: u64, hi: u64) -> usize {
    // SAFETY: see module-level safety contract.
    unsafe {
        let flip = _mm256_set1_epi64x(FLIP);
        let lo_v = _mm256_xor_si256(_mm256_set1_epi64x(lo as i64), flip);
        let hi_v = _mm256_xor_si256(_mm256_set1_epi64x(hi as i64), flip);

        let n = values.len();
        let chunks = n / LANES;
        let base_ptr = values.as_ptr();
        let mut count = 0usize;

        for c in 0..chunks {
            let raw = _mm256_loadu_si256(base_ptr.add(c * LANES) as *const __m256i);
            let v = _mm256_xor_si256(raw, flip);
            let below = _mm256_cmpgt_epi64(lo_v, v);
            let above = _mm256_cmpgt_epi64(v, hi_v);
            let bad = _mm256_or_si256(below, above);
            let bad_bits = _mm256_movemask_pd(_mm256_castsi256_pd(bad)) as u32;
            count += (!bad_bits & 0x0F).count_ones() as usize;
        }

        for i in (chunks * LANES)..n {
            let v = *values.get_unchecked(i);
            if v >= lo && v <= hi {
                count += 1;
            }
        }
        count
    }
}

/// AVX2 [`crate::simd::scalar::min_max`].
///
/// # Safety
/// Requires runtime AVX2 support. See the module-level safety contract.
#[target_feature(enable = "avx2")]
pub unsafe fn min_max(values: &[u64]) -> Option<(u64, u64)> {
    let n = values.len();
    // Below one unrolled block the setup and horizontal reduction cost more than
    // the scalar loop saves.
    if n < BLOCK {
        return super::scalar::min_max(values);
    }

    // SAFETY: see module-level safety contract. `n >= BLOCK` guarantees the
    // initial load is in bounds; the block loop is bounded by `i + BLOCK <= n`.
    unsafe {
        let flip = _mm256_set1_epi64x(FLIP);
        let base_ptr = values.as_ptr();

        // Seed every accumulator chain with the first vector, so all four are
        // valid without needing a sentinel value.
        let first = _mm256_xor_si256(_mm256_loadu_si256(base_ptr as *const __m256i), flip);
        let mut lo_acc = [first; UNROLL];
        let mut hi_acc = [first; UNROLL];

        let mut i = 0usize;
        while i + BLOCK <= n {
            // Four independent chains — no chain reads another's result, so the
            // out-of-order engine runs all four concurrently.
            for u in 0..UNROLL {
                let v = _mm256_xor_si256(
                    _mm256_loadu_si256(base_ptr.add(i + u * LANES) as *const __m256i),
                    flip,
                );
                // AVX2 has no 64-bit min/max; select via compare mask + blend.
                // `blendv` picks from `v` wherever the mask byte's high bit is
                // set, and `cmpgt` sets all 64 bits of a winning lane, so whole
                // lanes move together.
                let gt = _mm256_cmpgt_epi64(v, hi_acc[u]);
                hi_acc[u] = _mm256_blendv_epi8(hi_acc[u], v, gt);
                let lt = _mm256_cmpgt_epi64(lo_acc[u], v);
                lo_acc[u] = _mm256_blendv_epi8(lo_acc[u], v, lt);
            }
            i += BLOCK;
        }

        // Collapse the four chains into one.
        let mut lo_v = lo_acc[0];
        let mut hi_v = hi_acc[0];
        for u in 1..UNROLL {
            let lt = _mm256_cmpgt_epi64(lo_v, lo_acc[u]);
            lo_v = _mm256_blendv_epi8(lo_v, lo_acc[u], lt);
            let gt = _mm256_cmpgt_epi64(hi_acc[u], hi_v);
            hi_v = _mm256_blendv_epi8(hi_v, hi_acc[u], gt);
        }

        let mut lo_lanes = [0i64; LANES];
        let mut hi_lanes = [0i64; LANES];
        _mm256_storeu_si256(lo_lanes.as_mut_ptr() as *mut __m256i, lo_v);
        _mm256_storeu_si256(hi_lanes.as_mut_ptr() as *mut __m256i, hi_v);

        // Undo the range shift, then reduce the 4 lanes horizontally.
        let mut lo = (lo_lanes[0] ^ FLIP) as u64;
        let mut hi = (hi_lanes[0] ^ FLIP) as u64;
        for k in 1..LANES {
            let a = (lo_lanes[k] ^ FLIP) as u64;
            if a < lo {
                lo = a;
            }
            let b = (hi_lanes[k] ^ FLIP) as u64;
            if b > hi {
                hi = b;
            }
        }

        // Remainder: whatever did not fill a whole block.
        for k in i..n {
            let v = *values.get_unchecked(k);
            if v < lo {
                lo = v;
            }
            if v > hi {
                hi = v;
            }
        }

        Some((lo, hi))
    }
}

/// AVX2 [`crate::simd::scalar::sum`].
///
/// # Safety
/// Requires runtime AVX2 support. See the module-level safety contract.
#[target_feature(enable = "avx2")]
pub unsafe fn sum(values: &[u64]) -> u64 {
    // SAFETY: see module-level safety contract. Loop bounds keep every access
    // within `values`.
    unsafe {
        let n = values.len();
        let base_ptr = values.as_ptr();

        // Four independent addition chains — see `UNROLL`. `add` has 1-cycle
        // latency but the loop is throughput-bound, so multiple chains matter.
        let mut acc = [_mm256_setzero_si256(); UNROLL];

        let mut i = 0usize;
        while i + BLOCK <= n {
            for (u, a) in acc.iter_mut().enumerate() {
                let v = _mm256_loadu_si256(base_ptr.add(i + u * LANES) as *const __m256i);
                // Wrapping is intended and matches the scalar kernel.
                *a = _mm256_add_epi64(*a, v);
            }
            i += BLOCK;
        }

        // Any whole vectors left after the last full block.
        let mut tail_acc = _mm256_setzero_si256();
        while i + LANES <= n {
            let v = _mm256_loadu_si256(base_ptr.add(i) as *const __m256i);
            tail_acc = _mm256_add_epi64(tail_acc, v);
            i += LANES;
        }

        let mut combined = tail_acc;
        for a in acc {
            combined = _mm256_add_epi64(combined, a);
        }

        let mut lanes = [0u64; LANES];
        _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, combined);
        let mut total = lanes.iter().fold(0u64, |a, &b| a.wrapping_add(b));

        for k in i..n {
            total = total.wrapping_add(*values.get_unchecked(k));
        }
        total
    }
}

/// AVX2 [`crate::simd::scalar::find_first_eq`].
///
/// # Safety
/// Requires runtime AVX2 support. See the module-level safety contract.
#[target_feature(enable = "avx2")]
pub unsafe fn find_first_eq(values: &[u64], needle: u64) -> Option<usize> {
    // SAFETY: see module-level safety contract.
    unsafe {
        let n = values.len();
        let chunks = n / LANES;
        let base_ptr = values.as_ptr();
        // Equality needs no range shift — bit patterns compare identically.
        let needle_v = _mm256_set1_epi64x(needle as i64);

        for c in 0..chunks {
            let v = _mm256_loadu_si256(base_ptr.add(c * LANES) as *const __m256i);
            let eq = _mm256_cmpeq_epi64(v, needle_v);
            let bits = _mm256_movemask_pd(_mm256_castsi256_pd(eq)) as u32 & 0x0F;
            if bits != 0 {
                // trailing_zeros gives the lowest matching lane — the FIRST match.
                return Some(c * LANES + bits.trailing_zeros() as usize);
            }
        }

        for i in (chunks * LANES)..n {
            if *values.get_unchecked(i) == needle {
                return Some(i);
            }
        }
        None
    }
}

/// AVX2 [`crate::simd::scalar::zone_overlap_indices`].
///
/// Overlap condition per zone: `mins[i] <= hi && maxes[i] >= lo`.
///
/// # Safety
/// Requires runtime AVX2 support. See the module-level safety contract.
#[target_feature(enable = "avx2")]
pub unsafe fn zone_overlap_indices(
    mins: &[u64],
    maxes: &[u64],
    lo: u64,
    hi: u64,
    out: &mut Vec<u32>,
) {
    debug_assert_eq!(mins.len(), maxes.len());
    // SAFETY: see module-level safety contract. Indices bounded by
    // `chunks * LANES <= mins.len()`; loads are unaligned-safe.
    unsafe {
        let flip = _mm256_set1_epi64x(FLIP);
        // Broadcast flipped query bounds so comparisons are unsigned-correct.
        let lo_v = _mm256_xor_si256(_mm256_set1_epi64x(lo as i64), flip);
        let hi_v = _mm256_xor_si256(_mm256_set1_epi64x(hi as i64), flip);

        let n = mins.len();
        let chunks = n / LANES;
        let min_ptr = mins.as_ptr();
        let max_ptr = maxes.as_ptr();

        for c in 0..chunks {
            let base = c * LANES;
            let raw_min = _mm256_loadu_si256(min_ptr.add(base) as *const __m256i);
            let raw_max = _mm256_loadu_si256(max_ptr.add(base) as *const __m256i);

            // Apply unsigned-to-signed range shift.
            let min_s = _mm256_xor_si256(raw_min, flip);
            let max_s = _mm256_xor_si256(raw_max, flip);

            // Prune lane if min > hi (unsigned) or lo > max (unsigned).
            let min_above_hi = _mm256_cmpgt_epi64(min_s, hi_v);
            let lo_above_max = _mm256_cmpgt_epi64(lo_v, max_s);
            let prune = _mm256_or_si256(min_above_hi, lo_above_max);

            let prune_bits = _mm256_movemask_pd(_mm256_castsi256_pd(prune)) as u32;
            let mut good = !prune_bits & 0x0F;

            while good != 0 {
                let lane = good.trailing_zeros() as usize;
                out.push((base + lane) as u32);
                good &= good - 1;
            }
        }

        for i in (chunks * LANES)..n {
            let min = *min_ptr.add(i);
            let max = *max_ptr.add(i);
            if min <= hi && max >= lo {
                out.push(i as u32);
            }
        }
    }
}

/// Hardware CRC-32C via the `crc32` instruction (SSE4.2).
///
/// ~20× faster than the bitwise scalar version. Produces identical output to
/// [`crate::simd::scalar::crc32c`] — and, as documented there, a *different*
/// value from CRC-32/ISO-HDLC.
///
/// # Safety
/// Requires runtime SSE4.2 support.
#[target_feature(enable = "sse4.2")]
pub unsafe fn crc32c(data: &[u8]) -> u32 {
    // SAFETY: `chunks * 8 <= data.len()` bounds the word loop; the tail index is
    // `< data.len()` by its loop bound. `read_unaligned` handles any alignment.
    unsafe {
        let mut crc = !0u32 as u64;
        let chunks = data.len() / 8;
        let base_ptr = data.as_ptr();

        for c in 0..chunks {
            let word = (base_ptr.add(c * 8) as *const u64).read_unaligned();
            crc = _mm_crc32_u64(crc, word);
        }

        let mut crc = crc as u32;
        for i in (chunks * 8)..data.len() {
            crc = _mm_crc32_u8(crc, *data.get_unchecked(i));
        }
        !crc
    }
}
