// gRPC identity/gzip message framing, metadata and canonical status codecs, plus
// native TLS HTTP/2 client/server calls with typed Protobuf message helpers.
//
// A call is a stream of length-prefixed messages ended by a status. The framing, metadata and
// status codecs work on every target; [open_call], [serve_call] and the rest need fibers,
// `Extensions/Io` and TLS, and are not built for wasm.
//
// Framing a message and reading it back:
//
// ```jai
// wire, _ := frame_message(u8.[1, 2, 3]);
// defer free(wire.data);
//
// decoder: Envelope_Decoder;
// defer free_decoder(*decoder);
// feed_envelope(*decoder, wire);
// message := take_message(*decoder);
// defer free(message.data);
// assert(message.count == 3);
// ```
//
// A client call (the channel's `transport.tls` is a configured `Tls.Tls_Config` with `alpn = ["h2"]`):
//
// ```jai,no_run,skip_on(wasm)
// channel: Channel;
// channel.transport.base_url = "https://localhost:50051";
// defer close_channel(*channel);
//
// call, status := open_call(*channel, Method.{ path = "/echo.Echo/Say" });
// assert(status.code == .OK);
// defer close(*call);
//
// send_bytes(*call, u8.[1, 2, 3]);
// close_send(*call);
// message, kind := receive_bytes(*call);
// if kind == .MESSAGE free(message.data);
// final := finish(*call);
// defer free_status(*final);
// ```

#load "wire.jai";
#load "status.jai";
#load "metadata.jai";

#if OS != .WASM {
    #load "call.jai";
    #load "server.jai";
}

#scope_module
#import "Basic";
#import "String";
Base64 :: #import "Base64";
Http :: #import "Extensions/Http";
Deflate :: #import "Extensions/Deflate";
Proto :: #import "Extensions/Protobuf";

#if OS != .WASM {
    Io :: #import "Extensions/Io";
}
