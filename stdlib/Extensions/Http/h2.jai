// An HTTP/2 connection (RFC 9113), the part the server and the client share: the frames in and
// out, flow control, the table of streams and the rules for what each frame may do. The two
// sides plug in at the point where a header block arrives (`h2_server_headers`,
// `h2_client_headers`).
//
// Three kinds of fibers touch a connection, all on one scheduler, so no lock is needed around
// its state: a reader loop that takes frames off the wire, a writer that sends what the others
// have queued, and the fibers of the requests (handlers on the server, callers on the client).
// Nothing but the writer waits for the socket to take bytes, so a reader never stops reading
// because the peer has stopped reading, and nobody waits for a lock held across a write.
//
// Frames are appended to `conn.out` in the order they must reach the peer. That is also what
// keeps HPACK honest: a header block is encoded at the moment its frame is queued.

#scope_module

// How much of an unsent frame queue a sender tolerates before it waits for the writer.
H2_QUEUE_LIMIT :: 256 * 1024;

// Unacknowledged connection-level bytes that are returned to the peer in one WINDOW_UPDATE.
H2_UPDATE_BATCH :: 16 * 1024;

// A header block (HEADERS, CONTINUATIONs) larger than this many bytes per byte of header list limit is refused.
H2_BLOCK_FACTOR :: 4;

H2_Role :: enum u8 {
    SERVER;
    CLIENT;
}

// One request/response exchange on a connection.
H2_Stream :: struct {
    id: u32;
    conn: *H2_Conn;
    refs := 1; // the connection's table; whoever uses the stream adds one

    send_window: s64; // what we may still send
    recv_window: s64; // what the peer may still send
    unacked: s64; // bytes the user took that the peer has not been told about

    recv: [..] u8; // DATA received and not yet taken
    received: s64; // DATA bytes received in all, for the `content-length` check
    declared: s64 = -1; // the announced body length

    local_ended: bool; // we sent END_STREAM
    remote_ended: bool; // the peer sent it
    head_ended: bool; // initial response HEADERS carried END_STREAM
    reset_by_peer: bool;
    reset_sent: bool;
    reset_code: u32;
    removed: bool; // taken out of the connection's table

    changed: Io.Condition; // something happened to the stream or the connection

    // The server side: the fiber running the handler.
    fiber: Io.Fiber_Handle;
    has_fiber: bool;

    // The client side: the response head, once it arrives.
    head: Hpack_Block;
    trailers: Hpack_Block;
    status: int;
    has_head: bool;
    failure: string; // why the stream ended in an error, for the caller
    goaway_refused: bool;
    refused: bool; // the server did not process it (RST_STREAM REFUSED_STREAM or GOAWAY): the request may be retried
    exchange_taken: bool; // server handler owns the duplex response
    head_only: bool; // the request was HEAD: the response has a length and no body
}

// Everything about one HTTP/2 connection.
H2_Conn :: struct {
    role: H2_Role;
    link: Link;
    refs := 1;

    local: H2_Settings; // our limits on what we receive
    remote: H2_Settings; // the peer's limits on what we send
    encoder: Hpack_Encoder;
    decoder: Hpack_Decoder;

    streams: [..] *H2_Stream;
    last_peer_stream: u32; // the highest stream id the peer opened
    next_stream: u32 = 1; // the id of the next stream we open (client)
    open_remote: int; // streams the peer opened and that are not closed
    open_local: int; // streams we opened and that are not closed
    admission_changed: *Io.Condition; // client pool capacity; null on servers

    send_window: s64 = H2_DEFAULT_WINDOW;
    recv_window: s64 = H2_DEFAULT_WINDOW;
    recv_unacked: s64;

    out: [..] u8; // frames queued for the writer
    spare: [..] u8; // what the writer is sending
    wake_writer: Io.Condition;
    drained: Io.Condition; // the writer sent something
    tasks: Io.Wait_Group; // the writer and the handler fibers

    closing: bool; // the writer ends once it has sent what is queued
    dead: bool; // no more frames are queued or taken
    goaway_sent: bool;
    goaway_received: bool;
    goaway_last: u32;
    goaway_code: u32;
    draining: bool; // the server is shutting down: new streams are refused

    reader: Io.Fiber_Handle;
    writer: Io.Fiber_Handle;

    // The client's reader closed the link: nothing of the connection touches its TLS config any more.
    link_closed: bool;
    closed: Io.Condition;

    // A header block that continues in the next frames.
    fragment: [..] u8;
    fragment_stream: u32;
    fragment_flags: u8;
    fragment_frames: int;

    // Limits and timeouts, in milliseconds (0 or less: none).
    write_timeout: s64;
    idle_timeout: s64;
    body_timeout: s64;
    max_header_list: s64;
    max_headers: int;

    server: *Server;
    connection: *Connection;
    app: *App;

    // The client side: the pool's view of the connection.
    host: string;
    port: u16;
    tls: *Tls.Tls_Config;
    owner: *void;
    idle_since: s64;
    settings_ack_pending: int;
}

// A new connection over `link`. `local` are the limits we announce; nothing is sent yet.
h2_conn_new :: (role: H2_Role, link: Link, local: H2_Settings) -> *H2_Conn {
    conn := New(H2_Conn);

    conn.role = role;
    conn.link = link;
    conn.local = local;
    conn.next_stream = ifx role == .CLIENT then 1 else 2;

    hpack_encoder_init(*conn.encoder);
    hpack_decoder_init(*conn.decoder, local.header_table_size);

    return conn;
}

// Drops a reference; the last one frees the connection.
h2_conn_release :: (conn: *H2_Conn) {
    conn.refs -= 1;

    if conn.refs > 0 return;

    hpack_encoder_free(*conn.encoder);
    hpack_decoder_free(*conn.decoder);

    for conn.streams h2_stream_free(it);

    array_reset(*conn.streams);
    array_reset(*conn.out);
    array_reset(*conn.spare);
    array_reset(*conn.fragment);

    if conn.host.count free(conn.host.data);
    if conn.server Io.done(*conn.server.finished);

    free(conn);
}

h2_stream_free :: (stream: *H2_Stream) {
    array_reset(*stream.recv);
    hpack_block_free(*stream.head);
    hpack_block_free(*stream.trailers);

    if stream.failure.count free(stream.failure.data);

    free(stream);
}

h2_stream_release :: (stream: *H2_Stream) {
    stream.refs -= 1;

    if stream.refs == 0 h2_stream_free(stream);
}

// Queues the connection's first frames and starts its writer. The client also sends the preface.
h2_conn_start :: (conn: *H2_Conn, window_bonus: s64) {
    if conn.role == .CLIENT put(*conn.out, H2_PREFACE);

    entries: [..] H2_Setting_Entry;
    entries.allocator = temp;

    defaults: H2_Settings;

    array_add(*entries, .{ cast(u16) H2_Setting.ENABLE_PUSH, 0 });

    if conn.local.max_concurrent_streams >= 0 array_add(*entries, .{ cast(u16) H2_Setting.MAX_CONCURRENT_STREAMS, cast(u32) conn.local.max_concurrent_streams });
    if conn.local.initial_window_size != defaults.initial_window_size array_add(*entries, .{ cast(u16) H2_Setting.INITIAL_WINDOW_SIZE, cast(u32) conn.local.initial_window_size });
    if conn.local.max_header_list_size >= 0 array_add(*entries, .{ cast(u16) H2_Setting.MAX_HEADER_LIST_SIZE, cast(u32) conn.local.max_header_list_size });

    h2_put_settings(*conn.out, entries);

    // Room for the connection as a whole beyond the protocol's default.
    if window_bonus > 0 {
        h2_put_window_update(*conn.out, 0, cast(u32) window_bonus);

        conn.recv_window += window_bonus;
    }

    conn.settings_ack_pending += 1;

    Io.add(*conn.tasks, 1);

    conn.refs += 1;

    conn.writer = Io.spawn(h2_writer, conn);
}

// ---- Writing

// Sends what is queued, one batch at a time, until the connection is closing and the queue is empty.
h2_writer :: (conn: *H2_Conn) {
    defer h2_conn_release(conn);
    defer Io.done(*conn.tasks);

    while true {
        while conn.out.count == 0 && !conn.closing {
            if Io.wait(*conn.wake_writer) != .NONE {
                h2_fail(conn);

                return;
            }
        }

        if conn.out.count == 0 break;

        batch := conn.out;

        conn.out = conn.spare;
        conn.out.count = 0;
        conn.spare = batch;

        error: Io.Io_Error;

        Io.with_deadline(within(conn.write_timeout), #code {
            _, error = Io.write(conn.link.writer, conn.spare);
        });

        conn.spare.count = 0;

        // The queue shrinks only now, so senders see the batch in flight as still queued.
        h2_notify_drained(conn);

        if error != .NONE {
            h2_fail(conn);

            return;
        }

        trim(*conn.spare, RETAIN_OUTPUT);
    }

    h2_notify_drained(conn);
}

// The socket failed: nothing more can be sent or received.
h2_fail :: (conn: *H2_Conn) {
    conn.closing = true;

    h2_mark_dead(conn);

    Io.cancel(conn.reader);
}

// Marks the connection unusable, wakes everybody who waits on it and ends the handlers.
h2_mark_dead :: (conn: *H2_Conn) {
    if conn.dead return;

    conn.dead = true;
    if conn.admission_changed Io.notify_all(conn.admission_changed);

    Io.notify_all(*conn.wake_writer);
    h2_notify_drained(conn);

    for conn.streams {
        if it.has_fiber Io.cancel(it.fiber);

        Io.notify_all(*it.changed);
    }
}

// Ends the connection because of an error of ours or the peer's: GOAWAY, then close once it is sent.
h2_terminate :: (conn: *H2_Conn, code: H2_Error, debug := "") {
    if !conn.goaway_sent && !conn.dead {
        h2_put_goaway(*conn.out, conn.last_peer_stream, code, debug);

        conn.goaway_sent = true;
    }

    conn.closing = true;

    h2_mark_dead(conn);
}

// Tells the peer that no stream above the ones in flight will be handled, and waits for those to end.
h2_shutdown :: (conn: *H2_Conn) {
    if conn.goaway_sent || conn.dead return;

    h2_put_goaway(*conn.out, conn.last_peer_stream, .NO_ERROR);

    conn.goaway_sent = true;
    conn.draining = true;

    Io.notify_one(*conn.wake_writer);
}

// Queue drain also wakes stream-specific senders; reset wakes only its own stream.
h2_notify_drained :: (conn: *H2_Conn) {
    Io.notify_all(*conn.drained);
    for conn.streams if it Io.notify_all(*it.changed);
}

h2_writable :: (conn: *H2_Conn, stream: *H2_Stream) -> bool {
    return !conn.dead && (!stream || (!stream.reset_by_peer && !stream.reset_sent && !stream.local_ended));
}

// A stream-specific wait can be interrupted without resuming the socket writer.
h2_wait_for_room :: (conn: *H2_Conn, stream: *H2_Stream = null) -> bool {
    while conn.out.count + conn.spare.count > H2_QUEUE_LIMIT {
        if !h2_writable(conn, stream) return false;

        error: Io.Io_Error;

        Io.with_deadline(within(conn.write_timeout), #code {
            if stream error = Io.wait(*stream.changed);
            else error = Io.wait(*conn.drained);
        });
        if error != .NONE return false;
    }

    return h2_writable(conn, stream);
}

// Queues a header block on `stream` (a new stream's id was chosen by the caller). The fields must
// have lowercase names.
h2_send_headers :: (conn: *H2_Conn, stream: u32, fields: [] Header_Field, end_stream: bool) -> bool {
    if conn.dead return false;

    block: [..] u8;
    block.allocator = temp;

    hpack_encode_start(*conn.encoder, *block);

    for fields hpack_encode_field(*conn.encoder, *block, it.name, it.value);

    h2_put_headers(*conn.out, stream, block, end_stream, conn.remote.max_frame_size);

    Io.notify_one(*conn.wake_writer);

    return true;
}

// Sends `data` on `stream`, as flow control and the peer's frame size allow, waiting for window
// when there is none. With `end_stream` the stream ends after the last byte. False when the
// stream or the connection is gone, or the peer stopped taking data for `write_timeout`.
h2_send_data :: (stream: *H2_Stream, data: [] u8, end_stream: bool) -> bool {
    conn := stream.conn;
    sent := 0;

    while sent < data.count || (end_stream && !stream.local_ended) {
        if conn.dead || stream.reset_by_peer || stream.reset_sent || stream.local_ended return false;

        if !h2_wait_for_room(conn, stream) return false;

        left := data.count - sent;
        count := min(min(left, stream.send_window), min(conn.send_window, conn.remote.max_frame_size));

        if left > 0 && count <= 0 {
            error: Io.Io_Error;

            Io.with_deadline(within(conn.write_timeout), #code {
                error = Io.wait(*stream.changed);
            });

            if error != .NONE {
                h2_reset_stream(conn, stream, .CANCEL);

                return false;
            }

            continue;
        }

        if left == 0 count = 0;

        last := end_stream && count == left;

        h2_put_data(*conn.out, stream.id, .{ count, data.data + sent }, last);

        stream.send_window -= count;
        conn.send_window -= count;
        sent += count;

        if last {
            stream.local_ended = true;
            h2_check_closed(stream);
        }

        Io.notify_one(*conn.wake_writer);
    }

    return true;
}

// Resets a stream: the peer is told, and nothing more is sent or taken on it.
h2_reset_stream :: (conn: *H2_Conn, stream: *H2_Stream, code: H2_Error) {
    if h2_stream_closed(stream) return;

    stream.reset_sent = true;
    stream.reset_code = cast(u32) code;

    if !conn.dead {
        h2_put_rst_stream(*conn.out, stream.id, code);

        Io.notify_one(*conn.wake_writer);
    }

    array_reset(*stream.recv);

    Io.notify_all(*stream.changed);
    h2_check_closed(stream);
}

// Resets a stream that has no object (it was refused before it was created).
h2_refuse_stream :: (conn: *H2_Conn, id: u32, code: H2_Error) {
    if conn.dead return;

    h2_put_rst_stream(*conn.out, id, code);

    Io.notify_one(*conn.wake_writer);
}

// Whether the stream has finished in both directions (or was reset).
h2_stream_closed :: (stream: *H2_Stream) -> bool {
    return stream.reset_sent || stream.reset_by_peer || (stream.local_ended && stream.remote_ended);
}

// Takes a closed stream out of the open count; the connection forgets it once nobody uses it.
h2_check_closed :: (stream: *H2_Stream) {
    if stream.removed || !h2_stream_closed(stream) return;

    conn := stream.conn;

    stream.removed = true;

    if h2_local_stream(conn, stream.id) {
        conn.open_local -= 1;
        h2_retire_drained_client(conn);
        if conn.admission_changed Io.notify_all(conn.admission_changed);

        if conn.open_local == 0 conn.idle_since = now_ms();
    } else {
        conn.open_remote -= 1;
    }

    for conn.streams {
        if it == stream {
            array_ordered_remove_by_index(*conn.streams, it_index);

            break;
        }
    }

    Io.notify_all(*stream.changed);

    h2_stream_release(stream);
}

// A peer may keep TCP open after GOAWAY. Once its accepted calls finish, the
// client retires the connection and joins the ordinary writer/TLS cleanup path.
h2_retire_drained_client :: (conn: *H2_Conn) {
    if conn.role == .CLIENT && conn.goaway_received && conn.open_local == 0 && !conn.closing h2_close_connection(conn);
}

// Whether we opened stream `id`.
h2_local_stream :: (conn: *H2_Conn, id: u32) -> bool {
    return (id % 2 == 1) == (conn.role == .CLIENT);
}

h2_find_stream :: (conn: *H2_Conn, id: u32) -> *H2_Stream {
    for conn.streams if it.id == id return it;

    return null;
}

// A new stream with the windows both sides start with.
h2_stream_new :: (conn: *H2_Conn, id: u32) -> *H2_Stream {
    stream := New(H2_Stream);

    stream.id = id;
    stream.conn = conn;
    stream.send_window = conn.remote.initial_window_size;
    stream.recv_window = conn.local.initial_window_size;
    stream.refs = 1;

    array_add(*conn.streams, stream);

    if h2_local_stream(conn, id) conn.open_local += 1;
    else conn.open_remote += 1;

    return stream;
}

// The user took `count` bytes of a stream's data: the peer may send that much more.
h2_consumed :: (stream: *H2_Stream, count: int) {
    conn := stream.conn;

    stream.unacked += count;

    if stream.remote_ended || stream.reset_sent || conn.dead return;

    if stream.unacked * 2 >= conn.local.initial_window_size {
        h2_put_window_update(*conn.out, stream.id, cast(u32) stream.unacked);

        stream.recv_window += stream.unacked;
        stream.unacked = 0;

        Io.notify_one(*conn.wake_writer);
    }
}

// ---- Reading

// Takes frames off the wire and acts on them until the connection ends. Returns when it has.
h2_read_loop :: (conn: *H2_Conn) {
    header_bytes: [H2_FRAME_HEADER_SIZE] u8;
    payload: [..] u8;

    defer array_reset(*payload);

    while !conn.dead {
        timeout := Io.NO_DEADLINE;

        // A server hangs up on a client that holds a connection open and does nothing with it.
        if conn.role == .SERVER && conn.open_remote == 0 timeout = within(conn.idle_timeout);

        error: Io.Io_Error;

        Io.with_deadline(timeout, #code {
            _, error = Io.read_full(conn.link.reader, header_bytes);
        });

        if error != .NONE break;

        header := h2_decode_header(header_bytes);

        if header.length > conn.local.max_frame_size {
            h2_terminate(conn, .FRAME_SIZE_ERROR, "the frame is larger than the limit in SETTINGS_MAX_FRAME_SIZE");

            break;
        }

        array_reserve(*payload, max(header.length, 1));
        payload.count = header.length;

        if header.length > 0 {
            Io.with_deadline(within(conn.write_timeout), #code {
                _, error = Io.read_full(conn.link.reader, payload);
            });

            if error != .NONE break;
        }

        code := h2_dispatch(conn, header, payload);

        if code != .NO_ERROR {
            h2_terminate(conn, code);

            break;
        }

        reset_temporary_storage();
    }

    // Whatever ended the loop, the connection is over.
    h2_mark_dead(conn);
}

// Acts on one frame. A returned error is a connection error.
h2_dispatch :: (conn: *H2_Conn, header: H2_Frame_Header, payload: [] u8) -> H2_Error {
    // A header block must be finished by CONTINUATION frames before anything else.
    if conn.fragment_stream != 0 {
        if header.type != .CONTINUATION || header.stream != conn.fragment_stream return .PROTOCOL_ERROR;
    } else if header.type == .CONTINUATION {
        return .PROTOCOL_ERROR;
    }

    if header.type == {
        case .DATA; return h2_on_data(conn, header, payload);
        case .HEADERS; return h2_on_headers(conn, header, payload);
        case .CONTINUATION; return h2_on_continuation(conn, header, payload);
        case .PRIORITY; return h2_on_priority(conn, header, payload);
        case .RST_STREAM; return h2_on_reset(conn, header, payload);
        case .SETTINGS; return h2_on_settings(conn, header, payload);
        case .PING; return h2_on_ping(conn, header, payload);
        case .GOAWAY; return h2_on_goaway(conn, header, payload);
        case .WINDOW_UPDATE; return h2_on_window_update(conn, header, payload);
        case .PUSH_PROMISE; return .PROTOCOL_ERROR; // we never allow push
        case; // Other members deliberately do nothing.
    }

    // A frame of a type we do not know is skipped.
    return .NO_ERROR;
}

// Whether `id` names a stream that was never opened.
h2_stream_idle :: (conn: *H2_Conn, id: u32) -> bool {
    if h2_local_stream(conn, id) return id >= conn.next_stream;

    return id > conn.last_peer_stream;
}

h2_on_data :: (conn: *H2_Conn, header: H2_Frame_Header, payload: [] u8) -> H2_Error {
    if header.stream == 0 return .PROTOCOL_ERROR;

    // The padding counts against flow control, and so does data for a stream that is gone.
    conn.recv_window -= payload.count;

    if conn.recv_window < 0 return .FLOW_CONTROL_ERROR;

    conn.recv_unacked += payload.count;

    if conn.recv_unacked >= H2_UPDATE_BATCH {
        h2_put_window_update(*conn.out, 0, cast(u32) conn.recv_unacked);

        conn.recv_window += conn.recv_unacked;
        conn.recv_unacked = 0;

        Io.notify_one(*conn.wake_writer);
    }

    data, error := h2_data_payload(header.flags, payload);

    if error != .NO_ERROR return error;

    if h2_stream_idle(conn, header.stream) return .PROTOCOL_ERROR;

    stream := h2_find_stream(conn, header.stream);

    // Closed and forgotten, or reset by us a moment ago: the data was already in flight.
    if !stream || stream.reset_sent || stream.reset_by_peer return .NO_ERROR;

    if stream.remote_ended {
        h2_reset_stream(conn, stream, .STREAM_CLOSED);

        return .NO_ERROR;
    }

    if conn.role == .CLIENT && !stream.has_head {
        h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

        return .NO_ERROR;
    }

    stream.recv_window -= payload.count;

    if stream.recv_window < 0 {
        h2_reset_stream(conn, stream, .FLOW_CONTROL_ERROR);

        return .NO_ERROR;
    }

    // Padding is discarded here, so it has already been consumed by the receiver.
    h2_consumed(stream, payload.count - data.count);

    append_bytes(*stream.recv, data.data, data.count);

    stream.received += data.count;

    if (header.flags & H2_END_STREAM) != 0 {
        if stream.declared >= 0 && stream.declared != stream.received {
            h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

            return .NO_ERROR;
        }

        stream.remote_ended = true;
    }

    Io.notify_all(*stream.changed);
    h2_check_closed(stream);

    return .NO_ERROR;
}

h2_on_headers :: (conn: *H2_Conn, header: H2_Frame_Header, payload: [] u8) -> H2_Error {
    if header.stream == 0 return .PROTOCOL_ERROR;

    block, error := h2_headers_payload(header.flags, payload);

    if error != .NO_ERROR return error;

    if (header.flags & H2_END_HEADERS) == 0 {
        if block.count > conn.max_header_list * H2_BLOCK_FACTOR return .ENHANCE_YOUR_CALM;

        conn.fragment.count = 0;
        append_bytes(*conn.fragment, block.data, block.count);

        conn.fragment_stream = header.stream;
        conn.fragment_flags = header.flags;
        conn.fragment_frames = 0;

        return .NO_ERROR;
    }

    return h2_headers_complete(conn, header.stream, block, (header.flags & H2_END_STREAM) != 0);
}

h2_on_continuation :: (conn: *H2_Conn, header: H2_Frame_Header, payload: [] u8) -> H2_Error {
    conn.fragment_frames += 1;

    if conn.fragment_frames > H2_MAX_CONTINUATIONS return .ENHANCE_YOUR_CALM;

    if conn.fragment.count + payload.count > conn.max_header_list * H2_BLOCK_FACTOR return .ENHANCE_YOUR_CALM;

    append_bytes(*conn.fragment, payload.data, payload.count);

    if (header.flags & H2_END_HEADERS) == 0 return .NO_ERROR;

    stream := conn.fragment_stream;
    end_stream := (conn.fragment_flags & H2_END_STREAM) != 0;

    conn.fragment_stream = 0;

    return h2_headers_complete(conn, stream, conn.fragment, end_stream);
}

// A whole header block has arrived: decode it (that must happen for every block, whatever
// becomes of the stream, to keep the table in step) and hand it to the server or client logic.
h2_headers_complete :: (conn: *H2_Conn, id: u32, bytes: [] u8, end_stream: bool) -> H2_Error {
    block: Hpack_Block;

    error := hpack_decode(*conn.decoder, bytes, conn.max_header_list, *block);

    if error != .NO_ERROR {
        hpack_block_free(*block);

        return error;
    }

    stream := h2_find_stream(conn, id);

    if conn.role == .SERVER return h2_server_headers(conn, id, stream, block, end_stream);

    return h2_client_headers(conn, id, stream, block, end_stream);
}

h2_on_priority :: (conn: *H2_Conn, header: H2_Frame_Header, payload: [] u8) -> H2_Error {
    if header.stream == 0 return .PROTOCOL_ERROR;

    // The size is wrong: a stream error, and the frame is ignored in any case.
    priority_size :: 5;

    if payload.count != priority_size {
        h2_refuse_stream(conn, header.stream, .FRAME_SIZE_ERROR);
    }

    return .NO_ERROR;
}

h2_on_reset :: (conn: *H2_Conn, header: H2_Frame_Header, payload: [] u8) -> H2_Error {
    if header.stream == 0 return .PROTOCOL_ERROR;
    if payload.count != 4 return .FRAME_SIZE_ERROR;
    if h2_stream_idle(conn, header.stream) return .PROTOCOL_ERROR;

    stream := h2_find_stream(conn, header.stream);

    if !stream return .NO_ERROR;

    code := h2_read_u32(payload, 0);
    // RFC 9113 section 8.1: a completed response survives NO_ERROR used to
    // stop the unfinished request direction. Retain its queued bytes/trailers.
    if conn.role == .CLIENT && stream.remote_ended && code == cast(u32) H2_Error.NO_ERROR {
        stream.local_ended = true;
        Io.notify_all(*stream.changed);
        h2_check_closed(stream);
        return .NO_ERROR;
    }

    stream.reset_by_peer = true;
    stream.reset_code = code;
    if stream.has_fiber Io.cancel(stream.fiber);

    // A refused stream was not processed at all.
    stream.refused = stream.reset_code == cast(u32) H2_Error.REFUSED_STREAM;

    array_reset(*stream.recv);

    Io.notify_all(*stream.changed);
    h2_check_closed(stream);

    return .NO_ERROR;
}

h2_on_settings :: (conn: *H2_Conn, header: H2_Frame_Header, payload: [] u8) -> H2_Error {
    if header.stream != 0 return .PROTOCOL_ERROR;

    if (header.flags & H2_ACK) != 0 {
        if payload.count != 0 return .FRAME_SIZE_ERROR;

        conn.settings_ack_pending -= 1;

        return .NO_ERROR;
    }

    before := conn.remote;
    updated := conn.remote;
    error := h2_read_settings(payload, *updated);

    if error != .NO_ERROR return error;

    // New streams start with the new window, and the ones in flight move by the difference.
    delta := updated.initial_window_size - before.initial_window_size;

    if delta != 0 {
        for conn.streams {
            if it.send_window + delta > H2_LARGEST_WINDOW return .FLOW_CONTROL_ERROR;
        }

        for conn.streams it.send_window += delta;
    }

    conn.remote = updated;
    if conn.admission_changed Io.notify_all(conn.admission_changed);

    if updated.header_table_size != before.header_table_size hpack_encoder_limit(*conn.encoder, updated.header_table_size);

    h2_put_settings_ack(*conn.out);

    Io.notify_one(*conn.wake_writer);

    // Windows may have opened, and a new frame size may let a waiting sender continue.
    for conn.streams Io.notify_all(*it.changed);

    return .NO_ERROR;
}

h2_on_ping :: (conn: *H2_Conn, header: H2_Frame_Header, payload: [] u8) -> H2_Error {
    if header.stream != 0 return .PROTOCOL_ERROR;
    if payload.count != 8 return .FRAME_SIZE_ERROR;

    if (header.flags & H2_ACK) == 0 && !conn.dead {
        h2_put_ping(*conn.out, payload, true);

        Io.notify_one(*conn.wake_writer);
    }

    return .NO_ERROR;
}

h2_on_goaway :: (conn: *H2_Conn, header: H2_Frame_Header, payload: [] u8) -> H2_Error {
    if header.stream != 0 return .PROTOCOL_ERROR;

    last, code, _, error := h2_read_goaway(payload);

    if error != .NO_ERROR return error;

    conn.goaway_received = true;
    if conn.admission_changed Io.notify_all(conn.admission_changed);

    conn.goaway_last = last;
    conn.goaway_code = code;

    // Streams the peer never looked at can be tried again elsewhere.
    index := conn.streams.count - 1;

    while index >= 0 {
        stream := conn.streams[index];

        if h2_local_stream(conn, stream.id) && stream.id > last {
            stream.refused = true;
            stream.goaway_refused = true;
            stream.reset_by_peer = true;

            Io.notify_all(*stream.changed);
            h2_check_closed(stream);
        }

        index -= 1;
    }

    h2_retire_drained_client(conn);

    return .NO_ERROR;
}

h2_on_window_update :: (conn: *H2_Conn, header: H2_Frame_Header, payload: [] u8) -> H2_Error {
    if payload.count != 4 return .FRAME_SIZE_ERROR;

    increment := cast(s64) (h2_read_u32(payload, 0) & 0x7fffffff);

    if header.stream == 0 {
        if increment == 0 return .PROTOCOL_ERROR;

        conn.send_window += increment;

        if conn.send_window > H2_LARGEST_WINDOW return .FLOW_CONTROL_ERROR;

        for conn.streams Io.notify_all(*it.changed);

        return .NO_ERROR;
    }

    if h2_stream_idle(conn, header.stream) return .PROTOCOL_ERROR;

    stream := h2_find_stream(conn, header.stream);

    if !stream return .NO_ERROR;

    if increment == 0 {
        h2_reset_stream(conn, stream, .PROTOCOL_ERROR);

        return .NO_ERROR;
    }

    stream.send_window += increment;

    if stream.send_window > H2_LARGEST_WINDOW {
        h2_reset_stream(conn, stream, .FLOW_CONTROL_ERROR);

        return .NO_ERROR;
    }

    Io.notify_all(*stream.changed);

    return .NO_ERROR;
}

// ---- Header fields shared by both sides

// Whether a regular header field may not appear in HTTP/2 (RFC 9113 section 8.2.2).
h2_connection_specific :: (name: string) -> bool {
    return name == "connection" || name == "proxy-connection" || name == "keep-alive" || name == "transfer-encoding" || name == "upgrade";
}

// Initial H2 fields share HTTP token/value syntax, with a single leading colon
// permitted for pseudo headers; each role separately validates their identities.
h2_field_syntax_ok :: (field: Header_Field) -> bool {
    if h2_has_uppercase(field.name) return false;
    regular := field;
    if regular.name.count && regular.name[0] == #char ":" regular.name = slice(regular.name, 1, regular.name.count - 1);
    return field_syntax_ok(regular);
}

// Trailer blocks contain regular fields only. TE is legal only with `trailers`.
h2_valid_trailers :: (fields: [] Header_Field) -> bool {
    // Share token/value and framing-field checks with the HTTP writers.
    if !fields_ok(fields) return false;

    for fields {
        if it.name.count == 0 || it.name[0] == #char ":" || h2_has_uppercase(it.name) || h2_connection_specific(it.name) return false;
        if it.name == "te" && it.value != "trailers" return false;
    }

    return true;
}

// Whether the name has an uppercase letter, which makes a message malformed.
h2_has_uppercase :: (name: string) -> bool {
    for name if it >= #char "A" && it <= #char "Z" return true;

    return false;
}

// The number in a `content-length` value, or -1.
h2_parse_length :: (value: string) -> s64 {
    if value.count == 0 || value.count > MAX_LENGTH_DIGITS return -1;

    number: s64;

    for value {
        if it < #char "0" || it > #char "9" return -1;

        number = number * 10 + (it - #char "0");
    }

    return number;
}
