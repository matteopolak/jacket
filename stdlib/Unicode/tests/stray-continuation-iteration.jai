#import "Basic";
U :: #import "Unicode";

forward :: (bytes: [] u8) -> [..] u32 {
    result: [..] u32;
    text := string.{ bytes.count, bytes.data };

    for: U.utf8_iter text array_add(*result, it);

    return result;
}

backward :: (bytes: [] u8) -> [..] u32 {
    result: [..] u32;
    text := string.{ bytes.count, bytes.data };

    for <: U.utf8_iter text array_add(*result, it);

    return result;
}

// Reverse iteration gives the same characters as forward iteration, last first.
same_both_ways :: (bytes: [] u8) {
    ahead := forward(bytes);
    behind := backward(bytes);

    defer array_free(ahead);
    defer array_free(behind);

    assert(ahead.count == behind.count);

    for ahead assert(it == behind[ahead.count - 1 - it_index]);
}

main :: () {
    // A stray continuation byte decodes to its low seven bits.
    low := forward(u8.[0x41, 0x81]);

    assert(low.count == 2 && low[0] == 0x41 && low[1] == 1);
    array_free(low);

    tail := backward(u8.[0x61, 0x80]);

    assert(tail.count == 2 && tail[0] == 0 && tail[1] == 0x61);
    array_free(tail);

    head := backward(u8.[0x80, 0x61, 0x62]);

    assert(head.count == 3 && head[0] == 0x62 && head[1] == 0x61 && head[2] == 0);
    array_free(head);

    mixed := backward(u8.[0xc3, 0xa9, 0x80]);

    assert(mixed.count == 2 && mixed[0] == 0 && mixed[1] == 0xe9);
    array_free(mixed);

    same_both_ways(u8.[0x80, 0xbf, 0x41]);
    same_both_ways(u8.[0xe2, 0x82, 0xac, 0x90, 0x91]);
    same_both_ways(u8.[0x61, 0xc3, 0xa9, 0x62]);
    print("ok\n");
}
