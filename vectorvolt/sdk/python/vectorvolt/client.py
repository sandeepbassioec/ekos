"""VectorVolt Python SDK — TCP client."""

import json
import socket
import struct
from dataclasses import dataclass
from typing import Any, Optional


@dataclass
class Row:
    pk: int
    data: dict[str, Any]


class VectorVoltClient:
    """Minimal TCP client for VectorVolt.

    Wire format (little-endian):
      Request:  [4-byte len][JSON payload]
      Response: [4-byte len][JSON payload]
    """

    def __init__(self, host: str = "127.0.0.1", port: int = 5050) -> None:
        self._host = host
        self._port = port
        self._sock: Optional[socket.socket] = None

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    def connect(self) -> None:
        """Open a TCP connection to the VectorVolt server."""
        self._sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self._sock.connect((self._host, self._port))

    def disconnect(self) -> None:
        """Close the TCP connection (idempotent)."""
        if self._sock is not None:
            try:
                self._sock.close()
            finally:
                self._sock = None

    # ------------------------------------------------------------------
    # Context-manager support
    # ------------------------------------------------------------------

    def __enter__(self) -> "VectorVoltClient":
        self.connect()
        return self

    def __exit__(self, *_: Any) -> None:
        self.disconnect()

    # ------------------------------------------------------------------
    # Private helpers
    # ------------------------------------------------------------------

    def _send(self, payload: dict) -> dict:
        if self._sock is None:
            raise ConnectionError("Not connected — call connect() first")
        raw = json.dumps(payload).encode()
        self._sock.sendall(struct.pack("<I", len(raw)) + raw)
        length_bytes = self._recv_exact(4)
        (length,) = struct.unpack("<I", length_bytes)
        response_bytes = self._recv_exact(length)
        return json.loads(response_bytes)

    def _recv_exact(self, n: int) -> bytes:
        buf = bytearray()
        while len(buf) < n:
            chunk = self._sock.recv(n - len(buf))
            if not chunk:
                raise EOFError("Connection closed by server")
            buf.extend(chunk)
        return bytes(buf)

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def insert(self, pk: int, data: dict) -> int:
        """Insert a row.  Returns the WAL sequence number."""
        resp = self._send({"op": "insert", "pk": pk, "data": data})
        return int(resp["seq"])

    def get(self, pk: int) -> Optional[Row]:
        """Fetch a row by primary key.  Returns None on cache miss."""
        resp = self._send({"op": "get", "pk": pk})
        if resp.get("found"):
            return Row(pk=int(resp["pk"]), data=resp["data"])
        return None

    def delete(self, pk: int) -> int:
        """Delete a row.  Returns the WAL sequence number (0 if not found)."""
        resp = self._send({"op": "delete", "pk": pk})
        return int(resp.get("seq", 0))

    def ping(self) -> bool:
        """Send a ping to the server.  Returns True on success."""
        try:
            resp = self._send({"op": "ping"})
            return resp.get("pong", False) is True
        except (ConnectionError, EOFError, OSError):
            return False
