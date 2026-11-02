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

packed_fixed_bulk_bounds_and_append :: () {
    Fixed :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { repeated fixed32 a=1; repeated sfixed64 b=2; repeated float c=3; repeated double d=4; }");
    // Packed and expanded occurrences must append in order.
    bytes := u8.[10, 8, 1, 0, 0, 0, 255, 255, 255, 255, 13, 2, 0, 0, 0, 18, 8, 255, 255, 255, 255, 255, 255, 255, 255, 26, 4, 0, 0, 128, 63, 34, 8, 0, 0, 0, 0, 0, 0, 240, 63];
    m, error := Proto.decode(Fixed.M, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*m);
    expect(m.a.count == 3 && m.a[0] == 1 && m.a[1] == 0xffffffff && m.a[2] == 2);
    expect(m.b.count == 1 && m.b[0] == -1);
    expect(m.c.count == 1 && m.c[0] == 1 && m.d.count == 1 && m.d[0] == 1);
    encoded, ee := Proto.encode(m);

    require(ee.kind == .NONE);
    defer free(encoded.data);
    back, be := Proto.decode(Fixed.M, encoded);

    require(be.kind == .NONE);
    defer Proto.free(*back);
    expect(Proto.equal(m, back));

    for width: 4..8 {
        if width != 4 && width != 8 continue;

        for count: 1..width - 1 {
            bad: [10] u8;
            bad[0] = ifx width == 4 then 10 else 18;
            bad[1] = cast(u8) count;
            _, de := Proto.decode(Fixed.M, .{ count + 2, bad.data });

            expect(de.kind == .TRUNCATED);
            Proto.free(*de);
        }
    }
} @test

wrong_wire_oneof_does_not_switch :: () {
    Choice :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { oneof x { string s=1; int32 n=2; } }");
    bytes := u8.[10, 1, 120, 18, 1, 0]; // field 2 has the wrong wire type
    m, error := Proto.decode(Choice.M, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*m);
    expect(m.x.which == .S && m.x.s == "x" && m._unknown.count == 3);
} @test

implicit_negative_zero_is_present :: () {
    Floating :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { float f=1; double d=2; }");
    bytes := u8.[13, 0, 0, 0, 128, 17, 0, 0, 0, 0, 0, 0, 0, 128];
    m, error := Proto.decode(Floating.M, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*m);
    encoded, ee := Proto.encode(m);

    require(ee.kind == .NONE);
    defer free(encoded.data);
    expect(Proto.bytes_equal(encoded, bytes));
    positive: Floating.M;

    expect(Proto.size(positive) == 0);
} @test
