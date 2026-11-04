#import "Basic";
G :: #import "Extensions/Grpc";
H :: #import "Extensions/Http";

main :: () {
    for key: string.["", "Upper", "grpc-custom", "content-type", "te", "host", "content-length", "bad name", ":path"] assert(!G.valid_metadata_name(key));
    for key: string.["x-key", "hello_1.2", "data-bin"] assert(G.valid_metadata_name(key));

    fields, ok := G.encode_metadata(G.Metadata_Entry.[.{ "x-key", "one" }, .{ "x-key", "two" }, .{ "data-bin", "\x00\xff" }]);

    assert(ok && fields.count == 3 && fields[2].value == "AP8");
    decoded, valid := G.decode_metadata(H.Header_Field.[.{ name = "data-bin", value = "YQ==,Yg" }]);

    assert(valid && decoded.count == 2 && decoded[0].value == "a" && decoded[1].value == "b");
    G.free_metadata(*decoded);
    _, valid = G.decode_metadata(H.Header_Field.[.{ name = "data-bin", value = "!" }]);
    assert(!valid);
    print("ok\n");
}
