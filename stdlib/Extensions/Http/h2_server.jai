// The server side of HTTP/2: a connection that negotiated `h2` is read by the connection's fiber
// (`h2_read_loop`), and every request on it runs `handle_request` on a fiber of its own, so the
// same routes, extractors and middleware answer both protocols. A handler cannot tell which one
// it is on: the request looks like an HTTP/1.1 one with a `Host` header, the body is read through
// the same `Body`, and streamed responses go through the same `Stream`.

#scope_module

// One request being handled: the request frame (the `*Request` a handler gets points at it), the
// strings it refers to, and the stream it travels on.
H2_Server_Stream :: struct {
    frame: Request_Frame;
    stream: *H2_Stream;
    conn: *H2_Conn;
    block: Hpack_Block;
    headers: [..] Header_Field;
    cookie: [..] u8;
    collected: [..] u8;
    scratch: [..] u8;
    started: s64;
}

// Serves a connection whose ALPN chose `h2`; returns when it is over. `link` is the TLS stream.
serve_h2 :: (connection: *Connection, link: Link) {
    server := connection.server;
    config := *server.app.config;

    local: H2_Settings;

    local.max_concurrent_streams = config.h2_max_streams;
    local.initial_window_size = config.h2_window;
    local.max_header_list_size = config.max_head_size;

    conn := h2_conn_new(.SERVER, link, local);

    conn.server = server;
    Io.add(*server.finished, 1); // retained until the connection's final reference
    conn.connection = connection;
    conn.app = server.app;
    conn.write_timeout = config.write_timeout;
    conn.idle_timeout = config.idle_timeout;
    conn.body_timeout = config.body_timeout;
    conn.max_header_list = config.max_head_size;
    conn.max_headers = config.max_headers;
    conn.reader = connection.fiber;

    connection.h2 = conn;
    connection.idle = true;

    defer connection.h2 = null;

    h2_conn_start(conn, config.h2_connection_window - H2_DEFAULT_WINDOW);

    preface: [24] u8;
    error: Io.Io_Error;

    Io.with_deadline(within(config.header_timeout), #code {
        _, error = Io.read_full(link.reader, preface);
    });

    if error == .NONE && string.{ preface.count, preface.data } == H2_PREFACE {
        h2_read_loop(conn);
    } else {
        h2_terminate(conn, .PROTOCOL_ERROR);
    }

    // The reader may have been cancelled, which would make every wait below fail at once.
    conn.closing = true;

    Io.notify_all(*conn.wake_writer);
    h2_mark_dead(conn);

    joined: Io.Io_Error;

    Io.with_grace(config.shutdown_timeout, #code {
        joined = Io.wait(*conn.tasks);
    });
    if joined != .NONE Io.cancel(conn.writer);

    // Canceled operations and shielded handler cleanup retain the TLS link.
    Io.with_grace(Io.NO_DEADLINE, #code {
        Io.wait(*conn.tasks);
    });

    conn.connection = null;
    h2_conn_release(conn);
}

// A header block of a request (or its trailers) has arrived.
h2_server_headers :: (conn: *H2_Conn, id: u32, stream: *H2_Stream, block: Hpack_Block, end_stream: bool) -> H2_Error {
    owned := block;

    if stream {
        // Requests retain terminal fields until the stream is released.
        if stream.reset_sent || stream.reset_by_peer {
            hpack_block_free(*owned);
            return .NO_ERROR;
        }

        if stream.remote_ended {
            hpack_block_free(*owned);
            h2_reset_stream(conn, stream, .STREAM_CLOSED);

            return .NO_ERROR;
        }

        if !end_stream || !h2_valid_trailers(owned.fields) {
            hpack_block_free(*owned);
            h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

            return .NO_ERROR;
        }

        stream.trailers = owned;
        if stream.declared >= 0 && stream.declared != stream.received {
            h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

            return .NO_ERROR;
        }

        stream.remote_ended = true;
        Io.notify_all(*stream.changed);
        h2_check_closed(stream);

        return .NO_ERROR;
    }

    // Streams are opened by the client, with ids that only grow.
    if id % 2 == 0 || id <= conn.last_peer_stream {
        hpack_block_free(*owned);

        return .PROTOCOL_ERROR;
    }

    conn.last_peer_stream = id;

    limit := conn.local.max_concurrent_streams;

    if conn.draining || conn.goaway_sent || (limit >= 0 && conn.open_remote >= limit) {
        hpack_block_free(*owned);
        h2_refuse_stream(conn, id, .REFUSED_STREAM);

        return .NO_ERROR;
    }

    if owned.too_large || owned.fields.count > conn.max_headers {
        hpack_block_free(*owned);
        h2_refuse_stream(conn, id, .ENHANCE_YOUR_CALM);

        return .NO_ERROR;
    }

    ctx := New(H2_Server_Stream);

    ctx.conn = conn;
    ctx.block = owned;

    declared, valid := h2_build_request(ctx, conn);

    if !valid {
        h2_free_request(ctx);
        h2_refuse_stream(conn, id, .PROTOCOL_ERROR);

        return .NO_ERROR;
    }

    stream = h2_stream_new(conn, id);
    stream.declared = declared;
    stream.remote_ended = end_stream;

    if end_stream && declared > 0 {
        // The body is promised but the message ends: malformed.
        h2_free_request(ctx);
        h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

        return .NO_ERROR;
    }

    // The request fiber retains both objects through its final task bookkeeping.
    stream.refs += 1;
    conn.refs += 1;

    ctx.stream = stream;
    ctx.started = now_ms();

    conn.connection.idle = false;

    stream.fiber = Io.spawn(h2_serve_request, ctx);
    stream.has_fiber = true;

    Io.add(*conn.tasks, 1);

    return .NO_ERROR;
}

// Turns the pseudo-header and regular fields of a block into an HTTP/1-looking `Request`. Returns
// the declared body length (-1 when there is none) and whether the message is well formed
// (RFC 9113 section 8.3).
h2_build_request :: (ctx: *H2_Server_Stream, conn: *H2_Conn) -> s64, bool {
    request := *ctx.frame.request;

    method: string;
    path: string;
    authority: string;
    scheme: string;

    has_method := false;
    has_path := false;
    has_authority := false;
    has_scheme := false;
    saw_regular := false;
    has_host := false;
    declared: s64 = -1;
    cookies := 0;

    for field: ctx.block.fields {
        name := field.name;

        if !h2_field_syntax_ok(field) return -1, false;

        if name.count > 0 && name[0] == #char ":" {
            if saw_regular return -1, false;

            if name == {
                case ":method";
                    if has_method return -1, false;

                    method = field.value;
                    has_method = true;

                case ":path";
                    if has_path return -1, false;

                    path = field.value;
                    has_path = true;

                case ":authority";
                    if has_authority return -1, false;

                    authority = field.value;
                    has_authority = true;

                case ":scheme";
                    if has_scheme return -1, false;

                    scheme = field.value;
                    has_scheme = true;

                case;
                    return -1, false;
            }

            continue;
        }

        saw_regular = true;

        if h2_connection_specific(name) return -1, false;
        if name == "te" && field.value != "trailers" return -1, false;

        if name == "content-length" {
            length := h2_parse_length(field.value);

            if length < 0 || (declared >= 0 && declared != length) return -1, false;

            declared = length;
        }

        if name == "host" has_host = true;

        if name == "cookie" {
            cookies += 1;

            continue;
        }

        array_add(*ctx.headers, field);
    }

    connect := method == "CONNECT";

    if !has_method || method.count == 0 return -1, false;
    if !connect && (!has_path || !has_scheme || path.count == 0) return -1, false;

    // Several cookie fields travel as one, as an HTTP/1.1 client would have sent them.
    if cookies > 0 {
        for field: ctx.block.fields {
            if field.name != "cookie" continue;

            if ctx.cookie.count > 0 put(*ctx.cookie, "; ");

            put(*ctx.cookie, field.value);
        }

        cookie: Header_Field;

        cookie.name = "cookie";
        cookie.value = .{ ctx.cookie.count, ctx.cookie.data };
        cookie.id = .COOKIE;

        array_add(*ctx.headers, cookie);
    }

    // The authority is what `Host` says in HTTP/1.1.
    if has_authority && !has_host {
        host: Header_Field;

        host.name = "host";
        host.value = authority;
        host.id = .HOST;

        array_insert_at(*ctx.headers, host, 0);
    }

    request.method = parse_method(method);
    request.method_text = method;
    request.target = ifx connect && !has_path then authority else path;
    request.version = .HTTP_1_1;
    request.headers = ctx.headers;
    request.peer = conn.link.peer;

    return declared, true;
}

h2_free_request :: (ctx: *H2_Server_Stream) {
    hpack_block_free(*ctx.block);

    array_reset(*ctx.headers);
    array_reset(*ctx.cookie);
    array_reset(*ctx.collected);
    array_reset(*ctx.scratch);

    free(ctx);
}

// The fiber of one request.
h2_serve_request :: (ctx: *H2_Server_Stream) {
    stream := ctx.stream;
    conn := ctx.conn;
    server := conn.server;
    app := server.app;
    config := *app.config;

    request := *ctx.frame.request;
    frame := *ctx.frame;

    body := *frame.body;

    body.request = request;
    body.h2 = stream;
    body.collected = *ctx.collected;
    body.config = config;
    body.limit = config.max_body_size;
    body.length = stream.declared;
    body.body.content_length = stream.declared;

    split_target(request);

    response := handle_request(app, request);

    if response.source && !conn.dead {
        problem: Http_Error;

        if !open_source(request, response.source, *problem) response = error_response(request, problem);
    }

    if !conn.dead && !stream.reset_by_peer && !stream.reset_sent {
        server.served += 1;

        if !stream.exchange_taken h2_send_response(ctx, *response);

        if config.log_requests {
            print("% % % (% ms)\n", request.method_text, request.target, response.status, now_ms() - ctx.started);
        }
    } else if response.source {
        close_source(response.source);
    }

    h2_release_pieces(body, true);

    // An unfinished message is cancelled: the request body, if the handler did not want it, or a
    // response that could not be completed.
    if !stream.local_ended {
        h2_reset_stream(conn, stream, .INTERNAL_ERROR);
    } else if !stream.remote_ended {
        h2_reset_stream(conn, stream, .NO_ERROR);
    }

    stream.has_fiber = false;

    h2_stream_release(stream);
    h2_free_request(ctx);

    if conn.connection conn.connection.idle = conn.open_remote == 0;

    reset_temporary_storage();

    Io.done(*conn.tasks);
    h2_conn_release(conn);
}

// Sends the head and body of `response` on the stream.
h2_send_response :: (ctx: *H2_Server_Stream, response: *Response) {
    stream := ctx.stream;
    conn := ctx.conn;
    config := *conn.server.app.config;
    request := *ctx.frame.request;

    fields: [..] Header_Field;
    fields.allocator = temp;

    status: Header_Field;

    status.name = ":status";
    status.value = tprint("%", response.status);

    array_add(*fields, status);

    if config.server_name.count > 0 array_add(*fields, .{ name = "server", value = config.server_name });

    array_add(*fields, .{ name = "date", value = http_date_now(*conn.server.dates) });

    head_only := request.method == .HEAD;
    length: s64 = response.body.count;

    if response.source length = response.source.length;

    for response.headers {
        name := to_lower_copy(it.name,, temp);

        if name == "content-length" {
            // A HEAD answer may say how long the body would be.
            if head_only && response.body.count == 0 && !response.source {
                stated := h2_parse_length(it.value);

                if stated >= 0 length = stated;
            }

            continue;
        }

        if h2_connection_specific(name) continue;

        field := it;

        field.name = name;
        field.id = lookup_header_name(name);

        array_add(*fields, field);
    }

    allowed := body_allowed(response.status);
    has_body := allowed && !head_only && (response.source != null || response.body.count > 0);

    if allowed && length >= 0 {
        array_add(*fields, .{ name = "content-length", value = tprint("%", length), id = .CONTENT_LENGTH });
    }

    if !h2_send_headers(conn, stream.id, fields, !has_body) return;

    if !has_body {
        stream.local_ended = true;
        h2_check_closed(stream);

        if response.source close_source(response.source);

        return;
    }

    if response.source {
        h2_send_source(ctx, response.source);

        return;
    }

    h2_send_data(stream, response.body, true);
}

// Produces a streamed or file response into the stream, and ends it.
h2_send_source :: (ctx: *H2_Server_Stream, source: *Response_Source) {
    stream := ctx.stream;
    config := *ctx.conn.server.app.config;

    frame: Stream_Frame;

    frame.out = *ctx.scratch;
    frame.write_timeout = config.write_timeout;
    frame.h2 = stream;

    ok := true;

    if source.file.count {
        ok = send_file(*frame, source);

        close_source(source);
    } else {
        source.produce(*frame.stream, source.data);

        ok = !frame.failed;
    }

    if ok && !stream.local_ended h2_send_data(stream, .[], true);
}

// Takes the next DATA of a request body for `Body`. The data is handed over as it stands in the
// stream, in a block that lives until the request ends (or until the reader is done with it).
h2_next_piece :: (frame: *Body_Frame) -> [] u8, Http_Error {
    stream := frame.h2;
    conn := stream.conn;

    h2_release_pieces(frame, false);

    while true {
        if stream.recv.count > 0 {
            piece: [] u8 = .{ stream.recv.count, stream.recv.data };

            array_add(*frame.held, piece);

            stream.recv = .{};

            h2_consumed(stream, piece.count);

            return piece, .{};
        }

        if stream.remote_ended {
            frame.finished = true;

            return .[], .{};
        }

        if stream.reset_by_peer || stream.reset_sent || conn.dead {
            frame.dead = true;

            return .[], refuse(frame, http_error(400, "the request was cancelled before the whole body arrived"));
        }

        error: Io.Io_Error;

        Io.with_deadline(within(frame.config.body_timeout), #code {
            error = Io.wait(*stream.changed);
        });

        if error == .TIMED_OUT return .[], refuse(frame, http_error(408, "the request body was not received in time"));

        if error != .NONE {
            frame.dead = true;

            return .[], refuse(frame, http_error(400, "the connection closed before the whole body arrived"));
        }
    }

    return .[], .{};
}

// Frees the blocks of body data the handler is done with: all of them at the end of the request,
// otherwise those no pending view points into.
h2_release_pieces :: (frame: *Body_Frame, all: bool) {
    if !frame.h2 return;

    index := 0;

    while index < frame.held.count {
        block := frame.held[index];

        if !all && (h2_points_into(frame.hold, block) || h2_points_into(frame.carry, block)) {
            index += 1;

            continue;
        }

        free(block.data);
        array_ordered_remove_by_index(*frame.held, index);
    }

    if all array_reset(*frame.held);
}

// Whether `view` is (part of) `block`.
h2_points_into :: (view: [] u8, block: [] u8) -> bool {
    if view.count == 0 return false;

    return view.data >= block.data && view.data < block.data + block.count;
}
