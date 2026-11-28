// HTTP for fibers: a server, a client, in-memory test tools, and the message codecs under them.
// @experimental
//
// - Server: register handlers on an [App] with [get] and the other method procedures, then [serve]
//   it. Handlers return a string, a value encoded as JSON, a [Response] or an [Http_Error].
// - Client: [get] and [send] on a [Client]; free each response with [close].
// - Tests: [test_client] sends requests to an [App] in memory, without a socket.
// - Codecs: a [Parser] reads HTTP/1.x requests or responses from buffers the caller fills, in pieces
//   of any size, with strict RFC 9112 framing; the writers build heads and chunked bodies.
//
// HTTP/2 is used when TLS negotiates `h2`; handlers and client calls do not change. The codecs
// touch no socket and build for every target, wasm included; the server, client and test tools
// need `Extensions/Io` and are left out on wasm. Failures are values: the parser's `.ERROR` event
// and `Parser.error`, an [Http_Error] from a handler, a [Client_Error] from a client call. Memory
// a parser or response owns is released with [parser_free] and [close]; everything else is
// borrowed or in temporary storage, as each procedure says.
//
// ```jai,skip_on(wasm)
// hello :: () -> string {
//     return "hello";
// }
//
// main :: () {
//     app: App;
//     defer free_app(*app);
//     get(*app, "/hello", hello);
//
//     api := test_client(*app);
//     defer free_test_client(api);
//     response, error := get(api, "/hello");
//     assert(error.kind == .NONE);
//     defer close(*response);
//     assert(response.status == 200);
// }
// ```
//
// The parser on its own:
//
// ```jai
// parser: Parser;
// parser_init(*parser, .REQUEST);
// defer parser_free(*parser);
//
// request := "GET /users?id=7 HTTP/1.1\r\nHost: example.com\r\n\r\n";
// bytes: [] u8 = .{ request.count, request.data };
//
// used, event := parse(*parser, bytes); // any chunking gives the same result
// assert(event.kind == .HEAD);
// assert(parser.head.method == .GET);
// assert(parser.head.target == "/users?id=7");
// ```

// Switches for the vector code paths, which give the same results as the scalar ones. Set
// http_use_simd to false to force the scalar paths (the tests run both).
http_use_simd := true;

#load "types.jai";
#load "bytes.jai";
#load "parser.jai";
#load "chunked.jai";
#load "writer.jai";
#load "date.jai";
#load "h2_frame.jai";
#load "hpack.jai";

// The server and client need fibers, which WebAssembly code cannot switch: there the module is
// the codec alone.
#if OS != .WASM {
    #load "date_native.jai";
    #load "url.jai";
    #load "message.jai";
    #load "router.jai";
    #load "body.jai";
    #load "stream.jai";
    #load "extract.jai";
    #load "app.jai";
    #load "link.jai";
    #load "h2.jai";
    #load "h2_server.jai";
    #load "h2_client.jai";
    #load "h2_exchange.jai";
    #load "server.jai";
    #load "client.jai";
    #load "test_client.jai";
    #load "expect.jai";
}

#scope_module

#load "scan.jai";

#import "Basic";
#import "String";
#import "Extensions/SIMD";

#if OS != .WASM {
    Io :: #import "Extensions/Io";
    Tls :: #import "Extensions/Tls";
    #import "Extensions/Json";
    #import "Compiler";
    #import "Atomics";
}
