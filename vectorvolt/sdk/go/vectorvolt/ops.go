package vectorvolt

// Wrappers for the remaining request opcodes of the VectorVolt wire protocol: one
// method per opcode the server dispatch (vectorvolt-net/src/server.rs) handles, laid
// out exactly as protocol.rs Request::encode_payload. Every method needs a connected
// client with a table selected unless noted.

// ColValue is a (column, value) pair: the new value for MultiSet, the delta for
// IncrementMulti.
type ColValue struct {
	Col   uint32
	Value uint64
}

func (w *writer) u32s(vs []uint32) *writer {
	w.u32(uint32(len(vs)))
	for _, v := range vs {
		w.u32(v)
	}
	return w
}

func (w *writer) pairs(ps []ColValue) *writer {
	w.u32(uint32(len(ps)))
	for _, p := range ps {
		w.u32(p.Col).u64(p.Value)
	}
	return w
}

func (w *writer) rowList(rows []Row) *writer {
	w.u32(uint32(len(rows)))
	for _, r := range rows {
		w.u64(r.PK).u64s(r.Values)
	}
	return w
}

func (c *Client) callOK(op uint8, p []byte) (uint64, error) {
	f, err := c.call(op, p, OpOk)
	if err != nil {
		return 0, err
	}
	return affected(f)
}

func (c *Client) callOKOpt(op uint8, p []byte) (uint64, bool, error) {
	f, err := c.call(op, p, OpOk, OpNotFound)
	if err != nil {
		return 0, false, err
	}
	if f.Opcode == OpNotFound {
		return 0, false, nil
	}
	v, err := affected(f)
	return v, err == nil, err
}

func parseRow(f *Frame) ([]uint64, error) {
	r := &reader{b: f.Payload}
	v, err := r.u64s()
	if err != nil {
		return nil, err
	}
	return v, r.finish()
}

func (c *Client) callRow(op uint8, p []byte) ([]uint64, error) {
	f, err := c.call(op, p, OpRow)
	if err != nil {
		return nil, err
	}
	return parseRow(f)
}

// callRowOpt returns (nil, nil) when the server answers NotFound.
func (c *Client) callRowOpt(op uint8, p []byte) ([]uint64, error) {
	f, err := c.call(op, p, OpRow, OpNotFound)
	if err != nil {
		return nil, err
	}
	if f.Opcode == OpNotFound {
		return nil, nil
	}
	return parseRow(f)
}

func (c *Client) callRows(op uint8, p []byte) ([][]uint64, error) {
	f, err := c.call(op, p, OpRows)
	if err != nil {
		return nil, err
	}
	r := &reader{b: f.Payload}
	n, err := r.u32()
	if err != nil {
		return nil, err
	}
	if int(n) > (len(r.b)-r.at)/4 {
		return nil, protoErr("declared more rows than the payload holds")
	}
	rows := make([][]uint64, n)
	for i := range rows {
		if rows[i], err = r.u64s(); err != nil {
			return nil, err
		}
	}
	return rows, r.finish()
}

func (c *Client) callExists(op uint8, p []byte) (bool, error) {
	f, err := c.call(op, p, OpExists)
	if err != nil {
		return false, err
	}
	r := &reader{b: f.Payload}
	b, err := r.take(1)
	if err != nil {
		return false, err
	}
	return b[0] != 0, r.finish()
}

func (c *Client) callMinMax(op uint8, p []byte) (min, max, count uint64, found bool, err error) {
	f, err := c.call(op, p, OpMinMaxResult, OpNotFound)
	if err != nil || f.Opcode == OpNotFound {
		return
	}
	r := &reader{b: f.Payload}
	if min, err = r.u64(); err != nil {
		return
	}
	if max, err = r.u64(); err != nil {
		return
	}
	if count, err = r.u64(); err != nil {
		return
	}
	err = r.finish()
	found = err == nil
	return
}

func (c *Client) callKeys(op uint8, p []byte) ([]uint64, error) {
	f, err := c.call(op, p, OpKeys)
	if err != nil {
		return nil, err
	}
	r := &reader{b: f.Payload}
	n, err := r.u32()
	if err != nil {
		return nil, err
	}
	if int(n) > (len(r.b)-r.at)/8 {
		return nil, protoErr("declared more keys than the payload holds")
	}
	out := make([]uint64, n)
	for i := range out {
		if out[i], err = r.u64(); err != nil {
			return nil, err
		}
	}
	return out, r.finish()
}

func (c *Client) callRowSet(op uint8, p []byte) ([]Row, error) {
	f, err := c.call(op, p, OpRowSet)
	if err != nil {
		return nil, err
	}
	r := &reader{b: f.Payload}
	n, err := r.u32()
	if err != nil {
		return nil, err
	}
	if int(n) > (len(r.b)-r.at)/4 {
		return nil, protoErr("declared more entries than the payload holds")
	}
	out := make([]Row, n)
	for i := range out {
		if out[i].PK, err = r.u64(); err != nil {
			return nil, err
		}
		if out[i].Values, err = r.u64s(); err != nil {
			return nil, err
		}
	}
	return out, r.finish()
}

func (c *Client) callBatch(op uint8, p []byte) (succeeded, skipped uint32, err error) {
	return batchResult(c.call(op, p, OpBatchResult))
}

func (c *Client) callSchema(op uint8, p []byte) ([]string, error) {
	f, err := c.call(op, p, OpSchema)
	if err != nil {
		return nil, err
	}
	r := &reader{b: f.Payload}
	cols, err := r.strs()
	if err != nil {
		return nil, err
	}
	return cols, r.finish()
}

// Compact compacts the active table (protocol.rs Request::Compact, opcode 7). Returns the result count.
func (c *Client) Compact() (uint64, error) {
	return c.callOK(OpCompact, nil)
}

// SecondaryLookup: SecondaryLookup. protocol.rs Request::SecondaryLookup, opcode 11,
// payload: u32 col_idx, u64 value. Returns the matching rows.
func (c *Client) SecondaryLookup(colIdx uint32, value uint64) ([][]uint64, error) {
	w := &writer{}
	w.u32(colIdx).u64(value)
	return c.callRows(OpSecondaryLookup, w.b)
}

// FkExists: FkExists. protocol.rs Request::FkExists, opcode 12, payload: u64 value.
// Returns whether the key exists.
func (c *Client) FkExists(value uint64) (bool, error) {
	w := &writer{}
	w.u64(value)
	return c.callExists(OpFkExists, w.b)
}

// Upsert: Insert or replace a row (delete existing, then insert). protocol.rs
// Request::Upsert, opcode 22, payload: u64 pk, u64s values. Returns the affected / result
// count.
func (c *Client) Upsert(pk uint64, values []uint64) (uint64, error) {
	w := &writer{}
	w.u64(pk).u64s(values)
	return c.callOK(OpUpsert, w.b)
}

// GetMany: Fetch multiple rows by primary key in one round trip. protocol.rs
// Request::GetMany, opcode 23, payload: u64s pks. Returns the found rows as (pk, values)
// entries; missing keys are omitted.
func (c *Client) GetMany(pks []uint64) ([]Row, error) {
	w := &writer{}
	w.u64s(pks)
	return c.callRowSet(OpGetMany, w.b)
}

// Update: Update an existing row; error if the primary key is not found. protocol.rs
// Request::Update, opcode 24, payload: u64 pk, u64s values. Returns the affected / result
// count.
func (c *Client) Update(pk uint64, values []uint64) (uint64, error) {
	w := &writer{}
	w.u64(pk).u64s(values)
	return c.callOK(OpUpdate, w.b)
}

// Truncate: Delete all rows from the selected table. protocol.rs Request::Truncate,
// opcode 25, payload: (empty payload). Returns the affected / result count.
func (c *Client) Truncate() (uint64, error) {
	return c.callOK(OpTruncate, nil)
}

// Count: Count rows where column `col` falls in `[lo, hi]`. protocol.rs Request::Count,
// opcode 26, payload: u32 col, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) Count(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpCount, w.b)
}

// BatchUpsert: Insert-or-replace multiple rows in a single round trip. protocol.rs
// Request::BatchUpsert, opcode 27, payload: rows rows. Returns the affected / result
// count.
func (c *Client) BatchUpsert(rows []Row) (uint64, error) {
	w := &writer{}
	w.rowList(rows)
	return c.callOK(OpBatchUpsert, w.b)
}

// BatchUpdate: Update multiple existing rows in a single round trip. protocol.rs
// Request::BatchUpdate, opcode 28, payload: rows rows. Returns (succeeded, skipped).
func (c *Client) BatchUpdate(rows []Row) (succeeded, skipped uint32, err error) {
	w := &writer{}
	w.rowList(rows)
	return c.callBatch(OpBatchUpdate, w.b)
}

// DescribeTable: Return the column schema for the currently selected table. protocol.rs
// Request::DescribeTable, opcode 29, payload: (empty payload). Returns the column names
// in schema order.
func (c *Client) DescribeTable() ([]string, error) {
	return c.callSchema(OpDescribeTable, nil)
}

// TableExists: Check whether a named table exists in the catalog. protocol.rs
// Request::TableExists, opcode 30, payload: string name. Returns whether the key exists.
func (c *Client) TableExists(name string) (bool, error) {
	w := &writer{}
	w.str(name)
	return c.callExists(OpTableExists, w.b)
}

// ScanKeys: Range scan returning only primary keys (no values). protocol.rs
// Request::ScanKeys, opcode 31, payload: u32 col, u64 lo, u64 hi, u32 limit. Returns the
// primary keys in scan order.
func (c *Client) ScanKeys(col uint32, lo uint64, hi uint64, limit uint32) ([]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi).u32(limit)
	return c.callKeys(OpScanKeys, w.b)
}

// MinMax: Return the min, max, and count of a column value over a pk range. protocol.rs
// Request::MinMax, opcode 32, payload: u32 col, u64 lo, u64 hi. Returns (min, max,
// count), or nil/false when the range is empty (NotFound).
func (c *Client) MinMax(col uint32, lo uint64, hi uint64) (min, max, count uint64, found bool, err error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callMinMax(OpMinMax, w.b)
}

// Sum: Sum a column's values over a pk range (saturating at u64::MAX). protocol.rs
// Request::Sum, opcode 33, payload: u32 col, u64 lo, u64 hi. Returns the affected /
// result count.
func (c *Client) Sum(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpSum, w.b)
}

// GetAndDelete: Fetch a row by PK and atomically delete it. protocol.rs
// Request::GetAndDelete, opcode 34, payload: u64 pk. Returns the row values, or nil/false
// when the server answers NotFound.
func (c *Client) GetAndDelete(pk uint64) ([]uint64, error) {
	w := &writer{}
	w.u64(pk)
	return c.callRowOpt(OpGetAndDelete, w.b)
}

// Increment: Atomically add `delta` to column `col` of the row at `pk` (saturating at
// u64::MAX). protocol.rs Request::Increment, opcode 35, payload: u64 pk, u32 col, u64
// delta. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) Increment(pk uint64, col uint32, delta uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(delta)
	return c.callOKOpt(OpIncrement, w.b)
}

// CompareAndSwap: Conditional update: set `values[col]` to `replacement` iff it currently
// equals `expected`. protocol.rs Request::CompareAndSwap, opcode 36, payload: u64 pk, u32
// col, u64 expected, u64 replacement. Returns the result, or nil/false when the server
// answers NotFound.
func (c *Client) CompareAndSwap(pk uint64, col uint32, expected uint64, replacement uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(expected).u64(replacement)
	return c.callOKOpt(OpCompareAndSwap, w.b)
}

// SetField: Set a single column of an existing row without providing all other column
// values. protocol.rs Request::SetField, opcode 37, payload: u64 pk, u32 col, u64 value.
// Returns the result, or nil/false when the server answers NotFound.
func (c *Client) SetField(pk uint64, col uint32, value uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(value)
	return c.callOKOpt(OpSetField, w.b)
}

// InsertIfAbsent: Insert a row only if the primary key is not already present.
// protocol.rs Request::InsertIfAbsent, opcode 38, payload: u64 pk, u64s values. Returns
// the affected / result count.
func (c *Client) InsertIfAbsent(pk uint64, values []uint64) (uint64, error) {
	w := &writer{}
	w.u64(pk).u64s(values)
	return c.callOK(OpInsertIfAbsent, w.b)
}

// DeleteRange: Delete all rows where column `col` falls in `[lo, hi]`. protocol.rs
// Request::DeleteRange, opcode 39, payload: u32 col, u64 lo, u64 hi. Returns the affected
// / result count.
func (c *Client) DeleteRange(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpDeleteRange, w.b)
}

// PopMin: Fetch and atomically delete the row with the minimum column value in `[lo,
// hi]`. protocol.rs Request::PopMin, opcode 40, payload: u32 col, u64 lo, u64 hi. Returns
// the row values, or nil/false when the server answers NotFound.
func (c *Client) PopMin(col uint32, lo uint64, hi uint64) ([]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callRowOpt(OpPopMin, w.b)
}

// PopMax: Fetch and atomically delete the row with the maximum column value in `[lo,
// hi]`. protocol.rs Request::PopMax, opcode 41, payload: u32 col, u64 lo, u64 hi. Returns
// the row values, or nil/false when the server answers NotFound.
func (c *Client) PopMax(col uint32, lo uint64, hi uint64) ([]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callRowOpt(OpPopMax, w.b)
}

// MultiSet: Set multiple `(col, value)` pairs on an existing row atomically. protocol.rs
// Request::MultiSet, opcode 42, payload: u64 pk, pairs pairs. Returns the result, or
// nil/false when the server answers NotFound.
func (c *Client) MultiSet(pk uint64, pairs []ColValue) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).pairs(pairs)
	return c.callOKOpt(OpMultiSet, w.b)
}

// DecrementClamp: Subtract `delta` from a column, clamping at 0 (saturating_sub).
// protocol.rs Request::DecrementClamp, opcode 43, payload: u64 pk, u32 col, u64 delta.
// Returns the result, or nil/false when the server answers NotFound.
func (c *Client) DecrementClamp(pk uint64, col uint32, delta uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(delta)
	return c.callOKOpt(OpDecrementClamp, w.b)
}

// GetOrInsert: Return existing row or insert-and-return provided values. protocol.rs
// Request::GetOrInsert, opcode 44, payload: u64 pk, u64s values. Returns the row values.
func (c *Client) GetOrInsert(pk uint64, values []uint64) ([]uint64, error) {
	w := &writer{}
	w.u64(pk).u64s(values)
	return c.callRow(OpGetOrInsert, w.b)
}

// BatchSetField: Set the same column to the same value across many rows in one round
// trip. protocol.rs Request::BatchSetField, opcode 45, payload: u32 col, u64 value, u64s
// pks. Returns the affected / result count.
func (c *Client) BatchSetField(col uint32, value uint64, pks []uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(value).u64s(pks)
	return c.callOK(OpBatchSetField, w.b)
}

// Top: Return the top-N rows by column value (highest first). protocol.rs Request::Top,
// opcode 46, payload: u32 col, u64 lo, u64 hi, u32 n. Returns the matching rows.
func (c *Client) Top(col uint32, lo uint64, hi uint64, n uint32) ([][]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi).u32(n)
	return c.callRows(OpTop, w.b)
}

// Bottom: Return the bottom-N rows by column value (lowest first). protocol.rs
// Request::Bottom, opcode 47, payload: u32 col, u64 lo, u64 hi, u32 n. Returns the
// matching rows.
func (c *Client) Bottom(col uint32, lo uint64, hi uint64, n uint32) ([][]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi).u32(n)
	return c.callRows(OpBottom, w.b)
}

// PkExists: Exact primary-key existence check (not probabilistic). protocol.rs
// Request::PkExists, opcode 48, payload: u64 pk. Returns whether the key exists.
func (c *Client) PkExists(pk uint64) (bool, error) {
	w := &writer{}
	w.u64(pk)
	return c.callExists(OpPkExists, w.b)
}

// IncrementMulti: Atomically increment multiple `(col, delta)` pairs on one row
// (saturating at u64::MAX). protocol.rs Request::IncrementMulti, opcode 49, payload: u64
// pk, pairs pairs. Returns the row values, or nil/false when the server answers NotFound.
func (c *Client) IncrementMulti(pk uint64, pairs []ColValue) ([]uint64, error) {
	w := &writer{}
	w.u64(pk).pairs(pairs)
	return c.callRowOpt(OpIncrementMulti, w.b)
}

// CopyRow: Copy the row at `src_pk` to `dst_pk` (insert-or-replace). protocol.rs
// Request::CopyRow, opcode 50, payload: u64 src_pk, u64 dst_pk. Returns the result, or
// nil/false when the server answers NotFound.
func (c *Client) CopyRow(srcPk uint64, dstPk uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(srcPk).u64(dstPk)
	return c.callOKOpt(OpCopyRow, w.b)
}

// SwapField: Swap the value of column `col` between `pk1` and `pk2`. protocol.rs
// Request::SwapField, opcode 51, payload: u64 pk1, u64 pk2, u32 col. Returns the result,
// or nil/false when the server answers NotFound.
func (c *Client) SwapField(pk1 uint64, pk2 uint64, col uint32) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk1).u64(pk2).u32(col)
	return c.callOKOpt(OpSwapField, w.b)
}

// Clamp: Clamp column `col` of row `pk` into `[lo, hi]`. protocol.rs Request::Clamp,
// opcode 52, payload: u64 pk, u32 col, u64 lo, u64 hi. Returns the result, or nil/false
// when the server answers NotFound.
func (c *Client) Clamp(pk uint64, col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpClamp, w.b)
}

// Average: Return `[sum, count]` for column `col` over the scan range `[lo, hi]`.
// protocol.rs Request::Average, opcode 53, payload: u32 col, u64 lo, u64 hi. Returns the
// row values, or nil/false when the server answers NotFound.
func (c *Client) Average(col uint32, lo uint64, hi uint64) ([]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callRowOpt(OpAverage, w.b)
}

// DeleteIf: Delete the row at `pk` only when `values[col] == value`. protocol.rs
// Request::DeleteIf, opcode 54, payload: u64 pk, u32 col, u64 value. Returns the result,
// or nil/false when the server answers NotFound.
func (c *Client) DeleteIf(pk uint64, col uint32, value uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(value)
	return c.callOKOpt(OpDeleteIf, w.b)
}

// SetFieldIf: Set `values[col]` to `new_value` only if it currently equals `expected`.
// protocol.rs Request::SetFieldIf, opcode 55, payload: u64 pk, u32 col, u64 expected, u64
// new_value. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) SetFieldIf(pk uint64, col uint32, expected uint64, newValue uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(expected).u64(newValue)
	return c.callOKOpt(OpSetFieldIf, w.b)
}

// RankOf: Return the 0-based rank of `pk`'s column `col` value within `[lo, hi]`.
// protocol.rs Request::RankOf, opcode 56, payload: u64 pk, u32 col, u64 lo, u64 hi.
// Returns the result, or nil/false when the server answers NotFound.
func (c *Client) RankOf(pk uint64, col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpRankOf, w.b)
}

// Percentile: Return the value at the Nth percentile of column `col` in `[lo, hi]`.
// protocol.rs Request::Percentile, opcode 57, payload: u32 col, u64 lo, u64 hi, u32 pct.
// Returns the result, or nil/false when the server answers NotFound.
func (c *Client) Percentile(col uint32, lo uint64, hi uint64, pct uint32) (uint64, bool, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi).u32(pct)
	return c.callOKOpt(OpPercentile, w.b)
}

// SumFields: Sum multiple column values of a single row. protocol.rs Request::SumFields,
// opcode 58, payload: u64 pk, u32s cols. Returns the result, or nil/false when the server
// answers NotFound.
func (c *Client) SumFields(pk uint64, cols []uint32) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32s(cols)
	return c.callOKOpt(OpSumFields, w.b)
}

// MaxField: Return the maximum across multiple column values of a single row. protocol.rs
// Request::MaxField, opcode 59, payload: u64 pk, u32s cols. Returns the result, or
// nil/false when the server answers NotFound.
func (c *Client) MaxField(pk uint64, cols []uint32) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32s(cols)
	return c.callOKOpt(OpMaxField, w.b)
}

// FilterScan: Range scan with an additional equality filter on a second column.
// protocol.rs Request::FilterScan, opcode 60, payload: u32 scan_col, u64 lo, u64 hi, u32
// filter_col, u64 filter_val. Returns the matching rows.
func (c *Client) FilterScan(scanCol uint32, lo uint64, hi uint64, filterCol uint32, filterVal uint64) ([][]uint64, error) {
	w := &writer{}
	w.u32(scanCol).u64(lo).u64(hi).u32(filterCol).u64(filterVal)
	return c.callRows(OpFilterScan, w.b)
}

// BatchIncrementField: Increment the same column by `delta` across many rows in one round
// trip. protocol.rs Request::BatchIncrementField, opcode 61, payload: u32 col, u64 delta,
// u64s pks. Returns the affected / result count.
func (c *Client) BatchIncrementField(col uint32, delta uint64, pks []uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(delta).u64s(pks)
	return c.callOK(OpBatchIncrementField, w.b)
}

// CountDistinct: Count distinct values of column `col` in scan range `[lo, hi]`.
// protocol.rs Request::CountDistinct, opcode 62, payload: u32 col, u64 lo, u64 hi.
// Returns the affected / result count.
func (c *Client) CountDistinct(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpCountDistinct, w.b)
}

// Median: Return the median value of column `col` in `[lo, hi]`. protocol.rs
// Request::Median, opcode 63, payload: u32 col, u64 lo, u64 hi. Returns the result, or
// nil/false when the server answers NotFound.
func (c *Client) Median(col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpMedian, w.b)
}

// BatchDeleteIf: Delete each pk in the list only if `values[col] == value`. protocol.rs
// Request::BatchDeleteIf, opcode 64, payload: u32 col, u64 value, u64s pks. Returns the
// affected / result count.
func (c *Client) BatchDeleteIf(col uint32, value uint64, pks []uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(value).u64s(pks)
	return c.callOK(OpBatchDeleteIf, w.b)
}

// BatchPkExists: Check multiple primary keys for existence in one round trip. protocol.rs
// Request::BatchPkExists, opcode 65, payload: u64s pks. Returns the affected / result
// count.
func (c *Client) BatchPkExists(pks []uint64) (uint64, error) {
	w := &writer{}
	w.u64s(pks)
	return c.callOK(OpBatchPkExists, w.b)
}

// FieldBetween: Range scan on `scan_col` in `[lo, hi]` keeping rows where `filter_col` in
// `[filter_lo, filter_hi]`. protocol.rs Request::FieldBetween, opcode 66, payload: u32
// scan_col, u64 lo, u64 hi, u32 filter_col, u64 filter_lo, u64 filter_hi. Returns the
// matching rows.
func (c *Client) FieldBetween(scanCol uint32, lo uint64, hi uint64, filterCol uint32, filterLo uint64, filterHi uint64) ([][]uint64, error) {
	w := &writer{}
	w.u32(scanCol).u64(lo).u64(hi).u32(filterCol).u64(filterLo).u64(filterHi)
	return c.callRows(OpFieldBetween, w.b)
}

// ScalarMin: Minimum value of column `col` over scan range `[lo, hi]`. protocol.rs
// Request::ScalarMin, opcode 67, payload: u32 col, u64 lo, u64 hi. Returns the result, or
// nil/false when the server answers NotFound.
func (c *Client) ScalarMin(col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpScalarMin, w.b)
}

// ScalarMax: Maximum value of column `col` over scan range `[lo, hi]`. protocol.rs
// Request::ScalarMax, opcode 68, payload: u32 col, u64 lo, u64 hi. Returns the result, or
// nil/false when the server answers NotFound.
func (c *Client) ScalarMax(col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpScalarMax, w.b)
}

// MultiGet: Return only the listed columns of a single row. protocol.rs
// Request::MultiGet, opcode 69, payload: u64 pk, u32s cols. Returns the row values, or
// nil/false when the server answers NotFound.
func (c *Client) MultiGet(pk uint64, cols []uint32) ([]uint64, error) {
	w := &writer{}
	w.u64(pk).u32s(cols)
	return c.callRowOpt(OpMultiGet, w.b)
}

// IncrementOrInsert: Add `delta` to column `col` if `pk` exists; otherwise insert
// `values` (with `values[col]` as initial). protocol.rs Request::IncrementOrInsert,
// opcode 70, payload: u64 pk, u32 col, u64 delta, u64s values. Returns the affected /
// result count.
func (c *Client) IncrementOrInsert(pk uint64, col uint32, delta uint64, values []uint64) (uint64, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(delta).u64s(values)
	return c.callOK(OpIncrementOrInsert, w.b)
}

// ZeroField: Set column `col` to 0 for every row in scan range `[lo, hi]`. protocol.rs
// Request::ZeroField, opcode 71, payload: u32 col, u64 lo, u64 hi. Returns the affected /
// result count.
func (c *Client) ZeroField(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpZeroField, w.b)
}

// ScanPage: Paginated range scan: skip `offset` rows then return up to `limit`.
// protocol.rs Request::ScanPage, opcode 72, payload: u32 col, u64 lo, u64 hi, u32 offset,
// u32 limit. Returns the matching rows.
func (c *Client) ScanPage(col uint32, lo uint64, hi uint64, offset uint32, limit uint32) ([][]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi).u32(offset).u32(limit)
	return c.callRows(OpScanPage, w.b)
}

// SumField: SumField. protocol.rs Request::SumField, opcode 73, payload: u32 col, u64 lo,
// u64 hi. Returns the affected / result count.
func (c *Client) SumField(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpSumField, w.b)
}

// BatchGet: BatchGet. protocol.rs Request::BatchGet, opcode 74, payload: u64s pks.
// Returns the matching rows.
func (c *Client) BatchGet(pks []uint64) ([][]uint64, error) {
	w := &writer{}
	w.u64s(pks)
	return c.callRows(OpBatchGet, w.b)
}

// SetFieldRange: SetFieldRange. protocol.rs Request::SetFieldRange, opcode 75, payload:
// u32 col, u64 value, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) SetFieldRange(col uint32, value uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(value).u64(lo).u64(hi)
	return c.callOK(OpSetFieldRange, w.b)
}

// MaxRow: MaxRow. protocol.rs Request::MaxRow, opcode 76, payload: u32 col, u64 lo, u64
// hi. Returns the row values, or nil/false when the server answers NotFound.
func (c *Client) MaxRow(col uint32, lo uint64, hi uint64) ([]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callRowOpt(OpMaxRow, w.b)
}

// MinRow: MinRow. protocol.rs Request::MinRow, opcode 77, payload: u32 col, u64 lo, u64
// hi. Returns the row values, or nil/false when the server answers NotFound.
func (c *Client) MinRow(col uint32, lo uint64, hi uint64) ([]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callRowOpt(OpMinRow, w.b)
}

// IncrementRange: IncrementRange. protocol.rs Request::IncrementRange, opcode 78,
// payload: u32 col, u64 delta, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) IncrementRange(col uint32, delta uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(delta).u64(lo).u64(hi)
	return c.callOK(OpIncrementRange, w.b)
}

// DeleteMin: DeleteMin. protocol.rs Request::DeleteMin, opcode 79, payload: u32 col, u64
// lo, u64 hi. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) DeleteMin(col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpDeleteMin, w.b)
}

// DeleteMax: DeleteMax. protocol.rs Request::DeleteMax, opcode 80, payload: u32 col, u64
// lo, u64 hi. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) DeleteMax(col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpDeleteMax, w.b)
}

// UniqueCount: UniqueCount. protocol.rs Request::UniqueCount, opcode 81, payload: u32
// col, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) UniqueCount(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpUniqueCount, w.b)
}

// TopN: TopN. protocol.rs Request::TopN, opcode 82, payload: u32 col, u32 n, u64 lo, u64
// hi. Returns the matching rows.
func (c *Client) TopN(col uint32, n uint32, lo uint64, hi uint64) ([][]uint64, error) {
	w := &writer{}
	w.u32(col).u32(n).u64(lo).u64(hi)
	return c.callRows(OpTopN, w.b)
}

// BottomN: BottomN. protocol.rs Request::BottomN, opcode 83, payload: u32 col, u32 n, u64
// lo, u64 hi. Returns the matching rows.
func (c *Client) BottomN(col uint32, n uint32, lo uint64, hi uint64) ([][]uint64, error) {
	w := &writer{}
	w.u32(col).u32(n).u64(lo).u64(hi)
	return c.callRows(OpBottomN, w.b)
}

// MoveField: MoveField. protocol.rs Request::MoveField, opcode 84, payload: u64 src_pk,
// u64 dst_pk, u32 col. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) MoveField(srcPk uint64, dstPk uint64, col uint32) (uint64, bool, error) {
	w := &writer{}
	w.u64(srcPk).u64(dstPk).u32(col)
	return c.callOKOpt(OpMoveField, w.b)
}

// CountIf: CountIf. protocol.rs Request::CountIf, opcode 85, payload: u32 col, u64 value,
// u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) CountIf(col uint32, value uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(value).u64(lo).u64(hi)
	return c.callOK(OpCountIf, w.b)
}

// SetIfGreater: SetIfGreater. protocol.rs Request::SetIfGreater, opcode 86, payload: u32
// col, u64 value, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) SetIfGreater(col uint32, value uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(value).u64(lo).u64(hi)
	return c.callOK(OpSetIfGreater, w.b)
}

// SetIfLess: SetIfLess. protocol.rs Request::SetIfLess, opcode 87, payload: u32 col, u64
// value, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) SetIfLess(col uint32, value uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(value).u64(lo).u64(hi)
	return c.callOK(OpSetIfLess, w.b)
}

// AbsDiff: AbsDiff. protocol.rs Request::AbsDiff, opcode 88, payload: u64 pk1, u64 pk2,
// u32 col. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) AbsDiff(pk1 uint64, pk2 uint64, col uint32) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk1).u64(pk2).u32(col)
	return c.callOKOpt(OpAbsDiff, w.b)
}

// SwapFields: SwapFields. protocol.rs Request::SwapFields, opcode 89, payload: u64 pk1,
// u64 pk2, u32 col. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) SwapFields(pk1 uint64, pk2 uint64, col uint32) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk1).u64(pk2).u32(col)
	return c.callOKOpt(OpSwapFields, w.b)
}

// CopyRange: CopyRange. protocol.rs Request::CopyRange, opcode 90, payload: u32 src_col,
// u32 dst_col, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) CopyRange(srcCol uint32, dstCol uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(srcCol).u32(dstCol).u64(lo).u64(hi)
	return c.callOK(OpCopyRange, w.b)
}

// PropagateMax: PropagateMax. protocol.rs Request::PropagateMax, opcode 91, payload: u32
// col, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) PropagateMax(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpPropagateMax, w.b)
}

// PropagateMin: PropagateMin. protocol.rs Request::PropagateMin, opcode 92, payload: u32
// col, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) PropagateMin(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpPropagateMin, w.b)
}

// ClampRange: ClampRange. protocol.rs Request::ClampRange, opcode 93, payload: u32 col,
// u64 min_val, u64 max_val, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) ClampRange(col uint32, minVal uint64, maxVal uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(minVal).u64(maxVal).u64(lo).u64(hi)
	return c.callOK(OpClampRange, w.b)
}

// BatchConditionalSet: BatchConditionalSet. protocol.rs Request::BatchConditionalSet,
// opcode 94, payload: u32 col, u64 expected, u64 new_value, u64 lo, u64 hi. Returns the
// affected / result count.
func (c *Client) BatchConditionalSet(col uint32, expected uint64, newValue uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(expected).u64(newValue).u64(lo).u64(hi)
	return c.callOK(OpBatchConditionalSet, w.b)
}

// WindowRank: WindowRank. protocol.rs Request::WindowRank, opcode 95, payload: u64 pk,
// u32 col, u64 lo, u64 hi. Returns the result, or nil/false when the server answers
// NotFound.
func (c *Client) WindowRank(pk uint64, col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpWindowRank, w.b)
}

// WindowPct: WindowPct. protocol.rs Request::WindowPct, opcode 96, payload: u64 pk, u32
// col, u64 lo, u64 hi. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) WindowPct(pk uint64, col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpWindowPct, w.b)
}

// SumProduct: SumProduct. protocol.rs Request::SumProduct, opcode 97, payload: u32 col_a,
// u32 col_b, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) SumProduct(colA uint32, colB uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(colA).u32(colB).u64(lo).u64(hi)
	return c.callOK(OpSumProduct, w.b)
}

// NthValue: NthValue. protocol.rs Request::NthValue, opcode 98, payload: u32 col, u32 n,
// u64 lo, u64 hi. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) NthValue(col uint32, n uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u32(col).u32(n).u64(lo).u64(hi)
	return c.callOKOpt(OpNthValue, w.b)
}

// DenseRankWin: DenseRankWin. protocol.rs Request::DenseRankWin, opcode 99, payload: u64
// pk, u32 col, u64 lo, u64 hi. Returns the result, or nil/false when the server answers
// NotFound.
func (c *Client) DenseRankWin(pk uint64, col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpDenseRankWin, w.b)
}

// SetFieldMany: SetFieldMany. protocol.rs Request::SetFieldMany, opcode 100, payload: u32
// col, u64 value, u64s pks. Returns the affected / result count.
func (c *Client) SetFieldMany(col uint32, value uint64, pks []uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(value).u64s(pks)
	return c.callOK(OpSetFieldMany, w.b)
}

// MultiColumnSum: MultiColumnSum. protocol.rs Request::MultiColumnSum, opcode 101,
// payload: u64s pks, u32 col. Returns the affected / result count.
func (c *Client) MultiColumnSum(pks []uint64, col uint32) (uint64, error) {
	w := &writer{}
	w.u64s(pks).u32(col)
	return c.callOK(OpMultiColumnSum, w.b)
}

// FieldHistogram: FieldHistogram. protocol.rs Request::FieldHistogram, opcode 102,
// payload: u32 col, u64 bucket_size, u64 lo, u64 hi. Returns the matching rows.
func (c *Client) FieldHistogram(col uint32, bucketSize uint64, lo uint64, hi uint64) ([][]uint64, error) {
	w := &writer{}
	w.u32(col).u64(bucketSize).u64(lo).u64(hi)
	return c.callRows(OpFieldHistogram, w.b)
}

// ZScoreFilter: ZScoreFilter. protocol.rs Request::ZScoreFilter, opcode 103, payload: u32
// col, u64 lo, u64 hi. Returns the matching rows.
func (c *Client) ZScoreFilter(col uint32, lo uint64, hi uint64) ([][]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callRows(OpZScoreFilter, w.b)
}

// RunningSum: RunningSum. protocol.rs Request::RunningSum, opcode 104, payload: u32 col,
// u64 lo, u64 hi. Returns the matching rows.
func (c *Client) RunningSum(col uint32, lo uint64, hi uint64) ([][]uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callRows(OpRunningSum, w.b)
}

// FloorDiv: FloorDiv. protocol.rs Request::FloorDiv, opcode 105, payload: u32 col, u64
// divisor, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) FloorDiv(col uint32, divisor uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(divisor).u64(lo).u64(hi)
	return c.callOK(OpFloorDiv, w.b)
}

// MulField: MulField. protocol.rs Request::MulField, opcode 106, payload: u32 col, u64
// factor, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) MulField(col uint32, factor uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(factor).u64(lo).u64(hi)
	return c.callOK(OpMulField, w.b)
}

// SetFieldAdd: SetFieldAdd. protocol.rs Request::SetFieldAdd, opcode 107, payload: u64
// pk, u32 col, u64 addend. Returns the result, or nil/false when the server answers
// NotFound.
func (c *Client) SetFieldAdd(pk uint64, col uint32, addend uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(addend)
	return c.callOKOpt(OpSetFieldAdd, w.b)
}

// SetFieldMul: SetFieldMul. protocol.rs Request::SetFieldMul, opcode 108, payload: u64
// pk, u32 col, u64 factor. Returns the result, or nil/false when the server answers
// NotFound.
func (c *Client) SetFieldMul(pk uint64, col uint32, factor uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(factor)
	return c.callOKOpt(OpSetFieldMul, w.b)
}

// SwapPkFields: SwapPkFields. protocol.rs Request::SwapPkFields, opcode 109, payload: u64
// pk, u32 col_a, u32 col_b. Returns the result, or nil/false when the server answers
// NotFound.
func (c *Client) SwapPkFields(pk uint64, colA uint32, colB uint32) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(colA).u32(colB)
	return c.callOKOpt(OpSwapPkFields, w.b)
}

// CountRange: CountRange. protocol.rs Request::CountRange, opcode 110, payload: u32 col,
// u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) CountRange(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpCountRange, w.b)
}

// SumRange: SumRange. protocol.rs Request::SumRange, opcode 111, payload: u32 col, u64
// lo, u64 hi. Returns the affected / result count.
func (c *Client) SumRange(col uint32, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOK(OpSumRange, w.b)
}

// MinRange: MinRange. protocol.rs Request::MinRange, opcode 112, payload: u32 col, u64
// lo, u64 hi. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) MinRange(col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpMinRange, w.b)
}

// MaxRange: MaxRange. protocol.rs Request::MaxRange, opcode 113, payload: u32 col, u64
// lo, u64 hi. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) MaxRange(col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpMaxRange, w.b)
}

// AvgRange: AvgRange. protocol.rs Request::AvgRange, opcode 114, payload: u32 col, u64
// lo, u64 hi. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) AvgRange(col uint32, lo uint64, hi uint64) (uint64, bool, error) {
	w := &writer{}
	w.u32(col).u64(lo).u64(hi)
	return c.callOKOpt(OpAvgRange, w.b)
}

// IncrementField: IncrementField. protocol.rs Request::IncrementField, opcode 115,
// payload: u64 pk, u32 col. Returns the result, or nil/false when the server answers
// NotFound.
func (c *Client) IncrementField(pk uint64, col uint32) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col)
	return c.callOKOpt(OpIncrementField, w.b)
}

// DecrementField: DecrementField. protocol.rs Request::DecrementField, opcode 116,
// payload: u64 pk, u32 col. Returns the result, or nil/false when the server answers
// NotFound.
func (c *Client) DecrementField(pk uint64, col uint32) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col)
	return c.callOKOpt(OpDecrementField, w.b)
}

// SetMin: SetMin. protocol.rs Request::SetMin, opcode 117, payload: u64 pk, u32 col, u64
// value. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) SetMin(pk uint64, col uint32, value uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(value)
	return c.callOKOpt(OpSetMin, w.b)
}

// SetMax: SetMax. protocol.rs Request::SetMax, opcode 118, payload: u64 pk, u32 col, u64
// value. Returns the result, or nil/false when the server answers NotFound.
func (c *Client) SetMax(pk uint64, col uint32, value uint64) (uint64, bool, error) {
	w := &writer{}
	w.u64(pk).u32(col).u64(value)
	return c.callOKOpt(OpSetMax, w.b)
}

// XorField: XorField. protocol.rs Request::XorField, opcode 119, payload: u32 col, u64
// mask, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) XorField(col uint32, mask uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(mask).u64(lo).u64(hi)
	return c.callOK(OpXorField, w.b)
}

// AndField: AndField. protocol.rs Request::AndField, opcode 120, payload: u32 col, u64
// mask, u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) AndField(col uint32, mask uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(mask).u64(lo).u64(hi)
	return c.callOK(OpAndField, w.b)
}

// OrField: OrField. protocol.rs Request::OrField, opcode 121, payload: u32 col, u64 mask,
// u64 lo, u64 hi. Returns the affected / result count.
func (c *Client) OrField(col uint32, mask uint64, lo uint64, hi uint64) (uint64, error) {
	w := &writer{}
	w.u32(col).u64(mask).u64(lo).u64(hi)
	return c.callOK(OpOrField, w.b)
}

// BatchGetField: BatchGetField. protocol.rs Request::BatchGetField, opcode 122, payload:
// u32 col, u64s pks. Returns the matching rows.
func (c *Client) BatchGetField(col uint32, pks []uint64) ([][]uint64, error) {
	w := &writer{}
	w.u32(col).u64s(pks)
	return c.callRows(OpBatchGetField, w.b)
}
