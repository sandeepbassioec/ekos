import { useEffect, useRef, useState } from 'react';
import * as Y from 'yjs';
import { IndexeddbPersistence } from 'y-indexeddb';
import { WebrtcProvider } from 'y-webrtc';

export interface CollabUser {
  clientId: number;
  name: string;
  color: string;
}

export interface CollabState {
  ydoc: Y.Doc;
  provider: WebrtcProvider | null;
  connected: boolean;
  peers: number;
  localUser: { name: string; color: string };
}

const USER_COLORS = ['#7c3aed','#0891b2','#059669','#d97706','#dc2626','#db2777'];

function randomColor() { return USER_COLORS[Math.floor(Math.random() * USER_COLORS.length)]; }

// Stable per-tab user color stored in sessionStorage
function getLocalUser(displayName?: string): { name: string; color: string } {
  const stored = sessionStorage.getItem('esip-collab-user');
  if (stored) { try { return JSON.parse(stored); } catch { /* fall through */ } }
  const user = { name: displayName || 'Anonymous', color: randomColor() };
  sessionStorage.setItem('esip-collab-user', JSON.stringify(user));
  return user;
}

export function useCollaboration(roomId: string | undefined, displayName?: string): CollabState | null {
  const ydocRef = useRef<Y.Doc | null>(null);
  const persistRef = useRef<IndexeddbPersistence | null>(null);
  const providerRef = useRef<WebrtcProvider | null>(null);
  const [connected, setConnected] = useState(false);
  const [peers, setPeers] = useState(0);
  const [state, setState] = useState<CollabState | null>(null);

  useEffect(() => {
    if (!roomId) return;

    const localUser = getLocalUser(displayName);
    const ydoc = new Y.Doc();
    ydocRef.current = ydoc;

    // Local persistence via IndexedDB
    const persist = new IndexeddbPersistence(`esip-doc-${roomId}`, ydoc);
    persistRef.current = persist;

    // WebRTC provider for P2P real-time sync
    let provider: WebrtcProvider | null = null;
    try {
      provider = new WebrtcProvider(`esip-doc-${roomId}`, ydoc, {
        signaling: ['wss://signaling.yjs.dev'],
        maxConns: 20,
        filterBcConns: false,
      });
      providerRef.current = provider;

      // Set local awareness state (cursor colour + name)
      provider.awareness.setLocalStateField('user', localUser);

      // Track connectivity
      const onStatus = ({ connected: c }: { connected: boolean }) => setConnected(c);
      provider.on('status', onStatus);

      // Track peer count from awareness
      const onAwareness = () => {
        const states = provider!.awareness.getStates();
        setPeers(states.size - 1); // exclude self
      };
      provider.awareness.on('change', onAwareness);
      onAwareness();
    } catch {
      // WebRTC unavailable (e.g., no signaling server) — local-only mode
    }

    setState({ ydoc, provider, connected: false, peers: 0, localUser });

    return () => {
      provider?.destroy();
      persist.destroy();
      ydoc.destroy();
      providerRef.current = null;
      persistRef.current = null;
      ydocRef.current = null;
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [roomId]);

  // Keep connected/peers in sync
  useEffect(() => {
    if (!state) return;
    setState(s => s ? { ...s, connected, peers } : s);
  }, [connected, peers]); // eslint-disable-line react-hooks/exhaustive-deps

  return state;
}
