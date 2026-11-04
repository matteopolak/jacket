// Method descriptors preserve wire paths independently of generated identifiers.
Method :: struct {
    path: string;
    client_streaming: bool;
    server_streaming: bool;
}

// Scheduler-bound RPC scope. Keep its address and TLS configuration stable until
// all calls and close_channel finish; active call and connection limits bound admission.
Channel :: struct {
    transport: Http.Http2_Channel;
    max_active_calls := 128;
    active_calls: int;
}

// Per-call deadlines and message/metadata bounds. Client outgoing metadata is
// borrowed through open_call; server initial metadata is borrowed through serve_call.
Call_Options :: struct {
    // Absolute monotonic milliseconds; zero means no RPC deadline.
    deadline_ms: s64;
    metadata: [] Metadata_Entry;
    max_message := 4 * 1024 * 1024;
    max_metadata := 64 * 1024;
}

// Opaque RPC handle. Join its sender and receiver before close; returned messages
// and statuses own independent storage.
Call :: struct {
    state: *void;
}

// MESSAGE carries one owned message. On clients, END and ERROR still require
// finish to obtain the authoritative RPC status.
Receive_Kind :: enum u8 {
    MESSAGE;
    END;
    ERROR;
}

// Opens an RPC without retry or redirect. The channel and TLS configuration must
// remain alive until every call and close_channel have completed.
// Example: `call, status := open_call(*channel, method, options); defer close(*call);`.
open_call :: (channel: *Channel, method: Method, options := Call_Options.{}) -> Call, Status {
    if channel.active_calls >= channel.max_active_calls return .{}, .{ code = .RESOURCE_EXHAUSTED, cause = .LIMIT };
    if options.max_message < 0 || options.max_metadata < 0 return .{}, .{ code = .INVALID_ARGUMENT, cause = .LIMIT };

    remaining := rpc_remaining(options.deadline_ms);
    if remaining == 0 return .{}, .{ code = .DEADLINE_EXCEEDED, cause = .DEADLINE };

    metadata, metadata_error := metadata_encode(options.metadata, options.max_metadata);
    if metadata_error != .NONE return .{}, .{ code = ifx metadata_error == .LIMIT then Status_Code.RESOURCE_EXHAUSTED else Status_Code.INVALID_ARGUMENT, cause = ifx metadata_error == .LIMIT then Local_Cause.LIMIT else Local_Cause.PROTOCOL };

    fields: [..] Http.Header_Field;
    fields.allocator = temp;
    array_add(*fields, .{ name = "content-type", value = "application/grpc+proto" });
    array_add(*fields, .{ name = "te", value = "trailers" });
    array_add(*fields, .{ name = "grpc-accept-encoding", value = "identity" });
    if remaining > 0 array_add(*fields, .{ name = "grpc-timeout", value = encode_timeout(remaining) });
    for metadata array_add(*fields, it);

    if !metadata_headers_fit(fields, options.max_metadata) return .{}, .{ code = .RESOURCE_EXHAUSTED, cause = .LIMIT };

    channel.active_calls += 1;
    exchange: Http.Http2_Exchange;
    failure: Http.Client_Error;

    Io.with_deadline(remaining, #code {
        exchange, failure = Http.open_http2_exchange(*channel.transport, method.path, fields);
    });

    if failure.kind != .NONE {
        channel.active_calls -= 1;
        result := rpc_transport_status(failure);
        if rpc_remaining(options.deadline_ms) == 0 result = .{ code = .DEADLINE_EXCEEDED, cause = .DEADLINE };

        Http.free_error(*failure);

        return .{}, result;
    }

    state := New(Call_State);
    state.exchange = exchange;
    state.channel = channel;
    state.method = method;
    state.options = options;
    state.options.metadata = .[];
    state.decoder.max_message = options.max_message;
    state.client = true;

    return .{ state }, .{};
}

// Sends an owned-by-caller encoded Protobuf message. Framing owns a temporary
// copy only until this operation returns. One sender may coexist with one receiver.
// Example: `status := send_bytes(*call, encoded); defer free_status(*status);`.
send_bytes :: (call: *Call, message: [] u8) -> Status {
    state := cast(*Call_State) call.state;
    if !state return .{ code = .CANCELLED, cause = .CANCELLED };
    if state.terminal return clone_status(state.status);
    if state.send_closed return .{ code = .FAILED_PRECONDITION };
    if state.sending return rpc_fail(state, .FAILED_PRECONDITION, .PROTOCOL);

    state.sending = true;
    defer state.sending = false;
    streaming := ifx state.client then state.method.client_streaming else state.method.server_streaming;
    if state.sent > 0 && !streaming return rpc_fail(state, .INTERNAL, .PROTOCOL);

    bytes, failure := frame_message(message, state.options.max_message);
    if failure != .NONE return rpc_fail(state, .RESOURCE_EXHAUSTED, .LIMIT);

    defer free(bytes.data);
    error: Http.Client_Error;
    remaining := rpc_remaining(state.options.deadline_ms);

    if remaining == 0 return rpc_fail(state, .DEADLINE_EXCEEDED, .DEADLINE);

    Io.with_deadline(remaining, #code {
        error = Http.exchange_write(*state.exchange, bytes);
    });

    if error.kind != .NONE {
        result := rpc_write_error(state, error);
        Http.free_error(*error);
        return result;
    }

    state.sent += 1;

    return .{};
}

// Encodes a generated Protobuf value and sends it through the shared framing path.
// Example: `status := send(*call, request); defer free_status(*status);`.
send :: (call: *Call, message: $T) -> Status {
    state := cast(*Call_State) call.state;
    if !state return .{ code = .CANCELLED, cause = .CANCELLED };
    if Proto.size(message) > state.options.max_message return rpc_fail(state, .RESOURCE_EXHAUSTED, .LIMIT);

    bytes, failure := Proto.encode(message);
    if failure.kind != .NONE {
        Proto.free(*failure);
        return rpc_fail(state, .INTERNAL, .PROTOCOL);
    }

    defer free(bytes.data);

    return send_bytes(call, bytes);
}

// Half-closes the client's sending direction. No automatic retry occurs.
// Example: `status := close_send(*call); defer free_status(*status);`.
close_send :: (call: *Call) -> Status {
    state := cast(*Call_State) call.state;
    if !state return .{ code = .CANCELLED, cause = .CANCELLED };
    if state.send_closed return .{};
    if state.terminal return clone_status(state.status);
    if state.sending return rpc_fail(state, .FAILED_PRECONDITION, .PROTOCOL);

    state.sending = true;
    defer state.sending = false;
    if !state.method.client_streaming && state.sent != 1 return rpc_fail(state, .INTERNAL, .PROTOCOL);

    error: Http.Client_Error;
    remaining := rpc_remaining(state.options.deadline_ms);

    if remaining == 0 return rpc_fail(state, .DEADLINE_EXCEEDED, .DEADLINE);

    Io.with_deadline(remaining, #code {
        error = Http.exchange_write(*state.exchange, .[], true);
    });

    if error.kind != .NONE {
        result := rpc_write_error(state, error);
        Http.free_error(*error);
        return result;
    }

    state.send_closed = true;

    return .{};
}

// Receives one framed message into caller-owned memory. END is a byte-stream
// event, not application success; always inspect finish's authoritative Status.
// Example: `bytes, kind := receive_bytes(*call); if bytes.data free(bytes.data);`.
receive_bytes :: (call: *Call) -> message: [] u8, kind: Receive_Kind {
    state := cast(*Call_State) call.state;
    if !state return .[], .ERROR;

    if state.receiving {
        discard := rpc_fail(state, .FAILED_PRECONDITION, .PROTOCOL);
        free_status(*discard);
        return .[], .ERROR;
    }

    state.receiving = true;
    defer state.receiving = false;
    if state.terminal return .[], ifx state.status.code == .OK then Receive_Kind.END else Receive_Kind.ERROR;

    remaining := rpc_remaining(state.options.deadline_ms);
    if remaining == 0 {
        discard := rpc_fail(state, .DEADLINE_EXCEEDED, .DEADLINE);
        free_status(*discard);
        return .[], .ERROR;
    }

    if state.client && !state.has_head && !rpc_receive_head(state, remaining) return .[], .ERROR;

    while true {
        if !state.invalid_media && state.input_at < state.input_count {
            used, error := feed_envelope(*state.decoder, .{ state.input_count - state.input_at, state.input.data + state.input_at });

            state.input_at += used;

            if error != .NONE {
                code := ifx error == .TOO_LARGE then Status_Code.RESOURCE_EXHAUSTED else ifx error == .UNSUPPORTED_COMPRESSION then Status_Code.UNIMPLEMENTED else Status_Code.INTERNAL;
                discard := rpc_fail(state, code, .PROTOCOL);
                free_status(*discard);
                return .[], .ERROR;
            }

            if state.decoder.ready {
                bytes := take_message(*state.decoder);

                state.received += 1;
                streaming := ifx state.client then state.method.server_streaming else state.method.client_streaming;
                if state.received > 1 && !streaming {
                    if bytes.data free(bytes.data);
                    discard := rpc_fail(state, .INTERNAL, .PROTOCOL);
                    free_status(*discard);
                    return .[], .ERROR;
                }

                return bytes, .MESSAGE;
            }
        }

        state.input_at = 0;
        error: Http.Client_Error;

        remaining = rpc_remaining(state.options.deadline_ms);
        if remaining == 0 {
            discard := rpc_fail(state, .DEADLINE_EXCEEDED, .DEADLINE);
            free_status(*discard);
            return .[], .ERROR;
        }

        Io.with_deadline(remaining, #code {
            state.input_count, error = Http.exchange_read(*state.exchange, state.input);
        });

        if error.kind != .NONE {
            rpc_record_transport(state, *error);
            return .[], .ERROR;
        }

        if state.input_count == 0 {
            if finish_envelopes(*state.decoder) != .NONE {
                discard := rpc_fail(state, .INTERNAL, .PROTOCOL);
                free_status(*discard);
                return .[], .ERROR;
            }

            state.read_closed = true;
            if !state.client return .[], .END;

            fields, complete := Http.exchange_trailers(*state.exchange);
            if !complete {
                discard := rpc_fail(state, .UNKNOWN, .PROTOCOL);
                free_status(*discard);
                return .[], .ERROR;
            }

            if state.trailers_only state.status = state.initial_status;
            else state.status = parse_status(fields, state.http_status, state.options.max_metadata);

            state.initial_status = .{};
            if state.invalid_media {
                if state.status.code == .OK state.status.code = .UNKNOWN;
                state.status.cause = .PROTOCOL;
            }

            if state.status.code == .OK && !state.method.server_streaming && state.received != 1 state.status.code = .INTERNAL;

            state.terminal = true;

            return .[], ifx state.status.code == .OK then Receive_Kind.END else Receive_Kind.ERROR;
        }
    }

    return .[], .ERROR;
}

// Decodes into owned Protobuf storage. Release a MESSAGE with Proto.free.
// Example: `response, kind := receive(*call, Api.Response); defer Proto.free(*response);`.
receive :: (call: *Call, $T: Type) -> value: T, kind: Receive_Kind {
    bytes, kind := receive_bytes(call);
    if kind != .MESSAGE return .{}, kind;

    defer if bytes.data free(bytes.data);
    value, error := Proto.decode(T, bytes);
    if error.kind != .NONE {
        Proto.free(*error);
        state := cast(*Call_State) call.state;
        discard := rpc_fail(state, .INTERNAL, .PROTOCOL);

        free_status(*discard);

        return .{}, .ERROR;
    }

    return value, .MESSAGE;
}

// Waits for final status, discarding unread messages one at a time. Returns an
// independently owned Status, freed with free_status. Non-OK may follow messages.
// Example: `status := finish(*call); defer free_status(*status);`.
finish :: (call: *Call) -> Status {
    state := cast(*Call_State) call.state;
    if !state return .{ code = .CANCELLED, cause = .CANCELLED };

    while !state.terminal {
        bytes, kind := receive_bytes(call);
        if bytes.data free(bytes.data);
        if kind != .MESSAGE break;
    }

    return clone_status(state.status);
}

// Explicit cancellation wins if no terminal result has been observed locally.
// Wakes blocked transport operations; join those fibers before close.
// Example: `cancel(*call); // then join its sender and receiver`.
cancel :: (call: *Call) {
    state := cast(*Call_State) call.state;
    if !state || state.terminal return;

    state.status.code = .CANCELLED;
    state.status.cause = .CANCELLED;
    state.terminal = true;
    Http.exchange_cancel(*state.exchange);
}

// Releases call storage after sender/receiver fibers return. Idempotent on a
// cleared handle; received messages and returned statuses remain caller-owned.
// Example: `close(*call); close(*call); // the second call is harmless`.
close :: (call: *Call) {
    state := cast(*Call_State) call.state;
    if !state return;

    cancel(call);
    Http.close_exchange(*state.exchange);
    free_decoder(*state.decoder);
    free_status(*state.status);
    free_status(*state.initial_status);
    free_metadata(*state.initial_metadata);
    if state.channel state.channel.active_calls -= 1;

    free(state);
    call.state = null;
}

// Releases the channel after its calls have ended.
// Example: `close_channel(*channel); // after closing every call`.
close_channel :: (channel: *Channel) {
    assert(channel.active_calls == 0, "close every call before its channel");
    Http.close_http2_channel(*channel.transport);
}

// Initial application metadata borrows call-owned storage until close. On clients
// it becomes available after the first receive has obtained response headers.
// Example: `for initial_metadata(*call) print("%\n", it.name);`.
initial_metadata :: (call: *Call) -> [] Metadata_Entry {
    state := cast(*Call_State) call.state;
    if !state return .[];
    return state.initial_metadata;
}

// The absolute monotonic deadline for explicit child-call budget inheritance.
// Example: `child_options.deadline_ms = call_deadline(*parent);`.
call_deadline :: (call: *Call) -> s64 {
    state := cast(*Call_State) call.state;
    if !state return 0;
    return state.options.deadline_ms;
}

// Cooperative cancellation/deadline observation. CPU work must yield to the
// scheduler and poll; this does not preempt application code.
// Example: `if cancelled(call) return .{ code = .CANCELLED };`.
cancelled :: (call: *Call) -> bool {
    state := cast(*Call_State) call.state;
    if !state return true;
    if state.terminal return state.status.code != .OK;

    if rpc_remaining(state.options.deadline_ms) == 0 {
        discard := rpc_fail(state, .DEADLINE_EXCEEDED, .DEADLINE);
        free_status(*discard);
        return true;
    }

    return Http.exchange_cancelled(*state.exchange);
}

#scope_module
Call_State :: struct {
    exchange: Http.Http2_Exchange;
    channel: *Channel;
    method: Method;
    options: Call_Options;
    decoder: Envelope_Decoder;
    input: [8192] u8;
    input_at: int;
    input_count: int;
    client: bool;
    sending: bool;
    receiving: bool;
    has_head: bool;
    invalid_media: bool;
    http_status: int;
    trailers_only: bool;
    initial_status: Status;
    initial_metadata: [..] Metadata_Entry;
    terminal: bool;
    status: Status;
    sent: int;
    received: int;
    send_closed: bool;
    read_closed: bool;
}

rpc_remaining :: (deadline: s64) -> s64 {
    if deadline == 0 return Io.NO_DEADLINE;
    now := to_milliseconds(current_time_monotonic());
    if deadline <= now return 0;
    return deadline - now;
}

encode_timeout :: (milliseconds: s64) -> string {
    value := milliseconds;
    unit := "m";

    if value > 99999999 {
        value = value / 1000 + cast(s64) (value % 1000 != 0);
        unit = "S";
    }

    if value > 99999999 {
        value = value / 60 + cast(s64) (value % 60 != 0);
        unit = "M";
    }

    if value > 99999999 {
        value = value / 60 + cast(s64) (value % 60 != 0);
        unit = "H";
    }

    return tprint("%%", min(value, 99999999), unit);
}

rpc_transport_status :: (error: Http.Client_Error) -> Status {
    if error.kind == .TOO_LARGE return .{ code = .RESOURCE_EXHAUSTED, cause = .LIMIT };
    if error.kind == .TIMED_OUT return .{ code = .DEADLINE_EXCEEDED, cause = .DEADLINE };
    if error.transport == .CANCELED return .{ code = .CANCELLED, cause = .CANCELLED };
    return .{ code = .UNAVAILABLE, cause = .TRANSPORT };
}

rpc_fail :: (state: *Call_State, code: Status_Code, cause: Local_Cause) -> Status {
    return rpc_terminal(state, .{ code = code, cause = cause });
}

rpc_terminal :: (state: *Call_State, status: Status) -> Status {
    if !state.terminal {
        state.status = status;
        state.terminal = true;
        Http.exchange_cancel(*state.exchange);
    }

    return clone_status(state.status);
}

rpc_exchange_status :: (state: *Call_State, error: Http.Client_Error) -> Status {
    if rpc_remaining(state.options.deadline_ms) == 0 return .{ code = .DEADLINE_EXCEEDED, cause = .DEADLINE };

    result := rpc_transport_status(error);
    if result.cause != .TRANSPORT return result;

    terminal := Http.exchange_terminal(*state.exchange);

    result.http2_code = terminal.code;
    result.not_processed = terminal.not_processed;
    if terminal.kind == .RESET result.code = http2_reset_status(terminal.code);
    else if terminal.kind == .GOAWAY result.code = .UNAVAILABLE;
    else if !state.client result.code = .CANCELLED;

    return result;
}

rpc_record_transport :: (state: *Call_State, error: *Http.Client_Error) {
    status := rpc_exchange_status(state, error.*);
    Http.free_error(error);
    discard := rpc_terminal(state, status);
    free_status(*discard);
}

clone_status :: (status: Status) -> Status {
    result := status;
    result.message = copy_string(status.message);
    result.status_details = copy_string(status.status_details);
    result.metadata = .{};
    for status.metadata array_add(*result.metadata, .{ copy_string(it.name), copy_string(it.value) });

    return result;
}

// Validate and retain the initial block once, independently of message assembly.
rpc_receive_head :: (state: *Call_State, remaining: s64) -> bool {
    status: int;
    fields: [] Http.Header_Field;
    error: Http.Client_Error;

    Io.with_deadline(remaining, #code {
        status, fields, error = Http.exchange_head(*state.exchange);
    });

    if error.kind != .NONE {
        rpc_record_transport(state, *error);
        return false;
    }

    state.http_status = status;
    state.has_head = true;
    metadata, metadata_error := metadata_decode(fields, state.options.max_metadata);
    if metadata_error != .NONE {
        discard := rpc_fail(state, ifx metadata_error == .LIMIT then Status_Code.RESOURCE_EXHAUSTED else Status_Code.INTERNAL, ifx metadata_error == .LIMIT then Local_Cause.LIMIT else Local_Cause.PROTOCOL);
        free_status(*discard);
        return false;
    }

    state.initial_metadata = metadata;
    content_types := 0;

    for fields if it.name == "content-type" {
        content_types += 1;
        if !protobuf_content_type(it.value) state.invalid_media = true;
    }

    if content_types != 1 state.invalid_media = true;

    for fields {
        if it.name == "grpc-encoding" && it.value != "identity" {
            discard := rpc_fail(state, .UNIMPLEMENTED, .PROTOCOL);
            free_status(*discard);
            return false;
        }
    }

    for fields if it.name == "grpc-status" state.trailers_only = true;

    if state.trailers_only {
        if !Http.exchange_head_ended(*state.exchange) {
            discard := rpc_fail(state, .INTERNAL, .PROTOCOL);
            free_status(*discard);
            return false;
        }

        state.initial_status = parse_status(fields, status, state.options.max_metadata);
    }

    return true;
}

// A peer can finish before consuming the upload. Stopping that direction is
// not an RPC failure: its already-complete response remains authoritative.
rpc_write_error :: (state: *Call_State, error: Http.Client_Error) -> Status {
    if state.client && Http.exchange_response_ended(*state.exchange) {
        state.send_closed = true;
        return .{ code = .FAILED_PRECONDITION };
    }

    return rpc_terminal(state, rpc_exchange_status(state, error));
}
