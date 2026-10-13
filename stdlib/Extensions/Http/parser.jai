// The incremental parser for requests and responses. See docs/stdlib/http-codec.md.
//
//     parser: Parser;
//     parser_init(*parser, .REQUEST);
//
//     data := bytes_from_the_socket;
//
//     while true {
//         used, event := parse(*parser, data);
//         data = advance(data, used);
//
//         if event.kind == {
//             case .HEAD;      handle(parser.head);
//             case .BODY;      collect(event.data);
//             case .END;       respond();
//             case .NEED_MORE; break;              // read more, then call parse again
//             case .ERROR;     reject(parser.error);
//         }
//     }

// A header line of the head or the trailers, as offsets into the buffer that holds it (the buffer
// moves while it grows; strings are made once the head is complete).
Span :: struct {
    name_start: int;
    name_end: int;
    value_start: int;
    value_end: int;
    id: Header_Name;
}

// Where the reader is inside a chunk extension (RFC 9112 7.1.1).
Ext_State :: enum u8 {
    BEFORE_SEMI; // after the size or an extension: blanks, then `;` or CRLF
    NAME_START; // after `;`
    NAME;
    AFTER_NAME; // blanks, then `=`, `;` or CRLF
    VALUE_START; // after `=`
    TOKEN_VALUE;
    QUOTED;
    QUOTED_PAIR;
    AFTER_QUOTED; // right after the closing quote
    AFTER_VALUE;
}

Parser_State :: enum u8 {
    START; // reading the start line and the headers
    BODY_NONE; // the head is out and there is no body: END is next
    BODY_LENGTH;
    BODY_CLOSE;
    CHUNK_SIZE;
    CHUNK_EXT;
    CHUNK_SIZE_LF;
    CHUNK_DATA;
    CHUNK_DATA_CR;
    CHUNK_DATA_LF;
    TRAILERS;
    DONE; // the message ended the connection (Connection: close...): nothing may follow
    SWITCHED;
    FAILED;
}

Parser :: struct {
    kind: Parser_Kind;
    limits: Limits;

    // For a response parser: the method of the request being answered. HEAD and CONNECT change
    // how the response body is framed (RFC 9112 6.3), so set it before the response head arrives.
    response_to: Method;

    head: Head; // valid after a HEAD event, until the next message begins
    trailers: [] Header_Field; // valid after END of a chunked body, until the next message begins
    error: Parse_Error;

    state: Parser_State;

    // Internal.
    started: bool; // head bytes of the current message are in `bytes`
    in_headers: bool; // the start line has been read
    bytes: [..] u8; // the head: start line and header lines, CRLFs included
    line_start: int;
    spans: [..] Span;
    list: [..] Header_Field;
    skipped_lines: int;

    // The start line, as offsets into `bytes`.
    method_start: int;
    method_end: int;
    target_start: int;
    target_end: int;
    reason_start: int;
    reason_end: int;
    version: Version;
    status: int;
    method: Method;

    // What the headers seen so far say (note_header).
    content_length_seen: bool;
    content_length: s64;
    encoding_seen: bool;
    chunked: bool;
    other_coding: bool;
    host_count: int;
    close: bool;
    keep_alive_token: bool;
    upgrade_token: bool;
    upgrade_header: bool;
    expect_continue: bool;

    remaining: s64; // bytes left in the Content-Length body or the current chunk
    total: s64; // decoded body bytes so far
    chunk_size: s64;
    chunk_digits: int;
    chunk_ext: int; // bytes of the extension so far
    ext: Ext_State;
    ext_token: int; // bytes of the name or value being read
    offset: int; // bytes of this message consumed before the current call

    tbytes: [..] u8; // trailers
    tline_start: int;
    tspans: [..] Span;
    tlist: [..] Header_Field;
}

parser_init :: (p: *Parser, kind: Parser_Kind, limits := Limits.{}) {
    parser_free(p);
    p.* = .{};
    p.kind = kind;
    p.limits = limits;
}

// Starts over for a new connection, keeping the allocated buffers.
parser_reset :: (p: *Parser) {
    kind := p.kind;
    limits := p.limits;

    bytes := p.bytes;
    spans := p.spans;
    list := p.list;
    tbytes := p.tbytes;
    tspans := p.tspans;
    tlist := p.tlist;

    p.* = .{};
    p.kind = kind;
    p.limits = limits;
    p.bytes = bytes;
    p.spans = spans;
    p.list = list;
    p.tbytes = tbytes;
    p.tspans = tspans;
    p.tlist = tlist;
    p.bytes.count = 0;
    p.spans.count = 0;
    p.list.count = 0;
    p.tbytes.count = 0;
    p.tspans.count = 0;
    p.tlist.count = 0;
}

parser_free :: (p: *Parser) {
    array_reset(*p.bytes);
    array_reset(*p.spans);
    array_reset(*p.list);
    array_reset(*p.tbytes);
    array_reset(*p.tspans);
    array_reset(*p.tlist);
}

// Between messages: gives back the head and trailer storage that grew past `limit` bytes (each of
// the six arrays on its own), so a connection that once saw a huge head does not keep it. The
// previous message's `head` and `trailers` are gone afterwards, like when the next message begins.
// Does nothing while a message is under way.
parser_trim :: (p: *Parser, limit: int) {
    if p.started return;

    freed := false;

    if p.bytes.allocated > limit {
        array_reset(*p.bytes);
        freed = true;
    }

    if p.spans.allocated * size_of(Span) > limit {
        array_reset(*p.spans);
        freed = true;
    }

    if p.list.allocated * size_of(Header_Field) > limit {
        array_reset(*p.list);
        freed = true;
    }

    if p.tbytes.allocated > limit {
        array_reset(*p.tbytes);
        freed = true;
    }

    if p.tspans.allocated * size_of(Span) > limit {
        array_reset(*p.tspans);
        freed = true;
    }

    if p.tlist.allocated * size_of(Header_Field) > limit {
        array_reset(*p.tlist);
        freed = true;
    }

    if freed {
        p.head = .{};
        p.trailers = .[];
    }
}

// True when no message has begun: the connection may close without it being an error.
parser_idle :: (p: *Parser) -> bool {
    if p.state != .START return false;

    return !p.started || (!p.in_headers && p.bytes.count == 0);
}

// Drops the first `count` bytes of `data`.
advance :: inline (data: [] u8, count: int) -> [] u8 {
    result: [] u8;
    result.count = data.count - count;
    result.data = data.data + count;

    return result;
}

// Reads from `data` and returns the number of bytes used and one event. Call again with the
// rest until NEED_MORE (the input is then used up); END and HEAD can come with no bytes used, so
// loop on the event, not on the byte count. BODY data is a slice of `data`.
parse :: (p: *Parser, data: [] u8) -> consumed: int, event: Event {
    event: Event;

    if p.state == {
        case .FAILED;
            event.kind = .ERROR;

            return 0, event;

        case .SWITCHED;
            event.kind = .SWITCHED;

            return 0, event;

        case .DONE;
            if data.count == 0 {
                event.kind = .NEED_MORE;

                return 0, event;
            }

            return 0, fail(p, .DATA_AFTER_CLOSE, "data after a message that ends the connection");

        case .START;
            return parse_head(p, data);

        case .BODY_NONE;
            return 0, finish_message(p);

        case .BODY_LENGTH;
            if data.count == 0 {
                event.kind = .NEED_MORE;

                return 0, event;
            }

            n := data.count;
            if n > p.remaining n = cast(int) p.remaining;

            event.kind = .BODY;
            event.data.count = n;
            event.data.data = data.data;
            p.remaining -= n;
            p.total += n;

            if p.remaining == 0 p.state = .BODY_NONE;

            return n, event;

        case .BODY_CLOSE;
            if data.count == 0 {
                event.kind = .NEED_MORE;

                return 0, event;
            }

            if p.limits.max_body_size >= 0 && p.total + data.count > p.limits.max_body_size {
                return 0, fail(p, .BODY_TOO_LARGE, "body is over the size limit");
            }

            p.total += data.count;
            event.kind = .BODY;
            event.data = data;

            return data.count, event;
    }

    return parse_chunked(p, data);
}

// The connection has closed. Ends a body that runs until close; reports CLOSED when it fell
// between messages and an UNEXPECTED_EOF error when it fell inside one.
parse_eof :: (p: *Parser) -> Event {
    event: Event;

    if p.state == {
        case .FAILED;
            event.kind = .ERROR;

            return event;

        case .SWITCHED;
            event.kind = .CLOSED;

            return event;

        case .DONE;
            event.kind = .CLOSED;

            return event;

        case .BODY_NONE;
            return finish_message(p);

        case .BODY_CLOSE;
            return finish_message(p);

        case .START;
            if parser_idle(p) {
                event.kind = .CLOSED;

                return event;
            }
    }

    return fail(p, .UNEXPECTED_EOF, "the connection closed inside a message");
}

#scope_module

fail :: (p: *Parser, kind: Parse_Error_Kind, message: string) -> Event {
    p.state = .FAILED;
    p.error.kind = kind;
    p.error.message = message;
    p.error.offset = p.offset;

    event: Event;
    event.kind = .ERROR;

    return event;
}

finish_message :: (p: *Parser) -> Event {
    event: Event;
    event.kind = .END;

    switches := false;

    if p.kind == .RESPONSE {
        if p.head.status == 101 switches = true;
        if p.response_to == .CONNECT && p.head.status >= 200 && p.head.status < 300 switches = true;
    }

    p.started = false;
    p.in_headers = false;
    p.offset = 0;

    if switches p.state = .SWITCHED;
    else if !p.head.keep_alive p.state = .DONE;
    else p.state = .START;

    return event;
}

append_bytes :: (to: *[..] u8, source: *u8, count: int) {
    if count <= 0 return;

    needed := to.count + count;

    if needed > to.allocated {
        grown := to.allocated * 2;
        if grown < needed grown = needed;
        if grown < 512 grown = 512;

        array_reserve(to, grown);
    }

    memcpy(to.data + to.count, source, count);
    to.count = needed;
}

view :: inline (base: *u8, start: int, end: int) -> string {
    result: string;
    result.data = base + start;
    result.count = end - start;

    return result;
}

begin_message :: (p: *Parser) {
    p.started = true;
    p.in_headers = false;
    p.bytes.count = 0;
    p.spans.count = 0;
    p.list.count = 0;
    p.tbytes.count = 0;
    p.tspans.count = 0;
    p.tlist.count = 0;
    p.line_start = 0;
    p.skipped_lines = 0;
    p.offset = 0;
    p.total = 0;
    p.head = .{};
    p.trailers.count = 0;
    p.content_length_seen = false;
    p.content_length = 0;
    p.encoding_seen = false;
    p.chunked = false;
    p.other_coding = false;
    p.host_count = 0;
    p.close = false;
    p.keep_alive_token = false;
    p.upgrade_token = false;
    p.upgrade_header = false;
    p.expect_continue = false;
}

// Appends the next line of `data` (from `pos`) to `bytes`. Returns 1 when the line is complete,
// 0 when `data` ran out first, and -1 after recording an error. `too_large` is the error for a
// line over max_line_size.
collect_line :: (p: *Parser, data: [] u8, pos: *int, bytes: *[..] u8, line_start: int, too_large: Parse_Error_Kind, total_limit: int) -> int {
    i := pos.*;
    newline, bad := find_line_end(data, i, bytes.count > line_start && bytes.data[bytes.count - 1] == #char "\r");

    if bad {
        fail(p, .BAD_LINE_ENDING, "a line must end in CRLF, and CR and LF may not appear alone");

        return -1;
    }

    end := data.count;
    if newline >= 0 end = newline + 1;

    n := end - i;
    content := bytes.count - line_start + n;

    if newline >= 0 content -= 2; // the CRLF
    else content -= 1; // a CR may be waiting for its LF

    if content > p.limits.max_line_size {
        fail(p, too_large, "line is over the length limit");

        return -1;
    }

    if bytes.count + n > total_limit {
        fail(p, .HEADERS_TOO_LARGE, "head is over the size limit");

        return -1;
    }

    append_bytes(bytes, data.data + i, n);
    pos.* = end;

    if newline < 0 return 0;

    return 1;
}

parse_head :: (p: *Parser, data: [] u8) -> consumed: int, event: Event {
    event: Event;

    if !p.started {
        if data.count == 0 {
            event.kind = .NEED_MORE;

            return 0, event;
        }

        begin_message(p);
    }

    pos := 0;

    while true {
        if pos >= data.count {
            event.kind = .NEED_MORE;

            return data.count, event;
        }

        too_large := Parse_Error_Kind.HEADERS_TOO_LARGE;
        if !p.in_headers && p.kind == .REQUEST too_large = .URI_TOO_LONG;

        before := p.bytes.count;
        status := collect_line(p, data, *pos, *p.bytes, p.line_start, too_large, p.limits.max_head_size);

        if status < 0 {
            event.kind = .ERROR;

            return pos, event;
        }

        p.offset += p.bytes.count - before;

        if status == 0 {
            // The line is not finished, but its first byte may already be wrong.
            if p.in_headers && is_ows(p.bytes[p.line_start]) {
                fail(p, .OBS_FOLD, "line folding is not accepted");
                event.kind = .ERROR;

                return pos, event;
            }

            continue;
        }

        done := head_line(p);

        if done < 0 {
            event.kind = .ERROR;

            return pos, event;
        }

        if done > 0 {
            event.kind = .HEAD;

            return pos, event;
        }
    }

    return pos, event;
}

// Handles the line just collected. 1 when the head is complete, 0 for more, -1 on an error.
head_line :: (p: *Parser) -> int {
    count := p.bytes.count;

    if count - p.line_start < 2 || p.bytes[count - 2] != #char "\r" {
        fail(p, .BAD_LINE_ENDING, "a line must end in CRLF");

        return -1;
    }

    start := p.line_start;
    end := count - 2;

    if !p.in_headers {
        if start == end {
            if p.limits.skip_leading_empty_lines && p.skipped_lines < 64 {
                p.skipped_lines += 1;
                p.bytes.count = start;

                return 0;
            }

            fail(p, .BAD_START_LINE, "the message starts with an empty line");

            return -1;
        }

        ok: bool;

        if p.kind == .REQUEST ok = parse_request_line(p, start, end);
        else ok = parse_status_line(p, start, end);

        if !ok return -1;

        p.in_headers = true;
        p.line_start = count;

        return 0;
    }

    if start == end return finish_head(p);

    if p.spans.count >= p.limits.max_headers {
        fail(p, .TOO_MANY_HEADERS, "too many header lines");

        return -1;
    }

    span: Span;

    if !parse_field_line(p, p.bytes.data, start, end, *span, false) return -1;

    span.id = lookup_header_name(view(p.bytes.data, span.name_start, span.name_end));

    if !note_header(p, span.id, view(p.bytes.data, span.value_start, span.value_end)) return -1;

    array_add(*p.spans, span);
    p.line_start = count;

    return 0;
}

// name ":" OWS value OWS, in base[start, end). Refuses obs-fold, whitespace before the colon, a
// name that is not a token and control bytes in the value.
parse_field_line :: (p: *Parser, base: *u8, start: int, end: int, span: *Span, trailer: bool) -> bool {
    if is_ows(base[start]) {
        if trailer fail(p, .BAD_TRAILER, "a trailer line starts with whitespace");
        else fail(p, .OBS_FOLD, "line folding is not accepted");

        return false;
    }

    i := start;

    while i < end && is_token_char(base[i]) i += 1;

    if i == start || i >= end || base[i] != #char ":" {
        if trailer {
            fail(p, .BAD_TRAILER, "a trailer needs a token name and a colon");
        } else if i < end && i > start && is_ows(base[i]) {
            fail(p, .WHITESPACE_BEFORE_COLON, "whitespace before the colon of a header");
        } else {
            fail(p, .BAD_HEADER_NAME, "a header needs a token name and a colon");
        }

        return false;
    }

    span.name_start = start;
    span.name_end = i;

    v := i + 1;

    while v < end && is_ows(base[v]) v += 1;

    e := end;

    while e > v && is_ows(base[e - 1]) e -= 1;

    if find_bad_value_byte(base, v, e) >= 0 {
        if trailer fail(p, .BAD_TRAILER, "a trailer value holds a control byte");
        else fail(p, .BAD_HEADER_VALUE, "a header value holds a control byte");

        return false;
    }

    span.value_start = v;
    span.value_end = e;

    return true;
}

parse_version :: (p: *Parser, text: string) -> bool {
    if text.count != 8 || !begins_with(text, "HTTP/") || text[6] != #char "." || !is_digit(text[5]) || !is_digit(text[7]) {
        fail(p, .BAD_VERSION, "the version must look like HTTP/1.1");

        return false;
    }

    if text[5] != #char "1" || text[7] > #char "1" {
        fail(p, .VERSION_NOT_SUPPORTED, "only HTTP/1.0 and HTTP/1.1 are supported");

        return false;
    }

    if text[7] == #char "0" p.version = .HTTP_1_0;
    else p.version = .HTTP_1_1;

    return true;
}

parse_request_line :: (p: *Parser, start: int, end: int) -> bool {
    base := p.bytes.data;

    first := find_byte(base, start, end, #char " ");

    if first < 0 {
        fail(p, .BAD_START_LINE, "the request line needs a method, a target and a version");

        return false;
    }

    if first == start {
        fail(p, .BAD_METHOD, "the request line starts with a space");

        return false;
    }

    for start..first - 1 {
        if !is_token_char(base[it]) {
            fail(p, .BAD_METHOD, "the method is not a token");

            return false;
        }
    }

    second := find_byte(base, first + 1, end, #char " ");

    if second < 0 {
        fail(p, .BAD_START_LINE, "the request line needs a method, a target and a version");

        return false;
    }

    if second == first + 1 {
        fail(p, .BAD_TARGET, "the request target is empty");

        return false;
    }

    if find_bad_target_byte(base, first + 1, second) >= 0 {
        fail(p, .BAD_TARGET, "the request target holds a space, a control byte or a byte over 0x7e");

        return false;
    }

    if !parse_version(p, view(base, second + 1, end)) return false;

    p.method_start = start;
    p.method_end = first;
    p.target_start = first + 1;
    p.target_end = second;
    p.method = parse_method(view(base, start, first));

    return check_target_form(p);
}

// RFC 9112 3.2: origin-form, absolute-form, authority-form (CONNECT only) and asterisk-form
// (OPTIONS only).
check_target_form :: (p: *Parser) -> bool {
    target := view(p.bytes.data, p.target_start, p.target_end);

    if find_byte(target.data, 0, target.count, #char "#") >= 0 {
        fail(p, .BAD_TARGET, "the request target holds a fragment");

        return false;
    }

    form := Target_Form.NONE;

    if p.method == .CONNECT {
        form = .AUTHORITY;

        colon := -1;

        for c: target {
            if c == #char "/" || c == #char "?" || c == #char "@" {
                form = .NONE;

                break;
            }

            if c == #char ":" colon = it_index;
        }

        port := target.count - colon - 1;

        if form == .AUTHORITY && (colon < 1 || port < 1 || port > 5) form = .NONE;

        if form == .AUTHORITY {
            for slice(target, colon + 1, port) {
                if !is_digit(it) form = .NONE;
            }
        }
    } else if target[0] == #char "/" {
        form = .ORIGIN;
    } else if target == "*" {
        if p.method == .OPTIONS form = .ASTERISK;
    } else {
        // scheme ":" "//"
        i := 0;

        if (target[0] >= #char "a" && target[0] <= #char "z") || (target[0] >= #char "A" && target[0] <= #char "Z") {
            i = 1;

            while i < target.count {
                c := target[i];

                if !((c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || is_digit(c) || c == #char "+" || c == #char "-" || c == #char ".") break;

                i += 1;
            }

            if i + 2 < target.count && target[i] == #char ":" && target[i + 1] == #char "/" && target[i + 2] == #char "/" form = .ABSOLUTE;
        }
    }

    if form == .NONE {
        fail(p, .BAD_TARGET, "the request target does not fit the method");

        return false;
    }

    p.head.target_form = form;

    return true;
}

parse_status_line :: (p: *Parser, start: int, end: int) -> bool {
    base := p.bytes.data;

    if end - start < 12 || base[start + 8] != #char " " {
        // "HTTP/1.1 200": at least the version, SP and three digits
        if end - start < 12 {
            if end - start >= 5 && !begins_with(view(base, start, end), "HTTP/") {
                fail(p, .BAD_START_LINE, "not an HTTP response");

                return false;
            }

            fail(p, .BAD_START_LINE, "the status line needs a version and a three-digit status");

            return false;
        }

        fail(p, .BAD_START_LINE, "the version must be followed by one space");

        return false;
    }

    if !parse_version(p, view(base, start, start + 8)) return false;

    c0 := base[start + 9];
    c1 := base[start + 10];
    c2 := base[start + 11];

    if !is_digit(c0) || !is_digit(c1) || !is_digit(c2) || c0 < #char "1" || c0 > #char "5" {
        fail(p, .BAD_STATUS, "the status must be three digits from 100 to 599");

        return false;
    }

    p.status = (cast(int) c0 - #char "0") * 100 + (cast(int) c1 - #char "0") * 10 + (cast(int) c2 - #char "0");

    reason_start := start + 12;
    reason_end := end;

    if end > start + 12 {
        if base[start + 12] != #char " " {
            fail(p, .BAD_STATUS, "the status must be followed by a space");

            return false;
        }

        reason_start = start + 13;

        if find_bad_value_byte(base, reason_start, reason_end) >= 0 {
            fail(p, .BAD_STATUS, "the reason phrase holds a control byte");

            return false;
        }
    }

    p.reason_start = reason_start;
    p.reason_end = reason_end;

    return true;
}

// Parses one Content-Length value: 1*DIGIT that fits 63 bits.
parse_content_length :: (value: string) -> s64, bool {
    if value.count == 0 return 0, false;

    result: s64;

    for c: value {
        if !is_digit(c) return 0, false;

        digit := cast(s64) (c - #char "0");

        if result > (0x7fff_ffff_ffff_ffff - digit) / 10 return 0, false;

        result = result * 10 + digit;
    }

    return result, true;
}

// The head is complete: build the strings, then decide the framing (RFC 9112 6.3) and the
// connection behavior. 1 for done, -1 after an error.
finish_head :: (p: *Parser) -> int {
    base := p.bytes.data;

    p.list.count = 0;
    array_reserve(*p.list, p.spans.count);

    for span: p.spans {
        header: Header_Field;
        header.name = view(base, span.name_start, span.name_end);
        header.value = view(base, span.value_start, span.value_end);
        header.id = span.id;

        array_add(*p.list, header);
    }

    head := *p.head;
    head.version = p.version;
    head.headers = p.list;
    head.content_length = -1;

    if p.kind == .REQUEST {
        head.method = p.method;
        head.method_text = view(base, p.method_start, p.method_end);
        head.target = view(base, p.target_start, p.target_end);
    } else {
        head.status = p.status;
        head.reason = view(base, p.reason_start, p.reason_end);
        head.informational = p.status < 200 && p.status != 101;
    }

    content_length_seen := p.content_length_seen;
    content_length := p.content_length;
    encoding_seen := p.encoding_seen;
    chunked := p.chunked;
    other_coding := p.other_coding;
    host_count := p.host_count;
    close := p.close;
    keep_alive := p.keep_alive_token;
    upgrade_token := p.upgrade_token;
    upgrade_header := p.upgrade_header;
    expect_continue := p.expect_continue;

    if p.kind == .REQUEST {
        if host_count > 1 {
            fail(p, .DUPLICATE_HOST, "more than one Host header");

            return -1;
        }

        if host_count == 0 && p.version == .HTTP_1_1 && p.limits.require_host {
            fail(p, .MISSING_HOST, "an HTTP/1.1 request needs a Host header");

            return -1;
        }
    }

    if encoding_seen {
        if content_length_seen {
            fail(p, .CONTENT_LENGTH_WITH_TRANSFER_ENCODING, "Content-Length and Transfer-Encoding together");

            return -1;
        }

        if p.version == .HTTP_1_0 {
            fail(p, .BAD_TRANSFER_ENCODING, "Transfer-Encoding does not exist in HTTP/1.0");

            return -1;
        }

        if !chunked {
            fail(p, .BAD_TRANSFER_ENCODING, "Transfer-Encoding must end in chunked");

            return -1;
        }

        if other_coding {
            fail(p, .UNSUPPORTED_TRANSFER_ENCODING, "only the chunked transfer coding is supported");

            return -1;
        }
    }

    // Framing.
    kind := Body_Kind.NONE;
    no_body := false;

    if p.kind == .RESPONSE {
        status := p.status;

        if status < 200 || status == 204 || status == 304 no_body = true;
        if p.response_to == .HEAD no_body = true;
        if p.response_to == .CONNECT && status >= 200 && status < 300 no_body = true;
    }

    if content_length_seen head.content_length = content_length;

    if no_body {
        kind = .NONE;
    } else if encoding_seen {
        kind = .CHUNKED;
    } else if content_length_seen {
        if content_length > 0 kind = .LENGTH;
    } else if p.kind == .RESPONSE {
        kind = .UNTIL_CLOSE;
    }

    if kind == .LENGTH && p.limits.max_body_size >= 0 && content_length > p.limits.max_body_size {
        fail(p, .BODY_TOO_LARGE, "Content-Length is over the body size limit");

        return -1;
    }

    head.body = kind;

    if p.version == .HTTP_1_1 head.keep_alive = !close;
    else head.keep_alive = keep_alive && !close;

    if kind == .UNTIL_CLOSE head.keep_alive = false;

    head.upgrade = upgrade_token && upgrade_header;
    head.expect_continue = expect_continue && p.version == .HTTP_1_1 && kind != .NONE;

    if kind == {
        case .NONE;
            p.state = .BODY_NONE;

        case .LENGTH;
            p.state = .BODY_LENGTH;
            p.remaining = content_length;

        case .CHUNKED;
            p.state = .CHUNK_SIZE;
            p.chunk_size = 0;
            p.chunk_digits = 0;

        case .UNTIL_CLOSE;
            p.state = .BODY_CLOSE;
    }

    return 1;
}

// Looks at a header as its line arrives: framing, connection and Host rules that can fail at
// once, so a bad message is refused without waiting for the rest of its head.
note_header :: (p: *Parser, id: Header_Name, value: string) -> bool {
    if id == {
        case .CONTENT_LENGTH;
            if p.content_length_seen {
                fail(p, .DUPLICATE_CONTENT_LENGTH, "Content-Length appears twice");

                return false;
            }

            parsed, ok := parse_content_length(value);

            if !ok {
                fail(p, .BAD_CONTENT_LENGTH, "Content-Length must be digits that fit 63 bits");

                return false;
            }

            p.content_length_seen = true;
            p.content_length = parsed;

        case .TRANSFER_ENCODING;
            p.encoding_seen = true;

            if !scan_transfer_encoding(value, *p.chunked, *p.other_coding) {
                fail(p, .BAD_TRANSFER_ENCODING, "Transfer-Encoding is empty, repeats chunked or does not end in chunked");

                return false;
            }

        case .HOST;
            p.host_count += 1;

            if p.kind == .REQUEST && p.host_count > 1 {
                fail(p, .DUPLICATE_HOST, "more than one Host header");

                return false;
            }

            if !host_value_ok(value) {
                fail(p, .BAD_HEADER_VALUE, "Host is not a host name");

                return false;
            }

        case .CONNECTION;
            if has_token(value, "close") p.close = true;
            if has_token(value, "keep-alive") p.keep_alive_token = true;
            if has_token(value, "upgrade") p.upgrade_token = true;

        case .UPGRADE;
            p.upgrade_header = true;

        case .EXPECT;
            if p.kind == .REQUEST {
                if !equal_nocase(value, "100-continue") {
                    fail(p, .BAD_EXPECT, "the only expectation supported is 100-continue");

                    return false;
                }

                p.expect_continue = true;
            }
    }

    if p.content_length_seen && p.encoding_seen {
        fail(p, .CONTENT_LENGTH_WITH_TRANSFER_ENCODING, "Content-Length and Transfer-Encoding together");

        return false;
    }

    return true;
}

// Reads a Transfer-Encoding value as a list of codings. `chunked` is set when the last coding is
// chunked; `other` when it has another coding. False for an empty list element, chunked twice
// or chunked followed by another coding.
scan_transfer_encoding :: (value: string, chunked: *bool, other: *bool) -> bool {
    i := 0;

    while i <= value.count {
        end := i;

        while end < value.count && value[end] != #char "," end += 1;

        start := i;
        stop := end;

        while start < stop && is_ows(value[start]) start += 1;
        while stop > start && is_ows(value[stop - 1]) stop -= 1;

        item := slice(value, start, stop - start);

        if item.count == 0 return false;

        for item {
            if !is_token_char(it) return false;
        }

        if chunked.* return false; // something follows chunked

        if equal_nocase(item, "chunked") chunked.* = true;
        else other.* = true;

        i = end + 1;
    }

    return true;
}

// A Host value may not hold blanks, userinfo or path separators.
host_value_ok :: (value: string) -> bool {
    for c: value {
        if c == #char " " || c == 9 || c == #char "@" || c == #char "/" || c == #char "\\" || c == #char "," return false;
    }

    return true;
}

#scope_export
