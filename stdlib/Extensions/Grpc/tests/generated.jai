// test: skip_on(.WASM, reason = "native RPC/HTTP scheduler")
#load "../../Http/tests/support/server_support.jai";
Tls :: #import "Extensions/Tls";
G :: #import "Extensions/Grpc";
P :: #import "Extensions/Protobuf";
Api :: #import "Extensions/Protobuf/Schema"(GRPC = true, TEXT = "syntax='proto3'; package fixture; message Payload { bytes data=1; } service Echo { rpc Unary(Payload) returns(Payload); rpc Reject(Payload) returns(Payload); rpc Bidi(stream Payload) returns(stream Payload); } service Immediate { rpc Unary(Payload) returns(Payload); }");
#load "../../Tls/tests/fixtures/certs.jai";

unary :: (call: *Api.fixture.Echo.Unary_Server, state: *void) -> G.Status {
    count := cast(*int) state;
    count.* += 1;
    message, kind := Api.fixture.Echo.Unary_Server.receive(call);
    if kind != .MESSAGE return .{ code = .INTERNAL };

    defer P.free(*message);

    return Api.fixture.Echo.Unary_Server.send(call, message);
}

// Handwritten HTTP/2 peer: a valid response may precede request completion.
immediate :: (request: *Request) -> Response {
    exchange, accepted := accept_http2_exchange(request, Header_Field.[.{ name = "content-type", value = "application/grpc+proto" }]);

    assert(accepted);
    defer close_exchange(*exchange);
    bytes, failure := P.encode(Api.fixture.Payload.{ data = u8.[9, 8, 7] });

    assert(failure.kind == .NONE);
    defer free(bytes.data);
    frame, framed := G.frame_message(bytes);

    assert(framed == .NONE);
    defer free(frame.data);
    assert(exchange_write(*exchange, frame).kind == .NONE);
    assert(exchange_finish(*exchange, Header_Field.[.{ name = "grpc-status", value = "0" }, .{ name = "x-early", value = "kept" }]));

    return status_only(200);
}

reject :: (_call: *Api.fixture.Echo.Reject_Server, _state: *void) -> G.Status {
    status := G.Status.{ code = .INVALID_ARGUMENT, message = "rejected before upload" };
    array_add(*status.metadata, .{ "x-rejected", "kept" },, temp);
    return status;
}

bidi :: (call: *Api.fixture.Echo.Bidi_Server, _state: *void) -> G.Status {
    while true {
        message, kind := Api.fixture.Echo.Bidi_Server.receive(call);
        if kind == .END break;
        if kind != .MESSAGE return .{ code = .INTERNAL };

        defer P.free(*message);
        status := Api.fixture.Echo.Bidi_Server.send(call, message);
        if status.code != .OK return status;
    }

    return .{};
}

exercise :: (base: string, _server: *Server) {
    config: Tls.Tls_Config;
    config.roots_pem = CA_PEM;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    channel: G.Channel;
    channel.transport.base_url = base;
    channel.transport.tls = *config;
    defer G.close_channel(*channel);
    response, status := Api.fixture.Echo.Unary_call(*channel, Api.fixture.Payload.{ data = u8.[1, 2, 3] });

    defer P.free(*response);
    defer G.free_status(*status);
    assert(status.code == .OK && response.data.count == 3 && response.data[2] == 3);
    local_response, local_status := Api.fixture.Echo.Reject_call(*channel, Api.fixture.Payload.{ data = u8.[1, 2, 3] }, .{ max_message = 1 });

    defer P.free(*local_response);
    defer G.free_status(*local_status);
    assert(local_status.code == .RESOURCE_EXHAUSTED && local_status.cause == .LIMIT, "true local terminal failures survive convenience finish");
    client, opened := Api.fixture.Echo.Bidi_open(*channel);

    assert(opened.code == .OK);
    defer Api.fixture.Echo.Bidi_Client.close(*client);
    sent := Api.fixture.Echo.Bidi_Client.send(*client, Api.fixture.Payload.{ data = u8.[4] });

    assert(sent.code == .OK);
    value, kind := Api.fixture.Echo.Bidi_Client.receive(*client);

    defer P.free(*value);
    assert(kind == .MESSAGE && value.data.count == 1 && value.data[0] == 4);
    large := NewArray(600 * 1024 + 3, u8);

    defer free(large.data);
    for *large it.* = cast(u8) (it_index % 251);

    early, early_status := Api.fixture.Immediate.Unary_call(*channel, Api.fixture.Payload.{ data = large });

    defer P.free(*early);
    defer G.free_status(*early_status);
    assert(early_status.code == .OK && early.data.count == 3 && early.data[0] == 9 && early.data[2] == 7, "an early successful response preserves its actual payload");
    assert(early_status.metadata.count == 1 && early_status.metadata[0].value == "kept");

    rejected, rejection := Api.fixture.Echo.Reject_call(*channel, Api.fixture.Payload.{ data = large });

    defer P.free(*rejected);
    defer G.free_status(*rejection);
    assert(rejection.code == .INVALID_ARGUMENT && rejection.message == "rejected before upload", "unary convenience preserves early remote status after blocked upload stops");
    assert(rejection.metadata.count == 1 && rejection.metadata[0].name == "x-rejected" && rejection.metadata[0].value == "kept");

    large_sent := Api.fixture.Echo.Bidi_Client.send(*client, Api.fixture.Payload.{ data = large });

    assert(large_sent.code == .OK);
    large_value, large_kind := Api.fixture.Echo.Bidi_Client.receive(*client);

    defer P.free(*large_value);
    assert(large_kind == .MESSAGE && large_value.data.count == large.count);
    for large_value.data assert(it == cast(u8) (it_index % 251));

    half := Api.fixture.Echo.Bidi_Client.close_send(*client);

    assert(half.code == .OK);
    final := Api.fixture.Echo.Bidi_Client.finish(*client);

    defer G.free_status(*final);
    assert(final.code == .OK);
}

app: App;

main :: () {
    if !supported() return;

    count := 0;
    handlers := Api.fixture.Echo.Handlers.{ state = *count, Unary = unary, Reject = reject, Bidi = bidi };

    Api.fixture.Echo.register(*app, *handlers);
    post(*app, Api.fixture.Immediate.Unary.path, immediate);
    config: Tls.Tls_Config;
    config.certificate_chain_pem = SERVER_PEM;
    config.private_key_pem = SERVER_KEY;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    app.config.tls = *config;
    app.config.write_timeout = 0;
    app.config.h2_window = 16384;
    run_server(*app, exercise);
    assert(count == 1);
    finish();
}
