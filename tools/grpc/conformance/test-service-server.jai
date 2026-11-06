// Canonical TestService server; compile with the pinned GRPC_PROTO_ROOT.
// Usage: test-service-server raw|typed READY_FILE STOP_FILE
Http :: #import "Extensions/Http";
#load "test-service-common.jai";
#load "../../../stdlib/Extensions/Tls/tests/fixtures/certs.jai";

select_metadata :: (call: *$T) -> G.Status {
    entries: [..] G.Metadata_Entry;
    entries.allocator = temp;
    for metadata(call) if it.name == "x-grpc-test-echo-initial" array_add(*entries, it);

    #if T == G.Call return G.set_initial_metadata(call, entries);
    else return T.set_initial_metadata(call, entries);
}

result :: (call: *$T, expected: *Api.grpc.testing.EchoStatus = null) -> G.Status {
    status: G.Status;

    if expected {
        status.code = cast(G.Status_Code) expected.code;
        status.message = tprint("%", expected.message);
    }

    for metadata(call) if it.name == "x-grpc-test-echo-trailing-bin" array_add(*status.metadata, it,, temp);

    return status;
}

empty :: (call: *$T) -> G.Status {
    initial := select_metadata(call);
    if initial.code != .OK return initial;

    request, kind := read_message(call, Api.grpc.testing.Empty);
    if kind != .MESSAGE return .{ code = .INTERNAL };

    defer P.free(*request);

    return write_message(call, Api.grpc.testing.Empty.{});
}

unary :: (call: *$T) -> G.Status {
    initial := select_metadata(call);
    if initial.code != .OK return initial;

    request, kind := read_message(call, Api.grpc.testing.SimpleRequest);
    if kind != .MESSAGE return .{ code = .INTERNAL };

    defer P.free(*request);
    if request.expect_compressed && request.expect_compressed.value != compressed_request(call) return .{ code = .INVALID_ARGUMENT };
    if request.response_status return result(call, request.response_status);
    if request.response_size < 0 || request.response_size > 4 * 1024 * 1024 return .{ code = .INVALID_ARGUMENT };

    bytes := NewArray(request.response_size, u8);

    defer free(bytes.data);
    payload := Api.grpc.testing.Payload.{ body = bytes };
    response := Api.grpc.testing.SimpleResponse.{ payload = *payload };
    sent := write_message(call, response, request.response_compressed && request.response_compressed.value);

    if sent.code != .OK return sent;

    return result(call);
}

input_stream :: (call: *$T) -> G.Status {
    initial := select_metadata(call);
    if initial.code != .OK return initial;

    total: s32;

    while true {
        request, kind := read_message(call, Api.grpc.testing.StreamingInputCallRequest);
        if kind == .END break;
        if kind != .MESSAGE return .{ code = .INTERNAL };

        defer P.free(*request);
        if request.expect_compressed && request.expect_compressed.value != compressed_request(call) return .{ code = .INVALID_ARGUMENT };
        if request.payload total += cast(s32) request.payload.body.count;
    }

    return write_message(call, Api.grpc.testing.StreamingInputCallResponse.{ aggregated_payload_size = total });
}

output_stream :: (call: *$T, bidi: bool) -> G.Status {
    initial := select_metadata(call);
    if initial.code != .OK return initial;

    while true {
        request, kind := read_message(call, Api.grpc.testing.StreamingOutputCallRequest);
        if kind == .END return result(call);
        if kind != .MESSAGE return .{ code = .INTERNAL };

        defer P.free(*request);
        if request.response_status return result(call, request.response_status);

        for request.response_parameters {
            if it.interval_us > 0 Io.sleep((cast(s64) it.interval_us + 999) / 1000);
            if it.size < 0 || it.size > 4 * 1024 * 1024 return .{ code = .INVALID_ARGUMENT };

            bytes := NewArray(it.size, u8);

            defer free(bytes.data);
            payload := Api.grpc.testing.Payload.{ body = bytes };
            response := Api.grpc.testing.StreamingOutputCallResponse.{ payload = *payload };

            if it.fill_peer_socket_address && it.fill_peer_socket_address.value response.peer_socket_address = peer(call);

            sent := write_message(call, response, it.compressed && it.compressed.value);
            if sent.code != .OK return sent;
        }

        if !bidi return result(call);
    }

    return .{ code = .INTERNAL };
}

Empty :: (call: *Api.grpc.testing.TestService.EmptyCall_Server, _: *void) -> G.Status {
    return empty(call);
}

Unary :: (call: *Api.grpc.testing.TestService.UnaryCall_Server, _: *void) -> G.Status {
    return unary(call);
}

Input :: (call: *Api.grpc.testing.TestService.StreamingInputCall_Server, _: *void) -> G.Status {
    return input_stream(call);
}

Output :: (call: *Api.grpc.testing.TestService.StreamingOutputCall_Server, _: *void) -> G.Status {
    return output_stream(call, false);
}

Bidi :: (call: *Api.grpc.testing.TestService.FullDuplexCall_Server, _: *void) -> G.Status {
    return output_stream(call, true);
}

raw_handler :: (call: *G.Call, data: *void) -> G.Status {
    mode := (cast(*int) data).*;
    if mode == 0 return empty(call);
    if mode == 1 return unary(call);
    if mode == 2 return input_stream(call);
    if mode == 3 return output_stream(call, false);

    return output_stream(call, true);
}

Raw_Route :: struct {
    method: G.Method;
    mode: int;
}

raw_route :: (request: *Http.Request, data: *void) -> Http.Response {
    route := cast(*Raw_Route) data;
    return G.serve_call(request, route.method, raw_handler, *route.mode, .{ compression = .GZIP });
}

Control :: struct {
    app: *Http.App;
    ready: string;
    stop: string;
}

serve :: (control: *Control) {
    server, error := Http.start(control.app);

    assert(error.kind == .NONE);
    assert(write_entire_file(control.ready, tprint("%", server.address.port)));
    while !file_exists(control.stop) Io.sleep(10);

    Http.stop(server);
    Http.wait(server);
    Http.close_server(*server);
}

main :: () {
    args := get_command_line_arguments();

    defer free(args.data);
    assert(args.count == 4, "supply raw/typed, ready-file and stop-file");
    app: Http.App;

    defer Http.free_app(*app);
    handlers := Api.grpc.testing.TestService.Handlers.{ options = .{ compression = .GZIP }, EmptyCall = Empty, UnaryCall = Unary, StreamingInputCall = Input, StreamingOutputCall = Output, FullDuplexCall = Bidi };
    routes := Raw_Route.[.{ Api.grpc.testing.TestService.EmptyCall, 0 }, .{ Api.grpc.testing.TestService.UnaryCall, 1 }, .{ Api.grpc.testing.TestService.StreamingInputCall, 2 }, .{ Api.grpc.testing.TestService.StreamingOutputCall, 3 }, .{ Api.grpc.testing.TestService.FullDuplexCall, 4 }];

    if args[1] == "typed" Api.grpc.testing.TestService.register(*app, *handlers);
    else {
        assert(args[1] == "raw");
        for *routes Http.register_handler(*app, .POST, it.method.path, raw_route, it);
    }

    config: Tls.Tls_Config;
    config.certificate_chain_pem = SERVER_PEM;
    config.private_key_pem = SERVER_KEY;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    app.config.tls = *config;
    app.config.handle_signals = false;
    app.config.address = "127.0.0.1:0";
    app.config.write_timeout = 0;
    app.config.h2_max_streams = 2;
    control := Control.{ *app, args[2], args[3] };

    Io.io_run(serve, *control);
}
