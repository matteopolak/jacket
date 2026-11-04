// The client side of HTTP/2: when TLS picks `h2`, the connection becomes an `H2_Conn` that the
// pool keeps per origin, and every request to that origin is a stream on it. Requests that run
// at the same time share the connection; each response body reads from its own stream.

#scope_module

// The window a response stream starts with, and the room for all of them together.
H2_CLIENT_WINDOW :: 256 * 1024;
H2_CLIENT_CONNECTION_WINDOW :: 1024 * 1024;

// The largest response head the client accepts, and the time a write may take.
H2_CLIENT_HEAD_LIMIT :: 64 * 1024;
H2_CLIENT_WRITE_TIMEOUT :: 30_000;

// A header block of a response (or its trailers) has arrived.
h2_client_headers :: (conn: *H2_Conn, id: u32, stream: *H2_Stream, block: Hpack_Block, end_stream: bool) -> H2_Error {
    owned := block;

    if !stream {
        hpack_block_free(*owned);

        // A stream we have forgotten or reset sends its last frames: nothing to do. One we never opened is an error.
        if h2_stream_idle(conn, id) return .PROTOCOL_ERROR;

        return .NO_ERROR;
    }

    if stream.reset_sent || stream.reset_by_peer {
        hpack_block_free(*owned);

        return .NO_ERROR;
    }

    if stream.has_head {
        // Trailer blocks cannot contain pseudo headers or connection fields and
        // must end the response. Validate before transferring ownership.
        if !end_stream || stream.remote_ended || !h2_valid_trailers(owned.fields) {
            hpack_block_free(*owned);
            h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

            return .NO_ERROR;
        }

        stream.trailers = owned;
        h2_end_response(conn, stream);

        return .NO_ERROR;
    }

    status := 0;
    declared: s64 = -1;
    saw_regular := false;

    for field: owned.fields {
        if h2_has_uppercase(field.name) || h2_connection_specific(field.name) {
            hpack_block_free(*owned);
            h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

            return .NO_ERROR;
        }

        if field.name.count > 0 && field.name[0] == #char ":" {
            if field.name != ":status" || saw_regular || status != 0 {
                hpack_block_free(*owned);
                h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

                return .NO_ERROR;
            }

            status = h2_parse_length(field.value);

            continue;
        }

        saw_regular = true;

        if field.name == "content-length" declared = h2_parse_length(field.value);
    }

    if status < 100 || status > H2_MAX_STATUS {
        hpack_block_free(*owned);
        h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

        return .NO_ERROR;
    }

    // An interim response is followed by the real one.
    if status < HTTP_OK {
        hpack_block_free(*owned);

        if end_stream h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

        return .NO_ERROR;
    }

    stream.head = owned;
    stream.status = status;
    stream.declared = declared;
    stream.has_head = true;

    if end_stream {
        h2_end_response(conn, stream);

        return .NO_ERROR;
    }

    Io.notify_all(*stream.changed);

    return .NO_ERROR;
}

// The peer ended the response: check the length it promised.
h2_end_response :: (conn: *H2_Conn, stream: *H2_Stream) {
    if stream.declared >= 0 && stream.declared != stream.received && stream.status != HTTP_NOT_MODIFIED && !stream.head_only {
        h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

        return;
    }

    stream.remote_ended = true;
    Io.notify_all(*stream.changed);
    h2_check_closed(stream);
}

// ---- The pool

// Another fiber is connecting to an origin that may speak HTTP/2; requests for it wait for the answer.
H2_Connecting :: struct {
    owner: *void;
    host: string;
    port: u16;
    tls: *Tls.Tls_Config;
    done: Io.Condition;
}

// An origin that turned out to speak HTTP/1.1 only, so that requests need not wait for each other.
H2_Known_Origin :: struct {
    owner: *void;
    host: string;
    port: u16;
    tls: *Tls.Tls_Config;
    since: s64;
}

h2_same_origin :: (owner: *void, host: string, port: u16, tls: *Tls.Tls_Config, other_owner: *void, other_host: string, other_port: u16, other_tls: *Tls.Tls_Config) -> bool {
    return owner == other_owner && port == other_port && tls == other_tls && host == other_host;
}

// Whether connections for this URL offer `h2`.
h2_offered :: (client: *Client, url: Url) -> bool {
    if !url.secure return false;

    config := tls_config_of(client, url);

    if config == *default_tls return client_settings.http2;

    for config.alpn if it == "h2" return true;

    return false;
}

// An HTTP/2 connection to the URL's origin that can take another stream, or null. The request
// gets a reference to it.
h2_connection :: (client: *Client, url: Url) -> *H2_Conn {
    pool := client_pool();
    owner := client.transport.data;
    tls := tls_config_of(client, url);
    now := now_ms();

    for pool.h2 {
        if !h2_same_origin(it.owner, it.host, it.port, it.tls, owner, url.host, url.port, tls) continue;

        if it.dead || it.goaway_received || it.goaway_sent continue;

        if it.open_local == 0 && now - it.idle_since > client_settings.idle_timeout {
            h2_close_connection(it);

            continue;
        }

        limit := it.remote.max_concurrent_streams;

        if limit >= 0 && it.open_local >= limit continue;

        it.refs += 1;

        return it;
    }

    return null;
}

// Ends a connection the pool no longer wants: it flushes what it has queued and closes.
h2_close_connection :: (conn: *H2_Conn) {
    conn.closing = true;
    conn.goaway_sent = true;

    Io.notify_all(*conn.wake_writer);
    Io.cancel(conn.reader);
}

// Takes over a link on which TLS chose `h2`: starts its fibers and puts it in the pool. The
// request gets a reference.
h2_adopt :: (client: *Client, url: Url, link: Link) -> *H2_Conn {
    pool := client_pool();

    local: H2_Settings;

    local.initial_window_size = H2_CLIENT_WINDOW;
    local.max_header_list_size = H2_CLIENT_HEAD_LIMIT;

    conn := h2_conn_new(.CLIENT, link, local);

    conn.host = copy_string(url.host);
    conn.port = url.port;
    conn.tls = tls_config_of(client, url);
    conn.owner = client.transport.data;
    conn.write_timeout = H2_CLIENT_WRITE_TIMEOUT;
    conn.max_header_list = H2_CLIENT_HEAD_LIMIT;
    conn.idle_since = now_ms();

    // The pool's reference, the reader's and the request's.
    conn.refs = 3;

    array_add(*pool.h2, conn);

    conn.reader = Io.spawn(h2_client_reader, conn);

    h2_conn_start(conn, H2_CLIENT_CONNECTION_WINDOW - H2_DEFAULT_WINDOW);

    return conn;
}

// The fiber that reads a client connection until it ends.
h2_client_reader :: (conn: *H2_Conn) {
    h2_read_loop(conn);

    // Out of the pool, so nobody starts a stream on it.
    pool := client_pool();

    for pool.h2 {
        if it == conn {
            remove it;

            h2_conn_release(conn);

            break;
        }
    }

    // Streams still waiting find out; what is queued is sent, then the socket closes.
    conn.closing = true;

    Io.notify_all(*conn.wake_writer);
    h2_mark_dead(conn);

    Io.with_grace(H2_CLIENT_WRITE_TIMEOUT, #code {
        Io.wait(*conn.tasks);
    });

    close(conn.link);

    conn.link_closed = true;

    Io.notify_all(*conn.closed);
    h2_conn_release(conn);
}

// Closes the connections that have no stream in flight and waits until their links are closed, so
// that the TLS configs they were made from may be freed afterwards.
h2_close_idle :: (pool: *Client_Pool) {
    closing: [..] *H2_Conn;
    closing.allocator = temp;

    for pool.h2 {
        if it.open_local != 0 continue;

        it.refs += 1;

        array_add(*closing, it);
        h2_close_connection(it);
    }

    for closing {
        while !it.link_closed {
            if Io.wait(*it.closed) != .NONE break;
        }

        h2_conn_release(it);
    }
}

// ---- Requests

// Sends the request on a new stream of `conn` and reads the head of the response. `retry` says
// the request never reached the server's handler, so it may go to another connection.
converse_h2 :: (client: *Client, conn: *H2_Conn, method: Method, method_text: string, url: Url, headers: [] Header_Field, body: [] u8, max_response_size: int, note: Request_Note) -> Client_Response, reply_started: bool, Client_Error, retry: bool {
    if conn.dead || conn.goaway_received || conn.goaway_sent {
        return .{}, false, client_error(.TRANSPORT, .CONNECTION_RESET, "the HTTP/2 connection was closed"), true;
    }

    fields: [..] Header_Field;
    fields.allocator = temp;

    authority := host_with_port(url);

    for headers {
        if it.id == .HOST authority = it.value;
    }

    array_add(*fields, .{ name = ":method", value = method_text });
    array_add(*fields, .{ name = ":scheme", value = "https" });
    array_add(*fields, .{ name = ":authority", value = authority });
    array_add(*fields, .{ name = ":path", value = url.target });

    for headers {
        name := to_lower_copy(it.name,, temp);

        if name == "host" || name == "content-length" || h2_connection_specific(name) continue;
        if name == "te" && it.value != "trailers" continue;

        field := it;

        field.name = name;
        field.id = lookup_header_name(name);

        array_add(*fields, field);
    }

    sends_length := body.count > 0 || method == .POST || method == .PUT || method == .PATCH;

    if sends_length array_add(*fields, .{ name = "content-length", value = tprint("%", body.count), id = .CONTENT_LENGTH });

    id := conn.next_stream;

    conn.next_stream += 2;

    stream := h2_stream_new(conn, id);

    // The request's reference, which a body source takes over.
    stream.refs += 1;
    stream.head_only = method == .HEAD;

    if !h2_send_headers(conn, id, fields, body.count == 0) {
        h2_reset_stream(conn, stream, .CANCEL);
        h2_stream_release(stream);

        return .{}, false, client_error(.TRANSPORT, .CONNECTION_RESET, "the HTTP/2 connection was closed"), true;
    }

    if body.count == 0 {
        stream.local_ended = true;
    } else if !h2_send_data(stream, body, true) {
        failure := h2_stream_failure(stream);

        h2_reset_stream(conn, stream, .CANCEL);
        h2_stream_release(stream);

        return .{}, false, failure, false;
    }

    // The head of the response.
    while !stream.has_head {
        if stream.reset_by_peer || stream.reset_sent || conn.dead {
            failure := h2_stream_failure(stream);
            refused := stream.refused;

            h2_reset_stream(conn, stream, .CANCEL);
            h2_stream_release(stream);

            return .{}, false, failure, refused;
        }

        error := Io.wait(*stream.changed);

        if error != .NONE {
            h2_reset_stream(conn, stream, .CANCEL);
            h2_stream_release(stream);

            return .{}, true, io_failure(error, "waiting for the response"), false;
        }
    }

    list: [..] Header_Field;
    list.allocator = temp;

    for stream.head.fields {
        if it.name.count > 0 && it.name[0] == #char ":" continue;

        array_add(*list, it);
    }

    response := build_response(stream.status, "", list, note);

    source := take_source();

    release_buffer(source);

    parser_init(*source.parser, .RESPONSE);

    source.h2 = stream;
    source.owner = client.transport.data;
    source.max_size = max_response_size;
    source.held = true;
    source.kind = ifx stream.remote_ended && stream.recv.count == 0 then Body_Kind.NONE else ifx stream.declared >= 0 then Body_Kind.LENGTH else Body_Kind.CHUNKED;
    source.length = stream.declared;

    // The source keeps the connection alive for as long as the body can be read.
    conn.refs += 1;

    response.source = source;
    response.http2 = true;

    // Everything has arrived already (a small answer): take it into memory, and the stream is free at once.
    if stream.remote_ended {
        failure := read_all(source, *source.stored);

        if failure.kind != .NONE {
            source.deferred = failure;
            source.failed = true;
        } else {
            source.has_stored = true;
        }
    }

    return response, true, .{}, false;
}

// Why a stream ended without a response, as the error the caller gets.
h2_stream_failure :: (stream: *H2_Stream) -> Client_Error {
    conn := stream.conn;

    if stream.reset_by_peer && !stream.refused {
        return client_error(.TRANSPORT, .CONNECTION_RESET, tprint("the server reset the stream (HTTP/2 error code %)", stream.reset_code));
    }

    if conn.goaway_received return client_error(.TRANSPORT, .CONNECTION_RESET, tprint("the server closed the HTTP/2 connection (error code %)", conn.goaway_code));

    return client_error(.TRANSPORT, .CONNECTION_RESET, "the HTTP/2 connection was lost before the response arrived");
}

// Makes `source.pending` hold body bytes, waiting for the stream when it has none. True when the body has ended.
h2_fill_pending :: (source: *Body_Source) -> bool, Client_Error {
    stream := source.h2;
    conn := stream.conn;

    while source.pending.count == 0 {
        if source.finished return true, .{};

        if stream.recv.count > 0 {
            // The data stays where the connection put it, and goes back to the allocator when the next piece is taken.
            if source.h2_block.data free(source.h2_block.data);

            source.h2_block = .{ stream.recv.count, stream.recv.data };
            source.pending = source.h2_block;
            source.received += source.h2_block.count;

            stream.recv = .{};

            h2_consumed(stream, source.h2_block.count);

            return false, .{};
        }

        if stream.remote_ended {
            source.finished = true;

            connection_done(source);

            return true, .{};
        }

        if stream.reset_by_peer || stream.reset_sent || conn.dead {
            return false, h2_stream_failure(stream);
        }

        remaining := Io.NO_DEADLINE;

        if source.deadline_ms > 0 {
            remaining = source.deadline_ms - now_ms();

            if remaining <= 0 return false, io_failure(.TIMED_OUT, "reading the response body");
        }

        error: Io.Io_Error;

        Io.with_deadline(remaining, #code {
            error = Io.wait(*stream.changed);
        });

        if error != .NONE return false, io_failure(error, "reading the response body");
    }

    return false, .{};
}

// The body is over or given up: the stream goes (it is reset if the server still has more to
// say), and the source lets go of the connection.
h2_source_release :: (source: *Body_Source) {
    stream := source.h2;

    if !stream return;

    conn := stream.conn;

    if source.finished && stream.remote_ended && !stream.reset_by_peer && !stream.reset_sent {
        source.h2_trailers = stream.trailers;
        stream.trailers = .{};
        source.h2_complete = true;
    }

    source.h2 = null;
    source.held = false;

    if !h2_stream_closed(stream) h2_reset_stream(conn, stream, .CANCEL);

    h2_stream_release(stream);
    h2_conn_release(conn);
}

// A connection for a request to `url`: an HTTP/2 one with room (the request holds a reference to it), or
// a link for HTTP/1.1. Requests that arrive while the first connection to a likely HTTP/2 origin is being
// made wait for it, so that they share it instead of each making its own.
open_connection :: (client: *Client, url: Url) -> conn: *H2_Conn, link: Link, reused: bool, Client_Error {
    pool := client_pool();
    owner := client.transport.data;
    tls := tls_config_of(client, url);

    while true {
        conn := h2_connection(client, url);

        if conn return conn, .{}, true, .{};

        gate: *H2_Connecting;

        if h2_offered(client, url) && !h2_known_http1(pool, owner, url, tls) {
            waiting: *H2_Connecting;

            for pool.connecting {
                if h2_same_origin(it.owner, it.host, it.port, it.tls, owner, url.host, url.port, tls) {
                    waiting = it;

                    break;
                }
            }

            if waiting {
                error := Io.wait(*waiting.done);

                if error != .NONE return null, .{}, false, io_failure(error, tprint("waiting for the connection to %:%", url.host, url.port));

                continue;
            }

            gate = New(H2_Connecting);

            gate.owner = owner;
            gate.host = url.host;
            gate.port = url.port;
            gate.tls = tls;

            array_add(*pool.connecting, gate);
        }

        link, reused, error := checkout(client, url);

        opened: *H2_Conn;

        if error.kind == .NONE && link.h2 {
            opened = h2_adopt(client, url, link);
        } else if error.kind == .NONE && gate {
            array_add(*pool.http1_only, H2_Known_Origin.{ owner = owner, host = copy_string(url.host), port = url.port, tls = tls, since = now_ms() });
        }

        if gate {
            for pool.connecting {
                if it == gate {
                    array_unordered_remove_by_index(*pool.connecting, it_index);

                    break;
                }
            }

            Io.notify_all(*gate.done);

            free(gate);
        }

        if error.kind != .NONE return null, .{}, false, error;

        if opened return opened, .{}, false, .{};

        return null, link, reused, .{};
    }

    return null, .{}, false, .{};
}

// Whether a recent connection to the origin ended up speaking HTTP/1.1.
h2_known_http1 :: (pool: *Client_Pool, owner: *void, url: Url, tls: *Tls.Tls_Config) -> bool {
    now := now_ms();
    index := 0;
    found := false;

    while index < pool.http1_only.count {
        entry := pool.http1_only[index];

        if now - entry.since > client_settings.idle_timeout {
            free(entry.host.data);
            array_unordered_remove_by_index(*pool.http1_only, index);

            continue;
        }

        if h2_same_origin(entry.owner, entry.host, entry.port, entry.tls, owner, url.host, url.port, tls) found = true;

        index += 1;
    }

    return found;
}

H2_MAX_STATUS :: 599;
