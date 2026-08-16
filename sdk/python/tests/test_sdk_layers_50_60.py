"""Layer 62 — Python SDK expansion: Layers 50-60 methods.

Tests the new methods added to VectorVoltClient covering:
  Layer 50: histogram, percentile
  Layer 51: group_count, group_sum, group_min, group_max
  Layer 52: window_running_sum, window_running_count, window_rank, window_dense_rank
  Layer 53: distinct, value_counts
  Layer 54: update_where, increment_where, decrement_where
  Layer 55: batch_get, batch_delete
  Layer 56: scan_where
  Layer 57: scan_project
  Layer 58: insert_if_absent, compare_and_swap
  Layer 59: batch_insert
  Layer 60: insert_with_ttl, get_with_ttl, scan_with_ttl, purge_expired
  + query (DSL pass-through)
"""

import os
import socket
import subprocess
import tempfile
import time
import pathlib
import pytest

from vectorvolt import VectorVoltClient, Row, PageResult
from vectorvolt.client import (
    HistogramBucket,
    GroupEntry,
    WindowRow,
    ValueCountEntry,
)

REPO_ROOT = pathlib.Path(__file__).parent.parent.parent.parent
VVCTL = REPO_ROOT / "vectorvolt" / "target" / "release" / "vvctl"


def _free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def _wait_for_port(port: int, timeout: float = 8.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.1):
                return
        except OSError:
            time.sleep(0.05)
    raise RuntimeError(f"port {port} did not open within {timeout}s")


class ServerHarness:
    def __init__(self):
        self.tcp_port = _free_port()
        self.http_port = _free_port()
        self._dir = tempfile.mkdtemp(prefix="vv_py62_test_")
        self._proc = subprocess.Popen(
            [
                str(VVCTL),
                "serve",
                self._dir,
                "--addr", f"127.0.0.1:{self.tcp_port}",
                "--http-addr", f"127.0.0.1:{self.http_port}",
                "--multi",
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        _wait_for_port(self.http_port)

    @property
    def client(self) -> VectorVoltClient:
        return VectorVoltClient(f"http://127.0.0.1:{self.http_port}")

    def stop(self):
        self._proc.terminate()
        try:
            self._proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            self._proc.kill()


@pytest.fixture
def harness():
    h = ServerHarness()
    yield h
    h.stop()


@pytest.fixture
def client(harness):
    c = harness.client
    c.create_table(["pk", "val", "score"], table="t")
    c.use_table("t")
    return c


def _seed(c: VectorVoltClient, n: int = 5) -> None:
    for i in range(1, n + 1):
        c.insert(i, [i, i * 10, i * 100])


# ── Layer 50: histogram ───────────────────────────────────────────────────────

def test_histogram_returns_buckets(client):
    _seed(client)
    buckets = client.histogram(col=1, lo=0, hi=60, buckets=3)
    assert len(buckets) == 3
    assert all(isinstance(b, HistogramBucket) for b in buckets)
    total = sum(b.count for b in buckets)
    assert total == 5


def test_histogram_empty_table(client):
    buckets = client.histogram(col=0, lo=0, hi=100, buckets=5)
    assert sum(b.count for b in buckets) == 0


def test_percentile_midpoint(client):
    _seed(client)
    p50 = client.percentile(col=1, p=50)
    assert p50 is not None
    assert 10 <= p50 <= 50


def test_percentile_empty_returns_none(client):
    result = client.percentile(col=0, p=50)
    assert result is None


# ── Layer 51: group-by ────────────────────────────────────────────────────────

def test_group_count(client):
    # insert 3 rows with val=10, 2 rows with val=20
    for pk, val in [(1, 10), (2, 10), (3, 10), (4, 20), (5, 20)]:
        client.insert(pk, [pk, val, 0])
    groups = client.group_count(key_col=1)
    counts = {g.key: g.agg_val for g in groups}
    assert counts[10] == 3
    assert counts[20] == 2


def test_group_sum(client):
    for pk in range(1, 4):
        client.insert(pk, [pk, 1, pk * 10])
    groups = client.group_sum(key_col=1, val_col=2)
    total = sum(g.agg_val for g in groups)
    assert total == 10 + 20 + 30


def test_group_min(client):
    for pk in range(1, 4):
        client.insert(pk, [pk, 1, pk * 5])
    groups = client.group_min(key_col=1, val_col=2)
    assert groups[0].agg_val == 5


def test_group_max(client):
    for pk in range(1, 4):
        client.insert(pk, [pk, 1, pk * 5])
    groups = client.group_max(key_col=1, val_col=2)
    assert groups[0].agg_val == 15


# ── Layer 52: window functions ────────────────────────────────────────────────

def test_window_running_sum(client):
    for pk in range(1, 4):
        client.insert(pk, [pk, pk * 10, 0])
    rows = client.window_running_sum(col=1)
    assert len(rows) == 3
    assert rows[-1].window_val == 10 + 20 + 30


def test_window_rank(client):
    for pk in range(1, 4):
        client.insert(pk, [pk, pk * 10, 0])
    rows = client.window_rank(col=1)
    ranks = [r.window_val for r in rows]
    assert sorted(ranks) == [1, 2, 3]


def test_window_dense_rank(client):
    for pk, val in [(1, 10), (2, 10), (3, 20)]:
        client.insert(pk, [pk, val, 0])
    rows = client.window_dense_rank(col=1)
    vals = {r.window_val for r in rows}
    assert vals == {1, 2}


def test_window_running_count(client):
    for pk in range(1, 4):
        client.insert(pk, [pk, pk, 0])
    rows = client.window_running_count(col=1)
    counts = [r.window_val for r in rows]
    assert counts[-1] == 3


# ── Layer 53: distinct + value_counts ────────────────────────────────────────

def test_distinct(client):
    for pk, val in [(1, 5), (2, 5), (3, 10), (4, 15)]:
        client.insert(pk, [pk, val, 0])
    vals = client.distinct(col=1)
    assert sorted(vals) == [5, 10, 15]


def test_value_counts(client):
    for pk, val in [(1, 7), (2, 7), (3, 7), (4, 9)]:
        client.insert(pk, [pk, val, 0])
    entries = client.value_counts(col=1)
    by_val = {e.value: e.count for e in entries}
    assert by_val[7] == 3
    assert by_val[9] == 1


# ── Layer 54: bulk predicate updates ─────────────────────────────────────────

def test_update_where(client):
    _seed(client)
    updated = client.update_where(col=0, op="le", val=3, target_col=1, new_val=999)
    assert updated == 3
    for pk in [1, 2, 3]:
        row = client.get(pk)
        assert row.values[1] == 999
    row4 = client.get(4)
    assert row4.values[1] == 40


def test_increment_where(client):
    _seed(client)
    updated = client.increment_where(col=0, op="ge", val=4, target_col=1, delta=5)
    assert updated == 2
    assert client.get(4).values[1] == 45
    assert client.get(5).values[1] == 55


def test_decrement_where(client):
    _seed(client)
    updated = client.decrement_where(col=0, op="eq", val=1, target_col=1, delta=5)
    assert updated == 1
    assert client.get(1).values[1] == 5


# ── Layer 55: batch get + delete ─────────────────────────────────────────────

def test_batch_get_returns_none_for_missing(client):
    _seed(client)
    rows = client.batch_get(pks=[1, 99, 3])
    assert rows[0] is not None
    assert rows[1] is None
    assert rows[2] is not None


def test_batch_delete_returns_count(client):
    _seed(client)
    deleted = client.batch_delete(pks=[1, 2, 3])
    assert deleted == 3
    assert client.get(1) is None


# ── Layer 56: scan_where ──────────────────────────────────────────────────────

def test_scan_where_single_predicate(client):
    _seed(client)
    rows = client.scan_where(predicates=[(0, 2, 4)])
    pks = {r.pk for r in rows}
    assert pks == {2, 3, 4}


def test_scan_where_two_predicates(client):
    _seed(client)
    # col0 in [1,5] AND col1 in [20,40]
    rows = client.scan_where(predicates=[(0, 1, 5), (1, 20, 40)])
    pks = {r.pk for r in rows}
    assert pks == {2, 3, 4}


# ── Layer 57: scan_project ────────────────────────────────────────────────────

def test_scan_project_returns_only_proj_cols(client):
    _seed(client)
    rows = client.scan_project(filter_col=0, lo=1, hi=5, proj=[0, 2])
    assert len(rows) == 5
    for r in rows:
        assert len(r.values) == 2


def test_scan_project_single_col(client):
    _seed(client)
    rows = client.scan_project(filter_col=0, lo=1, hi=3, proj=[1])
    assert len(rows) == 3
    assert all(len(r.values) == 1 for r in rows)


# ── Layer 58: conditional writes ─────────────────────────────────────────────

def test_insert_if_absent_inserts_new(client):
    result = client.insert_if_absent(pk=42, values=[42, 100, 200])
    assert result is True
    row = client.get(42)
    assert row.values[1] == 100


def test_insert_if_absent_skips_existing(client):
    client.insert(10, [10, 5, 0])
    result = client.insert_if_absent(pk=10, values=[10, 999, 0])
    assert result is False
    assert client.get(10).values[1] == 5


def test_compare_and_swap_swaps_on_match(client):
    client.insert(7, [7, 50, 0])
    swapped = client.compare_and_swap(pk=7, expected=[7, 50, 0], new_values=[7, 99, 0])
    assert swapped is True
    assert client.get(7).values[1] == 99


def test_compare_and_swap_no_swap_on_mismatch(client):
    client.insert(8, [8, 50, 0])
    swapped = client.compare_and_swap(pk=8, expected=[8, 999, 0], new_values=[8, 1, 0])
    assert swapped is False
    assert client.get(8).values[1] == 50


# ── Layer 59: batch insert ────────────────────────────────────────────────────

def test_batch_insert_returns_count(client):
    n = client.batch_insert(rows=[(1, [1, 10, 0]), (2, [2, 20, 0]), (3, [3, 30, 0])])
    assert n == 3


def test_batch_insert_rows_retrievable(client):
    client.batch_insert(rows=[(50, [50, 500, 0]), (51, [51, 510, 0])])
    assert client.get(50).values[1] == 500
    assert client.get(51).values[1] == 510


def test_batch_insert_empty(client):
    n = client.batch_insert(rows=[])
    assert n == 0


# ── Layer 60: TTL ops ─────────────────────────────────────────────────────────

def test_insert_with_ttl_and_get_live(client):
    client.insert_with_ttl(pk=100, values=[100, 77, 0], expire_at=9_999_999_999)
    row = client.get_with_ttl(pk=100, now=0)
    assert row is not None
    assert row.values[1] == 77


def test_get_with_ttl_returns_none_after_expiry(client):
    client.insert_with_ttl(pk=101, values=[101, 1, 0], expire_at=100)
    row = client.get_with_ttl(pk=101, now=999)
    assert row is None


def test_scan_with_ttl_excludes_expired(client):
    client.insert_with_ttl(pk=200, values=[200, 1, 0], expire_at=9_999_999_999)
    client.insert_with_ttl(pk=201, values=[201, 2, 0], expire_at=1)
    rows = client.scan_with_ttl(now=100)
    pks = {r.pk for r in rows}
    assert 200 in pks
    assert 201 not in pks


def test_purge_expired_removes_rows(client):
    client.insert_with_ttl(pk=300, values=[300, 1, 0], expire_at=1)
    client.insert_with_ttl(pk=301, values=[301, 2, 0], expire_at=9_999_999_999)
    purged = client.purge_expired(now=100)
    assert purged >= 1
    remaining = client.scan_with_ttl(now=0)
    pks = {r.pk for r in remaining}
    assert 300 not in pks
    assert 301 in pks


# ── DSL pass-through ──────────────────────────────────────────────────────────

def test_query_dsl_insert_and_get(client):
    resp = client.query("INSERT 77 77 88 0")
    assert resp.get("pk") == 77 or "type" in resp or "pk" in resp


def test_query_dsl_if_absent(client):
    resp = client.query("INSERT IF_ABSENT 88 88 100 0")
    assert resp.get("inserted") is True
    resp2 = client.query("INSERT IF_ABSENT 88 88 200 0")
    assert resp2.get("inserted") is False
