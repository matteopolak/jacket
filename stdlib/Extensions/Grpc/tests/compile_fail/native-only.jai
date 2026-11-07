// test: only(.WASM) compile_fail("generated gRPC services require native Io")
Api :: #import "Extensions/Protobuf/Schema"(GRPC = true, TEXT = "syntax='proto3'; message M {} service S { rpc Unary(M) returns(M); }");

main :: () {}
