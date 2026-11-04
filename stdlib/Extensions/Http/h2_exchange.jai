// A scheduler-bound, HTTP/2-only connection scope. Keep it and its TLS config
// alive until all exchanges and close_http2_channel have completed.
Http2_Channel :: struct {
    base_url: string;
    tls: *Tls.Tls_Config;
    write_timeout: s64; // milliseconds; zero means no transport write timeout
    client: Client;
}

// Opaque independently readable/writable stream. One reader and one writer may
// run concurrently on the same Io scheduler. Release with close_exchange.
Http2_Exchange :: struct {
    state: *void;
}

// Opens a TLS HTTP/2 stream, without replay, redirects or HTTP/1 fallback. Returns
// after request headers are queued, so upload and response reads are independent.
// Call inside Io.io_run. The channel's address is its connection-pool identity.
//
// ```jai,ignore
// exchange, error := open_http2_exchange(*channel, "/example.Service/Method", headers);
// defer close_exchange(*exchange);
// ```
open_http2_exchange :: (channel: *Http2_Channel, path: string, headers: [] Header_Field) -> Http2_Exchange, Client_Error {
    if !Io.in_scheduler() return .{}, client_error(.TRANSPORT, .OTHER, "HTTP/2 exchange requires an active Io scheduler");
    if !fields_ok(headers) return .{}, client_error(.TRANSPORT, .OTHER, "invalid HTTP/2 request fields");
    client := *channel.client;
    client.base_url = channel.base_url;
    client.tls = channel.tls;
    client.transport.data = channel;
    text, error := absolute_url(client, path);
    if error.kind != .NONE return .{}, error;
    url, problem := parse_url(text);
    if problem != .NONE || !url.secure return .{}, client_error(.INVALID_URL, .NONE, "HTTP/2 exchanges require an absolute https URL");
    if !h2_offered(client, url) return .{}, client_error(.TLS, .NONE, "TLS configuration must offer h2");
    conn, link, _, failure := open_connection(client, url);
    if failure.kind != .NONE return .{}, failure;
    if !conn {
        close(link);
        return .{}, client_error(.TLS, .NONE, "peer did not negotiate h2");
    }
    defer h2_conn_release(conn);
    if conn.dead || conn.goaway_received || conn.goaway_sent return .{}, client_error(.TRANSPORT, .CONNECTION_RESET, "HTTP/2 connection is draining");
    conn.write_timeout = channel.write_timeout;
    fields: [..] Header_Field;
    fields.allocator = temp;
    array_add(*fields, .{ name = ":method", value = "POST" });
    array_add(*fields, .{ name = ":scheme", value = "https" });
    array_add(*fields, .{ name = ":authority", value = host_with_port(url) });
    array_add(*fields, .{ name = ":path", value = url.target });
    for headers {
        field := it;
        field.name = to_lower_copy(it.name,, temp);
        if h2_connection_specific(field.name) || field.name == "host" return .{}, client_error(.TRANSPORT, .OTHER, "invalid HTTP/2 exchange field");
        array_add(*fields, field);
    }
    id := conn.next_stream;
    conn.next_stream += 2;
    stream := h2_stream_new(conn, id);
    state := h2_exchange_state(stream);
    if !h2_send_headers(conn, id, fields, false) {
        exchange := Http2_Exchange.{ state };
        close_exchange(*exchange);
        return .{}, client_error(.TRANSPORT, .CONNECTION_RESET, "sending HTTP/2 request headers failed");
    }
    return .{ state }, .{};
}

// Starts an HTTP/2 response from a handler, returning a duplex stream. The handler
// must complete its work before returning; no second ordinary response is sent.
// Request and initial response fields remain borrowed until close_exchange.
//
// ```jai,ignore
// exchange, ok := accept_http2_exchange(request, Header_Field.[.{ name = "content-type", value = "application/grpc" }]);
// defer close_exchange(*exchange);
// ```
accept_http2_exchange :: (request: *Request, headers: [] Header_Field, status := 200) -> Http2_Exchange, bool {
    stream := frame_of(request).body.h2;
    if !stream || stream.exchange_taken || !fields_ok(headers) return .{}, false;
    fields: [..] Header_Field;
    fields.allocator = temp;
    array_add(*fields, .{ name = ":status", value = tprint("%", status) });
    for headers {
        field := it;
        field.name = to_lower_copy(it.name,, temp);
        if h2_connection_specific(field.name) return .{}, false;
        array_add(*fields, field);
    }
    if !h2_send_headers(stream.conn, stream.id, fields, false) return .{}, false;
    stream.exchange_taken = true;
    return .{ h2_exchange_state(stream) }, true;
}

// Waits for initial response headers. The returned fields borrow exchange storage.
// Byte EOF and status 200 alone do not establish application-protocol success.
exchange_head :: (exchange: *Http2_Exchange) -> status: int, fields: [] Header_Field, failure: Client_Error {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state return 0, .[], client_error(.TRANSPORT, .OTHER, "exchange is closed");
    stream := state.stream;
    while !stream.has_head {
        if h2_exchange_failed(stream) return 0, .[], h2_stream_failure(stream);
        error := Io.wait(*stream.changed);
        if error != .NONE return 0, .[], io_failure(error, "waiting for HTTP/2 response headers");
    }
    return stream.status, stream.head.fields, .{};
}

// Sends bytes, borrowing them until this returns. Half-close sends an empty
// END_STREAM when bytes are empty. Calls are never automatically replayed.
exchange_write :: (exchange: *Http2_Exchange, bytes: [] u8, end_stream := false) -> Client_Error {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state return client_error(.TRANSPORT, .OTHER, "exchange is closed");
    if state.writing return client_error(.TRANSPORT, .OTHER, "exchange already has a writer");
    state.writing = true;
    defer state.writing = false;
    if !h2_send_data(state.stream, bytes, end_stream) return h2_stream_failure(state.stream);
    return .{};
}

// Reads into caller memory. Zero bytes with no error means validated byte EOF.
// Window credit follows bytes consumed; no background message queue is created.
exchange_read :: (exchange: *Http2_Exchange, buffer: [] u8) -> count: int, failure: Client_Error {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state return 0, client_error(.TRANSPORT, .OTHER, "exchange is closed");
    if state.reading return 0, client_error(.TRANSPORT, .OTHER, "exchange already has a reader");
    if buffer.count == 0 return 0, client_error(.TRANSPORT, .OTHER, "exchange read requires nonempty buffer");
    state.reading = true;
    defer state.reading = false;
    stream := state.stream;
    while true {
        if h2_exchange_failed(stream) return 0, h2_stream_failure(stream);
        if state.at < state.block.count {
            count := min(buffer.count, state.block.count - state.at);
            memcpy(buffer.data, state.block.data + state.at, count);
            state.at += count;
            h2_consumed(stream, count);
            return count, .{};
        }
        if state.block.data free(state.block.data);
        state.block = .[];
        state.at = 0;
        if stream.recv.count > 0 {
            state.block = .{ stream.recv.count, stream.recv.data };
            stream.recv = [..] u8.{};
            continue;
        }
        if stream.remote_ended {
            state.read_eof = true;
            return 0, .{};
        }
        error := Io.wait(*stream.changed);
        if error != .NONE return 0, io_failure(error, "reading HTTP/2 exchange");
    }
    return 0, .{};
}

// Returns terminal fields after exchange_read reports EOF. Storage lives until
// close_exchange; an empty field list can still be complete.
exchange_trailers :: (exchange: *Http2_Exchange) -> fields: [] Header_Field, complete: bool {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state || !state.read_eof || h2_exchange_failed(state.stream) return .[], false;
    return state.stream.trailers.fields, true;
}

// Ends server output with terminal fields. No DATA may follow this operation.
exchange_finish :: (exchange: *Http2_Exchange, fields: [] Header_Field) -> bool {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state || state.writing return false;
    state.writing = true;
    defer state.writing = false;
    frame: Stream_Frame;
    frame.h2 = state.stream;
    return finish_stream(*frame.stream, fields);
}

// Cancels the stream and wakes blocked operations. Does not release the handle;
// wait for its reader/writer fibers before close_exchange.
exchange_cancel :: (exchange: *Http2_Exchange) {
    state := cast(*H2_Exchange_State) exchange.state;
    if state h2_reset_stream(state.stream.conn, state.stream, .CANCEL);
}

// Releases an exchange; unfinished directions are reset. Readers and writers
// must have returned first. Calling again on the cleared handle is safe.
close_exchange :: (exchange: *Http2_Exchange) {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state return;
    assert(!state.reading && !state.writing, "join exchange operations before closing");
    exchange_cancel(exchange);
    if state.block.data free(state.block.data);
    conn := state.stream.conn;
    h2_stream_release(state.stream);
    h2_conn_release(conn);
    free(state);
    exchange.state = null;
}

// Closes this scope's connections and waits for TLS links to stop using their
// configuration. First close all its exchanges. Does not touch other channels.
close_http2_channel :: (channel: *Http2_Channel) {
    pool := client_pool();
    closing: [..] *H2_Conn;
    closing.allocator = temp;
    for pool.h2 if it.owner == channel {
        it.refs += 1;
        array_add(*closing, it);
        h2_close_connection(it);
    }
    for closing {
        while !it.link_closed if Io.wait(*it.closed) != .NONE break;
        h2_conn_release(it);
    }
}

#scope_module

H2_Exchange_State :: struct {
    stream: *H2_Stream;
    block: [] u8;
    at: int;
    reading: bool;
    writing: bool;
    read_eof: bool;
}

h2_exchange_state :: (stream: *H2_Stream) -> *H2_Exchange_State {
    state := New(H2_Exchange_State);
    state.stream = stream;
    stream.refs += 1;
    stream.conn.refs += 1;
    return state;
}

h2_exchange_failed :: (stream: *H2_Stream) -> bool {
    return stream.reset_by_peer || stream.reset_sent || (stream.conn.dead && !stream.remote_ended);
}
