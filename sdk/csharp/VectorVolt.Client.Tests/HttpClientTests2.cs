using System.Net.Http.Json;
using Xunit;

namespace VectorVolt.Client.Tests;

/// <summary>
/// Layer 62 — VectorVoltHttpClient integration tests covering Layers 50-60:
///   histogram, percentile, group-by ×4, window functions ×4,
///   distinct, value_counts, update/increment/decrement-where,
///   batch_get, batch_delete, scan_where, scan_project,
///   insert_if_absent, compare_and_swap, batch_insert,
///   TTL ops (insert/get/scan/purge), query DSL pass-through.
/// </summary>
public sealed class HttpClientTests2 : IDisposable
{
    private readonly ServerHarness _harness;
    private readonly VectorVoltHttpClient _client;

    public HttpClientTests2()
    {
        _harness = ServerHarness.StartWithHttp();
        _client  = new VectorVoltHttpClient($"http://127.0.0.1:{_harness.HttpPort}");

        // create default test table
        using var http = new HttpClient();
        http.PostAsJsonAsync(
            $"http://127.0.0.1:{_harness.HttpPort}/tables/t",
            new { columns = new[] { "pk", "val", "score" } }).Wait();
    }

    public void Dispose()
    {
        _client.Dispose();
        _harness.Dispose();
    }

    private async Task Seed(int n = 5)
    {
        for (ulong i = 1; i <= (ulong)n; i++)
            await _client.UpsertAsync("t", i, new ulong[] { i, i * 10, i * 100 });
    }

    // ── Layer 50: Histogram ───────────────────────────────────────────────────

    [Fact]
    public async Task Histogram_ReturnsBuckets()
    {
        await Seed();
        var buckets = await _client.HistogramAsync("t", col: 1, lo: 0, hi: 60, buckets: 3);
        Assert.Equal(3, buckets.Length);
        Assert.Equal(5UL, buckets.Sum(b => (long)b.Count));
    }

    [Fact]
    public async Task Histogram_EmptyTable_AllZeroCounts()
    {
        var buckets = await _client.HistogramAsync("t", col: 0, lo: 0, hi: 100, buckets: 5);
        Assert.Equal(0UL, buckets.Sum(b => (long)b.Count));
    }

    // ── Layer 50: Percentile ─────────────────────────────────────────────────

    [Fact]
    public async Task Percentile_ReturnsValue()
    {
        await Seed();
        var p50 = await _client.PercentileAsync("t", col: 1, p: 50);
        Assert.NotNull(p50);
        Assert.InRange(p50.Value, 10UL, 50UL);
    }

    [Fact]
    public async Task Percentile_EmptyTable_ReturnsNull()
    {
        var result = await _client.PercentileAsync("t", col: 0, p: 50);
        Assert.Null(result);
    }

    // ── Layer 51: Group-by ────────────────────────────────────────────────────

    [Fact]
    public async Task GroupCount_CountsPerKey()
    {
        // 3 rows with val=10, 2 with val=20
        ulong[][] rows = { [1,10,0],[2,10,0],[3,10,0],[4,20,0],[5,20,0] };
        for (ulong i = 0; i < 5; i++)
            await _client.UpsertAsync("t", i + 1, rows[i]);
        var groups = await _client.GroupCountAsync("t", keyCol: 1);
        var dict = groups.ToDictionary(g => g.Key, g => g.AggVal);
        Assert.Equal(3UL, dict[10]);
        Assert.Equal(2UL, dict[20]);
    }

    [Fact]
    public async Task GroupSum_SumsValCol()
    {
        for (ulong i = 1; i <= 3; i++)
            await _client.UpsertAsync("t", i, new ulong[] { i, 1, i * 10 });
        var groups = await _client.GroupSumAsync("t", keyCol: 1, valCol: 2);
        var total = groups.Sum(g => (long)g.AggVal);
        Assert.Equal(60L, total);
    }

    [Fact]
    public async Task GroupMin_FindsMinimum()
    {
        for (ulong i = 1; i <= 3; i++)
            await _client.UpsertAsync("t", i, new ulong[] { i, 1, i * 5 });
        var groups = await _client.GroupMinAsync("t", keyCol: 1, valCol: 2);
        Assert.Equal(5UL, groups[0].AggVal);
    }

    [Fact]
    public async Task GroupMax_FindsMaximum()
    {
        for (ulong i = 1; i <= 3; i++)
            await _client.UpsertAsync("t", i, new ulong[] { i, 1, i * 5 });
        var groups = await _client.GroupMaxAsync("t", keyCol: 1, valCol: 2);
        Assert.Equal(15UL, groups[0].AggVal);
    }

    // ── Layer 52: Window functions ────────────────────────────────────────────

    [Fact]
    public async Task WindowRunningSum_AccumulatesCorrectly()
    {
        for (ulong i = 1; i <= 3; i++)
            await _client.UpsertAsync("t", i, new ulong[] { i, i * 10, 0 });
        var rows = await _client.WindowRunningSumAsync("t", col: 1);
        Assert.Equal(3, rows.Length);
        Assert.Equal(60UL, rows[^1].WindowVal);
    }

    [Fact]
    public async Task WindowRank_AssignsDistinctRanks()
    {
        for (ulong i = 1; i <= 3; i++)
            await _client.UpsertAsync("t", i, new ulong[] { i, i * 10, 0 });
        var rows = await _client.WindowRankAsync("t", col: 1);
        var ranks = rows.Select(r => r.WindowVal).OrderBy(v => v).ToArray();
        Assert.Equal(new ulong[] { 1, 2, 3 }, ranks);
    }

    [Fact]
    public async Task WindowDenseRank_NoGapsOnTies()
    {
        ulong[] pks  = [1, 2, 3];
        ulong[] vals = [10, 10, 20];
        for (int i = 0; i < 3; i++)
            await _client.UpsertAsync("t", pks[i], new ulong[] { pks[i], vals[i], 0 });
        var rows = await _client.WindowDenseRankAsync("t", col: 1);
        var distinct = rows.Select(r => r.WindowVal).ToHashSet();
        Assert.Equal(new HashSet<ulong> { 1, 2 }, distinct);
    }

    [Fact]
    public async Task WindowRunningCount_IncrementsPerRow()
    {
        for (ulong i = 1; i <= 3; i++)
            await _client.UpsertAsync("t", i, new ulong[] { i, i, 0 });
        var rows = await _client.WindowRunningCountAsync("t", col: 1);
        Assert.Equal(3UL, rows[^1].WindowVal);
    }

    // ── Layer 53: Distinct + value_counts ────────────────────────────────────

    [Fact]
    public async Task Distinct_ReturnsUniqueValues()
    {
        ulong[] vals = [5, 5, 10, 15];
        for (ulong i = 0; i < 4; i++)
            await _client.UpsertAsync("t", i + 1, new ulong[] { i + 1, vals[i], 0 });
        var result = await _client.DistinctAsync("t", col: 1);
        Assert.Equal(new ulong[] { 5, 10, 15 }, result.OrderBy(v => v).ToArray());
    }

    [Fact]
    public async Task ValueCounts_CountsOccurrences()
    {
        ulong[] vals = [7, 7, 7, 9];
        for (ulong i = 0; i < 4; i++)
            await _client.UpsertAsync("t", i + 1, new ulong[] { i + 1, vals[i], 0 });
        var entries = await _client.ValueCountsAsync("t", col: 1);
        var dict = entries.ToDictionary(e => e.Value, e => e.Count);
        Assert.Equal(3UL, dict[7]);
        Assert.Equal(1UL, dict[9]);
    }

    // ── Layer 54: Bulk predicate updates ─────────────────────────────────────

    [Fact]
    public async Task UpdateWhere_UpdatesMatchingRows()
    {
        await Seed();
        var updated = await _client.UpdateWhereAsync("t", col: 0, op: "le", val: 3,
            targetCol: 1, newVal: 999);
        Assert.Equal(3UL, updated);
        var rows = await _client.ScanSortedAsync("t", 0, 1, 3, 0);
        Assert.All(rows, r => Assert.Equal(999UL, r.Values[1]));
    }

    [Fact]
    public async Task IncrementWhere_IncrementsMatchingRows()
    {
        await Seed();
        var updated = await _client.IncrementWhereAsync("t", col: 0, op: "ge", val: 4,
            targetCol: 1, delta: 5);
        Assert.Equal(2UL, updated);
        var rows = await _client.ScanSortedAsync("t", 0, 4, 5, 0);
        Assert.Equal(45UL, rows[0].Values[1]);
        Assert.Equal(55UL, rows[1].Values[1]);
    }

    [Fact]
    public async Task DecrementWhere_DecrementsMatchingRow()
    {
        await Seed();
        var updated = await _client.DecrementWhereAsync("t", col: 0, op: "eq", val: 1,
            targetCol: 1, delta: 5);
        Assert.Equal(1UL, updated);
        var rows = await _client.ScanSortedAsync("t", 0, 1, 1, 0);
        Assert.Equal(5UL, rows[0].Values[1]);
    }

    // ── Layer 55: Batch get + delete ─────────────────────────────────────────

    [Fact]
    public async Task BatchGet_ReturnsNullForMissing()
    {
        await Seed();
        var rows = await _client.BatchGetAsync("t", new ulong[] { 1, 99, 3 });
        Assert.Equal(3, rows.Length);
        Assert.NotNull(rows[0]);
        Assert.Null(rows[1]);
        Assert.NotNull(rows[2]);
    }

    [Fact]
    public async Task BatchDelete_ReturnsCount()
    {
        await Seed();
        var deleted = await _client.BatchDeleteAsync("t", new ulong[] { 1, 2, 3 });
        Assert.Equal(3UL, deleted);
        var remaining = await _client.BatchGetAsync("t", new ulong[] { 1 });
        Assert.Null(remaining[0]);
    }

    // ── Layer 56: Scan where ──────────────────────────────────────────────────

    [Fact]
    public async Task ScanWhere_SinglePredicate()
    {
        await Seed();
        var rows = await _client.ScanWhereAsync("t",
            new[] { (col: 0u, lo: 2UL, hi: 4UL) });
        var pks = rows.Select(r => r.Pk).ToHashSet();
        Assert.Equal(new HashSet<ulong> { 2, 3, 4 }, pks);
    }

    [Fact]
    public async Task ScanWhere_TwoPredicates()
    {
        await Seed();
        var rows = await _client.ScanWhereAsync("t", new[]
        {
            (col: 0u, lo: 1UL, hi: 5UL),
            (col: 1u, lo: 20UL, hi: 40UL),
        });
        var pks = rows.Select(r => r.Pk).ToHashSet();
        Assert.Equal(new HashSet<ulong> { 2, 3, 4 }, pks);
    }

    // ── Layer 57: Scan project ────────────────────────────────────────────────

    [Fact]
    public async Task ScanProject_ReturnsOnlyProjectedCols()
    {
        await Seed();
        var rows = await _client.ScanProjectAsync("t", filterCol: 0, lo: 1, hi: 5,
            projCols: new uint[] { 0, 2 });
        Assert.Equal(5, rows.Length);
        Assert.All(rows, r => Assert.Equal(2, r.Values.Length));
    }

    [Fact]
    public async Task ScanProject_SingleCol()
    {
        await Seed();
        var rows = await _client.ScanProjectAsync("t", filterCol: 0, lo: 1, hi: 3,
            projCols: new uint[] { 1 });
        Assert.Equal(3, rows.Length);
        Assert.All(rows, r => Assert.Single(r.Values));
    }

    // ── Layer 58: Conditional writes ─────────────────────────────────────────

    [Fact]
    public async Task InsertIfAbsent_InsertsNew()
    {
        var inserted = await _client.InsertIfAbsentAsync("t", 42,
            new ulong[] { 42, 100, 0 });
        Assert.True(inserted);
    }

    [Fact]
    public async Task InsertIfAbsent_SkipsExisting()
    {
        await _client.UpsertAsync("t", 10, new ulong[] { 10, 5, 0 });
        var inserted = await _client.InsertIfAbsentAsync("t", 10,
            new ulong[] { 10, 999, 0 });
        Assert.False(inserted);
    }

    [Fact]
    public async Task CompareAndSwap_SwapsOnMatch()
    {
        await _client.UpsertAsync("t", 7, new ulong[] { 7, 50, 0 });
        var swapped = await _client.CompareAndSwapAsync("t", 7,
            expected: new ulong[] { 7, 50, 0 },
            newValues: new ulong[] { 7, 99, 0 });
        Assert.True(swapped);
    }

    [Fact]
    public async Task CompareAndSwap_NoSwapOnMismatch()
    {
        await _client.UpsertAsync("t", 8, new ulong[] { 8, 50, 0 });
        var swapped = await _client.CompareAndSwapAsync("t", 8,
            expected: new ulong[] { 8, 999, 0 },
            newValues: new ulong[] { 8, 1, 0 });
        Assert.False(swapped);
    }

    // ── Layer 59: Batch insert ────────────────────────────────────────────────

    [Fact]
    public async Task BatchInsert_ReturnsCount()
    {
        var n = await _client.BatchInsertAsync("t", new[]
        {
            (pk: 1UL, values: (IReadOnlyList<ulong>)new ulong[] { 1, 10, 0 }),
            (pk: 2UL, values: (IReadOnlyList<ulong>)new ulong[] { 2, 20, 0 }),
            (pk: 3UL, values: (IReadOnlyList<ulong>)new ulong[] { 3, 30, 0 }),
        });
        Assert.Equal(3UL, n);
    }

    [Fact]
    public async Task BatchInsert_RowsRetrievable()
    {
        await _client.BatchInsertAsync("t", new[]
        {
            (pk: 50UL, values: (IReadOnlyList<ulong>)new ulong[] { 50, 500, 0 }),
            (pk: 51UL, values: (IReadOnlyList<ulong>)new ulong[] { 51, 510, 0 }),
        });
        var rows = await _client.BatchGetAsync("t", new ulong[] { 50, 51 });
        Assert.Equal(500UL, rows[0]!.Values[1]);
        Assert.Equal(510UL, rows[1]!.Values[1]);
    }

    [Fact]
    public async Task BatchInsert_EmptyReturnsZero()
    {
        var n = await _client.BatchInsertAsync("t",
            Array.Empty<(ulong pk, IReadOnlyList<ulong> values)>());
        Assert.Equal(0UL, n);
    }

    // ── Layer 60: TTL operations ──────────────────────────────────────────────

    [Fact]
    public async Task InsertWithTtl_GetWithinTtl_ReturnsRow()
    {
        await _client.InsertWithTtlAsync("t", 100, new ulong[] { 100, 77, 0 },
            expireAt: 9_999_999_999UL);
        var row = await _client.GetWithTtlAsync("t", 100, now: 0);
        Assert.NotNull(row);
        Assert.Equal(77UL, row!.Values[1]);
    }

    [Fact]
    public async Task GetWithTtl_ReturnsNullAfterExpiry()
    {
        await _client.InsertWithTtlAsync("t", 101, new ulong[] { 101, 1, 0 },
            expireAt: 100);
        var row = await _client.GetWithTtlAsync("t", 101, now: 999);
        Assert.Null(row);
    }

    [Fact]
    public async Task ScanWithTtl_ExcludesExpired()
    {
        await _client.InsertWithTtlAsync("t", 200, new ulong[] { 200, 1, 0 },
            expireAt: 9_999_999_999UL);
        await _client.InsertWithTtlAsync("t", 201, new ulong[] { 201, 2, 0 },
            expireAt: 1);
        var rows = await _client.ScanWithTtlAsync("t", now: 100);
        var pks = rows.Select(r => r.Pk).ToHashSet();
        Assert.Contains(200UL, pks);
        Assert.DoesNotContain(201UL, pks);
    }

    [Fact]
    public async Task PurgeExpired_RemovesExpiredRows()
    {
        await _client.InsertWithTtlAsync("t", 300, new ulong[] { 300, 1, 0 },
            expireAt: 1);
        await _client.InsertWithTtlAsync("t", 301, new ulong[] { 301, 2, 0 },
            expireAt: 9_999_999_999UL);
        var purged = await _client.PurgeExpiredAsync("t", now: 100);
        Assert.True(purged >= 1);
        var remaining = await _client.ScanWithTtlAsync("t", now: 0);
        var pks = remaining.Select(r => r.Pk).ToHashSet();
        Assert.DoesNotContain(300UL, pks);
        Assert.Contains(301UL, pks);
    }

    // ── DSL pass-through ─────────────────────────────────────────────────────

    [Fact]
    public async Task QueryDsl_InsertIfAbsent()
    {
        var result = await _client.QueryAsync("t", "INSERT IF_ABSENT 88 88 100 0");
        Assert.True(result.GetProperty("inserted").GetBoolean());
        var result2 = await _client.QueryAsync("t", "INSERT IF_ABSENT 88 88 200 0");
        Assert.False(result2.GetProperty("inserted").GetBoolean());
    }
}
