"""Raw mock server shared by the SDK tests: serves one connection, records frames."""

import socket
import ssl
import threading
from typing import Optional

from vectorvolt import wire


class RawServer:
    """Replies with handler(op, request_id, body) -> frame bytes; keeps raw request frames.

    With ``ssl_context`` the accepted connection is wrapped in TLS (server side).
    """

    def __init__(self, handler, ssl_context: Optional[ssl.SSLContext] = None):
        self.handler = handler
        self.frames = []
        self.error = None
        self.srv = socket.socket()
        self.srv.bind(("127.0.0.1", 0))
        self.srv.listen(1)
        self.port = self.srv.getsockname()[1]
        self._ctx = ssl_context
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
            if self._ctx is not None:
                c = self._ctx.wrap_socket(c, server_side=True)
            while True:
                hdr = self._read(c, wire.HEADER_SIZE)
                op, _, rid, plen, _ = wire.decode_header(hdr)
                body = self._read(c, plen) if plen else b""
                self.frames.append(hdr + body)
                c.sendall(self.handler(op, rid, body))
        except (EOFError, OSError, ssl.SSLError) as e:
            self.error = e
        finally:
            try:
                c.close()
            except OSError:
                pass

    def close(self):
        self.srv.close()
