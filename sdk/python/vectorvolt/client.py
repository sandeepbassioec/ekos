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
