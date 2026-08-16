"""VectorVolt Python client — thin HTTP wrapper over the REST gateway."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional
import urllib.request
import urllib.error
import json


class VectorVoltError(Exception):
    """Base error for all VectorVolt SDK errors."""


class TableNotFoundError(VectorVoltError):
    """Raised when the requested table does not exist on the server."""


class TableAlreadyExistsError(VectorVoltError):
    """Raised when a table with that name already exists."""


@dataclass
class Row:
    pk: int
    values: list[int]


@dataclass
class PageResult:
    rows: list[Row]
    next_cursor: Optional[int]


@dataclass
class HistogramBucket:
    lo: int
    hi: int
    count: int


@dataclass
class GroupEntry:
    key: int
    agg_val: int


@dataclass
class WindowRow:
    pk: int
    window_val: int


@dataclass
class ValueCountEntry:
    value: int
    count: int


class VectorVoltClient:
    """HTTP client for a running VectorVolt server with the REST gateway enabled.

    Args:
        base_url: Server base URL, e.g. ``"http://127.0.0.1:7879"``.
        table:    Default table name for DML operations. Can be overridden
                  per call or changed with :meth:`use_table`.
        timeout:  Request timeout in seconds (default 5).

    Example::

        client = VectorVoltClient("http://127.0.0.1:7879", table="orders")
        client.create_table(["pk", "qty", "price"])
        client.insert(1, [1, 10, 999])
        row = client.get(1)
        print(row.values)  # [1, 10, 999]
    """

    def __init__(
        self,
        base_url: str,
        table: str = "default",
        timeout: float = 5.0,
    ) -> None:
        self._base = base_url.rstrip("/")
        self._table = table
        self._timeout = timeout

    # ── table selection ───────────────────────────────────────────────────────

    def use_table(self, name: str) -> "VectorVoltClient":
        """Switch the active table. Returns self for chaining."""
        self._table = name
        return self

    # ── DDL ───────────────────────────────────────────────────────────────────

    def create_table(
        self,
        columns: list[str],
        table: Optional[str] = None,
    ) -> None:
        """Create a table with the given column names (first = pk).

        Raises :exc:`TableAlreadyExistsError` if the table already exists.
        """
        name = table or self._table
        self._req("POST", f"/tables/{name}", body={"columns": columns}, expect=201)

    def drop_table(self, table: Optional[str] = None) -> None:
        """Drop a table. Raises :exc:`TableNotFoundError` if not found."""
        name = table or self._table
        self._req("DELETE", f"/tables/{name}")

    def list_tables(self) -> list[str]:
        """Return a sorted list of all table names on the server."""
        resp = self._req("GET", "/tables")
        return resp["tables"]

    def describe_table(self, table: Optional[str] = None) -> list[str]:
        """Return the column names of a table."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/schema")
        return resp["columns"]

    # ── DML ───────────────────────────────────────────────────────────────────

    def insert(
        self,
        pk: int,
        values: list[int],
        table: Optional[str] = None,
    ) -> Row:
        """Insert a row. ``values`` must include pk as the first element."""
        name = table or self._table
        resp = self._req("POST", f"/tables/{name}/rows/{pk}", body={"values": values})
        return Row(pk=resp["pk"], values=resp["values"])

    def get(self, pk: int, table: Optional[str] = None) -> Optional[Row]:
        """Get a row by primary key. Returns ``None`` if not found."""
        name = table or self._table
        try:
            resp = self._req("GET", f"/tables/{name}/rows/{pk}")
            return Row(pk=resp["pk"], values=resp["values"])
        except TableNotFoundError:
            return None

    def delete(self, pk: int, table: Optional[str] = None) -> bool:
        """Delete a row. Returns True on success."""
        name = table or self._table
        self._req("DELETE", f"/tables/{name}/rows/{pk}")
        return True

    def scan(
        self,
        col: int,
        lo: int,
        hi: int,
        table: Optional[str] = None,
    ) -> list[Row]:
        """Range scan: return rows where ``col`` value is in ``[lo, hi]``."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/scan?col={col}&lo={lo}&hi={hi}")
        return [Row(pk=r["pk"], values=r["values"]) for r in resp["rows"]]

    def secondary_lookup(
        self,
        col: int,
        val: int,
        table: Optional[str] = None,
    ) -> list[Row]:
        """Return all rows where column ``col`` equals ``val``."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/secondary?col={col}&val={val}")
        return [Row(pk=r["pk"], values=r["values"]) for r in resp["rows"]]

    # ── Layer 44 / 45: analytical + write/mutate ──────────────────────────────

    def upsert(
        self,
        pk: int,
        values: list[int],
        table: Optional[str] = None,
    ) -> None:
        """Insert or overwrite a row (upsert semantics)."""
        name = table or self._table
        self._req("PUT", f"/tables/{name}/rows/{pk}", body={"values": values})

    def update_field(
        self,
        pk: int,
        col: int,
        val: int,
        table: Optional[str] = None,
    ) -> None:
        """Update a single column of an existing row."""
        name = table or self._table
        self._req("PATCH", f"/tables/{name}/rows/{pk}/field", body={"col": col, "val": val})

    def delete_range(
        self,
        col: int,
        lo: int,
        hi: int,
        table: Optional[str] = None,
    ) -> int:
        """Delete all rows where ``col`` is in ``[lo, hi]``. Returns deleted count."""
        name = table or self._table
        resp = self._req("DELETE", f"/tables/{name}/rows/range?col={col}&lo={lo}&hi={hi}")
        return resp["count"]

    def count(
        self,
        col: int,
        lo: int,
        hi: int,
        table: Optional[str] = None,
    ) -> int:
        """Count rows where ``col`` is in ``[lo, hi]``."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/count?col={col}&lo={lo}&hi={hi}")
        return resp["count"]

    def sum_range(
        self,
        col: int,
        lo: int,
        hi: int,
        table: Optional[str] = None,
    ) -> Optional[int]:
        """Sum of ``col`` values for rows in ``[lo, hi]``. None if no rows match."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/sum?col={col}&lo={lo}&hi={hi}")
        return resp["value"]

    def min_range(
        self,
        col: int,
        lo: int,
        hi: int,
        table: Optional[str] = None,
    ) -> Optional[int]:
        """Minimum ``col`` value for rows in ``[lo, hi]``. None if no rows match."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/min?col={col}&lo={lo}&hi={hi}")
        return resp["value"]

    def max_range(
        self,
        col: int,
        lo: int,
        hi: int,
        table: Optional[str] = None,
    ) -> Optional[int]:
        """Maximum ``col`` value for rows in ``[lo, hi]``. None if no rows match."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/max?col={col}&lo={lo}&hi={hi}")
        return resp["value"]

    def avg(
        self,
        col: int,
        lo: int,
        hi: int,
        table: Optional[str] = None,
    ) -> Optional[int]:
        """Rounded average of ``col`` for rows in ``[lo, hi]``. None if empty."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/avg?col={col}&lo={lo}&hi={hi}")
        return resp["value"]

    def scan_sorted(
        self,
        col: int,
        lo: int,
        hi: int,
        sort: int,
        asc: bool = True,
        limit: int = 100,
        table: Optional[str] = None,
    ) -> list[Row]:
        """Scan rows filtered by ``col`` in ``[lo, hi]``, sorted by ``sort`` column."""
        name = table or self._table
        asc_int = 1 if asc else 0
        resp = self._req(
            "GET",
            f"/tables/{name}/scan/sorted?col={col}&lo={lo}&hi={hi}"
            f"&sort={sort}&asc={asc_int}&limit={limit}",
        )
        return [Row(pk=r["pk"], values=r["values"]) for r in resp["rows"]]

    def scan_page(
        self,
        col: int,
        lo: int,
        hi: int,
        after: int = 0,
        limit: int = 100,
        table: Optional[str] = None,
    ) -> PageResult:
        """Cursor-based page scan. Pass ``after=result.next_cursor`` for the next page."""
        name = table or self._table
        resp = self._req(
            "GET",
            f"/tables/{name}/scan/page?col={col}&lo={lo}&hi={hi}"
            f"&after={after}&limit={limit}",
        )
        rows = [Row(pk=r["pk"], values=r["values"]) for r in resp["rows"]]
        return PageResult(rows=rows, next_cursor=resp.get("next_cursor"))

    # ── Layer 50: histogram + percentile ─────────────────────────────────────

    def histogram(
        self,
        col: int,
        lo: int,
        hi: int,
        buckets: int,
        table: Optional[str] = None,
    ) -> list[HistogramBucket]:
        """Compute a frequency histogram over ``col`` in ``[lo, hi]``.

        Args:
            col:     Column index to histogram.
            lo:      Range lower bound (inclusive).
            hi:      Range upper bound (inclusive).
            buckets: Number of equal-width buckets.

        Returns a list of :class:`HistogramBucket` objects.
        """
        name = table or self._table
        resp = self._req(
            "GET",
            f"/tables/{name}/histogram?col={col}&lo={lo}&hi={hi}&buckets={buckets}",
        )
        return [HistogramBucket(lo=b["lo"], hi=b["hi"], count=b["count"]) for b in resp["buckets"]]

    def percentile(
        self,
        col: int,
        p: int,
        table: Optional[str] = None,
    ) -> Optional[int]:
        """Compute an approximate P-th percentile of ``col``. Returns None if table is empty."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/percentile?col={col}&p={p}")
        return resp.get("value")

    # ── Layer 51: group-by aggregates ────────────────────────────────────────

    def group_count(
        self,
        key_col: int,
        table: Optional[str] = None,
    ) -> list[GroupEntry]:
        """Count rows per unique value in ``key_col``."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/group/count?key_col={key_col}")
        return [GroupEntry(key=e["group_val"], agg_val=e["agg_val"]) for e in resp["entries"]]

    def group_sum(
        self,
        key_col: int,
        val_col: int,
        table: Optional[str] = None,
    ) -> list[GroupEntry]:
        """Sum ``val_col`` per unique value in ``key_col``."""
        name = table or self._table
        resp = self._req(
            "GET", f"/tables/{name}/group/sum?key_col={key_col}&val_col={val_col}"
        )
        return [GroupEntry(key=e["group_val"], agg_val=e["agg_val"]) for e in resp["entries"]]

    def group_min(
        self,
        key_col: int,
        val_col: int,
        table: Optional[str] = None,
    ) -> list[GroupEntry]:
        """Minimum of ``val_col`` per unique value in ``key_col``."""
        name = table or self._table
        resp = self._req(
            "GET", f"/tables/{name}/group/min?key_col={key_col}&val_col={val_col}"
        )
        return [GroupEntry(key=e["group_val"], agg_val=e["agg_val"]) for e in resp["entries"]]

    def group_max(
        self,
        key_col: int,
        val_col: int,
        table: Optional[str] = None,
    ) -> list[GroupEntry]:
        """Maximum of ``val_col`` per unique value in ``key_col``."""
        name = table or self._table
        resp = self._req(
            "GET", f"/tables/{name}/group/max?key_col={key_col}&val_col={val_col}"
        )
        return [GroupEntry(key=e["group_val"], agg_val=e["agg_val"]) for e in resp["entries"]]

    # ── Layer 52: window functions ────────────────────────────────────────────

    def window_running_sum(
        self,
        col: int,
        table: Optional[str] = None,
    ) -> list[WindowRow]:
        """Running sum of ``col`` over all rows ordered by pk."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/window/running_sum?col={col}")
        return [WindowRow(pk=r["pk"], window_val=r["window_val"]) for r in resp["rows"]]

    def window_running_count(
        self,
        col: int,
        table: Optional[str] = None,
    ) -> list[WindowRow]:
        """Running count over all rows ordered by pk."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/window/running_count?col={col}")
        return [WindowRow(pk=r["pk"], window_val=r["window_val"]) for r in resp["rows"]]

    def window_rank(
        self,
        col: int,
        table: Optional[str] = None,
    ) -> list[WindowRow]:
        """Rank of each row by ``col`` value (gaps on ties)."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/window/rank?col={col}")
        return [WindowRow(pk=r["pk"], window_val=r["window_val"]) for r in resp["rows"]]

    def window_dense_rank(
        self,
        col: int,
        table: Optional[str] = None,
    ) -> list[WindowRow]:
        """Dense rank of each row by ``col`` value (no gaps on ties)."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/window/dense_rank?col={col}")
        return [WindowRow(pk=r["pk"], window_val=r["window_val"]) for r in resp["rows"]]

    # ── Layer 53: distinct + value_counts ────────────────────────────────────

    def distinct(
        self,
        col: int,
        table: Optional[str] = None,
    ) -> list[int]:
        """Return sorted list of distinct values in ``col``."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/distinct?col={col}")
        return resp["values"]

    def value_counts(
        self,
        col: int,
        table: Optional[str] = None,
    ) -> list[ValueCountEntry]:
        """Return value → count map for ``col``."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/value_counts?col={col}")
        return [ValueCountEntry(value=e["value"], count=e["count"]) for e in resp["entries"]]

    # ── Layer 54: bulk predicate field updates ────────────────────────────────

    def update_where(
        self,
        col: int,
        op: str,
        val: int,
        target_col: int,
        new_val: int,
        table: Optional[str] = None,
    ) -> int:
        """Set ``target_col = new_val`` on every row where ``col op val``.

        ``op`` is one of: ``eq``, ``ne``, ``lt``, ``le``, ``gt``, ``ge``.
        Returns the number of rows updated.
        """
        name = table or self._table
        resp = self._req(
            "GET",
            f"/tables/{name}/update_where"
            f"?col={col}&op={op}&val={val}&target_col={target_col}&new_val={new_val}",
        )
        return resp["updated"]

    def increment_where(
        self,
        col: int,
        op: str,
        val: int,
        target_col: int,
        delta: int,
        table: Optional[str] = None,
    ) -> int:
        """Add ``delta`` to ``target_col`` on every row matching the predicate.
        Returns the number of rows updated."""
        name = table or self._table
        resp = self._req(
            "GET",
            f"/tables/{name}/increment_where"
            f"?col={col}&op={op}&val={val}&target_col={target_col}&delta={delta}",
        )
        return resp["updated"]

    def decrement_where(
        self,
        col: int,
        op: str,
        val: int,
        target_col: int,
        delta: int,
        table: Optional[str] = None,
    ) -> int:
        """Subtract ``delta`` from ``target_col`` on every row matching the predicate.
        Returns the number of rows updated."""
        name = table or self._table
        resp = self._req(
            "GET",
            f"/tables/{name}/decrement_where"
            f"?col={col}&op={op}&val={val}&target_col={target_col}&delta={delta}",
        )
        return resp["updated"]

    # ── Layer 55: batch get + batch delete ────────────────────────────────────

    def batch_get(
        self,
        pks: list[int],
        table: Optional[str] = None,
    ) -> list[Optional[Row]]:
        """Fetch multiple rows by primary key. Missing rows appear as ``None``."""
        name = table or self._table
        resp = self._req("POST", f"/tables/{name}/batch/get", body={"pks": pks})
        result = []
        for r in resp["rows"]:
            if r is None:
                result.append(None)
            else:
                result.append(Row(pk=r["pk"], values=r["values"]))
        return result

    def batch_delete(
        self,
        pks: list[int],
        table: Optional[str] = None,
    ) -> int:
        """Delete multiple rows by primary key. Returns the count of rows deleted."""
        name = table or self._table
        resp = self._req("POST", f"/tables/{name}/batch/delete", body={"pks": pks})
        return resp["deleted"]

    # ── Layer 56: multi-predicate scan ────────────────────────────────────────

    def scan_where(
        self,
        predicates: list[tuple[int, int, int]],
        limit: Optional[int] = None,
        table: Optional[str] = None,
    ) -> list[Row]:
        """Scan rows matching up to 3 range predicates.

        Each predicate is a ``(col, lo, hi)`` tuple.
        All predicates are ANDed together.
        """
        name = table or self._table
        if not predicates:
            raise VectorVoltError("scan_where requires at least one predicate")
        parts = []
        for i, (col, lo, hi) in enumerate(predicates[:3]):
            parts.append(f"col{i}={col}&lo{i}={lo}&hi{i}={hi}")
        qs = "&".join(parts)
        if limit is not None:
            qs += f"&limit={limit}"
        resp = self._req("GET", f"/tables/{name}/scan_where?{qs}")
        return [Row(pk=r["pk"], values=r["values"]) for r in resp["rows"]]

    # ── Layer 57: column projection scan ─────────────────────────────────────

    def scan_project(
        self,
        filter_col: int,
        lo: int,
        hi: int,
        proj: list[int],
        table: Optional[str] = None,
    ) -> list[Row]:
        """Scan rows filtered by ``filter_col`` in ``[lo, hi]``, returning only
        the columns listed in ``proj``.  The returned ``Row.values`` contain only
        the projected columns in the order given."""
        name = table or self._table
        proj_str = ",".join(str(c) for c in proj)
        resp = self._req(
            "GET",
            f"/tables/{name}/scan/project"
            f"?filter_col={filter_col}&lo={lo}&hi={hi}&proj={proj_str}",
        )
        return [Row(pk=r["pk"], values=r["values"]) for r in resp["rows"]]

    # ── Layer 58: conditional writes ──────────────────────────────────────────

    def insert_if_absent(
        self,
        pk: int,
        values: list[int],
        table: Optional[str] = None,
    ) -> bool:
        """Insert a row only if ``pk`` does not already exist.

        Returns ``True`` if the row was inserted, ``False`` if it already existed.
        """
        name = table or self._table
        resp = self._req(
            "POST",
            f"/tables/{name}/rows/{pk}/if_absent",
            body={"values": values},
        )
        return resp["inserted"]

    def compare_and_swap(
        self,
        pk: int,
        expected: list[int],
        new_values: list[int],
        table: Optional[str] = None,
    ) -> bool:
        """Atomically replace a row when its current values match ``expected``.

        Returns ``True`` if the swap happened, ``False`` if the values did not match.
        """
        name = table or self._table
        resp = self._req(
            "POST",
            f"/tables/{name}/rows/{pk}/cas",
            body={"expected": expected, "new_values": new_values},
        )
        return resp["swapped"]

    # ── Layer 59: batch insert ────────────────────────────────────────────────

    def batch_insert(
        self,
        rows: list[tuple[int, list[int]]],
        table: Optional[str] = None,
    ) -> int:
        """Insert multiple rows in one request.

        Each row is a ``(pk, values)`` tuple.
        Returns the number of rows inserted.
        """
        name = table or self._table
        body = {"rows": [{"pk": pk, "values": vals} for pk, vals in rows]}
        resp = self._req("POST", f"/tables/{name}/batch/insert", body=body)
        return resp["inserted"]

    # ── Layer 60: TTL operations ──────────────────────────────────────────────

    def insert_with_ttl(
        self,
        pk: int,
        values: list[int],
        expire_at: int,
        table: Optional[str] = None,
    ) -> None:
        """Insert a row that automatically expires at ``expire_at`` (Unix nanoseconds)."""
        name = table or self._table
        self._req(
            "POST",
            f"/tables/{name}/rows/{pk}/ttl",
            body={"values": values, "expire_at": expire_at},
        )

    def get_with_ttl(
        self,
        pk: int,
        now: int,
        table: Optional[str] = None,
    ) -> Optional[Row]:
        """Get a row, returning ``None`` if it has expired (``expire_at <= now``)."""
        name = table or self._table
        try:
            resp = self._req("GET", f"/tables/{name}/get_with_ttl/{pk}?now={now}")
            return Row(pk=resp["pk"], values=resp["values"])
        except TableNotFoundError:
            return None

    def scan_with_ttl(
        self,
        now: int,
        table: Optional[str] = None,
    ) -> list[Row]:
        """Return all rows that have not yet expired at timestamp ``now``."""
        name = table or self._table
        resp = self._req("GET", f"/tables/{name}/scan_with_ttl?now={now}")
        return [Row(pk=r["pk"], values=r["values"]) for r in resp["rows"]]

    def purge_expired(
        self,
        now: int,
        table: Optional[str] = None,
    ) -> int:
        """Remove all rows whose ``expire_at <= now``. Returns the count purged."""
        name = table or self._table
        resp = self._req("POST", f"/tables/{name}/purge_expired?now={now}")
        return resp["purged"]

    # ── DSL pass-through ──────────────────────────────────────────────────────

    def query(
        self,
        dsl: str,
        table: Optional[str] = None,
    ) -> dict:
        """Send a raw DSL statement to the server and return the parsed JSON response.

        This covers every operation supported by the DSL, including Layers 50-60
        statements::

            client.query("HISTOGRAM col=0 lo=0 hi=100 BUCKETS=10")
            client.query("CAS 42 EXPECTED 42 5 NEW 42 6")
            client.query("BATCH INSERT ROW 1 1 10 ROW 2 2 20")
        """
        name = table or self._table
        url = f"{self._base}/tables/{name}/query"
        data = dsl.encode()
        headers = {"Content-Type": "text/plain", "Accept": "application/json"}
        req = urllib.request.Request(url, data=data, headers=headers, method="POST")
        try:
            with urllib.request.urlopen(req, timeout=self._timeout) as resp:
                raw = resp.read()
                return json.loads(raw) if raw else {}
        except urllib.error.HTTPError as e:
            raw = e.read().decode(errors="replace")
            raise VectorVoltError(f"HTTP {e.code}: {raw}") from e
        except urllib.error.URLError as e:
            raise VectorVoltError(f"Connection error: {e.reason}") from e

    # ── internals ─────────────────────────────────────────────────────────────

    def _req(
        self,
        method: str,
        path: str,
        body: Optional[dict] = None,
        expect: int = 200,
    ) -> dict:
        url = self._base + path
        data = json.dumps(body).encode() if body is not None else None
        headers = {"Content-Type": "application/json", "Accept": "application/json"}
        req = urllib.request.Request(url, data=data, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req, timeout=self._timeout) as resp:
                raw = resp.read()
                result = json.loads(raw) if raw else {}
                status = resp.status
                if expect != 200 and status != expect:
                    raise VectorVoltError(f"HTTP {status}: {raw.decode()}")
                return result
        except urllib.error.HTTPError as e:
            raw = e.read().decode(errors="replace")
            status = e.code
            if status == 404:
                raise TableNotFoundError(raw) from e
            if status == 409:
                raise TableAlreadyExistsError(raw) from e
            raise VectorVoltError(f"HTTP {status}: {raw}") from e
        except urllib.error.URLError as e:
            raise VectorVoltError(f"Connection error: {e.reason}") from e
