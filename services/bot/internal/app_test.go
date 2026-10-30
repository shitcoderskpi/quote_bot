package bot

import (
	"net"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestIsEndpointReachable_Success(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer ts.Close()

	if !isEndpointReachable(ts.URL, 500*time.Millisecond) {
		t.Fatalf("expected reachable for test server %s", ts.URL)
	}
}

func TestIsEndpointReachable_Unreachable(t *testing.T) {
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("failed to listen: %v", err)
	}
	addr := l.Addr().String()
	_ = l.Close()

	if isEndpointReachable("http://"+addr, 200*time.Millisecond) {
		t.Fatalf("expected unreachable for closed port %s", addr)
	}
}

func TestIsEndpointReachable_InvalidURL(t *testing.T) {
	if isEndpointReachable(":::invalid", 200*time.Millisecond) {
		t.Fatalf("expected unreachable for invalid url")
	}
}

