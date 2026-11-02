#scope_file
#import "Basic";
#import "String";
#import "Extensions/Testing";
Schema :: #import "Extensions/Protobuf/Schema";
Proto :: #import "Extensions/Protobuf";
Resolved :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; package a.b; message Outer { message Inner { Outer parent=1; } Inner child=1; .a.b.Outer absolute=2; a.b.Outer qualified=3; optional bytes payload=4; } enum Top { ZERO=0; ONE=1; }", MAP = .ARRAY);

Collisions :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message Node { Node next=1; } message Field { Node value=1; } message Map { map<string,int32> values=1; } message Variant { oneof kind { int32 n=1; string s=2; int32 type=3; int32 which=4; int32 none=5; int32 proto_x=6; int32 payload=7; } } message Reader { Field value=1; }");

#scope_export

schema_api_and_names :: () {
    source :: #string END
// lexer comments and quoted escapes
syntax = 'proto3';
package hello.world;
option (custom.option) = { name: 'value' nested: { x: 1 } };
message Thing {
  /* nested comment-free body */
  optional string type = 0x1 [json_name='x', (custom.option)=true];
  repeated fixed32 values = 02 [packed=false];
  reserved 3 to 5, 8 to max;
  reserved 'gone';
  enum Status { option allow_alias=true; STATUS_NONE=0; STATUS_OK=1; STATUS_ALIAS=1; }
}
service Service { rpc Call(Thing) returns (Thing); }
END;
    output, error := Schema.protobuf_generate(source, .{ strip_enum_prefix = true });

    require(error.message.count == 0);
    defer free(output);
    expect(contains(output, "hello :: struct"));
    expect(contains(output, "world :: struct"));
    expect(contains(output, "type_: proto_r.Optional(string)"));
    expect(contains(output, "json(rename="));
    expect(contains(output, "OK :: 1"));
    item: Resolved.a.b.Outer;

    expect(item.child == null && item.absolute == null && item.qualified == null && !item.payload.has);
    _, invalid := Schema.protobuf_generate("syntax='proto3';\nmessage M {\n int32 bad=0;\n}", .{ path = "example.proto" });

    defer free(invalid.message);
    expect_equal(invalid.line, 3);
    expect_equal(invalid.column, 2);
    expect_equal(invalid.path, "example.proto");
} @test

schema_parser_failures :: () {
    inputs := string.[
        "syntax='proto3'; message M { string a=1 [json_name='unterminated]; }",
        "syntax='proto3'; /* unterminated",
        "syntax='proto3'; message M {",
        "syntax='proto3'; enum E { A=0; B=0; }",
        "syntax='proto3'; message M { string a=1 [packed=true]; }",
        "syntax='proto3'; message M { int32 type=1; int32 type_=2; }",
        "syntax='proto3'; message M { reserved 5 to 2; }",
        "syntax='proto3'; message M { oneof o { string x=1; int32 X=2; } }",
        "syntax='proto3'; message M { int32 x=999999999999999999999999999; }"
    ];
    for inputs {
        output, error := Schema.protobuf_generate(it);
        expect_equal(output.count, 0);
        expect(error.message.count > 0);
        free(error.message);
    }
} @test

namespace_collisions :: () {
    value: Collisions.Node;
    bytes := Proto.encode(value);

    defer free(bytes.data);
    decoded, error := Proto.decode(Collisions.Node, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*decoded);
    map: Collisions.Map;
    mbytes := Proto.encode(map);

    defer free(mbytes.data);
    m, me := Proto.decode(Collisions.Map, mbytes);

    require(me.kind == .NONE);
    defer Proto.free(*m);
    expect(Proto.equal(map, m));
    choice: Collisions.Variant;

    expect(choice.kind.which == .NONE);
    bytes_reserved := u8.[24, 1, 32, 2, 40, 3, 48, 4, 56, 5];
    decoded_choice, ce := Proto.decode(Collisions.Variant, bytes_reserved);

    require(ce.kind == .NONE);
    defer Proto.free(*decoded_choice);
    expect(decoded_choice.kind.which == .PAYLOAD && decoded_choice.kind.payload_ == 5);
    choice.kind.which = .TYPE;
    choice.kind.type_ = 1;
    encoded_choice := Proto.encode(choice);

    defer free(encoded_choice.data);
    expect(encoded_choice.count == 2 && encoded_choice[0] == 24);
} @test
