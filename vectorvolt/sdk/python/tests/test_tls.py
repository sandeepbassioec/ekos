"""TLS client tests against a local mock TLS server (self-signed cert generated in-test).

Certificates come from the ``openssl`` CLI (stdlib has no cert generator); the tests are
skipped when it is not installed. No network access is used.
"""

import os
import shutil
import ssl
import subprocess
import tempfile
import unittest

from tests._mock import RawServer
from vectorvolt import VectorVoltClient, wire

OPENSSL = shutil.which("openssl")


def make_cert(d, name, cn="localhost", san="DNS:localhost,IP:127.0.0.1"):
    cert, key = os.path.join(d, name + ".crt"), os.path.join(d, name + ".key")
    subprocess.run(
        [OPENSSL, "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", key, "-out", cert,
         "-days", "2", "-subj", "/CN=" + cn, "-addext", "subjectAltName=" + san],
        check=True, capture_output=True)
    return cert, key


def pong(op, rid, body):
    return wire.encode_frame(rid, wire.PONG)


@unittest.skipUnless(OPENSSL, "openssl CLI not available")
class TlsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.dir = tempfile.mkdtemp()
        cls.server_cert, cls.server_key = make_cert(cls.dir, "server")
        cls.other_cert, _ = make_cert(cls.dir, "other", cn="other", san="DNS:other")
        cls.client_cert, cls.client_key = make_cert(cls.dir, "client", cn="client")

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.dir, ignore_errors=True)

    def server_ctx(self, require_client=False):
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        ctx.load_cert_chain(self.server_cert, self.server_key)
        if require_client:
            ctx.verify_mode = ssl.CERT_REQUIRED
            ctx.load_verify_locations(self.client_cert)
        return ctx

    def serve(self, **kw):
        s = RawServer(pong, self.server_ctx(kw.pop("require_client", False)))
        self.addCleanup(s.close)
        return s

    def test_verified_connection_with_ca_file(self):
        s = self.serve()
        c = VectorVoltClient("127.0.0.1", s.port, timeout=5, tls=True, ca_file=self.server_cert)
        c.connect()
        self.addCleanup(c.disconnect)
        self.assertTrue(c.ping())

    def test_server_name_override(self):
        s = self.serve()
        c = VectorVoltClient("127.0.0.1", s.port, timeout=5, tls=True, ca_file=self.server_cert,
                             server_name="localhost")
        c.connect()
        self.addCleanup(c.disconnect)
        self.assertTrue(c.ping())

    def test_untrusted_cert_is_rejected_by_default(self):
        s = self.serve()
        c = VectorVoltClient("127.0.0.1", s.port, timeout=5, tls=True, ca_file=self.other_cert)
        with self.assertRaises(ssl.SSLError):
            c.connect()
        self.assertIsNone(c._sock)

    def test_wrong_server_name_is_rejected(self):
        s = self.serve()
        c = VectorVoltClient("127.0.0.1", s.port, timeout=5, tls=True, ca_file=self.server_cert,
                             server_name="not-localhost")
        with self.assertRaises(ssl.SSLError):
            c.connect()

    def test_insecure_skip_verify(self):
        s = self.serve()
        c = VectorVoltClient("127.0.0.1", s.port, timeout=5, tls=True, insecure_skip_verify=True)
        c.connect()
        self.addCleanup(c.disconnect)
        self.assertTrue(c.ping())

    def test_mtls_requires_client_cert(self):
        s = self.serve(require_client=True)
        c = VectorVoltClient("127.0.0.1", s.port, timeout=5, tls=True, ca_file=self.server_cert,
                             client_cert=self.client_cert, client_key=self.client_key)
        c.connect()
        self.addCleanup(c.disconnect)
        self.assertTrue(c.ping())

    def test_mtls_without_client_cert_fails(self):
        s = self.serve(require_client=True)
        c = VectorVoltClient("127.0.0.1", s.port, timeout=5, tls=True, ca_file=self.server_cert)
        # TLS 1.3 reports the rejected client certificate on the first read, not the handshake.
        with self.assertRaises((ssl.SSLError, ConnectionError, EOFError, OSError)):
            c.connect()
            c.get(1)  # get() surfaces the transport error (ping() would swallow it)

    def test_plain_default_and_option_validation(self):
        self.assertIsNone(VectorVoltClient()._ssl)
        with self.assertRaises(ValueError):
            VectorVoltClient(ca_file="x")
        with self.assertRaises(ValueError):
            VectorVoltClient(tls=True, client_cert="c")


if __name__ == "__main__":
    unittest.main()
