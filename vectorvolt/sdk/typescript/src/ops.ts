/**
 * Wrappers for the remaining request opcodes of the VectorVolt wire protocol: one
 * method per opcode the server dispatch (vectorvolt-net/src/server.rs) handles, laid
 * out exactly as protocol.rs Request::encode_payload. Every method needs a connected
 * client with a table selected unless noted. Extended by VectorVoltClient.
 */

import { Op, ProtocolError, Reader, Writer } from "./wire.js";
import type { Frame } from "./wire.js";

export type Row = bigint[];

const EMPTY = new Uint8Array(0);

function putU32s(w: Writer, vs: readonly number[]): void {
  w.u32(vs.length);
  for (const v of vs) w.u32(v);
}

function putPairs(w: Writer, ps: readonly (readonly [number, bigint])[]): void {
  w.u32(ps.length);
  for (const [c, v] of ps) w.u32(c).u64(v);
}

function putRows(w: Writer, rows: readonly (readonly [bigint, readonly bigint[]])[]): void {
  w.u32(rows.length);
  for (const [pk, v] of rows) w.u64(pk).u64s(v);
}

function affected(f: Frame): bigint {
  const r = new Reader(f.payload);
  const v = r.u64();
  r.finish();
  return v;
}

function parseRow(f: Frame): Row {
  const r = new Reader(f.payload);
  const v = r.u64s();
  r.finish();
  return v;
}

function parseRows(f: Frame): Row[] {
  const r = new Reader(f.payload);
  const n = r.u32();
  if (n > Math.floor(r.remaining() / 4)) throw new ProtocolError("declared more rows than the payload holds");
  const rows: Row[] = [];
  for (let i = 0; i < n; i++) rows.push(r.u64s());
  r.finish();
  return rows;
}

function parseExists(f: Frame): boolean {
  const r = new Reader(f.payload);
  const b = r.u8();
  r.finish();
  return b !== 0;
}

function parseMinMax(f: Frame): { min: bigint; max: bigint; count: bigint } {
  const r = new Reader(f.payload);
  const out = { min: r.u64(), max: r.u64(), count: r.u64() };
  r.finish();
  return out;
}

function parseKeys(f: Frame): bigint[] {
  const r = new Reader(f.payload);
  const n = r.u32();
  if (n > Math.floor(r.remaining() / 8)) throw new ProtocolError("declared more keys than the payload holds");
  const out: bigint[] = [];
  for (let i = 0; i < n; i++) out.push(r.u64());
  r.finish();
  return out;
}

function parseRowSet(f: Frame): [bigint, Row][] {
  const r = new Reader(f.payload);
  const n = r.u32();
  if (n > Math.floor(r.remaining() / 4)) throw new ProtocolError("declared more entries than the payload holds");
  const out: [bigint, Row][] = [];
  for (let i = 0; i < n; i++) {
    const pk = r.u64();
    out.push([pk, r.u64s()]);
  }
  r.finish();
  return out;
}

function parseBatch(f: Frame): { succeeded: number; skipped: number } {
  const r = new Reader(f.payload);
  const out = { succeeded: r.u32(), skipped: r.u32() };
  r.finish();
  return out;
}

function parseSchema(f: Frame): string[] {
  const r = new Reader(f.payload);
  const out = r.strings();
  r.finish();
  return out;
}

export abstract class OpsBase {
  protected abstract call(opcode: number, payload: Uint8Array, ...expect: number[]): Promise<Frame>;

  /** Compact. protocol.rs Request::Compact, opcode 7, payload: (empty payload). Returns
   * the affected / result count. */
  async compact(): Promise<bigint> {
    return affected(await this.call(Op.Compact, EMPTY, Op.Ok));
  }

  /** SecondaryLookup. protocol.rs Request::SecondaryLookup, opcode 11, payload: u32
   * col_idx, u64 value. Returns the matching rows. */
  async secondaryLookup(colIdx: number, value: bigint): Promise<Row[]> {
    const w = new Writer();
    w.u32(colIdx);
    w.u64(value);
    return parseRows(await this.call(Op.SecondaryLookup, w.build(), Op.Rows));
  }

  /** FkExists. protocol.rs Request::FkExists, opcode 12, payload: u64 value. Returns
   * whether the key exists. */
  async fkExists(value: bigint): Promise<boolean> {
    const w = new Writer();
    w.u64(value);
    return parseExists(await this.call(Op.FkExists, w.build(), Op.Exists));
  }

  /** Insert or replace a row (delete existing, then insert). protocol.rs
   * Request::Upsert, opcode 22, payload: u64 pk, u64s values. Returns the affected /
   * result count. */
  async upsert(pk: bigint, values: readonly bigint[]): Promise<bigint> {
    const w = new Writer();
    w.u64(pk);
    w.u64s(values);
    return affected(await this.call(Op.Upsert, w.build(), Op.Ok));
  }

  /** Fetch multiple rows by primary key in one round trip. protocol.rs
   * Request::GetMany, opcode 23, payload: u64s pks. Returns the found rows as (pk,
   * values) entries; missing keys are omitted. */
  async getMany(pks: readonly bigint[]): Promise<[bigint, Row][]> {
    const w = new Writer();
    w.u64s(pks);
    return parseRowSet(await this.call(Op.GetMany, w.build(), Op.RowSet));
  }

  /** Update an existing row; error if the primary key is not found. protocol.rs
   * Request::Update, opcode 24, payload: u64 pk, u64s values. Returns the affected /
   * result count. */
  async update(pk: bigint, values: readonly bigint[]): Promise<bigint> {
    const w = new Writer();
    w.u64(pk);
    w.u64s(values);
    return affected(await this.call(Op.Update, w.build(), Op.Ok));
  }

  /** Delete all rows from the selected table. protocol.rs Request::Truncate, opcode 25,
   * payload: (empty payload). Returns the affected / result count. */
  async truncate(): Promise<bigint> {
    return affected(await this.call(Op.Truncate, EMPTY, Op.Ok));
  }

  /** Count rows where column `col` falls in `[lo, hi]`. protocol.rs Request::Count,
   * opcode 26, payload: u32 col, u64 lo, u64 hi. Returns the affected / result count. */
  async count(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.Count, w.build(), Op.Ok));
  }

  /** Insert-or-replace multiple rows in a single round trip. protocol.rs
   * Request::BatchUpsert, opcode 27, payload: rows rows. Returns the affected / result
   * count. */
  async batchUpsert(rows: readonly (readonly [bigint, readonly bigint[]])[]): Promise<bigint> {
    const w = new Writer();
    putRows(w, rows);
    return affected(await this.call(Op.BatchUpsert, w.build(), Op.Ok));
  }

  /** Update multiple existing rows in a single round trip. protocol.rs
   * Request::BatchUpdate, opcode 28, payload: rows rows. Returns (succeeded, skipped). */
  async batchUpdate(rows: readonly (readonly [bigint, readonly bigint[]])[]): Promise<{ succeeded: number; skipped: number }> {
    const w = new Writer();
    putRows(w, rows);
    return parseBatch(await this.call(Op.BatchUpdate, w.build(), Op.BatchResult));
  }

  /** Return the column schema for the currently selected table. protocol.rs
   * Request::DescribeTable, opcode 29, payload: (empty payload). Returns the column
   * names in schema order. */
  async describeTable(): Promise<string[]> {
    return parseSchema(await this.call(Op.DescribeTable, EMPTY, Op.Schema));
  }

  /** Check whether a named table exists in the catalog. protocol.rs
   * Request::TableExists, opcode 30, payload: string name. Returns whether the key
   * exists. */
  async tableExists(name: string): Promise<boolean> {
    const w = new Writer();
    w.string(name);
    return parseExists(await this.call(Op.TableExists, w.build(), Op.Exists));
  }

  /** Range scan returning only primary keys (no values). protocol.rs Request::ScanKeys,
   * opcode 31, payload: u32 col, u64 lo, u64 hi, u32 limit. Returns the primary keys in
   * scan order. */
  async scanKeys(col: number, lo: bigint, hi: bigint, limit: number): Promise<bigint[]> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    w.u32(limit);
    return parseKeys(await this.call(Op.ScanKeys, w.build(), Op.Keys));
  }

  /** Return the min, max, and count of a column value over a pk range. protocol.rs
   * Request::MinMax, opcode 32, payload: u32 col, u64 lo, u64 hi. Returns (min, max,
   * count), or null when the range is empty (NotFound). */
  async minMax(col: number, lo: bigint, hi: bigint): Promise<{ min: bigint; max: bigint; count: bigint } | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.MinMax, w.build(), Op.MinMaxResult, Op.NotFound);
    return f.opcode === Op.NotFound ? null : parseMinMax(f);
  }

  /** Sum a column's values over a pk range (saturating at u64::MAX). protocol.rs
   * Request::Sum, opcode 33, payload: u32 col, u64 lo, u64 hi. Returns the affected /
   * result count. */
  async sum(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.Sum, w.build(), Op.Ok));
  }

  /** Fetch a row by PK and atomically delete it. protocol.rs Request::GetAndDelete,
   * opcode 34, payload: u64 pk. Returns the row values, or null when the server answers
   * NotFound. */
  async getAndDelete(pk: bigint): Promise<Row | null> {
    const w = new Writer();
    w.u64(pk);
    const f = await this.call(Op.GetAndDelete, w.build(), Op.Row, Op.NotFound);
    return f.opcode === Op.NotFound ? null : parseRow(f);
  }

  /** Atomically add `delta` to column `col` of the row at `pk` (saturating at
   * u64::MAX). protocol.rs Request::Increment, opcode 35, payload: u64 pk, u32 col, u64
   * delta. Returns the result, or null when the server answers NotFound. */
  async increment(pk: bigint, col: number, delta: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(delta);
    const f = await this.call(Op.Increment, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Conditional update: set `values[col]` to `replacement` iff it currently equals
   * `expected`. protocol.rs Request::CompareAndSwap, opcode 36, payload: u64 pk, u32
   * col, u64 expected, u64 replacement. Returns the result, or null when the server
   * answers NotFound. */
  async compareAndSwap(pk: bigint, col: number, expected: bigint, replacement: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(expected);
    w.u64(replacement);
    const f = await this.call(Op.CompareAndSwap, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Set a single column of an existing row without providing all other column values.
   * protocol.rs Request::SetField, opcode 37, payload: u64 pk, u32 col, u64 value.
   * Returns the result, or null when the server answers NotFound. */
  async setField(pk: bigint, col: number, value: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(value);
    const f = await this.call(Op.SetField, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Insert a row only if the primary key is not already present. protocol.rs
   * Request::InsertIfAbsent, opcode 38, payload: u64 pk, u64s values. Returns the
   * affected / result count. */
  async insertIfAbsent(pk: bigint, values: readonly bigint[]): Promise<bigint> {
    const w = new Writer();
    w.u64(pk);
    w.u64s(values);
    return affected(await this.call(Op.InsertIfAbsent, w.build(), Op.Ok));
  }

  /** Delete all rows where column `col` falls in `[lo, hi]`. protocol.rs
   * Request::DeleteRange, opcode 39, payload: u32 col, u64 lo, u64 hi. Returns the
   * affected / result count. */
  async deleteRange(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.DeleteRange, w.build(), Op.Ok));
  }

  /** Fetch and atomically delete the row with the minimum column value in `[lo, hi]`.
   * protocol.rs Request::PopMin, opcode 40, payload: u32 col, u64 lo, u64 hi. Returns
   * the row values, or null when the server answers NotFound. */
  async popMin(col: number, lo: bigint, hi: bigint): Promise<Row | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.PopMin, w.build(), Op.Row, Op.NotFound);
    return f.opcode === Op.NotFound ? null : parseRow(f);
  }

  /** Fetch and atomically delete the row with the maximum column value in `[lo, hi]`.
   * protocol.rs Request::PopMax, opcode 41, payload: u32 col, u64 lo, u64 hi. Returns
   * the row values, or null when the server answers NotFound. */
  async popMax(col: number, lo: bigint, hi: bigint): Promise<Row | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.PopMax, w.build(), Op.Row, Op.NotFound);
    return f.opcode === Op.NotFound ? null : parseRow(f);
  }

  /** Set multiple `(col, value)` pairs on an existing row atomically. protocol.rs
   * Request::MultiSet, opcode 42, payload: u64 pk, pairs pairs. Returns the result, or
   * null when the server answers NotFound. */
  async multiSet(pk: bigint, pairs: readonly (readonly [number, bigint])[]): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    putPairs(w, pairs);
    const f = await this.call(Op.MultiSet, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Subtract `delta` from a column, clamping at 0 (saturating_sub). protocol.rs
   * Request::DecrementClamp, opcode 43, payload: u64 pk, u32 col, u64 delta. Returns
   * the result, or null when the server answers NotFound. */
  async decrementClamp(pk: bigint, col: number, delta: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(delta);
    const f = await this.call(Op.DecrementClamp, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Return existing row or insert-and-return provided values. protocol.rs
   * Request::GetOrInsert, opcode 44, payload: u64 pk, u64s values. Returns the row
   * values. */
  async getOrInsert(pk: bigint, values: readonly bigint[]): Promise<Row> {
    const w = new Writer();
    w.u64(pk);
    w.u64s(values);
    return parseRow(await this.call(Op.GetOrInsert, w.build(), Op.Row));
  }

  /** Set the same column to the same value across many rows in one round trip.
   * protocol.rs Request::BatchSetField, opcode 45, payload: u32 col, u64 value, u64s
   * pks. Returns the affected / result count. */
  async batchSetField(col: number, value: bigint, pks: readonly bigint[]): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(value);
    w.u64s(pks);
    return affected(await this.call(Op.BatchSetField, w.build(), Op.Ok));
  }

  /** Return the top-N rows by column value (highest first). protocol.rs Request::Top,
   * opcode 46, payload: u32 col, u64 lo, u64 hi, u32 n. Returns the matching rows. */
  async top(col: number, lo: bigint, hi: bigint, n: number): Promise<Row[]> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    w.u32(n);
    return parseRows(await this.call(Op.Top, w.build(), Op.Rows));
  }

  /** Return the bottom-N rows by column value (lowest first). protocol.rs
   * Request::Bottom, opcode 47, payload: u32 col, u64 lo, u64 hi, u32 n. Returns the
   * matching rows. */
  async bottom(col: number, lo: bigint, hi: bigint, n: number): Promise<Row[]> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    w.u32(n);
    return parseRows(await this.call(Op.Bottom, w.build(), Op.Rows));
  }

  /** Exact primary-key existence check (not probabilistic). protocol.rs
   * Request::PkExists, opcode 48, payload: u64 pk. Returns whether the key exists. */
  async pkExists(pk: bigint): Promise<boolean> {
    const w = new Writer();
    w.u64(pk);
    return parseExists(await this.call(Op.PkExists, w.build(), Op.Exists));
  }

  /** Atomically increment multiple `(col, delta)` pairs on one row (saturating at
   * u64::MAX). protocol.rs Request::IncrementMulti, opcode 49, payload: u64 pk, pairs
   * pairs. Returns the row values, or null when the server answers NotFound. */
  async incrementMulti(pk: bigint, pairs: readonly (readonly [number, bigint])[]): Promise<Row | null> {
    const w = new Writer();
    w.u64(pk);
    putPairs(w, pairs);
    const f = await this.call(Op.IncrementMulti, w.build(), Op.Row, Op.NotFound);
    return f.opcode === Op.NotFound ? null : parseRow(f);
  }

  /** Copy the row at `src_pk` to `dst_pk` (insert-or-replace). protocol.rs
   * Request::CopyRow, opcode 50, payload: u64 src_pk, u64 dst_pk. Returns the result,
   * or null when the server answers NotFound. */
  async copyRow(srcPk: bigint, dstPk: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(srcPk);
    w.u64(dstPk);
    const f = await this.call(Op.CopyRow, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Swap the value of column `col` between `pk1` and `pk2`. protocol.rs
   * Request::SwapField, opcode 51, payload: u64 pk1, u64 pk2, u32 col. Returns the
   * result, or null when the server answers NotFound. */
  async swapField(pk1: bigint, pk2: bigint, col: number): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk1);
    w.u64(pk2);
    w.u32(col);
    const f = await this.call(Op.SwapField, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Clamp column `col` of row `pk` into `[lo, hi]`. protocol.rs Request::Clamp, opcode
   * 52, payload: u64 pk, u32 col, u64 lo, u64 hi. Returns the result, or null when the
   * server answers NotFound. */
  async clamp(pk: bigint, col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.Clamp, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Return `[sum, count]` for column `col` over the scan range `[lo, hi]`. protocol.rs
   * Request::Average, opcode 53, payload: u32 col, u64 lo, u64 hi. Returns the row
   * values, or null when the server answers NotFound. */
  async average(col: number, lo: bigint, hi: bigint): Promise<Row | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.Average, w.build(), Op.Row, Op.NotFound);
    return f.opcode === Op.NotFound ? null : parseRow(f);
  }

  /** Delete the row at `pk` only when `values[col] == value`. protocol.rs
   * Request::DeleteIf, opcode 54, payload: u64 pk, u32 col, u64 value. Returns the
   * result, or null when the server answers NotFound. */
  async deleteIf(pk: bigint, col: number, value: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(value);
    const f = await this.call(Op.DeleteIf, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Set `values[col]` to `new_value` only if it currently equals `expected`.
   * protocol.rs Request::SetFieldIf, opcode 55, payload: u64 pk, u32 col, u64 expected,
   * u64 new_value. Returns the result, or null when the server answers NotFound. */
  async setFieldIf(pk: bigint, col: number, expected: bigint, newValue: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(expected);
    w.u64(newValue);
    const f = await this.call(Op.SetFieldIf, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Return the 0-based rank of `pk`'s column `col` value within `[lo, hi]`.
   * protocol.rs Request::RankOf, opcode 56, payload: u64 pk, u32 col, u64 lo, u64 hi.
   * Returns the result, or null when the server answers NotFound. */
  async rankOf(pk: bigint, col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.RankOf, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Return the value at the Nth percentile of column `col` in `[lo, hi]`. protocol.rs
   * Request::Percentile, opcode 57, payload: u32 col, u64 lo, u64 hi, u32 pct. Returns
   * the result, or null when the server answers NotFound. */
  async percentile(col: number, lo: bigint, hi: bigint, pct: number): Promise<bigint | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    w.u32(pct);
    const f = await this.call(Op.Percentile, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Sum multiple column values of a single row. protocol.rs Request::SumFields, opcode
   * 58, payload: u64 pk, u32s cols. Returns the result, or null when the server answers
   * NotFound. */
  async sumFields(pk: bigint, cols: readonly number[]): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    putU32s(w, cols);
    const f = await this.call(Op.SumFields, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Return the maximum across multiple column values of a single row. protocol.rs
   * Request::MaxField, opcode 59, payload: u64 pk, u32s cols. Returns the result, or
   * null when the server answers NotFound. */
  async maxField(pk: bigint, cols: readonly number[]): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    putU32s(w, cols);
    const f = await this.call(Op.MaxField, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Range scan with an additional equality filter on a second column. protocol.rs
   * Request::FilterScan, opcode 60, payload: u32 scan_col, u64 lo, u64 hi, u32
   * filter_col, u64 filter_val. Returns the matching rows. */
  async filterScan(scanCol: number, lo: bigint, hi: bigint, filterCol: number, filterVal: bigint): Promise<Row[]> {
    const w = new Writer();
    w.u32(scanCol);
    w.u64(lo);
    w.u64(hi);
    w.u32(filterCol);
    w.u64(filterVal);
    return parseRows(await this.call(Op.FilterScan, w.build(), Op.Rows));
  }

  /** Increment the same column by `delta` across many rows in one round trip.
   * protocol.rs Request::BatchIncrementField, opcode 61, payload: u32 col, u64 delta,
   * u64s pks. Returns the affected / result count. */
  async batchIncrementField(col: number, delta: bigint, pks: readonly bigint[]): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(delta);
    w.u64s(pks);
    return affected(await this.call(Op.BatchIncrementField, w.build(), Op.Ok));
  }

  /** Count distinct values of column `col` in scan range `[lo, hi]`. protocol.rs
   * Request::CountDistinct, opcode 62, payload: u32 col, u64 lo, u64 hi. Returns the
   * affected / result count. */
  async countDistinct(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.CountDistinct, w.build(), Op.Ok));
  }

  /** Return the median value of column `col` in `[lo, hi]`. protocol.rs
   * Request::Median, opcode 63, payload: u32 col, u64 lo, u64 hi. Returns the result,
   * or null when the server answers NotFound. */
  async median(col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.Median, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Delete each pk in the list only if `values[col] == value`. protocol.rs
   * Request::BatchDeleteIf, opcode 64, payload: u32 col, u64 value, u64s pks. Returns
   * the affected / result count. */
  async batchDeleteIf(col: number, value: bigint, pks: readonly bigint[]): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(value);
    w.u64s(pks);
    return affected(await this.call(Op.BatchDeleteIf, w.build(), Op.Ok));
  }

  /** Check multiple primary keys for existence in one round trip. protocol.rs
   * Request::BatchPkExists, opcode 65, payload: u64s pks. Returns the affected / result
   * count. */
  async batchPkExists(pks: readonly bigint[]): Promise<bigint> {
    const w = new Writer();
    w.u64s(pks);
    return affected(await this.call(Op.BatchPkExists, w.build(), Op.Ok));
  }

  /** Range scan on `scan_col` in `[lo, hi]` keeping rows where `filter_col` in
   * `[filter_lo, filter_hi]`. protocol.rs Request::FieldBetween, opcode 66, payload:
   * u32 scan_col, u64 lo, u64 hi, u32 filter_col, u64 filter_lo, u64 filter_hi. Returns
   * the matching rows. */
  async fieldBetween(scanCol: number, lo: bigint, hi: bigint, filterCol: number, filterLo: bigint, filterHi: bigint): Promise<Row[]> {
    const w = new Writer();
    w.u32(scanCol);
    w.u64(lo);
    w.u64(hi);
    w.u32(filterCol);
    w.u64(filterLo);
    w.u64(filterHi);
    return parseRows(await this.call(Op.FieldBetween, w.build(), Op.Rows));
  }

  /** Minimum value of column `col` over scan range `[lo, hi]`. protocol.rs
   * Request::ScalarMin, opcode 67, payload: u32 col, u64 lo, u64 hi. Returns the
   * result, or null when the server answers NotFound. */
  async scalarMin(col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.ScalarMin, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Maximum value of column `col` over scan range `[lo, hi]`. protocol.rs
   * Request::ScalarMax, opcode 68, payload: u32 col, u64 lo, u64 hi. Returns the
   * result, or null when the server answers NotFound. */
  async scalarMax(col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.ScalarMax, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** Return only the listed columns of a single row. protocol.rs Request::MultiGet,
   * opcode 69, payload: u64 pk, u32s cols. Returns the row values, or null when the
   * server answers NotFound. */
  async multiGet(pk: bigint, cols: readonly number[]): Promise<Row | null> {
    const w = new Writer();
    w.u64(pk);
    putU32s(w, cols);
    const f = await this.call(Op.MultiGet, w.build(), Op.Row, Op.NotFound);
    return f.opcode === Op.NotFound ? null : parseRow(f);
  }

  /** Add `delta` to column `col` if `pk` exists; otherwise insert `values` (with
   * `values[col]` as initial). protocol.rs Request::IncrementOrInsert, opcode 70,
   * payload: u64 pk, u32 col, u64 delta, u64s values. Returns the affected / result
   * count. */
  async incrementOrInsert(pk: bigint, col: number, delta: bigint, values: readonly bigint[]): Promise<bigint> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(delta);
    w.u64s(values);
    return affected(await this.call(Op.IncrementOrInsert, w.build(), Op.Ok));
  }

  /** Set column `col` to 0 for every row in scan range `[lo, hi]`. protocol.rs
   * Request::ZeroField, opcode 71, payload: u32 col, u64 lo, u64 hi. Returns the
   * affected / result count. */
  async zeroField(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.ZeroField, w.build(), Op.Ok));
  }

  /** Paginated range scan: skip `offset` rows then return up to `limit`. protocol.rs
   * Request::ScanPage, opcode 72, payload: u32 col, u64 lo, u64 hi, u32 offset, u32
   * limit. Returns the matching rows. */
  async scanPage(col: number, lo: bigint, hi: bigint, offset: number, limit: number): Promise<Row[]> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    w.u32(offset);
    w.u32(limit);
    return parseRows(await this.call(Op.ScanPage, w.build(), Op.Rows));
  }

  /** SumField. protocol.rs Request::SumField, opcode 73, payload: u32 col, u64 lo, u64
   * hi. Returns the affected / result count. */
  async sumField(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.SumField, w.build(), Op.Ok));
  }

  /** BatchGet. protocol.rs Request::BatchGet, opcode 74, payload: u64s pks. Returns the
   * matching rows. */
  async batchGet(pks: readonly bigint[]): Promise<Row[]> {
    const w = new Writer();
    w.u64s(pks);
    return parseRows(await this.call(Op.BatchGet, w.build(), Op.Rows));
  }

  /** SetFieldRange. protocol.rs Request::SetFieldRange, opcode 75, payload: u32 col,
   * u64 value, u64 lo, u64 hi. Returns the affected / result count. */
  async setFieldRange(col: number, value: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(value);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.SetFieldRange, w.build(), Op.Ok));
  }

  /** MaxRow. protocol.rs Request::MaxRow, opcode 76, payload: u32 col, u64 lo, u64 hi.
   * Returns the row values, or null when the server answers NotFound. */
  async maxRow(col: number, lo: bigint, hi: bigint): Promise<Row | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.MaxRow, w.build(), Op.Row, Op.NotFound);
    return f.opcode === Op.NotFound ? null : parseRow(f);
  }

  /** MinRow. protocol.rs Request::MinRow, opcode 77, payload: u32 col, u64 lo, u64 hi.
   * Returns the row values, or null when the server answers NotFound. */
  async minRow(col: number, lo: bigint, hi: bigint): Promise<Row | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.MinRow, w.build(), Op.Row, Op.NotFound);
    return f.opcode === Op.NotFound ? null : parseRow(f);
  }

  /** IncrementRange. protocol.rs Request::IncrementRange, opcode 78, payload: u32 col,
   * u64 delta, u64 lo, u64 hi. Returns the affected / result count. */
  async incrementRange(col: number, delta: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(delta);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.IncrementRange, w.build(), Op.Ok));
  }

  /** DeleteMin. protocol.rs Request::DeleteMin, opcode 79, payload: u32 col, u64 lo,
   * u64 hi. Returns the result, or null when the server answers NotFound. */
  async deleteMin(col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.DeleteMin, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** DeleteMax. protocol.rs Request::DeleteMax, opcode 80, payload: u32 col, u64 lo,
   * u64 hi. Returns the result, or null when the server answers NotFound. */
  async deleteMax(col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.DeleteMax, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** UniqueCount. protocol.rs Request::UniqueCount, opcode 81, payload: u32 col, u64
   * lo, u64 hi. Returns the affected / result count. */
  async uniqueCount(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.UniqueCount, w.build(), Op.Ok));
  }

  /** TopN. protocol.rs Request::TopN, opcode 82, payload: u32 col, u32 n, u64 lo, u64
   * hi. Returns the matching rows. */
  async topN(col: number, n: number, lo: bigint, hi: bigint): Promise<Row[]> {
    const w = new Writer();
    w.u32(col);
    w.u32(n);
    w.u64(lo);
    w.u64(hi);
    return parseRows(await this.call(Op.TopN, w.build(), Op.Rows));
  }

  /** BottomN. protocol.rs Request::BottomN, opcode 83, payload: u32 col, u32 n, u64 lo,
   * u64 hi. Returns the matching rows. */
  async bottomN(col: number, n: number, lo: bigint, hi: bigint): Promise<Row[]> {
    const w = new Writer();
    w.u32(col);
    w.u32(n);
    w.u64(lo);
    w.u64(hi);
    return parseRows(await this.call(Op.BottomN, w.build(), Op.Rows));
  }

  /** MoveField. protocol.rs Request::MoveField, opcode 84, payload: u64 src_pk, u64
   * dst_pk, u32 col. Returns the result, or null when the server answers NotFound. */
  async moveField(srcPk: bigint, dstPk: bigint, col: number): Promise<bigint | null> {
    const w = new Writer();
    w.u64(srcPk);
    w.u64(dstPk);
    w.u32(col);
    const f = await this.call(Op.MoveField, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** CountIf. protocol.rs Request::CountIf, opcode 85, payload: u32 col, u64 value, u64
   * lo, u64 hi. Returns the affected / result count. */
  async countIf(col: number, value: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(value);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.CountIf, w.build(), Op.Ok));
  }

  /** SetIfGreater. protocol.rs Request::SetIfGreater, opcode 86, payload: u32 col, u64
   * value, u64 lo, u64 hi. Returns the affected / result count. */
  async setIfGreater(col: number, value: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(value);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.SetIfGreater, w.build(), Op.Ok));
  }

  /** SetIfLess. protocol.rs Request::SetIfLess, opcode 87, payload: u32 col, u64 value,
   * u64 lo, u64 hi. Returns the affected / result count. */
  async setIfLess(col: number, value: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(value);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.SetIfLess, w.build(), Op.Ok));
  }

  /** AbsDiff. protocol.rs Request::AbsDiff, opcode 88, payload: u64 pk1, u64 pk2, u32
   * col. Returns the result, or null when the server answers NotFound. */
  async absDiff(pk1: bigint, pk2: bigint, col: number): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk1);
    w.u64(pk2);
    w.u32(col);
    const f = await this.call(Op.AbsDiff, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** SwapFields. protocol.rs Request::SwapFields, opcode 89, payload: u64 pk1, u64 pk2,
   * u32 col. Returns the result, or null when the server answers NotFound. */
  async swapFields(pk1: bigint, pk2: bigint, col: number): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk1);
    w.u64(pk2);
    w.u32(col);
    const f = await this.call(Op.SwapFields, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** CopyRange. protocol.rs Request::CopyRange, opcode 90, payload: u32 src_col, u32
   * dst_col, u64 lo, u64 hi. Returns the affected / result count. */
  async copyRange(srcCol: number, dstCol: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(srcCol);
    w.u32(dstCol);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.CopyRange, w.build(), Op.Ok));
  }

  /** PropagateMax. protocol.rs Request::PropagateMax, opcode 91, payload: u32 col, u64
   * lo, u64 hi. Returns the affected / result count. */
  async propagateMax(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.PropagateMax, w.build(), Op.Ok));
  }

  /** PropagateMin. protocol.rs Request::PropagateMin, opcode 92, payload: u32 col, u64
   * lo, u64 hi. Returns the affected / result count. */
  async propagateMin(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.PropagateMin, w.build(), Op.Ok));
  }

  /** ClampRange. protocol.rs Request::ClampRange, opcode 93, payload: u32 col, u64
   * min_val, u64 max_val, u64 lo, u64 hi. Returns the affected / result count. */
  async clampRange(col: number, minVal: bigint, maxVal: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(minVal);
    w.u64(maxVal);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.ClampRange, w.build(), Op.Ok));
  }

  /** BatchConditionalSet. protocol.rs Request::BatchConditionalSet, opcode 94, payload:
   * u32 col, u64 expected, u64 new_value, u64 lo, u64 hi. Returns the affected / result
   * count. */
  async batchConditionalSet(col: number, expected: bigint, newValue: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(expected);
    w.u64(newValue);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.BatchConditionalSet, w.build(), Op.Ok));
  }

  /** WindowRank. protocol.rs Request::WindowRank, opcode 95, payload: u64 pk, u32 col,
   * u64 lo, u64 hi. Returns the result, or null when the server answers NotFound. */
  async windowRank(pk: bigint, col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.WindowRank, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** WindowPct. protocol.rs Request::WindowPct, opcode 96, payload: u64 pk, u32 col,
   * u64 lo, u64 hi. Returns the result, or null when the server answers NotFound. */
  async windowPct(pk: bigint, col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.WindowPct, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** SumProduct. protocol.rs Request::SumProduct, opcode 97, payload: u32 col_a, u32
   * col_b, u64 lo, u64 hi. Returns the affected / result count. */
  async sumProduct(colA: number, colB: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(colA);
    w.u32(colB);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.SumProduct, w.build(), Op.Ok));
  }

  /** NthValue. protocol.rs Request::NthValue, opcode 98, payload: u32 col, u32 n, u64
   * lo, u64 hi. Returns the result, or null when the server answers NotFound. */
  async nthValue(col: number, n: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u32(col);
    w.u32(n);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.NthValue, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** DenseRankWin. protocol.rs Request::DenseRankWin, opcode 99, payload: u64 pk, u32
   * col, u64 lo, u64 hi. Returns the result, or null when the server answers NotFound. */
  async denseRankWin(pk: bigint, col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.DenseRankWin, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** SetFieldMany. protocol.rs Request::SetFieldMany, opcode 100, payload: u32 col, u64
   * value, u64s pks. Returns the affected / result count. */
  async setFieldMany(col: number, value: bigint, pks: readonly bigint[]): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(value);
    w.u64s(pks);
    return affected(await this.call(Op.SetFieldMany, w.build(), Op.Ok));
  }

  /** MultiColumnSum. protocol.rs Request::MultiColumnSum, opcode 101, payload: u64s
   * pks, u32 col. Returns the affected / result count. */
  async multiColumnSum(pks: readonly bigint[], col: number): Promise<bigint> {
    const w = new Writer();
    w.u64s(pks);
    w.u32(col);
    return affected(await this.call(Op.MultiColumnSum, w.build(), Op.Ok));
  }

  /** FieldHistogram. protocol.rs Request::FieldHistogram, opcode 102, payload: u32 col,
   * u64 bucket_size, u64 lo, u64 hi. Returns the matching rows. */
  async fieldHistogram(col: number, bucketSize: bigint, lo: bigint, hi: bigint): Promise<Row[]> {
    const w = new Writer();
    w.u32(col);
    w.u64(bucketSize);
    w.u64(lo);
    w.u64(hi);
    return parseRows(await this.call(Op.FieldHistogram, w.build(), Op.Rows));
  }

  /** ZScoreFilter. protocol.rs Request::ZScoreFilter, opcode 103, payload: u32 col, u64
   * lo, u64 hi. Returns the matching rows. */
  async zScoreFilter(col: number, lo: bigint, hi: bigint): Promise<Row[]> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return parseRows(await this.call(Op.ZScoreFilter, w.build(), Op.Rows));
  }

  /** RunningSum. protocol.rs Request::RunningSum, opcode 104, payload: u32 col, u64 lo,
   * u64 hi. Returns the matching rows. */
  async runningSum(col: number, lo: bigint, hi: bigint): Promise<Row[]> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return parseRows(await this.call(Op.RunningSum, w.build(), Op.Rows));
  }

  /** FloorDiv. protocol.rs Request::FloorDiv, opcode 105, payload: u32 col, u64
   * divisor, u64 lo, u64 hi. Returns the affected / result count. */
  async floorDiv(col: number, divisor: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(divisor);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.FloorDiv, w.build(), Op.Ok));
  }

  /** MulField. protocol.rs Request::MulField, opcode 106, payload: u32 col, u64 factor,
   * u64 lo, u64 hi. Returns the affected / result count. */
  async mulField(col: number, factor: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(factor);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.MulField, w.build(), Op.Ok));
  }

  /** SetFieldAdd. protocol.rs Request::SetFieldAdd, opcode 107, payload: u64 pk, u32
   * col, u64 addend. Returns the result, or null when the server answers NotFound. */
  async setFieldAdd(pk: bigint, col: number, addend: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(addend);
    const f = await this.call(Op.SetFieldAdd, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** SetFieldMul. protocol.rs Request::SetFieldMul, opcode 108, payload: u64 pk, u32
   * col, u64 factor. Returns the result, or null when the server answers NotFound. */
  async setFieldMul(pk: bigint, col: number, factor: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(factor);
    const f = await this.call(Op.SetFieldMul, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** SwapPkFields. protocol.rs Request::SwapPkFields, opcode 109, payload: u64 pk, u32
   * col_a, u32 col_b. Returns the result, or null when the server answers NotFound. */
  async swapPkFields(pk: bigint, colA: number, colB: number): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(colA);
    w.u32(colB);
    const f = await this.call(Op.SwapPkFields, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** CountRange. protocol.rs Request::CountRange, opcode 110, payload: u32 col, u64 lo,
   * u64 hi. Returns the affected / result count. */
  async countRange(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.CountRange, w.build(), Op.Ok));
  }

  /** SumRange. protocol.rs Request::SumRange, opcode 111, payload: u32 col, u64 lo, u64
   * hi. Returns the affected / result count. */
  async sumRange(col: number, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.SumRange, w.build(), Op.Ok));
  }

  /** MinRange. protocol.rs Request::MinRange, opcode 112, payload: u32 col, u64 lo, u64
   * hi. Returns the result, or null when the server answers NotFound. */
  async minRange(col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.MinRange, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** MaxRange. protocol.rs Request::MaxRange, opcode 113, payload: u32 col, u64 lo, u64
   * hi. Returns the result, or null when the server answers NotFound. */
  async maxRange(col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.MaxRange, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** AvgRange. protocol.rs Request::AvgRange, opcode 114, payload: u32 col, u64 lo, u64
   * hi. Returns the result, or null when the server answers NotFound. */
  async avgRange(col: number, lo: bigint, hi: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u32(col);
    w.u64(lo);
    w.u64(hi);
    const f = await this.call(Op.AvgRange, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** IncrementField. protocol.rs Request::IncrementField, opcode 115, payload: u64 pk,
   * u32 col. Returns the result, or null when the server answers NotFound. */
  async incrementField(pk: bigint, col: number): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    const f = await this.call(Op.IncrementField, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** DecrementField. protocol.rs Request::DecrementField, opcode 116, payload: u64 pk,
   * u32 col. Returns the result, or null when the server answers NotFound. */
  async decrementField(pk: bigint, col: number): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    const f = await this.call(Op.DecrementField, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** SetMin. protocol.rs Request::SetMin, opcode 117, payload: u64 pk, u32 col, u64
   * value. Returns the result, or null when the server answers NotFound. */
  async setMin(pk: bigint, col: number, value: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(value);
    const f = await this.call(Op.SetMin, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** SetMax. protocol.rs Request::SetMax, opcode 118, payload: u64 pk, u32 col, u64
   * value. Returns the result, or null when the server answers NotFound. */
  async setMax(pk: bigint, col: number, value: bigint): Promise<bigint | null> {
    const w = new Writer();
    w.u64(pk);
    w.u32(col);
    w.u64(value);
    const f = await this.call(Op.SetMax, w.build(), Op.Ok, Op.NotFound);
    return f.opcode === Op.NotFound ? null : affected(f);
  }

  /** XorField. protocol.rs Request::XorField, opcode 119, payload: u32 col, u64 mask,
   * u64 lo, u64 hi. Returns the affected / result count. */
  async xorField(col: number, mask: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(mask);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.XorField, w.build(), Op.Ok));
  }

  /** AndField. protocol.rs Request::AndField, opcode 120, payload: u32 col, u64 mask,
   * u64 lo, u64 hi. Returns the affected / result count. */
  async andField(col: number, mask: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(mask);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.AndField, w.build(), Op.Ok));
  }

  /** OrField. protocol.rs Request::OrField, opcode 121, payload: u32 col, u64 mask, u64
   * lo, u64 hi. Returns the affected / result count. */
  async orField(col: number, mask: bigint, lo: bigint, hi: bigint): Promise<bigint> {
    const w = new Writer();
    w.u32(col);
    w.u64(mask);
    w.u64(lo);
    w.u64(hi);
    return affected(await this.call(Op.OrField, w.build(), Op.Ok));
  }

  /** BatchGetField. protocol.rs Request::BatchGetField, opcode 122, payload: u32 col,
   * u64s pks. Returns the matching rows. */
  async batchGetField(col: number, pks: readonly bigint[]): Promise<Row[]> {
    const w = new Writer();
    w.u32(col);
    w.u64s(pks);
    return parseRows(await this.call(Op.BatchGetField, w.build(), Op.Rows));
  }
}
