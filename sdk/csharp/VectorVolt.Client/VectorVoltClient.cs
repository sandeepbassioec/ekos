using System.Security.Cryptography;
using System.Text;
using VectorVolt.Client.Protocol;
using VectorVolt.Client.Transport;

namespace VectorVolt.Client;

/// <summary>
/// Official .NET client for VectorVolt.
///
/// <para>The client owns an <see cref="IVectorVoltTransport"/> that it received at
/// construction time. Call <see cref="Dispose"/> to close the transport.</para>
///
/// <para>Thread safety: a single client is NOT thread-safe. Use one client per
/// thread/task, or protect with a lock. Connection pooling is a higher-level
/// concern.</para>
/// </summary>
public sealed class VectorVoltClient : IDisposable
{
    private readonly IVectorVoltTransport _transport;
    private ulong _nextRequestId = 1;
    private bool _disposed;

    /// <summary>
    /// Create a client over an already-connected transport.
    /// After construction, call <see cref="AuthenticateAsync"/> before any other
    /// operation.
    /// </summary>
    public VectorVoltClient(IVectorVoltTransport transport)
    {
        _transport = transport ?? throw new ArgumentNullException(nameof(transport));
    }

    // ---- Convenience factory ----

    /// <summary>
    /// Connect via TCP, authenticate, and return a ready client.
    /// </summary>
    public static VectorVoltClient Connect(
        string host,
        int port,
        string secret,
        int connectTimeoutMs = 5_000)
    {
        var transport = SocketTransport.Connect(host, port, connectTimeoutMs);
        var client = new VectorVoltClient(transport);
        try
        {
            client.Authenticate(secret);
            return client;
        }
        catch
        {
            client.Dispose();
            throw;
        }
    }

    // ---- Auth ----

    /// <summary>
    /// Perform the two-round-trip authentication handshake.
    /// Must be called once after connecting and before any other method.
    /// </summary>
    public void Authenticate(string secret)
    {
        // Round 1: send Hello
        var helloFrame = WireProtocol.BuildFrame(WireProtocol.Op.Hello, NextId(), ReadOnlySpan<byte>.Empty);
        _transport.Send(helloFrame);

        // Round 2: receive Challenge (opcode 0x82, payload = 32-byte nonce)
        var ch = WireProtocol.ReadHeader(_transport);
        var nonce = WireProtocol.ReadPayload(_transport, ch);
        if (nonce.Length != 32)
            throw new VectorVoltProtocolException($"Expected 32-byte nonce, got {nonce.Length}");

        // Compute SHA3-256(secret_bytes || nonce)
        var secretBytes = Encoding.UTF8.GetBytes(secret);
        Span<byte> message = stackalloc byte[secretBytes.Length + 32];
        secretBytes.CopyTo(message);
        nonce.CopyTo(message.Slice(secretBytes.Length));

        Span<byte> digest = stackalloc byte[32];
        SHA3_256.HashData(message, digest);

        // Send Auth frame (opcode 0x03, payload = digest)
        var authFrame = WireProtocol.BuildFrame(WireProtocol.Op.Auth, NextId(), digest);
        _transport.Send(authFrame);

        // Expect Ok (opcode 0x80+0x03 = 0x83 or generic Ok 0x80)
        var ok = WireProtocol.ReadHeader(_transport);
        _ = WireProtocol.ReadPayload(_transport, ok);
        if (ok.Opcode == WireProtocol.Op.Error)
            throw new VectorVoltAuthException("Authentication rejected by server");
    }

    // ---- Control plane ----

    public void CreateTable(string tableName)
    {
        Span<byte> payload = stackalloc byte[2 + Encoding.UTF8.GetByteCount(tableName)];
        var w = new PayloadWriter(payload);
        WireProtocol.WriteString(ref w, tableName);
        SendAndExpectOk(WireProtocol.Op.CreateTable, w.Written);
    }

    public void DropTable(string tableName)
    {
        Span<byte> payload = stackalloc byte[2 + Encoding.UTF8.GetByteCount(tableName)];
        var w = new PayloadWriter(payload);
        WireProtocol.WriteString(ref w, tableName);
        SendAndExpectOk(WireProtocol.Op.DropTable, w.Written);
    }

    public IReadOnlyList<string> ListTables()
    {
        SendFrame(WireProtocol.Op.ListTables, ReadOnlySpan<byte>.Empty);
        var (_, payload) = ReceiveResponse();
        var r = new PayloadReader(payload);
        int count = r.ReadU16();
        var names = new string[count];
        for (int i = 0; i < count; i++)
            names[i] = WireProtocol.ReadString(ref r);
        return names;
    }

    public void UseTable(string tableName)
    {
        Span<byte> payload = stackalloc byte[2 + Encoding.UTF8.GetByteCount(tableName)];
        var w = new PayloadWriter(payload);
        WireProtocol.WriteString(ref w, tableName);
        SendAndExpectOk(WireProtocol.Op.UseTable, w.Written);
    }

    public bool TableExists(string tableName)
    {
        Span<byte> payload = stackalloc byte[2 + Encoding.UTF8.GetByteCount(tableName)];
        var w = new PayloadWriter(payload);
        WireProtocol.WriteString(ref w, tableName);
        SendFrame(WireProtocol.Op.TableExists, w.Written);
        var (header, _) = ReceiveResponse();
        return header.Opcode != WireProtocol.Op.Error;
    }

    public TableStats GetTableStats()
    {
        SendFrame(WireProtocol.Op.TableStats, ReadOnlySpan<byte>.Empty);
        var (_, payload) = ReceiveResponse();
        var r = new PayloadReader(payload);
        return new TableStats
        {
            RowCount       = r.ReadI64(),
            PartitionCount = r.ReadI64(),
            DiskBytes      = r.ReadI64(),
        };
    }

    public TableSchema DescribeTable()
    {
        SendFrame(WireProtocol.Op.DescribeTable, ReadOnlySpan<byte>.Empty);
        var (_, payload) = ReceiveResponse();
        var r = new PayloadReader(payload);
        var tableName = WireProtocol.ReadString(ref r);
        int colCount = r.ReadU16();
        var cols = new ColumnDescriptor[colCount];
        for (int i = 0; i < colCount; i++)
        {
            cols[i] = new ColumnDescriptor
            {
                Index    = r.ReadU16(),
                Name     = WireProtocol.ReadString(ref r),
                TypeName = WireProtocol.ReadString(ref r),
                Nullable = r.ReadU8() != 0,
            };
        }
        return new TableSchema { TableName = tableName, Columns = cols };
    }

    // ---- DML ----

    public void Insert(long primaryKey, IReadOnlyList<byte[]?> columns)
    {
        int size = 8 + 2 + columns.Sum(c => 1 + (c?.Length ?? 0) + (c != null ? 4 : 0));
        var payload = new byte[size];
        var w = new PayloadWriter(payload);
        w.WriteI64(primaryKey);
        w.WriteU16((ushort)columns.Count);
        foreach (var col in columns)
        {
            if (col == null) { w.WriteU8(0); }
            else             { w.WriteU8(1); w.WriteU32((uint)col.Length); w.WriteBytes(col); }
        }
        SendAndExpectOk(WireProtocol.Op.Insert, w.Written);
    }

    public Row? Get(long primaryKey)
    {
        Span<byte> payload = stackalloc byte[8];
        var w = new PayloadWriter(payload);
        w.WriteI64(primaryKey);
        SendFrame(WireProtocol.Op.Get, w.Written);
        var (header, responsePayload) = ReceiveResponse();
        if (header.Opcode == WireProtocol.Op.Error) return null;
        return DecodeRow(responsePayload);
    }

    public IReadOnlyList<Row> Scan(long fromKey, long toKey, int limit = 1000)
    {
        Span<byte> payload = stackalloc byte[8 + 8 + 4];
        var w = new PayloadWriter(payload);
        w.WriteI64(fromKey);
        w.WriteI64(toKey);
        w.WriteI32(limit);
        SendFrame(WireProtocol.Op.Scan, w.Written);
        var (_, responsePayload) = ReceiveResponse();
        var r = new PayloadReader(responsePayload);
        int count = r.ReadI32();
        var rows = new Row[count];
        for (int i = 0; i < count; i++)
            rows[i] = DecodeRowFromReader(ref r);
        return rows;
    }

    public bool Delete(long primaryKey)
    {
        Span<byte> payload = stackalloc byte[8];
        var w = new PayloadWriter(payload);
        w.WriteI64(primaryKey);
        SendFrame(WireProtocol.Op.Delete, w.Written);
        var (header, _) = ReceiveResponse();
        return header.Opcode != WireProtocol.Op.Error;
    }

    public void Truncate()        => SendAndExpectOk(WireProtocol.Op.Truncate, ReadOnlySpan<byte>.Empty);

    // ---- Aggregates ----

    public long CountRows()
    {
        SendFrame(WireProtocol.Op.CountRows, ReadOnlySpan<byte>.Empty);
        var (_, payload) = ReceiveResponse();
        var r = new PayloadReader(payload);
        return r.ReadI64();
    }

    public long SumField(int columnIndex)       => ReadI64Aggregate(WireProtocol.Op.SumField, columnIndex);
    public long AvgField(int columnIndex)       => ReadI64Aggregate(WireProtocol.Op.AvgField, columnIndex);

    public MinMaxResult MinField(int columnIndex) => ReadMinMaxAggregate(WireProtocol.Op.MinField, columnIndex);
    public MinMaxResult MaxField(int columnIndex) => ReadMinMaxAggregate(WireProtocol.Op.MaxField, columnIndex);

    // ---- Engine operations ----

    public void Compact() => SendAndExpectOk(WireProtocol.Op.Compact, ReadOnlySpan<byte>.Empty);
    public void Flush()   => SendAndExpectOk(WireProtocol.Op.Flush,   ReadOnlySpan<byte>.Empty);

    // ---- Batch operations (Option 3 core — full 122-opcode parity is Layer 9) ----

    public BatchResult BatchInsert(IReadOnlyList<(long Key, IReadOnlyList<byte[]?> Columns)> rows)
    {
        int payloadSize = 4 + rows.Sum(r =>
            8 + 2 + r.Columns.Sum(c => 1 + (c != null ? 4 + c.Length : 0)));
        var payload = new byte[payloadSize];
        var w = new PayloadWriter(payload);
        w.WriteI32(rows.Count);
        foreach (var (key, cols) in rows)
        {
            w.WriteI64(key);
            w.WriteU16((ushort)cols.Count);
            foreach (var col in cols)
            {
                if (col == null) { w.WriteU8(0); }
                else             { w.WriteU8(1); w.WriteU32((uint)col.Length); w.WriteBytes(col); }
            }
        }
        SendFrame(WireProtocol.Op.BatchInsert, w.Written);
        var (_, responsePayload) = ReceiveResponse();
        var r2 = new PayloadReader(responsePayload);
        return new BatchResult { RowsAffected = r2.ReadI32(), RowsFailed = r2.ReadI32() };
    }

    public BatchResult BatchDelete(IReadOnlyList<long> keys)
    {
        var payload = new byte[4 + keys.Count * 8];
        var w = new PayloadWriter(payload);
        w.WriteI32(keys.Count);
        foreach (var k in keys) w.WriteI64(k);
        SendFrame(WireProtocol.Op.BatchDelete, w.Written);
        var (_, responsePayload) = ReceiveResponse();
        var r = new PayloadReader(responsePayload);
        return new BatchResult { RowsAffected = r.ReadI32(), RowsFailed = r.ReadI32() };
    }

    // ---- Internals ----

    private void SendFrame(byte opcode, ReadOnlySpan<byte> payload)
    {
        var frame = WireProtocol.BuildFrame(opcode, NextId(), payload);
        _transport.Send(frame);
    }

    private (WireProtocol.ParsedHeader header, byte[] payload) ReceiveResponse()
    {
        var header = WireProtocol.ReadHeader(_transport);
        var payload = WireProtocol.ReadPayload(_transport, header);
        if (header.Opcode == WireProtocol.Op.Error)
        {
            var msg = payload.Length > 0 ? Encoding.UTF8.GetString(payload) : "Unknown server error";
            throw new VectorVoltServerException(WireProtocol.Op.Error, msg);
        }
        return (header, payload);
    }

    private void SendAndExpectOk(byte opcode, ReadOnlySpan<byte> payload)
    {
        SendFrame(opcode, payload);
        ReceiveResponse();
    }

    private static Row DecodeRow(byte[] payload)
    {
        var r = new PayloadReader(payload);
        return DecodeRowFromReader(ref r);
    }

    private static Row DecodeRowFromReader(ref PayloadReader r)
    {
        long pk = r.ReadI64();
        int colCount = r.ReadU16();
        var cols = new byte[]?[colCount];
        for (int i = 0; i < colCount; i++)
        {
            byte present = r.ReadU8();
            if (present == 0) { cols[i] = null; continue; }
            uint len = r.ReadU32();
            cols[i] = r.ReadBytes((int)len).ToArray();
        }
        return new Row { PrimaryKey = pk, Columns = cols };
    }

    private long ReadI64Aggregate(byte opcode, int columnIndex)
    {
        Span<byte> payload = stackalloc byte[4];
        var w = new PayloadWriter(payload);
        w.WriteI32(columnIndex);
        SendFrame(opcode, w.Written);
        var (_, responsePayload) = ReceiveResponse();
        var r = new PayloadReader(responsePayload);
        return r.ReadI64();
    }

    private MinMaxResult ReadMinMaxAggregate(byte opcode, int columnIndex)
    {
        Span<byte> payload = stackalloc byte[4];
        var w = new PayloadWriter(payload);
        w.WriteI32(columnIndex);
        SendFrame(opcode, w.Written);
        var (_, responsePayload) = ReceiveResponse();
        if (responsePayload.Length == 0)
            return new MinMaxResult { HasValue = false };
        return new MinMaxResult { HasValue = true, Value = responsePayload };
    }

    private ulong NextId() => _nextRequestId++;

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        _transport.Dispose();
    }
}
