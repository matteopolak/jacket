// The request body, read by the handler when it wants it. A handler that takes a `*Body` decides
// how: all at once (`read_all`, `read_json`) or piece by piece (`read`), so an upload never has to
// fit in memory. `Json(T)` and `Form(T)` are the buffered forms of the same thing.
//
//     upload :: (name: Path("name", string), body: *Body) -> Http_Error {
//         file := open(name.value);
//         buffer: [16 * 1024] u8;
//
//         while true {
//             count, error := read(body, buffer);
//             if error.kind != .NONE return error;
//             if count == 0 break;
//
//             write(file, buffer, count);
//         }
//
//         return .{};
//     }
//
//     post(*app, "/files/{name}", upload, max_body = 1 << 30);
//
// The bytes come off the connection as they arrive. `Expect: 100-continue` is answered when the
// handler first asks for bytes, so a request the handler refuses without reading is refused
// before the client sends the body. A body the handler leaves unread is read and dropped after it
// returns when it is small; otherwise the connection closes after the response.

// The body of the request being handled. Take it as a handler parameter (`body: *Body`); it is
// valid until the handler returns.
Body :: struct {
    // The length the client declared, or -1 when the body is chunked.
    content_length: s64 = -1;
}

// Copies the next bytes of the body, at most `buffer.count`, into `buffer` and returns how many.
// Zero means the body is over (an empty `buffer` also reads zero). Waits for bytes when none have arrived,
// within `config.body_timeout`. A body over the route's limit (`max_body`, else
// `config.max_body_size`) is a 413, and a client that stops sending a 408; the error stays the
// same on later calls.
read :: (body: *Body, buffer: [] u8) -> int, Http_Error {
    frame := cast(*Body_Frame) body;

    if buffer.count == 0 return 0, .{};

    if frame.carry.count == 0 {
        piece, error := take_piece(frame);

        if error.kind != .NONE return 0, error;

        if piece.count == 0 return 0, .{};

        frame.carry = piece;
    }

    count := min(buffer.count, frame.carry.count);

    memcpy(buffer.data, frame.carry.data, count);

    frame.carry = advance(frame.carry, count);

    return count, .{};
}

// The rest of the body. The bytes live in the connection's own buffer and stay valid until the
// response has been written; copy them to keep them longer. Read as `read`: over the limit is a
// 413, and a declared length over the limit is refused without reading anything.
read_all :: (body: *Body) -> [] u8, Http_Error {
    frame := cast(*Body_Frame) body;

    collected := frame.collected;

    collected.count = 0;

    first: [] u8;
    have_first := false;

    if frame.carry.count > 0 {
        first = frame.carry;
        frame.carry = .[];
        have_first = true;
        frame.hold = first;
    }

    while true {
        piece, error := take_piece(frame);

        if error.kind != .NONE {
            frame.hold = .[];

            return .[], error;
        }

        if piece.count == 0 break;

        if !have_first && collected.count == 0 {
            // Most requests: one piece, still in the read buffer, never copied. If another read
            // is needed `next_piece` copies it first (`hold`).
            first = piece;
            have_first = true;
            frame.hold = first;
        } else {
            if frame.hold.count > 0 {
                append_bytes(collected, frame.hold.data, frame.hold.count);

                frame.hold = .[];
            }

            append_bytes(collected, piece.data, piece.count);
        }
    }

    frame.hold = .[];

    if collected.count > 0 return collected.*, .{};

    return first, .{};
}

// The rest of the body decoded as JSON into a `T`, with `app.json_options`. `Content-Type:
// application/json` (or `+json`) is required (415); a body that is not JSON is a 400 and one of
// the wrong shape a 422, both naming the JSON path.
//
// The value is built in the temporary allocator, like everything a request decodes: strings and
// arrays in it are gone once the response is written, so `copy_string` what you keep.
read_json :: (body: *Body, $T: Type) -> T, Http_Error {
    frame := cast(*Body_Frame) body;
    request := frame.request;

    content_type, found := header(request, "Content-Type");

    if !found || !(media_type_is(content_type, "application/json") || media_type_has_suffix(content_type, "+json")) {
        return .{}, input_error(415, "body", "", ifx found then tprint("expected a JSON body (Content-Type: application/json), got `%`", content_type) else "expected a JSON body (Content-Type: application/json), but there was no Content-Type header");
    }

    bytes, error := read_all(body);

    if error.kind != .NONE return .{}, error;

    value, failure := from_json(T, string.{ bytes.count, bytes.data }, frame_of(request).app.json_options,, temp);

    if failure.kind != .NONE {
        status := ifx failure.kind == .SYNTAX || failure.kind == .LIMIT then 400 else 422;

        where := ifx failure.path.count then tprint("%: ", failure.path) else "";

        return .{}, input_error(status, "body", failure.path, tprint("%%, at line %, column %", where, failure.message, failure.line, failure.column));
    }

    return value, .{};
}

#scope_module

// Everything the connection loop and the reading procedures share about one request body. The
// public `Body` is its first member, so the `*Body` a handler gets is also a `*Body_Frame`.
Body_Frame :: struct {
    body: Body;

    request: *Request;
    parser: *Parser;
    link: Link;
    buffer: [] u8; // the connection's read buffer
    pending: *[] u8; // bytes read from the socket and not yet parsed
    out: *[..] u8; // the connection's response buffer (flushed before `100 Continue`)
    collected: *[..] u8; // where `read_all` joins pieces
    config: *Server_Config;

    limit: s64; // -1: none
    length: s64 = -1; // declared, or -1
    total: s64; // bytes handed to the handler

    carry: [] u8; // the rest of a piece `read` has not copied out yet
    hold: [] u8; // a piece of the read buffer `read_all` still points into

    finished: bool; // the parser reported the end of the body
    continue_pending: bool; // the client asked for `100 Continue` and has not had it
    failure: Http_Error; // sticky: the first thing that went wrong
    close: bool; // the connection must close after the response
    dead: bool; // the client is gone: nothing can be sent
}

body_of :: (request: *Request) -> *Body {
    return *frame_of(request).body.body;
}

// Bytes a handler may leave unread before the connection closes instead of reading them.
UNREAD_LIMIT :: 64 * 1024;

media_type_is :: (value: string, wanted: string) -> bool {
    return equal_nocase(media_type_of(value), wanted);
}

media_type_has_suffix :: (value: string, suffix: string) -> bool {
    media := media_type_of(value);

    return media.count > suffix.count && equal_nocase(slice(media, media.count - suffix.count, suffix.count), suffix);
}

// `application/json; charset=utf-8` is the media type `application/json`.
media_type_of :: (value: string) -> string {
    media := value;
    semicolon := find_index_from_left(media, #char ";");

    if semicolon >= 0 media = slice(media, 0, semicolon);

    while media.count > 0 && is_ows(media[media.count - 1]) media.count -= 1;

    while media.count > 0 && is_ows(media[0]) {
        media.data += 1;
        media.count -= 1;
    }

    return media;
}

refuse :: (frame: *Body_Frame, error: Http_Error) -> Http_Error {
    frame.failure = error;
    frame.close = true;

    return error;
}

too_large :: (frame: *Body_Frame) -> Http_Error {
    return refuse(frame, http_error(413, tprint("the request body is over the limit of % bytes", frame.limit)));
}

// The next piece of the body that counts against the limit: empty at the end of the body.
take_piece :: (frame: *Body_Frame) -> [] u8, Http_Error {
    if frame.failure.kind != .NONE return .[], frame.failure;

    if frame.limit >= 0 && frame.length > frame.limit return .[], too_large(frame);

    piece, error := next_piece(frame);

    if error.kind != .NONE return .[], error;

    frame.total += piece.count;

    if frame.limit >= 0 && frame.total > frame.limit return .[], too_large(frame);

    return piece, .{};
}

// The next piece of the body as the parser finds it, reading from the connection when it needs
// more. A view into the read buffer, valid until the next call.
next_piece :: no_inline (frame: *Body_Frame) -> [] u8, Http_Error {
    if frame.finished return .[], .{};

    if frame.failure.kind != .NONE return .[], frame.failure;

    sent := send_continue(frame);

    if sent.kind != .NONE return .[], sent;

    while true {
        used, event := parse(frame.parser, frame.pending.*);

        frame.pending.* = advance(frame.pending.*, used);

        if event.kind == .BODY return event.data, .{};

        if event.kind == .END {
            frame.finished = true;

            return .[], .{};
        }

        if event.kind == .NEED_MORE {
            error := fill(frame);

            if error.kind != .NONE return .[], error;

            continue;
        }

        if event.kind == .ERROR return .[], refuse(frame, parse_failure(frame.parser));

        frame.dead = true;

        return .[], refuse(frame, http_error(400, "the request body ended unexpectedly"));
    }

    return .[], .{};
}

// Answers `Expect: 100-continue` the first time the handler asks for the body, whether or not the
// client already sent the body along with the head. A request refused first never gets it.
send_continue :: no_inline (frame: *Body_Frame) -> Http_Error {
    if !frame.continue_pending return .{};

    frame.continue_pending = false;

    config := frame.config;
    write_error: Io.Io_Error;
    out := frame.out;

    if out.count > 0 {
        Io.with_deadline(within(config.write_timeout), #code {
            _, write_error = Io.write(frame.link.writer, out.*);
        });

        out.count = 0;
    }

    if write_error == .NONE {
        write_continue(out);

        Io.with_deadline(within(config.write_timeout), #code {
            _, write_error = Io.write(frame.link.writer, out.*);
        });

        out.count = 0;
    }

    if write_error != .NONE {
        frame.dead = true;

        return refuse(frame, http_error(400, "the connection closed"));
    }

    return .{};
}

// Reads from the connection into the read buffer; everything the parser had is used up.
fill :: no_inline (frame: *Body_Frame) -> Http_Error {
    config := frame.config;

    // The read below overwrites the buffer a `read_all` view points into.
    if frame.hold.count > 0 {
        append_bytes(frame.collected, frame.hold.data, frame.hold.count);

        frame.hold = .[];
    }

    sent := send_continue(frame);

    if sent.kind != .NONE return sent;

    count: s64;
    error: Io.Io_Error;

    Io.with_deadline(within(config.body_timeout), #code {
        count, error = Io.read(frame.link.reader, frame.buffer);
    });

    if error == .TIMED_OUT return refuse(frame, http_error(408, "the request body was not received in time"));

    if error != .NONE || count == 0 {
        frame.dead = true;

        return refuse(frame, http_error(400, "the connection closed before the whole body arrived"));
    }

    frame.pending.* = .{ count, frame.buffer.data };

    return .{};
}

// After the handler: reads and drops what it left of the body. False when the connection cannot
// carry another request (the client is gone, the body was refused or is too large to be worth
// reading).
finish_body :: no_inline (frame: *Body_Frame) -> bool {
    if frame.dead || frame.close return false;

    if frame.finished return true;

    // The client is waiting to be told to send it.
    if frame.continue_pending return false;

    // Do not wait for a client to send what the handler did not want.
    if frame.length >= 0 && frame.length - frame.total > UNREAD_LIMIT return false;

    left := UNREAD_LIMIT;

    while true {
        piece, error := next_piece(frame);

        if error.kind != .NONE return false;

        if frame.finished return true;

        left -= piece.count;

        if left < 0 return false;
    }

    return false;
}
