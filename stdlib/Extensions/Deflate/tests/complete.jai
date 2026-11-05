// Complete protocol payloads reject trailing bytes and bound all concatenated gzip members.
#load "support/helpers.jai";

main :: () {
    for container: Deflate_Container.[.RAW, .ZLIB, .GZIP] {
        packed := deflate(bytes_of("first"), container);

        defer free(packed.data);
        output, ok := inflate_complete(packed, container, 5);

        check(ok && same_bytes(output, bytes_of("first")), "one complete stream");
        if output.data free(output.data);

        extra := NewArray(packed.count + 1, u8);

        defer free(extra.data);
        memcpy(extra.data, packed.data, packed.count);
        extra[packed.count] = 0x7f;
        output, ok = inflate_complete(extra, container, 100);
        check(!ok && output.count == 0 && !output.data, "trailing garbage rejected atomically");
    }

    first := deflate(bytes_of("first"), .GZIP);
    second := deflate(bytes_of("second"), .GZIP);
    empty := deflate(.[], .GZIP);

    defer free(first.data);
    defer free(second.data);
    defer free(empty.data);
    joined := NewArray(first.count + empty.count + second.count, u8);

    defer free(joined.data);
    memcpy(joined.data, first.data, first.count);
    memcpy(joined.data + first.count, empty.data, empty.count);
    memcpy(joined.data + first.count + empty.count, second.data, second.count);
    output, ok := inflate_complete(joined, .GZIP, 11);

    check(ok && same_bytes(output, bytes_of("firstsecond")), "all gzip members and empty member decoded");
    if output.data free(output.data);

    output, ok = inflate_complete(joined, .GZIP, 10);
    check(!ok && !output.data && output.count == 0, "aggregate decompressed limit");

    joined[joined.count - 8] ^= 1;
    output, ok = inflate_complete(joined, .GZIP, 100);
    check(!ok && !output.data, "later member CRC checked before returning any payload");
    output, ok = inflate_complete(.[], .GZIP, 100);
    check(!ok && !output.data, "empty input is not a gzip stream");
    output, ok = inflate_complete(first, .GZIP, -1);
    check(!ok && !output.data, "negative output bound rejected");
    assert(failures == 0);
    print("ok\n");
}
