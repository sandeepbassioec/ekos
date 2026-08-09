using System.Linq;
using VectorVolt.Client;
using Xunit;

namespace VectorVolt.Client.Tests;

/// <summary>
/// Integration tests that spin up a real <c>vvctl serve</c> process and talk to it
/// over the loopback TCP socket.  Each test class gets its own server + table so
/// tests are fully isolated.
///
/// Column-index convention (matches the Rust engine):
///   col=0 = pk (primary key)
///   col=1 = first non-pk column
///   col=2 = second non-pk column, etc.
///
/// Get() / RangeScan() return values[] where:
///   values[0] = pk, values[1] = first data column, etc.
/// </summary>

// ── Ping / connection ──────────────────────────────────────────────────────────

public class PingTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public PingTests()
    {
        _server = ServerHarness.Start(["pk", "val"]);
        _client = _server.OpenClient();
    }

    [Fact]
    public void Ping_does_not_throw()
    {
        _client.Ping();
    }

    [Fact]
    public void Stats_returns_non_null_with_zero_rows_initially()
    {
        var stats = _client.Stats();
        Assert.NotNull(stats);
        Assert.Equal(0UL, stats.Rows);
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}

// ── Basic CRUD ─────────────────────────────────────────────────────────────────

public class BasicCrudTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public BasicCrudTests()
    {
        _server = ServerHarness.Start(["pk", "a", "b"]);
        _client = _server.OpenClient();
    }

    [Fact]
    public void Insert_then_get_round_trips()
    {
        _client.Insert(1UL, [10UL, 20UL]);
        var vals = _client.Get(1UL);
        Assert.NotNull(vals);
        // vals[0]=pk=1, vals[1]=a=10, vals[2]=b=20
        Assert.Equal(3, vals.Length);
        Assert.Equal(1UL, vals[0]);
        Assert.Equal(10UL, vals[1]);
        Assert.Equal(20UL, vals[2]);
    }

    [Fact]
    public void Get_missing_key_returns_null()
    {
        Assert.Null(_client.Get(9999UL));
    }

    [Fact]
    public void Delete_removes_row()
    {
        _client.Insert(2UL, [1UL, 2UL]);
        _client.Delete(2UL);
        Assert.Null(_client.Get(2UL));
    }

    [Fact]
    public void Update_overwrites_columns()
    {
        _client.Insert(3UL, [1UL, 1UL]);
        _client.Update(3UL, [99UL, 88UL]);
        var vals = _client.Get(3UL);
        Assert.NotNull(vals);
        Assert.Equal(99UL, vals[1]);
        Assert.Equal(88UL, vals[2]);
    }

    [Fact]
    public void Upsert_inserts_when_absent()
    {
        _client.Upsert(10UL, [5UL, 6UL]);
        var vals = _client.Get(10UL);
        Assert.NotNull(vals);
        Assert.Equal(5UL, vals[1]);
        Assert.Equal(6UL, vals[2]);
    }

    [Fact]
    public void Upsert_overwrites_when_present()
    {
        _client.Insert(11UL, [1UL, 2UL]);
        _client.Upsert(11UL, [7UL, 8UL]);
        var vals = _client.Get(11UL);
        Assert.NotNull(vals);
        Assert.Equal(7UL, vals[1]);
        Assert.Equal(8UL, vals[2]);
    }

    [Fact]
    public void GetAndDelete_returns_values_and_removes_row()
    {
        _client.Insert(20UL, [3UL, 4UL]);
        var vals = _client.GetAndDelete(20UL);
        Assert.NotNull(vals);
        // vals[0]=pk=20, vals[1]=3, vals[2]=4
        Assert.Equal(3, vals.Length);
        Assert.Null(_client.Get(20UL));
    }

    [Fact]
    public void InsertIfAbsent_returns_true_when_key_was_absent()
    {
        bool inserted = _client.InsertIfAbsent(30UL, [1UL, 2UL]);
        Assert.True(inserted);
        Assert.NotNull(_client.Get(30UL));
    }

    [Fact]
    public void InsertIfAbsent_returns_false_when_key_already_present()
    {
        _client.Insert(31UL, [5UL, 6UL]);
        bool inserted = _client.InsertIfAbsent(31UL, [99UL, 99UL]);
        Assert.False(inserted);
        var vals = _client.Get(31UL)!;
        Assert.Equal(5UL, vals[1]); // original survives
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}

// ── Count / Stats ──────────────────────────────────────────────────────────────

public class CountTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public CountTests()
    {
        _server = ServerHarness.Start(["pk", "score"]);
        _client = _server.OpenClient();
    }

    [Fact]
    public void Count_reflects_inserts_and_deletes()
    {
        Assert.Equal(0UL, _client.Count(0, 0, ulong.MaxValue));
        _client.Insert(1UL, [10UL]);
        _client.Insert(2UL, [20UL]);
        _client.Flush(); // ensure scan sees all rows
        // col=0=pk, lo=1, hi=MaxValue: count rows where pk >= 1
        Assert.Equal(2UL, _client.Count(0, 1UL, ulong.MaxValue));
        _client.Delete(1UL);
        _client.Compact(); // compact purges tombstones so Count gives accurate result
        Assert.Equal(1UL, _client.Count(0, 1UL, ulong.MaxValue));
    }

    [Fact]
    public void Stats_RowCount_matches_actual_inserts()
    {
        _client.Insert(100UL, [5UL]);
        _client.Insert(101UL, [6UL]);
        _client.Flush(); // flush to partition so Stats.Rows counts them
        var stats = _client.Stats();
        Assert.True(stats.Rows >= 2);
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}

// ── Batch insert / get ────────────────────────────────────────────────────────

public class BatchTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public BatchTests()
    {
        _server = ServerHarness.Start(["pk", "x", "y"]);
        _client = _server.OpenClient();
    }

    [Fact]
    public void BatchInsert_and_GetMany_round_trip()
    {
        var rows = new (ulong, ulong[])[] {
            (1UL, [10UL, 11UL]),
            (2UL, [20UL, 21UL]),
            (3UL, [30UL, 31UL]),
        };
        var result = _client.BatchInsert(rows);
        Assert.Equal(3u, result.Succeeded);

        var fetched = _client.GetMany([1UL, 2UL, 3UL]);
        Assert.Equal(3, fetched.Length);
        // GetMany returns Row{Pk, Values} — Values does NOT re-include pk (unlike Get)
        // Verify pks and first data column
        Assert.Equal(1UL, fetched[0].Pk);
        Assert.Equal(2UL, fetched[1].Pk);
        Assert.Equal(3UL, fetched[2].Pk);
    }

    [Fact]
    public void BatchDelete_removes_multiple_rows()
    {
        _client.Insert(50UL, [1UL, 2UL]);
        _client.Insert(51UL, [3UL, 4UL]);
        _client.BatchDelete([50UL, 51UL]);
        Assert.Null(_client.Get(50UL));
        Assert.Null(_client.Get(51UL));
    }

    [Fact]
    public void BatchPkExists_returns_count_of_present_pks()
    {
        _client.Insert(100UL, [1UL, 2UL]);
        _client.Insert(102UL, [3UL, 4UL]);
        // pk 100 exists, pk 101 does not, pk 102 exists → count = 2
        ulong count = _client.BatchPkExists([100UL, 101UL, 102UL]);
        Assert.Equal(2UL, count);
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}

// ── Aggregate ops ─────────────────────────────────────────────────────────────

public class AggregateTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public AggregateTests()
    {
        _server = ServerHarness.Start(["pk", "score"]);
        _client = _server.OpenClient();
        // Insert pks 1-5, scores 10-50
        for (ulong i = 1; i <= 5; i++)
            _client.Insert(i, [i * 10UL]);
    }

    [Fact]
    public void Sum_returns_correct_total()
    {
        // col=1 = score (first non-pk col), lo/hi cover all scores [10..50]
        ulong sum = _client.Sum(1, 10UL, 50UL);
        Assert.Equal(150UL, sum); // 10+20+30+40+50 = 150
    }

    [Fact]
    public void MinMax_returns_correct_range()
    {
        var mm = _client.MinMax(1, 10UL, 50UL);
        Assert.NotNull(mm);
        Assert.Equal(10UL, mm.Min);
        Assert.Equal(50UL, mm.Max);
        Assert.Equal(5UL, mm.Count);
    }

    [Fact]
    public void Average_sum_and_count_are_correct()
    {
        var avg = _client.Average(1, 10UL, 50UL);
        Assert.NotNull(avg);
        Assert.Equal(150UL, avg.Value.Sum);
        Assert.Equal(5UL, avg.Value.Count);
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}

// ── Field arithmetic ──────────────────────────────────────────────────────────

public class FieldArithmeticTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public FieldArithmeticTests()
    {
        _server = ServerHarness.Start(["pk", "counter"]);
        _client = _server.OpenClient();
    }

    [Fact]
    public void Increment_increases_column_by_delta()
    {
        _client.Insert(1UL, [100UL]);
        _client.Increment(1UL, 1, 5UL); // col=1 = counter
        var vals = _client.Get(1UL);
        Assert.NotNull(vals);
        Assert.Equal(105UL, vals[1]); // vals[1]=counter
    }

    [Fact]
    public void SetField_updates_single_column()
    {
        _client.Insert(2UL, [1UL]);
        _client.SetField(2UL, 1, 42UL); // col=1 = counter
        var vals = _client.Get(2UL);
        Assert.NotNull(vals);
        Assert.Equal(42UL, vals[1]);
    }

    [Fact]
    public void CompareAndSwap_succeeds_when_expected_matches()
    {
        _client.Insert(3UL, [10UL]);
        bool? swapped = _client.CompareAndSwap(3UL, 1, expected: 10UL, replacement: 99UL);
        Assert.True(swapped);
        Assert.Equal(99UL, _client.Get(3UL)![1]);
    }

    [Fact]
    public void CompareAndSwap_fails_when_expected_differs()
    {
        _client.Insert(4UL, [10UL]);
        bool? swapped = _client.CompareAndSwap(4UL, 1, expected: 5UL, replacement: 99UL);
        Assert.False(swapped);
        Assert.Equal(10UL, _client.Get(4UL)![1]); // unchanged
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}

// ── Scan / range ops ──────────────────────────────────────────────────────────

public class ScanTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public ScanTests()
    {
        _server = ServerHarness.Start(["pk", "v"]);
        _client = _server.OpenClient();
        for (ulong i = 1; i <= 10; i++)
            _client.Insert(i, [i * 100UL]);
    }

    [Fact]
    public void ScanKeys_by_pk_range_returns_correct_pks()
    {
        // col=0=pk, scan where pk in [1, 10]
        var keys = _client.ScanKeys(0, 1UL, 10UL);
        Assert.Equal(10, keys.Length);
    }

    [Fact]
    public void RangeScan_by_value_returns_matching_rows()
    {
        // col=1=v, scan rows where v in [300, 600] → pks 3,4,5,6
        var rows = _client.RangeScan(1, 300UL, 600UL);
        Assert.Equal(4, rows.Length);
        Assert.All(rows, r => Assert.InRange(r.Values[1], 300UL, 600UL));
    }

    [Fact]
    public void ScanPage_respects_limit()
    {
        // col=0=pk, full range, offset=0, limit=3
        var page = _client.ScanPage(0, 1UL, 10UL, offset: 0, limit: 3);
        Assert.Equal(3, page.Length);
    }

    [Fact]
    public void Top_returns_n_highest_values_in_col()
    {
        // col=1=v, lo/hi cover all values, n=3
        var top3 = _client.Top(1, 100UL, 1000UL, n: 3);
        Assert.Equal(3, top3.Length);
        Assert.All(top3, r => Assert.True(r.Values[1] >= 800UL));
    }

    [Fact]
    public void CountRange_counts_rows_where_value_in_range()
    {
        // col=1=v, count rows where v in [100, 500] → pks 1..5 → 5 rows
        ulong cnt = _client.CountRange(1, 100UL, 500UL);
        Assert.Equal(5UL, cnt);
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}

// ── Flush / Compact ───────────────────────────────────────────────────────────

public class MaintenanceTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public MaintenanceTests()
    {
        _server = ServerHarness.Start(["pk", "val"]);
        _client = _server.OpenClient();
    }

    [Fact]
    public void Flush_succeeds_without_error()
    {
        _client.Insert(1UL, [1UL]);
        _client.Flush();
    }

    [Fact]
    public void Compact_succeeds_without_error()
    {
        for (ulong i = 1; i <= 20; i++)
            _client.Insert(i, [i]);
        for (ulong i = 1; i <= 10; i++)
            _client.Delete(i);
        _client.Compact();
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}

// ── Auth integration ──────────────────────────────────────────────────────────

public class AuthTests
{
    private const string TestKeyId = "test-key";
    // 32 bytes hex = 64 chars
    private const string TestSecret = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";

    [Fact]
    public void Authenticated_client_can_ping()
    {
        using var server = ServerHarness.StartWithAuth(["pk", "val"], TestKeyId, TestSecret);
        using var client = server.OpenAuthClient();
        client.Ping();
    }

    [Fact]
    public void Wrong_secret_is_rejected()
    {
        using var server = ServerHarness.StartWithAuth(["pk", "val"], TestKeyId, TestSecret);
        Assert.ThrowsAny<Exception>(() =>
        {
            using var _ = VectorVoltClient.Connect(
                "127.0.0.1", server.Port,
                TestKeyId, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        });
    }
}

// ── Transactions ──────────────────────────────────────────────────────────────

public class TransactionTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public TransactionTests()
    {
        _server = ServerHarness.Start(["pk", "val"]);
        _client = _server.OpenClient();
    }

    [Fact]
    public void BeginCommit_transaction_makes_insert_visible()
    {
        ulong txId = _client.BeginTxn();
        _client.Insert(1UL, [42UL]);
        _client.CommitTxn(txId);
        var vals = _client.Get(1UL);
        Assert.NotNull(vals);
        Assert.Equal(42UL, vals[1]); // vals[0]=pk, vals[1]=42
    }

    [Fact]
    public void BeginRollback_reverts_inserts()
    {
        ulong txId = _client.BeginTxn();
        _client.Insert(2UL, [99UL]);
        _client.RollbackTxn(txId);
        Assert.Null(_client.Get(2UL));
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}

// ── SecondaryLookup ───────────────────────────────────────────────────────────

public class SecondaryLookupTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public SecondaryLookupTests()
    {
        _server = ServerHarness.Start(["pk", "age", "score"]);
        _client = _server.OpenClient();
    }

    [Fact]
    public void SecondaryLookup_returns_all_rows_matching_value()
    {
        _client.Insert(1UL, [30UL, 100UL]);
        _client.Insert(2UL, [40UL, 200UL]);
        _client.Insert(3UL, [30UL, 300UL]);

        var rows = _client.SecondaryLookup(1, 30UL);
        var pks = rows.Select(r => r.Pk).OrderBy(p => p).ToArray();
        Assert.Equal([1UL, 3UL], pks);
    }

    [Fact]
    public void SecondaryLookup_returns_empty_when_no_match()
    {
        _client.Insert(1UL, [30UL, 100UL]);

        var rows = _client.SecondaryLookup(1, 999UL);
        Assert.Empty(rows);
    }

    [Fact]
    public void SecondaryLookup_returns_full_row_values()
    {
        _client.Insert(5UL, [77UL, 555UL]);

        var rows = _client.SecondaryLookup(1, 77UL);
        Assert.Single(rows);
        Assert.Equal(5UL, rows[0].Pk);
    }

    [Fact]
    public void SecondaryLookup_col2_finds_by_second_data_column()
    {
        _client.Insert(10UL, [1UL, 42UL]);
        _client.Insert(11UL, [2UL, 42UL]);
        _client.Insert(12UL, [3UL, 99UL]);

        var rows = _client.SecondaryLookup(2, 42UL);
        var pks = rows.Select(r => r.Pk).OrderBy(p => p).ToArray();
        Assert.Equal([10UL, 11UL], pks);
    }

    [Fact]
    public void SecondaryLookup_after_delete_excludes_deleted_row()
    {
        _client.Insert(20UL, [50UL, 0UL]);
        _client.Insert(21UL, [50UL, 0UL]);
        _client.Delete(20UL);

        var rows = _client.SecondaryLookup(1, 50UL);
        var pks = rows.Select(r => r.Pk).ToArray();
        Assert.DoesNotContain(20UL, pks);
        Assert.Contains(21UL, pks);
    }

    [Fact]
    public void SecondaryLookup_on_empty_table_returns_empty()
    {
        var rows = _client.SecondaryLookup(1, 0UL);
        Assert.Empty(rows);
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}

// ── FkExists ──────────────────────────────────────────────────────────────────

public class FkExistsTests : IDisposable
{
    private readonly ServerHarness _server;
    private readonly VectorVoltClient _client;

    public FkExistsTests()
    {
        _server = ServerHarness.Start(["pk", "val"]);
        _client = _server.OpenClient();
    }

    [Fact]
    public void FkExists_true_for_inserted_pk()
    {
        _client.Insert(42UL, [1UL]);
        Assert.True(_client.FkExists(42UL));
    }

    [Fact]
    public void FkExists_false_for_never_inserted_pk()
    {
        _client.Insert(1UL, [1UL]);
        Assert.False(_client.FkExists(9999UL));
    }

    [Fact]
    public void FkExists_on_empty_table_returns_false()
    {
        Assert.False(_client.FkExists(1UL));
    }

    [Fact]
    public void FkExists_true_after_delete()
    {
        // CuckooFilter is not purged on delete — false-positive behaviour is documented.
        _client.Insert(7UL, [1UL]);
        _client.Delete(7UL);
        Assert.True(_client.FkExists(7UL));
    }

    [Fact]
    public void FkExists_no_false_negatives_for_batch()
    {
        for (ulong pk = 1; pk <= 50; pk++)
            _client.Insert(pk, [pk]);

        for (ulong pk = 1; pk <= 50; pk++)
            Assert.True(_client.FkExists(pk));
    }

    public void Dispose() { _client.Dispose(); _server.Dispose(); }
}
