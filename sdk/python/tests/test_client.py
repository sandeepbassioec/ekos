"""Layer 15 — Python SDK integration tests.

Each test class starts a live ``vvctl serve --multi --http-addr`` process,
drives it through :class:`VectorVoltClient`, and shuts it down afterward.
"""

import os
import socket
import subprocess
import tempfile
import time
import pathlib
import pytest

from vectorvolt import VectorVoltClient, TableNotFoundError, TableAlreadyExistsError

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
        self._dir = tempfile.mkdtemp(prefix="vv_py_test_")
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
    return harness.client


# ── list tables ───────────────────────────────────────────────────────────────

def test_list_tables_empty(client):
    assert client.list_tables() == []


# ── create + describe ─────────────────────────────────────────────────────────

def test_create_and_describe(client):
    client.create_table(["pk", "age", "score"], table="users")
    cols = client.describe_table(table="users")
    assert cols == ["pk", "age", "score"]


def test_create_appears_in_list(client):
    client.create_table(["pk", "val"], table="items")
    assert "items" in client.list_tables()


# ── drop ──────────────────────────────────────────────────────────────────────

def test_drop_removes_from_list(client):
    client.create_table(["pk", "v"], table="tmp")
    client.drop_table(table="tmp")
    assert "tmp" not in client.list_tables()


def test_drop_nonexistent_raises(client):
    with pytest.raises((TableNotFoundError, Exception)):
        client.drop_table(table="ghost")


# ── duplicate table ───────────────────────────────────────────────────────────

def test_create_duplicate_raises(client):
    client.create_table(["pk", "v"], table="dup")
    with pytest.raises((TableAlreadyExistsError, Exception)):
        client.create_table(["pk", "v"], table="dup")


# ── insert + get ──────────────────────────────────────────────────────────────

def test_insert_and_get(client):
    client.create_table(["pk", "age", "score"], table="t")
    client.use_table("t")
    row = client.insert(42, [42, 30, 99])
    assert row.pk == 42

    fetched = client.get(42)
    assert fetched is not None
    assert fetched.pk == 42
    assert fetched.values == [42, 30, 99]


def test_get_missing_returns_none(client):
    client.create_table(["pk", "v"], table="t2")
    client.use_table("t2")
    assert client.get(9999) is None


# ── delete ────────────────────────────────────────────────────────────────────

def test_delete_then_get_returns_none(client):
    client.create_table(["pk", "v"], table="t3")
    client.use_table("t3")
    client.insert(5, [5, 100])
    client.delete(5)
    assert client.get(5) is None


# ── scan ──────────────────────────────────────────────────────────────────────

def test_scan_range(client):
    client.create_table(["pk", "age", "score"], table="t4")
    client.use_table("t4")
    for pk in range(1, 6):
        client.insert(pk, [pk, pk * 10, 0])
    # col=1 (age), lo=20, hi=30 → pks 2 and 3
    rows = client.scan(1, 20, 30)
    pks = sorted(r.pk for r in rows)
    assert pks == [2, 3]


def test_scan_empty(client):
    client.create_table(["pk", "v"], table="t5")
    client.use_table("t5")
    assert client.scan(1, 0, 999) == []


# ── secondary lookup ──────────────────────────────────────────────────────────

def test_secondary_lookup(client):
    client.create_table(["pk", "age", "score"], table="t6")
    client.use_table("t6")
    for pk in range(1, 4):
        client.insert(pk, [pk, 55, 0])
    client.insert(4, [4, 99, 0])
    rows = client.secondary_lookup(1, 55)
    pks = sorted(r.pk for r in rows)
    assert pks == [1, 2, 3]


# ── multi-table isolation ─────────────────────────────────────────────────────

def test_multi_table_isolation(client):
    client.create_table(["pk", "val"], table="alpha")
    client.create_table(["pk", "val"], table="beta")
    client.insert(1, [1, 10], table="alpha")
    client.insert(1, [1, 20], table="beta")
    a = client.get(1, table="alpha")
    b = client.get(1, table="beta")
    assert a.values[1] == 10
    assert b.values[1] == 20


# ── use_table chaining ────────────────────────────────────────────────────────

def test_use_table_chaining(client):
    client.create_table(["pk", "v"], table="chain")
    c2 = client.use_table("chain")
    assert c2 is client  # returns self
    client.insert(7, [7, 777])
    assert client.get(7).values[1] == 777
