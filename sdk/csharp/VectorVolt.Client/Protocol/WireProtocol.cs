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
///
/// <para>Wire encoding primitives:</para>
/// <list type="bullet">
///   <item>Integers: little-endian u8/u16/u32/u64.</item>
///   <item>String: u32 byte-length followed by UTF-8 bytes (no null terminator).</item>
///   <item>u64 slice: u32 count followed by count × u64 values.</item>
///   <item>Byte slice: u32 byte-length followed by raw bytes.</item>
/// </list>
/// </summary>
internal static class WireProtocol
{
    internal const int HeaderSize = 24;
    private  const byte MagicV   = (byte)'V';
    private  const byte Version  = 1;

    // ── Request opcodes (1–122) ──────────────────────────────────────────────
    internal static class Op
    {
        // Core CRUD
        internal const byte Ping             = 1;
        internal const byte Get              = 2;
        internal const byte Insert           = 3;
        internal const byte Delete           = 4;
        internal const byte RangeScan        = 5;
        internal const byte Flush            = 6;
        internal const byte Compact          = 7;
        internal const byte Stats            = 8;

        // Auth
        internal const byte Hello            = 9;
        internal const byte Auth             = 10;

        // Secondary index / FK
        internal const byte SecondaryLookup  = 11;
        internal const byte FkExists         = 12;

        // Table DDL
        internal const byte UseTable         = 13;
        internal const byte CreateTable      = 14;
        internal const byte DropTable        = 15;
        internal const byte ListTables       = 16;

        // Batch DML
        internal const byte BatchInsert      = 17;
        internal const byte BatchDelete      = 18;

        // Transactions
        internal const byte BeginTxn         = 19;
        internal const byte CommitTxn        = 20;
        internal const byte RollbackTxn      = 21;

        // More DML
        internal const byte Upsert           = 22;
        internal const byte GetMany          = 23;
        internal const byte Update           = 24;
        internal const byte Truncate         = 25;
        internal const byte Count            = 26;
        internal const byte BatchUpsert      = 27;
        internal const byte BatchUpdate      = 28;

        // Schema
        internal const byte DescribeTable    = 29;
        internal const byte TableExists      = 30;

        // Scans
        internal const byte ScanKeys         = 31;
        internal const byte MinMax           = 32;
        internal const byte Sum              = 33;

        // Atomic ops
        internal const byte GetAndDelete     = 34;
        internal const byte Increment        = 35;
        internal const byte CompareAndSwap   = 36;
        internal const byte SetField         = 37;
        internal const byte InsertIfAbsent   = 38;
        internal const byte DeleteRange      = 39;
        internal const byte PopMin           = 40;
        internal const byte PopMax           = 41;
        internal const byte MultiSet         = 42;
        internal const byte DecrementClamp   = 43;
        internal const byte GetOrInsert      = 44;
        internal const byte BatchSetField    = 45;
        internal const byte Top              = 46;
        internal const byte Bottom           = 47;
        internal const byte PkExists         = 48;
        internal const byte IncrementMulti   = 49;
        internal const byte CopyRow          = 50;
        internal const byte SwapField        = 51;
        internal const byte Clamp            = 52;
        internal const byte Average          = 53;
        internal const byte DeleteIf         = 54;
        internal const byte SetFieldIf       = 55;
        internal const byte RankOf           = 56;
        internal const byte Percentile       = 57;
        internal const byte SumFields        = 58;
        internal const byte MaxField         = 59;
        internal const byte FilterScan       = 60;
        internal const byte BatchIncrField   = 61;
        internal const byte CountDistinct    = 62;
        internal const byte Median           = 63;
        internal const byte BatchDeleteIf    = 64;
        internal const byte BatchPkExists    = 65;
        internal const byte FieldBetween     = 66;
        internal const byte ScalarMin        = 67;
        internal const byte ScalarMax        = 68;
        internal const byte MultiGet         = 69;
        internal const byte IncrOrInsert     = 70;
        internal const byte ZeroField        = 71;
        internal const byte ScanPage         = 72;
        internal const byte SumField         = 73;
        internal const byte BatchGet         = 74;
        internal const byte SetFieldRange    = 75;
        internal const byte MaxRow           = 76;
        internal const byte MinRow           = 77;
        internal const byte IncrementRange   = 78;
        internal const byte DeleteMin        = 79;
        internal const byte DeleteMax        = 80;
        internal const byte UniqueCount      = 81;
        internal const byte TopN             = 82;
        internal const byte BottomN          = 83;
        internal const byte MoveField        = 84;
        internal const byte CountIf          = 85;
        internal const byte SetIfGreater     = 86;
        internal const byte SetIfLess        = 87;
        internal const byte AbsDiff          = 88;
        internal const byte SwapFields       = 89;
        internal const byte CopyRange        = 90;
        internal const byte PropagateMax     = 91;
        internal const byte PropagateMin     = 92;
        internal const byte ClampRange       = 93;
        internal const byte BatchCondSet     = 94;
        internal const byte WindowRank       = 95;
        internal const byte WindowPct        = 96;
        internal const byte SumProduct       = 97;
        internal const byte NthValue         = 98;
        internal const byte DenseRankWin     = 99;
        internal const byte SetFieldMany     = 100;
        internal const byte MultiColumnSum   = 101;
        internal const byte FieldHistogram   = 102;
        internal const byte ZScoreFilter     = 103;
        internal const byte RunningSum       = 104;
        internal const byte FloorDiv         = 105;
        internal const byte MulField         = 106;
        internal const byte SetFieldAdd      = 107;
        internal const byte SetFieldMul      = 108;
        internal const byte SwapPkFields     = 109;
        internal const byte CountRange       = 110;
        internal const byte SumRange         = 111;
        internal const byte MinRange         = 112;
        internal const byte MaxRange         = 113;
        internal const byte AvgRange         = 114;
        internal const byte IncrementField   = 115;
        internal const byte DecrementField   = 116;
        internal const byte SetMin           = 117;
        internal const byte SetMax           = 118;
        internal const byte XorField         = 119;
        internal const byte AndField         = 120;
        internal const byte OrField          = 121;
        internal const byte BatchGetField    = 122;

        // Response opcodes (129–145)
        internal const byte Pong             = 129;
        internal const byte Row              = 130;
        internal const byte NotFound         = 131;
        internal const byte Ok               = 132;
        internal const byte Rows             = 133;
        internal const byte StatsReply       = 134;
        internal const byte Error            = 135;
        internal const byte Challenge        = 136;
        internal const byte Exists           = 137;
        internal const byte TableSelected    = 138;
        internal const byte TableList        = 139;
        internal const byte BatchResult      = 140;
        internal const byte TxnStarted       = 141;
        internal const byte RowSet           = 142;
        internal const byte Schema           = 143;
        internal const byte Keys             = 144;
        internal const byte MinMaxResult     = 145;
    }

    // ── Frame builder ────────────────────────────────────────────────────────

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
        header[5] = 0; header[6] = 0; header[7] = 0;
        BinaryPrimitives.WriteUInt64LittleEndian(header.Slice(8),  requestId);
        BinaryPrimitives.WriteUInt32LittleEndian(header.Slice(16), payloadLen);
        BinaryPrimitives.WriteUInt32LittleEndian(header.Slice(20), crc32);
    }

    // ── Frame reader ─────────────────────────────────────────────────────────

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

    // ── Payload helpers ──────────────────────────────────────────────────────

    /// <summary>Encode a UTF-8 string as [u32 byte-length][bytes].</summary>
    internal static void WriteString(ref PayloadWriter w, string s)
    {
        var bytes = Encoding.UTF8.GetBytes(s);
        w.WriteU32((uint)bytes.Length);
        w.WriteBytes(bytes);
    }

    internal static string ReadString(ref PayloadReader r)
    {
        uint len = r.ReadU32();
        return Encoding.UTF8.GetString(r.ReadBytes((int)len));
    }

    /// <summary>Encode a string list as [u32 count][string …].</summary>
    internal static void WriteStrings(ref PayloadWriter w, IReadOnlyList<string> list)
    {
        w.WriteU32((uint)list.Count);
        foreach (var s in list)
            WriteString(ref w, s);
    }

    internal static string[] ReadStrings(ref PayloadReader r)
    {
        uint count = r.ReadU32();
        var result = new string[count];
        for (int i = 0; i < (int)count; i++)
            result[i] = ReadString(ref r);
        return result;
    }

    /// <summary>Encode a u64 slice as [u32 count][u64 …].</summary>
    internal static void WriteU64s(ref PayloadWriter w, IReadOnlyList<ulong> values)
    {
        w.WriteU32((uint)values.Count);
        foreach (var v in values)
            w.WriteU64(v);
    }

    internal static ulong[] ReadU64s(ref PayloadReader r)
    {
        uint count = r.ReadU32();
        var result = new ulong[count];
        for (int i = 0; i < (int)count; i++)
            result[i] = r.ReadU64();
        return result;
    }

    /// <summary>Encode raw bytes as [u32 length][bytes].</summary>
    internal static void WriteBytes(ref PayloadWriter w, ReadOnlySpan<byte> data)
    {
        w.WriteU32((uint)data.Length);
        w.WriteBytes(data);
    }

    internal static byte[] ReadBytes(ref PayloadReader r)
    {
        uint len = r.ReadU32();
        return r.ReadBytes((int)len).ToArray();
    }

    // ── CRC ──────────────────────────────────────────────────────────────────

    internal static uint Crc32(ReadOnlySpan<byte> data)
    {
        if (data.IsEmpty) return 0;
        var crc = new Crc32();
        crc.Append(data);
        return BinaryPrimitives.ReadUInt32LittleEndian(crc.GetCurrentHash());
    }
}

// ── Payload writer / reader (stack-friendly ref structs) ─────────────────────

internal ref struct PayloadWriter
{
    private readonly Span<byte> _buf;
    private int _pos;

    internal PayloadWriter(Span<byte> buf) { _buf = buf; _pos = 0; }

    internal int Position => _pos;

    internal void WriteU8(byte v)   { _buf[_pos++] = v; }
    internal void WriteU16(ushort v){ BinaryPrimitives.WriteUInt16LittleEndian(_buf.Slice(_pos), v); _pos += 2; }
    internal void WriteI32(int v)   { BinaryPrimitives.WriteInt32LittleEndian(_buf.Slice(_pos),  v); _pos += 4; }
    internal void WriteU32(uint v)  { BinaryPrimitives.WriteUInt32LittleEndian(_buf.Slice(_pos), v); _pos += 4; }
    internal void WriteI64(long v)  { BinaryPrimitives.WriteInt64LittleEndian(_buf.Slice(_pos),  v); _pos += 8; }
    internal void WriteU64(ulong v) { BinaryPrimitives.WriteUInt64LittleEndian(_buf.Slice(_pos), v); _pos += 8; }

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
