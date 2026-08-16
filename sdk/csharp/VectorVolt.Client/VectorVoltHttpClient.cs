using System.Net.Http.Json;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace VectorVolt.Client;

/// <summary>
/// HTTP/REST client for VectorVolt — covers the Layer 44 analytical and write/mutate
/// endpoints that are not part of the TCP wire protocol.
///
/// <para>For TCP-based operations (the full 122-opcode surface), use
/// <see cref="VectorVoltClient"/>.</para>
/// </summary>
public sealed class VectorVoltHttpClient : IDisposable
{
    private readonly HttpClient _http;
    private bool _disposed;

    private static readonly JsonSerializerOptions _opts = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };

    /// <param name="baseUrl">Server HTTP base URL, e.g. <c>http://127.0.0.1:7879</c>.</param>
    /// <param name="bearerToken">Optional Bearer token for auth-enabled servers.</param>
    public VectorVoltHttpClient(string baseUrl, string? bearerToken = null)
    {
        _http = new HttpClient { BaseAddress = new Uri(baseUrl.TrimEnd('/') + "/") };
        if (bearerToken != null)
            _http.DefaultRequestHeaders.Authorization =
                new System.Net.Http.Headers.AuthenticationHeaderValue("Bearer", bearerToken);
    }

    // ── UPSERT ───────────────────────────────────────────────────────────────

    public async Task UpsertAsync(string table, ulong pk, IReadOnlyList<ulong> values,
        CancellationToken ct = default)
    {
        var body = new { values };
        var resp = await _http.PutAsJsonAsync($"tables/{table}/rows/{pk}", body, _opts, ct);
        resp.EnsureSuccessStatusCode();
    }

    // ── UPDATE FIELD ─────────────────────────────────────────────────────────

    public async Task UpdateFieldAsync(string table, ulong pk, uint col, ulong val,
        CancellationToken ct = default)
    {
        var body = new { col, val };
        var resp = await _http.PatchAsJsonAsync($"tables/{table}/rows/{pk}/field", body, _opts, ct);
        resp.EnsureSuccessStatusCode();
    }

    // ── DELETE RANGE ─────────────────────────────────────────────────────────

    public async Task<ulong> DeleteRangeAsync(string table, uint col, ulong lo, ulong hi,
        CancellationToken ct = default)
    {
        var resp = await _http.DeleteAsync(
            $"tables/{table}/rows/range?col={col}&lo={lo}&hi={hi}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return doc.GetProperty("count").GetUInt64();
    }

    // ── COUNT ────────────────────────────────────────────────────────────────

    public async Task<ulong> CountAsync(string table, uint col, ulong lo, ulong hi,
        CancellationToken ct = default)
    {
        var resp = await _http.GetAsync($"tables/{table}/count?col={col}&lo={lo}&hi={hi}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return doc.GetProperty("count").GetUInt64();
    }

    // ── SUM ──────────────────────────────────────────────────────────────────

    public async Task<ulong?> SumAsync(string table, uint col, ulong lo, ulong hi,
        CancellationToken ct = default)
        => await GetAggAsync(table, "sum", col, lo, hi, ct);

    // ── MIN / MAX ────────────────────────────────────────────────────────────

    public async Task<ulong?> MinAsync(string table, uint col, ulong lo, ulong hi,
        CancellationToken ct = default)
        => await GetAggAsync(table, "min", col, lo, hi, ct);

    public async Task<ulong?> MaxAsync(string table, uint col, ulong lo, ulong hi,
        CancellationToken ct = default)
        => await GetAggAsync(table, "max", col, lo, hi, ct);

    // ── AVG ──────────────────────────────────────────────────────────────────

    public async Task<ulong?> AvgAsync(string table, uint col, ulong lo, ulong hi,
        CancellationToken ct = default)
        => await GetAggAsync(table, "avg", col, lo, hi, ct);

    // ── SCAN SORTED ──────────────────────────────────────────────────────────

    public async Task<HttpRow[]> ScanSortedAsync(
        string table, uint col, ulong lo, ulong hi,
        uint sortCol, bool asc = true, uint limit = 100,
        CancellationToken ct = default)
    {
        var ascInt = asc ? 1 : 0;
        var resp = await _http.GetAsync(
            $"tables/{table}/scan/sorted?col={col}&lo={lo}&hi={hi}" +
            $"&sort={sortCol}&asc={ascInt}&limit={limit}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return ParseRows(doc.GetProperty("rows"));
    }

    // ── SCAN PAGE ────────────────────────────────────────────────────────────

    public async Task<HttpPageResult> ScanPageAsync(
        string table, uint col, ulong lo, ulong hi,
        ulong after = 0, uint limit = 100,
        CancellationToken ct = default)
    {
        var resp = await _http.GetAsync(
            $"tables/{table}/scan/page?col={col}&lo={lo}&hi={hi}" +
            $"&after={after}&limit={limit}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        var rows = ParseRows(doc.GetProperty("rows"));
        ulong? cursor = null;
        if (doc.TryGetProperty("next_cursor", out var cur) && cur.ValueKind != JsonValueKind.Null)
            cursor = cur.GetUInt64();
        return new HttpPageResult(rows, cursor);
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    private async Task<ulong?> GetAggAsync(
        string table, string endpoint, uint col, ulong lo, ulong hi,
        CancellationToken ct)
    {
        var resp = await _http.GetAsync(
            $"tables/{table}/{endpoint}?col={col}&lo={lo}&hi={hi}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        var v = doc.GetProperty("value");
        return v.ValueKind == JsonValueKind.Null ? null : v.GetUInt64();
    }

    private static HttpRow[] ParseRows(JsonElement arr)
    {
        var result = new HttpRow[arr.GetArrayLength()];
        int i = 0;
        foreach (var el in arr.EnumerateArray())
        {
            var pk = el.GetProperty("pk").GetUInt64();
            var vals = el.GetProperty("values");
            var values = new ulong[vals.GetArrayLength()];
            int j = 0;
            foreach (var v in vals.EnumerateArray()) values[j++] = v.GetUInt64();
            result[i++] = new HttpRow(pk, values);
        }
        return result;
    }

    public void Dispose()
    {
        if (!_disposed) { _http.Dispose(); _disposed = true; }
    }
}

/// <summary>A row returned from an HTTP endpoint.</summary>
public record HttpRow(ulong Pk, ulong[] Values);

/// <summary>Result of a cursor-based page scan.</summary>
public record HttpPageResult(HttpRow[] Rows, ulong? NextCursor);
