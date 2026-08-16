"""Layer 45 — Python SDK expansion: analytical + write/mutate methods.

Tests the 10 new methods added to VectorVoltClient:
  upsert, update_field, delete_range, count, sum_range, min_range,
  max_range, avg, scan_sorted, scan_page
"""

import os
import socket
import subprocess
import tempfile
import time
import pathlib
import pytest

from vectorvolt import VectorVoltClient, Row, PageResult

# ── helpers ───────────────────────────────────────────────────────────────────

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
        self._dir = tempfile.mkdtemp(prefix="vv_py45_test_")
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


# ── UPSERT ────────────────────────────────────────────────────────────────────

def test_upsert_inserts_new_row(client):
    client.upsert(1, [1, 42, 0])
    row = client.get(1)
    assert row is not None
    assert row.values[1] == 42


def test_upsert_overwrites_existing(client):
    client.insert(1, [1, 10, 0])
    client.upsert(1, [1, 99, 0])
    row = client.get(1)
    assert row.values[1] == 99


# ── UPDATE FIELD ──────────────────────────────────────────────────────────────

def test_update_field_changes_one_column(client):
    client.insert(1, [1, 5, 0])
    client.update_field(1, col=1, val=77)
    row = client.get(1)
    assert row.values[1] == 77


def test_update_field_does_not_change_other_columns(client):
    client.insert(1, [1, 5, 99])
    client.update_field(1, col=1, val=77)
    row = client.get(1)
    assert row.values[2] == 99


# ── DELETE RANGE ──────────────────────────────────────────────────────────────

def test_delete_range_returns_count(client):
    _seed(client, 5)
    deleted = client.delete_range(col=0, lo=2, hi=4)
    assert deleted == 3


def test_delete_range_removes_rows(client):
    _seed(client, 5)
    client.delete_range(col=0, lo=2, hi=4)
    assert client.get(1) is not None
    assert client.get(3) is None
    assert client.get(5) is not None


# ── COUNT ─────────────────────────────────────────────────────────────────────

def test_count_matching_rows(client):
    _seed(client, 5)
    assert client.count(col=1, lo=10, hi=30) == 3


def test_count_zero_when_no_match(client):
    client.insert(1, [1, 99, 0])
    assert client.count(col=1, lo=0, hi=10) == 0


# ── SUM / MIN / MAX / AVG ─────────────────────────────────────────────────────

def test_sum_range(client):
    for i in range(1, 5):
        client.insert(i, [i, i, 0])
    assert client.sum_range(col=1, lo=1, hi=4) == 10


def test_min_range(client):
    client.insert(1, [1, 50, 0])
    client.insert(2, [2, 10, 0])
    client.insert(3, [3, 80, 0])
    assert client.min_range(col=1, lo=1, hi=100) == 10


def test_max_range(client):
    client.insert(1, [1, 50, 0])
    client.insert(2, [2, 10, 0])
    client.insert(3, [3, 80, 0])
    assert client.max_range(col=1, lo=1, hi=100) == 80


def test_avg_returns_rounded_mean(client):
    for i in [10, 20, 30]:
        pk = i // 10
        client.insert(pk, [pk, i, 0])
    assert client.avg(col=1, lo=1, hi=100) == 20


def test_avg_none_when_empty(client):
    assert client.avg(col=1, lo=1, hi=100) is None


# ── SCAN SORTED ───────────────────────────────────────────────────────────────

def test_scan_sorted_ascending(client):
    client.insert(1, [1, 30, 0])
    client.insert(2, [2, 10, 0])
    client.insert(3, [3, 20, 0])
    rows = client.scan_sorted(col=0, lo=1, hi=3, sort=1, asc=True, limit=10)
    vals = [r.values[1] for r in rows]
    assert vals == [10, 20, 30]


# ── SCAN PAGE ─────────────────────────────────────────────────────────────────

def test_scan_page_returns_page_result(client):
    _seed(client, 5)
    result = client.scan_page(col=0, lo=1, hi=5, after=0, limit=3)
    assert isinstance(result, PageResult)
    assert len(result.rows) == 3
    assert result.next_cursor is not None


def test_scan_page_second_page_exhausts(client):
    _seed(client, 5)
    page1 = client.scan_page(col=0, lo=1, hi=5, after=0, limit=3)
    page2 = client.scan_page(col=0, lo=1, hi=5, after=page1.next_cursor, limit=3)
    assert len(page2.rows) == 2
    assert page2.next_cursor is None
