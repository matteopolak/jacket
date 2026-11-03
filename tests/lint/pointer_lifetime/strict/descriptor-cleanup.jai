#import "Basic";
P :: #import "Pool";

free_bytes :: (bytes: [] u8) {
    context.allocator.proc(.FREE, 0, 0, bytes.data, context.allocator.data);
}

reset_bytes :: (bytes: *[] u8) {
    free_bytes(bytes.*);
    bytes.* = .{};
}

bad_owned_return :: (bytes: [] u8) -> [] u8 {
    defer free_bytes(bytes);
    return bytes;
}

good_header_return :: (bytes: *[] u8) -> *[] u8 {
    defer reset_bytes(bytes);
    return bytes;
}

bad_loop :: () {
    bytes: [] u8;

    for 1..2 {
        defer reset_bytes(*bytes);
    }
}

good_pool :: () {
    pool: P.Pool;

    for 1..2 {
        defer P.reset(*pool);
    }
}

main :: () {}
