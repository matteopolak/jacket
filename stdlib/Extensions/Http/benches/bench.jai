#scope_file
// Run with jacket bench --path stdlib/Extensions/Http.
#import "Extensions/Testing";
#import "Basic";
#import "Extensions/Http";

BROWSER :: "GET /static/js/app.4f3a9c.js?v=20260112&lang=en-US HTTP/1.1\r\nHost: www.example.com\r\nUser-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8\r\nAccept-Language: en-US,en;q=0.9\r\nAccept-Encoding: gzip, deflate, br\r\nConnection: keep-alive\r\nCache-Control: max-age=0\r\nReferer: https://www.example.com/index.html\r\nCookie: session=3f9a1c0be2d44f0aa6d1; theme=dark; consent=1\r\n\r\n";

make_stream :: (unit: string, copies: int) -> string {
    builder: String_Builder;
    defer free_buffers(*builder);

    for 1..copies append(*builder, unit);

    return builder_to_string(*builder);
}

make_large_headers :: (copies: int) -> string {
    cookie: String_Builder;

    defer free_buffers(*cookie);

    for 0..40 print_to_builder(*cookie, "k%=%; ", it, "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz");

    unit := tprint("GET /a/very/long/path/%?q=% HTTP/1.1\r\nHost: example.com\r\nCookie: %\r\nX-Forwarded-For: 203.0.113.7, 198.51.100.23, 192.0.2.99\r\nAuthorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk5yJV_adQssw5c\r\n\r\n", "segment/another/segment/and/another/one/more", "search+terms+that+go+on+and+on+and+on", builder_to_string(*cookie,, temp));

    return make_stream(unit, copies);
}

make_chunked :: (chunks: int, size: int) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    payload: [..] u8;

    defer array_reset(*payload);

    for 0..size - 1 array_add(*payload, cast(u8) (#char "a" + it % 26));

    append(*builder, "POST /upload HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n");

    for 1..chunks {
        print_to_builder(*builder, "%\r\n", FormatInt.{ value = size, base = 16 });
        append(*builder, as_string(payload.data, size));
        append(*builder, "\r\n");
    }

    append(*builder, "0\r\n\r\n");

    return builder_to_string(*builder);
}

as_string :: (data: *u8, count: int) -> string {
    result: string;
    result.data = data;
    result.count = count;

    return result;
}

// Parses the stream `repeat` times in reads of 4 KiB; returns the messages ended.
run :: (stream: string, repeat: int, require_host: bool) -> int {
    parser: Parser;
    limits: Limits;
    limits.require_host = require_host;
    limits.max_body_size = -1;
    parser_init(*parser, .REQUEST, limits);
    defer parser_free(*parser);

    ends := 0;
    keep: u64;

    for 1..repeat {
        position := 0;

        while position < stream.count {
            n := 4096;
            if n > stream.count - position n = stream.count - position;

            data: [] u8;
            data.count = n;
            data.data = stream.data + position;
            position += n;

            while true {
                used, event := parse(*parser, data);

                data = advance(data, used);

                if event.kind == .NEED_MORE break;

                if event.kind == .END ends += 1;
                if event.kind == .HEAD keep += cast(u64) parser.head.headers.count;
                if event.kind == .BODY keep += cast(u64) event.data.count;

                if event.kind == .ERROR {
                    print("error: %\n", parser.error);

                    return -1;
                }
            }
        }
    }

    return ends;
}

#scope_export
browser_scalar :: (b: *Bench) {
    stream := make_stream(BROWSER, 200);

    defer free(stream);
    http_use_simd = false;
    defer http_use_simd = true;
    set_bytes(b, stream.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        ends := run(black_box(stream), 1, true);
        assert(ends >= 0);
        black_box(ends);
    }
} @bench

browser_vector :: (b: *Bench) {
    stream := make_stream(BROWSER, 200);

    defer free(stream);
    http_use_simd = true;
    defer http_use_simd = true;
    set_bytes(b, stream.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        ends := run(black_box(stream), 1, true);
        assert(ends >= 0);
        black_box(ends);
    }
} @bench

headers_scalar :: (b: *Bench) {
    stream := make_large_headers(20);

    defer free(stream);
    http_use_simd = false;
    defer http_use_simd = true;
    set_bytes(b, stream.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        ends := run(black_box(stream), 1, true);
        assert(ends >= 0);
        black_box(ends);
    }
} @bench

headers_vector :: (b: *Bench) {
    stream := make_large_headers(20);

    defer free(stream);
    http_use_simd = true;
    defer http_use_simd = true;
    set_bytes(b, stream.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        ends := run(black_box(stream), 1, true);
        assert(ends >= 0);
        black_box(ends);
    }
} @bench

chunked_64k_scalar :: (b: *Bench) {
    stream := make_chunked(256, 65536);

    defer free(stream);
    http_use_simd = false;
    defer http_use_simd = true;
    set_bytes(b, stream.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        ends := run(black_box(stream), 1, true);
        assert(ends >= 0);
        black_box(ends);
    }
} @bench

chunked_64k_vector :: (b: *Bench) {
    stream := make_chunked(256, 65536);

    defer free(stream);
    http_use_simd = true;
    defer http_use_simd = true;
    set_bytes(b, stream.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        ends := run(black_box(stream), 1, true);
        assert(ends >= 0);
        black_box(ends);
    }
} @bench

chunked_100_scalar :: (b: *Bench) {
    stream := make_chunked(40000, 100);

    defer free(stream);
    http_use_simd = false;
    defer http_use_simd = true;
    set_bytes(b, stream.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        ends := run(black_box(stream), 1, true);
        assert(ends >= 0);
        black_box(ends);
    }
} @bench

chunked_100_vector :: (b: *Bench) {
    stream := make_chunked(40000, 100);

    defer free(stream);
    http_use_simd = true;
    defer http_use_simd = true;
    set_bytes(b, stream.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        ends := run(black_box(stream), 1, true);
        assert(ends >= 0);
        black_box(ends);
    }
} @bench
