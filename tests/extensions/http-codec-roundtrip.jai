// Extensions/Http: a pipelined, chunked exchange written by the writers and read back by the
// parsers, one byte at a time and all at once; strictness and limits spot checks. The module's own
// tests (stdlib/Extensions/Http/tests) go much further.
#import "Basic";
#import "Extensions/Http";

#if OS == .WASM {
    // Existing C termination service in both the sandbox and WASI test hosts.
    exit :: (status: s32) #foreign;
}

failures := 0;

check :: (condition: bool, what: string) {
    if condition return;

    print("FAIL: %\n", what);
    failures += 1;
}

Seen :: struct {
    heads: int;
    ends: int;
    body: [..] u8;
    error: Parse_Error_Kind;
}

feed :: (parser: *Parser, seen: *Seen, input: string, piece: int) {
    position := 0;

    while position < input.count && seen.error == .NONE {
        n := piece;
        if n > input.count - position n = input.count - position;

        data: [] u8;
        data.count = n;
        data.data = input.data + position;
        position += n;

        while true {
            used, event := parse(parser, data);

            data = advance(data, used);

            if event.kind == .NEED_MORE break;

            if event.kind == .HEAD seen.heads += 1;
            if event.kind == .END seen.ends += 1;
            if event.kind == .BODY for event.data array_add(*seen.body, it);

            if event.kind == .ERROR {
                seen.error = parser.error.kind;

                break;
            }
        }
    }
}

main :: () {
    // A client pipelines three requests: a POST with a length, a chunked PUT, a bodyless GET.
    wire: [..] u8;

    write_request_head(*wire, "POST", "/items", .[.{ "Host", "api.test" }, .{ "Content-Type", "text/plain" }], 5);
    for "hello" array_add(*wire, it);

    write_request_head(*wire, "PUT", "/items/1", .[.{ "Host", "api.test" }], -1);
    write_chunk(*wire, "chunk one, ");
    write_chunk(*wire, "chunk two");
    write_last_chunk(*wire, .[.{ "Checksum", "abc" }]);

    write_request_head(*wire, "GET", "/items", .[.{ "Host", "api.test" }], 0);

    text: string;
    text.data = wire.data;
    text.count = wire.count;

    for piece: int.[1, 2, 3, 7, 64, 100000] {
        parser: Parser;

        parser_init(*parser, .REQUEST);
        seen: Seen;

        feed(*parser, *seen, text, piece);

        check(seen.error == .NONE, tprint("no error with pieces of %", piece));
        check(seen.heads == 3 && seen.ends == 3, tprint("three requests with pieces of %", piece));
        check(cast(string) .{ seen.body.count, seen.body.data } == "hellochunk one, chunk two", tprint("body with pieces of %", piece));
        check(parser.trailers.count == 0, "the last request has no trailers");

        parser_free(*parser);
        array_reset(*seen.body);
    }

    // A server answers each with a framed response; a client parser reads them back.
    answers: [..] u8;

    write_response_head(*answers, 201, .[.{ "Location", "/items/1" }], 2);
    for "ok" array_add(*answers, it);

    write_response_head(*answers, 200, .[], -1);
    write_chunk(*answers, "streamed");
    write_last_chunk(*answers);
    write_response_head(*answers, 204, .[], 0);

    answer_text: string;
    answer_text.data = answers.data;
    answer_text.count = answers.count;

    parser: Parser;

    parser_init(*parser, .RESPONSE);
    seen: Seen;

    feed(*parser, *seen, answer_text, 1);
    check(seen.error == .NONE && seen.heads == 3 && seen.ends == 3, "three responses");
    check(cast(string) .{ seen.body.count, seen.body.data } == "okstreamed", "response bodies");
    parser_free(*parser);

    // Strictness and limits.
    bad := string.[
        "GET / HTTP/1.1\r\nHost: a\r\nContent-Length: 1\r\nTransfer-Encoding: chunked\r\n\r\n",
        "GET / HTTP/1.1\r\nHost: a\r\nX: 1\r\n folded\r\n\r\n",
        "GET / HTTP/1.1\r\nHost: a\nX: 1\r\n\r\n",
        "GET / HTTP/1.1\r\nHost : a\r\n\r\n",
        "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 1\r\nContent-Length: 1\r\n\r\n",
    ];

    for bad {
        p: Parser;

        parser_init(*p, .REQUEST);
        s: Seen;

        feed(*p, *s, it, 1000);
        check(s.error != .NONE && s.heads == 0, tprint("refused: %", it));
        parser_free(*p);
    }

    limits: Limits;
    limits.max_body_size = 4;
    p: Parser;

    parser_init(*p, .REQUEST, limits);
    s: Seen;

    feed(*p, *s, "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 5\r\n\r\n", 1000);
    check(s.error == .BODY_TOO_LARGE && error_status(s.error) == 413, "a body over the limit is a 413");
    parser_free(*p);

    p2: Parser;

    parser_init(*p2, .REQUEST);
    s2: Seen;

    feed(*p2, *s2, "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\nffffffffffffffffff\r\n", 1000);
    check(s2.heads == 1 && s2.error == .CHUNK_SIZE_OVERFLOW, "a chunk size over 63 bits");
    parser_free(*p2);

    check(lookup_header_name("content-TYPE") == .CONTENT_TYPE, "header names");

    if failures == 0 print("ok\n");
    else exit(1);
}
