using System.Security.Cryptography;
using System.Text;
using VectorVolt.Client.Protocol;
using VectorVolt.Client.Transport;

namespace VectorVolt.Client;

/// <summary>
/// Official .NET client for VectorVolt — full 122-opcode parity with the Rust client.
///
/// <para>All column values are <c>ulong</c> (u64 little-endian on the wire) matching the
/// server's native type. Primary keys are also <c>ulong</c>.</para>
///
/// <para>Thread safety: a single client is NOT thread-safe. Use one client per
/// thread/task, or protect with a lock.</para>
/// </summary>
public sealed class VectorVoltClient : IDisposable
{
    private readonly IVectorVoltTransport _transport;
    private ulong _nextId = 1;
    private bool _disposed;

    /// <summary>Create a client over an already-connected, already-authenticated transport.</summary>
    public VectorVoltClient(IVectorVoltTransport transport)
    {
        _transport = transport ?? throw new ArgumentNullException(nameof(transport));
    }

    // ── Convenience factory ───────────────────────────────────────────────────

    /// <summary>Connect via TCP, authenticate, and return a ready-to-use client.</summary>
    public static VectorVoltClient Connect(
        string host, int port, string keyId, string secret, int connectTimeoutMs = 5_000)
    {
        var transport = SocketTransport.Connect(host, port, connectTimeoutMs);
        var client = new VectorVoltClient(transport);
        try   { client.Authenticate(keyId, secret); return client; }
        catch { client.Dispose(); throw; }
    }

    // ── Auth ─────────────────────────────────────────────────────────────────

    /// <summary>Two-round-trip auth: Hello → Challenge → Auth → Ok.</summary>
    public void Authenticate(string keyId, string secret)
    {
        SendFrame(WireProtocol.Op.Hello, ReadOnlySpan<byte>.Empty);

        var ch = WireProtocol.ReadHeader(_transport);
        var challengePayload = WireProtocol.ReadPayload(_transport, ch);
        if (ch.Opcode != WireProtocol.Op.Challenge)
            throw new VectorVoltAuthException($"Expected Challenge, got opcode {ch.Opcode}");

        var r = new PayloadReader(challengePayload);
        uint nonceLen = r.ReadU32();
        var nonce = r.ReadBytes((int)nonceLen);

        // The server stores the secret as raw bytes (from hex::decode of the --secret CLI arg).
        // Decode hex here to match. If not hex, fall back to UTF-8 so plaintext secrets work too.
        byte[] secretBytes;
        try   { secretBytes = Convert.FromHexString(secret); }
        catch { secretBytes = Encoding.UTF8.GetBytes(secret); }

        var message = new byte[secretBytes.Length + (int)nonceLen];
        secretBytes.CopyTo(message, 0);
        nonce.CopyTo(message.AsSpan(secretBytes.Length));

        // Heap-allocate digest so it can safely be passed into PayloadWriter.WriteBytes
        byte[] digest = new byte[32];
        SHA3_256.HashData(message, digest);

        // Auth payload: [u32 key_id_len][key_id bytes][u32 response_len][response bytes]
        var keyIdBytes = Encoding.UTF8.GetBytes(keyId);
        var authPayload = new byte[4 + keyIdBytes.Length + 4 + 32];
        var w = new PayloadWriter(authPayload);
        w.WriteU32((uint)keyIdBytes.Length);
        w.WriteBytes(keyIdBytes);
        w.WriteU32(32);
        w.WriteBytes(digest);

        SendFrame(WireProtocol.Op.Auth, w.Written);
        var ok = WireProtocol.ReadHeader(_transport);
        _ = WireProtocol.ReadPayload(_transport, ok);
        if (ok.Opcode == WireProtocol.Op.Error)
            throw new VectorVoltAuthException("Authentication rejected by server");
    }

    // ── Ping / Stats ─────────────────────────────────────────────────────────

    public void Ping()
    {
        SendFrame(WireProtocol.Op.Ping, ReadOnlySpan<byte>.Empty);
        var h = WireProtocol.ReadHeader(_transport);
        _ = WireProtocol.ReadPayload(_transport, h);
    }

    /// <summary>Returns (rows, partitions, bytes) for the active table.</summary>
    public TableStats Stats()
    {
        SendFrame(WireProtocol.Op.Stats, ReadOnlySpan<byte>.Empty);
        var payload = Recv();
        var r = new PayloadReader(payload);
        return new TableStats { Rows = r.ReadU64(), Partitions = r.ReadU64(), Bytes = r.ReadU64() };
    }

    // ── Table DDL ────────────────────────────────────────────────────────────

    public void UseTable(string name)         => SendVoidStr(WireProtocol.Op.UseTable, name);
    public void DropTable(string name)        => SendVoidStr(WireProtocol.Op.DropTable, name);
    public void Truncate()                    => SendOk(WireProtocol.Op.Truncate);
    public void Flush()                       => SendOk(WireProtocol.Op.Flush);

    /// <summary>Create a table and select it. <paramref name="schema"/> is the ordered list of column names.</summary>
    public void CreateTable(string name, IReadOnlyList<string> schema)
    {
        var nameBytes  = Encoding.UTF8.GetBytes(name);
        int schemaSize = 4 + schema.Sum(s => 4 + Encoding.UTF8.GetByteCount(s));
        var payload    = new byte[4 + nameBytes.Length + schemaSize];
        var w          = new PayloadWriter(payload);
        WireProtocol.WriteString(ref w, name);
        WireProtocol.WriteStrings(ref w, schema);
        SendOk(WireProtocol.Op.CreateTable, w.Written);
    }

    public string[] ListTables()
    {
        SendFrame(WireProtocol.Op.ListTables, ReadOnlySpan<byte>.Empty);
        var payload = Recv();
        var r = new PayloadReader(payload);
        return WireProtocol.ReadStrings(ref r);
    }

    public bool TableExists(string name)
    {
        var nameBytes = Encoding.UTF8.GetBytes(name);
        var payload   = new byte[4 + nameBytes.Length];
        var w         = new PayloadWriter(payload);
        WireProtocol.WriteString(ref w, name);
        SendFrame(WireProtocol.Op.TableExists, w.Written);
        var h = WireProtocol.ReadHeader(_transport);
        var p = WireProtocol.ReadPayload(_transport, h);
        return h.Opcode == WireProtocol.Op.Exists && p.Length > 0 && p[0] != 0;
    }

    /// <summary>Returns column names for the currently selected table.</summary>
    public string[] DescribeTable()
    {
        SendFrame(WireProtocol.Op.DescribeTable, ReadOnlySpan<byte>.Empty);
        var payload = Recv();
        var r = new PayloadReader(payload);
        return WireProtocol.ReadStrings(ref r);
    }

    // ── Core CRUD ────────────────────────────────────────────────────────────

    public void Insert(ulong pk, IReadOnlyList<ulong> values)
    {
        var payload = AllocU64PkValues(pk, values);
        SendOk(WireProtocol.Op.Insert, payload.Written);
    }

    public void Upsert(ulong pk, IReadOnlyList<ulong> values)
    {
        var payload = AllocU64PkValues(pk, values);
        SendOk(WireProtocol.Op.Upsert, payload.Written);
    }

    public void Update(ulong pk, IReadOnlyList<ulong> values)
    {
        var payload = AllocU64PkValues(pk, values);
        SendOk(WireProtocol.Op.Update, payload.Written);
    }

    /// <summary>Returns <c>null</c> if pk not found.</summary>
    public ulong[]? Get(ulong pk)
    {
        Span<byte> p = stackalloc byte[8];
        BinaryPrimitives(p, pk);
        SendFrame(WireProtocol.Op.Get, p);
        var h = WireProtocol.ReadHeader(_transport);
        var resp = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return null;
        CheckError(h, resp);
        var r = new PayloadReader(resp);
        return WireProtocol.ReadU64s(ref r);
    }

    /// <summary>Returns <c>false</c> if pk not found.</summary>
    public bool Delete(ulong pk)
    {
        Span<byte> p = stackalloc byte[8];
        BinaryPrimitives(p, pk);
        SendFrame(WireProtocol.Op.Delete, p);
        var h = WireProtocol.ReadHeader(_transport);
        var resp = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return false;
        CheckError(h, resp);
        var r = new PayloadReader(resp);
        return r.ReadU64() > 0;
    }

    // ── Scans ────────────────────────────────────────────────────────────────

    /// <summary>Scan rows where column <paramref name="col"/> is in [lo, hi]. limit=0 = unlimited.</summary>
    public Row[] RangeScan(uint col, ulong lo, ulong hi, uint limit = 0)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 4];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(lo); w.WriteU64(hi); w.WriteU32(limit);
        SendFrame(WireProtocol.Op.RangeScan, w.Written);
        return RecvRows();
    }

    public ulong[] ScanKeys(uint col, ulong lo, ulong hi, uint limit = 0)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 4];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(lo); w.WriteU64(hi); w.WriteU32(limit);
        SendFrame(WireProtocol.Op.ScanKeys, w.Written);
        var payload = Recv();
        var r = new PayloadReader(payload);
        uint count = r.ReadU32();
        var keys = new ulong[count];
        for (int i = 0; i < (int)count; i++) keys[i] = r.ReadU64();
        return keys;
    }

    public Row[] ScanPage(uint col, ulong lo, ulong hi, uint offset, uint limit)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 4 + 4];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(lo); w.WriteU64(hi); w.WriteU32(offset); w.WriteU32(limit);
        SendFrame(WireProtocol.Op.ScanPage, w.Written);
        return RecvRows();
    }

    public Row[] FilterScan(uint scanCol, ulong lo, ulong hi, uint filterCol, ulong filterVal)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 4 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(scanCol); w.WriteU64(lo); w.WriteU64(hi); w.WriteU32(filterCol); w.WriteU64(filterVal);
        SendFrame(WireProtocol.Op.FilterScan, w.Written);
        return RecvRows();
    }

    public Row[] FieldBetween(uint scanCol, ulong lo, ulong hi, uint filterCol, ulong filterLo, ulong filterHi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(scanCol); w.WriteU64(lo); w.WriteU64(hi);
        w.WriteU32(filterCol); w.WriteU64(filterLo); w.WriteU64(filterHi);
        SendFrame(WireProtocol.Op.FieldBetween, w.Written);
        return RecvRows();
    }

    public Row[] Top(uint col, ulong lo, ulong hi, uint n)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 4];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(lo); w.WriteU64(hi); w.WriteU32(n);
        SendFrame(WireProtocol.Op.Top, w.Written);
        return RecvRows();
    }

    public Row[] Bottom(uint col, ulong lo, ulong hi, uint n)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 4];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(lo); w.WriteU64(hi); w.WriteU32(n);
        SendFrame(WireProtocol.Op.Bottom, w.Written);
        return RecvRows();
    }

    public Row[] TopN(uint col, uint n, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU32(n); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.TopN, w.Written);
        return RecvRows();
    }

    public Row[] BottomN(uint col, uint n, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU32(n); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.BottomN, w.Written);
        return RecvRows();
    }

    public Row[] SecondaryLookup(uint colIdx, ulong value)
    {
        Span<byte> p = stackalloc byte[4 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(colIdx); w.WriteU64(value);
        SendFrame(WireProtocol.Op.SecondaryLookup, w.Written);
        return RecvRows();
    }

    public Row[] ZScoreFilter(uint col, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8];
        SendColLoHi(WireProtocol.Op.ZScoreFilter, col, lo, hi);
        return RecvRows();
    }

    public Row[] RunningSum(uint col, ulong lo, ulong hi)
    {
        SendColLoHi(WireProtocol.Op.RunningSum, col, lo, hi);
        return RecvRows();
    }

    public Row[] FieldHistogram(uint col, ulong bucketSize, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(bucketSize); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.FieldHistogram, w.Written);
        return RecvRows();
    }

    // ── Aggregates ───────────────────────────────────────────────────────────

    public ulong Count(uint col, ulong lo, ulong hi)    => SendColLoHiU64(WireProtocol.Op.Count, col, lo, hi);
    public ulong Sum(uint col, ulong lo, ulong hi)      => SendColLoHiU64(WireProtocol.Op.Sum, col, lo, hi);
    public ulong CountDistinct(uint col, ulong lo, ulong hi) => SendColLoHiU64(WireProtocol.Op.CountDistinct, col, lo, hi);
    public ulong CountRange(uint col, ulong lo, ulong hi)   => SendColLoHiU64(WireProtocol.Op.CountRange, col, lo, hi);
    public ulong SumRange(uint col, ulong lo, ulong hi)     => SendColLoHiU64(WireProtocol.Op.SumRange, col, lo, hi);
    public ulong SumField(uint col, ulong lo, ulong hi)     => SendColLoHiU64(WireProtocol.Op.SumField, col, lo, hi);
    public ulong UniqueCount(uint col, ulong lo, ulong hi)  => SendColLoHiU64(WireProtocol.Op.UniqueCount, col, lo, hi);
    public ulong ZeroField(uint col, ulong lo, ulong hi)    => SendColLoHiU64(WireProtocol.Op.ZeroField, col, lo, hi);
    public ulong PropagateMax(uint col, ulong lo, ulong hi) => SendColLoHiU64(WireProtocol.Op.PropagateMax, col, lo, hi);
    public ulong PropagateMin(uint col, ulong lo, ulong hi) => SendColLoHiU64(WireProtocol.Op.PropagateMin, col, lo, hi);

    public ulong? ScalarMin(uint col, ulong lo, ulong hi) => SendColLoHiOptional(WireProtocol.Op.ScalarMin, col, lo, hi);
    public ulong? ScalarMax(uint col, ulong lo, ulong hi) => SendColLoHiOptional(WireProtocol.Op.ScalarMax, col, lo, hi);
    public ulong? Median(uint col, ulong lo, ulong hi)    => SendColLoHiOptional(WireProtocol.Op.Median, col, lo, hi);
    public ulong? MinRange(uint col, ulong lo, ulong hi)  => SendColLoHiOptional(WireProtocol.Op.MinRange, col, lo, hi);
    public ulong? MaxRange(uint col, ulong lo, ulong hi)  => SendColLoHiOptional(WireProtocol.Op.MaxRange, col, lo, hi);
    public ulong? AvgRange(uint col, ulong lo, ulong hi)  => SendColLoHiOptional(WireProtocol.Op.AvgRange, col, lo, hi);
    public ulong? DeleteMin(uint col, ulong lo, ulong hi) => SendColLoHiOptional(WireProtocol.Op.DeleteMin, col, lo, hi);
    public ulong? DeleteMax(uint col, ulong lo, ulong hi) => SendColLoHiOptional(WireProtocol.Op.DeleteMax, col, lo, hi);

    public MinMaxStats? MinMax(uint col, ulong lo, ulong hi)
    {
        SendColLoHi(WireProtocol.Op.MinMax, col, lo, hi);
        var h = WireProtocol.ReadHeader(_transport);
        var resp = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return null;
        CheckError(h, resp);
        var r = new PayloadReader(resp);
        return new MinMaxStats { Min = r.ReadU64(), Max = r.ReadU64(), Count = r.ReadU64() };
    }

    /// <summary>Returns (sum, count) or null if no rows match.</summary>
    public (ulong Sum, ulong Count)? Average(uint col, ulong lo, ulong hi)
    {
        SendColLoHi(WireProtocol.Op.Average, col, lo, hi);
        var h = WireProtocol.ReadHeader(_transport);
        var resp = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return null;
        CheckError(h, resp);
        var r = new PayloadReader(resp);
        var vals = WireProtocol.ReadU64s(ref r);
        return (vals[0], vals[1]);
    }

    public ulong? Percentile(uint col, ulong lo, ulong hi, uint pct)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 4];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(lo); w.WriteU64(hi); w.WriteU32(pct);
        SendFrame(WireProtocol.Op.Percentile, w.Written);
        return RecvOptionalU64();
    }

    public ulong? NthValue(uint col, uint n, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU32(n); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.NthValue, w.Written);
        return RecvOptionalU64();
    }

    public ulong SumProduct(uint colA, uint colB, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(colA); w.WriteU32(colB); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.SumProduct, w.Written);
        return RecvU64();
    }

    public ulong CountIf(uint col, ulong value, ulong lo, ulong hi)   => SendColValLoHiU64(WireProtocol.Op.CountIf,     col, value, lo, hi);
    public ulong SetIfGreater(uint col, ulong value, ulong lo, ulong hi) => SendColValLoHiU64(WireProtocol.Op.SetIfGreater, col, value, lo, hi);
    public ulong SetIfLess(uint col, ulong value, ulong lo, ulong hi)  => SendColValLoHiU64(WireProtocol.Op.SetIfLess,    col, value, lo, hi);
    public ulong SetFieldRange(uint col, ulong value, ulong lo, ulong hi) => SendColValLoHiU64(WireProtocol.Op.SetFieldRange, col, value, lo, hi);
    public ulong FloorDiv(uint col, ulong divisor, ulong lo, ulong hi) => SendColValLoHiU64(WireProtocol.Op.FloorDiv,    col, divisor, lo, hi);
    public ulong MulField(uint col, ulong factor, ulong lo, ulong hi)  => SendColValLoHiU64(WireProtocol.Op.MulField,    col, factor, lo, hi);

    public ulong XorField(uint col, ulong mask, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(mask); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.XorField, w.Written);
        return RecvU64();
    }

    public ulong AndField(uint col, ulong mask, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(mask); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.AndField, w.Written);
        return RecvU64();
    }

    public ulong OrField(uint col, ulong mask, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(mask); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.OrField, w.Written);
        return RecvU64();
    }

    // ── Atomic single-row ops ────────────────────────────────────────────────

    public bool PkExists(ulong pk)
    {
        Span<byte> p = stackalloc byte[8];
        new PayloadWriter(p).WriteU64(pk);
        SendFrame(WireProtocol.Op.PkExists, p);
        return RecvExists();
    }

    public bool FkExists(ulong value)
    {
        Span<byte> p = stackalloc byte[8];
        new PayloadWriter(p).WriteU64(value);
        SendFrame(WireProtocol.Op.FkExists, p);
        return RecvExists();
    }

    public ulong[]? GetAndDelete(ulong pk)
    {
        Span<byte> p = stackalloc byte[8];
        new PayloadWriter(p).WriteU64(pk);
        SendFrame(WireProtocol.Op.GetAndDelete, p);
        return RecvOptionalRow();
    }

    public bool InsertIfAbsent(ulong pk, IReadOnlyList<ulong> values)
    {
        var payload = AllocU64PkValues(pk, values);
        SendFrame(WireProtocol.Op.InsertIfAbsent, payload.Written);
        return RecvU64() == 1;
    }

    public ulong[]? GetOrInsert(ulong pk, IReadOnlyList<ulong> values)
    {
        var payload = AllocU64PkValues(pk, values);
        SendFrame(WireProtocol.Op.GetOrInsert, payload.Written);
        return RecvOptionalRow();
    }

    public ulong[]? IncrementMulti(ulong pk, IReadOnlyList<(uint Col, ulong Delta)> pairs)
    {
        var payload = new byte[8 + 4 + pairs.Count * (4 + 8)];
        var w = new PayloadWriter(payload);
        w.WriteU64(pk);
        w.WriteU32((uint)pairs.Count);
        foreach (var (col, delta) in pairs) { w.WriteU32(col); w.WriteU64(delta); }
        SendFrame(WireProtocol.Op.IncrementMulti, w.Written);
        return RecvOptionalRow();
    }

    public ulong[]? MultiGet(ulong pk, IReadOnlyList<uint> cols)
    {
        var payload = new byte[8 + 4 + cols.Count * 4];
        var w = new PayloadWriter(payload);
        w.WriteU64(pk); w.WriteU32((uint)cols.Count);
        foreach (var col in cols) w.WriteU32(col);
        SendFrame(WireProtocol.Op.MultiGet, w.Written);
        return RecvOptionalRow();
    }

    public bool MultiSet(ulong pk, IReadOnlyList<(uint Col, ulong Value)> pairs)
    {
        var payload = new byte[8 + 4 + pairs.Count * (4 + 8)];
        var w = new PayloadWriter(payload);
        w.WriteU64(pk); w.WriteU32((uint)pairs.Count);
        foreach (var (col, val) in pairs) { w.WriteU32(col); w.WriteU64(val); }
        SendFrame(WireProtocol.Op.MultiSet, w.Written);
        var h = WireProtocol.ReadHeader(_transport);
        var resp = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return false;
        CheckError(h, resp);
        return true;
    }

    public ulong[]? SumFields(ulong pk, IReadOnlyList<uint> cols)   => SendPkCols(WireProtocol.Op.SumFields, pk, cols);
    public ulong[]? MaxFieldCols(ulong pk, IReadOnlyList<uint> cols) => SendPkCols(WireProtocol.Op.MaxField,  pk, cols);

    public ulong? Increment(ulong pk, uint col, ulong delta) => SendPkColVal(WireProtocol.Op.Increment, pk, col, delta);
    public ulong? DecrementClamp(ulong pk, uint col, ulong delta) => SendPkColVal(WireProtocol.Op.DecrementClamp, pk, col, delta);
    public ulong? SetFieldAdd(ulong pk, uint col, ulong addend) => SendPkColVal(WireProtocol.Op.SetFieldAdd, pk, col, addend);
    public ulong? SetFieldMul(ulong pk, uint col, ulong factor) => SendPkColVal(WireProtocol.Op.SetFieldMul, pk, col, factor);
    public ulong? IncrementField(ulong pk, uint col)  => SendPkColOpt(WireProtocol.Op.IncrementField, pk, col);
    public ulong? DecrementField(ulong pk, uint col)  => SendPkColOpt(WireProtocol.Op.DecrementField, pk, col);
    public ulong? SetMin(ulong pk, uint col, ulong value) => SendPkColVal(WireProtocol.Op.SetMin, pk, col, value);
    public ulong? SetMax(ulong pk, uint col, ulong value) => SendPkColVal(WireProtocol.Op.SetMax, pk, col, value);

    public bool SetField(ulong pk, uint col, ulong value)
    {
        Span<byte> p = stackalloc byte[8 + 4 + 8];
        var w = new PayloadWriter(p);
        w.WriteU64(pk); w.WriteU32(col); w.WriteU64(value);
        SendFrame(WireProtocol.Op.SetField, w.Written);
        var h = WireProtocol.ReadHeader(_transport);
        var resp = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return false;
        CheckError(h, resp); return true;
    }

    public bool? SetFieldIf(ulong pk, uint col, ulong expected, ulong newValue)
    {
        Span<byte> p = stackalloc byte[8 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU64(pk); w.WriteU32(col); w.WriteU64(expected); w.WriteU64(newValue);
        SendFrame(WireProtocol.Op.SetFieldIf, w.Written);
        return RecvOptionalBool();
    }

    public bool? CompareAndSwap(ulong pk, uint col, ulong expected, ulong replacement)
    {
        Span<byte> p = stackalloc byte[8 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU64(pk); w.WriteU32(col); w.WriteU64(expected); w.WriteU64(replacement);
        SendFrame(WireProtocol.Op.CompareAndSwap, w.Written);
        return RecvOptionalBool();
    }

    public bool? DeleteIf(ulong pk, uint col, ulong value)
    {
        Span<byte> p = stackalloc byte[8 + 4 + 8];
        var w = new PayloadWriter(p);
        w.WriteU64(pk); w.WriteU32(col); w.WriteU64(value);
        SendFrame(WireProtocol.Op.DeleteIf, w.Written);
        return RecvOptionalBool();
    }

    public ulong? RankOf(ulong pk, uint col, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[8 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU64(pk); w.WriteU32(col); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.RankOf, w.Written);
        return RecvOptionalU64();
    }

    public ulong? Clamp(ulong pk, uint col, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[8 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU64(pk); w.WriteU32(col); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.Clamp, w.Written);
        return RecvOptionalU64();
    }

    public ulong? WindowRank(ulong pk, uint col, ulong lo, ulong hi)  => SendPkColLoHiOpt(WireProtocol.Op.WindowRank,   pk, col, lo, hi);
    public ulong? WindowPct(ulong pk, uint col, ulong lo, ulong hi)   => SendPkColLoHiOpt(WireProtocol.Op.WindowPct,    pk, col, lo, hi);
    public ulong? DenseRankWin(ulong pk, uint col, ulong lo, ulong hi) => SendPkColLoHiOpt(WireProtocol.Op.DenseRankWin, pk, col, lo, hi);

    public ulong? PopMin(uint col, ulong lo, ulong hi)
    {
        SendColLoHi(WireProtocol.Op.PopMin, col, lo, hi);
        return RecvOptionalRow()?.FirstOrDefault();
    }

    public ulong? PopMax(uint col, ulong lo, ulong hi)
    {
        SendColLoHi(WireProtocol.Op.PopMax, col, lo, hi);
        return RecvOptionalRow()?.FirstOrDefault();
    }

    public Row? MaxRow(uint col, ulong lo, ulong hi)
    {
        SendColLoHi(WireProtocol.Op.MaxRow, col, lo, hi);
        var vals = RecvOptionalRow();
        return vals == null ? null : new Row { Values = vals };
    }

    public Row? MinRow(uint col, ulong lo, ulong hi)
    {
        SendColLoHi(WireProtocol.Op.MinRow, col, lo, hi);
        var vals = RecvOptionalRow();
        return vals == null ? null : new Row { Values = vals };
    }

    public ulong IncrementOrInsert(ulong pk, uint col, ulong delta, IReadOnlyList<ulong> values)
    {
        var payload = new byte[8 + 4 + 8 + 4 + values.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteU64(pk); w.WriteU32(col); w.WriteU64(delta);
        WireProtocol.WriteU64s(ref w, values);
        SendFrame(WireProtocol.Op.IncrOrInsert, w.Written);
        return RecvU64();
    }

    public ulong? SwapPkFields(ulong pk, uint colA, uint colB)
    {
        Span<byte> p = stackalloc byte[8 + 4 + 4];
        var w = new PayloadWriter(p);
        w.WriteU64(pk); w.WriteU32(colA); w.WriteU32(colB);
        SendFrame(WireProtocol.Op.SwapPkFields, w.Written);
        return RecvOptionalU64();
    }

    // ── Bulk range ops ───────────────────────────────────────────────────────

    public ulong DeleteRange(uint col, ulong lo, ulong hi) => SendColLoHiU64(WireProtocol.Op.DeleteRange, col, lo, hi);
    public ulong IncrementRange(uint col, ulong delta, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(delta); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.IncrementRange, w.Written);
        return RecvU64();
    }

    public ulong ClampRange(uint col, ulong minVal, ulong maxVal, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(minVal); w.WriteU64(maxVal); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.ClampRange, w.Written);
        return RecvU64();
    }

    public ulong BatchConditionalSet(uint col, ulong expected, ulong newValue, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(expected); w.WriteU64(newValue); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.BatchCondSet, w.Written);
        return RecvU64();
    }

    public ulong CopyRange(uint srcCol, uint dstCol, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(srcCol); w.WriteU32(dstCol); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(WireProtocol.Op.CopyRange, w.Written);
        return RecvU64();
    }

    public ulong? SumFieldsSingle(ulong pk, IReadOnlyList<uint> cols)
    {
        var payload = new byte[8 + 4 + cols.Count * 4];
        var w = new PayloadWriter(payload);
        w.WriteU64(pk); w.WriteU32((uint)cols.Count);
        foreach (var col in cols) w.WriteU32(col);
        SendFrame(WireProtocol.Op.SumFields, w.Written);
        return RecvOptionalU64();
    }

    // ── Batch ops ────────────────────────────────────────────────────────────

    public BatchResult BatchInsert(IReadOnlyList<(ulong Pk, ulong[] Values)> rows)
    {
        var payload = BuildRowsPayload(rows);
        SendFrame(WireProtocol.Op.BatchInsert, payload.Written);
        return RecvBatchResult();
    }

    public BatchResult BatchDelete(IReadOnlyList<ulong> pks)
    {
        var payload = new byte[4 + pks.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteU32((uint)pks.Count);
        foreach (var pk in pks) w.WriteU64(pk);
        SendFrame(WireProtocol.Op.BatchDelete, w.Written);
        return RecvBatchResult();
    }

    public ulong BatchUpsert(IReadOnlyList<(ulong Pk, ulong[] Values)> rows)
    {
        var payload = BuildRowsPayload(rows);
        SendFrame(WireProtocol.Op.BatchUpsert, payload.Written);
        return RecvU64();
    }

    public BatchResult BatchUpdate(IReadOnlyList<(ulong Pk, ulong[] Values)> rows)
    {
        var payload = BuildRowsPayload(rows);
        SendFrame(WireProtocol.Op.BatchUpdate, payload.Written);
        return RecvBatchResult();
    }

    public Row[] GetMany(IReadOnlyList<ulong> pks)
    {
        var payload = new byte[4 + pks.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteU32((uint)pks.Count);
        foreach (var pk in pks) w.WriteU64(pk);
        SendFrame(WireProtocol.Op.GetMany, w.Written);
        var responsePayload = Recv();
        var r = new PayloadReader(responsePayload);
        uint count = r.ReadU32();
        var rows = new Row[count];
        for (int i = 0; i < (int)count; i++)
        {
            ulong pk = r.ReadU64();
            rows[i] = new Row { Pk = pk, Values = WireProtocol.ReadU64s(ref r) };
        }
        return rows;
    }

    public Row[] BatchGet(IReadOnlyList<ulong> pks)
    {
        var payload = new byte[4 + pks.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteU32((uint)pks.Count);
        foreach (var pk in pks) w.WriteU64(pk);
        SendFrame(WireProtocol.Op.BatchGet, w.Written);
        return RecvRows();
    }

    public ulong BatchSetField(uint col, ulong value, IReadOnlyList<ulong> pks)
    {
        var payload = new byte[4 + 8 + 4 + pks.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteU32(col); w.WriteU64(value); w.WriteU32((uint)pks.Count);
        foreach (var pk in pks) w.WriteU64(pk);
        SendFrame(WireProtocol.Op.BatchSetField, w.Written);
        return RecvU64();
    }

    public ulong SetFieldMany(uint col, ulong value, IReadOnlyList<ulong> pks)
    {
        var payload = new byte[4 + 8 + 4 + pks.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteU32(col); w.WriteU64(value); w.WriteU32((uint)pks.Count);
        foreach (var pk in pks) w.WriteU64(pk);
        SendFrame(WireProtocol.Op.SetFieldMany, w.Written);
        return RecvU64();
    }

    public ulong BatchIncrementField(uint col, ulong delta, IReadOnlyList<ulong> pks)
    {
        var payload = new byte[4 + 8 + 4 + pks.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteU32(col); w.WriteU64(delta); w.WriteU32((uint)pks.Count);
        foreach (var pk in pks) w.WriteU64(pk);
        SendFrame(WireProtocol.Op.BatchIncrField, w.Written);
        return RecvU64();
    }

    public ulong BatchDeleteIf(uint col, ulong value, IReadOnlyList<ulong> pks)
    {
        var payload = new byte[4 + 8 + 4 + pks.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteU32(col); w.WriteU64(value); w.WriteU32((uint)pks.Count);
        foreach (var pk in pks) w.WriteU64(pk);
        SendFrame(WireProtocol.Op.BatchDeleteIf, w.Written);
        return RecvU64();
    }

    public ulong BatchPkExists(IReadOnlyList<ulong> pks)
    {
        var payload = new byte[4 + pks.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteU32((uint)pks.Count);
        foreach (var pk in pks) w.WriteU64(pk);
        SendFrame(WireProtocol.Op.BatchPkExists, w.Written);
        return RecvU64();
    }

    public Row[] BatchGetField(uint col, IReadOnlyList<ulong> pks)
    {
        var payload = new byte[4 + 4 + pks.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteU32(col); w.WriteU32((uint)pks.Count);
        foreach (var pk in pks) w.WriteU64(pk);
        SendFrame(WireProtocol.Op.BatchGetField, w.Written);
        return RecvRows();
    }

    public ulong MultiColumnSum(IReadOnlyList<ulong> pks, uint col)
    {
        var payload = new byte[4 + pks.Count * 8 + 4];
        var w = new PayloadWriter(payload);
        w.WriteU32((uint)pks.Count);
        foreach (var pk in pks) w.WriteU64(pk);
        w.WriteU32(col);
        SendFrame(WireProtocol.Op.MultiColumnSum, w.Written);
        return RecvU64();
    }

    // ── Row-level ops ────────────────────────────────────────────────────────

    public bool CopyRow(ulong srcPk, ulong dstPk)
    {
        Span<byte> p = stackalloc byte[8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU64(srcPk); w.WriteU64(dstPk);
        SendFrame(WireProtocol.Op.CopyRow, w.Written);
        var h = WireProtocol.ReadHeader(_transport);
        var resp = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return false;
        CheckError(h, resp); return true;
    }

    public bool SwapField(ulong pk1, ulong pk2, uint col)
    {
        Span<byte> p = stackalloc byte[8 + 8 + 4];
        var w = new PayloadWriter(p);
        w.WriteU64(pk1); w.WriteU64(pk2); w.WriteU32(col);
        SendFrame(WireProtocol.Op.SwapField, w.Written);
        var h = WireProtocol.ReadHeader(_transport);
        var resp = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return false;
        CheckError(h, resp); return true;
    }

    public ulong? AbsDiff(ulong pk1, ulong pk2, uint col)
    {
        Span<byte> p = stackalloc byte[8 + 8 + 4];
        var w = new PayloadWriter(p);
        w.WriteU64(pk1); w.WriteU64(pk2); w.WriteU32(col);
        SendFrame(WireProtocol.Op.AbsDiff, w.Written);
        return RecvOptionalU64();
    }

    public ulong? SwapFields(ulong pk1, ulong pk2, uint col)
    {
        Span<byte> p = stackalloc byte[8 + 8 + 4];
        var w = new PayloadWriter(p);
        w.WriteU64(pk1); w.WriteU64(pk2); w.WriteU32(col);
        SendFrame(WireProtocol.Op.SwapFields, w.Written);
        return RecvOptionalU64();
    }

    public ulong? MoveField(ulong srcPk, ulong dstPk, uint col)
    {
        Span<byte> p = stackalloc byte[8 + 8 + 4];
        var w = new PayloadWriter(p);
        w.WriteU64(srcPk); w.WriteU64(dstPk); w.WriteU32(col);
        SendFrame(WireProtocol.Op.MoveField, w.Written);
        return RecvOptionalU64();
    }

    // ── Transactions ─────────────────────────────────────────────────────────

    public ulong BeginTxn()
    {
        SendFrame(WireProtocol.Op.BeginTxn, ReadOnlySpan<byte>.Empty);
        var payload = Recv();
        var r = new PayloadReader(payload);
        return r.ReadU64();
    }

    public ulong CommitTxn(ulong txnId)
    {
        Span<byte> p = stackalloc byte[8];
        new PayloadWriter(p).WriteU64(txnId);
        SendFrame(WireProtocol.Op.CommitTxn, p);
        return RecvU64();
    }

    public void RollbackTxn(ulong txnId)
    {
        Span<byte> p = stackalloc byte[8];
        new PayloadWriter(p).WriteU64(txnId);
        SendOk(WireProtocol.Op.RollbackTxn, p);
    }

    // ── Engine ops ───────────────────────────────────────────────────────────

    public ulong Compact()
    {
        SendFrame(WireProtocol.Op.Compact, ReadOnlySpan<byte>.Empty);
        return RecvU64();
    }

    // ── Internals ────────────────────────────────────────────────────────────

    private void SendFrame(byte opcode, ReadOnlySpan<byte> payload)
    {
        var frame = WireProtocol.BuildFrame(opcode, _nextId++, payload);
        _transport.Send(frame);
    }

    private byte[] Recv()
    {
        var h = WireProtocol.ReadHeader(_transport);
        var p = WireProtocol.ReadPayload(_transport, h);
        CheckError(h, p);
        return p;
    }

    private void SendOk(byte opcode, ReadOnlySpan<byte> payload = default)
    {
        SendFrame(opcode, payload);
        _ = Recv();
    }

    private void SendVoidStr(byte opcode, string name)
    {
        var nb = Encoding.UTF8.GetBytes(name);
        var buf = new byte[4 + nb.Length];
        var w = new PayloadWriter(buf);
        WireProtocol.WriteString(ref w, name);
        SendOk(opcode, w.Written);
    }

    private static void CheckError(in WireProtocol.ParsedHeader h, byte[] payload)
    {
        if (h.Opcode != WireProtocol.Op.Error) return;
        var r = new PayloadReader(payload);
        _ = r.ReadU16(); // error code
        uint msgLen = r.ReadU32();
        var msg = Encoding.UTF8.GetString(r.ReadBytes((int)msgLen));
        throw new VectorVoltServerException(h.Opcode, msg);
    }

    private ulong RecvU64()
    {
        var p = Recv();
        var r = new PayloadReader(p);
        return r.ReadU64();
    }

    private ulong? RecvOptionalU64()
    {
        var h = WireProtocol.ReadHeader(_transport);
        var p = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return null;
        CheckError(h, p);
        var r = new PayloadReader(p);
        return r.ReadU64();
    }

    private ulong[]? RecvOptionalRow()
    {
        var h = WireProtocol.ReadHeader(_transport);
        var p = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return null;
        CheckError(h, p);
        var r = new PayloadReader(p);
        return WireProtocol.ReadU64s(ref r);
    }

    private bool? RecvOptionalBool()
    {
        var h = WireProtocol.ReadHeader(_transport);
        var p = WireProtocol.ReadPayload(_transport, h);
        if (h.Opcode == WireProtocol.Op.NotFound) return null;
        CheckError(h, p);
        var r = new PayloadReader(p);
        return r.ReadU64() == 1;
    }

    private bool RecvExists()
    {
        var h = WireProtocol.ReadHeader(_transport);
        var p = WireProtocol.ReadPayload(_transport, h);
        CheckError(h, p);
        return p.Length > 0 && p[0] != 0;
    }

    private Row[] RecvRows()
    {
        var p = Recv();
        var r = new PayloadReader(p);
        uint count = r.ReadU32();
        var rows = new Row[count];
        for (int i = 0; i < (int)count; i++)
        {
            var vals = WireProtocol.ReadU64s(ref r);
            rows[i] = new Row { Pk = vals.Length > 0 ? vals[0] : 0UL, Values = vals };
        }
        return rows;
    }

    private BatchResult RecvBatchResult()
    {
        var p = Recv();
        var r = new PayloadReader(p);
        return new BatchResult { Succeeded = r.ReadU32(), Skipped = r.ReadU32() };
    }

    // Small helpers to cut repetition in the hot opcode paths
    private static PayloadWriter AllocU64PkValues(ulong pk, IReadOnlyList<ulong> values)
    {
        // Wire protocol: pk | count | pk | val0 | val1 | ...
        // pk is repeated as values[0] to match engine's schema (which includes pk column).
        var buf = new byte[8 + 4 + (1 + values.Count) * 8];
        var w = new PayloadWriter(buf);
        w.WriteU64(pk);
        w.WriteU32((uint)(1 + values.Count));
        w.WriteU64(pk);
        foreach (var v in values) w.WriteU64(v);
        return w;
    }

    private static PayloadWriter BuildRowsPayload(IReadOnlyList<(ulong Pk, ulong[] Values)> rows)
    {
        // Each row: pk | count | pk | val0 | val1 | ...
        // pk repeated as values[0] to match engine schema (pk column included).
        int size = 4 + rows.Sum(row => 8 + 4 + (1 + row.Values.Length) * 8);
        var payload = new byte[size];
        var w = new PayloadWriter(payload);
        w.WriteU32((uint)rows.Count);
        foreach (var (pk, values) in rows)
        {
            w.WriteU64(pk);
            w.WriteU32((uint)(1 + values.Length));
            w.WriteU64(pk);
            foreach (var v in values) w.WriteU64(v);
        }
        return w;
    }

    private void SendColLoHi(byte opcode, uint col, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(opcode, w.Written);
    }

    private ulong SendColLoHiU64(byte opcode, uint col, ulong lo, ulong hi)
    {
        SendColLoHi(opcode, col, lo, hi);
        return RecvU64();
    }

    private ulong? SendColLoHiOptional(byte opcode, uint col, ulong lo, ulong hi)
    {
        SendColLoHi(opcode, col, lo, hi);
        return RecvOptionalU64();
    }

    private ulong SendColValLoHiU64(byte opcode, uint col, ulong value, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[4 + 8 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU32(col); w.WriteU64(value); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(opcode, w.Written);
        return RecvU64();
    }

    private ulong? SendPkColVal(byte opcode, ulong pk, uint col, ulong value)
    {
        Span<byte> p = stackalloc byte[8 + 4 + 8];
        var w = new PayloadWriter(p);
        w.WriteU64(pk); w.WriteU32(col); w.WriteU64(value);
        SendFrame(opcode, w.Written);
        return RecvOptionalU64();
    }

    private ulong? SendPkColOpt(byte opcode, ulong pk, uint col)
    {
        Span<byte> p = stackalloc byte[8 + 4];
        var w = new PayloadWriter(p);
        w.WriteU64(pk); w.WriteU32(col);
        SendFrame(opcode, w.Written);
        return RecvOptionalU64();
    }

    private ulong? SendPkColLoHiOpt(byte opcode, ulong pk, uint col, ulong lo, ulong hi)
    {
        Span<byte> p = stackalloc byte[8 + 4 + 8 + 8];
        var w = new PayloadWriter(p);
        w.WriteU64(pk); w.WriteU32(col); w.WriteU64(lo); w.WriteU64(hi);
        SendFrame(opcode, w.Written);
        return RecvOptionalU64();
    }

    private ulong[]? SendPkCols(byte opcode, ulong pk, IReadOnlyList<uint> cols)
    {
        var payload = new byte[8 + 4 + cols.Count * 4];
        var w = new PayloadWriter(payload);
        w.WriteU64(pk); w.WriteU32((uint)cols.Count);
        foreach (var col in cols) w.WriteU32(col);
        SendFrame(opcode, w.Written);
        return RecvOptionalRow();
    }

    // Helper: write u64 pk into a span (avoids PayloadWriter allocation for single-field sends)
    private static void BinaryPrimitives(Span<byte> span, ulong value)
        => System.Buffers.Binary.BinaryPrimitives.WriteUInt64LittleEndian(span, value);

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        _transport.Dispose();
    }
}

