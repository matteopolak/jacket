// Http: HTTP/1.0 and HTTP/1.1 message codecs with no IO of their own. A jaic extension, not
// official Jai: other Jai compilers have no such module, and it may change between jaic
// releases. See docs/stdlib/http-codec.md.
//
//     #import "Extensions/Http";
//
//     parser: Parser;
//     parser_init(*parser, .REQUEST);
//
//     used, event := parse(*parser, bytes);     // any chunking: the result is the same
//     if event.kind == .HEAD print("% %\n", parser.head.method_text, parser.head.target);
//
// The parser reads requests or responses from byte buffers the caller fills, in pieces of any
// size, and reports the head, the body in pieces and the end of each message, with strict
// RFC 9112 framing. The writers build heads and chunked bodies. Nothing here touches a socket,
// so the same code runs on every target, wasm included.

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
    #load "url.jai";
    #load "message.jai";
    #load "router.jai";
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
    #import "Extensions/Json";
    #import "Compiler";
    #import "Atomics";
}
