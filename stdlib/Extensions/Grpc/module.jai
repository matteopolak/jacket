// gRPC identity message framing, metadata and canonical status codecs, plus
// native TLS HTTP/2 client/server calls with typed Protobuf message helpers.

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
Proto :: #import "Extensions/Protobuf";

#if OS != .WASM {
    Io :: #import "Extensions/Io";
}
