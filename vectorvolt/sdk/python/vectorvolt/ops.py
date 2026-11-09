"""Wrappers for the remaining request opcodes of the VectorVolt wire protocol.

One method per opcode the server dispatch (vectorvolt-net/src/server.rs) handles,
laid out exactly as protocol.rs ``Request::encode_payload``. Mixed into
``VectorVoltClient``; every method needs a connected client with a table selected
(``use_table`` / ``create_table``) unless noted.
"""

from __future__ import annotations

import struct
from typing import Optional, Sequence, Tuple

from . import wire
from .wire import ProtocolError, Reader


def _u32(v: int) -> bytes:
    return struct.pack("<I", v)


def _u64(v: int) -> bytes:
    return struct.pack("<Q", v)


def _u32s(vs: Sequence[int]) -> bytes:
    vs = list(vs)
    return struct.pack("<I", len(vs)) + b"".join(struct.pack("<I", v) for v in vs)


def _pairs(ps: Sequence[Tuple[int, int]]) -> bytes:
    ps = list(ps)
    return struct.pack("<I", len(ps)) + b"".join(struct.pack("<IQ", c, v) for c, v in ps)


def _rows(rows: Sequence[Tuple[int, Sequence[int]]]) -> bytes:
    rows = list(rows)
    return struct.pack("<I", len(rows)) + b"".join(_u64(pk) + wire.w_u64s(v) for pk, v in rows)


def _affected(f: wire.Frame) -> int:
    r = Reader(f.payload)
    v = r.u64()
    r.finish()
    return v


def _row(f: wire.Frame) -> list:
    r = Reader(f.payload)
    v = r.u64s()
    r.finish()
    return v


def _rows_of(f: wire.Frame) -> list:
    r = Reader(f.payload)
    n = r.u32()
    if n > (len(f.payload) - r.at) // 4:
        raise ProtocolError("declared more rows than the payload holds")
    rows = [r.u64s() for _ in range(n)]
    r.finish()
    return rows


def _exists(f: wire.Frame) -> bool:
    r = Reader(f.payload)
    b = r.take(1)[0]
    r.finish()
    return b != 0


def _minmax(f: wire.Frame) -> tuple:
    r = Reader(f.payload)
    out = (r.u64(), r.u64(), r.u64())
    r.finish()
    return out


def _keys(f: wire.Frame) -> list:
    r = Reader(f.payload)
    n = r.u32()
    if n > (len(f.payload) - r.at) // 8:
        raise ProtocolError("declared more keys than the payload holds")
    out = [r.u64() for _ in range(n)]
    r.finish()
    return out


def _rowset(f: wire.Frame) -> list:
    r = Reader(f.payload)
    n = r.u32()
    if n > (len(f.payload) - r.at) // 4:
        raise ProtocolError("declared more entries than the payload holds")
    out = [(r.u64(), r.u64s()) for _ in range(n)]
    r.finish()
    return out


def _batch(f: wire.Frame) -> tuple:
    r = Reader(f.payload)
    out = (r.u32(), r.u32())
    r.finish()
    return out


def _schema(f: wire.Frame) -> list:
    r = Reader(f.payload)
    out = r.strings()
    r.finish()
    return out


class OpsMixin:
    """Typed wrappers for every remaining request opcode."""

    def _expect(self, opcode: int, payload: bytes, *expected: int) -> wire.Frame:  # provided by the client
        raise NotImplementedError

    def secondary_lookup(self, col_idx: int, value: int) -> list:
        """SecondaryLookup. protocol.rs Request::SecondaryLookup, opcode 11, payload:
        u32 col_idx, u64 value. Returns the matching rows."""
        f = self._expect(wire.SECONDARY_LOOKUP, _u32(col_idx) + _u64(value), wire.ROWS)
        return _rows_of(f)

    def fk_exists(self, value: int) -> bool:
        """FkExists. protocol.rs Request::FkExists, opcode 12, payload: u64 value.
        Returns whether the key exists."""
        f = self._expect(wire.FK_EXISTS, _u64(value), wire.EXISTS)
        return _exists(f)

    def upsert(self, pk: int, values: Sequence[int]) -> int:
        """Insert or replace a row (delete existing, then insert). protocol.rs
        Request::Upsert, opcode 22, payload: u64 pk, u64s values. Returns the affected /
        result count."""
        f = self._expect(wire.UPSERT, _u64(pk) + wire.w_u64s(values), wire.OK)
        return _affected(f)

    def get_many(self, pks: Sequence[int]) -> list:
        """Fetch multiple rows by primary key in one round trip. protocol.rs
        Request::GetMany, opcode 23, payload: u64s pks. Returns the found rows as (pk,
        values) entries; missing keys are omitted."""
        f = self._expect(wire.GET_MANY, wire.w_u64s(pks), wire.ROW_SET)
        return _rowset(f)

    def update(self, pk: int, values: Sequence[int]) -> int:
        """Update an existing row; error if the primary key is not found. protocol.rs
        Request::Update, opcode 24, payload: u64 pk, u64s values. Returns the affected /
        result count."""
        f = self._expect(wire.UPDATE, _u64(pk) + wire.w_u64s(values), wire.OK)
        return _affected(f)

    def truncate(self) -> int:
        """Delete all rows from the selected table. protocol.rs Request::Truncate,
        opcode 25, payload: (empty payload). Returns the affected / result count."""
        f = self._expect(wire.TRUNCATE, b"", wire.OK)
        return _affected(f)

    def count(self, col: int, lo: int, hi: int) -> int:
        """Count rows where column `col` falls in `[lo, hi]`. protocol.rs
        Request::Count, opcode 26, payload: u32 col, u64 lo, u64 hi. Returns the
        affected / result count."""
        f = self._expect(wire.COUNT, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def batch_upsert(self, rows: Sequence[Tuple[int, Sequence[int]]]) -> int:
        """Insert-or-replace multiple rows in a single round trip. protocol.rs
        Request::BatchUpsert, opcode 27, payload: rows rows. Returns the affected /
        result count."""
        f = self._expect(wire.BATCH_UPSERT, _rows(rows), wire.OK)
        return _affected(f)

    def batch_update(self, rows: Sequence[Tuple[int, Sequence[int]]]) -> tuple:
        """Update multiple existing rows in a single round trip. protocol.rs
        Request::BatchUpdate, opcode 28, payload: rows rows. Returns (succeeded,
        skipped)."""
        f = self._expect(wire.BATCH_UPDATE, _rows(rows), wire.BATCH_RESULT)
        return _batch(f)

    def describe_table(self) -> list:
        """Return the column schema for the currently selected table. protocol.rs
        Request::DescribeTable, opcode 29, payload: (empty payload). Returns the column
        names in schema order."""
        f = self._expect(wire.DESCRIBE_TABLE, b"", wire.SCHEMA)
        return _schema(f)

    def table_exists(self, name: str) -> bool:
        """Check whether a named table exists in the catalog. protocol.rs
        Request::TableExists, opcode 30, payload: string name. Returns whether the key
        exists."""
        f = self._expect(wire.TABLE_EXISTS, wire.w_string(name), wire.EXISTS)
        return _exists(f)

    def scan_keys(self, col: int, lo: int, hi: int, limit: int) -> list:
        """Range scan returning only primary keys (no values). protocol.rs
        Request::ScanKeys, opcode 31, payload: u32 col, u64 lo, u64 hi, u32 limit.
        Returns the primary keys in scan order."""
        f = self._expect(wire.SCAN_KEYS, _u32(col) + _u64(lo) + _u64(hi) + _u32(limit), wire.KEYS)
        return _keys(f)

    def min_max(self, col: int, lo: int, hi: int) -> Optional[tuple]:
        """Return the min, max, and count of a column value over a pk range. protocol.rs
        Request::MinMax, opcode 32, payload: u32 col, u64 lo, u64 hi. Returns (min, max,
        count), or None when the range is empty (NotFound)."""
        f = self._expect(wire.MIN_MAX, _u32(col) + _u64(lo) + _u64(hi), wire.MIN_MAX_RESULT, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _minmax(f)

    def sum(self, col: int, lo: int, hi: int) -> int:
        """Sum a column's values over a pk range (saturating at u64::MAX). protocol.rs
        Request::Sum, opcode 33, payload: u32 col, u64 lo, u64 hi. Returns the affected
        / result count."""
        f = self._expect(wire.SUM, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def get_and_delete(self, pk: int) -> Optional[list]:
        """Fetch a row by PK and atomically delete it. protocol.rs
        Request::GetAndDelete, opcode 34, payload: u64 pk. Returns the row values, or
        None when the server answers NotFound."""
        f = self._expect(wire.GET_AND_DELETE, _u64(pk), wire.ROW, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _row(f)

    def increment(self, pk: int, col: int, delta: int) -> Optional[int]:
        """Atomically add `delta` to column `col` of the row at `pk` (saturating at
        u64::MAX). protocol.rs Request::Increment, opcode 35, payload: u64 pk, u32 col,
        u64 delta. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.INCREMENT, _u64(pk) + _u32(col) + _u64(delta), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def compare_and_swap(self, pk: int, col: int, expected: int, replacement: int) -> Optional[int]:
        """Conditional update: set `values[col]` to `replacement` iff it currently
        equals `expected`. protocol.rs Request::CompareAndSwap, opcode 36, payload: u64
        pk, u32 col, u64 expected, u64 replacement. Returns the result, or None when the
        server answers NotFound."""
        f = self._expect(wire.COMPARE_AND_SWAP, _u64(pk) + _u32(col) + _u64(expected) + _u64(replacement), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def set_field(self, pk: int, col: int, value: int) -> Optional[int]:
        """Set a single column of an existing row without providing all other column
        values. protocol.rs Request::SetField, opcode 37, payload: u64 pk, u32 col, u64
        value. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.SET_FIELD, _u64(pk) + _u32(col) + _u64(value), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def insert_if_absent(self, pk: int, values: Sequence[int]) -> int:
        """Insert a row only if the primary key is not already present. protocol.rs
        Request::InsertIfAbsent, opcode 38, payload: u64 pk, u64s values. Returns the
        affected / result count."""
        f = self._expect(wire.INSERT_IF_ABSENT, _u64(pk) + wire.w_u64s(values), wire.OK)
        return _affected(f)

    def delete_range(self, col: int, lo: int, hi: int) -> int:
        """Delete all rows where column `col` falls in `[lo, hi]`. protocol.rs
        Request::DeleteRange, opcode 39, payload: u32 col, u64 lo, u64 hi. Returns the
        affected / result count."""
        f = self._expect(wire.DELETE_RANGE, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def pop_min(self, col: int, lo: int, hi: int) -> Optional[list]:
        """Fetch and atomically delete the row with the minimum column value in `[lo,
        hi]`. protocol.rs Request::PopMin, opcode 40, payload: u32 col, u64 lo, u64 hi.
        Returns the row values, or None when the server answers NotFound."""
        f = self._expect(wire.POP_MIN, _u32(col) + _u64(lo) + _u64(hi), wire.ROW, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _row(f)

    def pop_max(self, col: int, lo: int, hi: int) -> Optional[list]:
        """Fetch and atomically delete the row with the maximum column value in `[lo,
        hi]`. protocol.rs Request::PopMax, opcode 41, payload: u32 col, u64 lo, u64 hi.
        Returns the row values, or None when the server answers NotFound."""
        f = self._expect(wire.POP_MAX, _u32(col) + _u64(lo) + _u64(hi), wire.ROW, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _row(f)

    def multi_set(self, pk: int, pairs: Sequence[Tuple[int, int]]) -> Optional[int]:
        """Set multiple `(col, value)` pairs on an existing row atomically. protocol.rs
        Request::MultiSet, opcode 42, payload: u64 pk, pairs pairs. Returns the result,
        or None when the server answers NotFound."""
        f = self._expect(wire.MULTI_SET, _u64(pk) + _pairs(pairs), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def decrement_clamp(self, pk: int, col: int, delta: int) -> Optional[int]:
        """Subtract `delta` from a column, clamping at 0 (saturating_sub). protocol.rs
        Request::DecrementClamp, opcode 43, payload: u64 pk, u32 col, u64 delta. Returns
        the result, or None when the server answers NotFound."""
        f = self._expect(wire.DECREMENT_CLAMP, _u64(pk) + _u32(col) + _u64(delta), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def get_or_insert(self, pk: int, values: Sequence[int]) -> list:
        """Return existing row or insert-and-return provided values. protocol.rs
        Request::GetOrInsert, opcode 44, payload: u64 pk, u64s values. Returns the row
        values."""
        f = self._expect(wire.GET_OR_INSERT, _u64(pk) + wire.w_u64s(values), wire.ROW)
        return _row(f)

    def batch_set_field(self, col: int, value: int, pks: Sequence[int]) -> int:
        """Set the same column to the same value across many rows in one round trip.
        protocol.rs Request::BatchSetField, opcode 45, payload: u32 col, u64 value, u64s
        pks. Returns the affected / result count."""
        f = self._expect(wire.BATCH_SET_FIELD, _u32(col) + _u64(value) + wire.w_u64s(pks), wire.OK)
        return _affected(f)

    def top(self, col: int, lo: int, hi: int, n: int) -> list:
        """Return the top-N rows by column value (highest first). protocol.rs
        Request::Top, opcode 46, payload: u32 col, u64 lo, u64 hi, u32 n. Returns the
        matching rows."""
        f = self._expect(wire.TOP, _u32(col) + _u64(lo) + _u64(hi) + _u32(n), wire.ROWS)
        return _rows_of(f)

    def bottom(self, col: int, lo: int, hi: int, n: int) -> list:
        """Return the bottom-N rows by column value (lowest first). protocol.rs
        Request::Bottom, opcode 47, payload: u32 col, u64 lo, u64 hi, u32 n. Returns the
        matching rows."""
        f = self._expect(wire.BOTTOM, _u32(col) + _u64(lo) + _u64(hi) + _u32(n), wire.ROWS)
        return _rows_of(f)

    def pk_exists(self, pk: int) -> bool:
        """Exact primary-key existence check (not probabilistic). protocol.rs
        Request::PkExists, opcode 48, payload: u64 pk. Returns whether the key exists."""
        f = self._expect(wire.PK_EXISTS, _u64(pk), wire.EXISTS)
        return _exists(f)

    def increment_multi(self, pk: int, pairs: Sequence[Tuple[int, int]]) -> Optional[list]:
        """Atomically increment multiple `(col, delta)` pairs on one row (saturating at
        u64::MAX). protocol.rs Request::IncrementMulti, opcode 49, payload: u64 pk,
        pairs pairs. Returns the row values, or None when the server answers NotFound."""
        f = self._expect(wire.INCREMENT_MULTI, _u64(pk) + _pairs(pairs), wire.ROW, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _row(f)

    def copy_row(self, src_pk: int, dst_pk: int) -> Optional[int]:
        """Copy the row at `src_pk` to `dst_pk` (insert-or-replace). protocol.rs
        Request::CopyRow, opcode 50, payload: u64 src_pk, u64 dst_pk. Returns the
        result, or None when the server answers NotFound."""
        f = self._expect(wire.COPY_ROW, _u64(src_pk) + _u64(dst_pk), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def swap_field(self, pk1: int, pk2: int, col: int) -> Optional[int]:
        """Swap the value of column `col` between `pk1` and `pk2`. protocol.rs
        Request::SwapField, opcode 51, payload: u64 pk1, u64 pk2, u32 col. Returns the
        result, or None when the server answers NotFound."""
        f = self._expect(wire.SWAP_FIELD, _u64(pk1) + _u64(pk2) + _u32(col), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def clamp(self, pk: int, col: int, lo: int, hi: int) -> Optional[int]:
        """Clamp column `col` of row `pk` into `[lo, hi]`. protocol.rs Request::Clamp,
        opcode 52, payload: u64 pk, u32 col, u64 lo, u64 hi. Returns the result, or None
        when the server answers NotFound."""
        f = self._expect(wire.CLAMP, _u64(pk) + _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def average(self, col: int, lo: int, hi: int) -> Optional[list]:
        """Return `[sum, count]` for column `col` over the scan range `[lo, hi]`.
        protocol.rs Request::Average, opcode 53, payload: u32 col, u64 lo, u64 hi.
        Returns the row values, or None when the server answers NotFound."""
        f = self._expect(wire.AVERAGE, _u32(col) + _u64(lo) + _u64(hi), wire.ROW, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _row(f)

    def delete_if(self, pk: int, col: int, value: int) -> Optional[int]:
        """Delete the row at `pk` only when `values[col] == value`. protocol.rs
        Request::DeleteIf, opcode 54, payload: u64 pk, u32 col, u64 value. Returns the
        result, or None when the server answers NotFound."""
        f = self._expect(wire.DELETE_IF, _u64(pk) + _u32(col) + _u64(value), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def set_field_if(self, pk: int, col: int, expected: int, new_value: int) -> Optional[int]:
        """Set `values[col]` to `new_value` only if it currently equals `expected`.
        protocol.rs Request::SetFieldIf, opcode 55, payload: u64 pk, u32 col, u64
        expected, u64 new_value. Returns the result, or None when the server answers
        NotFound."""
        f = self._expect(wire.SET_FIELD_IF, _u64(pk) + _u32(col) + _u64(expected) + _u64(new_value), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def rank_of(self, pk: int, col: int, lo: int, hi: int) -> Optional[int]:
        """Return the 0-based rank of `pk`'s column `col` value within `[lo, hi]`.
        protocol.rs Request::RankOf, opcode 56, payload: u64 pk, u32 col, u64 lo, u64
        hi. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.RANK_OF, _u64(pk) + _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def percentile(self, col: int, lo: int, hi: int, pct: int) -> Optional[int]:
        """Return the value at the Nth percentile of column `col` in `[lo, hi]`.
        protocol.rs Request::Percentile, opcode 57, payload: u32 col, u64 lo, u64 hi,
        u32 pct. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.PERCENTILE, _u32(col) + _u64(lo) + _u64(hi) + _u32(pct), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def sum_fields(self, pk: int, cols: Sequence[int]) -> Optional[int]:
        """Sum multiple column values of a single row. protocol.rs Request::SumFields,
        opcode 58, payload: u64 pk, u32s cols. Returns the result, or None when the
        server answers NotFound."""
        f = self._expect(wire.SUM_FIELDS, _u64(pk) + _u32s(cols), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def max_field(self, pk: int, cols: Sequence[int]) -> Optional[int]:
        """Return the maximum across multiple column values of a single row. protocol.rs
        Request::MaxField, opcode 59, payload: u64 pk, u32s cols. Returns the result, or
        None when the server answers NotFound."""
        f = self._expect(wire.MAX_FIELD, _u64(pk) + _u32s(cols), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def filter_scan(self, scan_col: int, lo: int, hi: int, filter_col: int, filter_val: int) -> list:
        """Range scan with an additional equality filter on a second column. protocol.rs
        Request::FilterScan, opcode 60, payload: u32 scan_col, u64 lo, u64 hi, u32
        filter_col, u64 filter_val. Returns the matching rows."""
        f = self._expect(wire.FILTER_SCAN, _u32(scan_col) + _u64(lo) + _u64(hi) + _u32(filter_col) + _u64(filter_val), wire.ROWS)
        return _rows_of(f)

    def batch_increment_field(self, col: int, delta: int, pks: Sequence[int]) -> int:
        """Increment the same column by `delta` across many rows in one round trip.
        protocol.rs Request::BatchIncrementField, opcode 61, payload: u32 col, u64
        delta, u64s pks. Returns the affected / result count."""
        f = self._expect(wire.BATCH_INCREMENT_FIELD, _u32(col) + _u64(delta) + wire.w_u64s(pks), wire.OK)
        return _affected(f)

    def count_distinct(self, col: int, lo: int, hi: int) -> int:
        """Count distinct values of column `col` in scan range `[lo, hi]`. protocol.rs
        Request::CountDistinct, opcode 62, payload: u32 col, u64 lo, u64 hi. Returns the
        affected / result count."""
        f = self._expect(wire.COUNT_DISTINCT, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def median(self, col: int, lo: int, hi: int) -> Optional[int]:
        """Return the median value of column `col` in `[lo, hi]`. protocol.rs
        Request::Median, opcode 63, payload: u32 col, u64 lo, u64 hi. Returns the
        result, or None when the server answers NotFound."""
        f = self._expect(wire.MEDIAN, _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def batch_delete_if(self, col: int, value: int, pks: Sequence[int]) -> int:
        """Delete each pk in the list only if `values[col] == value`. protocol.rs
        Request::BatchDeleteIf, opcode 64, payload: u32 col, u64 value, u64s pks.
        Returns the affected / result count."""
        f = self._expect(wire.BATCH_DELETE_IF, _u32(col) + _u64(value) + wire.w_u64s(pks), wire.OK)
        return _affected(f)

    def batch_pk_exists(self, pks: Sequence[int]) -> int:
        """Check multiple primary keys for existence in one round trip. protocol.rs
        Request::BatchPkExists, opcode 65, payload: u64s pks. Returns the affected /
        result count."""
        f = self._expect(wire.BATCH_PK_EXISTS, wire.w_u64s(pks), wire.OK)
        return _affected(f)

    def field_between(self, scan_col: int, lo: int, hi: int, filter_col: int, filter_lo: int, filter_hi: int) -> list:
        """Range scan on `scan_col` in `[lo, hi]` keeping rows where `filter_col` in
        `[filter_lo, filter_hi]`. protocol.rs Request::FieldBetween, opcode 66, payload:
        u32 scan_col, u64 lo, u64 hi, u32 filter_col, u64 filter_lo, u64 filter_hi.
        Returns the matching rows."""
        f = self._expect(wire.FIELD_BETWEEN, _u32(scan_col) + _u64(lo) + _u64(hi) + _u32(filter_col) + _u64(filter_lo) + _u64(filter_hi), wire.ROWS)
        return _rows_of(f)

    def scalar_min(self, col: int, lo: int, hi: int) -> Optional[int]:
        """Minimum value of column `col` over scan range `[lo, hi]`. protocol.rs
        Request::ScalarMin, opcode 67, payload: u32 col, u64 lo, u64 hi. Returns the
        result, or None when the server answers NotFound."""
        f = self._expect(wire.SCALAR_MIN, _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def scalar_max(self, col: int, lo: int, hi: int) -> Optional[int]:
        """Maximum value of column `col` over scan range `[lo, hi]`. protocol.rs
        Request::ScalarMax, opcode 68, payload: u32 col, u64 lo, u64 hi. Returns the
        result, or None when the server answers NotFound."""
        f = self._expect(wire.SCALAR_MAX, _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def multi_get(self, pk: int, cols: Sequence[int]) -> Optional[list]:
        """Return only the listed columns of a single row. protocol.rs
        Request::MultiGet, opcode 69, payload: u64 pk, u32s cols. Returns the row
        values, or None when the server answers NotFound."""
        f = self._expect(wire.MULTI_GET, _u64(pk) + _u32s(cols), wire.ROW, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _row(f)

    def increment_or_insert(self, pk: int, col: int, delta: int, values: Sequence[int]) -> int:
        """Add `delta` to column `col` if `pk` exists; otherwise insert `values` (with
        `values[col]` as initial). protocol.rs Request::IncrementOrInsert, opcode 70,
        payload: u64 pk, u32 col, u64 delta, u64s values. Returns the affected / result
        count."""
        f = self._expect(wire.INCREMENT_OR_INSERT, _u64(pk) + _u32(col) + _u64(delta) + wire.w_u64s(values), wire.OK)
        return _affected(f)

    def zero_field(self, col: int, lo: int, hi: int) -> int:
        """Set column `col` to 0 for every row in scan range `[lo, hi]`. protocol.rs
        Request::ZeroField, opcode 71, payload: u32 col, u64 lo, u64 hi. Returns the
        affected / result count."""
        f = self._expect(wire.ZERO_FIELD, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def scan_page(self, col: int, lo: int, hi: int, offset: int, limit: int) -> list:
        """Paginated range scan: skip `offset` rows then return up to `limit`.
        protocol.rs Request::ScanPage, opcode 72, payload: u32 col, u64 lo, u64 hi, u32
        offset, u32 limit. Returns the matching rows."""
        f = self._expect(wire.SCAN_PAGE, _u32(col) + _u64(lo) + _u64(hi) + _u32(offset) + _u32(limit), wire.ROWS)
        return _rows_of(f)

    def sum_field(self, col: int, lo: int, hi: int) -> int:
        """SumField. protocol.rs Request::SumField, opcode 73, payload: u32 col, u64 lo,
        u64 hi. Returns the affected / result count."""
        f = self._expect(wire.SUM_FIELD, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def batch_get(self, pks: Sequence[int]) -> list:
        """BatchGet. protocol.rs Request::BatchGet, opcode 74, payload: u64s pks.
        Returns the matching rows."""
        f = self._expect(wire.BATCH_GET, wire.w_u64s(pks), wire.ROWS)
        return _rows_of(f)

    def set_field_range(self, col: int, value: int, lo: int, hi: int) -> int:
        """SetFieldRange. protocol.rs Request::SetFieldRange, opcode 75, payload: u32
        col, u64 value, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.SET_FIELD_RANGE, _u32(col) + _u64(value) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def max_row(self, col: int, lo: int, hi: int) -> Optional[list]:
        """MaxRow. protocol.rs Request::MaxRow, opcode 76, payload: u32 col, u64 lo, u64
        hi. Returns the row values, or None when the server answers NotFound."""
        f = self._expect(wire.MAX_ROW, _u32(col) + _u64(lo) + _u64(hi), wire.ROW, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _row(f)

    def min_row(self, col: int, lo: int, hi: int) -> Optional[list]:
        """MinRow. protocol.rs Request::MinRow, opcode 77, payload: u32 col, u64 lo, u64
        hi. Returns the row values, or None when the server answers NotFound."""
        f = self._expect(wire.MIN_ROW, _u32(col) + _u64(lo) + _u64(hi), wire.ROW, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _row(f)

    def increment_range(self, col: int, delta: int, lo: int, hi: int) -> int:
        """IncrementRange. protocol.rs Request::IncrementRange, opcode 78, payload: u32
        col, u64 delta, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.INCREMENT_RANGE, _u32(col) + _u64(delta) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def delete_min(self, col: int, lo: int, hi: int) -> Optional[int]:
        """DeleteMin. protocol.rs Request::DeleteMin, opcode 79, payload: u32 col, u64
        lo, u64 hi. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.DELETE_MIN, _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def delete_max(self, col: int, lo: int, hi: int) -> Optional[int]:
        """DeleteMax. protocol.rs Request::DeleteMax, opcode 80, payload: u32 col, u64
        lo, u64 hi. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.DELETE_MAX, _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def unique_count(self, col: int, lo: int, hi: int) -> int:
        """UniqueCount. protocol.rs Request::UniqueCount, opcode 81, payload: u32 col,
        u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.UNIQUE_COUNT, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def top_n(self, col: int, n: int, lo: int, hi: int) -> list:
        """TopN. protocol.rs Request::TopN, opcode 82, payload: u32 col, u32 n, u64 lo,
        u64 hi. Returns the matching rows."""
        f = self._expect(wire.TOP_N, _u32(col) + _u32(n) + _u64(lo) + _u64(hi), wire.ROWS)
        return _rows_of(f)

    def bottom_n(self, col: int, n: int, lo: int, hi: int) -> list:
        """BottomN. protocol.rs Request::BottomN, opcode 83, payload: u32 col, u32 n,
        u64 lo, u64 hi. Returns the matching rows."""
        f = self._expect(wire.BOTTOM_N, _u32(col) + _u32(n) + _u64(lo) + _u64(hi), wire.ROWS)
        return _rows_of(f)

    def move_field(self, src_pk: int, dst_pk: int, col: int) -> Optional[int]:
        """MoveField. protocol.rs Request::MoveField, opcode 84, payload: u64 src_pk,
        u64 dst_pk, u32 col. Returns the result, or None when the server answers
        NotFound."""
        f = self._expect(wire.MOVE_FIELD, _u64(src_pk) + _u64(dst_pk) + _u32(col), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def count_if(self, col: int, value: int, lo: int, hi: int) -> int:
        """CountIf. protocol.rs Request::CountIf, opcode 85, payload: u32 col, u64
        value, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.COUNT_IF, _u32(col) + _u64(value) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def set_if_greater(self, col: int, value: int, lo: int, hi: int) -> int:
        """SetIfGreater. protocol.rs Request::SetIfGreater, opcode 86, payload: u32 col,
        u64 value, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.SET_IF_GREATER, _u32(col) + _u64(value) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def set_if_less(self, col: int, value: int, lo: int, hi: int) -> int:
        """SetIfLess. protocol.rs Request::SetIfLess, opcode 87, payload: u32 col, u64
        value, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.SET_IF_LESS, _u32(col) + _u64(value) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def abs_diff(self, pk1: int, pk2: int, col: int) -> Optional[int]:
        """AbsDiff. protocol.rs Request::AbsDiff, opcode 88, payload: u64 pk1, u64 pk2,
        u32 col. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.ABS_DIFF, _u64(pk1) + _u64(pk2) + _u32(col), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def swap_fields(self, pk1: int, pk2: int, col: int) -> Optional[int]:
        """SwapFields. protocol.rs Request::SwapFields, opcode 89, payload: u64 pk1, u64
        pk2, u32 col. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.SWAP_FIELDS, _u64(pk1) + _u64(pk2) + _u32(col), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def copy_range(self, src_col: int, dst_col: int, lo: int, hi: int) -> int:
        """CopyRange. protocol.rs Request::CopyRange, opcode 90, payload: u32 src_col,
        u32 dst_col, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.COPY_RANGE, _u32(src_col) + _u32(dst_col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def propagate_max(self, col: int, lo: int, hi: int) -> int:
        """PropagateMax. protocol.rs Request::PropagateMax, opcode 91, payload: u32 col,
        u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.PROPAGATE_MAX, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def propagate_min(self, col: int, lo: int, hi: int) -> int:
        """PropagateMin. protocol.rs Request::PropagateMin, opcode 92, payload: u32 col,
        u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.PROPAGATE_MIN, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def clamp_range(self, col: int, min_val: int, max_val: int, lo: int, hi: int) -> int:
        """ClampRange. protocol.rs Request::ClampRange, opcode 93, payload: u32 col, u64
        min_val, u64 max_val, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.CLAMP_RANGE, _u32(col) + _u64(min_val) + _u64(max_val) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def batch_conditional_set(self, col: int, expected: int, new_value: int, lo: int, hi: int) -> int:
        """BatchConditionalSet. protocol.rs Request::BatchConditionalSet, opcode 94,
        payload: u32 col, u64 expected, u64 new_value, u64 lo, u64 hi. Returns the
        affected / result count."""
        f = self._expect(wire.BATCH_CONDITIONAL_SET, _u32(col) + _u64(expected) + _u64(new_value) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def window_rank(self, pk: int, col: int, lo: int, hi: int) -> Optional[int]:
        """WindowRank. protocol.rs Request::WindowRank, opcode 95, payload: u64 pk, u32
        col, u64 lo, u64 hi. Returns the result, or None when the server answers
        NotFound."""
        f = self._expect(wire.WINDOW_RANK, _u64(pk) + _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def window_pct(self, pk: int, col: int, lo: int, hi: int) -> Optional[int]:
        """WindowPct. protocol.rs Request::WindowPct, opcode 96, payload: u64 pk, u32
        col, u64 lo, u64 hi. Returns the result, or None when the server answers
        NotFound."""
        f = self._expect(wire.WINDOW_PCT, _u64(pk) + _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def sum_product(self, col_a: int, col_b: int, lo: int, hi: int) -> int:
        """SumProduct. protocol.rs Request::SumProduct, opcode 97, payload: u32 col_a,
        u32 col_b, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.SUM_PRODUCT, _u32(col_a) + _u32(col_b) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def nth_value(self, col: int, n: int, lo: int, hi: int) -> Optional[int]:
        """NthValue. protocol.rs Request::NthValue, opcode 98, payload: u32 col, u32 n,
        u64 lo, u64 hi. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.NTH_VALUE, _u32(col) + _u32(n) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def dense_rank_win(self, pk: int, col: int, lo: int, hi: int) -> Optional[int]:
        """DenseRankWin. protocol.rs Request::DenseRankWin, opcode 99, payload: u64 pk,
        u32 col, u64 lo, u64 hi. Returns the result, or None when the server answers
        NotFound."""
        f = self._expect(wire.DENSE_RANK_WIN, _u64(pk) + _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def set_field_many(self, col: int, value: int, pks: Sequence[int]) -> int:
        """SetFieldMany. protocol.rs Request::SetFieldMany, opcode 100, payload: u32
        col, u64 value, u64s pks. Returns the affected / result count."""
        f = self._expect(wire.SET_FIELD_MANY, _u32(col) + _u64(value) + wire.w_u64s(pks), wire.OK)
        return _affected(f)

    def multi_column_sum(self, pks: Sequence[int], col: int) -> int:
        """MultiColumnSum. protocol.rs Request::MultiColumnSum, opcode 101, payload:
        u64s pks, u32 col. Returns the affected / result count."""
        f = self._expect(wire.MULTI_COLUMN_SUM, wire.w_u64s(pks) + _u32(col), wire.OK)
        return _affected(f)

    def field_histogram(self, col: int, bucket_size: int, lo: int, hi: int) -> list:
        """FieldHistogram. protocol.rs Request::FieldHistogram, opcode 102, payload: u32
        col, u64 bucket_size, u64 lo, u64 hi. Returns the matching rows."""
        f = self._expect(wire.FIELD_HISTOGRAM, _u32(col) + _u64(bucket_size) + _u64(lo) + _u64(hi), wire.ROWS)
        return _rows_of(f)

    def z_score_filter(self, col: int, lo: int, hi: int) -> list:
        """ZScoreFilter. protocol.rs Request::ZScoreFilter, opcode 103, payload: u32
        col, u64 lo, u64 hi. Returns the matching rows."""
        f = self._expect(wire.Z_SCORE_FILTER, _u32(col) + _u64(lo) + _u64(hi), wire.ROWS)
        return _rows_of(f)

    def running_sum(self, col: int, lo: int, hi: int) -> list:
        """RunningSum. protocol.rs Request::RunningSum, opcode 104, payload: u32 col,
        u64 lo, u64 hi. Returns the matching rows."""
        f = self._expect(wire.RUNNING_SUM, _u32(col) + _u64(lo) + _u64(hi), wire.ROWS)
        return _rows_of(f)

    def floor_div(self, col: int, divisor: int, lo: int, hi: int) -> int:
        """FloorDiv. protocol.rs Request::FloorDiv, opcode 105, payload: u32 col, u64
        divisor, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.FLOOR_DIV, _u32(col) + _u64(divisor) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def mul_field(self, col: int, factor: int, lo: int, hi: int) -> int:
        """MulField. protocol.rs Request::MulField, opcode 106, payload: u32 col, u64
        factor, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.MUL_FIELD, _u32(col) + _u64(factor) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def set_field_add(self, pk: int, col: int, addend: int) -> Optional[int]:
        """SetFieldAdd. protocol.rs Request::SetFieldAdd, opcode 107, payload: u64 pk,
        u32 col, u64 addend. Returns the result, or None when the server answers
        NotFound."""
        f = self._expect(wire.SET_FIELD_ADD, _u64(pk) + _u32(col) + _u64(addend), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def set_field_mul(self, pk: int, col: int, factor: int) -> Optional[int]:
        """SetFieldMul. protocol.rs Request::SetFieldMul, opcode 108, payload: u64 pk,
        u32 col, u64 factor. Returns the result, or None when the server answers
        NotFound."""
        f = self._expect(wire.SET_FIELD_MUL, _u64(pk) + _u32(col) + _u64(factor), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def swap_pk_fields(self, pk: int, col_a: int, col_b: int) -> Optional[int]:
        """SwapPkFields. protocol.rs Request::SwapPkFields, opcode 109, payload: u64 pk,
        u32 col_a, u32 col_b. Returns the result, or None when the server answers
        NotFound."""
        f = self._expect(wire.SWAP_PK_FIELDS, _u64(pk) + _u32(col_a) + _u32(col_b), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def count_range(self, col: int, lo: int, hi: int) -> int:
        """CountRange. protocol.rs Request::CountRange, opcode 110, payload: u32 col,
        u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.COUNT_RANGE, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def sum_range(self, col: int, lo: int, hi: int) -> int:
        """SumRange. protocol.rs Request::SumRange, opcode 111, payload: u32 col, u64
        lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.SUM_RANGE, _u32(col) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def min_range(self, col: int, lo: int, hi: int) -> Optional[int]:
        """MinRange. protocol.rs Request::MinRange, opcode 112, payload: u32 col, u64
        lo, u64 hi. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.MIN_RANGE, _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def max_range(self, col: int, lo: int, hi: int) -> Optional[int]:
        """MaxRange. protocol.rs Request::MaxRange, opcode 113, payload: u32 col, u64
        lo, u64 hi. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.MAX_RANGE, _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def avg_range(self, col: int, lo: int, hi: int) -> Optional[int]:
        """AvgRange. protocol.rs Request::AvgRange, opcode 114, payload: u32 col, u64
        lo, u64 hi. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.AVG_RANGE, _u32(col) + _u64(lo) + _u64(hi), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def increment_field(self, pk: int, col: int) -> Optional[int]:
        """IncrementField. protocol.rs Request::IncrementField, opcode 115, payload: u64
        pk, u32 col. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.INCREMENT_FIELD, _u64(pk) + _u32(col), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def decrement_field(self, pk: int, col: int) -> Optional[int]:
        """DecrementField. protocol.rs Request::DecrementField, opcode 116, payload: u64
        pk, u32 col. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.DECREMENT_FIELD, _u64(pk) + _u32(col), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def set_min(self, pk: int, col: int, value: int) -> Optional[int]:
        """SetMin. protocol.rs Request::SetMin, opcode 117, payload: u64 pk, u32 col,
        u64 value. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.SET_MIN, _u64(pk) + _u32(col) + _u64(value), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def set_max(self, pk: int, col: int, value: int) -> Optional[int]:
        """SetMax. protocol.rs Request::SetMax, opcode 118, payload: u64 pk, u32 col,
        u64 value. Returns the result, or None when the server answers NotFound."""
        f = self._expect(wire.SET_MAX, _u64(pk) + _u32(col) + _u64(value), wire.OK, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        return _affected(f)

    def xor_field(self, col: int, mask: int, lo: int, hi: int) -> int:
        """XorField. protocol.rs Request::XorField, opcode 119, payload: u32 col, u64
        mask, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.XOR_FIELD, _u32(col) + _u64(mask) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def and_field(self, col: int, mask: int, lo: int, hi: int) -> int:
        """AndField. protocol.rs Request::AndField, opcode 120, payload: u32 col, u64
        mask, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.AND_FIELD, _u32(col) + _u64(mask) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def or_field(self, col: int, mask: int, lo: int, hi: int) -> int:
        """OrField. protocol.rs Request::OrField, opcode 121, payload: u32 col, u64
        mask, u64 lo, u64 hi. Returns the affected / result count."""
        f = self._expect(wire.OR_FIELD, _u32(col) + _u64(mask) + _u64(lo) + _u64(hi), wire.OK)
        return _affected(f)

    def batch_get_field(self, col: int, pks: Sequence[int]) -> list:
        """BatchGetField. protocol.rs Request::BatchGetField, opcode 122, payload: u32
        col, u64s pks. Returns the matching rows."""
        f = self._expect(wire.BATCH_GET_FIELD, _u32(col) + wire.w_u64s(pks), wire.ROWS)
        return _rows_of(f)
