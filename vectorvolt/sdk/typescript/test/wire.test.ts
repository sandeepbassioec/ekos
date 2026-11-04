/**
 * Golden-byte + mock-server tests. Expected hex is built from the frame
 * definition in vectorvolt/crates/vectorvolt-net/src/protocol.rs: 24-byte LE
 * header ("VV", version 1, opcode, flags, 3 reserved, u64 request_id,
 * u32 payload_len, u32 CRC-32/IEEE of payload); payload fields little-endian,
 * strings/lists u32-length-prefixed. Opcodes: Ping=1 Get=2 Insert=3 ...
 * Pong=129 Row=130 Ok=132 Rows=133 Error=135 (protocol.rs OpCode).
 *
 * Run: node --import ./test/register.mjs --test test/
 */
import test from "node:test";
import assert from "node:assert/strict";
import net from "node:net";
import { createHash } from "node:crypto";
import {
  Op, Reader, Writer, HEADER_SIZE, MAX_PAYLOAD, crc32, decodeFrame, decodeHeader, encodeFrame,
  parseError, ProtocolError, ServerError, NotSupportedError,
} from "../src/wire.js";
import { VectorVoltClient } from "../src/client.js";

// Request::Ping id=1, empty payload (CRC-32 of "" = 0).
const PING_REQ = "565601010000000001000000000000000000000000000000";
// Request::Insert{pk:7, values:[10,20]} id=2.
const INSERT_REQ = "565601030000000002000000000000001c000000cb77f5420700000000000000" +
  "020000000a000000000000001400000000000000";
// Request::Get{pk:7} id=3.
const GET_REQ = "565601020000000003000000000000000800000070d6e76f0700000000000000";
// Response::Row{values:[10,20]} id=3 (opcode 130 = 0x82).
const ROW_RESP = "5656018200000000030000000000000014000000fc6859c9020000000a000000" +
  "000000001400000000000000";
// Response::Error{code:4, message:"not found"} id=4 (opcode 135 = 0x87).
const ERR_RESP = "565601870000000004000000000000000f000000285977ce0400090000006e6f7420666f756e64";
// Request::UseTable{name:"users"} id=5.
const USE_REQ = "5656010d0000000005000000000000000900000015033f7e050000007573657273";
// Request::CreateTable{name:"t", schema:["pk","a"]} id=6.
const CREATE_REQ = "5656010e00000000060000000000000014000000c900811801000000740200000002" +
  "000000706b0100000061";
// Request::RangeScan{col:1, lo:0, hi:u64::MAX, limit:10} id=7.
const SCAN_REQ = "56560105000000000700000000000000180000006556ae530100000000000000" +
  "00000000ffffffffffffffff0a000000";
// Response::Rows{rows:[[5],[]]} id=7 (opcode 133 = 0x85).
const ROWS_RESP = "5656018500000000070000000000000014000000800bd0f3020000000100000005" +
  "0000000000000000000000";
// Request::CommitTxn{txn_id:9} id=8.
const COMMIT_REQ = "565601140000000008000000000000000800000042c46d7a0900000000000000";
// Response::Pong id=1.
const PONG_RESP = "565601810000000001000000000000000000000000000000";

const hex = (b: Uint8Array) => Buffer.from(b).toString("hex");
const unhex = (s: string) => Buffer.from(s, "hex");
const MAX64 = 0xffffffffffffffffn;

test("CRC is IEEE CRC-32", () => {
  assert.equal(crc32(Buffer.from("123456789")), 0xcbf43926);
});

test("encode matches golden bytes", () => {
  assert.equal(hex(encodeFrame(1n, Op.Ping)), PING_REQ);
  assert.equal(hex(encodeFrame(2n, Op.Insert, new Writer().u64(7n).u64s([10n, 20n]).build())), INSERT_REQ);
  assert.equal(hex(encodeFrame(3n, Op.Get, new Writer().u64(7n).build())), GET_REQ);
  assert.equal(hex(encodeFrame(5n, Op.UseTable, new Writer().string("users").build())), USE_REQ);
  assert.equal(hex(encodeFrame(6n, Op.CreateTable, new Writer().string("t").strings(["pk", "a"]).build())), CREATE_REQ);
  assert.equal(hex(encodeFrame(7n, Op.RangeScan, new Writer().u32(1).u64(0n).u64(MAX64).u32(10).build())), SCAN_REQ);
  assert.equal(hex(encodeFrame(8n, Op.CommitTxn, new Writer().u64(9n).build())), COMMIT_REQ);
});

test("decode golden frames", () => {
  const f = decodeFrame(unhex(ROW_RESP));
  assert.equal(f.opcode, Op.Row);
  assert.equal(f.requestId, 3n);
  assert.deepEqual(new Reader(f.payload).u64s(), [10n, 20n]);
  assert.equal(decodeFrame(unhex(PONG_RESP)).opcode, Op.Pong);
  const r = new Reader(decodeFrame(unhex(ROWS_RESP)).payload);
  assert.equal(r.u32(), 2);
  assert.deepEqual([r.u64s(), r.u64s()], [[5n], []]);
});

test("error frame parses", () => {
  const f = decodeFrame(unhex(ERR_RESP));
  assert.equal(f.opcode, Op.Error);
  const e = parseError(f.payload);
  assert.ok(e instanceof ServerError);
  assert.equal(e.code, 4);
  assert.equal(e.serverMessage, "not found");
  assert.match(e.message, /NOT_FOUND/);
});

test("rejects bad magic / version / crc / truncation / oversize", () => {
  const good = unhex(PING_REQ);
  const mut = (i: number, v: number) => { const b = Buffer.from(good); b[i] = v; return b; };
  assert.throws(() => decodeFrame(mut(0, 0xff)), ProtocolError);
  assert.throws(() => decodeFrame(mut(2, 42)), ProtocolError);
  assert.throws(() => decodeFrame(good.subarray(0, 23)), ProtocolError);
  const g = unhex(GET_REQ);
  g[g.length - 1] ^= 0xff;
  assert.throws(() => decodeFrame(g), ProtocolError);
  assert.throws(() => decodeFrame(unhex(GET_REQ).subarray(0, -1)), ProtocolError);
  const big = Buffer.from(good);
  big.writeUInt32LE(MAX_PAYLOAD + 1, 16);
  assert.throws(() => decodeHeader(big), ProtocolError);
});

test("reader rejects lying counts", () => {
  const b = Buffer.alloc(4);
  b.writeUInt32LE(0xffffffff);
  assert.throws(() => new Reader(b).u64s(), ProtocolError);
});

// ---- mock-server client tests ----------------------------------------------

type Handler = (op: number, id: bigint, body: Buffer) => Buffer;

async function mock(handler: Handler): Promise<{ port: number; reqs: Buffer[]; close: () => void }> {
  const reqs: Buffer[] = [];
  const server = net.createServer((sock) => {
    let buf = Buffer.alloc(0);
    sock.on("data", (d) => {
      buf = Buffer.concat([buf, d]);
      for (;;) {
        if (buf.length < HEADER_SIZE) return;
        const h = decodeHeader(buf);
        if (buf.length < HEADER_SIZE + h.payloadLen) return;
        const body = buf.subarray(HEADER_SIZE, HEADER_SIZE + h.payloadLen);
        reqs.push(Buffer.from(buf.subarray(0, HEADER_SIZE + h.payloadLen)));
        buf = buf.subarray(HEADER_SIZE + h.payloadLen);
        sock.write(handler(h.opcode, h.requestId, Buffer.from(body)));
      }
    });
    sock.on("error", () => {});
  });
  await new Promise<void>((r) => server.listen(0, "127.0.0.1", r));
  const port = (server.address() as net.AddressInfo).port;
  return { port, reqs, close: () => server.close() };
}

const errFrame = (id: bigint, code: number, msg: string) =>
  encodeFrame(id, Op.Error, Buffer.concat([Buffer.from([code & 0xff, code >> 8]),
    new Writer().string(msg).build()]));

test("client: ping, insert/get/delete/scan/txn over one persistent connection", async () => {
  const m = await mock((op, id, body) => {
    switch (op) {
      case Op.Ping: return encodeFrame(id, Op.Pong);
      case Op.Insert: return encodeFrame(id, Op.Ok, new Writer().u64(1n).build());
      case Op.Get:
        return body.readBigUInt64LE() === 7n
          ? encodeFrame(id, Op.Row, new Writer().u64s([10n, 20n]).build())
          : encodeFrame(id, Op.NotFound);
      case Op.Delete: return encodeFrame(id, Op.Ok, new Writer().u64(0n).build());
      case Op.RangeScan:
        return encodeFrame(id, Op.Rows, new Writer().u32(2).u64s([5n]).u64s([]).build());
      case Op.BeginTxn: return encodeFrame(id, Op.TxnStarted, new Writer().u64(9n).build());
      case Op.CommitTxn: return encodeFrame(id, Op.Ok, new Writer().u64(3n).build());
      default: return errFrame(id, 15, "bad");
    }
  });
  const c = new VectorVoltClient("127.0.0.1", m.port);
  await c.connect();
  try {
    assert.equal(await c.ping(), true);
    await c.insert(7n, [10n, 20n]);
    assert.equal(hex(m.reqs[1]), INSERT_REQ); // second request on the same connection => id 2
    assert.deepEqual(await c.get(7n), [10n, 20n]);
    assert.equal(await c.get(8n), null);
    assert.equal(await c.delete(8n), false);
    assert.deepEqual(await c.rangeScan(1, 0n, MAX64, 10), [[5n], []]);
    const tid = await c.beginTxn();
    assert.equal(tid, 9n);
    assert.equal(await c.commitTxn(tid), 3n);
    await assert.rejects(c.flush(), (e: unknown) => e instanceof ServerError && e.code === 15);
    assert.equal(await c.ping(), true); // Error frame keeps the connection
  } finally {
    await c.disconnect();
    m.close();
  }
});

test("client: auth handshake uses SHA3-256(secret || nonce)", async () => {
  const nonce = Buffer.from(Array.from({ length: 32 }, (_, i) => i));
  const want = createHash("sha3-256").update("secret").update(nonce).digest();
  assert.equal(want.toString("hex"), "8d8308f62b86ad04269ee6bb23d09100344842bef9f1d6839ca2278612b67d34");
  const m = await mock((op, id, body) => {
    if (op === Op.Hello) return encodeFrame(id, Op.Challenge, new Writer().bytes(nonce).build());
    const r = new Reader(body);
    const ok = r.string() === "app1" && r.bytes().equals(want);
    return ok ? encodeFrame(id, Op.Ok, new Writer().u64(0n).build()) : errFrame(id, 7, "x");
  });
  const c = new VectorVoltClient("127.0.0.1", m.port);
  await c.connect({ keyId: "app1", secret: Buffer.from("secret") });
  await c.disconnect();
  m.close();
});

test("client: bad response CRC rejects and drops the connection", async () => {
  const m = await mock((_op, id) => {
    const f = encodeFrame(id, Op.Row, new Writer().u64s([1n]).build());
    f[f.length - 1] ^= 0xff;
    return f;
  });
  const c = new VectorVoltClient("127.0.0.1", m.port);
  await c.connect();
  await assert.rejects(c.get(1n), ProtocolError);
  await assert.rejects(c.get(1n), /Not connected/);
  m.close();
});

test("client: SQL is not supported", async () => {
  const c = new VectorVoltClient("127.0.0.1", 1);
  await assert.rejects(c.query("SELECT 1"), NotSupportedError);
  await assert.rejects(c.execute("INSERT"), NotSupportedError);
});
