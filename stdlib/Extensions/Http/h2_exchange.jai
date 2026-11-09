// A scheduler-bound, HTTP/2-only connection scope. Keep it and its TLS config
// alive until all exchanges and close_http2_channel have completed.
//
// ```jai,skip_on(wasm)
// channel: Http2_Channel;
// channel.base_url = "https://example.com";
//
// assert(channel.max_connections == 8);
// assert(channel.active_exchanges == 0);
// ```
Http2_Channel :: struct {
    base_url: string;
    tls: *Tls.Tls_Config;
    max_connections := 8;
    active_exchanges: int;
    write_timeout: s64; // milliseconds; zero means no transport write timeout
    client: Client;
}

// Opaque independently readable/writable stream. One reader and one writer may
// run concurrently on the same Io scheduler. Release with close_exchange.
//
// ```jai,skip_on(wasm)
// exchange: Http2_Exchange; // a handle that holds no stream
// assert(exchange.state == null);
// assert(exchange_cancelled(*exchange));
// ```
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
    if channel.max_connections <= 0 return .{}, client_error(.TOO_LARGE, .NONE, "HTTP/2 channel connection limit must be positive");
    if !Io.in_scheduler() return .{}, client_error(.TRANSPORT, .OTHER, "HTTP/2 exchange requires an active Io scheduler");
    if !fields_ok(headers) return .{}, client_error(.TRANSPORT, .OTHER, "invalid HTTP/2 request fields");

    channel.active_exchanges += 1;
    transferred := false;

    defer if !transferred channel.active_exchanges -= 1;
    client := *channel.client;
    client.base_url = channel.base_url;
    client.tls = channel.tls;
    client.transport.data = channel;
    text, error := absolute_url(client, path);
    if error.kind != .NONE return .{}, error;

    url, problem := parse_url(text);
    if problem != .NONE || !url.secure return .{}, client_error(.INVALID_URL, .NONE, "HTTP/2 exchanges require an absolute https URL");
    if !h2_offered(client, url) return .{}, client_error(.TLS, .NONE, "TLS configuration must offer h2");

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

    size: s64;

    for fields {
        if it.name.count > H2_CLIENT_HEAD_LIMIT || it.value.count > H2_CLIENT_HEAD_LIMIT return .{}, client_error(.TOO_LARGE, .NONE, "HTTP/2 request fields exceed the channel header limit");
        size += it.name.count + it.value.count + 32;
        if size > H2_CLIENT_HEAD_LIMIT return .{}, client_error(.TOO_LARGE, .NONE, "HTTP/2 request fields exceed the channel header limit");
    }

    conn: *H2_Conn;

    defer if conn h2_conn_release(conn);

    while true {
        link: Link;
        failure: Client_Error;
        conn, link, _, failure = open_connection(client, url, channel.max_connections, http2_only = true);
        if failure.kind != .NONE return .{}, failure;

        if !conn {
            // No stream exists after failed negotiation. Release TLS without a
            // yielding close_notify before returning its resource admission.
            close(link, now = true);
            return .{}, client_error(.TLS, .NONE, "peer did not negotiate h2");
        }

        if conn.dead || conn.goaway_received || conn.goaway_sent return .{}, client_error(.TRANSPORT, .CONNECTION_RESET, "HTTP/2 connection is draining");
        // Wait before assigning an id: cancellation cannot reset an unsent id.
        if !h2_wait_for_room(conn) return .{}, client_error(.TRANSPORT, .CONNECTION_RESET, "waiting for HTTP/2 header queue room failed");
        if conn.goaway_received || conn.goaway_sent return .{}, client_error(.TRANSPORT, .CONNECTION_RESET, "HTTP/2 connection is draining");

        // Another opener can occupy the last slot while the queue wait yields.
        // Reselect before transmitting anything; this never replays a request.
        limit := conn.remote.max_concurrent_streams;
        if limit >= 0 && conn.open_local >= limit {
            h2_conn_release(conn);
            conn = null;
            continue;
        }

        break;
    }

    conn.write_timeout = channel.write_timeout;

    id := conn.next_stream;

    conn.next_stream += 2;
    stream := h2_stream_new(conn, id);
    state := h2_exchange_state(stream);
    state.channel = channel;
    transferred = true;

    if !h2_send_headers(conn, id, fields, false) {
        exchange := Http2_Exchange.{ state };
        close_exchange(*exchange);
        return .{}, client_error(.TRANSPORT, .CONNECTION_RESET, "sending HTTP/2 request headers failed");
    }

    return .{ state }, .{};
}

// Starts an HTTP/2 response from a handler, returning a duplex stream. The handler
// must complete its work before returning; no second ordinary response is sent.
// Request fields remain borrowed until close_exchange; response fields are copied.
// defer_headers permits replacement before the first write/finish commits them.
//
// ```jai,ignore
// exchange, ok := accept_http2_exchange(request, Header_Field.[.{ name = "content-type", value = "application/grpc" }]);
// defer close_exchange(*exchange);
// ```
accept_http2_exchange :: (request: *Request, headers: [] Header_Field, status := 200, defer_headers := false) -> Http2_Exchange, bool {
    stream := frame_of(request).body.h2;
    if !stream || stream.exchange_taken || h2_exchange_failed(stream) || stream.local_ended || !h2_response_fields_ok(headers) return .{}, false;

    state := h2_exchange_state(stream);
    state.response_status = status;
    state.head_pending = true;
    h2_exchange_copy_head(state, headers);
    stream.exchange_taken = true;
    exchange := Http2_Exchange.{ state };

    if !defer_headers && !h2_exchange_flush_head(state) {
        close_exchange(*exchange);
        return .{}, false;
    }

    return exchange, true;
}

// Replaces deferred server response fields before the first write or finish.
// Fields are copied; caller storage may be released when this returns. Returns
// false for a client, committed response, concurrent writer or invalid fields.
//
// ```jai,skip_on(wasm)
// // Without a server-side exchange there is nothing to replace.
// exchange: Http2_Exchange;
// assert(!exchange_response_headers(*exchange, Header_Field.[.{ name = "x-selected", value = "a" }]));
// ```
exchange_response_headers :: (exchange: *Http2_Exchange, headers: [] Header_Field) -> bool {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state || state.stream.conn.role != .SERVER || !state.head_pending || state.writing || h2_exchange_failed(state.stream) || !h2_response_fields_ok(headers) return false;
    h2_exchange_copy_head(state, headers);
    return true;
}

// Waits for initial response headers. The returned fields borrow exchange storage.
// Byte EOF and status 200 alone do not establish application-protocol success.
//
// ```jai,skip_on(wasm)
// // A handle that holds no stream has no head to wait for.
// exchange: Http2_Exchange;
// status, fields, failure := exchange_head(*exchange);
//
// assert(status == 0 && fields.count == 0);
// assert(failure.kind == .TRANSPORT && failure.detail == "exchange is closed");
// ```
exchange_head :: (exchange: *Http2_Exchange) -> status: int, fields: [] Header_Field, failure: Client_Error {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state return 0, .[], client_error(.TRANSPORT, .OTHER, "exchange is closed");

    if state.reading return 0, .[], client_error(.TRANSPORT, .OTHER, "exchange already has a reader");

    state.reading = true;
    defer state.reading = false;
    stream := state.stream;
    while !stream.has_head {
        if h2_exchange_failed(stream) return 0, .[], h2_stream_failure(stream);
        error := Io.wait(*stream.changed);
        if error != .NONE return 0, .[], io_failure(error, "waiting for HTTP/2 response headers");
    }

    return stream.status, stream.head.fields, .{};
}

// Whether the initial response block itself ended the peer's send direction.
// This fact remains stable when later DATA or terminal trailers arrive.
//
// ```jai,skip_on(wasm)
// exchange: Http2_Exchange;
// assert(!exchange_head_ended(*exchange)); // no head has arrived
// ```
exchange_head_ended :: (exchange: *Http2_Exchange) -> bool {
    state := cast(*H2_Exchange_State) exchange.state;
    return state && state.stream.has_head && state.stream.head_ended;
}

// Whether the peer completed a valid response without a stream reset. Upload
// may already have been stopped; callers can still read its final bytes/trailers.
//
// ```jai,skip_on(wasm)
// exchange: Http2_Exchange;
// assert(!exchange_response_ended(*exchange)); // no response has completed
// ```
exchange_response_ended :: (exchange: *Http2_Exchange) -> bool {
    state := cast(*H2_Exchange_State) exchange.state;
    return state && state.stream.conn.role == .CLIENT && state.stream.remote_ended && !h2_exchange_failed(state.stream);
}

// Sends bytes, borrowing them until this returns. Half-close sends an empty
// END_STREAM when bytes are empty. Calls are never automatically replayed.
//
// ```jai,skip_on(wasm)
// // A closed handle refuses to write.
// exchange: Http2_Exchange;
// failure := exchange_write(*exchange, bytes_of("ping"));
//
// assert(failure.kind == .TRANSPORT && failure.detail == "exchange is closed");
// ```
exchange_write :: (exchange: *Http2_Exchange, bytes: [] u8, end_stream := false) -> Client_Error {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state return client_error(.TRANSPORT, .OTHER, "exchange is closed");
    if state.writing return client_error(.TRANSPORT, .OTHER, "exchange already has a writer");

    state.writing = true;
    defer state.writing = false;
    if !h2_exchange_flush_head(state) return h2_stream_failure(state.stream);
    if !h2_send_data(state.stream, bytes, end_stream) return h2_stream_failure(state.stream);

    return .{};
}

// Reads into caller memory. Zero bytes with no error means validated byte EOF.
// Window credit follows bytes consumed; no background message queue is created.
//
// ```jai,skip_on(wasm)
// // A closed handle has nothing to read.
// exchange: Http2_Exchange;
// buffer: [16] u8;
// count, failure := exchange_read(*exchange, buffer);
//
// assert(count == 0);
// assert(failure.kind == .TRANSPORT && failure.detail == "exchange is closed");
// ```
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
//
// ```jai,skip_on(wasm)
// exchange: Http2_Exchange;
// fields, complete := exchange_trailers(*exchange);
//
// assert(fields.count == 0 && !complete); // not complete: the body was not read to its end
// ```
exchange_trailers :: (exchange: *Http2_Exchange) -> fields: [] Header_Field, complete: bool {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state || !state.read_eof || h2_exchange_failed(state.stream) return .[], false;
    return state.stream.trailers.fields, true;
}

// Ends server output with terminal fields. No DATA may follow this operation.
//
// ```jai,skip_on(wasm)
// exchange: Http2_Exchange;
// assert(!exchange_finish(*exchange, Header_Field.[.{ name = "grpc-status", value = "0" }]));
// ```
exchange_finish :: (exchange: *Http2_Exchange, fields: [] Header_Field) -> bool {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state || state.writing return false;

    state.writing = true;
    defer state.writing = false;
    if !h2_exchange_flush_head(state) return false;

    frame: Stream_Frame;
    frame.h2 = state.stream;

    return finish_stream(*frame.stream, fields);
}

// Cancels the stream and wakes blocked operations. Does not release the handle;
// wait for its reader/writer fibers before close_exchange.
//
// ```jai,skip_on(wasm)
// exchange: Http2_Exchange;
// exchange_cancel(*exchange); // a handle with no stream: nothing to cancel
// assert(exchange_cancelled(*exchange));
// ```
exchange_cancel :: (exchange: *Http2_Exchange) {
    state := cast(*H2_Exchange_State) exchange.state;
    if state h2_reset_stream(state.stream.conn, state.stream, .CANCEL);
}

// Whether a reset or incomplete connection failure has terminated this exchange.
// Safe to poll from cooperative server work on the owning scheduler.
//
// ```jai,skip_on(wasm)
// exchange: Http2_Exchange;
// assert(exchange_cancelled(*exchange)); // an exchange that is closed counts as cancelled
// ```
exchange_cancelled :: (exchange: *Http2_Exchange) -> bool {
    state := cast(*H2_Exchange_State) exchange.state;
    return !state || h2_exchange_failed(state.stream);
}

// Failure classification returned by exchange_terminal; NONE leaves application
// completion to the received message and trailers.
//
// ```jai,skip_on(wasm)
// kinds := Http2_Terminal_Kind.[.NONE, .RESET, .GOAWAY, .CONNECTION];
//
// exchange: Http2_Exchange;
// assert(exchange_terminal(*exchange).kind == kinds[3]); // a closed handle: the connection is gone
// ```
Http2_Terminal_Kind :: enum u8 {
    NONE;
    RESET;
    GOAWAY;
    CONNECTION;
}

// Transport failure facts for status mapping and diagnostics. not_processed
// records an explicit peer refusal; callers must never infer automatic replay.
//
// ```jai,skip_on(wasm)
// exchange: Http2_Exchange;
// terminal := exchange_terminal(*exchange);
//
// assert(terminal.kind == .CONNECTION);
// assert(terminal.code == 0 && !terminal.not_processed);
// ```
Http2_Terminal :: struct {
    kind: Http2_Terminal_Kind;
    code: u32;
    not_processed: bool; // the peer explicitly refused this stream
}

// Exposes protocol failure facts without exposing the private stream. GOAWAY's
// accepted streams may continue; only refused or failed streams are terminal.
//
// ```jai,skip_on(wasm)
// exchange: Http2_Exchange;
// assert(exchange_terminal(*exchange).kind == .CONNECTION);
// ```
exchange_terminal :: (exchange: *Http2_Exchange) -> Http2_Terminal {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state return .{ kind = .CONNECTION };

    stream := state.stream;
    conn := stream.conn;

    if stream.goaway_refused return .{ kind = .GOAWAY, code = conn.goaway_code, not_processed = true };
    if stream.reset_by_peer || stream.reset_sent return .{ kind = .RESET, code = stream.reset_code, not_processed = stream.refused };

    if conn.dead && !stream.remote_ended {
        if conn.goaway_received return .{ kind = .GOAWAY, code = conn.goaway_code };
        return .{ kind = .CONNECTION };
    }

    return .{};
}

// Releases an exchange; unfinished directions are reset. Readers and writers
// must have returned first. Calling again on the cleared handle is safe.
//
// ```jai,skip_on(wasm)
// exchange: Http2_Exchange;
//
// close_exchange(*exchange);
// close_exchange(*exchange); // safe again on the cleared handle
// assert(exchange.state == null);
// ```
close_exchange :: (exchange: *Http2_Exchange) {
    state := cast(*H2_Exchange_State) exchange.state;
    if !state return;

    assert(!state.reading && !state.writing, "join exchange operations before closing");

    if state.stream.conn.role == .SERVER && state.stream.local_ended {
        h2_reset_stream(state.stream.conn, state.stream, .NO_ERROR);
    } else exchange_cancel(exchange);

    if state.block.data free(state.block.data);

    hpack_block_free(*state.response_head);

    conn := state.stream.conn;

    h2_stream_release(state.stream);
    h2_conn_release(conn);
    if state.channel state.channel.active_exchanges -= 1;

    free(state);
    exchange.state = null;
}

// Closes this scope's connections and waits for TLS links to stop using their
// configuration. First close all its exchanges. Does not touch other channels.
//
// ```jai,skip_on(wasm)
// Io :: #import "Extensions/Io";
//
// main :: () {
//     Io.io_run(() {
//         channel: Http2_Channel;
//         channel.base_url = "https://example.com";
//
//         // Every exchange is closed first; a channel that opened none has nothing to close.
//         close_http2_channel(*channel);
//         assert(channel.active_exchanges == 0);
//     });
// }
// ```
close_http2_channel :: (channel: *Http2_Channel) {
    assert(channel.active_exchanges == 0, "close every exchange before its channel");
    pool := client_pool();
    closing: [..] *H2_Conn;
    closing.allocator = temp;

    for pool.h2 if it.owner == channel {
        it.refs += 1;
        array_add(*closing, it);
        h2_close_connection(it);
    }

    for closing {
        h2_join_closed(it);
        h2_conn_release(it);
    }
}

#scope_module

H2_Exchange_State :: struct {
    channel: *Http2_Channel;
    stream: *H2_Stream;
    block: [] u8;
    at: int;
    reading: bool;
    writing: bool;
    read_eof: bool;
    head_pending: bool;
    response_status: int;
    response_head: Hpack_Block;
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

h2_response_fields_ok :: (headers: [] Header_Field) -> bool {
    if !fields_ok(headers) return false;
    for headers if h2_connection_specific(to_lower_copy(it.name,, temp)) return false;
    return true;
}

// All data is copied before replacing the old block, including aliased inputs.
h2_exchange_copy_head :: (state: *H2_Exchange_State, headers: [] Header_Field) {
    replacement: Hpack_Block;
    size := 0;

    for headers size += it.name.count + it.value.count;

    array_reserve(*replacement.storage, size);

    for headers {
        name_at := replacement.storage.count;

        append_bytes(*replacement.storage, it.name.data, it.name.count);

        for i: name_at..replacement.storage.count - 1 {
            c := replacement.storage[i];
            if c >= #char "A" && c <= #char "Z" replacement.storage[i] = c + 32;
        }

        value_at := replacement.storage.count;

        append_bytes(*replacement.storage, it.value.data, it.value.count);
        array_add(*replacement.fields, .{ name = .{ it.name.count, replacement.storage.data + name_at }, value = .{ it.value.count, replacement.storage.data + value_at } });
    }

    hpack_block_free(*state.response_head);
    state.response_head = replacement;
}

h2_exchange_flush_head :: (state: *H2_Exchange_State) -> bool {
    if !state.head_pending return true;

    stream := state.stream;
    if !h2_wait_for_room(stream.conn, stream) return false;

    fields: [..] Header_Field;
    fields.allocator = temp;
    array_add(*fields, .{ name = ":status", value = tprint("%", state.response_status) });
    for state.response_head.fields array_add(*fields, it);
    if !h2_send_headers(stream.conn, stream.id, fields, false) return false;

    state.head_pending = false;
    hpack_block_free(*state.response_head);

    return true;
}
