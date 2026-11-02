#scope_file
#import "Basic";
#import "Extensions/Testing";
Proto :: #import "Extensions/Protobuf";
Custom :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message Bag { map<string,string> items=1; }", MAP = .CUSTOM, MAP_MODULE = "Extensions/Protobuf/tests/fixtures/custom-map", MAP_TYPE = "Dict", MAP_INSERT = "put", MAP_DEINIT = "destroy");
#import "Extensions/Hash_Map";

#scope_export

custom_maps :: () {
    bytes := u8.[10, 6, 10, 1, 97, 18, 1, 98, 10, 6, 10, 1, 97, 18, 1, 99];
    value, error := Proto.decode(Custom.Bag, bytes);

    require(error.kind == .NONE);
    defer Proto.free(*value);
    expect_equal(value.items.count, 1);
    expect_equal(get(*value.items, "a"), "c");
    encoded := Proto.encode(value);

    defer free(encoded.data);
    again, ae := Proto.decode(Custom.Bag, encoded);

    require(ae.kind == .NONE);
    defer Proto.free(*again);
    expect(Proto.equal(value, again));
} @test
