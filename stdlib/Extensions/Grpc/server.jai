// Handlers run on the request fiber. Join any child sender/receiver fibers before
// returning. Storage referenced by their returned Status remains borrowed until
// serve_call returns after encoding it.
Call_Handler :: #type (call: *Call, data: *void) -> Status;

// Adapts a typed/generated route to the common RPC lifecycle. Identity Protobuf
// transport is accepted; malformed media types use HTTP 415, and unsupported
// compression receives UNIMPLEMENTED. The handler's state belongs to its caller.
// Example: `return serve_call(request, method, handler, *service_state);`.
serve_call :: (request: *Http.Request, method: Method, handler: Call_Handler, data: *void, options := Call_Options.{}) -> Http.Response {
    content_type, _ := Http.header(request, "content-type");
    if request.method != .POST || !protobuf_content_type(content_type) return Http.status_only(415);

    settings := options;
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
    array_add(*initial, .{ name = "grpc-accept-encoding", value = "identity" });
    configured, initial_valid := encode_metadata(settings.metadata, settings.max_metadata);
    if initial_valid for configured array_add(*initial, it);

    if !metadata_headers_fit(initial, settings.max_metadata) {
        initial_valid = false;
        initial.count = 2;
    }

    exchange, ok := Http.accept_http2_exchange(request, initial);
    if !ok return Http.status_only(505);

    state := New(Call_State);
    state.exchange = exchange;
    state.method = method;
    state.options = settings;
    state.decoder.max_message = settings.max_message;
    call := Call.{ state };

    defer close(*call);
    status: Status;
    encoding, has_encoding := Http.header(request, "grpc-encoding");

    if !initial_valid status.code = .INTERNAL;
    else if has_encoding && encoding != "identity" status.code = .UNIMPLEMENTED;
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
