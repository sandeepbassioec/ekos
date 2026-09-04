/**
 * Tests for VectorVoltClient using a real in-process TCP mock server.
 */

import net from "node:net";
import { describe, it, expect, beforeAll, afterAll, afterEach } from "@jest/globals";
import { VectorVoltClient, Row } from "./client.js";

// ---------------------------------------------------------------------------
// Mock server helpers
// ---------------------------------------------------------------------------

interface MockServer {
  server: net.Server;
  port: number;
  close: () => Promise<void>;
}

function encodeResponse(payload: Record<string, unknown>): Buffer {
  const json = Buffer.from(JSON.stringify(payload), "utf8");
  const len = Buffer.allocUnsafe(4);
  len.writeUInt32LE(json.length, 0);
  return Buffer.concat([len, json]);
}

function startMockServer(responses: Array<Record<string, unknown>>): Promise<MockServer> {
  return new Promise((resolve) => {
    let idx = 0;
    const server = net.createServer((sock) => {
      let buf = Buffer.alloc(0);
      sock.on("data", (chunk: Buffer) => {
        buf = Buffer.concat([buf, chunk]);
        // Drain complete frames.
        while (buf.length >= 4) {
          const len = buf.readUInt32LE(0);
          if (buf.length < 4 + len) break;
          buf = buf.subarray(4 + len); // consume request
          if (idx < responses.length) {
            sock.write(encodeResponse(responses[idx++]));
          }
        }
      });
    });
    server.listen(0, "127.0.0.1", () => {
      const addr = server.address() as net.AddressInfo;
      resolve({
        server,
        port: addr.port,
        close: () => new Promise((res) => server.close(() => res())),
      });
    });
  });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("VectorVoltClient", () => {
  it("connect and disconnect", async () => {
    const mock = await startMockServer([]);
    const client = new VectorVoltClient("127.0.0.1", mock.port);
    await client.connect();
    await client.disconnect();
    await mock.close();
  });

  it("insert returns bigint seq", async () => {
    const mock = await startMockServer([{ seq: 7 }]);
    const client = new VectorVoltClient("127.0.0.1", mock.port);
    await client.connect();
    const seq = await client.insert(1n, { name: "Alice" });
    expect(seq).toBe(7n);
    await client.disconnect();
    await mock.close();
  });

  it("get returns Row", async () => {
    const mock = await startMockServer([{ found: true, pk: 5, data: { x: 99 } }]);
    const client = new VectorVoltClient("127.0.0.1", mock.port);
    await client.connect();
    const row = await client.get(5n);
    expect(row).not.toBeNull();
    expect((row as Row).pk).toBe(5n);
    await client.disconnect();
    await mock.close();
  });

  it("get returns null on miss", async () => {
    const mock = await startMockServer([{ found: false }]);
    const client = new VectorVoltClient("127.0.0.1", mock.port);
    await client.connect();
    const row = await client.get(999n);
    expect(row).toBeNull();
    await client.disconnect();
    await mock.close();
  });

  it("delete returns seq", async () => {
    const mock = await startMockServer([{ seq: 3 }]);
    const client = new VectorVoltClient("127.0.0.1", mock.port);
    await client.connect();
    const seq = await client.delete(1n);
    expect(seq).toBe(3n);
    await client.disconnect();
    await mock.close();
  });

  it("ping returns true", async () => {
    const mock = await startMockServer([{ pong: true }]);
    const client = new VectorVoltClient("127.0.0.1", mock.port);
    await client.connect();
    const ok = await client.ping();
    expect(ok).toBe(true);
    await client.disconnect();
    await mock.close();
  });

  it("multiple inserts", async () => {
    const mock = await startMockServer([{ seq: 1 }, { seq: 2 }]);
    const client = new VectorVoltClient("127.0.0.1", mock.port);
    await client.connect();
    const s1 = await client.insert(1n, {});
    const s2 = await client.insert(2n, {});
    expect(s1).toBe(1n);
    expect(s2).toBe(2n);
    await client.disconnect();
    await mock.close();
  });

  it("context cleanup — disconnect is idempotent", async () => {
    const mock = await startMockServer([]);
    const client = new VectorVoltClient("127.0.0.1", mock.port);
    await client.connect();
    await client.disconnect();
    await expect(client.disconnect()).resolves.toBeUndefined();
    await mock.close();
  });

  it("bigint pk roundtrip", async () => {
    const bigPk = 1234567890n;
    const mock = await startMockServer([{ found: true, pk: 1234567890, data: {} }]);
    const client = new VectorVoltClient("127.0.0.1", mock.port);
    await client.connect();
    const row = await client.get(bigPk);
    expect((row as Row).pk).toBe(bigPk);
    await client.disconnect();
    await mock.close();
  });

  it("error on bad host", async () => {
    const client = new VectorVoltClient("0.0.0.0", 1); // no server there
    await expect(client.connect()).rejects.toBeDefined();
  });

  it("disconnect idempotent when never connected", async () => {
    const client = new VectorVoltClient("127.0.0.1", 9999);
    await expect(client.disconnect()).resolves.toBeUndefined();
  });

  it("reconnect after disconnect", async () => {
    const mock = await startMockServer([{ seq: 1 }]);
    const client = new VectorVoltClient("127.0.0.1", mock.port);
    await client.connect();
    await client.disconnect();
    await client.connect();
    const seq = await client.insert(1n, {});
    expect(seq).toBe(1n);
    await client.disconnect();
    await mock.close();
  });

  it("ping on not-connected client returns false", async () => {
    const client = new VectorVoltClient("127.0.0.1", 9999);
    const ok = await client.ping();
    expect(ok).toBe(false);
  });
});
