using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using VectorVolt.Core.HAL;
using Xunit;

namespace VectorVolt.Tests.HAL;

/// <summary>
/// Proves CommandFrame is exactly one cache line and every field
/// sits at its documented byte offset.
///
/// WHY THESE TESTS MATTER:
/// If someone adds a field or changes a type, and the struct grows past 64 bytes,
/// these tests catch it BEFORE the CPU silently starts fetching 2 cache lines
/// per command — which would halve our throughput.
/// </summary>
public class CommandFrameTests
{
    [Fact]
    public void Size_IsExactlyOneCacheLine()
    {
        // THE PROOF: sizeof(CommandFrame) must equal 64.
        // If this fails, someone broke the memory layout.
        int actual = Unsafe.SizeOf<CommandFrame>();
        Assert.Equal(CacheLineConstants.CacheLineSize, actual);
    }

    [Fact]
    public void Size_Is64Bytes()
    {
        // Same test, explicit number — so the test reads clearly
        // without needing to know what CacheLineSize is.
        Assert.Equal(64, Unsafe.SizeOf<CommandFrame>());
    }

    [Fact]
    public void Layout_IsExplicit()
    {
        // Verify the CLR isn't secretly rearranging our fields.
        // LayoutKind.Explicit means WE control the byte layout.
        // StructLayoutAttribute is a pseudo-custom attribute — it's baked into
        // metadata, not returned by GetCustomAttributes on all runtimes.
        // So we read it directly from the type's layout properties.
        var layout = typeof(CommandFrame).StructLayoutAttribute;
        Assert.NotNull(layout);
        Assert.Equal(LayoutKind.Explicit, layout!.Value);
    }

    [Fact]
    public void FieldOffsets_AreCorrect()
    {
        // Verify every field sits at the exact byte offset we documented.
        // This catches accidental reordering or type size changes.
        Assert.Equal(0, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.SequenceNumber)).ToInt32());
        Assert.Equal(8, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.Timestamp)).ToInt32());
        Assert.Equal(16, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.TenantId)).ToInt32());
        Assert.Equal(20, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.ShardId)).ToInt32());
        Assert.Equal(22, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.CommandType)).ToInt32());
        Assert.Equal(23, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.Flags)).ToInt32());
        Assert.Equal(24, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.AggregateId)).ToInt32());
        Assert.Equal(40, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.PayloadLength)).ToInt32());
        Assert.Equal(44, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.PayloadOffset)).ToInt32());
        Assert.Equal(48, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.CorrelationId)).ToInt32());
        Assert.Equal(56, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.Checksum)).ToInt32());
        Assert.Equal(60, Marshal.OffsetOf<CommandFrame>(nameof(CommandFrame.Reserved)).ToInt32());
    }

    [Fact]
    public void IsValueType_NoGcPressure()
    {
        // CommandFrame MUST be a struct (value type), not a class.
        // Classes live on the heap → GC must track them → GC pauses.
        // Structs live on the stack or inline → zero GC pressure.
        Assert.True(typeof(CommandFrame).IsValueType,
            "CommandFrame must be a struct, not a class");
    }

    [Fact]
    public void DefaultValues_AreAllZero()
    {
        // A fresh CommandFrame should be all zeros.
        // This proves there are no hidden constructors doing allocations.
        var frame = new CommandFrame();
        Assert.Equal(0L, frame.SequenceNumber);
        Assert.Equal(0L, frame.Timestamp);
        Assert.Equal(0, frame.TenantId);
        Assert.Equal((short)0, frame.ShardId);
        Assert.Equal((byte)0, frame.CommandType);
        Assert.Equal((byte)0, frame.Flags);
        Assert.Equal(Guid.Empty, frame.AggregateId);
        Assert.Equal(0, frame.PayloadLength);
        Assert.Equal(0, frame.PayloadOffset);
        Assert.Equal(0L, frame.CorrelationId);
        Assert.Equal(0u, frame.Checksum);
        Assert.Equal(0u, frame.Reserved);
    }

    [Fact]
    public void CanSetAndReadAllFields()
    {
        // Verify every field is independently writable and readable.
        // If FieldOffsets overlap, writing one field would corrupt another.
        var id = Guid.NewGuid();
        var frame = new CommandFrame
        {
            SequenceNumber = 42,
            Timestamp = 1234567890L,
            TenantId = 7,
            ShardId = 3,
            CommandType = 1,
            Flags = 0b_0000_0011, // priority + fsync
            AggregateId = id,
            PayloadLength = 256,
            PayloadOffset = 8192,
            CorrelationId = 999L,
            Checksum = 0xDEADBEEF,
            Reserved = 0
        };

        Assert.Equal(42L, frame.SequenceNumber);
        Assert.Equal(1234567890L, frame.Timestamp);
        Assert.Equal(7, frame.TenantId);
        Assert.Equal((short)3, frame.ShardId);
        Assert.Equal((byte)1, frame.CommandType);
        Assert.Equal((byte)3, frame.Flags);
        Assert.Equal(id, frame.AggregateId);
        Assert.Equal(256, frame.PayloadLength);
        Assert.Equal(8192, frame.PayloadOffset);
        Assert.Equal(999L, frame.CorrelationId);
        Assert.Equal(0xDEADBEEF, frame.Checksum);
        Assert.Equal(0u, frame.Reserved);
    }
}
