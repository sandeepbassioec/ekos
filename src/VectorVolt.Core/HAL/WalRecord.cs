using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace VectorVolt.Core.HAL;

/// <summary>
/// The on-disk receipt of every committed command in VectorVolt's Write-Ahead Log.
///
/// THE PHYSICS:
/// NVMe drives achieve peak throughput with sequential, aligned writes.
/// At 64 bytes, exactly 64 WalRecords pack into one 4KB NVMe page — zero waste.
/// During in-memory batching (before flush), each record is one cache line fetch.
///
/// THE HASH CHAIN:
/// PrevChecksum links each record to the previous one. If power fails mid-write
/// and a record gets corrupted, the chain breaks at exactly that point.
/// During recovery, we walk the chain forward and truncate at the first break.
/// Same principle as blockchain — but invented decades earlier (Merkle chain).
///
/// MEMORY MAP:
/// ┌────────┬────────┬────┬──┬─┬─┬────────────────┬────┬────┬────┬────┬────┬────┐
/// │SeqNum  │Timestmp│Tid │Sh│R│F│  AggregateId   │PLen│POff│Seg │Epo │CRC │Prev│
/// │ 8B     │ 8B     │ 4B │2B│1│1│    16B Guid    │ 4B │ 4B │ 4B │ 4B │ 4B │ 4B │
/// └────────┴────────┴────┴──┴─┴─┴────────────────┴────┴────┴────┴────┴────┴────┘
/// Offset:0   8       16   20 22 23 24             40   44   48   52   56   60
/// Total: 64 bytes = 1 cache line = 1/64th of a 4KB NVMe page.
/// </summary>
[StructLayout(LayoutKind.Explicit, Size = CacheLineConstants.CacheLineSize)]
public struct WalRecord
{
    /// <summary>
    /// Links back to the CommandFrame that produced this record.
    /// Same monotonic sequence — the source of deterministic replay.
    /// Offset: 0 | Size: 8 bytes
    /// </summary>
    [FieldOffset(0)]
    public long SequenceNumber;

    /// <summary>
    /// Commit timestamp in UTC ticks. This is when the engine committed
    /// the command, not when the client sent it.
    /// Offset: 8 | Size: 8 bytes
    /// </summary>
    [FieldOffset(8)]
    public long Timestamp;

    /// <summary>
    /// Tenant isolation key — scopes this record to one tenant.
    /// Offset: 16 | Size: 4 bytes
    /// </summary>
    [FieldOffset(16)]
    public int TenantId;

    /// <summary>
    /// The shard that processed and committed this command.
    /// Offset: 20 | Size: 2 bytes
    /// </summary>
    [FieldOffset(20)]
    public short ShardId;

    /// <summary>
    /// Distinguishes record types in the WAL:
    ///   0 = Command (normal committed transaction)
    ///   1 = Snapshot marker (snapshot taken at this LSN)
    ///   2 = Epoch fence (leader change — prevents split-brain replay)
    /// Offset: 22 | Size: 1 byte
    /// </summary>
    [FieldOffset(22)]
    public byte RecordType;

    /// <summary>
    /// Bitflags for WAL record metadata.
    /// Offset: 23 | Size: 1 byte
    /// </summary>
    [FieldOffset(23)]
    public byte Flags;

    /// <summary>
    /// The entity this record applies to.
    /// Offset: 24 | Size: 16 bytes
    /// </summary>
    [FieldOffset(24)]
    public Guid AggregateId;

    /// <summary>
    /// Size of the serialized event payload in the WAL segment file.
    /// Offset: 40 | Size: 4 bytes
    /// </summary>
    [FieldOffset(40)]
    public int PayloadLength;

    /// <summary>
    /// Byte offset within the WAL segment file where the payload begins.
    /// The payload is written sequentially after the header batch.
    /// Offset: 44 | Size: 4 bytes
    /// </summary>
    [FieldOffset(44)]
    public int PayloadOffset;

    /// <summary>
    /// Which WAL segment file contains this record's payload.
    /// WAL is split into fixed-size segments (256MB each) for manageability.
    /// Offset: 48 | Size: 4 bytes
    /// </summary>
    [FieldOffset(48)]
    public int SegmentId;

    /// <summary>
    /// Leader election epoch. During recovery, records from a stale leader
    /// (lower epoch) are discarded. This prevents split-brain data corruption.
    /// Offset: 52 | Size: 4 bytes
    /// </summary>
    [FieldOffset(52)]
    public int Epoch;

    /// <summary>
    /// CRC32 checksum of THIS record's contents.
    /// Offset: 56 | Size: 4 bytes
    /// </summary>
    [FieldOffset(56)]
    public uint Checksum;

    /// <summary>
    /// CRC32 checksum of the PREVIOUS record — creates the hash chain.
    /// If power fails mid-write, the chain break tells us exactly
    /// where to truncate. Walk forward, stop at first mismatch.
    /// Offset: 60 | Size: 4 bytes
    /// </summary>
    [FieldOffset(60)]
    public uint PrevChecksum;
}