#import "Basic";
#import "String";
S :: #import "Extensions/Protobuf/Schema";

main :: () {
    text := "syntax='proto3'; package fixture; message Ping {} message Pong {} service Echo { option deprecated=true; rpc Unary(Ping) returns(Pong); rpc Bidi(stream Ping) returns(stream Pong) { option deprecated=true; } }";
    output, error := S.protobuf_generate(text, .{ grpc = true });

    assert(error.message.count == 0);
    assert(contains(output, "/fixture.Echo/Unary") && contains(output, "/fixture.Echo/Bidi"));
    assert(contains(output, "client_streaming = true") && contains(output, "server_streaming = true"));
    free(output);

    for invalid: string.[
        "message Ping {} service Echo { rpc Bad(int32) returns(Ping); }",
        "message Ping {} service Echo { rpc Bad(Missing) returns(Ping); }",
        "message Ping {} service Echo { rpc Same(Ping) returns(Ping); rpc Same(Ping) returns(Ping); }",
        "message Ping {} service Ping {}",
        "message Ping {} service Echo { rpc register(Ping) returns(Ping); }"
    ] {
        generated, failure := S.protobuf_generate(invalid, .{ grpc = true });
        assert(!generated.count && failure.message.count > 0 && failure.line > 0);
        free(failure.message);
    }

    late, late_error := S.protobuf_generate("service Echo { rpc Unary(M) returns(M); } package later; message M {}", .{ grpc = true });

    assert(late_error.message.count == 0 && contains(late, "/later.Echo/Unary"));
    free(late);
    plain, plain_error := S.protobuf_generate(text);

    assert(plain_error.message.count == 0 && !contains(plain, "Extensions/Grpc"));
    free(plain);
    print("ok\n");
}
