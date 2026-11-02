// Borrow follows Json: use an arena whose free is a no-op and reset the arena instead.
#scope_file
#import "Basic";
#import "Extensions/Testing";
Proto :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { string text=1; bytes data=2; }");

#scope_export

borrowed_payloads :: () {
    bytes := u8.[10, 2, 104, 105, 18, 1, 255];

    push_allocator(temp);
    value, error := Proto.decode(Api.M, bytes, .{ borrow = true });

    require(error.kind == .NONE);
    expect(value.text.data == bytes.data + 2 && value.text == "hi");
    expect(value.data.data == bytes.data + 6 && value.data[0] == 255);
    // Failure cleanup also passes borrowed slices to this no-op free allocator.
    broken := u8.[10, 2, 104, 105, 18, 2, 255];
    empty, failure := Proto.decode(Api.M, broken, .{ borrow = true });

    expect(failure.kind == .TRUNCATED && empty.text.count == 0);
} @test
