// Configure the pinned upstream TestService implementation; do not reproduce its handlers.
// Usage: test-service-peer CERT KEY READY_FILE STOP_FILE [observed-peer]
package main

import (
	"net"
	"os"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/interop"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/peer"
)

// The Go helper lacks the catalogue's Fill Peer Socket Address extension. This
// optional adapter adds only that observed transport fact; upstream handlers
// still perform the actual request/response streaming.
type observedService struct{ testgrpc.TestServiceServer }
type observedStream struct {
	grpc.BidiStreamingServer[testgrpc.StreamingOutputCallRequest, testgrpc.StreamingOutputCallResponse]
	requested []bool
	sent      int
}

func (s *observedStream) Recv() (*testgrpc.StreamingOutputCallRequest, error) {
	request, err := s.BidiStreamingServer.Recv()
	s.requested = nil
	s.sent = 0
	if request != nil {
		for _, parameter := range request.GetResponseParameters() {
			s.requested = append(s.requested, parameter.GetFillPeerSocketAddress().GetValue())
		}
	}
	return request, err
}
func (s *observedStream) Send(response *testgrpc.StreamingOutputCallResponse) error {
	response.PeerSocketAddress = ""
	if s.sent < len(s.requested) && s.requested[s.sent] {
		observed, ok := peer.FromContext(s.Context())
		if !ok || observed.Addr == nil {
			panic("missing observed peer socket")
		}
		response.PeerSocketAddress = observed.Addr.String()
	}
	s.sent++
	return s.BidiStreamingServer.Send(response)
}
func (s *observedService) FullDuplexCall(stream grpc.BidiStreamingServer[testgrpc.StreamingOutputCallRequest, testgrpc.StreamingOutputCallResponse]) error {
	return s.TestServiceServer.FullDuplexCall(&observedStream{BidiStreamingServer: stream})
}

func main() {
	if len(os.Args) != 5 && len(os.Args) != 6 {
		panic("supply certificate, key, ready-file, stop-file and optional observed-peer")
	}
	creds, err := credentials.NewServerTLSFromFile(os.Args[1], os.Args[2])
	if err != nil {
		panic(err)
	}
	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		panic(err)
	}
	server := grpc.NewServer(grpc.Creds(creds), grpc.MaxConcurrentStreams(2))
	service := interop.NewTestServer()
	if len(os.Args) == 6 {
		if os.Args[5] != "observed-peer" {
			panic("unknown peer adapter")
		}
		service = &observedService{TestServiceServer: service}
	}
	testgrpc.RegisterTestServiceServer(server, service)
	if err := os.WriteFile(os.Args[3], []byte(listener.Addr().String()), 0600); err != nil {
		panic(err)
	}
	done := make(chan error, 1)
	go func() { done <- server.Serve(listener) }()
	for {
		if _, err := os.Stat(os.Args[4]); err == nil {
			break
		} else if !os.IsNotExist(err) {
			panic(err)
		}
		select {
		case err := <-done:
			panic(err)
		default:
		}
		time.Sleep(10 * time.Millisecond)
	}
	server.GracefulStop()
	if err := <-done; err != nil {
		panic(err)
	}
}
