"""Unit tests for VectorVoltClient — all I/O is mocked via unittest.mock."""

import json
import struct
import unittest
from unittest.mock import MagicMock, call, patch

from vectorvolt.client import Row, VectorVoltClient


def _encode(payload: dict) -> bytes:
    """Encode a dict into the wire-format used by the server response."""
    raw = json.dumps(payload).encode()
    return struct.pack("<I", len(raw)) + raw


def _make_sock(responses: list[dict]) -> MagicMock:
    """Return a mock socket whose recv(n) always returns exactly n bytes."""
    sock = MagicMock()
    recv_data = b"".join(_encode(r) for r in responses)
    pos = [0]

    def _recv(n: int) -> bytes:
        chunk = recv_data[pos[0] : pos[0] + n]
        pos[0] += n
        return chunk

    sock.recv.side_effect = _recv
    return sock


class TestConnectDisconnect(unittest.TestCase):
    def test_connect_calls_socket_connect(self):
        client = VectorVoltClient("127.0.0.1", 5050)
        mock_sock = MagicMock()
        with patch("vectorvolt.client.socket.socket", return_value=mock_sock):
            client.connect()
        mock_sock.connect.assert_called_once_with(("127.0.0.1", 5050))

    def test_disconnect_closes_socket(self):
        client = VectorVoltClient()
        mock_sock = MagicMock()
        client._sock = mock_sock
        client.disconnect()
        mock_sock.close.assert_called_once()
        self.assertIsNone(client._sock)

    def test_disconnect_idempotent_when_not_connected(self):
        client = VectorVoltClient()
        client.disconnect()  # Should not raise

    def test_context_manager_auto_disconnects(self):
        client = VectorVoltClient()
        mock_sock = MagicMock()
        mock_sock.recv.side_effect = []
        with patch("vectorvolt.client.socket.socket", return_value=mock_sock):
            with client:
                pass
        mock_sock.close.assert_called_once()


class TestInsert(unittest.TestCase):
    def _client_with(self, responses):
        client = VectorVoltClient()
        client._sock = _make_sock(responses)
        return client

    def test_insert_returns_seq(self):
        client = self._client_with([{"seq": 7}])
        result = client.insert(1, {"name": "Alice"})
        self.assertEqual(result, 7)

    def test_insert_with_nested_dict(self):
        client = self._client_with([{"seq": 42}])
        result = client.insert(2, {"meta": {"a": 1, "b": [1, 2, 3]}})
        self.assertEqual(result, 42)

    def test_multiple_inserts_increment_seq(self):
        client = VectorVoltClient()
        client._sock = _make_sock([{"seq": 1}, {"seq": 2}])
        seq1 = client.insert(1, {})
        seq2 = client.insert(2, {})
        self.assertEqual(seq1, 1)
        self.assertEqual(seq2, 2)


class TestGet(unittest.TestCase):
    def _client_with(self, responses):
        client = VectorVoltClient()
        client._sock = _make_sock(responses)
        return client

    def test_get_returns_row(self):
        client = self._client_with([{"found": True, "pk": 5, "data": {"x": 99}}])
        row = client.get(5)
        self.assertIsNotNone(row)
        self.assertEqual(row.pk, 5)
        self.assertEqual(row.data["x"], 99)

    def test_get_returns_none_on_miss(self):
        client = self._client_with([{"found": False}])
        row = client.get(999)
        self.assertIsNone(row)

    def test_get_parses_pk_correctly(self):
        client = self._client_with([{"found": True, "pk": 1234567890, "data": {}}])
        row = client.get(1234567890)
        self.assertEqual(row.pk, 1234567890)


class TestDelete(unittest.TestCase):
    def _client_with(self, responses):
        client = VectorVoltClient()
        client._sock = _make_sock(responses)
        return client

    def test_delete_returns_seq(self):
        client = self._client_with([{"seq": 3}])
        result = client.delete(1)
        self.assertEqual(result, 3)

    def test_delete_nonexistent_returns_0(self):
        client = self._client_with([{"seq": 0}])
        result = client.delete(9999)
        self.assertEqual(result, 0)


class TestPing(unittest.TestCase):
    def test_ping_returns_true(self):
        client = VectorVoltClient()
        client._sock = _make_sock([{"pong": True}])
        self.assertTrue(client.ping())

    def test_ping_on_closed_socket_returns_false(self):
        client = VectorVoltClient()
        # _sock is None — ping must not raise, just return False
        self.assertFalse(client.ping())


if __name__ == "__main__":
    unittest.main()
