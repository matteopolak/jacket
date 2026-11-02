#scope_file
#import "Basic";
#import "String";
#import "Extensions/Testing";
Proto :: #import "Extensions/Protobuf";
Schema :: #import "Extensions/Protobuf/Schema";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { bytes data=1; M child=2; uint64 high=536870911; }");

#scope_export

oversized_encode :: () {
    value: Api.M;
    value.data = .{ 2147483647, null };
    bytes, error := Proto.encode(value);

    expect(error.kind == .TOO_LARGE && bytes.count == 0);
    out: [..] u8;

    defer array_free(out);
    array_add(*out, 42);
    error = Proto.encode_into(*out, value);
    expect(error.kind == .TOO_LARGE && out.count == 1 && out[0] == 42);
    parent := Api.M.{ child = *value };
    _, nested := Proto.encode(parent);

    expect(nested.kind == .TOO_LARGE);
} @test

constant_multibyte_tags :: () {
    value := Api.M.{ high = 1 };
    bytes, error := Proto.encode(value);

    require(error.kind == .NONE);
    defer free(bytes.data);
    expected := u8.[248, 255, 255, 255, 15, 1];

    expect(Proto.bytes_equal(bytes, expected));
    decoded, de := Proto.decode(Api.M, bytes);

    require(de.kind == .NONE);
    defer Proto.free(*decoded);
    expect(decoded.high == 1);
} @test

compact_generation :: () {
    source, error := Schema.protobuf_generate("syntax='proto3'; message M { map<string,int32> counts=1; repeated uint64 n=2; optional bool b=3; oneof x { int32 a=4; } }");

    require(!error.message.count);
    defer free(source);
    expect(!contains(source, "assert(n"));
    expect(!contains(source, "// Schema field"));
    expect(!contains(source, "proto_map_1_ :: struct"));
    expect(!contains(source, "for *item: value.n"));
} @test
