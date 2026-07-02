using System.Runtime.InteropServices;

namespace VectorVolt.Core.HAL;

/// <summary>
/// Zero-GC bump allocator using unmanaged memory.
///
/// THE PHYSICS:
/// .NET's GC pauses ALL threads to scan the managed heap. At 1M+ ops/sec,
/// even a 1 ms pause = 50,000 lost commands. Solution: never use the managed heap.
///
/// HOW IT WORKS:
/// 1. At startup, allocate a big block of unmanaged memory (NativeMemory.AlignedAlloc)
/// 2. The GC doesn't know this memory exists — it can't pause us to scan it
/// 3. To allocate: increment _offset (one addition, ~1 ns)
/// 4. To free everything: set _offset = 0 (one assignment, ~1 ns)
///
/// WHY BUMP ALLOCATOR:
/// Traditional allocators (malloc) maintain free lists, handle fragmentation,
/// merge blocks — ~50-100 ns per allocation. We just bump a pointer: ~1-2 ns.
/// We can do this because the ShardExecutor frees EVERYTHING at once after each
/// command cycle — we never need to free individual allocations.
///
/// ALIGNMENT:
/// Every allocation is rounded up to 64 bytes (cache line boundary).
/// This guarantees no struct ever spans two cache lines.
///
/// LIFECYCLE:
/// Created once at shard startup → used for millions of cycles → disposed at shutdown.
/// Each cycle: Allocate → Process → Reset → Repeat.
/// </summary>
public sealed class MemoryArena : IDisposable
{
    private readonly unsafe byte* _buffer;  // Start of unmanaged memory block
    private readonly int _capacity;          // Total size in bytes (cache-line aligned)
    private int _offset;                     // Current allocation position (the "bump pointer")
    private bool _disposed;                  // Prevent double-free

    /// <summary>
    /// Creates a new MemoryArena with the specified capacity.
    ///
    /// THE PHYSICS:
    /// NativeMemory.AlignedAlloc allocates memory that:
    /// - Is invisible to the .NET GC (zero GC pressure)
    /// - Is aligned to the specified boundary (cache line = 64 bytes)
    /// - Must be manually freed (we do this in Dispose)
    /// </summary>
    /// <param name="capacityBytes">
    /// Requested capacity in bytes. Will be rounded UP to the next
    /// cache line boundary. Example: 100 → 128 bytes.
    /// </param>
    public unsafe MemoryArena(int capacityBytes)
    {
        if (capacityBytes <= 0)
            throw new ArgumentException("Capacity must be positive", nameof(capacityBytes));

        // Round up to cache line boundary: (size + 63) & ~63
        // Example: 100 + 63 = 163 → 163 & ~63 = 163 & 0xFFFFFFC0 = 128
        _capacity = AlignToCacheLine(capacityBytes);

        // Allocate unmanaged memory, aligned to cache line boundary.
        // This memory is INVISIBLE to the GC — zero scanning, zero pauses.
        _buffer = (byte*)NativeMemory.AlignedAlloc(
            byteCount: (nuint)_capacity,
            alignment: (nuint)CacheLineConstants.CacheLineSize
        );

        _offset = 0;
    }

    /// <summary>
    /// How many bytes have been allocated from this arena.
    /// </summary>
    public int UsedBytes => _offset;

    /// <summary>
    /// Total capacity in bytes (cache-line aligned).
    /// </summary>
    public int Capacity => _capacity;

    /// <summary>
    /// Allocate a block of memory from the arena, aligned to cache line boundary.
    ///
    /// HOW IT WORKS (3 steps, ~1-2 ns):
    /// 1. Round requested size up to next 64-byte boundary
    /// 2. Check if we have enough space
    /// 3. Bump the offset pointer and return the old position
    ///
    /// COST: One addition + one comparison. No locks, no free lists, no GC.
    /// </summary>
    /// <param name="sizeBytes">Number of bytes needed. Will be rounded up to 64.</param>
    /// <returns>Pointer to the allocated memory block.</returns>
    public unsafe byte* Allocate(int sizeBytes)
    {
        if (sizeBytes <= 0)
            throw new ArgumentException("Size must be positive", nameof(sizeBytes));

        // Round up to cache line boundary
        int alignedSize = AlignToCacheLine(sizeBytes);

        // Check capacity
        int newOffset = _offset + alignedSize;
        if (newOffset > _capacity)
            throw new OutOfMemoryException(
                $"MemoryArena exhausted: requested {alignedSize} bytes, " +
                $"but only {_capacity - _offset} of {_capacity} bytes remaining");

        // Bump the pointer — this IS the allocation
        byte* ptr = _buffer + _offset;
        _offset = newOffset;

        return ptr;
    }

    /// <summary>
    /// Reclaim ALL memory instantly by resetting the offset to zero.
    ///
    /// THE PHYSICS:
    /// Traditional free: walk free list, merge blocks, update metadata → ~50-100 ns
    /// Our reset: _offset = 0 → ~1 ns. One instruction. All memory reclaimed.
    ///
    /// Called by ShardExecutor after every command batch cycle.
    /// </summary>
    public void Reset()
    {
        _offset = 0;
    }

    /// <summary>
    /// Rounds a size up to the next cache line boundary (multiple of 64).
    ///
    /// THE MATH: (size + 63) &amp; ~63
    /// - Adding 63 ensures any non-multiple rounds up
    /// - Bitwise AND with ~63 (0xFFFFFFC0) clears the low 6 bits
    /// - This is branchless — no if/else, just two CPU instructions
    ///
    /// Examples: 1→64, 40→64, 64→64, 65→128, 100→128
    /// </summary>
    internal static int AlignToCacheLine(int size)
    {
        return (size + (CacheLineConstants.CacheLineSize - 1))
             & ~(CacheLineConstants.CacheLineSize - 1);
    }

    /// <summary>
    /// Frees the unmanaged memory block.
    ///
    /// CRITICAL: Because this memory is invisible to the GC, if we don't call
    /// Dispose, the memory leaks FOREVER. The GC will never reclaim it.
    /// Always use MemoryArena in a 'using' statement or call Dispose explicitly.
    /// </summary>
    public unsafe void Dispose()
    {
        if (!_disposed)
        {
            NativeMemory.AlignedFree(_buffer);
            _disposed = true;
        }
    }
}
