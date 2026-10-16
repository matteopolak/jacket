// The parser, the writers and the date helpers, case by case. Every message is also fed one byte
// at a time and, through observe_every_cut, split at every position; the vector and scalar scan
// paths must agree.
//
//     jaic stdlib/Extensions/Http/tests/codec.jai -interp

#load "lib/support.jai";

request_run :: () -> Run {
    return default_run(.REQUEST);
}

response_run :: () -> Run {
    return default_run(.RESPONSE);
}

test_request_heads :: () {
    d := make_driver(.REQUEST);

    defer free_driver(d);

    drive(d, "GET /index.html?a=b HTTP/1.1\r\nHost: example.com\r\nUser-Agent: t\r\nX-Custom:   padded value \t\r\nEmpty:\r\n\r\n");

    p := *d.parser;

    check(d.last == .NEED_MORE && d.heads == 1 && d.ends == 1, "one complete request");
    check(p.head.method == .GET && p.head.method_text == "GET", "method");
    check(p.head.target == "/index.html?a=b" && p.head.target_form == .ORIGIN, "target");
    check(p.head.version == .HTTP_1_1 && p.head.keep_alive, "1.1 keeps alive");
    check(p.head.body == .NONE, "a GET has no body");
    check(p.head.headers.count == 4, "four headers");
    check(p.head.headers[2].name == "X-Custom" && p.head.headers[2].value == "padded value", "value is trimmed");
    check(p.head.headers[3].name == "Empty" && p.head.headers[3].value == "", "empty value");
    check(p.head.headers[0].id == .HOST && p.head.headers[1].id == .USER_AGENT && p.head.headers[2].id == .UNKNOWN, "header ids");

    value, found := find_header(p.head.headers, "host");

    check(found && value == "example.com", "find_header ignores case");
    value, found = find_header(p.head.headers, .USER_AGENT);
    check(found && value == "t", "find_header by id");
    value, found = find_header(p.head.headers, "x-custom");
    check(found && value == "padded value", "find_header for an unknown name");
    value, found = find_header(p.head.headers, "missing");
    check(!found, "a missing header");

    // Methods and target forms.
    check(parse_method("PATCH") == .PATCH && parse_method("get") == .OTHER && parse_method("M-SEARCH") == .OTHER, "parse_method");

    d2 := make_driver(.REQUEST);

    defer free_driver(d2);
    drive(d2, "OPTIONS * HTTP/1.1\r\nHost: a\r\n\r\nCONNECT example.com:443 HTTP/1.1\r\nHost: example.com:443\r\n\r\nGET http://example.com/x HTTP/1.1\r\nHost: example.com\r\n\r\n");
    check(d2.ends == 3 && d2.parser.head.target_form == .ABSOLUTE, "asterisk, authority and absolute forms");

    head_text := observe(request_run(), "PURGE /cache HTTP/1.0\r\n\r\n", .{});

    check(head_text == "M\x01PURGE\x01/cache\x011.0\nD\nC\n", tprint("an unknown method is kept: %", head_text));
}

test_keep_alive :: () {
    Case :: struct {
        text: string;
        keep: bool;
    }

    cases := Case.[
        .{ "GET / HTTP/1.1\r\nHost: a\r\n\r\n", true },
        .{ "GET / HTTP/1.1\r\nHost: a\r\nConnection: close\r\n\r\n", false },
        .{ "GET / HTTP/1.1\r\nHost: a\r\nConnection: Keep-Alive\r\n\r\n", true },
        .{ "GET / HTTP/1.0\r\n\r\n", false },
        .{ "GET / HTTP/1.0\r\nConnection: keep-alive\r\n\r\n", true },
        .{ "GET / HTTP/1.0\r\nConnection: keep-alive, close\r\n\r\n", false },
        .{ "GET / HTTP/1.1\r\nHost: a\r\nConnection: TE, Close\r\n\r\n", false },
    ];

    for cases {
        d := make_driver(.REQUEST);
        defer free_driver(d);

        drive(d, it.text);
        check(d.heads == 1 && d.parser.head.keep_alive == it.keep, tprint("keep-alive for %", it.text));
    }

    // After a message that ends the connection nothing more is accepted.
    d := make_driver(.REQUEST);

    defer free_driver(d);
    drive(d, "GET / HTTP/1.1\r\nHost: a\r\nConnection: close\r\n\r\nGET / HTTP/1.1\r\n");
    check(d.ends == 1 && d.last == .ERROR && d.parser.error.kind == .DATA_AFTER_CLOSE, "data after Connection: close");
    check(error_status(d.parser.error.kind) == 400, "a 400");

    // An Upgrade request is flagged, and parsing carries on.
    e := make_driver(.REQUEST);

    defer free_driver(e);
    drive(e, "GET /chat HTTP/1.1\r\nHost: a\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\r\n");
    check(e.parser.head.upgrade && e.parser.head.keep_alive, "an upgrade request");

    f := make_driver(.REQUEST);

    defer free_driver(f);
    drive(f, "GET /chat HTTP/1.1\r\nHost: a\r\nUpgrade: websocket\r\n\r\n");
    check(!f.parser.head.upgrade, "Upgrade without Connection: upgrade is no upgrade");
}

test_request_bodies :: () {
    d := make_driver(.REQUEST);

    defer free_driver(d);
    drive(d, "POST /a HTTP/1.1\r\nHost: a\r\nContent-Length: 11\r\n\r\nhello world");
    check(d.ends == 1 && body_text(d) == "hello world" && d.parser.head.body == .LENGTH && d.parser.head.content_length == 11, "content-length body");

    e := make_driver(.REQUEST);

    defer free_driver(e);
    drive(e, "POST /a HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5;name=value;q=\"a b\"\r\nhello\r\nA\r\n 0123456 9\r\n0\r\nExpires: soon\r\nX-Sum: 1\r\n\r\n");
    check(e.ends == 1 && body_text(e) == "hello 0123456 9" && e.parser.head.body == .CHUNKED, "chunked body");
    check(e.parser.trailers.count == 2 && e.parser.trailers[0].name == "Expires" && e.parser.trailers[1].value == "1", "trailers");

    // The same chunks, cut everywhere.
    filler: [..] u8;

    defer array_reset(*filler);
    for 0..254 array_add(*filler, cast(u8) (#char "a" + it % 26));

    chunked_text := tprint("PUT /a HTTP/1.1\r\nTransfer-Encoding: Chunked\r\n\r\n1\r\na\r\nFf\r\n%\r\n0\r\n\r\n", text_of(filler));
    whole := observe_every_cut(request_run(), chunked_text, "chunks");

    check(whole.count > 0 && whole[whole.count - 2] == #char "C", "chunked request completes");

    // A body that is shorter than it says is not an end.
    f := make_driver(.REQUEST);

    defer free_driver(f);
    drive(f, "POST /a HTTP/1.1\r\nHost: a\r\nContent-Length: 10\r\n\r\nhello");
    check(f.ends == 0 && f.last == .NEED_MORE, "an unfinished body");
    event := parse_eof(*f.parser);

    check(event.kind == .ERROR && f.parser.error.kind == .UNEXPECTED_EOF, "end of input inside a body");

    // Content-Length: 0 is a message without body.
    g := make_driver(.REQUEST);

    defer free_driver(g);
    drive(g, "POST /a HTTP/1.1\r\nHost: a\r\nContent-Length: 0\r\n\r\n");
    check(g.ends == 1 && g.parser.head.body == .NONE && g.parser.head.content_length == 0, "zero length");
}

test_expect :: () {
    d := make_driver(.REQUEST);

    defer free_driver(d);
    drive(d, "POST /a HTTP/1.1\r\nHost: a\r\nContent-Length: 5\r\nExpect: 100-Continue\r\n\r\n");
    check(d.heads == 1 && d.parser.head.expect_continue && d.ends == 0, "the head comes before the body, with the flag");

    out: [..] u8;

    defer array_reset(*out);
    write_continue(*out);
    check(text_of(out) == "HTTP/1.1 100 Continue\r\n\r\n", "the interim response");

    drive(d, "hello");
    check(d.ends == 1 && body_text(d) == "hello", "then the body");

    check(error_kind_of(.REQUEST, "POST /a HTTP/1.1\r\nHost: a\r\nExpect: 200-ok\r\n\r\n") == .BAD_EXPECT, "another expectation");
    check(error_status(.BAD_EXPECT) == 417, "417");

    e := make_driver(.REQUEST);

    defer free_driver(e);
    drive(e, "GET /a HTTP/1.1\r\nHost: a\r\nExpect: 100-continue\r\n\r\n");
    check(!e.parser.head.expect_continue, "nothing to wait for without a body");

    f := make_driver(.REQUEST);

    defer free_driver(f);
    drive(f, "POST /a HTTP/1.0\r\nContent-Length: 1\r\nExpect: 100-continue\r\n\r\n");
    check(!f.parser.head.expect_continue, "HTTP/1.0 has no 100-continue");
}

test_pipelining :: () {
    text := "GET /1 HTTP/1.1\r\nHost: a\r\n\r\nPOST /2 HTTP/1.1\r\nHost: a\r\nContent-Length: 3\r\n\r\nabcGET /3 HTTP/1.1\r\nHost: a\r\n\r\n\r\nPOST /4 HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n";
    whole := observe_every_cut(request_run(), text, "pipelined requests");

    expected := "M\x01GET\x01/1\x011.1\nH\x01Host\x01a\nD\nC\nM\x01POST\x01/2\x011.1\nH\x01Host\x01a\nH\x01Content-Length\x013\nD\nB\x01616263\nC\nM\x01GET\x01/3\x011.1\nH\x01Host\x01a\nD\nC\nM\x01POST\x01/4\x011.1\nH\x01Host\x01a\nH\x01Transfer-Encoding\x01chunked\nD\nC\n";
    check(whole == expected, tprint("pipelined requests: %", whole));
}

// A parser that saw a huge head gives its storage back between messages and goes on working; the
// trim leaves alone what is under its limit and a message that has begun.
test_trim :: () {
    d := make_driver(.REQUEST);

    defer free_driver(d);

    builder: String_Builder;

    append(*builder, "GET /big HTTP/1.1\r\nHost: a\r\n");

    for 1..60 print(*builder, "X-H%: %\r\n", it, "0123456789012345678901234567890123456789012345678901234567890123456789");

    append(*builder, "\r\n");

    drive(d, builder_to_string(*builder,, temp));

    p := *d.parser;

    check(d.ends == 1 && p.head.headers.count == 61, "the big request");
    check(p.bytes.allocated > 4096, "its head storage grew");

    parser_trim(p, 1 << 20);
    check(p.bytes.allocated > 4096, "a limit above the size frees nothing");

    parser_trim(p, 4096);
    check(p.bytes.allocated == 0, "a limit below the size frees the head storage");

    drive(d, "GET /small HTTP/1.1\r\nHost: a\r\n\r\n");

    check(d.ends == 2 && p.head.target == "/small" && p.head.headers.count == 1, "the next request parses after a trim");

    // Half a request: nothing is freed under it.
    drive(d, "GET /half HTTP/1.1\r\nHost: b\r\n");

    used := p.bytes.allocated;

    parser_trim(p, 1);
    check(p.bytes.allocated == used && p.bytes.count > 0, "a message in progress is left alone");

    drive(d, "\r\n");
    check(d.ends == 3 && p.head.target == "/half", "and it completes");
}

test_responses :: () {
    d := make_driver(.RESPONSE);

    defer free_driver(d);
    drive(d, "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nContent-Type: text/plain\r\n\r\nhello");
    check(d.ends == 1 && body_text(d) == "hello" && d.parser.head.status == 200 && d.parser.head.reason == "OK", "a response with a length");

    // Read until close.
    e := make_driver(.RESPONSE);

    defer free_driver(e);
    drive(e, "HTTP/1.0 200 OK\r\nServer: old\r\n\r\npart one, ");
    drive(e, "part two");
    check(e.ends == 0 && e.parser.head.body == .UNTIL_CLOSE && !e.parser.head.keep_alive, "a body until close");
    event := parse_eof(*e.parser);

    check(event.kind == .END && body_text(e) == "part one, part two", "closing ends it");
    check(parse_eof(*e.parser).kind == .CLOSED, "and then there is nothing");

    // Responses without a body: 1xx, 204, 304, and to a HEAD request.
    f := make_driver(.RESPONSE);

    defer free_driver(f);
    drive(f, "HTTP/1.1 204 No Content\r\n\r\nHTTP/1.1 304 Not Modified\r\nContent-Length: 1234\r\nETag: x\r\n\r\nHTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
    check(f.ends == 4 && body_text(f) == "ok", "204, 304 and 100 carry no body");

    g := make_driver(.RESPONSE);

    defer free_driver(g);
    g.parser.response_to = .HEAD;
    drive(g, "HTTP/1.1 200 OK\r\nContent-Length: 1234\r\n\r\n");
    check(g.ends == 1 && g.parser.head.content_length == 1234 && g.parser.head.body == .NONE, "HEAD response keeps the length it names");

    // Chunked response with a trailer.
    h := make_driver(.RESPONSE);

    defer free_driver(h);
    drive(h, "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nTrailer: Digest\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\nDigest: abc\r\n\r\n");
    check(h.ends == 1 && body_text(h) == "Wikipedia" && h.parser.trailers.count == 1 && h.parser.trailers[0].value == "abc", "chunked response");

    // Reason phrases are optional.
    check(observe(response_run(), "HTTP/1.1 204\r\n\r\n", .{}) == "S\x01204\x01\x011.1\nD\nC\n", "no reason");
    check(observe(response_run(), "HTTP/1.1 200 \r\nContent-Length: 0\r\n\r\n", .{}) == "S\x01200\x01\x011.1\nH\x01Content-Length\x010\nD\nC\n", "an empty reason");

    // 101 and CONNECT tunnels switch protocols.
    i := make_driver(.RESPONSE);

    defer free_driver(i);
    drive(i, "HTTP/1.1 101 Switching Protocols\r\nConnection: upgrade\r\nUpgrade: websocket\r\n\r\n\x81\x05hello");
    check(i.ends == 1 && i.last == .SWITCHED, "101 switches");

    j := make_driver(.RESPONSE);

    defer free_driver(j);
    j.parser.response_to = .CONNECT;
    drive(j, "HTTP/1.1 200 Connection established\r\n\r\n");
    check(j.ends == 1 && j.last == .SWITCHED, "a CONNECT tunnel switches");

    k := make_driver(.RESPONSE);

    defer free_driver(k);
    k.parser.response_to = .CONNECT;
    drive(k, "HTTP/1.1 407 Proxy Authentication Required\r\nContent-Length: 0\r\n\r\n");
    check(k.ends == 1 && k.last == .NEED_MORE, "a refused CONNECT does not");

    check(error_kind_of(.RESPONSE, "HTTP/1.1 600 X\r\n\r\n") == .BAD_STATUS, "status 600");
    check(error_kind_of(.RESPONSE, "HTTP/1.1 99 X\r\n\r\n") == .BAD_START_LINE || error_kind_of(.RESPONSE, "HTTP/1.1 99 X\r\n\r\n") == .BAD_STATUS, "status 99");
    check(error_kind_of(.RESPONSE, "HTTP/2.0 200 OK\r\n\r\n") == .VERSION_NOT_SUPPORTED, "response version");
}

test_eof :: () {
    p: Parser;

    parser_init(*p, .REQUEST);
    defer parser_free(*p);

    check(parse_eof(*p).kind == .CLOSED, "closing before anything is clean");

    data := as_bytes("GET / HTT");
    used, event := parse(*p, data);

    check(event.kind == .NEED_MORE && !parser_idle(*p), "half a request line");
    check(parse_eof(*p).kind == .ERROR && p.error.kind == .UNEXPECTED_EOF, "closing inside it is an error");

    q: Parser;

    parser_init(*q, .REQUEST);
    defer parser_free(*q);
    data = as_bytes("GET / HTTP/1.1\r\nHost: a\r\n\r\n\r\n");
    used, event = parse(*q, data);
    used, event = parse(*q, advance(data, used));
    check(event.kind == .END, "the request ended");
    used, event = parse(*q, advance(data, 27));
    check(event.kind == .NEED_MORE && parser_idle(*q), "leading empty lines are skipped");
    check(parse_eof(*q).kind == .CLOSED, "closing between requests is clean");

    // After an error the parser stays failed.
    r: Parser;

    parser_init(*r, .REQUEST);
    defer parser_free(*r);
    used, event = parse(*r, as_bytes("BAD\r\n"));
    check(event.kind == .ERROR, "garbage");
    used, event = parse(*r, as_bytes("GET / HTTP/1.1\r\n\r\n"));
    check(event.kind == .ERROR && used == 0, "stays failed");

    parser_reset(*r);
    used, event = parse(*r, as_bytes("GET / HTTP/1.0\r\n\r\n"));
    check(event.kind == .HEAD, "parser_reset starts over");
}

test_limits :: () {
    limits: Limits;
    limits.require_host = false;

    limits.max_line_size = 64;
    long_target := tprint("GET /%0 HTTP/1.1\r\n\r\n", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    check(error_kind_of(.REQUEST, long_target, limits) == .URI_TOO_LONG && error_status(.URI_TOO_LONG) == 414, "request line over the limit");
    check(error_kind_of(.REQUEST, "GET / HTTP/1.1\r\nX: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\n\r\n", limits) == .HEADERS_TOO_LARGE, "header line over the limit");
    check(error_kind_of(.REQUEST, "GET / HTTP/1.1\r\nX: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", limits) == .HEADERS_TOO_LARGE, "a long line is refused before its end arrives");

    limits = .{};
    limits.require_host = false;
    limits.max_headers = 3;
    check(error_kind_of(.REQUEST, "GET / HTTP/1.1\r\nA: 1\r\nB: 2\r\nC: 3\r\n\r\n", limits) == .NONE, "at the header count limit");
    check(error_kind_of(.REQUEST, "GET / HTTP/1.1\r\nA: 1\r\nB: 2\r\nC: 3\r\nD: 4\r\n\r\n", limits) == .TOO_MANY_HEADERS, "over the header count limit");
    check(error_status(.TOO_MANY_HEADERS) == 431, "431");

    limits = .{};
    limits.require_host = false;
    limits.max_head_size = 40;
    check(error_kind_of(.REQUEST, "GET / HTTP/1.1\r\nA: 1\r\nB: 2\r\nC: 3\r\nD: 4\r\n\r\n", limits) == .HEADERS_TOO_LARGE, "head over its size limit");

    limits = .{};
    limits.require_host = false;
    limits.max_body_size = 10;
    check(error_kind_of(.REQUEST, "POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\n", limits) == .NONE, "a body at the limit");
    check(error_kind_of(.REQUEST, "POST / HTTP/1.1\r\nContent-Length: 11\r\n\r\n", limits) == .BODY_TOO_LARGE, "a declared body over the limit");
    check(error_status(.BODY_TOO_LARGE) == 413, "413");
    check(error_kind_of(.REQUEST, "POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n5\r\n12345\r\n6\r\n123456\r\n0\r\n\r\n", limits) == .BODY_TOO_LARGE, "a chunked body over the limit");
    check(error_kind_of(.RESPONSE, "HTTP/1.0 200 OK\r\n\r\n123456789012345", limits) == .BODY_TOO_LARGE, "a body until close over the limit");

    limits.max_body_size = -1;
    check(error_kind_of(.REQUEST, "POST / HTTP/1.1\r\nContent-Length: 9223372036854775807\r\n\r\n", limits) == .NONE, "no body limit");

    limits = .{};
    limits.require_host = false;
    limits.max_trailer_size = 20;
    check(error_kind_of(.REQUEST, "POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX-Trailer: aaaaaaaaaaaaaaaaaaaaaa\r\n\r\n", limits) == .HEADERS_TOO_LARGE, "trailers over their limit");

    limits = .{};
    limits.require_host = false;
    limits.max_chunk_extension = 8;
    check(error_kind_of(.REQUEST, "POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n1;aaaaaaaaaaaaa\r\nx\r\n0\r\n\r\n", limits) == .BAD_CHUNK, "a long chunk extension");

    limits = .{};
    check(error_kind_of(.REQUEST, "GET / HTTP/1.1\r\n\r\n", limits) == .MISSING_HOST, "Host is required for HTTP/1.1");
    check(error_kind_of(.REQUEST, "GET / HTTP/1.0\r\n\r\n", limits) == .NONE, "but not for HTTP/1.0");
    check(error_kind_of(.REQUEST, "GET / HTTP/1.1\r\nHost: a\r\nHost: b\r\n\r\n", limits) == .DUPLICATE_HOST, "two Host headers");
    check(error_kind_of(.REQUEST, "GET / HTTP/1.1\r\nHost: a@b\r\n\r\n", limits) == .BAD_HEADER_VALUE, "userinfo in Host");
}

test_status_mapping :: () {
    check(error_status(.NONE) == 200, "none");
    check(error_status(.BAD_START_LINE) == 400 && error_status(.OBS_FOLD) == 400 && error_status(.CHUNK_SIZE_OVERFLOW) == 400, "400s");
    check(error_status(.VERSION_NOT_SUPPORTED) == 505, "505");
    check(error_status(.UNSUPPORTED_TRANSFER_ENCODING) == 501, "501");

    check(error_kind_of(.REQUEST, "GET / HTTP/2.0\r\n\r\n") == .VERSION_NOT_SUPPORTED, "HTTP/2.0 in a request line");
    check(error_kind_of(.REQUEST, "GET / HTTP/1.2\r\n\r\n") == .VERSION_NOT_SUPPORTED, "HTTP/1.2");
    check(error_kind_of(.REQUEST, "GET / HTTP/0.9\r\n\r\n") == .VERSION_NOT_SUPPORTED, "HTTP/0.9");
    check(error_kind_of(.REQUEST, "GET / http/1.1\r\n\r\n") == .BAD_VERSION, "lowercase http");
    check(error_kind_of(.REQUEST, "GET / HTTP/1.10\r\n\r\n") == .BAD_VERSION, "HTTP/1.10");
    check(error_kind_of(.REQUEST, "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: gzip, chunked\r\n\r\n") == .UNSUPPORTED_TRANSFER_ENCODING, "gzip under chunked");
}

test_writers :: () {
    out: [..] u8;

    defer array_reset(*out);

    headers := Header_Field.[.{ "Content-Type", "text/plain" }, .{ "X-Id", "7" }];
    kind, ok := write_response_head(*out, 200, headers, 5);

    check(ok && kind == .LENGTH, "a response with a length");
    check(text_of(out) == "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Id: 7\r\nContent-Length: 5\r\n\r\n", tprint("response head: %", text_of(out)));

    out.count = 0;
    kind, ok = write_response_head(*out, 200, headers, -1);
    check(ok && kind == .CHUNKED && contains(text_of(out), "Transfer-Encoding: chunked\r\n"), "an unknown length is chunked");

    out.count = 0;
    options: Response_Options;
    options.version = .HTTP_1_0;
    kind, ok = write_response_head(*out, 200, NO_HEADERS, -1, options);
    check(ok && kind == .UNTIL_CLOSE && contains(text_of(out), "HTTP/1.0 200 OK") && !contains(text_of(out), "chunked") && !contains(text_of(out), "keep-alive"), "HTTP/1.0 cannot chunk");

    out.count = 0;
    options = .{};
    options.keep_alive = false;
    options.date = "Sun, 06 Nov 1994 08:49:37 GMT";
    options.server = "jaic";
    write_response_head(*out, 404, NO_HEADERS, 0, options);
    check(text_of(out) == "HTTP/1.1 404 Not Found\r\nDate: Sun, 06 Nov 1994 08:49:37 GMT\r\nServer: jaic\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", tprint("close and date: %", text_of(out)));

    out.count = 0;
    kind, ok = write_response_head(*out, 204, NO_HEADERS, 0);
    check(ok && kind == .NONE && text_of(out) == "HTTP/1.1 204 No Content\r\n\r\n", "204 has no framing");

    out.count = 0;
    options = .{};
    options.request_method = .HEAD;
    kind, ok = write_response_head(*out, 200, NO_HEADERS, 12, options);
    check(ok && kind == .NONE && contains(text_of(out), "Content-Length: 12\r\n"), "a HEAD response names the length of the body it does not send");

    // Header injection and framing headers set by hand are refused, and nothing is written.
    out.count = 0;
    kind, ok = write_response_head(*out, 200, .[.{ "X-A", "ok\r\nSet-Cookie: x=1" }], 0);
    check(!ok && out.count == 0, "CRLF in a value");
    kind, ok = write_response_head(*out, 200, .[.{ "X A", "ok" }], 0);
    check(!ok && out.count == 0, "a bad name");
    kind, ok = write_response_head(*out, 200, .[.{ "content-length", "3" }], 0);
    check(!ok && out.count == 0, "Content-Length set by hand");
    kind, ok = write_response_head(*out, 200, .[.{ "Transfer-Encoding", "chunked" }], 0);
    check(!ok && out.count == 0, "Transfer-Encoding set by hand");

    // A request head, and the parser reads back what the writer wrote.
    out.count = 0;
    kind, ok = write_request_head(*out, "POST", "/upload", .[.{ "Host", "example.com" }], 4);
    write_chunk(*out, "");
    check(ok && kind == .LENGTH && text_of(out) == "POST /upload HTTP/1.1\r\nHost: example.com\r\nContent-Length: 4\r\n\r\n", "request head");

    out.count = 0;
    kind, ok = write_request_head(*out, "GET", "/", .[.{ "Host", "a" }], 0);
    check(ok && kind == .NONE && !contains(text_of(out), "Content-Length"), "a GET without a body has no length");

    kind, ok = write_request_head(*out, "GET", "/a b", NO_HEADERS, 0);
    check(!ok, "a space in the target");

    // Chunks.
    out.count = 0;
    kind, ok = write_request_head(*out, "PUT", "/x", .[.{ "Host", "a" }], -1);
    check(ok && kind == .CHUNKED, "chunked request");
    write_chunk(*out, "hello");
    write_chunk(*out, "");
    write_chunk(*out, "0123456789abcdef");
    check(write_last_chunk(*out, .[.{ "Digest", "sha-256=abc" }]), "last chunk with a trailer");
    text := text_of(out);

    check(ends_with(text, "5\r\nhello\r\n10\r\n0123456789abcdef\r\n0\r\nDigest: sha-256=abc\r\n\r\n"), tprint("chunk encoding: %", text));

    d := make_driver(.REQUEST);

    defer free_driver(d);
    drive(d, text);
    check(d.ends == 1 && body_text(d) == "hello0123456789abcdef" && d.parser.trailers.count == 1, "the parser reads the chunks back");

    check(!write_last_chunk(*out, .[.{ "Content-Length", "1" }]), "no framing in trailers");

    // Response to request to response, with every framing.
    for body: string.["", "x", "a longer body than before, with some length to it"] {
        for chunked: 0..1 {
            out.count = 0;
            r: [..] u8;

            defer array_reset(*r);

            if chunked {
                write_response_head(*out, 200, NO_HEADERS, -1);
                write_chunk(*out, body);
                write_last_chunk(*out);
            } else {
                write_response_head(*out, 200, NO_HEADERS, body.count);
                write_chunk_free_body(*out, body);
            }

            e := make_driver(.RESPONSE);

            defer free_driver(e);
            drive(e, text_of(out));
            check(e.ends == 1 && body_text(e) == body && e.parser.head.keep_alive, tprint("response round trip of '%' chunked=%", body, chunked));
        }
    }
}

write_chunk_free_body :: (out: *[..] u8, body: string) {
    for body array_add(out, it);
}

test_dates :: () {
    text: [29] u8;

    format_http_date(784111777, *text);
    check(as_string(text.data, 29) == "Sun, 06 Nov 1994 08:49:37 GMT", "the RFC example");

    format_http_date(0, *text);
    check(as_string(text.data, 29) == "Thu, 01 Jan 1970 00:00:00 GMT", "the epoch");

    format_http_date(951782400, *text);
    check(as_string(text.data, 29) == "Tue, 29 Feb 2000 00:00:00 GMT", "a leap day");

    format_http_date(4102444799, *text);
    check(as_string(text.data, 29) == "Thu, 31 Dec 2099 23:59:59 GMT", "the end of the century");

    for s: s64.[0, 1, 86399, 86400, 784111777, 951782400, 1700000000, 4102444799, 253402300799] {
        format_http_date(s, *text);
        back, ok := parse_http_date(as_string(text.data, 29));
        check(ok && back == s, tprint("date round trip %", s));
    }

    _, ok := parse_http_date("Mon, 06 Nov 1994 08:49:37 GMT");

    check(!ok, "a weekday that does not fit");
    _, ok = parse_http_date("Sunday, 06-Nov-94 08:49:37 GMT");
    check(!ok, "RFC 850 dates are not accepted");
    _, ok = parse_http_date("Sun, 06 Nov 1994 08:49:37 UTC");
    check(!ok, "only GMT");

    cache: Date_Cache;
    first := http_date(*cache, 784111777);

    check(first == "Sun, 06 Nov 1994 08:49:37 GMT", "cache");
    first_data := first.data;
    second := http_date(*cache, 784111777);

    check(second.data == first_data, "same second, same text");
    third := http_date(*cache, 784111778);

    check(third == "Sun, 06 Nov 1994 08:49:38 GMT", "next second");

    // The live-clock helper belongs to the reviewed native adapter surface.
    // Explicit-seconds formatting, parsing and cache checks above also run on WASM.
    #if OS != .WASM {
        before, before_ok := to_timespec(current_time_consensus());
        now := http_date_now(*cache);
        after, after_ok := to_timespec(current_time_consensus());

        check(now.count == 29 && ends_with(now, " GMT"), "the current date has the right shape");
        seconds, parsed := parse_http_date(now);

        check(before_ok && after_ok && parsed && seconds >= before.tv_sec && seconds <= after.tv_sec,
              "the current date agrees with the canonical Unix-time conversion");
    }
}

test_names :: () {
    // Every well-known name finds itself, in any case, and nothing else.
    info := type_info(Header_Name);

    check(info.names.count == HEADER_NAME_COUNT, "the name table and the enum have the same size");

    for id: 1..HEADER_NAME_COUNT - 1 {
        name := header_name_string(cast(Header_Name) id);

        check(lookup_header_name(name) == cast(Header_Name) id, tprint("lookup of %", name));
        check(lookup_header_name(to_lower_copy(name)) == cast(Header_Name) id, tprint("lowercase lookup of %", name));
        check(lookup_header_name(to_upper_copy(name)) == cast(Header_Name) id, tprint("uppercase lookup of %", name));

        upper := to_upper_copy(name);

        for *upper {
            if it.* == #char "-" it.* = #char "_";
        }

        check(info.names[id] == upper, tprint("enum member for %: % vs %", name, info.names[id], upper));
    }

    check(lookup_header_name("X-Unknown") == .UNKNOWN && lookup_header_name("") == .UNKNOWN && lookup_header_name("Hos") == .UNKNOWN, "unknown names");

    check(has_token("keep-alive, Upgrade", "upgrade") && has_token("close", "CLOSE") && !has_token("closed", "close") && has_token("a ;q=1, b", "a"), "has_token");
}

test_methods_and_status :: () {
    check(method_name(.GET) == "GET" && method_name(.OTHER) == "", "method names");
    check(version_name(.HTTP_1_0) == "HTTP/1.0" && version_name(.HTTP_1_1) == "HTTP/1.1", "version names");
    check(status_reason(404) == "Not Found" && status_reason(299) == "", "status reasons");
}

// The vector and scalar scans give the same results for the messages above.
test_simd_parity :: () {
    texts := string.[
        "GET /a/very/long/target/that/goes/on/for/more/than/sixteen/bytes?and=a&query=string HTTP/1.1\r\nHost: example.com\r\nX-Long: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\n\r\n",
        "GET / HTTP/1.1\r\nHost: a\r\nX: aaaaaaaaaaaaaaa\x01aaaaaaaaaaaaa\r\n\r\n",
        "GET / HTTP/1.1\r\nHost: a\r\nX: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x7f\r\n\r\n",
        "GET /aaaaaaaaaaaaaaaaaaaaaaaaaaa\x80 HTTP/1.1\r\nHost: a\r\n\r\n",
        "GET /aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa HTTP/1.1\nHost: a\r\n\r\n",
        "GET /aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa HTTP/1.1\r\nHost: a\rX\r\n\r\n",
        "POST / HTTP/1.1\r\nHost: a\r\nX: caf\xc3\xa9 \xff tab\there\r\nContent-Length: 4\r\n\r\nbody",
    ];

    for texts {
        http_use_simd = true;
        vector := observe_every_cut(request_run(), it, "vector scan");

        http_use_simd = false;
        scalar := observe_every_cut(request_run(), it, "scalar scan");

        http_use_simd = true;

        check(vector == scalar, tprint("vector and scalar scans differ for %", it));
    }
}

main :: () {
    test_request_heads();
    test_keep_alive();
    test_request_bodies();
    test_expect();
    test_pipelining();
    test_trim();
    test_responses();
    test_eof();
    test_limits();
    test_status_mapping();
    test_writers();
    test_dates();
    test_names();
    test_methods_and_status();
    test_simd_parity();

    finish();
}
