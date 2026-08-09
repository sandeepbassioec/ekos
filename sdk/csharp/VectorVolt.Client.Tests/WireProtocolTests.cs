using System.Buffers.Binary;
using System.Text;
using VectorVolt.Client;
using VectorVolt.Client.Protocol;
using VectorVolt.Client.Transport;
using Xunit;

namespace VectorVolt.Client.Tests;

/// <summary>
/// Unit tests for the wire protocol codec.
/// No server or TCP socket required — everything runs in-process.
/// </summary>
public class WireProtocolTests
{
    // ---- BuildFrame ----

    [Fact]
    public void BuildFrame_EmptyPayload_HeaderIsCorrect()
    {
        var frame = WireProtocol.BuildFrame(0x01, requestId: 42, payload: ReadOnlySpan<byte>.Empty);

        Assert.Equal(WireProtocol.HeaderSize, frame.Length);
        Assert.Equal((byte)'V', frame[0]);
        Assert.Equal((byte)'V', frame[1]);
        Assert.Equal(1,         frame[2]);   // version
        Assert.Equal(0x01,      frame[3]);   // opcode
        Assert.Equal(0,         frame[4]);   // flags
        Assert.Equal(42UL, BinaryPrimitives.ReadUInt64LittleEndian(frame.AsSpan(8)));
        Assert.Equal(0U,   BinaryPrimitives.ReadUInt32LittleEndian(frame.AsSpan(16)));  // payload_len
        Assert.Equal(0U,   BinaryPrimitives.ReadUInt32LittleEndian(frame.AsSpan(20)));  // crc32 of empty
    }

    [Fact]
    public void BuildFrame_WithPayload_PayloadAppendedAfterHeader()
    {
        byte[] payload = [0x01, 0x02, 0x03];
        var frame = WireProtocol.BuildFrame(0x20, requestId: 1, payload: payload);

        Assert.Equal(WireProtocol.HeaderSize + 3, frame.Length);
        Assert.Equal(3U, BinaryPrimitives.ReadUInt32LittleEndian(frame.AsSpan(16)));
        Assert.Equal(new byte[] { 0x01, 0x02, 0x03 }, frame[WireProtocol.HeaderSize..]);
    }

    [Fact]
    public void BuildFrame_Crc32_MatchesPayload()
    {
        byte[] payload = Encoding.UTF8.GetBytes("hello world");
        var frame = WireProtocol.BuildFrame(0x01, requestId: 99, payload: payload);

        uint embedded = BinaryPrimitives.ReadUInt32LittleEndian(frame.AsSpan(20));
        uint expected = WireProtocol.Crc32(payload);
        Assert.Equal(expected, embedded);
    }

    [Fact]
    public void BuildFrame_DifferentPayloads_DifferentCrc()
    {
        byte[] p1 = [0xAA, 0xBB];
        byte[] p2 = [0xCC, 0xDD];
        uint c1 = WireProtocol.Crc32(p1);
        uint c2 = WireProtocol.Crc32(p2);
        Assert.NotEqual(c1, c2);
    }

    // ---- ReadHeader + ReadPayload via MemoryTransport ----

    [Fact]
    public void RoundTrip_EmptyPayload()
    {
        var frame = WireProtocol.BuildFrame(0x01, requestId: 7, payload: ReadOnlySpan<byte>.Empty);
        var transport = new MemoryTransport(frame);

        var header = WireProtocol.ReadHeader(transport);
        Assert.Equal(0x01,  header.Opcode);
        Assert.Equal(7UL,   header.RequestId);
        Assert.Equal(0U,    header.PayloadLen);

        var payload = WireProtocol.ReadPayload(transport, header);
        Assert.Empty(payload);
    }

    [Fact]
    public void RoundTrip_WithPayload_DataIntact()
    {
        byte[] original = Encoding.UTF8.GetBytes("VectorVolt");
        var frame = WireProtocol.BuildFrame(0x30, requestId: 123, payload: original);
        var transport = new MemoryTransport(frame);

        var header = WireProtocol.ReadHeader(transport);
        var payload = WireProtocol.ReadPayload(transport, header);

        Assert.Equal(original, payload);
    }

    [Fact]
    public void ReadHeader_BadMagic_Throws()
    {
        byte[] frame = new byte[WireProtocol.HeaderSize];
        frame[0] = 0xDE; frame[1] = 0xAD;   // wrong magic
        frame[2] = 1;                        // version

        var transport = new MemoryTransport(frame);
        Assert.Throws<VectorVoltProtocolException>(() => WireProtocol.ReadHeader(transport));
    }

    [Fact]
    public void ReadHeader_BadVersion_Throws()
    {
        byte[] frame = new byte[WireProtocol.HeaderSize];
        frame[0] = (byte)'V'; frame[1] = (byte)'V';
        frame[2] = 99;   // version mismatch

        var transport = new MemoryTransport(frame);
        Assert.Throws<VectorVoltProtocolException>(() => WireProtocol.ReadHeader(transport));
    }

    [Fact]
    public void ReadPayload_CrcMismatch_Throws()
    {
        byte[] payload = [0x01, 0x02, 0x03];
        var frame = WireProtocol.BuildFrame(0x01, requestId: 1, payload: payload);

        // Corrupt one payload byte
        frame[WireProtocol.HeaderSize + 1] ^= 0xFF;

        var transport = new MemoryTransport(frame);
        var header = WireProtocol.ReadHeader(transport);
        Assert.Throws<VectorVoltProtocolException>(() => WireProtocol.ReadPayload(transport, header));
    }

    // ---- PayloadWriter / PayloadReader ----

    [Fact]
    public void PayloadWriter_Reader_RoundTrip_Primitives()
    {
        Span<byte> buf = stackalloc byte[1 + 2 + 4 + 8];
        var w = new PayloadWriter(buf);
        w.WriteU8(0xAB);
        w.WriteU16(0x1234);
        w.WriteI32(-999);
        w.WriteI64(long.MinValue);

        var r = new PayloadReader(w.Written);
        Assert.Equal(0xAB,          r.ReadU8());
        Assert.Equal(0x1234,        r.ReadU16());
        Assert.Equal(-999,          r.ReadI32());
        Assert.Equal(long.MinValue, r.ReadI64());
    }

    [Fact]
    public void WriteReadString_RoundTrip()
    {
        const string value = "my_table_name";
        Span<byte> buf = stackalloc byte[2 + Encoding.UTF8.GetByteCount(value)];
        var w = new PayloadWriter(buf);
        WireProtocol.WriteString(ref w, value);

        var r = new PayloadReader(w.Written);
        Assert.Equal(value, WireProtocol.ReadString(ref r));
    }

    [Fact]
    public void WriteReadString_EmptyString()
    {
        Span<byte> buf = stackalloc byte[2];
        var w = new PayloadWriter(buf);
        WireProtocol.WriteString(ref w, string.Empty);

        var r = new PayloadReader(w.Written);
        Assert.Equal(string.Empty, WireProtocol.ReadString(ref r));
    }

    [Fact]
    public void WriteReadString_Unicode()
    {
        const string value = "タブル";
        var bytes = Encoding.UTF8.GetBytes(value);
        var buf = new byte[2 + bytes.Length];
        var w = new PayloadWriter(buf);
        WireProtocol.WriteString(ref w, value);

        var r = new PayloadReader(w.Written);
        Assert.Equal(value, WireProtocol.ReadString(ref r));
    }

    // ---- Helpers ----

    /// <summary>In-memory transport backed by a byte array — no sockets.</summary>
    private sealed class MemoryTransport : IVectorVoltTransport
    {
        private readonly byte[] _data;
        private int _readPos;
        private readonly List<byte> _written = new();

        public MemoryTransport(byte[] data) { _data = data; }

        public void Send(ReadOnlySpan<byte> data) => _written.AddRange(data.ToArray());

        public void ReceiveExact(Span<byte> buffer)
        {
            if (_readPos + buffer.Length > _data.Length)
                throw new VectorVoltException("MemoryTransport: not enough data");
            _data.AsSpan(_readPos, buffer.Length).CopyTo(buffer);
            _readPos += buffer.Length;
        }

        public byte[] SentBytes() => _written.ToArray();

        public void Dispose() { }
    }
}
