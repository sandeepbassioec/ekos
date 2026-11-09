package vectorvolt_test

// Golden-byte tests for every opcode wrapped in ops.go. Each case calls one wrapper
// against a mock server and checks the exact request frame (24-byte header + payload
// as laid out by vectorvolt/crates/vectorvolt-net/src/protocol.rs
// Request::encode_payload; request ids count 1..N on the one persistent connection)
// and the decoded canned reply (protocol.rs OpCode: Ok=132 Row=130 Rows=133
// Exists=137 NotFound=131 BatchResult=140 RowSet=142 Schema=143 Keys=144
// MinMaxResult=145).

import (
	"encoding/hex"
	"reflect"
	"testing"
	"time"

	vv "github.com/workerruntime/vectorvolt-go/vectorvolt"
)

type opCase struct {
	name   string
	call   func(c *vv.Client) (any, error)
	req    string
	respOp uint8
	resp   string
	want   any
}

var opCases = []opCase{
	// protocol.rs Request::Compact (opcode 7): (empty payload)
	{"Compact", func(c *vv.Client) (any, error) {
		v, err := c.Compact()
		return v, err
	},
		"565601070000000001000000000000000000000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::SecondaryLookup (opcode 11): u32 col_idx, u64 value
	{"SecondaryLookup", func(c *vv.Client) (any, error) {
		v, err := c.SecondaryLookup(11, 21)
		return v, err
	},
		"5656010b0000000002000000000000000c0000002e5d1c3e0b0000001500000000000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::FkExists (opcode 12): u64 value
	{"FkExists", func(c *vv.Client) (any, error) {
		v, err := c.FkExists(11)
		return v, err
	},
		"5656010c000000000300000000000000080000003fc348380b00000000000000",
		137, "01",
		true},
	// protocol.rs Request::Upsert (opcode 22): u64 pk, u64s values
	{"Upsert", func(c *vv.Client) (any, error) {
		v, err := c.Upsert(11, []uint64{102, 203})
		return v, err
	},
		"565601160000000004000000000000001c000000a851d3080b00000000000000020000006600000000000000cb00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::GetMany (opcode 23): u64s pks
	{"GetMany", func(c *vv.Client) (any, error) {
		v, err := c.GetMany([]uint64{101, 202})
		return v, err
	},
		"56560117000000000500000000000000140000006fe6656d020000006500000000000000ca00000000000000",
		142, "0100000001000000000000000200000002000000000000000300000000000000",
		[]vv.Row{{PK: 1, Values: []uint64{2, 3}}}},
	// protocol.rs Request::Update (opcode 24): u64 pk, u64s values
	{"Update", func(c *vv.Client) (any, error) {
		v, err := c.Update(11, []uint64{102, 203})
		return v, err
	},
		"565601180000000006000000000000001c000000a851d3080b00000000000000020000006600000000000000cb00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::Truncate (opcode 25): (empty payload)
	{"Truncate", func(c *vv.Client) (any, error) {
		v, err := c.Truncate()
		return v, err
	},
		"565601190000000007000000000000000000000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::Count (opcode 26): u32 col, u64 lo, u64 hi
	{"Count", func(c *vv.Client) (any, error) {
		v, err := c.Count(11, 21, 31)
		return v, err
	},
		"5656011a00000000080000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::BatchUpsert (opcode 27): rows rows
	{"BatchUpsert", func(c *vv.Client) (any, error) {
		v, err := c.BatchUpsert([]vv.Row{{PK: 5, Values: []uint64{6, 7}}, {PK: 8, Values: []uint64{}}})
		return v, err
	},
		"5656011b0000000009000000000000002c000000761480720200000005000000000000000200000006000000000000000700000000000000080000000000000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::BatchUpdate (opcode 28): rows rows
	{"BatchUpdate", func(c *vv.Client) (any, error) {
		s, k, err := c.BatchUpdate([]vv.Row{{PK: 5, Values: []uint64{6, 7}}, {PK: 8, Values: []uint64{}}})
		return []any{s, k}, err
	},
		"5656011c000000000a000000000000002c000000761480720200000005000000000000000200000006000000000000000700000000000000080000000000000000000000",
		140, "0400000005000000",
		[]any{uint32(4), uint32(5)}},
	// protocol.rs Request::DescribeTable (opcode 29): (empty payload)
	{"DescribeTable", func(c *vv.Client) (any, error) {
		v, err := c.DescribeTable()
		return v, err
	},
		"5656011d000000000b000000000000000000000000000000",
		143, "0200000002000000706b0100000061",
		[]string{"pk", "a"}},
	// protocol.rs Request::TableExists (opcode 30): string name
	{"TableExists", func(c *vv.Client) (any, error) {
		v, err := c.TableExists("tbl0")
		return v, err
	},
		"5656011e000000000c000000000000000800000011edf7a10400000074626c30",
		137, "01",
		true},
	// protocol.rs Request::ScanKeys (opcode 31): u32 col, u64 lo, u64 hi, u32 limit
	{"ScanKeys", func(c *vv.Client) (any, error) {
		v, err := c.ScanKeys(11, 21, 31, 41)
		return v, err
	},
		"5656011f000000000d0000000000000018000000b5b628410b00000015000000000000001f0000000000000029000000",
		144, "0200000009000000000000000800000000000000",
		[]uint64{9, 8}},
	// protocol.rs Request::MinMax (opcode 32): u32 col, u64 lo, u64 hi
	{"MinMax", func(c *vv.Client) (any, error) {
		mn, mx, n, ok, err := c.MinMax(11, 21, 31)
		return []any{mn, mx, n, ok}, err
	},
		"56560120000000000e0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		145, "050000000000000006000000000000000700000000000000",
		[]any{uint64(5), uint64(6), uint64(7), true}},
	// protocol.rs Request::MinMax (opcode 32): u32 col, u64 lo, u64 hi -- NotFound reply
	{"MinMax", func(c *vv.Client) (any, error) {
		mn, mx, n, ok, err := c.MinMax(11, 21, 31)
		return []any{mn, mx, n, ok}, err
	},
		"56560120000000000f0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]any{uint64(0), uint64(0), uint64(0), false}},
	// protocol.rs Request::Sum (opcode 33): u32 col, u64 lo, u64 hi
	{"Sum", func(c *vv.Client) (any, error) {
		v, err := c.Sum(11, 21, 31)
		return v, err
	},
		"5656012100000000100000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::GetAndDelete (opcode 34): u64 pk
	{"GetAndDelete", func(c *vv.Client) (any, error) {
		v, err := c.GetAndDelete(11)
		return v, err
	},
		"56560122000000001100000000000000080000003fc348380b00000000000000",
		130, "020000000b000000000000001600000000000000",
		[]uint64{11, 22}},
	// protocol.rs Request::GetAndDelete (opcode 34): u64 pk -- NotFound reply
	{"GetAndDelete", func(c *vv.Client) (any, error) {
		v, err := c.GetAndDelete(11)
		return v, err
	},
		"56560122000000001200000000000000080000003fc348380b00000000000000",
		131, "",
		[]uint64(nil)},
	// protocol.rs Request::Increment (opcode 35): u64 pk, u32 col, u64 delta
	{"Increment", func(c *vv.Client) (any, error) {
		v, ok, err := c.Increment(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560123000000001300000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::Increment (opcode 35): u64 pk, u32 col, u64 delta -- NotFound reply
	{"Increment", func(c *vv.Client) (any, error) {
		v, ok, err := c.Increment(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560123000000001400000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::CompareAndSwap (opcode 36): u64 pk, u32 col, u64 expected, u64 replacement
	{"CompareAndSwap", func(c *vv.Client) (any, error) {
		v, ok, err := c.CompareAndSwap(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"565601240000000015000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::CompareAndSwap (opcode 36): u64 pk, u32 col, u64 expected, u64 replacement -- NotFound reply
	{"CompareAndSwap", func(c *vv.Client) (any, error) {
		v, ok, err := c.CompareAndSwap(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"565601240000000016000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SetField (opcode 37): u64 pk, u32 col, u64 value
	{"SetField", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetField(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560125000000001700000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::SetField (opcode 37): u64 pk, u32 col, u64 value -- NotFound reply
	{"SetField", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetField(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560125000000001800000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::InsertIfAbsent (opcode 38): u64 pk, u64s values
	{"InsertIfAbsent", func(c *vv.Client) (any, error) {
		v, err := c.InsertIfAbsent(11, []uint64{102, 203})
		return v, err
	},
		"565601260000000019000000000000001c000000a851d3080b00000000000000020000006600000000000000cb00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::DeleteRange (opcode 39): u32 col, u64 lo, u64 hi
	{"DeleteRange", func(c *vv.Client) (any, error) {
		v, err := c.DeleteRange(11, 21, 31)
		return v, err
	},
		"56560127000000001a0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::PopMin (opcode 40): u32 col, u64 lo, u64 hi
	{"PopMin", func(c *vv.Client) (any, error) {
		v, err := c.PopMin(11, 21, 31)
		return v, err
	},
		"56560128000000001b0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		130, "020000000b000000000000001600000000000000",
		[]uint64{11, 22}},
	// protocol.rs Request::PopMin (opcode 40): u32 col, u64 lo, u64 hi -- NotFound reply
	{"PopMin", func(c *vv.Client) (any, error) {
		v, err := c.PopMin(11, 21, 31)
		return v, err
	},
		"56560128000000001c0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]uint64(nil)},
	// protocol.rs Request::PopMax (opcode 41): u32 col, u64 lo, u64 hi
	{"PopMax", func(c *vv.Client) (any, error) {
		v, err := c.PopMax(11, 21, 31)
		return v, err
	},
		"56560129000000001d0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		130, "020000000b000000000000001600000000000000",
		[]uint64{11, 22}},
	// protocol.rs Request::PopMax (opcode 41): u32 col, u64 lo, u64 hi -- NotFound reply
	{"PopMax", func(c *vv.Client) (any, error) {
		v, err := c.PopMax(11, 21, 31)
		return v, err
	},
		"56560129000000001e0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]uint64(nil)},
	// protocol.rs Request::MultiSet (opcode 42): u64 pk, pairs pairs
	{"MultiSet", func(c *vv.Client) (any, error) {
		v, ok, err := c.MultiSet(11, []vv.ColValue{{Col: 1, Value: 51}, {Col: 2, Value: 61}})
		return []any{v, ok}, err
	},
		"5656012a000000001f0000000000000024000000cf89d85b0b0000000000000002000000010000003300000000000000020000003d00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::MultiSet (opcode 42): u64 pk, pairs pairs -- NotFound reply
	{"MultiSet", func(c *vv.Client) (any, error) {
		v, ok, err := c.MultiSet(11, []vv.ColValue{{Col: 1, Value: 51}, {Col: 2, Value: 61}})
		return []any{v, ok}, err
	},
		"5656012a00000000200000000000000024000000cf89d85b0b0000000000000002000000010000003300000000000000020000003d00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::DecrementClamp (opcode 43): u64 pk, u32 col, u64 delta
	{"DecrementClamp", func(c *vv.Client) (any, error) {
		v, ok, err := c.DecrementClamp(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656012b000000002100000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::DecrementClamp (opcode 43): u64 pk, u32 col, u64 delta -- NotFound reply
	{"DecrementClamp", func(c *vv.Client) (any, error) {
		v, ok, err := c.DecrementClamp(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656012b000000002200000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::GetOrInsert (opcode 44): u64 pk, u64s values
	{"GetOrInsert", func(c *vv.Client) (any, error) {
		v, err := c.GetOrInsert(11, []uint64{102, 203})
		return v, err
	},
		"5656012c0000000023000000000000001c000000a851d3080b00000000000000020000006600000000000000cb00000000000000",
		130, "020000000b000000000000001600000000000000",
		[]uint64{11, 22}},
	// protocol.rs Request::BatchSetField (opcode 45): u32 col, u64 value, u64s pks
	{"BatchSetField", func(c *vv.Client) (any, error) {
		v, err := c.BatchSetField(11, 21, []uint64{103, 204})
		return v, err
	},
		"5656012d00000000240000000000000020000000f9e0430c0b0000001500000000000000020000006700000000000000cc00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::Top (opcode 46): u32 col, u64 lo, u64 hi, u32 n
	{"Top", func(c *vv.Client) (any, error) {
		v, err := c.Top(11, 21, 31, 41)
		return v, err
	},
		"5656012e00000000250000000000000018000000b5b628410b00000015000000000000001f0000000000000029000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::Bottom (opcode 47): u32 col, u64 lo, u64 hi, u32 n
	{"Bottom", func(c *vv.Client) (any, error) {
		v, err := c.Bottom(11, 21, 31, 41)
		return v, err
	},
		"5656012f00000000260000000000000018000000b5b628410b00000015000000000000001f0000000000000029000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::PkExists (opcode 48): u64 pk
	{"PkExists", func(c *vv.Client) (any, error) {
		v, err := c.PkExists(11)
		return v, err
	},
		"56560130000000002700000000000000080000003fc348380b00000000000000",
		137, "01",
		true},
	// protocol.rs Request::IncrementMulti (opcode 49): u64 pk, pairs pairs
	{"IncrementMulti", func(c *vv.Client) (any, error) {
		v, err := c.IncrementMulti(11, []vv.ColValue{{Col: 1, Value: 51}, {Col: 2, Value: 61}})
		return v, err
	},
		"5656013100000000280000000000000024000000cf89d85b0b0000000000000002000000010000003300000000000000020000003d00000000000000",
		130, "020000000b000000000000001600000000000000",
		[]uint64{11, 22}},
	// protocol.rs Request::IncrementMulti (opcode 49): u64 pk, pairs pairs -- NotFound reply
	{"IncrementMulti", func(c *vv.Client) (any, error) {
		v, err := c.IncrementMulti(11, []vv.ColValue{{Col: 1, Value: 51}, {Col: 2, Value: 61}})
		return v, err
	},
		"5656013100000000290000000000000024000000cf89d85b0b0000000000000002000000010000003300000000000000020000003d00000000000000",
		131, "",
		[]uint64(nil)},
	// protocol.rs Request::CopyRow (opcode 50): u64 src_pk, u64 dst_pk
	{"CopyRow", func(c *vv.Client) (any, error) {
		v, ok, err := c.CopyRow(11, 21)
		return []any{v, ok}, err
	},
		"56560132000000002a0000000000000010000000a75f3b820b000000000000001500000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::CopyRow (opcode 50): u64 src_pk, u64 dst_pk -- NotFound reply
	{"CopyRow", func(c *vv.Client) (any, error) {
		v, ok, err := c.CopyRow(11, 21)
		return []any{v, ok}, err
	},
		"56560132000000002b0000000000000010000000a75f3b820b000000000000001500000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SwapField (opcode 51): u64 pk1, u64 pk2, u32 col
	{"SwapField", func(c *vv.Client) (any, error) {
		v, ok, err := c.SwapField(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560133000000002c0000000000000014000000d0dd66a10b0000000000000015000000000000001f000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::SwapField (opcode 51): u64 pk1, u64 pk2, u32 col -- NotFound reply
	{"SwapField", func(c *vv.Client) (any, error) {
		v, ok, err := c.SwapField(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560133000000002d0000000000000014000000d0dd66a10b0000000000000015000000000000001f000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::Clamp (opcode 52): u64 pk, u32 col, u64 lo, u64 hi
	{"Clamp", func(c *vv.Client) (any, error) {
		v, ok, err := c.Clamp(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"56560134000000002e000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::Clamp (opcode 52): u64 pk, u32 col, u64 lo, u64 hi -- NotFound reply
	{"Clamp", func(c *vv.Client) (any, error) {
		v, ok, err := c.Clamp(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"56560134000000002f000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::Average (opcode 53): u32 col, u64 lo, u64 hi
	{"Average", func(c *vv.Client) (any, error) {
		v, err := c.Average(11, 21, 31)
		return v, err
	},
		"5656013500000000300000000000000014000000fac042040b00000015000000000000001f00000000000000",
		130, "020000000b000000000000001600000000000000",
		[]uint64{11, 22}},
	// protocol.rs Request::Average (opcode 53): u32 col, u64 lo, u64 hi -- NotFound reply
	{"Average", func(c *vv.Client) (any, error) {
		v, err := c.Average(11, 21, 31)
		return v, err
	},
		"5656013500000000310000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]uint64(nil)},
	// protocol.rs Request::DeleteIf (opcode 54): u64 pk, u32 col, u64 value
	{"DeleteIf", func(c *vv.Client) (any, error) {
		v, ok, err := c.DeleteIf(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560136000000003200000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::DeleteIf (opcode 54): u64 pk, u32 col, u64 value -- NotFound reply
	{"DeleteIf", func(c *vv.Client) (any, error) {
		v, ok, err := c.DeleteIf(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560136000000003300000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SetFieldIf (opcode 55): u64 pk, u32 col, u64 expected, u64 new_value
	{"SetFieldIf", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetFieldIf(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"565601370000000034000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::SetFieldIf (opcode 55): u64 pk, u32 col, u64 expected, u64 new_value -- NotFound reply
	{"SetFieldIf", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetFieldIf(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"565601370000000035000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::RankOf (opcode 56): u64 pk, u32 col, u64 lo, u64 hi
	{"RankOf", func(c *vv.Client) (any, error) {
		v, ok, err := c.RankOf(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"565601380000000036000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::RankOf (opcode 56): u64 pk, u32 col, u64 lo, u64 hi -- NotFound reply
	{"RankOf", func(c *vv.Client) (any, error) {
		v, ok, err := c.RankOf(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"565601380000000037000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::Percentile (opcode 57): u32 col, u64 lo, u64 hi, u32 pct
	{"Percentile", func(c *vv.Client) (any, error) {
		v, ok, err := c.Percentile(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"5656013900000000380000000000000018000000b5b628410b00000015000000000000001f0000000000000029000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::Percentile (opcode 57): u32 col, u64 lo, u64 hi, u32 pct -- NotFound reply
	{"Percentile", func(c *vv.Client) (any, error) {
		v, ok, err := c.Percentile(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"5656013900000000390000000000000018000000b5b628410b00000015000000000000001f0000000000000029000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SumFields (opcode 58): u64 pk, u32s cols
	{"SumFields", func(c *vv.Client) (any, error) {
		v, ok, err := c.SumFields(11, []uint32{3, 5})
		return []any{v, ok}, err
	},
		"5656013a000000003a0000000000000014000000bb69cd680b00000000000000020000000300000005000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::SumFields (opcode 58): u64 pk, u32s cols -- NotFound reply
	{"SumFields", func(c *vv.Client) (any, error) {
		v, ok, err := c.SumFields(11, []uint32{3, 5})
		return []any{v, ok}, err
	},
		"5656013a000000003b0000000000000014000000bb69cd680b00000000000000020000000300000005000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::MaxField (opcode 59): u64 pk, u32s cols
	{"MaxField", func(c *vv.Client) (any, error) {
		v, ok, err := c.MaxField(11, []uint32{3, 5})
		return []any{v, ok}, err
	},
		"5656013b000000003c0000000000000014000000bb69cd680b00000000000000020000000300000005000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::MaxField (opcode 59): u64 pk, u32s cols -- NotFound reply
	{"MaxField", func(c *vv.Client) (any, error) {
		v, ok, err := c.MaxField(11, []uint32{3, 5})
		return []any{v, ok}, err
	},
		"5656013b000000003d0000000000000014000000bb69cd680b00000000000000020000000300000005000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::FilterScan (opcode 60): u32 scan_col, u64 lo, u64 hi, u32 filter_col, u64 filter_val
	{"FilterScan", func(c *vv.Client) (any, error) {
		v, err := c.FilterScan(11, 21, 31, 41, 51)
		return v, err
	},
		"5656013c000000003e00000000000000200000000cdfbf3d0b00000015000000000000001f00000000000000290000003300000000000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::BatchIncrementField (opcode 61): u32 col, u64 delta, u64s pks
	{"BatchIncrementField", func(c *vv.Client) (any, error) {
		v, err := c.BatchIncrementField(11, 21, []uint64{103, 204})
		return v, err
	},
		"5656013d000000003f0000000000000020000000f9e0430c0b0000001500000000000000020000006700000000000000cc00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::CountDistinct (opcode 62): u32 col, u64 lo, u64 hi
	{"CountDistinct", func(c *vv.Client) (any, error) {
		v, err := c.CountDistinct(11, 21, 31)
		return v, err
	},
		"5656013e00000000400000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::Median (opcode 63): u32 col, u64 lo, u64 hi
	{"Median", func(c *vv.Client) (any, error) {
		v, ok, err := c.Median(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656013f00000000410000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::Median (opcode 63): u32 col, u64 lo, u64 hi -- NotFound reply
	{"Median", func(c *vv.Client) (any, error) {
		v, ok, err := c.Median(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656013f00000000420000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::BatchDeleteIf (opcode 64): u32 col, u64 value, u64s pks
	{"BatchDeleteIf", func(c *vv.Client) (any, error) {
		v, err := c.BatchDeleteIf(11, 21, []uint64{103, 204})
		return v, err
	},
		"5656014000000000430000000000000020000000f9e0430c0b0000001500000000000000020000006700000000000000cc00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::BatchPkExists (opcode 65): u64s pks
	{"BatchPkExists", func(c *vv.Client) (any, error) {
		v, err := c.BatchPkExists([]uint64{101, 202})
		return v, err
	},
		"56560141000000004400000000000000140000006fe6656d020000006500000000000000ca00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::FieldBetween (opcode 66): u32 scan_col, u64 lo, u64 hi, u32 filter_col, u64 filter_lo, u64 filter_hi
	{"FieldBetween", func(c *vv.Client) (any, error) {
		v, err := c.FieldBetween(11, 21, 31, 41, 51, 61)
		return v, err
	},
		"5656014200000000450000000000000028000000b122e3c30b00000015000000000000001f000000000000002900000033000000000000003d00000000000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::ScalarMin (opcode 67): u32 col, u64 lo, u64 hi
	{"ScalarMin", func(c *vv.Client) (any, error) {
		v, ok, err := c.ScalarMin(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656014300000000460000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::ScalarMin (opcode 67): u32 col, u64 lo, u64 hi -- NotFound reply
	{"ScalarMin", func(c *vv.Client) (any, error) {
		v, ok, err := c.ScalarMin(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656014300000000470000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::ScalarMax (opcode 68): u32 col, u64 lo, u64 hi
	{"ScalarMax", func(c *vv.Client) (any, error) {
		v, ok, err := c.ScalarMax(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656014400000000480000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::ScalarMax (opcode 68): u32 col, u64 lo, u64 hi -- NotFound reply
	{"ScalarMax", func(c *vv.Client) (any, error) {
		v, ok, err := c.ScalarMax(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656014400000000490000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::MultiGet (opcode 69): u64 pk, u32s cols
	{"MultiGet", func(c *vv.Client) (any, error) {
		v, err := c.MultiGet(11, []uint32{3, 5})
		return v, err
	},
		"56560145000000004a0000000000000014000000bb69cd680b00000000000000020000000300000005000000",
		130, "020000000b000000000000001600000000000000",
		[]uint64{11, 22}},
	// protocol.rs Request::MultiGet (opcode 69): u64 pk, u32s cols -- NotFound reply
	{"MultiGet", func(c *vv.Client) (any, error) {
		v, err := c.MultiGet(11, []uint32{3, 5})
		return v, err
	},
		"56560145000000004b0000000000000014000000bb69cd680b00000000000000020000000300000005000000",
		131, "",
		[]uint64(nil)},
	// protocol.rs Request::IncrementOrInsert (opcode 70): u64 pk, u32 col, u64 delta, u64s values
	{"IncrementOrInsert", func(c *vv.Client) (any, error) {
		v, err := c.IncrementOrInsert(11, 21, 31, []uint64{104, 205})
		return v, err
	},
		"56560146000000004c00000000000000280000006681ec570b00000000000000150000001f00000000000000020000006800000000000000cd00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::ZeroField (opcode 71): u32 col, u64 lo, u64 hi
	{"ZeroField", func(c *vv.Client) (any, error) {
		v, err := c.ZeroField(11, 21, 31)
		return v, err
	},
		"56560147000000004d0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::ScanPage (opcode 72): u32 col, u64 lo, u64 hi, u32 offset, u32 limit
	{"ScanPage", func(c *vv.Client) (any, error) {
		v, err := c.ScanPage(11, 21, 31, 41, 51)
		return v, err
	},
		"56560148000000004e000000000000001c000000f56b23740b00000015000000000000001f000000000000002900000033000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::SumField (opcode 73): u32 col, u64 lo, u64 hi
	{"SumField", func(c *vv.Client) (any, error) {
		v, err := c.SumField(11, 21, 31)
		return v, err
	},
		"56560149000000004f0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::BatchGet (opcode 74): u64s pks
	{"BatchGet", func(c *vv.Client) (any, error) {
		v, err := c.BatchGet([]uint64{101, 202})
		return v, err
	},
		"5656014a000000005000000000000000140000006fe6656d020000006500000000000000ca00000000000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::SetFieldRange (opcode 75): u32 col, u64 value, u64 lo, u64 hi
	{"SetFieldRange", func(c *vv.Client) (any, error) {
		v, err := c.SetFieldRange(11, 21, 31, 41)
		return v, err
	},
		"5656014b0000000051000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::MaxRow (opcode 76): u32 col, u64 lo, u64 hi
	{"MaxRow", func(c *vv.Client) (any, error) {
		v, err := c.MaxRow(11, 21, 31)
		return v, err
	},
		"5656014c00000000520000000000000014000000fac042040b00000015000000000000001f00000000000000",
		130, "020000000b000000000000001600000000000000",
		[]uint64{11, 22}},
	// protocol.rs Request::MaxRow (opcode 76): u32 col, u64 lo, u64 hi -- NotFound reply
	{"MaxRow", func(c *vv.Client) (any, error) {
		v, err := c.MaxRow(11, 21, 31)
		return v, err
	},
		"5656014c00000000530000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]uint64(nil)},
	// protocol.rs Request::MinRow (opcode 77): u32 col, u64 lo, u64 hi
	{"MinRow", func(c *vv.Client) (any, error) {
		v, err := c.MinRow(11, 21, 31)
		return v, err
	},
		"5656014d00000000540000000000000014000000fac042040b00000015000000000000001f00000000000000",
		130, "020000000b000000000000001600000000000000",
		[]uint64{11, 22}},
	// protocol.rs Request::MinRow (opcode 77): u32 col, u64 lo, u64 hi -- NotFound reply
	{"MinRow", func(c *vv.Client) (any, error) {
		v, err := c.MinRow(11, 21, 31)
		return v, err
	},
		"5656014d00000000550000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]uint64(nil)},
	// protocol.rs Request::IncrementRange (opcode 78): u32 col, u64 delta, u64 lo, u64 hi
	{"IncrementRange", func(c *vv.Client) (any, error) {
		v, err := c.IncrementRange(11, 21, 31, 41)
		return v, err
	},
		"5656014e0000000056000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::DeleteMin (opcode 79): u32 col, u64 lo, u64 hi
	{"DeleteMin", func(c *vv.Client) (any, error) {
		v, ok, err := c.DeleteMin(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656014f00000000570000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::DeleteMin (opcode 79): u32 col, u64 lo, u64 hi -- NotFound reply
	{"DeleteMin", func(c *vv.Client) (any, error) {
		v, ok, err := c.DeleteMin(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656014f00000000580000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::DeleteMax (opcode 80): u32 col, u64 lo, u64 hi
	{"DeleteMax", func(c *vv.Client) (any, error) {
		v, ok, err := c.DeleteMax(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656015000000000590000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::DeleteMax (opcode 80): u32 col, u64 lo, u64 hi -- NotFound reply
	{"DeleteMax", func(c *vv.Client) (any, error) {
		v, ok, err := c.DeleteMax(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560150000000005a0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::UniqueCount (opcode 81): u32 col, u64 lo, u64 hi
	{"UniqueCount", func(c *vv.Client) (any, error) {
		v, err := c.UniqueCount(11, 21, 31)
		return v, err
	},
		"56560151000000005b0000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::TopN (opcode 82): u32 col, u32 n, u64 lo, u64 hi
	{"TopN", func(c *vv.Client) (any, error) {
		v, err := c.TopN(11, 21, 31, 41)
		return v, err
	},
		"56560152000000005c000000000000001800000055046a6c0b000000150000001f000000000000002900000000000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::BottomN (opcode 83): u32 col, u32 n, u64 lo, u64 hi
	{"BottomN", func(c *vv.Client) (any, error) {
		v, err := c.BottomN(11, 21, 31, 41)
		return v, err
	},
		"56560153000000005d000000000000001800000055046a6c0b000000150000001f000000000000002900000000000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::MoveField (opcode 84): u64 src_pk, u64 dst_pk, u32 col
	{"MoveField", func(c *vv.Client) (any, error) {
		v, ok, err := c.MoveField(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560154000000005e0000000000000014000000d0dd66a10b0000000000000015000000000000001f000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::MoveField (opcode 84): u64 src_pk, u64 dst_pk, u32 col -- NotFound reply
	{"MoveField", func(c *vv.Client) (any, error) {
		v, ok, err := c.MoveField(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560154000000005f0000000000000014000000d0dd66a10b0000000000000015000000000000001f000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::CountIf (opcode 85): u32 col, u64 value, u64 lo, u64 hi
	{"CountIf", func(c *vv.Client) (any, error) {
		v, err := c.CountIf(11, 21, 31, 41)
		return v, err
	},
		"565601550000000060000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::SetIfGreater (opcode 86): u32 col, u64 value, u64 lo, u64 hi
	{"SetIfGreater", func(c *vv.Client) (any, error) {
		v, err := c.SetIfGreater(11, 21, 31, 41)
		return v, err
	},
		"565601560000000061000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::SetIfLess (opcode 87): u32 col, u64 value, u64 lo, u64 hi
	{"SetIfLess", func(c *vv.Client) (any, error) {
		v, err := c.SetIfLess(11, 21, 31, 41)
		return v, err
	},
		"565601570000000062000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::AbsDiff (opcode 88): u64 pk1, u64 pk2, u32 col
	{"AbsDiff", func(c *vv.Client) (any, error) {
		v, ok, err := c.AbsDiff(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656015800000000630000000000000014000000d0dd66a10b0000000000000015000000000000001f000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::AbsDiff (opcode 88): u64 pk1, u64 pk2, u32 col -- NotFound reply
	{"AbsDiff", func(c *vv.Client) (any, error) {
		v, ok, err := c.AbsDiff(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656015800000000640000000000000014000000d0dd66a10b0000000000000015000000000000001f000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SwapFields (opcode 89): u64 pk1, u64 pk2, u32 col
	{"SwapFields", func(c *vv.Client) (any, error) {
		v, ok, err := c.SwapFields(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656015900000000650000000000000014000000d0dd66a10b0000000000000015000000000000001f000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::SwapFields (opcode 89): u64 pk1, u64 pk2, u32 col -- NotFound reply
	{"SwapFields", func(c *vv.Client) (any, error) {
		v, ok, err := c.SwapFields(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656015900000000660000000000000014000000d0dd66a10b0000000000000015000000000000001f000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::CopyRange (opcode 90): u32 src_col, u32 dst_col, u64 lo, u64 hi
	{"CopyRange", func(c *vv.Client) (any, error) {
		v, err := c.CopyRange(11, 21, 31, 41)
		return v, err
	},
		"5656015a0000000067000000000000001800000055046a6c0b000000150000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::PropagateMax (opcode 91): u32 col, u64 lo, u64 hi
	{"PropagateMax", func(c *vv.Client) (any, error) {
		v, err := c.PropagateMax(11, 21, 31)
		return v, err
	},
		"5656015b00000000680000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::PropagateMin (opcode 92): u32 col, u64 lo, u64 hi
	{"PropagateMin", func(c *vv.Client) (any, error) {
		v, err := c.PropagateMin(11, 21, 31)
		return v, err
	},
		"5656015c00000000690000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::ClampRange (opcode 93): u32 col, u64 min_val, u64 max_val, u64 lo, u64 hi
	{"ClampRange", func(c *vv.Client) (any, error) {
		v, err := c.ClampRange(11, 21, 31, 41, 51)
		return v, err
	},
		"5656015d000000006a0000000000000024000000a55c87be0b00000015000000000000001f0000000000000029000000000000003300000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::BatchConditionalSet (opcode 94): u32 col, u64 expected, u64 new_value, u64 lo, u64 hi
	{"BatchConditionalSet", func(c *vv.Client) (any, error) {
		v, err := c.BatchConditionalSet(11, 21, 31, 41, 51)
		return v, err
	},
		"5656015e000000006b0000000000000024000000a55c87be0b00000015000000000000001f0000000000000029000000000000003300000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::WindowRank (opcode 95): u64 pk, u32 col, u64 lo, u64 hi
	{"WindowRank", func(c *vv.Client) (any, error) {
		v, ok, err := c.WindowRank(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"5656015f000000006c000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::WindowRank (opcode 95): u64 pk, u32 col, u64 lo, u64 hi -- NotFound reply
	{"WindowRank", func(c *vv.Client) (any, error) {
		v, ok, err := c.WindowRank(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"5656015f000000006d000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::WindowPct (opcode 96): u64 pk, u32 col, u64 lo, u64 hi
	{"WindowPct", func(c *vv.Client) (any, error) {
		v, ok, err := c.WindowPct(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"56560160000000006e000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::WindowPct (opcode 96): u64 pk, u32 col, u64 lo, u64 hi -- NotFound reply
	{"WindowPct", func(c *vv.Client) (any, error) {
		v, ok, err := c.WindowPct(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"56560160000000006f000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SumProduct (opcode 97): u32 col_a, u32 col_b, u64 lo, u64 hi
	{"SumProduct", func(c *vv.Client) (any, error) {
		v, err := c.SumProduct(11, 21, 31, 41)
		return v, err
	},
		"565601610000000070000000000000001800000055046a6c0b000000150000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::NthValue (opcode 98): u32 col, u32 n, u64 lo, u64 hi
	{"NthValue", func(c *vv.Client) (any, error) {
		v, ok, err := c.NthValue(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"565601620000000071000000000000001800000055046a6c0b000000150000001f000000000000002900000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::NthValue (opcode 98): u32 col, u32 n, u64 lo, u64 hi -- NotFound reply
	{"NthValue", func(c *vv.Client) (any, error) {
		v, ok, err := c.NthValue(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"565601620000000072000000000000001800000055046a6c0b000000150000001f000000000000002900000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::DenseRankWin (opcode 99): u64 pk, u32 col, u64 lo, u64 hi
	{"DenseRankWin", func(c *vv.Client) (any, error) {
		v, ok, err := c.DenseRankWin(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"565601630000000073000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::DenseRankWin (opcode 99): u64 pk, u32 col, u64 lo, u64 hi -- NotFound reply
	{"DenseRankWin", func(c *vv.Client) (any, error) {
		v, ok, err := c.DenseRankWin(11, 21, 31, 41)
		return []any{v, ok}, err
	},
		"565601630000000074000000000000001c000000c5c3b9ca0b00000000000000150000001f000000000000002900000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SetFieldMany (opcode 100): u32 col, u64 value, u64s pks
	{"SetFieldMany", func(c *vv.Client) (any, error) {
		v, err := c.SetFieldMany(11, 21, []uint64{103, 204})
		return v, err
	},
		"5656016400000000750000000000000020000000f9e0430c0b0000001500000000000000020000006700000000000000cc00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::MultiColumnSum (opcode 101): u64s pks, u32 col
	{"MultiColumnSum", func(c *vv.Client) (any, error) {
		v, err := c.MultiColumnSum([]uint64{101, 202}, 21)
		return v, err
	},
		"5656016500000000760000000000000018000000f1b120a3020000006500000000000000ca0000000000000015000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::FieldHistogram (opcode 102): u32 col, u64 bucket_size, u64 lo, u64 hi
	{"FieldHistogram", func(c *vv.Client) (any, error) {
		v, err := c.FieldHistogram(11, 21, 31, 41)
		return v, err
	},
		"565601660000000077000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::ZScoreFilter (opcode 103): u32 col, u64 lo, u64 hi
	{"ZScoreFilter", func(c *vv.Client) (any, error) {
		v, err := c.ZScoreFilter(11, 21, 31)
		return v, err
	},
		"5656016700000000780000000000000014000000fac042040b00000015000000000000001f00000000000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::RunningSum (opcode 104): u32 col, u64 lo, u64 hi
	{"RunningSum", func(c *vv.Client) (any, error) {
		v, err := c.RunningSum(11, 21, 31)
		return v, err
	},
		"5656016800000000790000000000000014000000fac042040b00000015000000000000001f00000000000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
	// protocol.rs Request::FloorDiv (opcode 105): u32 col, u64 divisor, u64 lo, u64 hi
	{"FloorDiv", func(c *vv.Client) (any, error) {
		v, err := c.FloorDiv(11, 21, 31, 41)
		return v, err
	},
		"56560169000000007a000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::MulField (opcode 106): u32 col, u64 factor, u64 lo, u64 hi
	{"MulField", func(c *vv.Client) (any, error) {
		v, err := c.MulField(11, 21, 31, 41)
		return v, err
	},
		"5656016a000000007b000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::SetFieldAdd (opcode 107): u64 pk, u32 col, u64 addend
	{"SetFieldAdd", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetFieldAdd(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656016b000000007c00000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::SetFieldAdd (opcode 107): u64 pk, u32 col, u64 addend -- NotFound reply
	{"SetFieldAdd", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetFieldAdd(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656016b000000007d00000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SetFieldMul (opcode 108): u64 pk, u32 col, u64 factor
	{"SetFieldMul", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetFieldMul(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656016c000000007e00000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::SetFieldMul (opcode 108): u64 pk, u32 col, u64 factor -- NotFound reply
	{"SetFieldMul", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetFieldMul(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656016c000000007f00000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SwapPkFields (opcode 109): u64 pk, u32 col_a, u32 col_b
	{"SwapPkFields", func(c *vv.Client) (any, error) {
		v, ok, err := c.SwapPkFields(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656016d000000008000000000000000100000006e18418a0b00000000000000150000001f000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::SwapPkFields (opcode 109): u64 pk, u32 col_a, u32 col_b -- NotFound reply
	{"SwapPkFields", func(c *vv.Client) (any, error) {
		v, ok, err := c.SwapPkFields(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656016d000000008100000000000000100000006e18418a0b00000000000000150000001f000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::CountRange (opcode 110): u32 col, u64 lo, u64 hi
	{"CountRange", func(c *vv.Client) (any, error) {
		v, err := c.CountRange(11, 21, 31)
		return v, err
	},
		"5656016e00000000820000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::SumRange (opcode 111): u32 col, u64 lo, u64 hi
	{"SumRange", func(c *vv.Client) (any, error) {
		v, err := c.SumRange(11, 21, 31)
		return v, err
	},
		"5656016f00000000830000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::MinRange (opcode 112): u32 col, u64 lo, u64 hi
	{"MinRange", func(c *vv.Client) (any, error) {
		v, ok, err := c.MinRange(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656017000000000840000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::MinRange (opcode 112): u32 col, u64 lo, u64 hi -- NotFound reply
	{"MinRange", func(c *vv.Client) (any, error) {
		v, ok, err := c.MinRange(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656017000000000850000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::MaxRange (opcode 113): u32 col, u64 lo, u64 hi
	{"MaxRange", func(c *vv.Client) (any, error) {
		v, ok, err := c.MaxRange(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656017100000000860000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::MaxRange (opcode 113): u32 col, u64 lo, u64 hi -- NotFound reply
	{"MaxRange", func(c *vv.Client) (any, error) {
		v, ok, err := c.MaxRange(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656017100000000870000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::AvgRange (opcode 114): u32 col, u64 lo, u64 hi
	{"AvgRange", func(c *vv.Client) (any, error) {
		v, ok, err := c.AvgRange(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656017200000000880000000000000014000000fac042040b00000015000000000000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::AvgRange (opcode 114): u32 col, u64 lo, u64 hi -- NotFound reply
	{"AvgRange", func(c *vv.Client) (any, error) {
		v, ok, err := c.AvgRange(11, 21, 31)
		return []any{v, ok}, err
	},
		"5656017200000000890000000000000014000000fac042040b00000015000000000000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::IncrementField (opcode 115): u64 pk, u32 col
	{"IncrementField", func(c *vv.Client) (any, error) {
		v, ok, err := c.IncrementField(11, 21)
		return []any{v, ok}, err
	},
		"56560173000000008a000000000000000c000000ccc5806d0b0000000000000015000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::IncrementField (opcode 115): u64 pk, u32 col -- NotFound reply
	{"IncrementField", func(c *vv.Client) (any, error) {
		v, ok, err := c.IncrementField(11, 21)
		return []any{v, ok}, err
	},
		"56560173000000008b000000000000000c000000ccc5806d0b0000000000000015000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::DecrementField (opcode 116): u64 pk, u32 col
	{"DecrementField", func(c *vv.Client) (any, error) {
		v, ok, err := c.DecrementField(11, 21)
		return []any{v, ok}, err
	},
		"56560174000000008c000000000000000c000000ccc5806d0b0000000000000015000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::DecrementField (opcode 116): u64 pk, u32 col -- NotFound reply
	{"DecrementField", func(c *vv.Client) (any, error) {
		v, ok, err := c.DecrementField(11, 21)
		return []any{v, ok}, err
	},
		"56560174000000008d000000000000000c000000ccc5806d0b0000000000000015000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SetMin (opcode 117): u64 pk, u32 col, u64 value
	{"SetMin", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetMin(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560175000000008e00000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::SetMin (opcode 117): u64 pk, u32 col, u64 value -- NotFound reply
	{"SetMin", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetMin(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560175000000008f00000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::SetMax (opcode 118): u64 pk, u32 col, u64 value
	{"SetMax", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetMax(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560176000000009000000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		132, "2a00000000000000",
		[]any{uint64(42), true}},
	// protocol.rs Request::SetMax (opcode 118): u64 pk, u32 col, u64 value -- NotFound reply
	{"SetMax", func(c *vv.Client) (any, error) {
		v, ok, err := c.SetMax(11, 21, 31)
		return []any{v, ok}, err
	},
		"56560176000000009100000000000000140000009eb9870c0b00000000000000150000001f00000000000000",
		131, "",
		[]any{uint64(0), false}},
	// protocol.rs Request::XorField (opcode 119): u32 col, u64 mask, u64 lo, u64 hi
	{"XorField", func(c *vv.Client) (any, error) {
		v, err := c.XorField(11, 21, 31, 41)
		return v, err
	},
		"565601770000000092000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::AndField (opcode 120): u32 col, u64 mask, u64 lo, u64 hi
	{"AndField", func(c *vv.Client) (any, error) {
		v, err := c.AndField(11, 21, 31, 41)
		return v, err
	},
		"565601780000000093000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::OrField (opcode 121): u32 col, u64 mask, u64 lo, u64 hi
	{"OrField", func(c *vv.Client) (any, error) {
		v, err := c.OrField(11, 21, 31, 41)
		return v, err
	},
		"565601790000000094000000000000001c000000ba3cbd960b00000015000000000000001f000000000000002900000000000000",
		132, "2a00000000000000",
		uint64(42)},
	// protocol.rs Request::BatchGetField (opcode 122): u32 col, u64s pks
	{"BatchGetField", func(c *vv.Client) (any, error) {
		v, err := c.BatchGetField(11, []uint64{102, 203})
		return v, err
	},
		"5656017a00000000950000000000000018000000389600ae0b000000020000006600000000000000cb00000000000000",
		133, "020000000200000001000000000000000200000000000000010000000300000000000000",
		[][]uint64{{1, 2}, {3}}},
}

func TestEveryWrappedOpcode(t *testing.T) {
	port, reqs := mockServer(t, func(op uint8, id uint64, body []byte) []byte {
		cs := opCases[id-1]
		return frame(id, cs.respOp, h(t, cs.resp))
	})
	c := vv.New("127.0.0.1", port)
	c.Timeout = 5 * time.Second
	if err := c.Connect(); err != nil {
		t.Fatal(err)
	}
	defer c.Close()
	for i, cs := range opCases {
		got, err := cs.call(c)
		if err != nil {
			t.Fatalf("%s (case %d): %v", cs.name, i+1, err)
		}
		if !reflect.DeepEqual(got, cs.want) {
			t.Errorf("%s (case %d): got %#v want %#v", cs.name, i+1, got, cs.want)
		}
		if hex.EncodeToString((*reqs)[i]) != cs.req {
			t.Errorf("%s (case %d): request frame\n got %x\nwant %s", cs.name, i+1, (*reqs)[i], cs.req)
		}
	}
	if len(*reqs) != len(opCases) {
		t.Fatalf("server saw %d requests, want %d", len(*reqs), len(opCases))
	}
}

func TestNotFoundOnNonOptionalOpIsProtocolError(t *testing.T) {
	port, _ := mockServer(t, func(op uint8, id uint64, body []byte) []byte {
		return frame(id, vv.OpNotFound, nil)
	})
	c := vv.New("127.0.0.1", port)
	c.Timeout = 5 * time.Second
	if err := c.Connect(); err != nil {
		t.Fatal(err)
	}
	defer c.Close()
	if _, err := c.Upsert(1, []uint64{2}); err == nil {
		t.Fatal("expected a protocol error")
	}
	if err := c.Ping(); err == nil {
		t.Fatal("connection must be dropped after an unexpected reply")
	}
}
