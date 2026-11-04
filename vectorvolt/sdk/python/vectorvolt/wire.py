"""VectorVolt wire codec (pure, no I/O).

Mirrors vectorvolt/crates/vectorvolt-net/src/protocol.rs -- the framing the
server (server.rs) and the Rust SDK client (client.rs) actually speak.

Frame = 24-byte little-endian header + payload:

    off size field
      0    2 magic        b"VV"
      2    1 version      1
      3    1 opcode
      4    1 flags        0
      5    3 reserved     0
      8    8 request_id   echoed by the server
     16    4 payload_len  <= 16 MiB
     20    4 payload_crc  CRC-32 IEEE (zlib.crc32) of the payload
"""

from __future__ import annotations

import struct
import zlib
from dataclasses import dataclass

MAGIC = b"VV"
VERSION = 1
HEADER_SIZE = 24
MAX_PAYLOAD = 16 * 1024 * 1024

_HEADER = struct.Struct("<2sBBB3xQII")

# Request opcodes (protocol.rs OpCode).
PING, GET, INSERT, DELETE, RANGE_SCAN, FLUSH, COMPACT, STATS, HELLO, AUTH = range(1, 11)
USE_TABLE, CREATE_TABLE, DROP_TABLE, LIST_TABLES = 13, 14, 15, 16
BATCH_INSERT, BATCH_DELETE = 17, 18
BEGIN_TXN, COMMIT_TXN, ROLLBACK_TXN = 19, 20, 21

# Response opcodes.
PONG, ROW, NOT_FOUND, OK, ROWS, STATS_REPLY, ERROR, CHALLENGE = range(129, 137)
EXISTS, TABLE_SELECTED, TABLE_LIST, BATCH_RESULT, TXN_STARTED = range(137, 142)

# Server error codes (vectorvolt-net lib.rs error_code).
ERR_NAMES = {
    1: "ENGINE", 2: "MALFORMED", 3: "DUPLICATE_KEY", 4: "NOT_FOUND", 5: "SCHEMA",
    6: "UNAUTHENTICATED", 7: "AUTH_FAILED", 8: "FORBIDDEN", 9: "NOT_SELECTED",
    10: "TABLE_NOT_FOUND", 11: "TABLE_EXISTS", 12: "TXN_ACTIVE", 13: "NO_TXN",
    14: "TXN_ID_MISMATCH", 15: "INVALID_ARGUMENT",
}


class VectorVoltError(Exception):
    """Base class for SDK errors."""


class ProtocolError(VectorVoltError):
    """Malformed / unexpected bytes on the wire (connection is unusable)."""


class NotSupportedError(VectorVoltError):
    """The server wire protocol has no such operation."""


class ServerError(VectorVoltError):
    """An Error frame returned by the server."""

    def __init__(self, code: int, message: str) -> None:
        self.code = code
        self.message = message
        super().__init__(f"[{code} {ERR_NAMES.get(code, 'UNKNOWN')}] {message}")


@dataclass
class Frame:
    opcode: int
    request_id: int
    payload: bytes
    flags: int = 0


def encode_frame(request_id: int, opcode: int, payload: bytes = b"") -> bytes:
    if len(payload) > MAX_PAYLOAD:
        raise ProtocolError(f"payload {len(payload)} exceeds max {MAX_PAYLOAD}")
    return _HEADER.pack(MAGIC, VERSION, opcode, 0, request_id, len(payload),
                        zlib.crc32(payload) & 0xFFFFFFFF) + payload


def decode_header(buf: bytes) -> tuple:
    """Validate a 24-byte header. Returns (opcode, flags, request_id, payload_len, crc)."""
    if len(buf) < HEADER_SIZE:
        raise ProtocolError(f"truncated header: need {HEADER_SIZE}, got {len(buf)}")
    magic, version, opcode, flags, rid, plen, crc = _HEADER.unpack_from(buf)
    if magic != MAGIC:
        raise ProtocolError("bad magic")
    if version != VERSION:
        raise ProtocolError(f"unsupported protocol version {version}")
    if plen > MAX_PAYLOAD:
        raise ProtocolError(f"payload {plen} exceeds max {MAX_PAYLOAD}")
    return opcode, flags, rid, plen, crc


def verify_payload(payload: bytes, plen: int, crc: int) -> None:
    if len(payload) != plen:
        raise ProtocolError(f"truncated payload: need {plen}, got {len(payload)}")
    actual = zlib.crc32(payload) & 0xFFFFFFFF
    if actual != crc:
        raise ProtocolError(f"CRC mismatch: header {crc:#010x}, payload {actual:#010x}")


def decode_frame(buf: bytes) -> Frame:
    """Decode one whole frame (header + payload), checking CRC."""
    opcode, flags, rid, plen, crc = decode_header(buf)
    payload = bytes(buf[HEADER_SIZE:HEADER_SIZE + plen])
    verify_payload(payload, plen, crc)
    return Frame(opcode, rid, payload, flags)


# ---- payload writers -------------------------------------------------------

def w_u64s(vs) -> bytes:
    vs = list(vs)
    return struct.pack("<I", len(vs)) + b"".join(struct.pack("<Q", v) for v in vs)


def w_string(s: str) -> bytes:
    b = s.encode("utf-8")
    return struct.pack("<I", len(b)) + b


def w_strings(ss) -> bytes:
    ss = list(ss)
    return struct.pack("<I", len(ss)) + b"".join(w_string(s) for s in ss)


# ---- payload reader --------------------------------------------------------

class Reader:
    def __init__(self, b: bytes) -> None:
        self.b = b
        self.at = 0

    def take(self, n: int) -> bytes:
        end = self.at + n
        if end > len(self.b):
            raise ProtocolError("payload truncated")
        s = self.b[self.at:end]
        self.at = end
        return s

    def u16(self) -> int:
        return struct.unpack("<H", self.take(2))[0]

    def u32(self) -> int:
        return struct.unpack("<I", self.take(4))[0]

    def u64(self) -> int:
        return struct.unpack("<Q", self.take(8))[0]

    def u64s(self) -> list:
        n = self.u32()
        if n > (len(self.b) - self.at) // 8:
            raise ProtocolError("declared more values than the payload holds")
        return [self.u64() for _ in range(n)]

    def string(self) -> str:
        n = self.u32()
        try:
            return self.take(n).decode("utf-8")
        except UnicodeDecodeError as e:
            raise ProtocolError("string is not UTF-8") from e

    def strings(self) -> list:
        n = self.u32()
        if n > (len(self.b) - self.at) // 4:
            raise ProtocolError("declared more strings than the payload holds")
        return [self.string() for _ in range(n)]

    def bytes_(self) -> bytes:
        return self.take(self.u32())

    def finish(self) -> None:
        if self.at != len(self.b):
            raise ProtocolError("trailing bytes after message")


def parse_error(payload: bytes) -> ServerError:
    """Error frame payload: u16 code, u32 len, UTF-8 message."""
    r = Reader(payload)
    code = r.u16()
    msg = r.string()
    r.finish()
    return ServerError(code, msg)
