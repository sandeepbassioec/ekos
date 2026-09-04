/**
 * VectorVolt TypeScript SDK — TCP client.
 *
 * Wire format (little-endian):
 *   Request:  [4-byte uint32 length][JSON payload]
 *   Response: [4-byte uint32 length][JSON payload]
 */

import net from "node:net";

export interface Row {
  pk: bigint;
  data: Record<string, unknown>;
}

export class VectorVoltClient {
  private socket: net.Socket | null = null;

  constructor(private host: string, private port: number) {}

  // -----------------------------------------------------------------------
  // Lifecycle
  // -----------------------------------------------------------------------

  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      const sock = new net.Socket();
      sock.connect(this.port, this.host, () => {
        this.socket = sock;
        resolve();
      });
      sock.once("error", reject);
    });
  }

  disconnect(): Promise<void> {
    return new Promise((resolve) => {
      if (this.socket == null) {
        resolve();
        return;
      }
      const s = this.socket;
      this.socket = null;
      s.destroy();
      resolve();
    });
  }

  // -----------------------------------------------------------------------
  // Public API
  // -----------------------------------------------------------------------

  async insert(pk: bigint, data: Record<string, unknown>): Promise<bigint> {
    const resp = await this.roundTrip({ op: "insert", pk: pk.toString(), data });
    return BigInt(resp["seq"] as number | string);
  }

  async get(pk: bigint): Promise<Row | null> {
    const resp = await this.roundTrip({ op: "get", pk: pk.toString() });
    if (!resp["found"]) return null;
    return {
      pk: BigInt(resp["pk"] as number | string),
      data: (resp["data"] as Record<string, unknown>) ?? {},
    };
  }

  async delete(pk: bigint): Promise<bigint> {
    const resp = await this.roundTrip({ op: "delete", pk: pk.toString() });
    return BigInt((resp["seq"] as number | string) ?? 0);
  }

  async ping(): Promise<boolean> {
    try {
      const resp = await this.roundTrip({ op: "ping" });
      return resp["pong"] === true;
    } catch {
      return false;
    }
  }

  // -----------------------------------------------------------------------
  // Private helpers
  // -----------------------------------------------------------------------

  private roundTrip(req: Record<string, unknown>): Promise<Record<string, unknown>> {
    return new Promise((resolve, reject) => {
      if (this.socket == null) {
        reject(new Error("VectorVoltClient: not connected"));
        return;
      }
      const payload = Buffer.from(JSON.stringify(req), "utf8");
      const lenBuf = Buffer.allocUnsafe(4);
      lenBuf.writeUInt32LE(payload.length, 0);
      const frame = Buffer.concat([lenBuf, payload]);

      const chunks: Buffer[] = [];
      let needed = 4;
      let haveLength = false;

      const onData = (chunk: Buffer) => {
        chunks.push(chunk);
        const total = chunks.reduce((s, c) => s + c.length, 0);
        if (!haveLength && total >= 4) {
          const joined = Buffer.concat(chunks);
          needed = joined.readUInt32LE(0) + 4;
          haveLength = true;
        }
        if (haveLength && total >= needed) {
          this.socket!.removeListener("data", onData);
          this.socket!.removeListener("error", onError);
          const full = Buffer.concat(chunks).subarray(4, needed);
          try {
            resolve(JSON.parse(full.toString("utf8")));
          } catch (e) {
            reject(e);
          }
        }
      };

      const onError = (err: Error) => {
        this.socket?.removeListener("data", onData);
        reject(err);
      };

      this.socket.on("data", onData);
      this.socket.once("error", onError);
      this.socket.write(frame);
    });
  }
}
