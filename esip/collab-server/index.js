/**
 * ESIP Collaboration WebSocket Server
 * Compatible with y-websocket protocol (y-websocket v3).
 *
 * Run: node collab-server/index.js
 * Or:  PORT=1234 node collab-server/index.js
 */

import http from 'http';
import { WebSocketServer } from 'ws';
import * as Y from 'yjs';
import * as syncProtocol from 'y-protocols/sync';
import * as awarenessProtocol from 'y-protocols/awareness';
import * as encoding from 'lib0/encoding';
import * as decoding from 'lib0/decoding';

const PORT = process.env.PORT || 1234;

// y-websocket message types
const MESSAGE_SYNC = 0;
const MESSAGE_AWARENESS = 1;

// Map of roomName -> { doc, awareness, conns: Set<WebSocket>, connIds: Map<WebSocket, number[]> }
const rooms = new Map();

function getRoom(name) {
  if (rooms.has(name)) return rooms.get(name);

  const doc = new Y.Doc();
  const awareness = new awarenessProtocol.Awareness(doc);
  const room = { doc, awareness, conns: new Set(), connIds: new Map() };
  rooms.set(name, room);

  // Broadcast doc updates to all peers except the origin
  doc.on('update', (update, origin) => {
    const enc = encoding.createEncoder();
    encoding.writeVarUint(enc, MESSAGE_SYNC);
    syncProtocol.writeUpdate(enc, update);
    const msg = encoding.toUint8Array(enc);
    room.conns.forEach(conn => {
      if (conn !== origin && conn.readyState === 1 /* OPEN */) {
        conn.send(msg, { binary: true });
      }
    });
  });

  // Broadcast awareness changes to all peers
  awareness.on('update', ({ added, updated, removed }) => {
    const changed = [...added, ...updated, ...removed];
    const enc = encoding.createEncoder();
    encoding.writeVarUint(enc, MESSAGE_AWARENESS);
    encoding.writeVarUint8Array(enc, awarenessProtocol.encodeAwarenessUpdate(awareness, changed));
    const msg = encoding.toUint8Array(enc);
    room.conns.forEach(conn => {
      if (conn.readyState === 1) conn.send(msg, { binary: true });
    });
  });

  return room;
}

function setupConnection(ws, req) {
  // Room name comes from the URL path, e.g. ws://localhost:1234/esip-doc-abc123
  const roomName = (req.url || '/').slice(1).split('?')[0] || 'default';
  const room = getRoom(roomName);

  ws.binaryType = 'arraybuffer';
  room.conns.add(ws);
  room.connIds.set(ws, []);

  // Send sync step 1 so client sends us its state vector
  {
    const enc = encoding.createEncoder();
    encoding.writeVarUint(enc, MESSAGE_SYNC);
    syncProtocol.writeSyncStep1(enc, room.doc);
    ws.send(encoding.toUint8Array(enc), { binary: true });
  }

  // Send current awareness states to the new client
  {
    const states = room.awareness.getStates();
    if (states.size > 0) {
      const enc = encoding.createEncoder();
      encoding.writeVarUint(enc, MESSAGE_AWARENESS);
      encoding.writeVarUint8Array(enc, awarenessProtocol.encodeAwarenessUpdate(room.awareness, Array.from(states.keys())));
      ws.send(encoding.toUint8Array(enc), { binary: true });
    }
  }

  ws.on('message', (raw) => {
    const data = raw instanceof ArrayBuffer ? new Uint8Array(raw) : new Uint8Array(raw.buffer, raw.byteOffset, raw.byteLength);
    try {
      const dec = decoding.createDecoder(data);
      const msgType = decoding.readVarUint(dec);

      if (msgType === MESSAGE_SYNC) {
        const enc = encoding.createEncoder();
        encoding.writeVarUint(enc, MESSAGE_SYNC);
        // readSyncMessage handles step1 (write step2 to enc), step2 (apply update), and update
        syncProtocol.readSyncMessage(dec, enc, room.doc, ws);
        if (encoding.length(enc) > 1) {
          ws.send(encoding.toUint8Array(enc), { binary: true });
        }
      } else if (msgType === MESSAGE_AWARENESS) {
        const update = decoding.readVarUint8Array(dec);
        // Track client IDs for cleanup on disconnect
        const decoded = awarenessProtocol.decodeAwarenessUpdate(update);
        const ids = room.connIds.get(ws) || [];
        decoded.forEach((_, clientId) => {
          if (!ids.includes(clientId)) ids.push(clientId);
        });
        room.connIds.set(ws, ids);
        awarenessProtocol.applyAwarenessUpdate(room.awareness, update, ws);
      }
    } catch (err) {
      console.error('[collab] message error:', err.message);
    }
  });

  ws.on('close', () => {
    room.conns.delete(ws);
    // Remove awareness states for this connection's clients
    const ids = room.connIds.get(ws) || [];
    room.connIds.delete(ws);
    if (ids.length > 0) {
      awarenessProtocol.removeAwarenessStates(room.awareness, ids, null);
    }
    // Clean up empty rooms
    if (room.conns.size === 0) {
      rooms.delete(roomName);
    }
  });

  ws.on('error', (err) => {
    console.error('[collab] ws error:', err.message);
  });
}

const server = http.createServer((_req, res) => {
  res.writeHead(200, { 'Content-Type': 'text/plain' });
  res.end('ESIP collab server OK\n');
});

const wss = new WebSocketServer({ server });
wss.on('connection', setupConnection);

server.listen(PORT, () => {
  console.log(`\nESIP collaboration server ready`);
  console.log(`  WebSocket: ws://localhost:${PORT}`);
  console.log(`  HTTP:      http://localhost:${PORT}\n`);
  console.log('Open the same document in multiple browser windows to collaborate in real time.\n');
});
