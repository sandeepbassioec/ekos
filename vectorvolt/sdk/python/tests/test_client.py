"""Golden-byte + mock-server tests for the VectorVolt Python SDK.

Expected hex is built from the frame definition in
vectorvolt/crates/vectorvolt-net/src/protocol.rs (24-byte LE header:
"VV", version 1, opcode, flags, 3 reserved, u64 request_id, u32 payload_len,
u32 CRC-32/IEEE of payload; payload fields little-endian, strings/lists
u32-length-prefixed). Opcodes: Ping=1 Get=2 Insert=3 ... Pong=129 Row=130
Ok=132 Rows=133 Error=135 (protocol.rs OpCode).
"""

import hashlib
import socket
import struct
import threading
import unittest
import zlib

from vectorvolt import NotSupportedError, ProtocolError, ServerError, VectorVoltClient
from vectorvolt import wire

# protocol.rs: Request::Ping, request_id=1, empty payload (CRC-32 of "" = 0).
PING_REQ = "565601010000000001000000000000000000000000000000"
# Request::Insert{pk:7, values:[10,20]} id=2 -> payload u64 7, u32 2, u64 10, u64 20.
INSERT_REQ = ("565601030000000002000000000000001c000000cb77f5420700000000000000"
              "020000000a000000000000001400000000000000")
# Request::Get{pk:7} id=3.
GET_REQ = "565601020000000003000000000000000800000070d6e76f0700000000000000"
# Response::Row{values:[10,20]} id=3 (opcode 130 = 0x82).
ROW_RESP = ("5656018200000000030000000000000014000000fc6859c9020000000a000000"
            "000000001400000000000000")
# Response::Error{code:4, message:"not found"} id=4 (opcode 135 = 0x87).
ERR_RESP = "565601870000000004000000000000000f000000285977ce0400090000006e6f7420666f756e64"
# Request::UseTable{name:"users"} id=5.
USE_REQ = "5656010d0000000005000000000000000900000015033f7e050000007573657273"
# Request::CreateTable{name:"t", schema:["pk","a"]} id=6.
CREATE_REQ = ("5656010e00000000060000000000000014000000c900811801000000740200000002"
              "000000706b0100000061")
# Request::RangeScan{col:1, lo:0, hi:u64::MAX, limit:10} id=7.
SCAN_REQ = ("56560105000000000700000000000000180000006556ae530100000000000000"
            "00000000ffffffffffffffff0a000000")
# Response::Rows{rows:[[5],[]]} id=7 (opcode 133 = 0x85).
ROWS_RESP = ("5656018500000000070000000000000014000000800bd0f3020000000100000005"
             "0000000000000000000000")
# Request::CommitTxn{txn_id:9} id=8.
COMMIT_REQ = "565601140000000008000000000000000800000042c46d7a0900000000000000"
# Response::Pong id=1.
PONG_RESP = "565601810000000001000000000000000000000000000000"


class WireGolden(unittest.TestCase):
    def test_crc_is_ieee(self):
        self.assertEqual(zlib.crc32(b"123456789"), 0xCBF43926)

    def test_encode_matches_golden(self):
        self.assertEqual(wire.encode_frame(1, wire.PING).hex(), PING_REQ)
        self.assertEqual(
            wire.encode_frame(2, wire.INSERT, struct.pack("<QIQQ", 7, 2, 10, 20)).hex(),
            INSERT_REQ)
        self.assertEqual(wire.encode_frame(3, wire.GET, struct.pack("<Q", 7)).hex(), GET_REQ)
        self.assertEqual(wire.encode_frame(5, wire.USE_TABLE, wire.w_string("users")).hex(),
                         USE_REQ)
        self.assertEqual(
            wire.encode_frame(6, wire.CREATE_TABLE,
                              wire.w_string("t") + wire.w_strings(["pk", "a"])).hex(),
            CREATE_REQ)
        self.assertEqual(
            wire.encode_frame(7, wire.RANGE_SCAN,
                              struct.pack("<IQQI", 1, 0, 2**64 - 1, 10)).hex(), SCAN_REQ)
        self.assertEqual(wire.encode_frame(8, wire.COMMIT_TXN, struct.pack("<Q", 9)).hex(),
                         COMMIT_REQ)

    def test_decode_golden(self):
        f = wire.decode_frame(bytes.fromhex(ROW_RESP))
        self.assertEqual((f.opcode, f.request_id), (wire.ROW, 3))
        self.assertEqual(wire.Reader(f.payload).u64s(), [10, 20])
        self.assertEqual(wire.decode_frame(bytes.fromhex(PONG_RESP)).opcode, wire.PONG)
        f = wire.decode_frame(bytes.fromhex(ROWS_RESP))
        r = wire.Reader(f.payload)
        self.assertEqual(r.u32(), 2)
        self.assertEqual((r.u64s(), r.u64s()), ([5], []))

    def test_error_frame(self):
        f = wire.decode_frame(bytes.fromhex(ERR_RESP))
        self.assertEqual(f.opcode, wire.ERROR)
        e = wire.parse_error(f.payload)
        self.assertEqual((e.code, e.message), (4, "not found"))
        self.assertIn("NOT_FOUND", str(e))

    def test_bad_magic_version_crc_truncation(self):
        good = bytes.fromhex(PING_REQ)
        b = bytearray(good)
        b[0] = 0xFF
        with self.assertRaises(ProtocolError):
            wire.decode_frame(bytes(b))
        b = bytearray(good)
        b[2] = 42
        with self.assertRaises(ProtocolError):
            wire.decode_frame(bytes(b))
        g = bytearray.fromhex(GET_REQ)
        g[-1] ^= 0xFF
        with self.assertRaises(ProtocolError):
            wire.decode_frame(bytes(g))
        with self.assertRaises(ProtocolError):
            wire.decode_frame(bytes.fromhex(GET_REQ)[:-1])
        with self.assertRaises(ProtocolError):
            wire.decode_frame(good[:23])

    def test_payload_cap(self):
        b = bytearray.fromhex(PING_REQ)
        struct.pack_into("<I", b, 16, wire.MAX_PAYLOAD + 1)
        with self.assertRaises(ProtocolError):
            wire.decode_frame(bytes(b))

    def test_reader_rejects_lying_counts(self):
        with self.assertRaises(ProtocolError):
            wire.Reader(struct.pack("<I", 0xFFFFFFFF)).u64s()
        with self.assertRaises(ProtocolError):
            wire.Reader(struct.pack("<I", 1) + b"\x00").finish()


class MockServer:
    """Serves handler replies on one connection; records requests."""

    def __init__(self, handler):
        self.handler = handler
        self.requests = []
        self.srv = socket.socket()
        self.srv.bind(("127.0.0.1", 0))
        self.srv.listen(1)
        self.port = self.srv.getsockname()[1]
        threading.Thread(target=self._run, daemon=True).start()

    @staticmethod
    def _read(c, n):
        buf = b""
        while len(buf) < n:
            d = c.recv(n - len(buf))
            if not d:
                raise EOFError
            buf += d
        return buf

    def _run(self):
        try:
            c, _ = self.srv.accept()
        except OSError:
            return
        try:
            while True:
                hdr = self._read(c, wire.HEADER_SIZE)
                op, _, rid, plen, _ = wire.decode_header(hdr)
                body = self._read(c, plen) if plen else b""
                self.requests.append((op, rid, body))
                c.sendall(self.handler(op, rid, body))
        except (EOFError, OSError):
            pass
        finally:
            c.close()

    def close(self):
        self.srv.close()


def err(rid, code, msg):
    return wire.encode_frame(rid, wire.ERROR,
                             struct.pack("<HI", code, len(msg)) + msg.encode())


class ClientTests(unittest.TestCase):
    def serve(self, handler, **connect_kw):
        s = MockServer(handler)
        self.addCleanup(s.close)
        c = VectorVoltClient("127.0.0.1", s.port, timeout=5)
        c.connect(**connect_kw)
        self.addCleanup(c.disconnect)
        return s, c

    def test_ping_and_persistent_connection(self):
        s, c = self.serve(lambda op, rid, b: wire.encode_frame(rid, wire.PONG))
        self.assertTrue(c.ping())
        self.assertTrue(c.ping())
        self.assertEqual([r[1] for r in s.requests], [1, 2])

    def test_rows_and_txn(self):
        def h(op, rid, body):
            if op == wire.INSERT:
                return wire.encode_frame(rid, wire.OK, struct.pack("<Q", 1))
            if op == wire.GET:
                if struct.unpack("<Q", body)[0] == 7:
                    return wire.encode_frame(rid, wire.ROW, wire.w_u64s([10, 20]))
                return wire.encode_frame(rid, wire.NOT_FOUND)
            if op == wire.DELETE:
                return wire.encode_frame(rid, wire.OK, struct.pack("<Q", 0))
            if op == wire.RANGE_SCAN:
                return wire.encode_frame(
                    rid, wire.ROWS,
                    struct.pack("<II", 2, 1) + struct.pack("<Q", 5) + struct.pack("<I", 0))
            if op == wire.BEGIN_TXN:
                return wire.encode_frame(rid, wire.TXN_STARTED, struct.pack("<Q", 9))
            if op == wire.COMMIT_TXN:
                return wire.encode_frame(rid, wire.OK, struct.pack("<Q", 3))
            return err(rid, 15, "bad")
        s, c = self.serve(h)
        c.insert(7, [10, 20])
        self.assertEqual(s.requests[0][2], struct.pack("<QIQQ", 7, 2, 10, 20))
        self.assertEqual(c.get(7), [10, 20])
        self.assertIsNone(c.get(8))
        self.assertFalse(c.delete(8))
        self.assertEqual(c.range_scan(1, 0, 2**64 - 1, 10), [[5], []])
        tid = c.begin_txn()
        self.assertEqual(tid, 9)
        self.assertEqual(c.commit_txn(tid), 3)
        with self.assertRaises(ServerError) as cm:
            c.flush()
        self.assertEqual(cm.exception.code, 15)
        self.assertIsNotNone(c._sock)  # an Error frame keeps the connection

    def test_auth_handshake(self):
        nonce = bytes(range(32))

        def h(op, rid, body):
            if op == wire.HELLO:
                return wire.encode_frame(rid, wire.CHALLENGE, struct.pack("<I", 32) + nonce)
            r = wire.Reader(body)
            kid = r.string()
            resp = r.bytes_()
            r.finish()
            if kid == "app1" and resp == hashlib.sha3_256(b"secret" + nonce).digest():
                return wire.encode_frame(rid, wire.OK, struct.pack("<Q", 0))
            return err(rid, 7, "x")
        self.serve(h, key_id="app1", secret=b"secret")

    def test_bad_crc_response_drops_connection(self):
        def h(op, rid, body):
            f = bytearray(wire.encode_frame(rid, wire.ROW, wire.w_u64s([1])))
            f[-1] ^= 0xFF
            return bytes(f)
        _, c = self.serve(h)
        with self.assertRaises(ProtocolError):
            c.get(1)
        self.assertIsNone(c._sock)

    def test_sql_not_supported(self):
        c = VectorVoltClient()
        with self.assertRaises(NotSupportedError):
            c.query("SELECT 1")
        with self.assertRaises(NotSupportedError):
            c.execute("INSERT ...")


if __name__ == "__main__":
    unittest.main()
