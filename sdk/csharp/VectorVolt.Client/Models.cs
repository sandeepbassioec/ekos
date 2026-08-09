namespace VectorVolt.Client;

/// <summary>A single row returned from a scan or point-read operation.</summary>
public sealed class Row
{
    /// <summary>Primary key bytes (little-endian i64).</summary>
    public long PrimaryKey { get; init; }

    /// <summary>Column values in schema order. Null means the column was not stored.</summary>
    public IReadOnlyList<byte[]?> Columns { get; init; } = Array.Empty<byte[]?>();
}

/// <summary>Statistics returned by <c>TableStats</c> (opcode 0x14).</summary>
public sealed class TableStats
{
    public long RowCount      { get; init; }
    public long PartitionCount { get; init; }
    public long DiskBytes     { get; init; }
}

/// <summary>Result of a <c>BatchInsert</c> or <c>BatchDelete</c> call.</summary>
public sealed class BatchResult
{
    public int RowsAffected { get; init; }
    public int RowsFailed   { get; init; }
}

/// <summary>Result of <c>MinField</c> / <c>MaxField</c> aggregate opcodes.</summary>
public sealed class MinMaxResult
{
    /// <summary>True when the table had at least one matching row.</summary>
    public bool HasValue { get; init; }

    /// <summary>
    /// Raw bytes of the min/max value in the server's native encoding.
    /// Interpret according to the column's declared type.
    /// </summary>
    public byte[] Value { get; init; } = Array.Empty<byte>();
}

/// <summary>Describes one column returned by <c>DescribeTable</c>.</summary>
public sealed class ColumnDescriptor
{
    public int    Index    { get; init; }
    public string Name     { get; init; } = string.Empty;
    public string TypeName { get; init; } = string.Empty;
    public bool   Nullable { get; init; }
}

/// <summary>Full schema returned by <c>DescribeTable</c>.</summary>
public sealed class TableSchema
{
    public string                      TableName { get; init; } = string.Empty;
    public IReadOnlyList<ColumnDescriptor> Columns { get; init; } = Array.Empty<ColumnDescriptor>();
}
