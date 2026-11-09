package vectorvolt_test

// TLS client tests against a local mock TLS server. Certificates are generated
// in-test (stdlib crypto/x509); no network access.

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/pem"
	"io"
	"math/big"
	"net"
	"os"
	"path/filepath"
	"testing"
	"time"

	vv "github.com/workerruntime/vectorvolt-go/vectorvolt"
)

type testCert struct {
	cert     tls.Certificate
	certFile string
	keyFile  string
	x509     *x509.Certificate
}

func newCert(t *testing.T, dir, name, cn string, dnsNames ...string) testCert {
	t.Helper()
	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	tmpl := &x509.Certificate{
		SerialNumber:          big.NewInt(time.Now().UnixNano()),
		Subject:               pkix.Name{CommonName: cn},
		NotBefore:             time.Now().Add(-time.Hour),
		NotAfter:              time.Now().Add(2 * time.Hour),
		KeyUsage:              x509.KeyUsageDigitalSignature | x509.KeyUsageCertSign,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth, x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
		IsCA:                  true,
		DNSNames:              dnsNames,
		IPAddresses:           []net.IP{net.ParseIP("127.0.0.1")},
	}
	der, err := x509.CreateCertificate(rand.Reader, tmpl, tmpl, &key.PublicKey, key)
	if err != nil {
		t.Fatal(err)
	}
	keyDER, err := x509.MarshalECPrivateKey(key)
	if err != nil {
		t.Fatal(err)
	}
	certPEM := pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: der})
	keyPEM := pem.EncodeToMemory(&pem.Block{Type: "EC PRIVATE KEY", Bytes: keyDER})
	tc := testCert{certFile: filepath.Join(dir, name+".crt"), keyFile: filepath.Join(dir, name+".key")}
	if err := os.WriteFile(tc.certFile, certPEM, 0o600); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(tc.keyFile, keyPEM, 0o600); err != nil {
		t.Fatal(err)
	}
	if tc.cert, err = tls.X509KeyPair(certPEM, keyPEM); err != nil {
		t.Fatal(err)
	}
	tc.x509, _ = x509.ParseCertificate(der)
	return tc
}

// tlsPongServer answers every request with Pong over TLS; returns the port.
func tlsPongServer(t *testing.T, cfg *tls.Config) int {
	t.Helper()
	ln, err := tls.Listen("tcp", "127.0.0.1:0", cfg)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { ln.Close() })
	go func() {
		for {
			c, err := ln.Accept()
			if err != nil {
				return
			}
			go func() {
				defer c.Close()
				for {
					hdr := make([]byte, vv.HeaderSize)
					if _, err := io.ReadFull(c, hdr); err != nil {
						return
					}
					_, _, id, plen, _, err := vv.DecodeHeader(hdr)
					if err != nil {
						return
					}
					if _, err := io.ReadFull(c, make([]byte, plen)); err != nil {
						return
					}
					if _, err := c.Write(frame(id, vv.OpPong, nil)); err != nil {
						return
					}
				}
			}()
		}
	}()
	return ln.Addr().(*net.TCPAddr).Port
}

func TestTLS(t *testing.T) {
	dir := t.TempDir()
	srv := newCert(t, dir, "server", "localhost", "localhost")
	other := newCert(t, dir, "other", "other", "other")
	cli := newCert(t, dir, "client", "client", "client")

	clientPool := x509.NewCertPool()
	clientPool.AddCert(cli.x509)
	plain := &tls.Config{Certificates: []tls.Certificate{srv.cert}}
	mtls := &tls.Config{
		Certificates: []tls.Certificate{srv.cert},
		ClientAuth:   tls.RequireAndVerifyClientCert,
		ClientCAs:    clientPool,
	}

	connectPing := func(t *testing.T, port int, o vv.TLSOptions) error {
		t.Helper()
		c := vv.NewTLS("127.0.0.1", port, o)
		c.Timeout = 5 * time.Second
		if err := c.Connect(); err != nil {
			return err
		}
		defer c.Close()
		return c.Ping()
	}

	t.Run("verified with CA file", func(t *testing.T) {
		if err := connectPing(t, tlsPongServer(t, plain), vv.TLSOptions{CAFile: srv.certFile}); err != nil {
			t.Fatal(err)
		}
	})
	t.Run("server name override", func(t *testing.T) {
		if err := connectPing(t, tlsPongServer(t, plain),
			vv.TLSOptions{CAFile: srv.certFile, ServerName: "localhost"}); err != nil {
			t.Fatal(err)
		}
	})
	t.Run("untrusted CA rejected by default", func(t *testing.T) {
		if err := connectPing(t, tlsPongServer(t, plain), vv.TLSOptions{CAFile: other.certFile}); err == nil {
			t.Fatal("expected certificate verification failure")
		}
	})
	t.Run("wrong server name rejected", func(t *testing.T) {
		if err := connectPing(t, tlsPongServer(t, plain),
			vv.TLSOptions{CAFile: srv.certFile, ServerName: "not-localhost"}); err == nil {
			t.Fatal("expected server name mismatch")
		}
	})
	t.Run("insecure skip verify", func(t *testing.T) {
		if err := connectPing(t, tlsPongServer(t, plain), vv.TLSOptions{InsecureSkipVerify: true}); err != nil {
			t.Fatal(err)
		}
	})
	t.Run("mTLS with client cert", func(t *testing.T) {
		if err := connectPing(t, tlsPongServer(t, mtls),
			vv.TLSOptions{CAFile: srv.certFile, CertFile: cli.certFile, KeyFile: cli.keyFile}); err != nil {
			t.Fatal(err)
		}
	})
	t.Run("mTLS without client cert fails", func(t *testing.T) {
		if err := connectPing(t, tlsPongServer(t, mtls), vv.TLSOptions{CAFile: srv.certFile}); err == nil {
			t.Fatal("expected the server to reject a missing client certificate")
		}
	})
	t.Run("cert without key is a config error", func(t *testing.T) {
		if err := connectPing(t, tlsPongServer(t, plain),
			vv.TLSOptions{CAFile: srv.certFile, CertFile: cli.certFile}); err == nil {
			t.Fatal("expected config error")
		}
	})
	t.Run("plain client against TLS server fails", func(t *testing.T) {
		c := vv.New("127.0.0.1", tlsPongServer(t, plain))
		c.Timeout = 2 * time.Second
		if err := c.Connect(); err != nil {
			return
		}
		defer c.Close()
		if err := c.Ping(); err == nil {
			t.Fatal("plain TCP must not work against a TLS listener")
		}
	})
}
