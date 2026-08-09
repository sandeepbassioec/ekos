using System.Buffers.Binary;
using System.IO.Hashing;
using System.Text;
using VectorVolt.Client.Transport;

namespace VectorVolt.Client.Protocol;

/// <summary>
/// Pure codec for the VectorVolt binary wire protocol.
///
/// <para>Frame layout (all fields little-endian):</para>
/// <code>
/// Offset  Size  Field
/// 0       2     magic          = 0x56 0x56 ('VV')
/// 2       1     version        = 1
/// 3       1     opcode
/// 4       1     flags
/// 5       3     reserved       = 0x00 0x00 0x00
/// 8       8     request_id     (u64)
/// 16      4     payload_len    (u32)
/// 20      4     payload_crc32  (CRC32 of payload bytes; 0 if payload_len == 0)
/// 24      N     payload
/// </code>
/// </summary>
internal static class WireProtocol
{
    internal const int HeaderSize = 24;
    private  const byte MagicV    = (byte)'V';
    private  const byte Version   = 1;

    // ---- Opcodes (request) ----
    internal static class Op
    {
        // Auth
        internal const byte Hello     = 0x01;
        internal const byte Auth      = 0x03;

        // Table DDL (control plane)
        internal const byte CreateTable   = 0x10;
        internal const byte DropTable     = 0x11;
        internal const byte ListTables    = 0x12;
        internal const byte UseTable      = 0x13;
        internal const byte TableStats    = 0x14;
        internal const byte DescribeTable = 0x15;
        internal const byte TableExists   = 0x16;

        // DML
        internal const byte Insert        = 0x20;
        internal const byte BatchInsert   = 0x21;
        internal const byte Get           = 0x30;
        internal const byte Scan          = 0x31;
        internal const byte Delete        = 0x40;
        internal const byte BatchDelete   = 0x41;
        internal const byte Truncate      = 0x42;

        // Aggregates
        internal const byte CountRows  = 0x50;
        internal const byte SumField   = 0x51;
        internal const byte MinField   = 0x52;
        internal const byte MaxField   = 0x53;
        internal const byte AvgField   = 0x54;
        internal const byte OrField    = 0x79;   // opcode 121

        // Engine
        internal const byte Compact    = 0x60;
        internal const byte Flush      = 0x61;

        // Response base (≥ 0x80)
        internal const byte OkBase     = 0x80;
        internal const byte Error      = 0xFF;
    }

    // ---- Frame builder ----

    /// <summary>
    /// Write a complete frame (header + payload) into a new byte array.
    /// Allocates once; the hot path can stack-allocate via WriteFrameTo.
    /// </summary>
    internal static byte[] BuildFrame(byte opcode, ulong requestId, ReadOnlySpan<byte> payload, byte flags = 0)
    {
        var frame = new byte[HeaderSize + payload.Length];
        WriteHeader(frame.AsSpan(), opcode, requestId, (uint)payload.Length, Crc32(payload), flags);
        payload.CopyTo(frame.AsSpan(HeaderSize));
        return frame;
    }

    internal static void WriteHeader(
        Span<byte> header,
        byte opcode,
        ulong requestId,
        uint payloadLen,
        uint crc32,
        byte flags = 0)
    {
        header[0] = MagicV;
        header[1] = MagicV;
        header[2] = Version;
        header[3] = opcode;
        header[4] = flags;
        header[5] = 0; header[6] = 0; header[7] = 0;   // reserved
        BinaryPrimitives.WriteUInt64LittleEndian(header.Slice(8),  requestId);
        BinaryPrimitives.WriteUInt32LittleEndian(header.Slice(16), payloadLen);
        BinaryPrimitives.WriteUInt32LittleEndian(header.Slice(20), crc32);
    }

    // ---- Frame reader ----

    internal readonly struct ParsedHeader
    {
        internal readonly byte   Opcode;
        internal readonly byte   Flags;
        internal readonly ulong  RequestId;
        internal readonly uint   PayloadLen;
        internal readonly uint   PayloadCrc;

        internal ParsedHeader(byte opcode, byte flags, ulong requestId, uint payloadLen, uint payloadCrc)
        {
            Opcode     = opcode;
            Flags      = flags;
            RequestId  = requestId;
            PayloadLen = payloadLen;
            PayloadCrc = payloadCrc;
        }
    }

    internal static ParsedHeader ReadHeader(IVectorVoltTransport transport)
    {
        Span<byte> buf = stackalloc byte[HeaderSize];
        transport.ReceiveExact(buf);

        if (buf[0] != MagicV || buf[1] != MagicV)
            throw new VectorVoltProtocolException(
                $"Bad magic: 0x{buf[0]:X2} 0x{buf[1]:X2} (expected 0x56 0x56)");

        if (buf[2] != Version)
            throw new VectorVoltProtocolException(
                $"Unsupported protocol version {buf[2]} (expected {Version})");

        return new ParsedHeader(
            opcode:     buf[3],
            flags:      buf[4],
            requestId:  BinaryPrimitives.ReadUInt64LittleEndian(buf.Slice(8)),
            payloadLen: BinaryPrimitives.ReadUInt32LittleEndian(buf.Slice(16)),
            payloadCrc: BinaryPrimitives.ReadUInt32LittleEndian(buf.Slice(20)));
    }

    internal static byte[] ReadPayload(IVectorVoltTransport transport, in ParsedHeader header)
    {
        if (header.PayloadLen == 0)
            return Array.Empty<byte>();

        var payload = new byte[header.PayloadLen];
        transport.ReceiveExact(payload);

        uint actual = Crc32(payload);
        if (actual != header.PayloadCrc)
            throw new VectorVoltProtocolException(
                $"CRC32 mismatch: expected 0x{header.PayloadCrc:X8}, got 0x{actual:X8}");

        return payload;
    }

    // ---- Payload helpers ----

    /// <summary>Encode a UTF-8 string as [u16 length][bytes].</summary>
    internal static void WriteString(ref PayloadWriter w, string s)
    {
        var bytes = Encoding.UTF8.GetBytes(s);
        w.WriteU16((ushort)bytes.Length);
        w.WriteBytes(bytes);
    }

    internal static string ReadString(ref PayloadReader r)
    {
        ushort len = r.ReadU16();
        return Encoding.UTF8.GetString(r.ReadBytes(len));
    }

    // ---- CRC ----

    internal static uint Crc32(ReadOnlySpan<byte> data)
    {
        if (data.IsEmpty) return 0;
        var crc = new Crc32();
        crc.Append(data);
        return BinaryPrimitives.ReadUInt32LittleEndian(crc.GetCurrentHash());
    }
}

// ---- Payload writer / reader (stack-friendly) ----

internal ref struct PayloadWriter
{
    private readonly Span<byte> _buf;
    private int _pos;

    internal PayloadWriter(Span<byte> buf) { _buf = buf; _pos = 0; }

    internal int Position => _pos;

    internal void WriteU8(byte v)  { _buf[_pos++] = v; }
    internal void WriteU16(ushort v) { BinaryPrimitives.WriteUInt16LittleEndian(_buf.Slice(_pos), v); _pos += 2; }
    internal void WriteI32(int v)  { BinaryPrimitives.WriteInt32LittleEndian(_buf.Slice(_pos), v);  _pos += 4; }
    internal void WriteU32(uint v) { BinaryPrimitives.WriteUInt32LittleEndian(_buf.Slice(_pos), v); _pos += 4; }
    internal void WriteI64(long v) { BinaryPrimitives.WriteInt64LittleEndian(_buf.Slice(_pos), v);  _pos += 8; }
    internal void WriteU64(ulong v){ BinaryPrimitives.WriteUInt64LittleEndian(_buf.Slice(_pos), v); _pos += 8; }

    internal void WriteBytes(ReadOnlySpan<byte> data)
    {
        data.CopyTo(_buf.Slice(_pos));
        _pos += data.Length;
    }

    internal ReadOnlySpan<byte> Written => _buf.Slice(0, _pos);
}

internal ref struct PayloadReader
{
    private readonly ReadOnlySpan<byte> _buf;
    private int _pos;

    internal PayloadReader(ReadOnlySpan<byte> buf) { _buf = buf; _pos = 0; }

    internal int Remaining => _buf.Length - _pos;

    internal byte   ReadU8()  => _buf[_pos++];
    internal ushort ReadU16() { var v = BinaryPrimitives.ReadUInt16LittleEndian(_buf.Slice(_pos)); _pos += 2; return v; }
    internal int    ReadI32() { var v = BinaryPrimitives.ReadInt32LittleEndian(_buf.Slice(_pos));  _pos += 4; return v; }
    internal uint   ReadU32() { var v = BinaryPrimitives.ReadUInt32LittleEndian(_buf.Slice(_pos)); _pos += 4; return v; }
    internal long   ReadI64() { var v = BinaryPrimitives.ReadInt64LittleEndian(_buf.Slice(_pos));  _pos += 8; return v; }
    internal ulong  ReadU64() { var v = BinaryPrimitives.ReadUInt64LittleEndian(_buf.Slice(_pos)); _pos += 8; return v; }

    internal ReadOnlySpan<byte> ReadBytes(int count)
    {
        var slice = _buf.Slice(_pos, count);
        _pos += count;
        return slice;
    }
}
