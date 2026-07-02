using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace VectorVolt.Core.HAL;

/// <summary>
/// The fundamental unit of work in VectorVolt. Every command from every client
/// enters the engine inside this 64-byte struct.
///
/// THE PHYSICS:
/// The CPU fetches data in 64-byte cache lines. This struct is exactly 64 bytes,
/// so ONE cache fetch loads the ENTIRE command. At millions of commands/sec,
/// this is the difference between microsecond and millisecond totals.
///
/// THE DESIGN:
/// Fixed-size header (this struct) + variable-size payload (in MemoryArena).
/// Like a postal envelope: the envelope is always 64 bytes, the package varies.
///
/// MEMORY MAP:
/// ┌────────┬────────┬────┬──┬─┬─┬────────────────┬────┬────┬────────┬────┬────┐
/// │SeqNum  │Timestmp│Tid │Sh│T│F│  AggregateId   │PLen│POff│CorrId  │CRC │Rsv │
/// │ 8B     │ 8B     │ 4B │2B│1│1│    16B Guid    │ 4B │ 4B │ 8B     │ 4B │ 4B │
/// └────────┴────────┴────┴──┴─┴─┴────────────────┴────┴────┴────────┴────┴────┘
/// Offset:0   8       16   20 22 23 24             40   44   48       56   60
/// Total: 64 bytes = 1 cache line. Zero waste. Zero GC.
/// </summary>
[StructLayout(LayoutKind.Explicit, Size = CacheLineConstants.CacheLineSize)]
public struct CommandFrame
{
    /// <summary>
    /// Monotonically increasing sequence number assigned by the ShardExecutor.
    /// This is the TOTAL ORDER — the source of determinism in the entire system.
    /// Every command gets a unique, ever-increasing number. During recovery,
    /// we replay commands in this order to rebuild state identically.
    /// Offset: 0 | Size: 8 bytes (long)
    /// </summary>
    [FieldOffset(0)]
    public long SequenceNumber;

    /// <summary>
    /// Engine-assigned timestamp in UTC ticks (100-nanosecond precision).
    /// NOT the client's wall clock — the engine assigns this to guarantee
    /// consistent ordering across all replicas.
    /// Offset: 8 | Size: 8 bytes (long)
    /// </summary>
    [FieldOffset(8)]
    public long Timestamp;

    /// <summary>
    /// Multi-tenant isolation key. Every piece of data in VectorVolt is scoped
    /// to a tenant. This is checked FIRST by the PolicyEngine — before any
    /// business logic runs.
    /// Offset: 16 | Size: 4 bytes (int) — supports ~2 billion tenants
    /// </summary>
    [FieldOffset(16)]
    public int TenantId;

    /// <summary>
    /// Target shard, determined by consistent hashing on AggregateId.
    /// Offset: 20 | Size: 2 bytes (short) — supports 32,767 shards
    /// </summary>
    [FieldOffset(20)]
    public short ShardId;

    /// <summary>
    /// Operation code: Create=1, Update=2, Delete=3, etc.
    /// Offset: 22 | Size: 1 byte — supports 256 command types
    /// </summary>
    [FieldOffset(22)]
    public byte CommandType;

    /// <summary>
    /// Bitflags for command metadata:
    /// Bit 0: High priority (QoS — processed before normal commands)
    /// Bit 1: Requires immediate fsync (don't batch, flush WAL now)
    /// Bit 2-7: Reserved for future use
    /// Offset: 23 | Size: 1 byte
    /// </summary>
    [FieldOffset(23)]
    public byte Flags;

    /// <summary>
    /// 128-bit unique identifier for the target entity/aggregate.
    /// Using Guid gives globally unique IDs without coordination between nodes.
    /// Offset: 24 (divisible by 8 — good SIMD alignment) | Size: 16 bytes
    /// </summary>
    [FieldOffset(24)]
    public Guid AggregateId;

    /// <summary>
    /// Length of the variable-size payload stored in MemoryArena.
    /// The actual payload data is NOT in this struct — that would blow
    /// the 64-byte budget. We store only a pointer (offset + length).
    /// Offset: 40 | Size: 4 bytes — supports payloads up to ~2 GB
    /// </summary>
    [FieldOffset(40)]
    public int PayloadLength;

    /// <summary>
    /// Byte offset into the MemoryArena where the payload begins.
    /// Arena stores the actual command data (serialized entity, etc).
    /// Offset: 44 | Size: 4 bytes
    /// </summary>
    [FieldOffset(44)]
    public int PayloadOffset;

    /// <summary>
    /// Distributed tracing ID. Tracks a command from client SDK →
    /// engine → WAL → projections. Essential for debugging in production.
    /// Offset: 48 | Size: 8 bytes (long)
    /// </summary>
    [FieldOffset(48)]
    public long CorrelationId;

    /// <summary>
    /// CRC32 checksum of the frame contents for integrity verification.
    /// Computed before WAL write, verified on WAL replay during recovery.
    /// Uses hardware-accelerated Sse42.Crc32 (~1 cycle per 8 bytes).
    /// Offset: 56 | Size: 4 bytes (uint)
    /// </summary>
    [FieldOffset(56)]
    public uint Checksum;

    /// <summary>
    /// Reserved padding to fill exactly 64 bytes.
    /// Available for future fields without breaking the cache line alignment.
    /// Offset: 60 | Size: 4 bytes (uint)
    /// </summary>
    [FieldOffset(60)]
    public uint Reserved;
}
