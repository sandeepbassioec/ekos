/**
 * VectorVolt wire codec (pure, no I/O). Mirrors
 * vectorvolt/crates/vectorvolt-net/src/protocol.rs -- the framing the server
 * (server.rs) and the Rust SDK client (client.rs) actually speak.
 *
 * Frame = 24-byte little-endian header + payload:
 *   off size field
 *     0    2 magic        "VV"
 *     2    1 version      1
 *     3    1 opcode
 *     4    1 flags        0
 *     5    3 reserved     0
 *     8    8 request_id   echoed by the server
 *    16    4 payload_len  <= 16 MiB
 *    20    4 payload_crc  CRC-32 IEEE of the payload
 *
 * Erasable-syntax TypeScript only (no enums / parameter properties) so Node can
 * run it directly with type stripping.
 */

export const HEADER_SIZE = 24;
export const MAX_PAYLOAD = 16 * 1024 * 1024;
const VERSION = 1;

export const Op = {
  // requests (protocol.rs OpCode)
  Ping: 1,
  Get: 2,
  Insert: 3,
  Delete: 4,
  RangeScan: 5,
  Flush: 6,
  Compact: 7,
  Stats: 8,
  Hello: 9,
  Auth: 10,
  SecondaryLookup: 11,
  FkExists: 12,
  UseTable: 13,
  CreateTable: 14,
  DropTable: 15,
  ListTables: 16,
  BatchInsert: 17,
  BatchDelete: 18,
  BeginTxn: 19,
  CommitTxn: 20,
  RollbackTxn: 21,
  Upsert: 22,
  GetMany: 23,
  Update: 24,
  Truncate: 25,
  Count: 26,
  BatchUpsert: 27,
  BatchUpdate: 28,
  DescribeTable: 29,
  TableExists: 30,
  ScanKeys: 31,
  MinMax: 32,
  Sum: 33,
  GetAndDelete: 34,
  Increment: 35,
  CompareAndSwap: 36,
  SetField: 37,
  InsertIfAbsent: 38,
  DeleteRange: 39,
  PopMin: 40,
  PopMax: 41,
  MultiSet: 42,
  DecrementClamp: 43,
  GetOrInsert: 44,
  BatchSetField: 45,
  Top: 46,
  Bottom: 47,
  PkExists: 48,
  IncrementMulti: 49,
  CopyRow: 50,
  SwapField: 51,
  Clamp: 52,
  Average: 53,
  DeleteIf: 54,
  SetFieldIf: 55,
  RankOf: 56,
  Percentile: 57,
  SumFields: 58,
  MaxField: 59,
  FilterScan: 60,
  BatchIncrementField: 61,
  CountDistinct: 62,
  Median: 63,
  BatchDeleteIf: 64,
  BatchPkExists: 65,
  FieldBetween: 66,
  ScalarMin: 67,
  ScalarMax: 68,
  MultiGet: 69,
  IncrementOrInsert: 70,
  ZeroField: 71,
  ScanPage: 72,
  SumField: 73,
  BatchGet: 74,
  SetFieldRange: 75,
  MaxRow: 76,
  MinRow: 77,
  IncrementRange: 78,
  DeleteMin: 79,
  DeleteMax: 80,
  UniqueCount: 81,
  TopN: 82,
  BottomN: 83,
  MoveField: 84,
  CountIf: 85,
  SetIfGreater: 86,
  SetIfLess: 87,
  AbsDiff: 88,
  SwapFields: 89,
  CopyRange: 90,
  PropagateMax: 91,
  PropagateMin: 92,
  ClampRange: 93,
  BatchConditionalSet: 94,
  WindowRank: 95,
  WindowPct: 96,
  SumProduct: 97,
  NthValue: 98,
  DenseRankWin: 99,
  SetFieldMany: 100,
  MultiColumnSum: 101,
  FieldHistogram: 102,
  ZScoreFilter: 103,
  RunningSum: 104,
  FloorDiv: 105,
  MulField: 106,
  SetFieldAdd: 107,
  SetFieldMul: 108,
  SwapPkFields: 109,
  CountRange: 110,
  SumRange: 111,
  MinRange: 112,
  MaxRange: 113,
  AvgRange: 114,
  IncrementField: 115,
  DecrementField: 116,
  SetMin: 117,
  SetMax: 118,
  XorField: 119,
  AndField: 120,
  OrField: 121,
  BatchGetField: 122,
  // responses
  Pong: 129,
  Row: 130,
  NotFound: 131,
  Ok: 132,
  Rows: 133,
  StatsReply: 134,
  Error: 135,
  Challenge: 136,
  Exists: 137,
  TableSelected: 138,
  TableList: 139,
  BatchResult: 140,
  TxnStarted: 141,
  RowSet: 142,
  Schema: 143,
  Keys: 144,
  MinMaxResult: 145,
} as const;

// Server error codes (vectorvolt-net lib.rs error_code).
const ERR_NAMES: Record<number, string> = {
  1: "ENGINE", 2: "MALFORMED", 3: "DUPLICATE_KEY", 4: "NOT_FOUND", 5: "SCHEMA",
  6: "UNAUTHENTICATED", 7: "AUTH_FAILED", 8: "FORBIDDEN", 9: "NOT_SELECTED",
  10: "TABLE_NOT_FOUND", 11: "TABLE_EXISTS", 12: "TXN_ACTIVE", 13: "NO_TXN",
  14: "TXN_ID_MISMATCH", 15: "INVALID_ARGUMENT",
};

export class VectorVoltError extends Error {}
/** Malformed / unexpected wire data; the connection is unusable. */
export class ProtocolError extends VectorVoltError {}
/** The server wire protocol has no such operation. */
export class NotSupportedError extends VectorVoltError {}
/** An Error frame returned by the server. */
export class ServerError extends VectorVoltError {
  code: number;
  serverMessage: string;
  constructor(code: number, message: string) {
    super(`[${code} ${ERR_NAMES[code] ?? "UNKNOWN"}] ${message}`);
    this.code = code;
    this.serverMessage = message;
  }
}

// ---- CRC-32 (IEEE 802.3, reflected, poly 0xEDB88320) ----------------------

const CRC_TABLE = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

export function crc32(buf: Uint8Array): number {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = CRC_TABLE[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

// ---- frames ---------------------------------------------------------------

export interface Frame {
  opcode: number;
  flags: number;
  requestId: bigint;
  payload: Buffer;
}

export interface Header {
  opcode: number;
  flags: number;
  requestId: bigint;
  payloadLen: number;
  crc: number;
}

export function encodeFrame(requestId: bigint, opcode: number, payload: Uint8Array = new Uint8Array(0)): Buffer {
  if (payload.length > MAX_PAYLOAD) {
    throw new ProtocolError(`payload ${payload.length} exceeds max ${MAX_PAYLOAD}`);
  }
  const out = Buffer.alloc(HEADER_SIZE + payload.length);
  out[0] = 0x56; // 'V'
  out[1] = 0x56; // 'V'
  out[2] = VERSION;
  out[3] = opcode;
  out.writeBigUInt64LE(requestId, 8);
  out.writeUInt32LE(payload.length, 16);
  out.writeUInt32LE(crc32(payload), 20);
  out.set(payload, HEADER_SIZE);
  return out;
}

export function decodeHeader(buf: Uint8Array): Header {
  if (buf.length < HEADER_SIZE) {
    throw new ProtocolError(`truncated header: need ${HEADER_SIZE}, got ${buf.length}`);
  }
  const b = Buffer.from(buf.buffer, buf.byteOffset, buf.length);
  if (b[0] !== 0x56 || b[1] !== 0x56) throw new ProtocolError("bad magic");
  if (b[2] !== VERSION) throw new ProtocolError(`unsupported protocol version ${b[2]}`);
  const payloadLen = b.readUInt32LE(16);
  if (payloadLen > MAX_PAYLOAD) {
    throw new ProtocolError(`payload ${payloadLen} exceeds max ${MAX_PAYLOAD}`);
  }
  return {
    opcode: b[3], flags: b[4], requestId: b.readBigUInt64LE(8),
    payloadLen, crc: b.readUInt32LE(20),
  };
}

export function verifyPayload(payload: Uint8Array, h: Header): void {
  if (payload.length !== h.payloadLen) {
    throw new ProtocolError(`truncated payload: need ${h.payloadLen}, got ${payload.length}`);
  }
  const actual = crc32(payload);
  if (actual !== h.crc) {
    throw new ProtocolError(`CRC mismatch: header ${h.crc.toString(16)}, payload ${actual.toString(16)}`);
  }
}

export function decodeFrame(buf: Uint8Array): Frame {
  const h = decodeHeader(buf);
  const payload = Buffer.from(buf.subarray(HEADER_SIZE, HEADER_SIZE + h.payloadLen));
  verifyPayload(payload, h);
  return { opcode: h.opcode, flags: h.flags, requestId: h.requestId, payload };
}

// ---- payload writer -------------------------------------------------------

export class Writer {
  private parts: Buffer[] = [];
  u32(v: number): this {
    const b = Buffer.alloc(4);
    b.writeUInt32LE(v);
    this.parts.push(b);
    return this;
  }
  u64(v: bigint): this {
    const b = Buffer.alloc(8);
    b.writeBigUInt64LE(v);
    this.parts.push(b);
    return this;
  }
  u64s(vs: readonly bigint[]): this {
    this.u32(vs.length);
    for (const v of vs) this.u64(v);
    return this;
  }
  bytes(v: Uint8Array): this {
    this.u32(v.length);
    this.parts.push(Buffer.from(v));
    return this;
  }
  string(s: string): this {
    return this.bytes(Buffer.from(s, "utf8"));
  }
  strings(ss: readonly string[]): this {
    this.u32(ss.length);
    for (const s of ss) this.string(s);
    return this;
  }
  build(): Buffer {
    return Buffer.concat(this.parts);
  }
}

// ---- payload reader (bounds-checked) --------------------------------------

export class Reader {
  at = 0;
  private b: Buffer;
  constructor(b: Buffer) {
    this.b = b;
  }
  private take(n: number): Buffer {
    if (n < 0 || this.at + n > this.b.length) throw new ProtocolError("payload truncated");
    const s = this.b.subarray(this.at, this.at + n);
    this.at += n;
    return s;
  }
  u8(): number { return this.take(1)[0]; }
  u16(): number { return this.take(2).readUInt16LE(); }
  u32(): number { return this.take(4).readUInt32LE(); }
  u64(): bigint { return this.take(8).readBigUInt64LE(); }
  u64s(): bigint[] {
    const n = this.u32();
    if (n > Math.floor((this.b.length - this.at) / 8)) {
      throw new ProtocolError("declared more values than the payload holds");
    }
    const out: bigint[] = [];
    for (let i = 0; i < n; i++) out.push(this.u64());
    return out;
  }
  bytes(): Buffer { return Buffer.from(this.take(this.u32())); }
  string(): string { return this.bytes().toString("utf8"); }
  strings(): string[] {
    const n = this.u32();
    if (n > Math.floor((this.b.length - this.at) / 4)) {
      throw new ProtocolError("declared more strings than the payload holds");
    }
    const out: string[] = [];
    for (let i = 0; i < n; i++) out.push(this.string());
    return out;
  }
  remaining(): number { return this.b.length - this.at; }
  finish(): void {
    if (this.at !== this.b.length) throw new ProtocolError("trailing bytes after message");
  }
}

/** Error frame payload: u16 code, u32 len, UTF-8 message. */
export function parseError(payload: Buffer): ServerError {
  const r = new Reader(payload);
  const code = r.u16();
  const msg = r.string();
  r.finish();
  return new ServerError(code, msg);
}
