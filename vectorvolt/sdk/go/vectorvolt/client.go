// Package vectorvolt is a TCP client for the VectorVolt server.
//
// It speaks the real server protocol (see wire.go / protocol.rs): a 24-byte
// little-endian frame header with CRC-32 (IEEE) over the payload. The server is
// a key/value engine (u64 primary keys, rows of u64 columns, named tables,
// per-connection transactions) with no SQL endpoint, so Query/Execute return
// ErrNotSupported.
package vectorvolt

import (
	"encoding/binary"
	"fmt"
	"io"
	"net"
	"sync"
	"time"
)

// Client holds ONE persistent connection; calls are serialised on it. A
// server-side transaction is bound to the connection, so BeginTxn ...
// CommitTxn/RollbackTxn span calls on the same Client.
type Client struct {
	addr string
	mu   sync.Mutex
	conn net.Conn
	next uint64
	// Timeout bounds each call (0 = none).
	Timeout time.Duration
}

// New creates a Client targeting host:port.
func New(host string, port int) *Client {
	return &Client{addr: fmt.Sprintf("%s:%d", host, port), next: 1}
}

// Connect opens the TCP connection.
func (c *Client) Connect() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	conn, err := net.Dial("tcp", c.addr)
	if err != nil {
		return err
	}
	if tc, ok := conn.(*net.TCPConn); ok {
		_ = tc.SetNoDelay(true)
	}
	c.conn = conn
	c.next = 1
	return nil
}

// ConnectAuthenticated connects and runs the Hello/Auth handshake.
func (c *Client) ConnectAuthenticated(keyID string, secret []byte) error {
	if err := c.Connect(); err != nil {
		return err
	}
	if err := c.Authenticate(keyID, secret); err != nil {
		_ = c.Close()
		return err
	}
	return nil
}

// Close shuts down the connection (idempotent).
func (c *Client) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.dropLocked()
}

func (c *Client) dropLocked() error {
	if c.conn == nil {
		return nil
	}
	err := c.conn.Close()
	c.conn = nil
	return err
}

// call sends one request and returns the CRC-checked response. Error frames
// become *ServerError (connection kept); transport/framing failures drop the
// connection because the stream may be desynchronised.
func (c *Client) call(opcode uint8, payload []byte, expect ...uint8) (*Frame, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.conn == nil {
		return nil, fmt.Errorf("vectorvolt: not connected")
	}
	id := c.next
	c.next++
	if c.next == 0 {
		c.next = 1
	}
	req, err := EncodeFrame(id, opcode, payload)
	if err != nil {
		return nil, err
	}
	if c.Timeout > 0 {
		_ = c.conn.SetDeadline(time.Now().Add(c.Timeout))
	}
	f, err := c.roundTripLocked(id, req)
	if err != nil {
		_ = c.dropLocked()
		return nil, err
	}
	if f.Opcode == OpError {
		se, perr := ParseError(f.Payload)
		if perr != nil {
			_ = c.dropLocked()
			return nil, perr
		}
		return nil, se
	}
	for _, e := range expect {
		if f.Opcode == e {
			return f, nil
		}
	}
	_ = c.dropLocked()
	return nil, protoErr("unexpected response opcode %d", f.Opcode)
}

func (c *Client) roundTripLocked(id uint64, req []byte) (*Frame, error) {
	if _, err := c.conn.Write(req); err != nil {
		return nil, err
	}
	hdr := make([]byte, HeaderSize)
	if _, err := io.ReadFull(c.conn, hdr); err != nil {
		return nil, err
	}
	op, fl, rid, plen, crc, err := DecodeHeader(hdr)
	if err != nil {
		return nil, err
	}
	body := make([]byte, plen)
	if plen > 0 {
		if _, err := io.ReadFull(c.conn, body); err != nil {
			return nil, err
		}
	}
	if err := VerifyPayload(body, plen, crc); err != nil {
		return nil, err
	}
	if rid != id {
		return nil, protoErr("response request_id mismatch; stream desynchronised")
	}
	return &Frame{Opcode: op, Flags: fl, RequestID: rid, Payload: body}, nil
}

func affected(f *Frame) (uint64, error) {
	r := &reader{b: f.Payload}
	v, err := r.u64()
	if err != nil {
		return 0, err
	}
	return v, r.finish()
}

// Authenticate runs Hello -> Challenge(nonce); Auth(key_id, SHA3-256(secret || nonce)).
func (c *Client) Authenticate(keyID string, secret []byte) error {
	f, err := c.call(OpHello, nil, OpChallenge)
	if err != nil {
		return err
	}
	r := &reader{b: f.Payload}
	nonce, err := r.bytes()
	if err != nil {
		return err
	}
	if err := r.finish(); err != nil {
		return err
	}
	digest := sha3Sum256(append(append([]byte{}, secret...), nonce...))
	w := &writer{}
	w.str(keyID).bytes(digest[:])
	_, err = c.call(OpAuth, w.b, OpOk)
	return err
}

// Ping returns nil on Pong.
func (c *Client) Ping() error {
	_, err := c.call(OpPing, nil, OpPong)
	return err
}

// UseTable selects the active table for this connection.
func (c *Client) UseTable(name string) error {
	w := &writer{}
	_, err := c.call(OpUseTable, w.str(name).b, OpTableSelected)
	return err
}

// CreateTable creates a table (column names in schema order) and selects it.
func (c *Client) CreateTable(name string, schema []string) error {
	w := &writer{}
	_, err := c.call(OpCreateTable, w.str(name).strs(schema).b, OpTableSelected)
	return err
}

// DropTable drops a table.
func (c *Client) DropTable(name string) error {
	w := &writer{}
	_, err := c.call(OpDropTable, w.str(name).b, OpOk)
	return err
}

// ListTables lists table names.
func (c *Client) ListTables() ([]string, error) {
	f, err := c.call(OpListTables, nil, OpTableList)
	if err != nil {
		return nil, err
	}
	r := &reader{b: f.Payload}
	names, err := r.strs()
	if err != nil {
		return nil, err
	}
	return names, r.finish()
}

// Insert stores a row.
func (c *Client) Insert(pk uint64, values []uint64) error {
	w := &writer{}
	_, err := c.call(OpInsert, w.u64(pk).u64s(values).b, OpOk)
	return err
}

// Get fetches a row; (nil, nil) when the key does not exist.
func (c *Client) Get(pk uint64) ([]uint64, error) {
	w := &writer{}
	f, err := c.call(OpGet, w.u64(pk).b, OpRow, OpNotFound)
	if err != nil {
		return nil, err
	}
	if f.Opcode == OpNotFound {
		return nil, nil
	}
	r := &reader{b: f.Payload}
	v, err := r.u64s()
	if err != nil {
		return nil, err
	}
	return v, r.finish()
}

// Delete removes a row; reports whether one was deleted.
func (c *Client) Delete(pk uint64) (bool, error) {
	w := &writer{}
	f, err := c.call(OpDelete, w.u64(pk).b, OpOk)
	if err != nil {
		return false, err
	}
	n, err := affected(f)
	return n > 0, err
}

// RangeScan scans col in [lo, hi]; limit 0 = unlimited.
func (c *Client) RangeScan(col uint32, lo, hi uint64, limit uint32) ([][]uint64, error) {
	w := &writer{}
	f, err := c.call(OpRangeScan, w.u32(col).u64(lo).u64(hi).u32(limit).b, OpRows)
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

// Row is a (pk, values) pair for BatchInsert.
type Row struct {
	PK     uint64
	Values []uint64
}

func batchResult(f *Frame, err error) (succeeded, skipped uint32, e error) {
	if err != nil {
		return 0, 0, err
	}
	r := &reader{b: f.Payload}
	if succeeded, e = r.u32(); e != nil {
		return
	}
	if skipped, e = r.u32(); e != nil {
		return
	}
	e = r.finish()
	return
}

// BatchInsert inserts many rows in one round trip.
func (c *Client) BatchInsert(rows []Row) (succeeded, skipped uint32, err error) {
	w := &writer{}
	w.u32(uint32(len(rows)))
	for _, row := range rows {
		w.u64(row.PK).u64s(row.Values)
	}
	return batchResult(c.call(OpBatchInsert, w.b, OpBatchResult))
}

// BatchDelete deletes many keys in one round trip.
func (c *Client) BatchDelete(pks []uint64) (succeeded, skipped uint32, err error) {
	w := &writer{}
	w.u32(uint32(len(pks)))
	for _, pk := range pks {
		w.u64(pk)
	}
	return batchResult(c.call(OpBatchDelete, w.b, OpBatchResult))
}

// Flush flushes the active table.
func (c *Client) Flush() error {
	_, err := c.call(OpFlush, nil, OpOk)
	return err
}

// Stats returns (rows, partitions, bytes).
func (c *Client) Stats() (rows, partitions, bytes uint64, err error) {
	f, err := c.call(OpStats, nil, OpStatsReply)
	if err != nil {
		return
	}
	r := &reader{b: f.Payload}
	if rows, err = r.u64(); err != nil {
		return
	}
	if partitions, err = r.u64(); err != nil {
		return
	}
	if bytes, err = r.u64(); err != nil {
		return
	}
	err = r.finish()
	return
}

// BeginTxn starts a transaction bound to this connection.
func (c *Client) BeginTxn() (uint64, error) {
	f, err := c.call(OpBeginTxn, nil, OpTxnStarted)
	if err != nil {
		return 0, err
	}
	return affected(f)
}

// CommitTxn commits; returns the number of operations applied.
func (c *Client) CommitTxn(txnID uint64) (uint64, error) {
	var b [8]byte
	binary.LittleEndian.PutUint64(b[:], txnID)
	f, err := c.call(OpCommitTxn, b[:], OpOk)
	if err != nil {
		return 0, err
	}
	return affected(f)
}

// RollbackTxn discards the transaction.
func (c *Client) RollbackTxn(txnID uint64) error {
	var b [8]byte
	binary.LittleEndian.PutUint64(b[:], txnID)
	_, err := c.call(OpRollbackTxn, b[:], OpOk)
	return err
}

// Query is not supported: the server wire protocol has no SQL endpoint.
func (c *Client) Query(sql string) error { return ErrNotSupported }

// Execute is not supported: the server wire protocol has no SQL endpoint.
func (c *Client) Execute(sql string) error { return ErrNotSupported }
