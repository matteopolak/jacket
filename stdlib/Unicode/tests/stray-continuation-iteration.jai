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

// For valid text, reverse iteration gives the same characters as forward iteration, last first.
same_both_ways :: (bytes: [] u8) {
    ahead := forward(bytes);
    behind := backward(bytes);

    defer array_free(ahead);
    defer array_free(behind);

    assert(ahead.count == behind.count);

    for ahead assert(it == behind[ahead.count - 1 - it_index]);
}

invalid_reports: s64;

counting_logger :: (message: string, data: *void, info: Log_Info) {
    invalid_reports += 1;
}

main :: () {
    new_context := context;
    new_context.logger = counting_logger;

    push_context new_context {
        // A stray continuation byte is invalid: iteration yields what precedes it, reports, and stops.
        low := forward(u8.[0x41, 0x81, 0x42]);

        assert(low.count == 1 && low[0] == 0x41);
        assert(invalid_reports == 1);
        array_free(low);

        // Going backwards, stray bytes after the last character are seen first.
        tail := backward(u8.[0x61, 0x80]);

        assert(tail.count == 0);
        assert(invalid_reports == 2);
        array_free(tail);

        head := backward(u8.[0x80, 0x61, 0x62]);

        assert(head.count == 2 && head[0] == 0x62 && head[1] == 0x61);
        assert(invalid_reports == 3);
        array_free(head);

        mixed := backward(u8.[0xc3, 0xa9, 0x80]);

        assert(mixed.count == 0);
        assert(invalid_reports == 4);
        array_free(mixed);

        // Valid text is untouched and reports nothing.
        same_both_ways(u8.[0xe2, 0x82, 0xac, 0x61]);
        same_both_ways(u8.[0x61, 0xc3, 0xa9, 0x62]);
        assert(invalid_reports == 4);
    }

    print("ok\n");
}
