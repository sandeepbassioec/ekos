using System.Net.Http.Json;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace VectorVolt.Client;

/// <summary>
/// HTTP/REST client for VectorVolt — covers the Layer 44-60 analytical and write/mutate
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

    // ── HISTOGRAM (Layer 50) ─────────────────────────────────────────────────

    public async Task<HistogramBucket[]> HistogramAsync(
        string table, uint col, ulong lo, ulong hi, uint buckets,
        CancellationToken ct = default)
    {
        var resp = await _http.GetAsync(
            $"tables/{table}/histogram?col={col}&lo={lo}&hi={hi}&buckets={buckets}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        var arr = doc.GetProperty("buckets");
        var result = new HistogramBucket[arr.GetArrayLength()];
        int i = 0;
        foreach (var el in arr.EnumerateArray())
            result[i++] = new HistogramBucket(
                el.GetProperty("lo").GetUInt64(),
                el.GetProperty("hi").GetUInt64(),
                el.GetProperty("count").GetUInt64());
        return result;
    }

    // ── PERCENTILE (Layer 50) ────────────────────────────────────────────────

    public async Task<ulong?> PercentileAsync(
        string table, uint col, uint p,
        CancellationToken ct = default)
    {
        var resp = await _http.GetAsync($"tables/{table}/percentile?col={col}&p={p}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        var v = doc.GetProperty("value");
        return v.ValueKind == JsonValueKind.Null ? null : v.GetUInt64();
    }

    // ── GROUP-BY (Layer 51) ──────────────────────────────────────────────────

    public async Task<GroupEntry[]> GroupCountAsync(
        string table, uint keyCol, CancellationToken ct = default)
        => await GetGroupAsync(table, "count", keyCol, null, ct);

    public async Task<GroupEntry[]> GroupSumAsync(
        string table, uint keyCol, uint valCol, CancellationToken ct = default)
        => await GetGroupAsync(table, "sum", keyCol, valCol, ct);

    public async Task<GroupEntry[]> GroupMinAsync(
        string table, uint keyCol, uint valCol, CancellationToken ct = default)
        => await GetGroupAsync(table, "min", keyCol, valCol, ct);

    public async Task<GroupEntry[]> GroupMaxAsync(
        string table, uint keyCol, uint valCol, CancellationToken ct = default)
        => await GetGroupAsync(table, "max", keyCol, valCol, ct);

    // ── WINDOW FUNCTIONS (Layer 52) ──────────────────────────────────────────

    public async Task<WindowRow[]> WindowRunningSumAsync(
        string table, uint col, CancellationToken ct = default)
        => await GetWindowAsync(table, "running_sum", col, ct);

    public async Task<WindowRow[]> WindowRunningCountAsync(
        string table, uint col, CancellationToken ct = default)
        => await GetWindowAsync(table, "running_count", col, ct);

    public async Task<WindowRow[]> WindowRankAsync(
        string table, uint col, CancellationToken ct = default)
        => await GetWindowAsync(table, "rank", col, ct);

    public async Task<WindowRow[]> WindowDenseRankAsync(
        string table, uint col, CancellationToken ct = default)
        => await GetWindowAsync(table, "dense_rank", col, ct);

    // ── DISTINCT + VALUE COUNTS (Layer 53) ───────────────────────────────────

    public async Task<ulong[]> DistinctAsync(
        string table, uint col, CancellationToken ct = default)
    {
        var resp = await _http.GetAsync($"tables/{table}/distinct?col={col}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        var arr = doc.GetProperty("values");
        var result = new ulong[arr.GetArrayLength()];
        int i = 0;
        foreach (var el in arr.EnumerateArray()) result[i++] = el.GetUInt64();
        return result;
    }

    public async Task<ValueCountEntry[]> ValueCountsAsync(
        string table, uint col, CancellationToken ct = default)
    {
        var resp = await _http.GetAsync($"tables/{table}/value_counts?col={col}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        var arr = doc.GetProperty("entries");
        var result = new ValueCountEntry[arr.GetArrayLength()];
        int i = 0;
        foreach (var el in arr.EnumerateArray())
            result[i++] = new ValueCountEntry(
                el.GetProperty("value").GetUInt64(),
                el.GetProperty("count").GetUInt64());
        return result;
    }

    // ── BULK PREDICATE UPDATES (Layer 54) ────────────────────────────────────

    public async Task<ulong> UpdateWhereAsync(
        string table, uint col, string op, ulong val, uint targetCol, ulong newVal,
        CancellationToken ct = default)
    {
        var resp = await _http.GetAsync(
            $"tables/{table}/update_where?col={col}&op={op}&val={val}" +
            $"&target_col={targetCol}&new_val={newVal}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return doc.GetProperty("updated").GetUInt64();
    }

    public async Task<ulong> IncrementWhereAsync(
        string table, uint col, string op, ulong val, uint targetCol, ulong delta,
        CancellationToken ct = default)
    {
        var resp = await _http.GetAsync(
            $"tables/{table}/increment_where?col={col}&op={op}&val={val}" +
            $"&target_col={targetCol}&delta={delta}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return doc.GetProperty("updated").GetUInt64();
    }

    public async Task<ulong> DecrementWhereAsync(
        string table, uint col, string op, ulong val, uint targetCol, ulong delta,
        CancellationToken ct = default)
    {
        var resp = await _http.GetAsync(
            $"tables/{table}/decrement_where?col={col}&op={op}&val={val}" +
            $"&target_col={targetCol}&delta={delta}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return doc.GetProperty("updated").GetUInt64();
    }

    // ── BATCH GET + DELETE (Layer 55) ────────────────────────────────────────

    public async Task<HttpRow?[]> BatchGetAsync(
        string table, IReadOnlyList<ulong> pks, CancellationToken ct = default)
    {
        var body = new { pks };
        var resp = await _http.PostAsJsonAsync($"tables/{table}/batch/get", body, _opts, ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        var arr = doc.GetProperty("rows");
        var result = new HttpRow?[arr.GetArrayLength()];
        int i = 0;
        foreach (var el in arr.EnumerateArray())
        {
            if (el.ValueKind == JsonValueKind.Null)
                result[i++] = null;
            else
                result[i++] = new HttpRow(
                    el.GetProperty("pk").GetUInt64(),
                    ParseUlongArray(el.GetProperty("values")));
        }
        return result;
    }

    public async Task<ulong> BatchDeleteAsync(
        string table, IReadOnlyList<ulong> pks, CancellationToken ct = default)
    {
        var body = new { pks };
        var resp = await _http.PostAsJsonAsync($"tables/{table}/batch/delete", body, _opts, ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return doc.GetProperty("deleted").GetUInt64();
    }

    // ── SCAN WHERE (Layer 56) ────────────────────────────────────────────────

    /// <param name="predicates">Up to 3 range predicates, each as (col, lo, hi).</param>
    public async Task<HttpRow[]> ScanWhereAsync(
        string table, IReadOnlyList<(uint col, ulong lo, ulong hi)> predicates,
        CancellationToken ct = default)
    {
        if (predicates.Count == 0)
            throw new ArgumentException("At least one predicate is required.", nameof(predicates));
        var sb = new StringBuilder($"tables/{table}/scan_where?");
        for (int i = 0; i < Math.Min(predicates.Count, 3); i++)
        {
            var (col, lo, hi) = predicates[i];
            if (i > 0) sb.Append('&');
            sb.Append($"col{i}={col}&lo{i}={lo}&hi{i}={hi}");
        }
        var resp = await _http.GetAsync(sb.ToString(), ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return ParseRows(doc.GetProperty("rows"));
    }

    // ── SCAN PROJECT (Layer 57) ──────────────────────────────────────────────

    public async Task<HttpRow[]> ScanProjectAsync(
        string table, uint filterCol, ulong lo, ulong hi,
        IReadOnlyList<uint> projCols,
        CancellationToken ct = default)
    {
        var projStr = string.Join(",", projCols);
        var resp = await _http.GetAsync(
            $"tables/{table}/scan/project?filter_col={filterCol}&lo={lo}&hi={hi}&proj={projStr}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return ParseRows(doc.GetProperty("rows"));
    }

    // ── INSERT IF ABSENT (Layer 58) ──────────────────────────────────────────

    public async Task<bool> InsertIfAbsentAsync(
        string table, ulong pk, IReadOnlyList<ulong> values,
        CancellationToken ct = default)
    {
        var body = new { values };
        var resp = await _http.PostAsJsonAsync(
            $"tables/{table}/rows/{pk}/if_absent", body, _opts, ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return doc.GetProperty("inserted").GetBoolean();
    }

    // ── COMPARE AND SWAP (Layer 58) ──────────────────────────────────────────

    public async Task<bool> CompareAndSwapAsync(
        string table, ulong pk,
        IReadOnlyList<ulong> expected, IReadOnlyList<ulong> newValues,
        CancellationToken ct = default)
    {
        var body = new { expected, new_values = newValues };
        var resp = await _http.PostAsJsonAsync(
            $"tables/{table}/rows/{pk}/cas", body, _opts, ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return doc.GetProperty("swapped").GetBoolean();
    }

    // ── BATCH INSERT (Layer 59) ──────────────────────────────────────────────

    public async Task<ulong> BatchInsertAsync(
        string table, IReadOnlyList<(ulong pk, IReadOnlyList<ulong> values)> rows,
        CancellationToken ct = default)
    {
        var body = new { rows = rows.Select(r => new { pk = r.pk, values = r.values }).ToArray() };
        var resp = await _http.PostAsJsonAsync($"tables/{table}/batch/insert", body, _opts, ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return doc.GetProperty("inserted").GetUInt64();
    }

    // ── TTL OPERATIONS (Layer 60) ────────────────────────────────────────────

    public async Task InsertWithTtlAsync(
        string table, ulong pk, IReadOnlyList<ulong> values, ulong expireAt,
        CancellationToken ct = default)
    {
        var body = new { values, expire_at = expireAt };
        var resp = await _http.PostAsJsonAsync(
            $"tables/{table}/rows/{pk}/ttl", body, _opts, ct);
        resp.EnsureSuccessStatusCode();
    }

    public async Task<HttpRow?> GetWithTtlAsync(
        string table, ulong pk, ulong now,
        CancellationToken ct = default)
    {
        var resp = await _http.GetAsync(
            $"tables/{table}/get_with_ttl/{pk}?now={now}", ct);
        if (resp.StatusCode == System.Net.HttpStatusCode.NotFound)
            return null;
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return new HttpRow(
            doc.GetProperty("pk").GetUInt64(),
            ParseUlongArray(doc.GetProperty("values")));
    }

    public async Task<HttpRow[]> ScanWithTtlAsync(
        string table, ulong now, CancellationToken ct = default)
    {
        var resp = await _http.GetAsync($"tables/{table}/scan_with_ttl?now={now}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return ParseRows(doc.GetProperty("rows"));
    }

    public async Task<ulong> PurgeExpiredAsync(
        string table, ulong now, CancellationToken ct = default)
    {
        var resp = await _http.PostAsync(
            $"tables/{table}/purge_expired?now={now}", null, ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        return doc.GetProperty("purged").GetUInt64();
    }

    // ── DSL PASS-THROUGH ─────────────────────────────────────────────────────

    public async Task<JsonElement> QueryAsync(
        string table, string dsl, CancellationToken ct = default)
    {
        var content = new StringContent(dsl, Encoding.UTF8, "text/plain");
        var resp = await _http.PostAsync($"tables/{table}/query", content, ct);
        resp.EnsureSuccessStatusCode();
        return await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
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

    private async Task<GroupEntry[]> GetGroupAsync(
        string table, string agg, uint keyCol, uint? valCol, CancellationToken ct)
    {
        var qs = valCol.HasValue
            ? $"tables/{table}/group/{agg}?key_col={keyCol}&val_col={valCol}"
            : $"tables/{table}/group/{agg}?key_col={keyCol}";
        var resp = await _http.GetAsync(qs, ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        var arr = doc.GetProperty("entries");
        var result = new GroupEntry[arr.GetArrayLength()];
        int i = 0;
        foreach (var el in arr.EnumerateArray())
            result[i++] = new GroupEntry(
                el.GetProperty("group_val").GetUInt64(),
                el.GetProperty("agg_val").GetUInt64());
        return result;
    }

    private async Task<WindowRow[]> GetWindowAsync(
        string table, string fn, uint col, CancellationToken ct)
    {
        var resp = await _http.GetAsync($"tables/{table}/window/{fn}?col={col}", ct);
        resp.EnsureSuccessStatusCode();
        var doc = await resp.Content.ReadFromJsonAsync<JsonElement>(cancellationToken: ct);
        var arr = doc.GetProperty("rows");
        var result = new WindowRow[arr.GetArrayLength()];
        int i = 0;
        foreach (var el in arr.EnumerateArray())
            result[i++] = new WindowRow(
                el.GetProperty("pk").GetUInt64(),
                el.GetProperty("window_val").GetUInt64());
        return result;
    }

    private static HttpRow[] ParseRows(JsonElement arr)
    {
        var result = new HttpRow[arr.GetArrayLength()];
        int i = 0;
        foreach (var el in arr.EnumerateArray())
        {
            var pk = el.GetProperty("pk").GetUInt64();
            result[i++] = new HttpRow(pk, ParseUlongArray(el.GetProperty("values")));
        }
        return result;
    }

    private static ulong[] ParseUlongArray(JsonElement arr)
    {
        var result = new ulong[arr.GetArrayLength()];
        int i = 0;
        foreach (var v in arr.EnumerateArray()) result[i++] = v.GetUInt64();
        return result;
    }

    public void Dispose()
    {
        if (!_disposed) { _http.Dispose(); _disposed = true; }
    }
}

// ── model types ──────────────────────────────────────────────────────────────

/// <summary>A row returned from an HTTP endpoint.</summary>
public record HttpRow(ulong Pk, ulong[] Values);

/// <summary>Result of a cursor-based page scan.</summary>
public record HttpPageResult(HttpRow[] Rows, ulong? NextCursor);

/// <summary>One bucket in a histogram result.</summary>
public record HistogramBucket(ulong Lo, ulong Hi, ulong Count);

/// <summary>One entry in a group-by result (key + aggregate value).</summary>
public record GroupEntry(ulong Key, ulong AggVal);

/// <summary>One row in a window-function result.</summary>
public record WindowRow(ulong Pk, ulong WindowVal);

/// <summary>One entry in a value_counts result.</summary>
public record ValueCountEntry(ulong Value, ulong Count);
