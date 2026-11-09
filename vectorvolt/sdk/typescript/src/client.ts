/**
 * VectorVolt TypeScript SDK -- TCP client speaking the real server protocol
 * (see wire.ts / vectorvolt-net/src/protocol.rs).
 *
 * The server is a key/value engine (u64 primary keys, rows of u64 columns,
 * named tables, per-connection transactions) with no SQL endpoint, so
 * query()/execute() reject with NotSupportedError.
 *
 * One persistent connection; calls are serialised (queued) on it. Server-side
 * transactions are bound to the connection, so beginTxn ... commitTxn span calls.
 */

import net from "node:net";
import tls from "node:tls";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import {
  HEADER_SIZE, Op, ProtocolError, NotSupportedError, ServerError, Reader, Writer,
  decodeHeader, encodeFrame, parseError, verifyPayload,
} from "./wire.js";
import type { Frame } from "./wire.js";
import { OpsBase } from "./ops.js";
import type { Row } from "./ops.js";

export type { Row };

/**
 * TLS settings (the server's net listener speaks TLS directly on its TCP port,
 * vectorvolt-net/src/tls.rs). Plain TCP is the default; pass this as the third
 * constructor argument to enable TLS.
 */
export interface TlsOptions {
  /** PEM file with the CA / pinned server certificate to trust (default: Node's built-in CAs). */
  caFile?: string;
  /** Name verified against the server certificate and sent as SNI (default: the host). */
  serverName?: string;
  /** Disable certificate verification (testing only). Off by default. */
  insecureSkipVerify?: boolean;
  /** PEM client certificate + key for mTLS; both or neither. */
  certFile?: string;
  keyFile?: string;
}

export class VectorVoltClient extends OpsBase {
  private socket: net.Socket | null = null;
  private nextId = 1n;
  private buf: Buffer = Buffer.alloc(0);
  private waiter: (() => void) | null = null;
  private fatal: Error | null = null;
  private queue: Promise<unknown> = Promise.resolve();
  private host: string;
  private port: number;
  private tlsOpts: TlsOptions | undefined;

  constructor(host: string, port: number, tlsOptions?: TlsOptions) {
    super();
    if (tlsOptions && !tlsOptions.certFile !== !tlsOptions.keyFile) {
      throw new Error("certFile and keyFile must be given together");
    }
    this.host = host;
    this.port = port;
    this.tlsOpts = tlsOptions;
  }

  private openSocket(onConnect: () => void, onError: (e: Error) => void): net.Socket {
    const o = this.tlsOpts;
    if (!o) {
      const sock = new net.Socket();
      sock.setNoDelay(true);
      sock.once("error", onError);
      sock.connect(this.port, this.host, onConnect);
      return sock;
    }
    const opts: tls.ConnectionOptions = {
      host: this.host,
      port: this.port,
      minVersion: "TLSv1.2",
      rejectUnauthorized: !o.insecureSkipVerify,
    };
    // Node warns when an IP address is used as SNI; an IP host is verified against IP SANs.
    const name = o.serverName ?? (net.isIP(this.host) ? undefined : this.host);
    if (name) opts.servername = name;
    if (o.caFile) opts.ca = readFileSync(o.caFile);
    if (o.certFile && o.keyFile) {
      opts.cert = readFileSync(o.certFile);
      opts.key = readFileSync(o.keyFile);
    }
    const sock = tls.connect(opts, onConnect);
    sock.setNoDelay(true);
    sock.once("error", onError);
    return sock;
  }

  // -- lifecycle -------------------------------------------------------------

  connect(auth?: { keyId: string; secret: Uint8Array }): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      const sock: net.Socket = this.openSocket(() => {
        sock.removeListener("error", reject);
        this.socket = sock;
        this.nextId = 1n;
        this.buf = Buffer.alloc(0);
        this.fatal = null;
        sock.on("data", (d) => {
          this.buf = Buffer.concat([this.buf, d]);
          this.wake();
        });
        const fail = (e: Error) => { this.fatal = this.fatal ?? e; this.wake(); };
        sock.on("error", fail);
        sock.on("close", () => fail(new Error("Connection closed by server")));
        resolve();
      }, reject);
    }).then(async () => {
      if (auth) {
        try {
          await this.authenticate(auth.keyId, auth.secret);
        } catch (e) {
          await this.disconnect();
          throw e;
        }
      }
    });
  }

  disconnect(): Promise<void> {
    const s = this.socket;
    this.socket = null;
    if (s) s.destroy();
    return Promise.resolve();
  }

  // -- transport -------------------------------------------------------------

  private wake(): void {
    const w = this.waiter;
    this.waiter = null;
    if (w) w();
  }

  private async readExact(n: number): Promise<Buffer> {
    while (this.buf.length < n) {
      if (this.fatal) throw this.fatal;
      await new Promise<void>((r) => { this.waiter = r; });
    }
    const out = this.buf.subarray(0, n);
    this.buf = this.buf.subarray(n);
    return out;
  }

  /** Queue one request/response exchange; resolves with the checked frame. */
  protected call(opcode: number, payload: Uint8Array, ...expect: number[]): Promise<Frame> {
    const run = async (): Promise<Frame> => {
      if (!this.socket) throw new Error("Not connected -- call connect() first");
      const id = this.nextId;
      this.nextId = (this.nextId + 1n) & 0xffffffffffffffffn || 1n;
      let frame: Frame;
      try {
        this.socket.write(encodeFrame(id, opcode, payload));
        const h = decodeHeader(await this.readExact(HEADER_SIZE));
        const body = h.payloadLen ? Buffer.from(await this.readExact(h.payloadLen)) : Buffer.alloc(0);
        verifyPayload(body, h);
        if (h.requestId !== id) {
          throw new ProtocolError("response request_id mismatch; stream desynchronised");
        }
        frame = { opcode: h.opcode, flags: h.flags, requestId: h.requestId, payload: body };
      } catch (e) {
        await this.disconnect(); // stream may be desynchronised
        throw e;
      }
      if (frame.opcode === Op.Error) throw parseError(frame.payload); // connection stays usable
      if (!expect.includes(frame.opcode)) {
        await this.disconnect();
        throw new ProtocolError(`unexpected response opcode ${frame.opcode}`);
      }
      return frame;
    };
    const p = this.queue.then(run, run);
    this.queue = p.catch(() => undefined);
    return p;
  }

  private static affected(f: Frame): bigint {
    const r = new Reader(f.payload);
    const v = r.u64();
    r.finish();
    return v;
  }

  // -- session ---------------------------------------------------------------

  /** Hello -> Challenge(nonce); Auth(key_id, SHA3-256(secret || nonce)). */
  async authenticate(keyId: string, secret: Uint8Array): Promise<void> {
    const f = await this.call(Op.Hello, new Uint8Array(0), Op.Challenge);
    const r = new Reader(f.payload);
    const nonce = r.bytes();
    r.finish();
    const digest = createHash("sha3-256").update(secret).update(nonce).digest();
    await this.call(Op.Auth, new Writer().string(keyId).bytes(digest).build(), Op.Ok);
  }

  /** True on Pong; transport failures resolve false. */
  async ping(): Promise<boolean> {
    try {
      await this.call(Op.Ping, new Uint8Array(0), Op.Pong);
      return true;
    } catch (e) {
      if (e instanceof ServerError) throw e; // server answered but refused
      return false;
    }
  }

  // -- tables ----------------------------------------------------------------

  async useTable(name: string): Promise<void> {
    await this.call(Op.UseTable, new Writer().string(name).build(), Op.TableSelected);
  }
  async createTable(name: string, schema: readonly string[]): Promise<void> {
    await this.call(Op.CreateTable, new Writer().string(name).strings(schema).build(), Op.TableSelected);
  }
  async dropTable(name: string): Promise<void> {
    await this.call(Op.DropTable, new Writer().string(name).build(), Op.Ok);
  }
  async listTables(): Promise<string[]> {
    const f = await this.call(Op.ListTables, new Uint8Array(0), Op.TableList);
    const r = new Reader(f.payload);
    const names = r.strings();
    r.finish();
    return names;
  }

  // -- rows ------------------------------------------------------------------

  async insert(pk: bigint, values: readonly bigint[]): Promise<void> {
    await this.call(Op.Insert, new Writer().u64(pk).u64s(values).build(), Op.Ok);
  }

  /** null when the key does not exist. */
  async get(pk: bigint): Promise<Row | null> {
    const f = await this.call(Op.Get, new Writer().u64(pk).build(), Op.Row, Op.NotFound);
    if (f.opcode === Op.NotFound) return null;
    const r = new Reader(f.payload);
    const v = r.u64s();
    r.finish();
    return v;
  }

  /** true if a row was deleted. */
  async delete(pk: bigint): Promise<boolean> {
    const f = await this.call(Op.Delete, new Writer().u64(pk).build(), Op.Ok);
    return VectorVoltClient.affected(f) > 0n;
  }

  /** limit 0 = unlimited. */
  async rangeScan(col: number, lo: bigint, hi: bigint, limit = 0): Promise<Row[]> {
    const f = await this.call(Op.RangeScan, new Writer().u32(col).u64(lo).u64(hi).u32(limit).build(), Op.Rows);
    const r = new Reader(f.payload);
    const n = r.u32();
    if (n > Math.floor(r.remaining() / 4)) throw new ProtocolError("declared more rows than the payload holds");
    const rows: Row[] = [];
    for (let i = 0; i < n; i++) rows.push(r.u64s());
    r.finish();
    return rows;
  }

  private static batch(f: Frame): { succeeded: number; skipped: number } {
    const r = new Reader(f.payload);
    const out = { succeeded: r.u32(), skipped: r.u32() };
    r.finish();
    return out;
  }

  async batchInsert(rows: readonly [bigint, readonly bigint[]][]): Promise<{ succeeded: number; skipped: number }> {
    const w = new Writer().u32(rows.length);
    for (const [pk, v] of rows) w.u64(pk).u64s(v);
    return VectorVoltClient.batch(await this.call(Op.BatchInsert, w.build(), Op.BatchResult));
  }

  async batchDelete(pks: readonly bigint[]): Promise<{ succeeded: number; skipped: number }> {
    const w = new Writer().u32(pks.length);
    for (const pk of pks) w.u64(pk);
    return VectorVoltClient.batch(await this.call(Op.BatchDelete, w.build(), Op.BatchResult));
  }

  // -- admin -----------------------------------------------------------------

  async flush(): Promise<void> {
    await this.call(Op.Flush, new Uint8Array(0), Op.Ok);
  }

  async stats(): Promise<{ rows: bigint; partitions: bigint; bytes: bigint }> {
    const f = await this.call(Op.Stats, new Uint8Array(0), Op.StatsReply);
    const r = new Reader(f.payload);
    const out = { rows: r.u64(), partitions: r.u64(), bytes: r.u64() };
    r.finish();
    return out;
  }

  // -- transactions (bound to this connection) -------------------------------

  async beginTxn(): Promise<bigint> {
    return VectorVoltClient.affected(await this.call(Op.BeginTxn, new Uint8Array(0), Op.TxnStarted));
  }
  /** Returns the number of operations applied. */
  async commitTxn(txnId: bigint): Promise<bigint> {
    return VectorVoltClient.affected(await this.call(Op.CommitTxn, new Writer().u64(txnId).build(), Op.Ok));
  }
  async rollbackTxn(txnId: bigint): Promise<void> {
    await this.call(Op.RollbackTxn, new Writer().u64(txnId).build(), Op.Ok);
  }

  // -- not supported by the server protocol ----------------------------------

  query(_sql: string): Promise<never> {
    return Promise.reject(new NotSupportedError("the VectorVolt wire protocol has no SQL endpoint; use get/rangeScan"));
  }
  execute(_sql: string): Promise<never> {
    return Promise.reject(new NotSupportedError("the VectorVolt wire protocol has no SQL endpoint; use insert/delete/batchInsert"));
  }
}
