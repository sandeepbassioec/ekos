package vectorvolt_test

// Golden-byte + mock-server tests. Expected hex is built from the frame
// definition in vectorvolt/crates/vectorvolt-net/src/protocol.rs: 24-byte LE
// header ("VV", version 1, opcode, flags, 3 reserved, u64 request_id,
// u32 payload_len, u32 CRC-32/IEEE of payload); payload fields little-endian,
// strings/lists u32-length-prefixed. Opcodes: Ping=1 Get=2 Insert=3 ...
// Pong=129 Row=130 Ok=132 Rows=133 Error=135 (protocol.rs OpCode).

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"errors"
	"hash/crc32"
	"io"
	"net"
	"strings"
	"testing"
	"time"

	vv "github.com/workerruntime/vectorvolt-go/vectorvolt"
)

const (
	// Request::Ping id=1, empty payload (CRC-32 of "" = 0).
	pingReq = "565601010000000001000000000000000000000000000000"
	// Request::Insert{pk:7, values:[10,20]} id=2.
	insertReq = "565601030000000002000000000000001c000000cb77f5420700000000000000" +
		"020000000a000000000000001400000000000000"
	// Request::Get{pk:7} id=3.
	getReq = "565601020000000003000000000000000800000070d6e76f0700000000000000"
	// Response::Row{values:[10,20]} id=3 (opcode 130 = 0x82).
	rowResp = "5656018200000000030000000000000014000000fc6859c9020000000a000000" +
		"000000001400000000000000"
	// Response::Error{code:4, message:"not found"} id=4 (opcode 135 = 0x87).
	errResp = "565601870000000004000000000000000f000000285977ce0400090000006e6f7420666f756e64"
	// Request::UseTable{name:"users"} id=5.
	useReq = "5656010d0000000005000000000000000900000015033f7e050000007573657273"
	// Request::CreateTable{name:"t", schema:["pk","a"]} id=6.
	createReq = "5656010e00000000060000000000000014000000c900811801000000740200000002" +
		"000000706b0100000061"
	// Request::RangeScan{col:1, lo:0, hi:u64::MAX, limit:10} id=7.
	scanReq = "56560105000000000700000000000000180000006556ae530100000000000000" +
		"00000000ffffffffffffffff0a000000"
	// Response::Rows{rows:[[5],[]]} id=7 (opcode 133 = 0x85).
	rowsResp = "5656018500000000070000000000000014000000800bd0f3020000000100000005" +
		"0000000000000000000000"
	// Request::CommitTxn{txn_id:9} id=8.
	commitReq = "565601140000000008000000000000000800000042c46d7a0900000000000000"
	// Response::Pong id=1.
	pongResp = "565601810000000001000000000000000000000000000000"
)

func h(t *testing.T, s string) []byte {
	t.Helper()
	b, err := hex.DecodeString(s)
	if err != nil {
		t.Fatal(err)
	}
	return b
}

func le(parts ...any) []byte {
	var b bytes.Buffer
	for _, p := range parts {
		binary.Write(&b, binary.LittleEndian, p)
	}
	return b.Bytes()
}

func lenStr(s string) []byte { return append(le(uint32(len(s))), s...) }

func TestCRCIsIEEE(t *testing.T) {
	if crc32.ChecksumIEEE([]byte("123456789")) != 0xCBF43926 {
		t.Fatal("not IEEE CRC-32")
	}
}

func TestEncodeMatchesGolden(t *testing.T) {
	cases := []struct {
		id      uint64
		op      uint8
		payload []byte
		want    string
	}{
		{1, vv.OpPing, nil, pingReq},
		{2, vv.OpInsert, le(uint64(7), uint32(2), uint64(10), uint64(20)), insertReq},
		{3, vv.OpGet, le(uint64(7)), getReq},
		{5, vv.OpUseTable, lenStr("users"), useReq},
		{6, vv.OpCreateTable, append(lenStr("t"), append(le(uint32(2)), append(lenStr("pk"), lenStr("a")...)...)...), createReq},
		{7, vv.OpRangeScan, le(uint32(1), uint64(0), ^uint64(0), uint32(10)), scanReq},
		{8, vv.OpCommitTxn, le(uint64(9)), commitReq},
	}
	for _, c := range cases {
		got, err := vv.EncodeFrame(c.id, c.op, c.payload)
		if err != nil {
			t.Fatal(err)
		}
		if hex.EncodeToString(got) != c.want {
			t.Errorf("op %d: got %x want %s", c.op, got, c.want)
		}
	}
}

func TestDecodeGolden(t *testing.T) {
	f, err := vv.DecodeFrame(h(t, rowResp))
	if err != nil || f.Opcode != vv.OpRow || f.RequestID != 3 {
		t.Fatalf("row: %v %+v", err, f)
	}
	if !bytes.Equal(f.Payload, le(uint32(2), uint64(10), uint64(20))) {
		t.Fatalf("payload %x", f.Payload)
	}
	if f, err = vv.DecodeFrame(h(t, pongResp)); err != nil || f.Opcode != vv.OpPong {
		t.Fatalf("pong: %v", err)
	}
	if f, err = vv.DecodeFrame(h(t, rowsResp)); err != nil || f.Opcode != vv.OpRows {
		t.Fatalf("rows: %v", err)
	}
}

func TestErrorFrame(t *testing.T) {
	f, err := vv.DecodeFrame(h(t, errResp))
	if err != nil || f.Opcode != vv.OpError {
		t.Fatalf("%v", err)
	}
	se, err := vv.ParseError(f.Payload)
	if err != nil || se.Code != 4 || se.Message != "not found" {
		t.Fatalf("%v %+v", err, se)
	}
	if !strings.Contains(se.Error(), "NOT_FOUND") {
		t.Fatal(se.Error())
	}
}

func TestRejectsBadFrames(t *testing.T) {
	good := h(t, pingReq)
	mut := func(i int, v byte) []byte { b := append([]byte{}, good...); b[i] = v; return b }
	for name, b := range map[string][]byte{
		"magic":     mut(0, 0xFF),
		"version":   mut(2, 42),
		"short hdr": good[:23],
	} {
		if _, err := vv.DecodeFrame(b); !errors.Is(err, vv.ErrProtocol) {
			t.Errorf("%s: %v", name, err)
		}
	}
	g := h(t, getReq)
	g[len(g)-1] ^= 0xFF
	if _, err := vv.DecodeFrame(g); !errors.Is(err, vv.ErrProtocol) {
		t.Errorf("crc: %v", err)
	}
	if _, err := vv.DecodeFrame(h(t, getReq)[:len(g)-1]); !errors.Is(err, vv.ErrProtocol) {
		t.Errorf("truncated: %v", err)
	}
	big := append([]byte{}, good...)
	binary.LittleEndian.PutUint32(big[16:], vv.MaxPayload+1)
	if _, err := vv.DecodeFrame(big); !errors.Is(err, vv.ErrProtocol) {
		t.Errorf("cap: %v", err)
	}
}

// mockServer serves one connection with handler; returns the port.
func mockServer(t *testing.T, handler func(op uint8, id uint64, body []byte) []byte) (int, *[][]byte) {
	t.Helper()
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { ln.Close() })
	var reqs [][]byte
	go func() {
		c, err := ln.Accept()
		if err != nil {
			return
		}
		defer c.Close()
		for {
			hdr := make([]byte, vv.HeaderSize)
			if _, err := io.ReadFull(c, hdr); err != nil {
				return
			}
			op, _, id, plen, _, err := vv.DecodeHeader(hdr)
			if err != nil {
				return
			}
			body := make([]byte, plen)
			if _, err := io.ReadFull(c, body); err != nil {
				return
			}
			reqs = append(reqs, append(append([]byte{}, hdr...), body...))
			if _, err := c.Write(handler(op, id, body)); err != nil {
				return
			}
		}
	}()
	return ln.Addr().(*net.TCPAddr).Port, &reqs
}

func frame(id uint64, op uint8, p []byte) []byte {
	b, _ := vv.EncodeFrame(id, op, p)
	return b
}

func TestClientRoundTrip(t *testing.T) {
	port, reqs := mockServer(t, func(op uint8, id uint64, body []byte) []byte {
		switch op {
		case vv.OpPing:
			return frame(id, vv.OpPong, nil)
		case vv.OpInsert:
			return frame(id, vv.OpOk, le(uint64(1)))
		case vv.OpGet:
			if binary.LittleEndian.Uint64(body) == 7 {
				return frame(id, vv.OpRow, le(uint32(2), uint64(10), uint64(20)))
			}
			return frame(id, vv.OpNotFound, nil)
		case vv.OpDelete:
			return frame(id, vv.OpOk, le(uint64(0)))
		case vv.OpRangeScan:
			return frame(id, vv.OpRows, le(uint32(2), uint32(1), uint64(5), uint32(0)))
		case vv.OpBeginTxn:
			return frame(id, vv.OpTxnStarted, le(uint64(9)))
		case vv.OpCommitTxn:
			return frame(id, vv.OpOk, le(uint64(3)))
		}
		return frame(id, vv.OpError, append(le(uint16(15), uint32(3)), "bad"...))
	})
	c := vv.New("127.0.0.1", port)
	c.Timeout = 5 * time.Second
	if err := c.Connect(); err != nil {
		t.Fatal(err)
	}
	defer c.Close()

	if err := c.Ping(); err != nil {
		t.Fatal(err)
	}
	if err := c.Insert(7, []uint64{10, 20}); err != nil {
		t.Fatal(err)
	}
	// Second request on the same connection carries request_id 2 (golden insert frame).
	if hex.EncodeToString((*reqs)[1]) != insertReq {
		t.Errorf("insert frame %x", (*reqs)[1])
	}
	if v, err := c.Get(7); err != nil || len(v) != 2 || v[0] != 10 || v[1] != 20 {
		t.Fatalf("get %v %v", v, err)
	}
	if v, err := c.Get(8); err != nil || v != nil {
		t.Fatalf("get miss %v %v", v, err)
	}
	if ok, err := c.Delete(8); err != nil || ok {
		t.Fatalf("delete %v %v", ok, err)
	}
	rows, err := c.RangeScan(1, 0, ^uint64(0), 10)
	if err != nil || len(rows) != 2 || len(rows[0]) != 1 || rows[0][0] != 5 || len(rows[1]) != 0 {
		t.Fatalf("scan %v %v", rows, err)
	}
	id, err := c.BeginTxn()
	if err != nil || id != 9 {
		t.Fatalf("begin %v %v", id, err)
	}
	if n, err := c.CommitTxn(id); err != nil || n != 3 {
		t.Fatalf("commit %v %v", n, err)
	}
	var se *vv.ServerError
	if err := c.Flush(); !errors.As(err, &se) || se.Code != 15 {
		t.Fatalf("flush err %v", err)
	}
	if err := c.Ping(); err != nil { // Error frame keeps the connection
		t.Fatalf("ping after error frame: %v", err)
	}
}

func TestAuthHandshake(t *testing.T) {
	nonce := make([]byte, 32)
	for i := range nonce {
		nonce[i] = byte(i)
	}
	// SHA3-256("secret" || 00..1f), computed with Python hashlib.sha3_256.
	want, _ := hex.DecodeString("8d8308f62b86ad04269ee6bb23d09100344842bef9f1d6839ca2278612b67d34")
	port, _ := mockServer(t, func(op uint8, id uint64, body []byte) []byte {
		if op == vv.OpHello {
			return frame(id, vv.OpChallenge, append(le(uint32(32)), nonce...))
		}
		exp := append(append(lenStr("app1"), le(uint32(32))...), want...)
		if bytes.Equal(body, exp) {
			return frame(id, vv.OpOk, le(uint64(0)))
		}
		return frame(id, vv.OpError, append(le(uint16(7), uint32(1)), 'x'))
	})
	c := vv.New("127.0.0.1", port)
	if err := c.ConnectAuthenticated("app1", []byte("secret")); err != nil {
		t.Fatal(err)
	}
	c.Close()
}

func TestSHA3KnownVector(t *testing.T) {
	// SHA3-256("") (FIPS 202). Exercised through a handshake with empty secret+nonce.
	nonce := []byte{}
	want, _ := hex.DecodeString("a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a")
	port, _ := mockServer(t, func(op uint8, id uint64, body []byte) []byte {
		if op == vv.OpHello {
			return frame(id, vv.OpChallenge, append(le(uint32(0)), nonce...))
		}
		if bytes.HasSuffix(body, want) {
			return frame(id, vv.OpOk, le(uint64(0)))
		}
		return frame(id, vv.OpError, append(le(uint16(7), uint32(1)), 'x'))
	})
	c := vv.New("127.0.0.1", port)
	if err := c.ConnectAuthenticated("k", nil); err != nil {
		t.Fatal(err)
	}
	c.Close()
}

func TestBadCRCDropsConnection(t *testing.T) {
	port, _ := mockServer(t, func(op uint8, id uint64, body []byte) []byte {
		f := frame(id, vv.OpRow, le(uint32(1), uint64(1)))
		f[len(f)-1] ^= 0xFF
		return f
	})
	c := vv.New("127.0.0.1", port)
	c.Timeout = 5 * time.Second
	_ = c.Connect()
	if _, err := c.Get(1); !errors.Is(err, vv.ErrProtocol) {
		t.Fatalf("want protocol error, got %v", err)
	}
	if err := c.Ping(); err == nil || !strings.Contains(err.Error(), "not connected") {
		t.Fatalf("connection should be dropped: %v", err)
	}
}

func TestSQLNotSupported(t *testing.T) {
	c := vv.New("127.0.0.1", 1)
	if !errors.Is(c.Query("SELECT 1"), vv.ErrNotSupported) || !errors.Is(c.Execute("x"), vv.ErrNotSupported) {
		t.Fatal("expected ErrNotSupported")
	}
}
