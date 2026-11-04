package vectorvolt

// Wire codec (pure, no I/O). Mirrors
// vectorvolt/crates/vectorvolt-net/src/protocol.rs -- the framing the server
// (server.rs) and the Rust SDK client (client.rs) actually speak.
//
// Frame = 24-byte little-endian header + payload:
//
//	off size field
//	  0    2 magic        "VV"
//	  2    1 version      1
//	  3    1 opcode
//	  4    1 flags        0
//	  5    3 reserved     0
//	  8    8 request_id   echoed by the server
//	 16    4 payload_len  <= 16 MiB
//	 20    4 payload_crc  CRC-32 IEEE (hash/crc32.ChecksumIEEE) of the payload

import (
	"encoding/binary"
	"errors"
	"fmt"
	"hash/crc32"
)

const (
	HeaderSize = 24
	MaxPayload = 16 * 1024 * 1024
	version    = 1
)

var magic = [2]byte{'V', 'V'}

// Request opcodes (protocol.rs OpCode).
const (
	OpPing        = 1
	OpGet         = 2
	OpInsert      = 3
	OpDelete      = 4
	OpRangeScan   = 5
	OpFlush       = 6
	OpCompact     = 7
	OpStats       = 8
	OpHello       = 9
	OpAuth        = 10
	OpUseTable    = 13
	OpCreateTable = 14
	OpDropTable   = 15
	OpListTables  = 16
	OpBatchInsert = 17
	OpBatchDelete = 18
	OpBeginTxn    = 19
	OpCommitTxn   = 20
	OpRollbackTxn = 21
)

// Response opcodes.
const (
	OpPong          = 129
	OpRow           = 130
	OpNotFound      = 131
	OpOk            = 132
	OpRows          = 133
	OpStatsReply    = 134
	OpError         = 135
	OpChallenge     = 136
	OpExists        = 137
	OpTableSelected = 138
	OpTableList     = 139
	OpBatchResult   = 140
	OpTxnStarted    = 141
)

// Server error codes (vectorvolt-net lib.rs error_code).
var errNames = map[uint16]string{
	1: "ENGINE", 2: "MALFORMED", 3: "DUPLICATE_KEY", 4: "NOT_FOUND", 5: "SCHEMA",
	6: "UNAUTHENTICATED", 7: "AUTH_FAILED", 8: "FORBIDDEN", 9: "NOT_SELECTED",
	10: "TABLE_NOT_FOUND", 11: "TABLE_EXISTS", 12: "TXN_ACTIVE", 13: "NO_TXN",
	14: "TXN_ID_MISMATCH", 15: "INVALID_ARGUMENT",
}

// ErrNotSupported is returned for operations the server protocol lacks (SQL).
var ErrNotSupported = errors.New("vectorvolt: not supported by the server wire protocol")

// ErrProtocol wraps malformed / unexpected wire data (connection is unusable).
var ErrProtocol = errors.New("vectorvolt: protocol error")

func protoErr(format string, a ...any) error {
	return fmt.Errorf("%w: %s", ErrProtocol, fmt.Sprintf(format, a...))
}

// ServerError is an Error frame returned by the server.
type ServerError struct {
	Code    uint16
	Message string
}

func (e *ServerError) Error() string {
	n := errNames[e.Code]
	if n == "" {
		n = "UNKNOWN"
	}
	return fmt.Sprintf("vectorvolt: server error [%d %s] %s", e.Code, n, e.Message)
}

// Frame is a decoded frame.
type Frame struct {
	Opcode    uint8
	Flags     uint8
	RequestID uint64
	Payload   []byte
}

// EncodeFrame serialises header + payload.
func EncodeFrame(requestID uint64, opcode uint8, payload []byte) ([]byte, error) {
	if len(payload) > MaxPayload {
		return nil, protoErr("payload %d exceeds max %d", len(payload), MaxPayload)
	}
	out := make([]byte, HeaderSize, HeaderSize+len(payload))
	out[0], out[1] = magic[0], magic[1]
	out[2] = version
	out[3] = opcode
	binary.LittleEndian.PutUint64(out[8:], requestID)
	binary.LittleEndian.PutUint32(out[16:], uint32(len(payload)))
	binary.LittleEndian.PutUint32(out[20:], crc32.ChecksumIEEE(payload))
	return append(out, payload...), nil
}

// DecodeHeader validates a 24-byte header.
func DecodeHeader(buf []byte) (opcode, flags uint8, requestID uint64, payloadLen, crc uint32, err error) {
	if len(buf) < HeaderSize {
		err = protoErr("truncated header: need %d, got %d", HeaderSize, len(buf))
		return
	}
	if buf[0] != magic[0] || buf[1] != magic[1] {
		err = protoErr("bad magic")
		return
	}
	if buf[2] != version {
		err = protoErr("unsupported protocol version %d", buf[2])
		return
	}
	opcode, flags = buf[3], buf[4]
	requestID = binary.LittleEndian.Uint64(buf[8:])
	payloadLen = binary.LittleEndian.Uint32(buf[16:])
	crc = binary.LittleEndian.Uint32(buf[20:])
	if payloadLen > MaxPayload {
		err = protoErr("payload %d exceeds max %d", payloadLen, MaxPayload)
	}
	return
}

// VerifyPayload checks length and CRC against the header values.
func VerifyPayload(payload []byte, payloadLen, crc uint32) error {
	if uint32(len(payload)) != payloadLen {
		return protoErr("truncated payload: need %d, got %d", payloadLen, len(payload))
	}
	if actual := crc32.ChecksumIEEE(payload); actual != crc {
		return protoErr("CRC mismatch: header %#08x, payload %#08x", crc, actual)
	}
	return nil
}

// DecodeFrame decodes one whole frame (header + payload), checking the CRC.
func DecodeFrame(buf []byte) (*Frame, error) {
	op, fl, rid, plen, crc, err := DecodeHeader(buf)
	if err != nil {
		return nil, err
	}
	if len(buf) < HeaderSize+int(plen) {
		return nil, protoErr("truncated payload: need %d, got %d", plen, len(buf)-HeaderSize)
	}
	p := buf[HeaderSize : HeaderSize+int(plen)]
	if err := VerifyPayload(p, plen, crc); err != nil {
		return nil, err
	}
	return &Frame{Opcode: op, Flags: fl, RequestID: rid, Payload: append([]byte(nil), p...)}, nil
}

// ---- payload writers ------------------------------------------------------

type writer struct{ b []byte }

func (w *writer) u32(v uint32) *writer { w.b = binary.LittleEndian.AppendUint32(w.b, v); return w }
func (w *writer) u64(v uint64) *writer { w.b = binary.LittleEndian.AppendUint64(w.b, v); return w }
func (w *writer) u64s(vs []uint64) *writer {
	w.u32(uint32(len(vs)))
	for _, v := range vs {
		w.u64(v)
	}
	return w
}
func (w *writer) bytes(b []byte) *writer { w.u32(uint32(len(b))); w.b = append(w.b, b...); return w }
func (w *writer) str(s string) *writer   { return w.bytes([]byte(s)) }
func (w *writer) strs(ss []string) *writer {
	w.u32(uint32(len(ss)))
	for _, s := range ss {
		w.str(s)
	}
	return w
}

// ---- payload reader (bounds-checked) --------------------------------------

type reader struct {
	b  []byte
	at int
}

func (r *reader) take(n int) ([]byte, error) {
	if n < 0 || r.at+n > len(r.b) {
		return nil, protoErr("payload truncated")
	}
	s := r.b[r.at : r.at+n]
	r.at += n
	return s, nil
}
func (r *reader) u16() (uint16, error) {
	b, err := r.take(2)
	if err != nil {
		return 0, err
	}
	return binary.LittleEndian.Uint16(b), nil
}
func (r *reader) u32() (uint32, error) {
	b, err := r.take(4)
	if err != nil {
		return 0, err
	}
	return binary.LittleEndian.Uint32(b), nil
}
func (r *reader) u64() (uint64, error) {
	b, err := r.take(8)
	if err != nil {
		return 0, err
	}
	return binary.LittleEndian.Uint64(b), nil
}
func (r *reader) u64s() ([]uint64, error) {
	n, err := r.u32()
	if err != nil {
		return nil, err
	}
	if int(n) > (len(r.b)-r.at)/8 {
		return nil, protoErr("declared more values than the payload holds")
	}
	out := make([]uint64, n)
	for i := range out {
		out[i], _ = r.u64()
	}
	return out, nil
}
func (r *reader) bytes() ([]byte, error) {
	n, err := r.u32()
	if err != nil {
		return nil, err
	}
	if int(n) > len(r.b)-r.at {
		return nil, protoErr("payload truncated")
	}
	return r.take(int(n))
}
func (r *reader) str() (string, error) {
	b, err := r.bytes()
	return string(b), err
}
func (r *reader) strs() ([]string, error) {
	n, err := r.u32()
	if err != nil {
		return nil, err
	}
	if int(n) > (len(r.b)-r.at)/4 {
		return nil, protoErr("declared more strings than the payload holds")
	}
	out := make([]string, n)
	for i := range out {
		if out[i], err = r.str(); err != nil {
			return nil, err
		}
	}
	return out, nil
}
func (r *reader) finish() error {
	if r.at != len(r.b) {
		return protoErr("trailing bytes after message")
	}
	return nil
}

// ParseError decodes an Error frame payload: u16 code, u32 len, UTF-8 message.
func ParseError(payload []byte) (*ServerError, error) {
	r := &reader{b: payload}
	code, err := r.u16()
	if err != nil {
		return nil, err
	}
	msg, err := r.str()
	if err != nil {
		return nil, err
	}
	return &ServerError{Code: code, Message: msg}, r.finish()
}
