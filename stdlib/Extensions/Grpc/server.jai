// Handlers run on the request fiber. Join any child sender/receiver fibers before
// returning. Storage referenced by their returned Status remains borrowed until
// serve_call returns after encoding it.
Call_Handler :: #type (call: *Call, data: *void) -> Status;

// Adapts a typed/generated route to the common RPC lifecycle. Identity/gzip Protobuf
// transport is accepted; malformed media types use HTTP 415, and unsupported
// compression receives UNIMPLEMENTED. The handler's state belongs to its caller.
// Example: `return serve_call(request, method, handler, *service_state);`.
serve_call :: (request: *Http.Request, method: Method, handler: Call_Handler, data: *void, options := Call_Options.{}) -> Http.Response {
    content_type, _ := Http.header(request, "content-type");
    if request.method != .POST || !protobuf_content_type(content_type) return Http.status_only(415);

    settings := options;
    accepted, _ := Http.header(request, "grpc-accept-encoding");

    if settings.compression == .GZIP && !accepts_gzip(accepted) settings.compression = .IDENTITY;

    timeout, has_timeout := Http.header(request, "grpc-timeout");

    if has_timeout {
        milliseconds, valid := parse_timeout(timeout);
        if !valid return Http.status_only(400);
        deadline := to_milliseconds(current_time_monotonic()) + milliseconds;
        if settings.deadline_ms == 0 || deadline < settings.deadline_ms settings.deadline_ms = deadline;
    }

    initial: [..] Http.Header_Field;
    initial.allocator = temp;
    array_add(*initial, .{ name = "content-type", value = "application/grpc+proto" });
    array_add(*initial, .{ name = "grpc-accept-encoding", value = "identity,gzip" });
    if settings.compression == .GZIP array_add(*initial, .{ name = "grpc-encoding", value = "gzip" });

    base_count := initial.count;
    configured, initial_valid := encode_metadata(settings.metadata, settings.max_metadata);

    if initial_valid for configured array_add(*initial, it);

    if !metadata_headers_fit(initial, settings.max_metadata) {
        initial_valid = false;
        initial.count = base_count;
    }

    exchange, ok := Http.accept_http2_exchange(request, initial, defer_headers = true);
    if !ok return Http.status_only(505);

    state := New(Call_State);
    state.exchange = exchange;
    state.method = method;
    state.options = settings;
    state.peer = request.peer;
    state.decoder.max_message = settings.max_message;
    state.decoder.max_wire_message = settings.max_wire_message;
    call := Call.{ state };

    defer close(*call);
    status: Status;
    encoding, has_encoding := Http.header(request, "grpc-encoding");

    if has_encoding && encoding == "gzip" state.decoder.compression = .GZIP;

    if !initial_valid status.code = .INTERNAL;
    else if has_encoding && encoding != "identity" && encoding != "gzip" status.code = .UNIMPLEMENTED;
    else if rpc_remaining(settings.deadline_ms) == 0 status.code = .DEADLINE_EXCEEDED;
    else {
        metadata, metadata_error := metadata_decode(request.headers, settings.max_metadata);

        state.initial_metadata = metadata;

        if metadata_error != .NONE status.code = ifx metadata_error == .LIMIT then Status_Code.RESOURCE_EXHAUSTED else Status_Code.INVALID_ARGUMENT;
        else {
            Io.with_deadline(rpc_remaining(settings.deadline_ms), #code {
                status = handler(*call, data);
            });
            if rpc_remaining(settings.deadline_ms) == 0 status = .{ code = .DEADLINE_EXCEEDED, cause = .DEADLINE };
        }
    }

    if state.terminal status = state.status;

    if status.code == .OK && !method.client_streaming && !state.read_closed {
        bytes, kind := receive_bytes(*call);
        if bytes.data free(bytes.data);
        if kind != .END status.code = .INTERNAL;
    }

    if status.code == .OK && !method.client_streaming && state.received != 1 status.code = .INTERNAL;
    if status.code == .OK && !method.server_streaming && state.sent != 1 status.code = .INTERNAL;

    fields: [..] Http.Header_Field;
    fields.allocator = temp;
    array_add(*fields, .{ name = "grpc-status", value = tprint("%", cast(int) status.code) });

    if status.message.count {
        message, fits := encode_status_message(status.message, settings.max_metadata);
        if fits array_add(*fields, .{ name = "grpc-message", value = message });
        else {
            fields.count = 0;
            array_add(*fields, .{ name = "grpc-status", value = "8" });
        }
    }

    if status.status_details.count {
        encoded_size := (status.status_details.count / 3) * 4 + (ifx status.status_details.count % 3 == 0 then 0 else status.status_details.count % 3 + 1);
        if encoded_size > settings.max_metadata {
            fields.count = 0;
            array_add(*fields, .{ name = "grpc-status", value = "8" });
        } else if status.code == .OK || !valid_status_details(status.status_details, status.code) {
            fields.count = 0;
            array_add(*fields, .{ name = "grpc-status", value = "13" });
        } else array_add(*fields, .{ name = "grpc-status-details-bin", value = Base64.base64_encode(status.status_details, pad = false,, temp) });
    }

    metadata, valid := encode_metadata(status.metadata, settings.max_metadata);
    if valid for metadata array_add(*fields, it);
    else {
        fields.count = 0;
        array_add(*fields, .{ name = "grpc-status", value = "13" });
    }

    if !metadata_headers_fit(fields, settings.max_metadata) {
        fields.count = 0;
        array_add(*fields, .{ name = "grpc-status", value = "8" });
    }

    // An already-reset exchange cannot send status; close still releases it.
    remaining := rpc_remaining(settings.deadline_ms);
    if remaining == 0 remaining = 1;

    Io.with_deadline(remaining, #code {
        Http.exchange_finish(*state.exchange, fields);
    });
    state.terminal = true;

    return Http.status_only(200);
}

// The server's observed remote socket address, copied by value. A client call or
// cleared handle returns an unspecified address; this does not identify a user.
// Example: `text := Io.address_to_string(peer_address(call));`.
peer_address :: (call: *Call) -> Io.Address {
    state := cast(*Call_State) call.state;
    if !state || state.client return .{};
    return state.peer;
}

// Replaces application response metadata before the first send or handler return.
// Entries are copied by the exchange; the caller may immediately release them.
// Invalid metadata leaves the previous block intact. Client calls and committed
// response headers return FAILED_PRECONDITION without cancelling the RPC.
// Example: `status := set_initial_metadata(call, Metadata_Entry.[.{"x-id", "123"}]);`.
set_initial_metadata :: (call: *Call, entries: [] Metadata_Entry) -> Status {
    state := cast(*Call_State) call.state;
    if !state || state.client || state.terminal || state.sending return .{ code = .FAILED_PRECONDITION };

    metadata, error := metadata_encode(entries, state.options.max_metadata);
    if error != .NONE return .{ code = ifx error == .LIMIT then Status_Code.RESOURCE_EXHAUSTED else Status_Code.INVALID_ARGUMENT };

    fields: [..] Http.Header_Field;
    fields.allocator = temp;
    array_add(*fields, .{ name = "content-type", value = "application/grpc+proto" });
    array_add(*fields, .{ name = "grpc-accept-encoding", value = "identity,gzip" });
    if state.options.compression == .GZIP array_add(*fields, .{ name = "grpc-encoding", value = "gzip" });
    for metadata array_add(*fields, it);
    if !metadata_headers_fit(fields, state.options.max_metadata) return .{ code = .RESOURCE_EXHAUSTED };
    if !Http.exchange_response_headers(*state.exchange, fields) return .{ code = .FAILED_PRECONDITION };

    return .{};
}

#scope_module
encode_status_message :: (message: string, limit: s64) -> string, bool {
    if message.count > limit return "", false;

    builder: String_Builder;

    defer free_buffers(*builder);
    used: s64;
    HEX :: "0123456789ABCDEF";

    for message {
        cost := ifx it >= 32 && it <= 126 && it != #char "%" then 1 else 3;
        if cost > limit - used return "", false;

        used += cost;

        if it >= 32 && it <= 126 && it != #char "%" append(*builder, string.{ 1, *it });
        else {
            append(*builder, "%");
            c := HEX[it >> 4];

            append(*builder, string.{ 1, *c });
            c = HEX[it & 15];
            append(*builder, string.{ 1, *c });
        }
    }

    return builder_to_string(*builder,, temp), true;
}

// Compression names are comma-separated tokens; surrounding HTTP whitespace is optional.
accepts_gzip :: (value: string) -> bool {
    start := 0;

    for end: 0..value.count {
        if end < value.count && value[end] != #char "," continue;

        first := start;
        last := end;

        while first < last && (value[first] == #char " " || value[first] == #char "\t") first += 1;
        while last > first && (value[last - 1] == #char " " || value[last - 1] == #char "\t") last -= 1;
        if slice(value, first, last - first) == "gzip" return true;

        start = end + 1;
    }

    return false;
}
