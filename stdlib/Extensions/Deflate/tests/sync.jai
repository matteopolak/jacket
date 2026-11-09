#import "Basic";
#import "Extensions/Deflate";

main :: () {
    input := cast([] u8) "hello hello hello";
    packed := deflate_sync(input);

    defer free(packed.data);
    plain, ok := inflate_sync(packed, .[], 1024);

    assert(ok && cast(string) plain == cast(string) input);
    free(plain.data);
    _, ok = inflate_sync(packed, .[], 2);
    assert(!ok);
    // zlib Z_SYNC_FLUSH, second message depends on the first message's dictionary.
    plain, ok = inflate_sync(u8.[0xCA, 0x40, 0x17, 0, 0], input, 1024);
    assert(ok && cast(string) plain == "hello hello hello");
    free(plain.data);
    data := NewArray(2048, u8);

    defer free(data.data);
    for i: 0..1023 data[i] = cast(u8) ((i * 71 + i / 256) % 256);

    memcpy(data.data + 1024, data.data, 1024);
    seeded := deflate_sync(input, 6, input);

    defer free(seeded.data);
    assert(seeded.count < packed.count);
    plain, ok = inflate_sync(seeded, input, 1024);
    assert(ok && cast(string) plain == cast(string) input);
    free(plain.data);
    large := deflate_sync(data);

    defer free(large.data);
    _, valid := inflate_sync(large, .[], 2048, 8);

    assert(!valid);
}
