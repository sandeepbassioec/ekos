/**
 * TLS client tests against a local mock TLS server (self-signed certs generated in-test
 * with the openssl CLI; skipped when it is not installed). No network access.
 */
import test from "node:test";
import assert from "node:assert/strict";
import tls from "node:tls";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Op, HEADER_SIZE, decodeHeader, encodeFrame } from "../src/wire.js";
import { VectorVoltClient } from "../src/client.js";
import type { TlsOptions } from "../src/client.js";

let haveOpenssl = true;
try {
  execFileSync("openssl", ["version"], { stdio: "ignore" });
} catch {
  haveOpenssl = false;
}

const dir = haveOpenssl ? mkdtempSync(join(tmpdir(), "vv-tls-")) : "";
function makeCert(name: string, cn: string, san: string): { cert: string; key: string } {
  const cert = join(dir, `${name}.crt`);
  const key = join(dir, `${name}.key`);
  execFileSync("openssl", ["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", key,
    "-out", cert, "-days", "2", "-subj", `/CN=${cn}`, "-addext", `subjectAltName=${san}`],
  { stdio: "ignore" });
  return { cert, key };
}

async function pongServer(opts: tls.TlsOptions): Promise<{ port: number; close: () => void }> {
  const server = tls.createServer(opts, (sock) => {
    let buf = Buffer.alloc(0);
    sock.on("data", (d) => {
      buf = Buffer.concat([buf, d]);
      for (;;) {
        if (buf.length < HEADER_SIZE) return;
        const h = decodeHeader(buf);
        if (buf.length < HEADER_SIZE + h.payloadLen) return;
        buf = buf.subarray(HEADER_SIZE + h.payloadLen);
        sock.write(encodeFrame(h.requestId, Op.Pong));
      }
    });
    sock.on("error", () => {});
  });
  server.on("tlsClientError", () => {});
  await new Promise<void>((r) => server.listen(0, "127.0.0.1", r));
  const port = (server.address() as import("node:net").AddressInfo).port;
  return { port, close: () => server.close() };
}

test("tls", { skip: !haveOpenssl && "openssl CLI not available" }, async (t) => {
  const srv = makeCert("server", "localhost", "DNS:localhost,IP:127.0.0.1");
  const other = makeCert("other", "other", "DNS:other");
  const cli = makeCert("client", "client", "DNS:client");
  t.after(() => rmSync(dir, { recursive: true, force: true }));

  const serverOpts: tls.TlsOptions = { cert: readFileSync(srv.cert), key: readFileSync(srv.key) };
  const mtlsOpts: tls.TlsOptions = {
    ...serverOpts, requestCert: true, rejectUnauthorized: true, ca: readFileSync(cli.cert),
  };

  async function ping(port: number, o: TlsOptions): Promise<boolean> {
    const c = new VectorVoltClient("127.0.0.1", port, o);
    await c.connect();
    try {
      return await c.ping();
    } finally {
      await c.disconnect();
    }
  }

  await t.test("verified with CA file", async () => {
    const s = await pongServer(serverOpts);
    assert.equal(await ping(s.port, { caFile: srv.cert }), true);
    s.close();
  });
  await t.test("server name override", async () => {
    const s = await pongServer(serverOpts);
    assert.equal(await ping(s.port, { caFile: srv.cert, serverName: "localhost" }), true);
    s.close();
  });
  await t.test("untrusted CA rejected by default", async () => {
    const s = await pongServer(serverOpts);
    await assert.rejects(ping(s.port, { caFile: other.cert }));
    s.close();
  });
  await t.test("wrong server name rejected", async () => {
    const s = await pongServer(serverOpts);
    await assert.rejects(ping(s.port, { caFile: srv.cert, serverName: "not-localhost" }));
    s.close();
  });
  await t.test("insecure skip verify", async () => {
    const s = await pongServer(serverOpts);
    assert.equal(await ping(s.port, { insecureSkipVerify: true }), true);
    s.close();
  });
  await t.test("mTLS with client cert", async () => {
    const s = await pongServer(mtlsOpts);
    assert.equal(
      await ping(s.port, { caFile: srv.cert, certFile: cli.cert, keyFile: cli.key }), true);
    s.close();
  });
  await t.test("mTLS without client cert fails", async () => {
    const s = await pongServer(mtlsOpts);
    // TLS 1.3 reports the rejected client certificate on the first read, not the handshake.
    const c = new VectorVoltClient("127.0.0.1", s.port, { caFile: srv.cert });
    let failed = false;
    try {
      await c.connect();
      await c.get(1n);
    } catch {
      failed = true;
    } finally {
      await c.disconnect();
    }
    assert.equal(failed, true);
    s.close();
  });
  await t.test("cert without key is a config error", () => {
    assert.throws(() => new VectorVoltClient("127.0.0.1", 1, { certFile: cli.cert }));
  });
});
