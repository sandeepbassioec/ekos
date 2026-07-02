using VectorVolt.Core.HAL;
using Xunit;

namespace VectorVolt.Tests.HAL;

/// <summary>
/// Proves MemoryArena correctly allocates and resets unmanaged memory
/// with cache-line alignment. Every allocation must be aligned to 64 bytes
/// so that structs never span two cache lines.
/// </summary>
public class MemoryArenaTests : IDisposable
{
    // Small arena for testing — 4KB (64 cache lines)
    private readonly MemoryArena _arena = new(4096);

    public void Dispose()
    {
        // CRITICAL: MemoryArena uses unmanaged memory.
        // If we don't Dispose, the memory leaks forever (GC can't see it).
        _arena.Dispose();
    }

    [Fact]
    public void NewArena_HasZeroUsedBytes()
    {
        // A fresh arena should have nothing allocated.
        Assert.Equal(0, _arena.UsedBytes);
    }

    [Fact]
    public void NewArena_ReportsCorrectCapacity()
    {
        // Capacity is rounded UP to the nearest cache line boundary.
        // 4096 is already a multiple of 64, so no rounding needed.
        Assert.Equal(4096, _arena.Capacity);
    }

    [Fact]
    public void Allocate_ReturnsNonNullPointer()
    {
        // We should get a valid pointer to unmanaged memory.
        unsafe
        {
            byte* ptr = _arena.Allocate(64);
            Assert.True(ptr != null, "Allocate returned null");
        }
    }

    [Fact]
    public void Allocate_ReturnsCacheLineAlignedPointer()
    {
        // PHYSICS: Every allocation must start at a 64-byte boundary.
        // If the pointer address % 64 != 0, the data spans two cache lines.
        unsafe
        {
            byte* ptr = _arena.Allocate(64);
            long address = (long)ptr;
            Assert.Equal(0, address % CacheLineConstants.CacheLineSize);
        }
    }

    [Fact]
    public void Allocate_RoundsUpToCacheLineBoundary()
    {
        // Request 40 bytes — should consume 64 bytes (rounded up).
        // This ensures the NEXT allocation is also cache-line aligned.
        unsafe
        {
            _arena.Allocate(40);
            Assert.Equal(64, _arena.UsedBytes); // 40 rounded up to 64
        }
    }

    [Fact]
    public void Allocate_SequentialCallsReturnDifferentPointers()
    {
        // Two allocations must not overlap — each gets its own memory.
        unsafe
        {
            byte* ptr1 = _arena.Allocate(64);
            byte* ptr2 = _arena.Allocate(64);
            Assert.True(ptr2 > ptr1, "Second allocation should be after first");
            Assert.True((ptr2 - ptr1) >= 64, "Allocations must not overlap");
        }
    }

    [Fact]
    public void Allocate_SecondPointerIsAlsoCacheLineAligned()
    {
        // Not just the first — EVERY allocation must be aligned.
        unsafe
        {
            _arena.Allocate(64);  // first
            byte* ptr2 = _arena.Allocate(64);  // second
            long address = (long)ptr2;
            Assert.Equal(0, address % CacheLineConstants.CacheLineSize);
        }
    }

    [Fact]
    public void Allocate_TracksUsedBytesCorrectly()
    {
        unsafe
        {
            _arena.Allocate(64);   // 64 bytes used
            _arena.Allocate(128);  // 128 more → 192 total
            _arena.Allocate(40);   // 40 rounded to 64 → 256 total
            Assert.Equal(256, _arena.UsedBytes);
        }
    }

    [Fact]
    public void Allocate_CanWriteAndReadData()
    {
        // Prove the memory is actually usable — write a pattern and read it back.
        unsafe
        {
            byte* ptr = _arena.Allocate(64);

            // Write a known pattern
            for (int i = 0; i < 64; i++)
            {
                ptr[i] = (byte)(i & 0xFF);
            }

            // Read it back and verify
            for (int i = 0; i < 64; i++)
            {
                Assert.Equal((byte)(i & 0xFF), ptr[i]);
            }
        }
    }

    [Fact]
    public void Reset_SetsUsedBytesToZero()
    {
        // PHYSICS: Reset is how we reclaim ALL memory instantly.
        // Instead of freeing individual allocations (expensive),
        // we just set _offset = 0 (one instruction, ~1 ns).
        unsafe
        {
            _arena.Allocate(64);
            _arena.Allocate(128);
            Assert.True(_arena.UsedBytes > 0);

            _arena.Reset();
            Assert.Equal(0, _arena.UsedBytes);
        }
    }

    [Fact]
    public void Reset_AllowsReuseOfMemory()
    {
        // After reset, we can allocate again from the beginning.
        // The total capacity is still available.
        unsafe
        {
            _arena.Allocate(64);
            _arena.Allocate(64);
            _arena.Reset();

            // Should succeed — we have full capacity again
            byte* ptr = _arena.Allocate(64);
            Assert.True(ptr != null);
            Assert.Equal(64, _arena.UsedBytes);
        }
    }

    [Fact]
    public void Allocate_ThrowsWhenArenaIsFull()
    {
        // If we try to allocate more than capacity, it must fail loudly.
        // Silent failure (returning garbage pointer) would corrupt data.
        unsafe
        {
            // Fill the arena: 4096 / 64 = 64 allocations
            for (int i = 0; i < 64; i++)
            {
                _arena.Allocate(64);
            }

            Assert.Equal(4096, _arena.UsedBytes);

            // The 65th allocation should throw — arena is full
            Assert.Throws<OutOfMemoryException>(() => _arena.Allocate(64));
        }
    }

    [Fact]
    public void Allocate_ZeroBytes_ThrowsArgumentException()
    {
        // Allocating 0 bytes makes no sense — it's always a bug.
        unsafe
        {
            Assert.Throws<ArgumentException>(() => _arena.Allocate(0));
        }
    }

    [Fact]
    public void Allocate_NegativeBytes_ThrowsArgumentException()
    {
        unsafe
        {
            Assert.Throws<ArgumentException>(() => _arena.Allocate(-1));
        }
    }

    [Fact]
    public void Capacity_RoundsUpToCacheLineBoundary()
    {
        // If someone creates an arena with size 100, it should round up to 128
        // (next multiple of 64) so the last allocation slot is complete.
        using var oddArena = new MemoryArena(100);
        Assert.Equal(128, oddArena.Capacity);
    }

    [Fact]
    public void MultipleResetCycles_WorkCorrectly()
    {
        // Simulates the ShardExecutor's real pattern:
        // allocate batch → process → reset → repeat
        unsafe
        {
            for (int cycle = 0; cycle < 10; cycle++)
            {
                // Allocate some data this cycle
                byte* ptr = _arena.Allocate(64);
                ptr[0] = (byte)cycle;  // Write cycle number

                _arena.Allocate(128);
                Assert.True(_arena.UsedBytes > 0);

                // Reset for next cycle
                _arena.Reset();
                Assert.Equal(0, _arena.UsedBytes);
            }
        }
    }
}
