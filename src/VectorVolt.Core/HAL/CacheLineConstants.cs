namespace VectorVolt.Core.HAL;

/// <summary>
/// Hardware constants derived from the physical properties of modern x86-64 CPUs.
///
/// THE PHYSICS:
/// - CPU caches transfer data in fixed 64-byte blocks called "cache lines"
/// - The TLB (Translation Lookaside Buffer) maps virtual → physical pages
/// - Larger pages = fewer TLB entries needed = fewer expensive page walks
///
/// These constants are NOT configurable — they come from silicon specifications.
/// Intel and AMD have used these values since ~2004 (Pentium 4 / Athlon 64).
/// </summary>
public static class CacheLineConstants
{
    /// <summary>
    /// The size in bytes of one CPU cache line on all modern x86-64 processors.
    ///
    /// PHYSICS: When the CPU reads any byte from RAM, it fetches the entire
    /// 64-byte block containing that byte into L1 cache. This means:
    /// - A 64-byte struct = 1 cache fetch (optimal)
    /// - A 65-byte struct = 2 cache fetches (2x the latency)
    /// - A 40-byte struct = 1 fetch but wastes 24 bytes of bandwidth
    /// </summary>
    public const int CacheLineSize = 64;

    /// <summary>
    /// Large Page (HugePage) size: 2MB.
    ///
    /// PHYSICS: The CPU's TLB has ~64 entries to cache virtual→physical translations.
    /// - With 4KB pages:  64 × 4KB  = 256KB  covered (tiny — constant TLB misses)
    /// - With 2MB pages:  64 × 2MB  = 128MB  covered (good — fits working set)
    /// - With 1GB pages:  64 × 1GB  = 64GB   covered (entire dataset)
    ///
    /// Each TLB miss costs ~10-30 ns — at millions of ops/sec, this adds up fast.
    /// 2MB is the default HugePage size because it's universally supported on all
    /// x86-64 CPUs and operating systems (Linux, Windows).
    /// </summary>
    public const int LargePageSize = 2 * 1024 * 1024;
}