// Settings for [ws_upgrade] and [ws_connect]. `protocols` lists the subprotocols to offer or accept
// in preference order and must outlive the connection. Timeouts are in milliseconds; 0 or less means
// none.
Ws_Config :: struct {
    limits: Ws_Options;
    compression := true;
    protocols: [] string;
    handshake_timeout: s64 = 30000;
    read_timeout: s64;
    write_timeout: s64 = 30000;
    close_timeout: s64 = 5000;
}

// Connection values must not be copied after first use. One reader and one writer may run concurrently.
Ws_Connection :: struct {
    codec: Ws_Codec;
    config: Ws_Config;
    protocol: string;
    link: Http.Link;
    pending: [] u8;
    initial: [] u8;
    buffer: [16384] u8;
    write_lock: Io.Fiber_Mutex;
    dictionary: [..] u8;
    close_sent: bool;
    owned: bool;
    disposed: bool;
}

// Runs a server connection after a successful [ws_upgrade]. The connection is borrowed until it
// returns; `data` is what you passed to `ws_upgrade`.
Ws_Handler :: #type (connection: *Ws_Connection, data: *void);

// Validates an upgrade and returns a routed response. Example: return ws_upgrade(request, echo).
// The callback borrows the connection until it returns; config.protocols and data must outlive it.
ws_upgrade :: (request: *Http.Request, handler: Ws_Handler, data: *void = null, config := Ws_Config.{}) -> Http.Response {
    key, key_ok, _ := unique_header(request.headers, "Sec-WebSocket-Key");
    version, version_ok, _ := unique_header(request.headers, "Sec-WebSocket-Version");

    if !version_ok || version != "13" return Http.with_header(Http.status_only(426), "Sec-WebSocket-Version", "13");
    if !handler || request.method != .GET || request.version != .HTTP_1_1 || !key_ok || key.count != 24 || !header_token(request.headers, "Upgrade", "websocket") || !header_token(request.headers, "Connection", "upgrade") return Http.text("invalid WebSocket handshake", 400);

    decoded, valid := Base64.base64_decode(key);

    defer if decoded.data free(decoded);
    if !valid || decoded.count != 16 return Http.text("invalid WebSocket key", 400);

    canonical := Base64.base64_encode(decoded);

    defer free(canonical);
    if canonical != key return Http.text("noncanonical WebSocket key", 400);

    selected: string;
    offered := joined_header(request.headers, "Sec-WebSocket-Protocol");

    if offered.count {
        rest := offered;
        while rest.count {
            _, item, after := cut(rest, #char ",");
            if !valid_token(trim(item, " \t")) return Http.text("invalid subprotocol offer", 400);
            rest = after;
        }

        for config.protocols {
            if !valid_token(it) return Http.text("invalid server subprotocol", 500);

            if protocol_offered(offered, it) {
                selected = it;
                break;
            }
        }
    }

    compression: Ws_Deflate;

    if config.compression {
        candidate, error := ws_negotiate(joined_header(request.headers, "Sec-WebSocket-Extensions"));
        // Declining an invalid/unsupported extension offer leaves the base protocol usable.
        if error.kind == .NONE compression = candidate;
    }

    state := New(Upgrade_State,, temp);
    state.* = .{ handler = handler, data = data, config = config, compression = compression, protocol = selected };
    response := Http.upgrade_response("websocket", serve_websocket, state);
    accept := ws_accept_key(key);

    response = Http.with_header(response, "Sec-WebSocket-Accept", copy_string(accept,, temp));
    free(accept);
    if selected.count response = Http.with_header(response, "Sec-WebSocket-Protocol", selected);
    if compression.enabled response = Http.with_header(response, "Sec-WebSocket-Extensions", extension_text(compression));

    return response;
}

// Connects on the calling Io fiber, sharing Http's transport and TLS configuration.
// Example: connection, error := ws_connect("wss://example.org/chat"); defer ws_dispose(*connection).
ws_connect :: (url: string, config := Ws_Config.{}, client: *Http.Client = null) -> Ws_Connection, Ws_Error {
    if !Io.in_scheduler() return .{}, .{ .TRANSPORT, "ws_connect requires an Io fiber", 0 };

    http_url: string;
    if begins_with(url, "ws://") http_url = tprint("http://%", slice(url, 5, url.count - 5));
    else if begins_with(url, "wss://") http_url = tprint("https://%", slice(url, 6, url.count - 6));
    else return .{}, .{ .HANDSHAKE, "expected ws:// or wss:// URL", 0 };

    nonce: [16] u8;
    if !Crypto.random_bytes(nonce) return .{}, .{ .TRANSPORT, "secure randomness unavailable", 0 };

    key := Base64.base64_encode(string.{ 16, nonce.data });

    defer free(key);
    expected := ws_accept_key(key);

    defer free(expected);
    headers: [..] Http.Header_Field;

    defer array_free(headers);
    array_add(*headers, .{ name = "Connection", value = "Upgrade" });
    array_add(*headers, .{ name = "Upgrade", value = "websocket" });
    array_add(*headers, .{ name = "Sec-WebSocket-Key", value = key });
    array_add(*headers, .{ name = "Sec-WebSocket-Version", value = "13" });
    if config.compression array_add(*headers, .{ name = "Sec-WebSocket-Extensions", value = "permessage-deflate; client_max_window_bits; server_max_window_bits=15" });

    if config.protocols.count {
        builder: String_Builder;
        builder.allocator = temp;

        for config.protocols {
            if !valid_token(it) return .{}, .{ .HANDSHAKE, "invalid subprotocol", 0 };
            if it_index append(*builder, ", ");
            append(*builder, it);
        }

        array_add(*headers, .{ name = "Sec-WebSocket-Protocol", value = builder_to_string(*builder,, temp) });
    }

    local: Http.Client;
    if !client client = *local;

    response: Http.Client_Response;
    link: Http.Link;
    pending: [] u8;
    failure: Http.Client_Error;

    Io.with_deadline(budget(config.handshake_timeout), #code {
        response, link, pending, failure = Http.client_upgrade(client, http_url, headers);
    });
    defer Http.close(*response);
    defer Http.free_error(*failure);
    if failure.kind != .NONE return .{}, .{ ifx failure.kind == .TIMED_OUT then .TIMED_OUT else .HANDSHAKE, "HTTP upgrade failed", 0 };

    success := false;
    defer if !success {
        Http.close(link);
        free(pending.data);
    }

    accept, accept_ok, _ := unique_header(response.headers, "Sec-WebSocket-Accept");
    if !accept_ok || accept != expected || !header_token(response.headers, "Upgrade", "websocket") || !header_token(response.headers, "Connection", "upgrade") return .{}, .{ .HANDSHAKE, "invalid switching response", 0 };

    protocol, protocol_ok, protocol_present := unique_header(response.headers, "Sec-WebSocket-Protocol");
    if protocol_present && (!protocol_ok || !valid_token(protocol)) return .{}, .{ .HANDSHAKE, "invalid subprotocol selection", 0 };

    if protocol.count {
        allowed := false;
        for config.protocols if it == protocol allowed = true;
        if !allowed return .{}, .{ .HANDSHAKE, "unsolicited subprotocol", 0 };
    }

    extensions := joined_header(response.headers, "Sec-WebSocket-Extensions");
    compression, error := ws_negotiate(extensions, true);

    if error.kind != .NONE return .{}, error;
    if compression.enabled && !config.compression return .{}, .{ .HANDSHAKE, "unsolicited compression", 0 };

    connection: Ws_Connection;
    connection.codec.role = .CLIENT;
    connection.codec.options = config.limits;
    connection.codec.compression = compression;
    connection.config = config;
    connection.protocol = copy_string(protocol);
    connection.link = link;
    connection.initial = pending;
    connection.pending = pending;
    connection.owned = true;
    success = true;

    return connection, .{};
}

// Reads a message or close; answers pings automatically and ignores pongs.
// Example: event := ws_read(*connection); if event.kind == .MESSAGE consume(event.data).
ws_read :: (connection: *Ws_Connection) -> Ws_Event {
    event: Ws_Event;

    if connection.disposed return .{ kind = .ERROR, error = .{ .CLOSED, "connection disposed", 0 } };

    Io.with_deadline(budget(connection.config.read_timeout), #code {
        event = read_event(connection);
    });
    if event.kind == .ERROR ws_dispose(connection);

    return event;
}

// Sends a complete text/binary message, fragmenting to max_frame_size, or a ping/pong.
// Example: error := ws_write(*connection, cast([] u8) "hello", .TEXT).
ws_write :: (connection: *Ws_Connection, data: [] u8, opcode := Ws_Opcode.TEXT) -> Ws_Error {
    if connection.disposed || connection.close_sent || connection.codec.closed return .{ .CLOSED, "connection is closing", 0 };
    if opcode != .TEXT && opcode != .BINARY && opcode != .PING && opcode != .PONG return .{ .PROTOCOL, "use ws_close for close frames", 0 };
    if data.count > connection.config.limits.max_message_size return .{ .LIMIT, "message exceeds limit", 0 };

    if opcode == .TEXT {
        utf: Ws_Codec;
        for data if !utf_byte(*utf, it) return .{ .UTF8, "invalid outgoing text", it_index };
        if utf.utf_remaining return .{ .UTF8, "incomplete outgoing text", data.count };
    }

    result: Ws_Error;

    Io.with_deadline(budget(connection.config.write_timeout), #code {
        result = send_message(connection, data, opcode);
    });
    if result.kind == .TRANSPORT || result.kind == .TIMED_OUT ws_dispose(connection);

    return result;
}

// Sends close and waits for the peer under close_timeout, then releases the connection.
// Example: error := ws_close(*connection, 1000, "done"). Do not call beside another reader.
ws_close :: (connection: *Ws_Connection, code: u16 = 1000, reason: string = "") -> Ws_Error {
    if connection.disposed return .{};
    if reason.count > 123 return .{ .PROTOCOL, "close reason too long", 0 };

    payload: [125] u8;
    payload[0] = cast(u8) (code >> 8);
    payload[1] = cast,trunc(u8) code;
    if reason.count memcpy(payload.data + 2, reason.data, reason.count);

    data: [] u8 = .{ 2 + reason.count, payload.data };
    validation := check_close(data, 0);

    if validation.kind != .NONE return validation;

    result: Ws_Error;

    Io.with_grace(budget(connection.config.close_timeout), #code {
        if !connection.close_sent result = send_message(connection, data, .CLOSE);

        while result.kind == .NONE && !connection.codec.closed {
            event := read_event(connection);
            if event.kind == .ERROR result = event.error;
        }
    });
    ws_dispose(connection);

    return result;
}

// Aborts without a close handshake and releases owned buffers. Example: defer ws_dispose(*connection).
ws_dispose :: (connection: *Ws_Connection) {
    if connection.disposed return;

    connection.disposed = true;
    if connection.owned Http.close(connection.link);

    ws_free(*connection.codec);
    array_free(connection.dictionary);
    free(connection.initial.data);
    free(connection.protocol);
    connection.initial = .[];
    connection.pending = .[];
    connection.protocol = "";
}

#scope_module
Http :: #import "Extensions/Http";
Io :: #import "Extensions/Io";
Crypto :: #import,file "../../internal/crypto/random.jai";

Upgrade_State :: struct {
    handler: Ws_Handler;
    data: *void;
    config: Ws_Config;
    compression: Ws_Deflate;
    protocol: string;
}

serve_websocket :: (link: Http.Link, pending: [] u8, data: *void) {
    state := cast(*Upgrade_State) data;
    connection: Ws_Connection;
    connection.link = link;
    connection.pending = pending;
    connection.config = state.config;
    connection.protocol = copy_string(state.protocol);
    connection.codec.options = state.config.limits;
    connection.codec.compression = state.compression;
    defer ws_dispose(*connection);
    state.handler(*connection, state.data);
    if !connection.disposed ws_close(*connection);
}

budget :: (ms: s64) -> s64 {
    return ifx ms > 0 then ms else Io.NO_DEADLINE;
}

unique_header :: (headers: [] Http.Header_Field, name: string) -> string, bool, bool {
    value: string;
    count := 0;

    for headers if equal_nocase(it.name, name) {
        value = it.value;
        count += 1;
    }

    return value, count == 1, count != 0;
}

joined_header :: (headers: [] Http.Header_Field, name: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    count := 0;

    for headers if equal_nocase(it.name, name) {
        if count append(*builder, ",");
        append(*builder, it.value);
        count += 1;
    }

    return builder_to_string(*builder,, temp);
}

header_token :: (headers: [] Http.Header_Field, name: string, token: string) -> bool {
    for headers if equal_nocase(it.name, name) && Http.has_token(it.value, token) return true;
    return false;
}

protocol_offered :: (offered: string, protocol: string) -> bool {
    rest := offered;
    while rest.count {
        _, item, after := cut(rest, #char ",");
        if trim(item, " \t") == protocol return true;
        rest = after;
    }

    return false;
}

transport_error :: (error: Io.Io_Error) -> Ws_Error {
    return .{ ifx error == .TIMED_OUT then .TIMED_OUT else .TRANSPORT, "WebSocket transport failed", 0 };
}

read_event :: (connection: *Ws_Connection) -> Ws_Event {
    while true {
        used, event := ws_parse(*connection.codec, connection.pending);

        connection.pending = .{ connection.pending.count - used, connection.pending.data + used };

        if event.kind == .ERROR {
            code: u16 = 1002;
            if event.error.kind == .UTF8 code = 1007;
            if event.error.kind == .LIMIT || event.error.kind == .COMPRESSION code = 1009;

            payload := u8.[cast(u8) (code >> 8), cast,trunc(u8) code];

            if !connection.close_sent {
                Io.with_grace(budget(connection.config.close_timeout), #code {
                    send_message(connection, payload, .CLOSE);
                });
            }

            return event;
        }

        if event.kind == .MESSAGE return event;

        if event.kind == .CLOSE {
            if !connection.close_sent {
                failure := send_message(connection, event.data, .CLOSE);
                if failure.kind != .NONE return .{ kind = .ERROR, error = failure };
            }

            return event;
        }

        if event.kind == .PING {
            failure := send_message(connection, event.data, .PONG);
            if failure.kind != .NONE return .{ kind = .ERROR, error = failure };
        }

        if event.kind != .NEED_MORE continue;

        count, error := Io.read(connection.link.reader, connection.buffer);
        if error != .NONE return .{ kind = .ERROR, error = transport_error(error) };
        if count == 0 return .{ kind = .ERROR, error = .{ .TRANSPORT, "EOF without close frame", connection.codec.offset } };

        connection.pending = .{ count, connection.buffer.data };
    }

    return .{};
}

send_message :: (connection: *Ws_Connection, data: [] u8, opcode: Ws_Opcode) -> Ws_Error {
    result: Ws_Error;

    Io.with_deadline(budget(connection.config.write_timeout), #code {
        result = send_frames(connection, data, opcode);
    });

    return result;
}

send_frames :: (connection: *Ws_Connection, data: [] u8, opcode: Ws_Opcode) -> Ws_Error {
    error := Io.lock(*connection.write_lock);
    if error != .NONE return transport_error(error);

    defer Io.unlock(*connection.write_lock);
    if connection.close_sent return .{ .CLOSED, "close already sent", 0 };

    control := cast(u8) opcode >= 8;
    packed: [] u8;
    payload := data;
    compressed := connection.codec.compression.enabled && !control;

    if compressed {
        packed = ws_deflate(connection.codec.role, connection.codec.compression, data, connection.dictionary);
        payload = packed;
    }

    defer if packed.data free(packed.data);
    limit := connection.config.limits.max_frame_size;
    if limit <= 0 return .{ .LIMIT, "frame limit must be positive", 0 };

    out: [..] u8;

    defer array_free(out);
    at := 0;

    while true {
        count := ifx control then payload.count else min(limit, payload.count - at);
        final := at + count == payload.count;
        key: [4] u8;
        masked := connection.codec.role == .CLIENT;

        if masked && !Crypto.random_bytes(key) return .{ .TRANSPORT, "secure randomness unavailable", 0 };

        frame_error := ws_frame(*out, ifx at == 0 then opcode else .CONTINUATION, .{ count, payload.data + at }, final, masked, key, compressed && at == 0, connection.config.limits);
        if frame_error.kind != .NONE return frame_error;

        _, failure := Io.write(connection.link.writer, out);
        if failure != .NONE return transport_error(failure);

        out.count = 0;
        at += count;
        if final break;
    }

    if compressed {
        settings := connection.codec.compression;
        no_context := ifx connection.codec.role == .CLIENT then settings.client_no_context_takeover else settings.server_no_context_takeover;
        bits := ifx connection.codec.role == .CLIENT then settings.client_max_window_bits else settings.server_max_window_bits;
        if !no_context retain_history(*connection.dictionary, data, bits);
    }

    if opcode == .CLOSE connection.close_sent = true;

    return .{};
}
