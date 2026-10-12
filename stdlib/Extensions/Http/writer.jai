// Writers: request and response heads, the chunked encoder and the small fixed responses. They
// append to a `[..] u8` the caller owns, so a server can build a whole response and send it with
// one write. Framing is chosen from the body length, so a caller cannot send a length and a
// chunked body at once.

// What the response writer needs to know besides the status and the headers.
Response_Options :: struct {
    version := Version.HTTP_1_1;
    reason: string; // empty for the standard phrase of the status
    request_method := Method.GET; // HEAD responses and CONNECT tunnels carry no body
    keep_alive := true; // false adds `Connection: close`
    date: string; // when not empty, sent as the Date header (see Date_Cache)
    server: string; // when not empty, sent as the Server header
}

// The standard reason phrase of a status, or "" for one this table does not know.
status_reason :: (status: int) -> string {
    if status == {
        case 100; return "Continue";
        case 101; return "Switching Protocols";
        case 102; return "Processing";
        case 103; return "Early Hints";
        case 200; return "OK";
        case 201; return "Created";
        case 202; return "Accepted";
        case 203; return "Non-Authoritative Information";
        case 204; return "No Content";
        case 205; return "Reset Content";
        case 206; return "Partial Content";
        case 207; return "Multi-Status";
        case 300; return "Multiple Choices";
        case 301; return "Moved Permanently";
        case 302; return "Found";
        case 303; return "See Other";
        case 304; return "Not Modified";
        case 307; return "Temporary Redirect";
        case 308; return "Permanent Redirect";
        case 400; return "Bad Request";
        case 401; return "Unauthorized";
        case 402; return "Payment Required";
        case 403; return "Forbidden";
        case 404; return "Not Found";
        case 405; return "Method Not Allowed";
        case 406; return "Not Acceptable";
        case 407; return "Proxy Authentication Required";
        case 408; return "Request Timeout";
        case 409; return "Conflict";
        case 410; return "Gone";
        case 411; return "Length Required";
        case 412; return "Precondition Failed";
        case 413; return "Content Too Large";
        case 414; return "URI Too Long";
        case 415; return "Unsupported Media Type";
        case 416; return "Range Not Satisfiable";
        case 417; return "Expectation Failed";
        case 418; return "I'm a teapot";
        case 421; return "Misdirected Request";
        case 422; return "Unprocessable Content";
        case 426; return "Upgrade Required";
        case 428; return "Precondition Required";
        case 429; return "Too Many Requests";
        case 431; return "Request Header Fields Too Large";
        case 451; return "Unavailable For Legal Reasons";
        case 500; return "Internal Server Error";
        case 501; return "Not Implemented";
        case 502; return "Bad Gateway";
        case 503; return "Service Unavailable";
        case 504; return "Gateway Timeout";
        case 505; return "HTTP Version Not Supported";
        case 507; return "Insufficient Storage";
        case 511; return "Network Authentication Required";
    }

    return "";
}

// Appends a request head, framing included, and returns how the body must be sent: with a
// non-negative `body_length`, `Content-Length` is added (except for a GET, HEAD, DELETE, CONNECT or
// TRACE without a body) and the body is sent as is (.LENGTH, or .NONE for length 0); with -1 the
// length is not known, `Transfer-Encoding: chunked` is added and the body goes through
// write_chunk and write_last_chunk (.CHUNKED). `ok` is false, and `out` is left as it was,
// when a header is not valid or sets the framing itself.
write_request_head :: (out: *[..] u8, method: string, target: string, headers: [] Header, body_length: s64) -> Body_Kind, bool {
    mark := out.count;

    valid := method.count > 0 && target.count > 0 && find_bad_target_byte(target.data, 0, target.count) < 0;

    for method {
        if !is_token_char(it) valid = false;
    }

    if valid valid = fields_ok(headers);

    if !valid {
        out.count = mark;

        return .NONE, false;
    }

    put(out, method);
    put(out, " ");
    put(out, target);
    put(out, " HTTP/1.1\r\n");
    put_headers(out, headers);

    kind := Body_Kind.NONE;

    if body_length < 0 {
        put(out, "Transfer-Encoding: chunked\r\n");

        kind = .CHUNKED;
    } else {
        bodyless := method == "GET" || method == "HEAD" || method == "DELETE" || method == "CONNECT" || method == "TRACE";

        if body_length > 0 || !bodyless {
            put(out, "Content-Length: ");
            put_decimal(out, body_length);
            put(out, "\r\n");
        }

        if body_length > 0 kind = .LENGTH;
    }

    put(out, "\r\n");

    return kind, true;
}

// Appends a response head and returns how the body must be sent (see write_request_head).
// `body_length` is the length of the body the response is for; for a HEAD response (set
// `request_method`) it only fills in Content-Length, and nothing may be sent after the head. 1xx,
// 204 and CONNECT 2xx responses get no framing headers. A length of -1 means unknown: chunked on
// HTTP/1.1, and for HTTP/1.0 the body ends when the connection closes (`Connection: close` is
// added and the caller must close).
write_response_head :: (out: *[..] u8, status: int, headers: [] Header, body_length: s64, options := Response_Options.{}) -> Body_Kind, bool {
    mark := out.count;

    if status < 100 || status > 599 || !fields_ok(headers) {
        out.count = mark;

        return .NONE, false;
    }

    reason := options.reason;
    if reason.count == 0 reason = status_reason(status);

    if find_bad_value_byte(reason.data, 0, reason.count) >= 0 {
        out.count = mark;

        return .NONE, false;
    }

    put(out, version_name(options.version));
    put(out, " ");
    put_decimal(out, status);
    put(out, " ");
    put(out, reason);
    put(out, "\r\n");

    if options.date.count > 0 && find_bad_value_byte(options.date.data, 0, options.date.count) < 0 {
        put(out, "Date: ");
        put(out, options.date);
        put(out, "\r\n");
    }

    if options.server.count > 0 && find_bad_value_byte(options.server.data, 0, options.server.count) < 0 {
        put(out, "Server: ");
        put(out, options.server);
        put(out, "\r\n");
    }

    put_headers(out, headers);

    kind := Body_Kind.NONE;
    framing_headers := true;
    keep_alive := options.keep_alive;

    if status < 200 || status == 204 framing_headers = false;
    if options.request_method == .CONNECT && status >= 200 && status < 300 framing_headers = false;

    if framing_headers {
        if body_length >= 0 {
            // A 304 describes the body a GET would have had; the caller names its length itself.
            if status != 304 {
                put(out, "Content-Length: ");
                put_decimal(out, body_length);
                put(out, "\r\n");
            }

            if body_length > 0 kind = .LENGTH;
        } else if options.request_method == .HEAD || status == 304 {
            // Nothing is sent, and the length is unknown: say nothing.
        } else if options.version == .HTTP_1_1 {
            put(out, "Transfer-Encoding: chunked\r\n");

            kind = .CHUNKED;
        } else {
            kind = .UNTIL_CLOSE;
            keep_alive = false;
        }
    }

    if options.request_method == .HEAD || status == 304 kind = .NONE;

    if options.version == .HTTP_1_1 {
        if !keep_alive put(out, "Connection: close\r\n");
    } else if keep_alive {
        put(out, "Connection: keep-alive\r\n");
    }

    put(out, "\r\n");

    return kind, true;
}

// `HTTP/1.1 100 Continue`, the answer to a request with `Expect: 100-continue` before its body
// is read.
write_continue :: (out: *[..] u8) {
    put(out, "HTTP/1.1 100 Continue\r\n\r\n");
}

// Appends `data` as one chunk. An empty `data` writes nothing: a chunk of size 0 would end the
// body (use write_last_chunk).
write_chunk :: (out: *[..] u8, data: [] u8) {
    if data.count == 0 return;

    put_hex(out, data.count);
    put(out, "\r\n");
    append_bytes(out, data.data, data.count);
    put(out, "\r\n");
}

write_chunk :: (out: *[..] u8, data: string) {
    bytes: [] u8;
    bytes.count = data.count;
    bytes.data = data.data;

    write_chunk(out, bytes);
}

// The zero-size chunk with optional trailers, which ends a chunked body. False, with nothing
// written, when a trailer is not valid or is a field a trailer may not be (framing, Host).
write_last_chunk :: (out: *[..] u8) -> bool {
    trailers: [] Header;

    return write_last_chunk(out, trailers);
}

write_last_chunk :: (out: *[..] u8, trailers: [] Header) -> bool {
    if !fields_ok(trailers) return false;

    for trailers {
        if it.id == .CONTENT_LENGTH || it.id == .TRANSFER_ENCODING || it.id == .HOST return false;
    }

    put(out, "0\r\n");
    put_headers(out, trailers);
    put(out, "\r\n");

    return true;
}

#scope_module

put :: (out: *[..] u8, text: string) {
    append_bytes(out, text.data, text.count);
}

put_decimal :: (out: *[..] u8, value: s64) {
    digits: [20] u8;
    n := 0;
    v := value;

    if v == 0 {
        digits[0] = #char "0";
        n = 1;
    }

    while v > 0 {
        digits[n] = cast(u8) (#char "0" + v % 10);
        n += 1;
        v /= 10;
    }

    while n > 0 {
        n -= 1;
        append_bytes(out, *digits[n], 1);
    }
}

put_hex :: (out: *[..] u8, value: s64) {
    digits: [16] u8;
    n := 0;
    v := value;

    while v > 0 {
        d := cast(u8) (v & 15);

        if d < 10 digits[n] = #char "0" + d;
        else digits[n] = #char "a" + d - 10;

        n += 1;
        v >>= 4;
    }

    while n > 0 {
        n -= 1;
        append_bytes(out, *digits[n], 1);
    }
}

put_headers :: (out: *[..] u8, headers: [] Header) {
    for headers {
        put(out, it.name);
        put(out, ": ");
        put(out, it.value);
        put(out, "\r\n");
    }
}

// Every header has a token name and a value without control bytes (a CR or LF in a value would
// let the caller's data start a new header or a new response), and none sets the framing.
fields_ok :: (headers: [] Header) -> bool {
    for header: headers {
        if header.name.count == 0 return false;

        for header.name {
            if !is_token_char(it) return false;
        }

        if find_bad_value_byte(header.value.data, 0, header.value.count) >= 0 return false;

        id := header.id;
        if id == .UNKNOWN id = lookup_header_name(header.name);

        if id == .CONTENT_LENGTH || id == .TRANSFER_ENCODING return false;
    }

    return true;
}

#scope_export
