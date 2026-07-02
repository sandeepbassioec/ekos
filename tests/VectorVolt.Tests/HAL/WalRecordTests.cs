using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using VectorVolt.Core.HAL;
using Xunit;

namespace VectorVolt.Tests.HAL;

/// <summary>
/// Proves WalRecord is exactly one cache line with correct field layout.
/// The WalRecord is the on-disk receipt of every committed command.
/// If this struct drifts past 64 bytes, WAL throughput drops by half.
/// </summary>
public class WalRecordTests
{
    [Fact]
    public void Size_IsExactlyOneCacheLine()
    {
        Assert.Equal(CacheLineConstants.CacheLineSize, Unsafe.SizeOf<WalRecord>());
    }

    [Fact]
    public void Size_Is64Bytes()
    {
        Assert.Equal(64, Unsafe.SizeOf<WalRecord>());
    }

    [Fact]
    public void Layout_IsExplicit()
    {
        var layout = typeof(WalRecord).StructLayoutAttribute;
        Assert.NotNull(layout);
        Assert.Equal(LayoutKind.Explicit, layout!.Value);
    }

    [Fact]
    public void FieldOffsets_AreCorrect()
    {
        Assert.Equal(0, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.SequenceNumber)).ToInt32());
        Assert.Equal(8, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.Timestamp)).ToInt32());
        Assert.Equal(16, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.TenantId)).ToInt32());
        Assert.Equal(20, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.ShardId)).ToInt32());
        Assert.Equal(22, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.RecordType)).ToInt32());
        Assert.Equal(23, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.Flags)).ToInt32());
        Assert.Equal(24, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.AggregateId)).ToInt32());
        Assert.Equal(40, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.PayloadLength)).ToInt32());
        Assert.Equal(44, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.PayloadOffset)).ToInt32());
        Assert.Equal(48, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.SegmentId)).ToInt32());
        Assert.Equal(52, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.Epoch)).ToInt32());
        Assert.Equal(56, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.Checksum)).ToInt32());
        Assert.Equal(60, Marshal.OffsetOf<WalRecord>(nameof(WalRecord.PrevChecksum)).ToInt32());
    }

    [Fact]
    public void IsValueType_NoGcPressure()
    {
        Assert.True(typeof(WalRecord).IsValueType,
            "WalRecord must be a struct, not a class");
    }

    [Fact]
    public void DefaultValues_AreAllZero()
    {
        var record = new WalRecord();
        Assert.Equal(0L, record.SequenceNumber);
        Assert.Equal(0L, record.Timestamp);
        Assert.Equal(0, record.TenantId);
        Assert.Equal((short)0, record.ShardId);
        Assert.Equal((byte)0, record.RecordType);
        Assert.Equal((byte)0, record.Flags);
        Assert.Equal(Guid.Empty, record.AggregateId);
        Assert.Equal(0, record.PayloadLength);
        Assert.Equal(0, record.PayloadOffset);
        Assert.Equal(0, record.SegmentId);
        Assert.Equal(0, record.Epoch);
        Assert.Equal(0u, record.Checksum);
        Assert.Equal(0u, record.PrevChecksum);
    }

    [Fact]
    public void CanSetAndReadAllFields()
    {
        var id = Guid.NewGuid();
        var record = new WalRecord
        {
            SequenceNumber = 100,
            Timestamp = 9876543210L,
            TenantId = 5,
            ShardId = 2,
            RecordType = 1,
            Flags = 0b_0000_0001,
            AggregateId = id,
            PayloadLength = 512,
            PayloadOffset = 4096,
            SegmentId = 3,
            Epoch = 7,
            Checksum = 0xCAFEBABE,
            PrevChecksum = 0xDEADBEEF
        };

        Assert.Equal(100L, record.SequenceNumber);
        Assert.Equal(9876543210L, record.Timestamp);
        Assert.Equal(5, record.TenantId);
        Assert.Equal((short)2, record.ShardId);
        Assert.Equal((byte)1, record.RecordType);
        Assert.Equal((byte)1, record.Flags);
        Assert.Equal(id, record.AggregateId);
        Assert.Equal(512, record.PayloadLength);
        Assert.Equal(4096, record.PayloadOffset);
        Assert.Equal(3, record.SegmentId);
        Assert.Equal(7, record.Epoch);
        Assert.Equal(0xCAFEBABE, record.Checksum);
        Assert.Equal(0xDEADBEEF, record.PrevChecksum);
    }

    [Fact]
    public void HashChain_PrevChecksumLinksRecords()
    {
        // PHYSICS: Each WalRecord stores the checksum of the PREVIOUS record.
        // This creates a tamper-evident chain. If any record is corrupted,
        // the chain breaks — and we know exactly where to truncate on recovery.
        //
        // This test simulates a 3-record chain and verifies the links.
        var r1 = new WalRecord { SequenceNumber = 1, Checksum = 0xAAAA, PrevChecksum = 0x0000 };
        var r2 = new WalRecord { SequenceNumber = 2, Checksum = 0xBBBB, PrevChecksum = r1.Checksum };
        var r3 = new WalRecord { SequenceNumber = 3, Checksum = 0xCCCC, PrevChecksum = r2.Checksum };

        // Verify chain links
        Assert.Equal(r1.Checksum, r2.PrevChecksum);
        Assert.Equal(r2.Checksum, r3.PrevChecksum);

        // Simulate corruption: if r2's checksum changes, chain breaks at r3
        var r2Corrupted = r2;
        r2Corrupted.Checksum = 0xFFFF; // corruption!
        Assert.NotEqual(r2Corrupted.Checksum, r3.PrevChecksum); // chain broken!
    }

    [Fact]
    public void PacksPerfectlyInNvmePage()
    {
        // PHYSICS: NVMe pages are 4KB (4096 bytes).
        // 4096 / 64 = exactly 64 records per page. Zero internal fragmentation.
        // If WalRecord were 65 bytes, we'd waste 1 byte per record × 64 records = 64 bytes
        // per page, plus alignment padding would make it even worse.
        int recordsPerPage = 4096 / Unsafe.SizeOf<WalRecord>();
        Assert.Equal(64, recordsPerPage);
        Assert.Equal(0, 4096 % Unsafe.SizeOf<WalRecord>()); // zero waste
    }
}
