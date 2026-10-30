// Http: HTTP/1.0 and HTTP/1.1 message codecs, plus a server, a client and test tools on top of them for fibers. A jaic
// extension, not official Jai: other Jai compilers have no such module, and it may change between jaic releases. See
// docs/extensions/http-codec.md, http-server.md and http-client.md.
//
// The codec has no IO of its own. A `Parser` reads requests or responses from byte buffers the caller fills, in pieces of
// any size, and reports the head, the body in pieces and the end of each message with strict RFC 9112 framing; the writers
// build heads and chunked bodies. Nothing in it touches a socket, so it runs on every target, wasm included. The server
// (`App`, `serve`) and the client (`Client`, `get`, `send`) need fibers and `Extensions/Io`, and are not built for wasm.
// Failures are values: the parser's `.ERROR` event and `Parser.error`, an `Http_Error` returned by a handler, a
// `Client_Error` returned by a client call. Memory the parser or a response owns is released with `parser_free` and
// `close`; everything else is borrowed or in the temporary allocator, as each procedure says.
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
#load "parser.jai";
#load "chunked.jai";
#load "writer.jai";
#load "date.jai";

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
