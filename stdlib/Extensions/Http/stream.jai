// Responses whose body is produced while it is sent: a file read from disk in pieces, or bytes a
// handler pushes as it has them. Neither needs the body in memory; both go out after the head
// from the connection's own fiber, so a slow client holds nothing but that fiber.
//
// ```jai,ignore
// events :: (request: *Request) -> Response {
//     return stream_response(produce_events, null, "text/event-stream");
// }
//
// produce_events :: (stream: *Stream, data: *void) {
//     for 1..10 {
//         if !send_chunk(stream, tprint("data: %\n\n", it)) return;   // false: the client left
//         Io.sleep(1000);
//     }
// }
// ```
//
// On HTTP/1.1 a stream is sent chunked, so the connection stays usable afterwards; to an HTTP/1.0
// client it ends with the connection. The headers, the status and the `data` pointer must stay
// valid until the producer returns: use string literals or the temporary allocator.

// Where a stream's bytes come from. Called once, after the head has been sent, on the
// connection's fiber; it may wait (`Io.sleep`, channels) between pieces. Not called for a HEAD
// request.
Stream_Producer :: #type (stream: *Stream, data: *void);

// The sending end of a streamed response, valid while its producer runs.
//
// ```jai,skip_on(wasm)
// saw_chunked := false;
//
// produce :: (stream: *Stream, data: *void) {
//     saw_chunked = stream.chunked;
//     send_chunk(stream, "x");
// }
//
// events :: () -> Response {
//     return stream_response(produce, null, "text/plain");
// }
//
// main :: () {
//     app: App;
//     defer free_app(*app);
//
//     get(*app, "/events", events);
//
//     api := test_client(*app);
//     response, _ := get(api, "/events");
//     defer close(*response);
//
//     text, _ := read_text(*response);
//     defer free(text);
//
//     assert(text == "x");
//     assert(saw_chunked); // HTTP/1.1: every chunk goes out as one chunk of the body
// }
// ```
Stream :: struct {
    // True on HTTP/1.1, where every `send_chunk` is one chunk of the chunked body.
    chunked: bool;
}

// A response whose body `produce` writes with `send_chunk`. `data` is handed to it as is.
//
// - produce: writes the body; it runs after the head has been sent.
// - data: handed to `produce` untouched. It has to outlive the response.
// - content_type: the `Content-Type` header value.
// - status: the HTTP status code.
//
// ```jai,skip_on(wasm)
// name := "Ada";
//
// produce :: (stream: *Stream, data: *void) {
//     who := cast(*string) data;
//
//     send_chunk(stream, "hello ");
//     send_chunk(stream, who.*);
// }
//
// greeting :: () -> Response {
//     return stream_response(produce, *name, "text/plain");
// }
//
// main :: () {
//     app: App;
//     defer free_app(*app);
//
//     get(*app, "/greeting", greeting);
//
//     api := test_client(*app);
//     response, _ := get(api, "/greeting");
//     defer close(*response);
//
//     text, _ := read_text(*response);
//     defer free(text);
//
//     assert(response.status == 200 && text == "hello Ada");
// }
// ```
stream_response :: (produce: Stream_Producer, data: *void, content_type: string, status := 200) -> Response {
    response: Response;

    response.status = status;
    response.source = New(Response_Source,, temp);
    response.source.produce = produce;
    response.source.data = data;
    response.source.length = -1;

    add_header(*response, "Content-Type", content_type);

    return response;
}

// A response with the contents of the file at `path`, read in pieces as the client takes them
// (`Content-Length` is the file's size when the response is sent). An empty `content_type` is
// chosen from the file's extension (`application/octet-stream` when unknown). A file that cannot
// be opened is a 404, or a 500 for any other failure; a file that shrinks while it is sent cuts
// the connection.
//
// - path: the file to send.
// - content_type: the `Content-Type` header value. Empty picks one from the extension.
// - status: the HTTP status code.
//
// ```jai,skip_on(wasm)
// page :: () -> Response {
//     return file_response("/nonexistent/for/sure/index.html");
// }
//
// main :: () {
//     response := file_response("/srv/site/style.css"); // nothing is opened until it is sent
//     type, _ := find_header(response.headers, "Content-Type");
//     assert(type == "text/css; charset=utf-8"); // chosen from the extension
//
//     app: App;
//     defer free_app(*app);
//
//     get(*app, "/", page);
//
//     api := test_client(*app);
//     missing, _ := get(api, "/");
//     defer close(*missing);
//
//     assert(missing.status == 404); // a file that cannot be opened
// }
// ```
file_response :: (path: string, content_type := "", status := 200) -> Response {
    response: Response;

    response.status = status;
    response.source = New(Response_Source,, temp);
    response.source.file = path;
    response.source.length = -1;

    add_header(*response, "Content-Type", ifx content_type.count then content_type else content_type_of_path(path));

    return response;
}

// Sends `data` to the client now, as one chunk of the body. False when the client has gone or the
// write timed out: the producer should return.
//
// ```jai,skip_on(wasm)
// produce :: (stream: *Stream, data: *void) {
//     bytes: [3] u8 = .[104, 105, 33];
//
//     if !send_chunk(stream, bytes) return; // false: the client has gone
// }
//
// hi :: () -> Response {
//     return stream_response(produce, null, "text/plain");
// }
//
// main :: () {
//     app: App;
//     defer free_app(*app);
//
//     get(*app, "/hi", hi);
//
//     api := test_client(*app);
//     response, _ := get(api, "/hi");
//     defer close(*response);
//
//     text, _ := read_text(*response);
//     defer free(text);
//
//     assert(text == "hi!");
// }
// ```
send_chunk :: (stream: *Stream, data: [] u8) -> bool {
    frame := cast(*Stream_Frame) stream;

    if frame.failed return false;

    if data.count == 0 return true;

    out := frame.out;

    if frame.stream.chunked {
        put_hex(out, data.count);
        put(out, "\r\n");

        if data.count >= LARGE_BODY {
            frame.failed = !flush(frame, out.*);

            out.count = 0;

            if !frame.failed frame.failed = !flush(frame, data);
            if !frame.failed frame.failed = !flush(frame, string_bytes("\r\n"));

            return !frame.failed;
        }

        append_bytes(out, data.data, data.count);
        put(out, "\r\n");
    } else if data.count >= LARGE_BODY {
        frame.failed = !flush(frame, data);

        return !frame.failed;
    } else {
        append_bytes(out, data.data, data.count);
    }

    frame.failed = !flush(frame, out.*);

    out.count = 0;

    return !frame.failed;
}

// `send_chunk` for text.
//
// ```jai,skip_on(wasm)
// produce :: (stream: *Stream, data: *void) {
//     for 1..3 {
//         if !send_chunk(stream, tprint("tick %;", it)) return;
//     }
// }
//
// ticks :: () -> Response {
//     return stream_response(produce, null, "text/plain");
// }
//
// main :: () {
//     app: App;
//     defer free_app(*app);
//
//     get(*app, "/ticks", ticks);
//
//     api := test_client(*app);
//     response, _ := get(api, "/ticks");
//     defer close(*response);
//
//     text, _ := read_text(*response);
//     defer free(text);
//
//     assert(text == "tick 1;tick 2;tick 3;");
// }
// ```
send_chunk :: (stream: *Stream, text: string) -> bool {
    return send_chunk(stream, string_bytes(text));
}

// Ends an HTTP/2 streamed response with terminal fields. Call once, after the
// last `send_chunk`, while the producer runs. Names must be lowercase regular
// header names; fields are borrowed only until this returns. False indicates
// an invalid block, another protocol, a closed stream, or transport failure.
//
// ```jai,ignore
// finish_stream(stream, Header_Field.[.{ name = "grpc-status", value = "0" }]);
// ```
finish_stream :: (stream: *Stream, fields: [] Header_Field) -> bool {
    frame := cast(*Stream_Frame) stream;
    h2 := frame.h2;

    if frame.failed || !h2 return false;
    if h2.local_ended || h2.reset_sent || h2.reset_by_peer || !h2_valid_trailers(fields) return false;
    if !h2_wait_for_room(h2.conn, h2) return false;
    // Waiting permits the reader or another fiber to terminate the stream.
    if h2.local_ended || h2.reset_sent || h2.reset_by_peer return false;

    if !h2_send_headers(h2.conn, h2.id, fields, true) {
        frame.failed = true;
        return false;
    }

    h2.local_ended = true;
    h2_check_closed(h2);

    return true;
}

#scope_module

Response_Source :: struct {
    produce: Stream_Producer;
    data: *void;
    file: string;
    length: s64; // the size of the file, once open; -1 for a stream
    handle: Io.Io_File;
    opened: bool;
}

Stream_Frame :: struct {
    stream: Stream;
    link: Link;
    out: *[..] u8;
    write_timeout: s64;
    failed: bool;
    h2: *H2_Stream; // the body goes out as DATA frames of this stream instead of to `link`
}

string_bytes :: (text: string) -> [] u8 {
    return .{ text.count, text.data };
}

flush :: (frame: *Stream_Frame, data: [] u8) -> bool {
    if frame.h2 return h2_send_data(frame.h2, data, false);

    error: Io.Io_Error;

    Io.with_deadline(within(frame.write_timeout), #code {
        _, error = Io.write(frame.link.writer, data);
    });

    return error == .NONE;
}

// Opens the file of a `file_response` and learns its size; false, with `error` the response to
// send instead, when it cannot be.
open_source :: no_inline (request: *Request, source: *Response_Source, error: *Http_Error) -> bool {
    if source.file.count == 0 return true;

    file, open_error := Io.open_file(source.file);

    if open_error != .NONE {
        error.* = ifx open_error == .NOT_FOUND || open_error == .IS_A_DIRECTORY then http_error(404, "the file does not exist") else internal_error("the file cannot be read");

        return false;
    }

    size, size_error := Io.file_size(*file);

    if size_error != .NONE {
        Io.close(*file);

        error.* = internal_error("the file cannot be read");

        return false;
    }

    source.handle = file;
    source.opened = true;
    source.length = size;

    return true;
}

// Sends the body of a streamed response after its head (already in `out`). False when the
// connection cannot carry another request.
send_source :: no_inline (server: *Server, link: Link, out: *[..] u8, source: *Response_Source, chunked: bool, send_body: bool) -> bool {
    config := *server.app.config;

    frame: Stream_Frame;

    frame.link = link;
    frame.out = out;
    frame.write_timeout = config.write_timeout;
    frame.stream.chunked = chunked;

    // The head goes out first, with whatever the connection still owed.
    if !flush(*frame, out.*) {
        close_source(source);

        return false;
    }

    out.count = 0;

    if !send_body {
        close_source(source);

        return true;
    }

    if source.file.count {
        ok := send_file(*frame, source);

        close_source(source);

        return ok;
    }

    source.produce(*frame.stream, source.data);

    if frame.failed return false;

    if chunked {
        write_last_chunk(out);

        frame.failed = !flush(*frame, out.*);

        out.count = 0;
    }

    return !frame.failed;
}

FILE_PIECE :: 64 * 1024;

send_file :: no_inline (frame: *Stream_Frame, source: *Response_Source) -> bool {
    piece := NewArray(FILE_PIECE, u8);

    defer array_free(piece);

    left := source.length;

    while left > 0 {
        count, error := Io.read(*source.handle, .{ min(left, FILE_PIECE), piece.data });

        // The file ended early: the length is already promised.
        if error != .NONE || count == 0 return false;

        if !flush(frame, .{ count, piece.data }) return false;

        left -= count;
    }

    return true;
}

close_source :: (source: *Response_Source) {
    if source.opened {
        source.opened = false;

        Io.close(*source.handle);
    }
}

content_type_of_path :: (path: string) -> string {
    dot := -1;

    for path {
        if it == #char "." dot = it_index;
        if it == #char "/" dot = -1;
    }

    if dot < 0 return "application/octet-stream";

    extension := slice(path, dot + 1, path.count - dot - 1);

    if equal_nocase(extension, "html") || equal_nocase(extension, "htm") return "text/html; charset=utf-8";
    if equal_nocase(extension, "css") return "text/css; charset=utf-8";
    if equal_nocase(extension, "js") || equal_nocase(extension, "mjs") return "text/javascript; charset=utf-8";
    if equal_nocase(extension, "json") return "application/json";
    if equal_nocase(extension, "txt") || equal_nocase(extension, "md") return "text/plain; charset=utf-8";
    if equal_nocase(extension, "csv") return "text/csv; charset=utf-8";
    if equal_nocase(extension, "xml") return "application/xml";
    if equal_nocase(extension, "svg") return "image/svg+xml";
    if equal_nocase(extension, "png") return "image/png";
    if equal_nocase(extension, "jpg") || equal_nocase(extension, "jpeg") return "image/jpeg";
    if equal_nocase(extension, "gif") return "image/gif";
    if equal_nocase(extension, "webp") return "image/webp";
    if equal_nocase(extension, "ico") return "image/x-icon";
    if equal_nocase(extension, "wasm") return "application/wasm";
    if equal_nocase(extension, "pdf") return "application/pdf";
    if equal_nocase(extension, "woff2") return "font/woff2";

    return "application/octet-stream";
}

#scope_export
