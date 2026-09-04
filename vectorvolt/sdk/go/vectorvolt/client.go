// Package vectorvolt provides a minimal TCP client for VectorVolt.
//
// Wire format (little-endian):
//
//	Request:  [4-byte uint32 len][JSON payload]
//	Response: [4-byte uint32 len][JSON payload]
package vectorvolt

import (
	"encoding/binary"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"sync"
)

// Row represents a single record returned from the server.
type Row struct {
	PK   uint64
	Data map[string]any
}

// Client is a TCP client for a single VectorVolt server.
type Client struct {
	addr string
	mu   sync.Mutex
	conn net.Conn
}

// New creates a new Client targeting host:port.
func New(host string, port int) *Client {
	return &Client{addr: fmt.Sprintf("%s:%d", host, port)}
}

// Connect opens a TCP connection to the server.
func (c *Client) Connect() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	conn, err := net.Dial("tcp", c.addr)
	if err != nil {
		return err
	}
	c.conn = conn
	return nil
}

// Close shuts down the connection.  Calling Close on an already-closed
// Client is safe.
func (c *Client) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.conn == nil {
		return nil
	}
	err := c.conn.Close()
	c.conn = nil
	return err
}

// Insert stores a row and returns the WAL sequence number.
func (c *Client) Insert(pk uint64, data map[string]any) (uint64, error) {
	req := map[string]any{"op": "insert", "pk": pk, "data": data}
	resp, err := c.roundTrip(req)
	if err != nil {
		return 0, err
	}
	return uint64(jsonUint(resp["seq"])), nil
}

// Get fetches a row by primary key.  Returns nil when not found.
func (c *Client) Get(pk uint64) (*Row, error) {
	req := map[string]any{"op": "get", "pk": pk}
	resp, err := c.roundTrip(req)
	if err != nil {
		return nil, err
	}
	if found, _ := resp["found"].(bool); !found {
		return nil, nil
	}
	row := &Row{
		PK:   uint64(jsonUint(resp["pk"])),
		Data: map[string]any{},
	}
	if d, ok := resp["data"].(map[string]any); ok {
		row.Data = d
	}
	return row, nil
}

// Delete removes a row and returns the WAL sequence number (0 when not found).
func (c *Client) Delete(pk uint64) (uint64, error) {
	req := map[string]any{"op": "delete", "pk": pk}
	resp, err := c.roundTrip(req)
	if err != nil {
		return 0, err
	}
	return uint64(jsonUint(resp["seq"])), nil
}

// Ping sends a health-check to the server.  Returns false on any error.
func (c *Client) Ping() bool {
	req := map[string]any{"op": "ping"}
	resp, err := c.roundTrip(req)
	if err != nil {
		return false
	}
	pong, _ := resp["pong"].(bool)
	return pong
}

// roundTrip sends a request and reads the response.
func (c *Client) roundTrip(req map[string]any) (map[string]any, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.conn == nil {
		return nil, fmt.Errorf("vectorvolt: not connected")
	}
	raw, err := json.Marshal(req)
	if err != nil {
		return nil, err
	}
	// Write length-prefixed frame.
	lenBuf := make([]byte, 4)
	binary.LittleEndian.PutUint32(lenBuf, uint32(len(raw)))
	if _, err = c.conn.Write(append(lenBuf, raw...)); err != nil {
		return nil, err
	}
	// Read response.
	if _, err = io.ReadFull(c.conn, lenBuf); err != nil {
		return nil, err
	}
	respLen := binary.LittleEndian.Uint32(lenBuf)
	respBuf := make([]byte, respLen)
	if _, err = io.ReadFull(c.conn, respBuf); err != nil {
		return nil, err
	}
	var resp map[string]any
	if err = json.Unmarshal(respBuf, &resp); err != nil {
		return nil, err
	}
	return resp, nil
}

// jsonUint safely converts a JSON number (float64) to uint64.
func jsonUint(v any) float64 {
	f, _ := v.(float64)
	return f
}
