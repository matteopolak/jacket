#import "Basic";
Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message Ping { uint64 id=1; string note=2; }");

main :: () {
    p := Api.Ping.{ id = 7, note = "Ada" };
    bytes := Proto.encode(p);

    defer free(bytes.data);
    assert(bytes.count == 7);
    q, error := Proto.decode(Api.Ping, bytes);

    assert(error.kind == .NONE);
    assert(Proto.equal(p, q));
    Proto.free(*q);
    Proto.free(*q);
}
