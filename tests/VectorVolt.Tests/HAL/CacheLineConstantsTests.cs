using VectorVolt.Core.HAL;
using Xunit;

namespace VectorVolt.Tests.HAL;

/// <summary>
/// Proves that our hardware constants match the actual CPU silicon specs.
/// These values are physical properties — if they're wrong, every struct
/// we build on top of them will be misaligned.
/// </summary>
public class CacheLineConstantsTests
{
    [Fact]
    public void CacheLineSize_Is64Bytes()
    {
        // WHY 64: Every x86-64 CPU since Pentium 4 uses 64-byte cache lines.
        // This is a hardware constant — not a software choice.
        Assert.Equal(64, CacheLineConstants.CacheLineSize);
    }

    [Fact]
    public void CacheLineSize_IsPowerOfTwo()
    {
        // WHY: Cache lines must be power-of-2 for bitwise alignment tricks.
        // Example: (address & ~63) gives the cache-line-aligned address.
        // If it's not power-of-2, this bitwise trick breaks.
        int size = CacheLineConstants.CacheLineSize;
        Assert.True(size > 0 && (size & (size - 1)) == 0,
            $"CacheLineSize {size} is not a power of two");
    }

    [Fact]
    public void LargePageSize_Is2MB()
    {
        // WHY 2MB: x86-64 supports 4KB, 2MB, and 1GB pages.
        // 2MB is the sweet spot — universally supported and 512x fewer TLB entries than 4KB.
        Assert.Equal(2 * 1024 * 1024, CacheLineConstants.LargePageSize);
    }

    [Fact]
    public void LargePageSize_IsMultipleOfCacheLine()
    {
        // WHY: A large page must contain a whole number of cache lines.
        // If it doesn't, the last cache line would span two pages — causing a page fault
        // in the middle of a cache line fetch. That would be catastrophic for performance.
        Assert.Equal(0, CacheLineConstants.LargePageSize % CacheLineConstants.CacheLineSize);
    }
}
