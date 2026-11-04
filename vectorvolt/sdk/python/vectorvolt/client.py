"""VectorVolt Python SDK -- TCP client speaking the real server protocol.

See ``wire.py`` for the framing. The server is a typed key/value engine
(u64 primary keys, rows of u64 columns, named tables, per-connection
transactions); it has NO SQL endpoint, so ``query``/``execute`` raise
``NotSupportedError``.
"""

from __future__ import annotations

import hashlib
import socket
from typing import Any, Optional, Sequence

from . import wire
from .wire import NotSupportedError, ProtocolError, Reader


class VectorVoltClient:
    """Blocking client holding ONE persistent connection.

    Server-side transactions are bound to the connection, so begin_txn ...
    commit_txn/rollback_txn span calls on the same client.
    """

    def __init__(self, host: str = "127.0.0.1", port: int = 5050,
                 timeout: Optional[float] = None) -> None:
        self._host = host
        self._port = port
        self._timeout = timeout
        self._sock: Optional[socket.socket] = None
        self._next_id = 1

    # -- lifecycle -----------------------------------------------------------

    def connect(self, key_id: Optional[str] = None, secret: Optional[bytes] = None) -> None:
        """Open the connection; authenticate when credentials are given."""
        s = socket.create_connection((self._host, self._port), self._timeout)
        s.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self._sock = s
        self._next_id = 1
        if key_id is not None:
            try:
                self.authenticate(key_id, secret or b"")
            except BaseException:
                self.disconnect()
                raise

    def disconnect(self) -> None:
        if self._sock is not None:
            try:
                self._sock.close()
            finally:
                self._sock = None

    def __enter__(self) -> "VectorVoltClient":
        self.connect()
        return self

    def __exit__(self, *_: Any) -> None:
        self.disconnect()

    # -- transport -----------------------------------------------------------

    def _recv_exact(self, n: int) -> bytes:
        buf = bytearray()
        while len(buf) < n:
            chunk = self._sock.recv(n - len(buf))
            if not chunk:
                raise EOFError("Connection closed by server")
            buf.extend(chunk)
        return bytes(buf)

    def _call(self, opcode: int, payload: bytes = b"") -> wire.Frame:
        """Send one request, return the (CRC-checked) response frame.

        Error frames raise ServerError (connection stays usable). Any
        transport/framing failure drops the connection, which is desynchronised.
        """
        if self._sock is None:
            raise ConnectionError("Not connected -- call connect() first")
        rid = self._next_id
        self._next_id = ((self._next_id + 1) & 0xFFFFFFFFFFFFFFFF) or 1
        try:
            self._sock.sendall(wire.encode_frame(rid, opcode, payload))
            op, flags, got_id, plen, crc = wire.decode_header(self._recv_exact(wire.HEADER_SIZE))
            body = self._recv_exact(plen) if plen else b""
            wire.verify_payload(body, plen, crc)
            if got_id != rid:
                raise ProtocolError("response request_id mismatch; stream desynchronised")
        except (ProtocolError, OSError, EOFError):
            self.disconnect()
            raise
        if op == wire.ERROR:
            raise wire.parse_error(body)
        return wire.Frame(op, got_id, body, flags)

    def _expect(self, opcode: int, payload: bytes, *expected: int) -> wire.Frame:
        f = self._call(opcode, payload)
        if f.opcode not in expected:
            self.disconnect()
            raise ProtocolError(f"unexpected response opcode {f.opcode}")
        return f

    @staticmethod
    def _affected(f: wire.Frame) -> int:
        r = Reader(f.payload)
        v = r.u64()
        r.finish()
        return v

    # -- session -------------------------------------------------------------

    def authenticate(self, key_id: str, secret: bytes) -> None:
        """Hello -> Challenge(nonce); Auth(key_id, SHA3-256(secret || nonce))."""
        f = self._expect(wire.HELLO, b"", wire.CHALLENGE)
        r = Reader(f.payload)
        nonce = r.bytes_()
        r.finish()
        digest = hashlib.sha3_256(secret + nonce).digest()
        self._expect(wire.AUTH, wire.w_string(key_id) + len(digest).to_bytes(4, "little") + digest,
                     wire.OK)

    def ping(self) -> bool:
        """Ping; True on Pong. Transport failures return False."""
        try:
            self._expect(wire.PING, b"", wire.PONG)
            return True
        except (ConnectionError, EOFError, OSError, ProtocolError):
            return False

    # -- tables --------------------------------------------------------------

    def use_table(self, name: str) -> None:
        self._expect(wire.USE_TABLE, wire.w_string(name), wire.TABLE_SELECTED)

    def create_table(self, name: str, schema: Sequence[str]) -> None:
        self._expect(wire.CREATE_TABLE, wire.w_string(name) + wire.w_strings(schema),
                     wire.TABLE_SELECTED)

    def drop_table(self, name: str) -> None:
        self._expect(wire.DROP_TABLE, wire.w_string(name), wire.OK)

    def list_tables(self) -> list:
        f = self._expect(wire.LIST_TABLES, b"", wire.TABLE_LIST)
        r = Reader(f.payload)
        names = r.strings()
        r.finish()
        return names

    # -- rows ----------------------------------------------------------------

    def insert(self, pk: int, values: Sequence[int]) -> None:
        self._expect(wire.INSERT, pk.to_bytes(8, "little") + wire.w_u64s(values), wire.OK)

    def get(self, pk: int) -> Optional[list]:
        f = self._expect(wire.GET, pk.to_bytes(8, "little"), wire.ROW, wire.NOT_FOUND)
        if f.opcode == wire.NOT_FOUND:
            return None
        r = Reader(f.payload)
        v = r.u64s()
        r.finish()
        return v

    def delete(self, pk: int) -> bool:
        """True if a row was deleted."""
        f = self._expect(wire.DELETE, pk.to_bytes(8, "little"), wire.OK)
        return self._affected(f) > 0

    def range_scan(self, col: int, lo: int, hi: int, limit: int = 0) -> list:
        """limit 0 = unlimited."""
        p = (col.to_bytes(4, "little") + lo.to_bytes(8, "little")
             + hi.to_bytes(8, "little") + limit.to_bytes(4, "little"))
        f = self._expect(wire.RANGE_SCAN, p, wire.ROWS)
        r = Reader(f.payload)
        n = r.u32()
        if n > (len(f.payload) - r.at) // 4:
            raise ProtocolError("declared more rows than the payload holds")
        rows = [r.u64s() for _ in range(n)]
        r.finish()
        return rows

    def batch_insert(self, rows: Sequence) -> tuple:
        """rows: [(pk, [values])]. Returns (succeeded, skipped)."""
        p = len(rows).to_bytes(4, "little") + b"".join(
            pk.to_bytes(8, "little") + wire.w_u64s(v) for pk, v in rows)
        return self._batch_result(self._expect(wire.BATCH_INSERT, p, wire.BATCH_RESULT))

    def batch_delete(self, pks: Sequence[int]) -> tuple:
        """Returns (succeeded, skipped)."""
        p = len(pks).to_bytes(4, "little") + b"".join(k.to_bytes(8, "little") for k in pks)
        return self._batch_result(self._expect(wire.BATCH_DELETE, p, wire.BATCH_RESULT))

    @staticmethod
    def _batch_result(f: wire.Frame) -> tuple:
        r = Reader(f.payload)
        out = (r.u32(), r.u32())
        r.finish()
        return out

    # -- admin ---------------------------------------------------------------

    def flush(self) -> None:
        self._expect(wire.FLUSH, b"", wire.OK)

    def compact(self) -> int:
        return self._affected(self._expect(wire.COMPACT, b"", wire.OK))

    def stats(self) -> tuple:
        """(rows, partitions, bytes)."""
        f = self._expect(wire.STATS, b"", wire.STATS_REPLY)
        r = Reader(f.payload)
        out = (r.u64(), r.u64(), r.u64())
        r.finish()
        return out

    # -- transactions (bound to this connection) -----------------------------

    def begin_txn(self) -> int:
        f = self._expect(wire.BEGIN_TXN, b"", wire.TXN_STARTED)
        r = Reader(f.payload)
        tid = r.u64()
        r.finish()
        return tid

    def commit_txn(self, txn_id: int) -> int:
        """Returns the number of operations applied."""
        return self._affected(
            self._expect(wire.COMMIT_TXN, txn_id.to_bytes(8, "little"), wire.OK))

    def rollback_txn(self, txn_id: int) -> None:
        self._expect(wire.ROLLBACK_TXN, txn_id.to_bytes(8, "little"), wire.OK)

    abort_txn = rollback_txn

    # -- not supported by the server protocol --------------------------------

    def query(self, sql: str) -> Any:
        raise NotSupportedError(
            "the VectorVolt wire protocol has no SQL endpoint; use get/range_scan")

    def execute(self, sql: str) -> Any:
        raise NotSupportedError(
            "the VectorVolt wire protocol has no SQL endpoint; use insert/delete/batch_*")
