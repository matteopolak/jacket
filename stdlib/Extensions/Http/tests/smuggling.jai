// Request smuggling and response splitting defenses: messages that two parsers could frame
// differently must be refused outright, and the ones that merely look odd must still be framed
// the one way the RFC gives. Each case runs whole and one byte at a time.
//
//     jaic stdlib/Extensions/Http/tests/smuggling.jai -interp

#load "support/support.jai";

Smuggle :: struct {
    name: string;
    input: string;
    expect: Parse_Error_Kind;
}

// Every one of these is refused with the kind named.
REFUSED :: Smuggle.[
    // Content-Length and Transfer-Encoding together (CL.TE and TE.CL).
    .{ "CL.TE", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 6\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\nX", .CONTENT_LENGTH_WITH_TRANSFER_ENCODING },
    .{ "TE.CL", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\nContent-Length: 6\r\n\r\n0\r\n\r\nX", .CONTENT_LENGTH_WITH_TRANSFER_ENCODING },
    .{ "TE.CL with an empty length", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\nContent-Length:\r\n\r\n0\r\n\r\n", .BAD_CONTENT_LENGTH },

    // Content-Length that parsers read differently.
    .{ "two lengths", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 5\r\nContent-Length: 6\r\n\r\nhello!", .DUPLICATE_CONTENT_LENGTH },
    .{ "two equal lengths", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 5\r\nContent-Length: 5\r\n\r\nhello", .DUPLICATE_CONTENT_LENGTH },
    .{ "a list of lengths", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 5, 5\r\n\r\nhello", .BAD_CONTENT_LENGTH },
    .{ "a plus sign", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: +5\r\n\r\nhello", .BAD_CONTENT_LENGTH },
    .{ "a minus sign", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: -5\r\n\r\nhello", .BAD_CONTENT_LENGTH },
    .{ "hex", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 0x5\r\n\r\nhello", .BAD_CONTENT_LENGTH },
    .{ "a space inside", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 1 5\r\n\r\nhello", .BAD_CONTENT_LENGTH },
    .{ "a decimal point", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 5.0\r\n\r\nhello", .BAD_CONTENT_LENGTH },
    .{ "an underscore", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 1_0\r\n\r\nhello", .BAD_CONTENT_LENGTH },
    .{ "empty", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: \r\n\r\n", .BAD_CONTENT_LENGTH },
    .{ "63 bits plus one", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 9223372036854775808\r\n\r\n", .BAD_CONTENT_LENGTH },
    .{ "far too big", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 99999999999999999999999999\r\n\r\n", .BAD_CONTENT_LENGTH },
    .{ "a non-ASCII digit", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: \xd9\xa5\r\n\r\n", .BAD_CONTENT_LENGTH },

    // Transfer-Encoding spellings that some parsers take as chunked and others do not.
    .{ "xchunked", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: xchunked\r\n\r\n0\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "chunkedx", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunkedx\r\n\r\n0\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "a vertical tab", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: \x0bchunked\r\n\r\n0\r\n\r\n", .BAD_HEADER_VALUE },
    .{ "a form feed", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\x0c\r\n\r\n0\r\n\r\n", .BAD_HEADER_VALUE },
    .{ "a NUL", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunk\x00ed\r\n\r\n0\r\n\r\n", .BAD_HEADER_VALUE },
    .{ "quoted", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: \"chunked\"\r\n\r\n0\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "chunked twice", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked, chunked\r\n\r\n0\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "chunked on two lines", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "chunked then identity", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\nTransfer-Encoding: identity\r\n\r\n0\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "not ending in chunked", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: gzip\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "empty", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding:\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "an empty element", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked,\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "a leading empty element", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: ,chunked\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "a parameter", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked;q=1\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "in HTTP/1.0", "POST / HTTP/1.0\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n", .BAD_TRANSFER_ENCODING },
    .{ "a coding we cannot undo", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: gzip, chunked\r\n\r\n", .UNSUPPORTED_TRANSFER_ENCODING },

    // Header syntax that hides a header from one parser and not another.
    .{ "whitespace before the colon", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding : chunked\r\n\r\n0\r\n\r\n", .WHITESPACE_BEFORE_COLON },
    .{ "a tab before the colon", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length\t: 5\r\n\r\nhello", .WHITESPACE_BEFORE_COLON },
    .{ "a folded line", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n x\r\n\r\n0\r\n\r\n", .OBS_FOLD },
    .{ "a folded name", "POST / HTTP/1.1\r\nHost: a\r\n Transfer-Encoding: chunked\r\n\r\n0\r\n\r\n", .OBS_FOLD },
    .{ "a tab fold", "POST / HTTP/1.1\r\nHost: a\r\nX: 1\r\n\tContent-Length: 5\r\n\r\nhello", .OBS_FOLD },
    .{ "no colon", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding chunked\r\n\r\n", .WHITESPACE_BEFORE_COLON },
    .{ "an empty name", "POST / HTTP/1.1\r\nHost: a\r\n: chunked\r\n\r\n", .BAD_HEADER_NAME },
    .{ "a name with a space", "POST / HTTP/1.1\r\nHost: a\r\nTransfer Encoding: chunked\r\n\r\n", .WHITESPACE_BEFORE_COLON },
    .{ "a name with a NUL", "POST / HTTP/1.1\r\nHost: a\r\nX\x00: 1\r\n\r\n", .BAD_HEADER_NAME },
    .{ "a name with a high byte", "POST / HTTP/1.1\r\nHost: a\r\nX\xe9: 1\r\n\r\n", .BAD_HEADER_NAME },
    .{ "a NUL in a value", "POST / HTTP/1.1\r\nHost: a\r\nX: a\x00b\r\n\r\n", .BAD_HEADER_VALUE },
    .{ "DEL in a value", "POST / HTTP/1.1\r\nHost: a\r\nX: a\x7fb\r\n\r\n", .BAD_HEADER_VALUE },

    // Line endings.
    .{ "a bare LF between headers", "POST / HTTP/1.1\r\nHost: a\nContent-Length: 5\r\n\r\nhello", .BAD_LINE_ENDING },
    .{ "a bare LF ends the head", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 5\n\nhello", .BAD_LINE_ENDING },
    .{ "a bare LF in the request line", "POST / HTTP/1.1\nHost: a\r\n\r\n", .BAD_LINE_ENDING },
    .{ "a bare CR in a value", "POST / HTTP/1.1\r\nHost: a\r\nX: a\rb\r\n\r\n", .BAD_LINE_ENDING },
    .{ "a bare CR in a name", "POST / HTTP/1.1\r\nHost: a\r\nX\rY: b\r\n\r\n", .BAD_LINE_ENDING },
    .{ "a double CR", "POST / HTTP/1.1\r\nHost: a\r\nX: b\r\r\n\r\n", .BAD_LINE_ENDING },
    .{ "a CR before the head ends", "POST / HTTP/1.1\r\nHost: a\r\n\r\r\nhello", .BAD_LINE_ENDING },

    // The request line.
    .{ "two spaces", "GET  / HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_TARGET },
    .{ "a tab as separator", "GET\t/ HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_METHOD },
    .{ "a space in the target", "GET /a b HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_VERSION },
    .{ "a tab in the target", "GET /a\tb HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_TARGET },
    .{ "no target", "GET HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_START_LINE },
    .{ "a trailing space", "GET / HTTP/1.1 \r\nHost: a\r\n\r\n", .BAD_VERSION },
    .{ "a leading space", " GET / HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_METHOD },
    .{ "a high byte in the target", "GET /caf\xc3\xa9 HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_TARGET },
    .{ "a fragment", "GET /#frag HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_TARGET },
    .{ "a method that is not a token", "G(T / HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_METHOD },
    .{ "authority form for GET", "GET example.com:80 HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_TARGET },
    .{ "a path for CONNECT", "CONNECT /x HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_TARGET },
    .{ "a CONNECT without a port", "CONNECT example.com HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_TARGET },
    .{ "a target of its own kind", "GET ftp HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_TARGET },
    .{ "asterisk for GET", "GET * HTTP/1.1\r\nHost: a\r\n\r\n", .BAD_TARGET },

    // Host.
    .{ "two Hosts", "GET / HTTP/1.1\r\nHost: a\r\nHost: b\r\n\r\n", .DUPLICATE_HOST },
    .{ "no Host", "GET / HTTP/1.1\r\n\r\n", .MISSING_HOST },
    .{ "userinfo in Host", "GET / HTTP/1.1\r\nHost: evil.com@good.com\r\n\r\n", .BAD_HEADER_VALUE },
    .{ "a comma in Host", "GET / HTTP/1.1\r\nHost: a.com, b.com\r\n\r\n", .BAD_HEADER_VALUE },

    // Chunked bodies.
    .{ "a size with 0x", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n0x5\r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "a negative size", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n-5\r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "a plus size", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n+5\r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "no size", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n\r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "a size that is too big for 63 bits", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n8000000000000000\r\nx\r\n0\r\n\r\n", .CHUNK_SIZE_OVERFLOW },
    .{ "a size of 64 bits", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\nffffffffffffffff\r\nx\r\n0\r\n\r\n", .CHUNK_SIZE_OVERFLOW },
    .{ "a size of 17 digits", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n10000000000000000\r\nx\r\n0\r\n\r\n", .CHUNK_SIZE_OVERFLOW },
    .{ "a size with a space", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5 \r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "a size followed by junk", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5 x\r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "a bare LF after the size", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "a bare CR after the size", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5\rhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "an LF inside an extension", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5;a\nb\r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "a NUL inside an extension", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5;a\x00\r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "an empty extension", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5;\r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "an open quote", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5;a=\"b\r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "data that is too long", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nhello\r\n0\r\n\r\n", .BAD_CHUNK },
    .{ "no CRLF after the data", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello0\r\n\r\n", .BAD_CHUNK },
    .{ "a bare LF after the data", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\n0\r\n\r\n", .BAD_CHUNK },
    .{ "an extra CR after the data", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\r\n0\r\n\r\n", .BAD_CHUNK },

    // Trailers.
    .{ "a Content-Length trailer", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nContent-Length: 5\r\n\r\n", .BAD_TRAILER },
    .{ "a Transfer-Encoding trailer", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nTransfer-Encoding: chunked\r\n\r\n", .BAD_TRAILER },
    .{ "a Host trailer", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nHost: b\r\n\r\n", .BAD_TRAILER },
    .{ "a folded trailer", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX: 1\r\n 2\r\n\r\n", .BAD_TRAILER },
    .{ "a trailer without a colon", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX 1\r\n\r\n", .BAD_TRAILER },
    .{ "a bare LF in a trailer", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX: 1\n\r\n", .BAD_LINE_ENDING },
];

// These look odd but are framed the one way the RFC gives: they are accepted.
ACCEPTED :: Smuggle.[
    .{ "a tab after the colon", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding:\tchunked\r\n\r\n0\r\n\r\n", .NONE },
    .{ "no blank after the colon", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding:chunked\r\n\r\n0\r\n\r\n", .NONE },
    .{ "capitals", "POST / HTTP/1.1\r\nHost: a\r\nTRANSFER-ENCODING: CHUNKED\r\n\r\n0\r\n\r\n", .NONE },
    .{ "blanks around", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding:  chunked \t\r\n\r\n0\r\n\r\n", .NONE },
    .{ "leading zeros in a length", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 0005\r\n\r\nhello", .NONE },
    .{ "a length with blanks", "POST / HTTP/1.1\r\nHost: a\r\nContent-Length:   5 \t\r\n\r\nhello", .NONE },
    .{ "leading zeros in a chunk size", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n0000000000000000000005\r\nhello\r\n0\r\n\r\n", .NONE },
    .{ "capitals in a chunk size", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\nA\r\n0123456789\r\n0\r\n\r\n", .NONE },
    .{ "extensions", "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n5 ; a = b ;c=\"d e\\\" f\";g\r\nhello\r\n0;done\r\n\r\n", .NONE },
    .{ "a leading blank line", "\r\nGET / HTTP/1.1\r\nHost: a\r\n\r\n", .NONE },
    .{ "obs-text in a value", "GET / HTTP/1.1\r\nHost: a\r\nX: caf\xe9\r\n\r\n", .NONE },
    .{ "a tab inside a value", "GET / HTTP/1.1\r\nHost: a\r\nX: a\tb\r\n\r\n", .NONE },
    .{ "a body on a GET", "GET / HTTP/1.1\r\nHost: a\r\nContent-Length: 3\r\n\r\nabc", .NONE },
];

main :: () {
    for REFUSED {
        got := error_kind_of(.REQUEST, it.input);
        check(got == it.expect, tprint("%: expected %, got %", it.name, it.expect, got));
    }

    for ACCEPTED {
        got := error_kind_of(.REQUEST, it.input);
        check(got == it.expect, tprint("%: expected %, got %", it.name, it.expect, got));
    }

    unlimited: Limits;
    unlimited.max_body_size = -1;
    check(error_kind_of(.REQUEST, "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n7fffffffffffffff\r\n", unlimited) == .NONE, "the largest chunk size");

    // A refused message leaves the parser failed: what follows is never read as a new message.
    d := make_driver(.REQUEST);

    defer free_driver(d);
    drive(d, "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 5\r\nContent-Length: 5\r\n\r\nhello");
    check(d.last == .ERROR && d.heads == 0, "no head is reported for a refused message");

    // The smuggled request is never seen: a chunked body that ends early, followed by a request,
    // is two requests; a body that goes on past the length is a body.
    e := make_driver(.REQUEST);

    defer free_driver(e);
    drive(e, "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 5\r\n\r\nhelloGET /admin HTTP/1.1\r\nHost: a\r\n\r\n");
    check(e.heads == 2 && e.ends == 2 && body_text(e) == "hello", "bytes after the declared length are the next request");

    // The same shapes for responses: no Content-Length with Transfer-Encoding either.
    check(error_kind_of(.RESPONSE, "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n") == .CONTENT_LENGTH_WITH_TRANSFER_ENCODING, "CL.TE in a response");
    check(error_kind_of(.RESPONSE, "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nContent-Length: 6\r\n\r\nhello!") == .DUPLICATE_CONTENT_LENGTH, "two lengths in a response");
    check(error_kind_of(.RESPONSE, "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked, gzip\r\n\r\n") == .BAD_TRANSFER_ENCODING, "chunked not last in a response");
    check(error_kind_of(.RESPONSE, "HTTP/1.1 200 OK\r\nX: a\nContent-Length: 5\r\n\r\nhello") == .BAD_LINE_ENDING, "a bare LF in a response");
    check(error_kind_of(.RESPONSE, "HTTP/1.1 200 OK\r\nContent-Length : 5\r\n\r\nhello") == .WHITESPACE_BEFORE_COLON, "a space before the colon in a response");
    check(error_kind_of(.RESPONSE, "HTTP/1.1 200 OK\nContent-Length: 5\r\n\r\nhello") == .BAD_LINE_ENDING, "a bare LF in the status line");
    check(error_kind_of(.RESPONSE, "HTTP/1.1 200 O\x00K\r\n\r\n") == .BAD_STATUS, "a NUL in the reason");

    finish();
}
