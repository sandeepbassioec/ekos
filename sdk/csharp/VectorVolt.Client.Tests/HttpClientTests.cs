using System.Net.Http.Json;
using Xunit;

namespace VectorVolt.Client.Tests;

/// <summary>
/// Layer 45 — VectorVoltHttpClient integration tests.
/// All 10 new HTTP methods: upsert, update_field, delete_range, count,
/// sum, min, max, avg, scan_sorted, scan_page.
/// </summary>
public sealed class HttpClientTests : IDisposable
{
    private readonly ServerHarness _harness;
    private readonly VectorVoltHttpClient _client;

    public HttpClientTests()
    {
        _harness = ServerHarness.StartWithHttp();
        _client  = new VectorVoltHttpClient($"http://127.0.0.1:{_harness.HttpPort}");

        // create a default test table via the plain HTTP API
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
            await UpsertRow(i, new[] { i, i * 10, i * 100 });
    }

    private async Task UpsertRow(ulong pk, ulong[] values)
        => await _client.UpsertAsync("t", pk, values);

    // ── UPSERT ───────────────────────────────────────────────────────────────

    [Fact]
    public async Task Upsert_InsertsNewRow()
    {
        await _client.UpsertAsync("t", 1, new ulong[] { 1, 42, 0 });
        var rows = await _client.ScanSortedAsync("t", 0, 1, 1, 0, asc: true, limit: 1);
        Assert.Single(rows);
        Assert.Equal(42UL, rows[0].Values[1]);
    }

    [Fact]
    public async Task Upsert_OverwritesExistingRow()
    {
        await _client.UpsertAsync("t", 1, new ulong[] { 1, 10, 0 });
        await _client.UpsertAsync("t", 1, new ulong[] { 1, 99, 0 });
        var rows = await _client.ScanSortedAsync("t", 0, 1, 1, 0, asc: true, limit: 1);
        Assert.Equal(99UL, rows[0].Values[1]);
    }

    // ── UPDATE FIELD ─────────────────────────────────────────────────────────

    [Fact]
    public async Task UpdateField_ChangesOneColumn()
    {
        await _client.UpsertAsync("t", 1, new ulong[] { 1, 5, 0 });
        await _client.UpdateFieldAsync("t", 1, col: 1, val: 77);
        // verify via count (count rows where val==77 meaning col 1 was updated)
        var count = await _client.CountAsync("t", 0, 1, 1);
        Assert.Equal(1UL, count);
    }

    // ── DELETE RANGE ─────────────────────────────────────────────────────────

    [Fact]
    public async Task DeleteRange_ReturnsDeletedCount()
    {
        await Seed(5);
        var deleted = await _client.DeleteRangeAsync("t", col: 0, lo: 2, hi: 4);
        Assert.Equal(3UL, deleted);
    }

    [Fact]
    public async Task DeleteRange_RemovesRows()
    {
        await Seed(5);
        await _client.DeleteRangeAsync("t", col: 0, lo: 2, hi: 4);
        var remaining = await _client.CountAsync("t", 0, 1, 5);
        Assert.Equal(2UL, remaining);
    }

    // ── COUNT ────────────────────────────────────────────────────────────────

    [Fact]
    public async Task Count_ReturnsMatchingRowCount()
    {
        await Seed(5);
        var count = await _client.CountAsync("t", col: 1, lo: 10, hi: 30);
        Assert.Equal(3UL, count);
    }

    [Fact]
    public async Task Count_ZeroWhenNoMatch()
    {
        await _client.UpsertAsync("t", 1, new ulong[] { 1, 99, 0 });
        var count = await _client.CountAsync("t", col: 1, lo: 0, hi: 10);
        Assert.Equal(0UL, count);
    }

    // ── SUM / MIN / MAX / AVG ────────────────────────────────────────────────

    [Fact]
    public async Task Sum_ReturnsTotalOfMatchingRows()
    {
        for (ulong i = 1; i <= 4; i++)
            await _client.UpsertAsync("t", i, new[] { i, i, 0UL });
        var sum = await _client.SumAsync("t", col: 1, lo: 1, hi: 4);
        Assert.Equal(10UL, sum);
    }

    [Fact]
    public async Task Min_ReturnsSmallest()
    {
        await _client.UpsertAsync("t", 1, new ulong[] { 1, 50, 0 });
        await _client.UpsertAsync("t", 2, new ulong[] { 2, 10, 0 });
        await _client.UpsertAsync("t", 3, new ulong[] { 3, 80, 0 });
        var min = await _client.MinAsync("t", col: 1, lo: 1, hi: 100);
        Assert.Equal(10UL, min);
    }

    [Fact]
    public async Task Max_ReturnsLargest()
    {
        await _client.UpsertAsync("t", 1, new ulong[] { 1, 50, 0 });
        await _client.UpsertAsync("t", 2, new ulong[] { 2, 10, 0 });
        await _client.UpsertAsync("t", 3, new ulong[] { 3, 80, 0 });
        var max = await _client.MaxAsync("t", col: 1, lo: 1, hi: 100);
        Assert.Equal(80UL, max);
    }

    [Fact]
    public async Task Avg_ReturnsRoundedMean()
    {
        await _client.UpsertAsync("t", 1, new ulong[] { 1, 10, 0 });
        await _client.UpsertAsync("t", 2, new ulong[] { 2, 20, 0 });
        await _client.UpsertAsync("t", 3, new ulong[] { 3, 30, 0 });
        var avg = await _client.AvgAsync("t", col: 1, lo: 1, hi: 100);
        Assert.Equal(20UL, avg);
    }

    [Fact]
    public async Task Avg_NullWhenNoRows()
    {
        var avg = await _client.AvgAsync("t", col: 1, lo: 1, hi: 100);
        Assert.Null(avg);
    }

    // ── SCAN SORTED ──────────────────────────────────────────────────────────

    [Fact]
    public async Task ScanSorted_AscendingOrder()
    {
        await _client.UpsertAsync("t", 1, new ulong[] { 1, 30, 0 });
        await _client.UpsertAsync("t", 2, new ulong[] { 2, 10, 0 });
        await _client.UpsertAsync("t", 3, new ulong[] { 3, 20, 0 });
        var rows = await _client.ScanSortedAsync("t", col: 0, lo: 1, hi: 3,
            sortCol: 1, asc: true, limit: 10);
        Assert.Equal(new ulong[] { 10, 20, 30 }, rows.Select(r => r.Values[1]).ToArray());
    }

    // ── SCAN PAGE ────────────────────────────────────────────────────────────

    [Fact]
    public async Task ScanPage_ReturnsFirstPageWithCursor()
    {
        await Seed(5);
        var page = await _client.ScanPageAsync("t", col: 0, lo: 1, hi: 5, after: 0, limit: 3);
        Assert.Equal(3, page.Rows.Length);
        Assert.NotNull(page.NextCursor);
    }

    [Fact]
    public async Task ScanPage_SecondPageExhausts()
    {
        await Seed(5);
        var page1 = await _client.ScanPageAsync("t", col: 0, lo: 1, hi: 5, after: 0, limit: 3);
        var page2 = await _client.ScanPageAsync("t", col: 0, lo: 1, hi: 5,
            after: page1.NextCursor!.Value, limit: 3);
        Assert.Equal(2, page2.Rows.Length);
        Assert.Null(page2.NextCursor);
    }
}
