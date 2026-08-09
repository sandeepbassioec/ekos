namespace VectorVolt.Client;

/// <summary>A single row. All column values are u64 matching the server's native type.</summary>
public sealed class Row
{
    public ulong   Pk     { get; init; }
    public ulong[] Values { get; init; } = Array.Empty<ulong>();
}

/// <summary>Statistics returned by <c>Stats()</c>.</summary>
public sealed class TableStats
{
    public ulong Rows       { get; init; }
    public ulong Partitions { get; init; }
    public ulong Bytes      { get; init; }
}

/// <summary>Result of a batch write operation (BatchInsert / BatchDelete / BatchUpdate / BatchUpsert).</summary>
public sealed class BatchResult
{
    public uint Succeeded { get; init; }
    public uint Skipped   { get; init; }
}

/// <summary>Result of <c>MinMax()</c>.</summary>
public sealed class MinMaxStats
{
    public ulong Min   { get; init; }
    public ulong Max   { get; init; }
    public ulong Count { get; init; }
}
