package vectorvolt_test

import (
	"encoding/binary"
	"encoding/json"
	"net"
	"sync"
	"testing"

	vectorvolt "github.com/workerruntime/vectorvolt-go/vectorvolt"
)

// mockServer spins up a tiny in-process TCP server that echoes scripted responses.
type mockServer struct {
	ln        net.Listener
	responses []map[string]any
	mu        sync.Mutex
	idx       int
}

func newMockServer(t *testing.T, responses []map[string]any) (*mockServer, string) {
	t.Helper()
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	s := &mockServer{ln: ln, responses: responses}
	go s.serve()
	return s, ln.Addr().String()
}

func (s *mockServer) serve() {
	for {
		conn, err := s.ln.Accept()
		if err != nil {
			return
		}
		go s.handle(conn)
	}
}

func (s *mockServer) handle(conn net.Conn) {
	defer conn.Close()
	lenBuf := make([]byte, 4)
	for {
		if _, err := conn.Read(lenBuf); err != nil {
			return
		}
		n := binary.LittleEndian.Uint32(lenBuf)
		buf := make([]byte, n)
		if _, err := conn.Read(buf); err != nil {
			return
		}
		s.mu.Lock()
		if s.idx >= len(s.responses) {
			s.mu.Unlock()
			return
		}
		resp := s.responses[s.idx]
		s.idx++
		s.mu.Unlock()

		raw, _ := json.Marshal(resp)
		out := make([]byte, 4+len(raw))
		binary.LittleEndian.PutUint32(out, uint32(len(raw)))
		copy(out[4:], raw)
		conn.Write(out)
	}
}

func (s *mockServer) close() { s.ln.Close() }

func parseAddr(addr string) (string, int) {
	host, portStr := net.SplitHostPort(addr)
	port := 0
	for _, b := range portStr {
		port = port*10 + int(b-'0')
	}
	return host, port
}

// ---- Tests ----

func TestNew(t *testing.T) {
	c := vectorvolt.New("127.0.0.1", 5050)
	if c == nil {
		t.Fatal("New returned nil")
	}
}

func TestConnect(t *testing.T) {
	srv, addr := newMockServer(t, nil)
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	if err := c.Connect(); err != nil {
		t.Fatal(err)
	}
	c.Close()
}

func TestClose(t *testing.T) {
	srv, addr := newMockServer(t, nil)
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	c.Connect()
	if err := c.Close(); err != nil {
		t.Fatal(err)
	}
}

func TestDoubleCloseIsSafe(t *testing.T) {
	srv, addr := newMockServer(t, nil)
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	c.Connect()
	c.Close()
	if err := c.Close(); err != nil {
		t.Fatal(err)
	}
}

func TestInsertReturnsSeq(t *testing.T) {
	srv, addr := newMockServer(t, []map[string]any{{"seq": float64(7)}})
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	c.Connect()
	defer c.Close()
	seq, err := c.Insert(1, map[string]any{"name": "Alice"})
	if err != nil {
		t.Fatal(err)
	}
	if seq != 7 {
		t.Fatalf("expected seq=7 got %d", seq)
	}
}

func TestGetReturnsRow(t *testing.T) {
	srv, addr := newMockServer(t, []map[string]any{{"found": true, "pk": float64(5), "data": map[string]any{"x": float64(99)}}})
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	c.Connect()
	defer c.Close()
	row, err := c.Get(5)
	if err != nil || row == nil {
		t.Fatalf("expected row, err=%v", err)
	}
	if row.PK != 5 {
		t.Fatalf("expected PK=5 got %d", row.PK)
	}
}

func TestGetReturnsNilOnMiss(t *testing.T) {
	srv, addr := newMockServer(t, []map[string]any{{"found": false}})
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	c.Connect()
	defer c.Close()
	row, err := c.Get(999)
	if err != nil {
		t.Fatal(err)
	}
	if row != nil {
		t.Fatal("expected nil row on miss")
	}
}

func TestDeleteReturnsSeq(t *testing.T) {
	srv, addr := newMockServer(t, []map[string]any{{"seq": float64(3)}})
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	c.Connect()
	defer c.Close()
	seq, err := c.Delete(1)
	if err != nil {
		t.Fatal(err)
	}
	if seq != 3 {
		t.Fatalf("expected seq=3 got %d", seq)
	}
}

func TestPingReturnsTrue(t *testing.T) {
	srv, addr := newMockServer(t, []map[string]any{{"pong": true}})
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	c.Connect()
	defer c.Close()
	if !c.Ping() {
		t.Fatal("expected Ping to return true")
	}
}

func TestInsertWithMapData(t *testing.T) {
	srv, addr := newMockServer(t, []map[string]any{{"seq": float64(42)}})
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	c.Connect()
	defer c.Close()
	seq, err := c.Insert(2, map[string]any{"nested": map[string]any{"a": 1}})
	if err != nil || seq != 42 {
		t.Fatalf("err=%v seq=%d", err, seq)
	}
}

func TestGetParsesPKCorrectly(t *testing.T) {
	srv, addr := newMockServer(t, []map[string]any{{"found": true, "pk": float64(1234567890), "data": map[string]any{}}})
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	c.Connect()
	defer c.Close()
	row, err := c.Get(1234567890)
	if err != nil || row == nil {
		t.Fatal(err)
	}
	if row.PK != 1234567890 {
		t.Fatalf("expected PK=1234567890 got %d", row.PK)
	}
}

func TestConcurrentInsertsDontRace(t *testing.T) {
	// Build responses for 10 concurrent inserts.
	responses := make([]map[string]any, 10)
	for i := range responses {
		responses[i] = map[string]any{"seq": float64(i + 1)}
	}
	srv, addr := newMockServer(t, responses)
	defer srv.close()
	host, port := parseAddr(addr)
	c := vectorvolt.New(host, port)
	c.Connect()
	defer c.Close()

	var wg sync.WaitGroup
	for i := range 10 {
		wg.Add(1)
		go func(pk int) {
			defer wg.Done()
			c.Insert(uint64(pk), map[string]any{"v": pk})
		}(i)
	}
	wg.Wait()
}

func TestPingOnClosedConnReturnsFalse(t *testing.T) {
	c := vectorvolt.New("127.0.0.1", 5050) // no server; conn is nil
	if c.Ping() {
		t.Fatal("expected false when not connected")
	}
}
