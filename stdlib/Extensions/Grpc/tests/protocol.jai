// test: skip_on(.WASM, reason = "native RPC protocol peers")
#load "../../Http/tests/support/server_support.jai";
Tls :: #import "Extensions/Tls";
G :: #import "Extensions/Grpc";
Api :: #import "Extensions/Protobuf/Schema"(TEXT = "syntax='proto3'; message M { bytes value=1; }");
#load "../../Tls/tests/fixtures/certs.jai";

bad_peer :: (request: *Request) -> Response {
    initial: [..] Header_Field;
    initial.allocator = temp;
    array_add(*initial, .{ name = "content-type", value = ifx request.path == "/content-type" then "text/plain" else "application/grpc+proto" });
    if request.path == "/initial-status" array_add(*initial, .{ name = "grpc-status", value = "0" });
    if request.path == "/initial-metadata" array_add(*initial, .{ name = "bad-bin", value = "!" });

    exchange, ok := accept_http2_exchange(request, initial, ifx request.path == "/http404" then 404 else 200);

    assert(ok);
    defer close_exchange(*exchange);
    // Consume the valid unary request before manufacturing a response fault.
    buffer: [64] u8;

    while true {
        count, error := exchange_read(*exchange, buffer);
        assert(error.kind == .NONE);
        if count == 0 break;
    }

    frame, failure := G.frame_message(.[]);

    assert(failure == .NONE);
    defer free(frame.data);

    if request.path == "/zero" || request.path == "/http404" {} else if request.path == "/truncated" assert(exchange_write(*exchange, u8.[0, 0, 0, 0, 2, 1]).kind == .NONE);
    else if request.path == "/compressed" assert(exchange_write(*exchange, u8.[1, 0, 0, 0, 0]).kind == .NONE);
    else if request.path == "/flag" assert(exchange_write(*exchange, u8.[2, 0, 0, 0, 0]).kind == .NONE);
    else if request.path == "/large" assert(exchange_write(*exchange, u8.[0, 0, 0, 0, 17]).kind == .NONE);
    else if request.path == "/protobuf" assert(exchange_write(*exchange, u8.[0, 0, 0, 0, 1, 0]).kind == .NONE);
    else {
        assert(exchange_write(*exchange, frame).kind == .NONE);
        if request.path == "/two" assert(exchange_write(*exchange, frame).kind == .NONE);
    }

    fields: [..] Header_Field;
    fields.allocator = temp;
    if request.path != "/missing" && request.path != "/http404" array_add(*fields, .{ name = "grpc-status", value = ifx request.path == "/initial-status" then "13" else "0" });
    if request.path == "/metadata" array_add(*fields, .{ name = "bad-bin", value = "!" });

    assert(exchange_finish(*exchange, fields));

    return status_only(200);
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

    for path: string.["/zero", "/two", "/truncated", "/flag", "/large", "/missing", "/metadata", "/protobuf", "/content-type", "/compressed", "/initial-metadata", "/http404", "/initial-status"] {
        call, opened := G.open_call(*channel, .{ path = path }, .{ max_message = 16 });

        assert(opened.code == .OK);
        sent := G.send_bytes(*call, .[]);

        assert(sent.code == .OK);
        half := G.close_send(*call);

        assert(half.code == .OK);

        if path == "/protobuf" {
            _, kind := G.receive(*call, Api.M);
            assert(kind == .ERROR);
        }

        status := G.finish(*call);
        expected := ifx path == "/large" then G.Status_Code.RESOURCE_EXHAUSTED else ifx path == "/missing" || path == "/content-type" then G.Status_Code.UNKNOWN else ifx path == "/compressed" || path == "/http404" then G.Status_Code.UNIMPLEMENTED else G.Status_Code.INTERNAL;

        assert(status.code == expected, "%: got %, expected %", path, status.code, expected);
        if path == "/initial-metadata" assert(status.cause == .PROTOCOL);

        G.close(*call);
        G.free_status(*status);
    }

    assert(channel.active_calls == 0);
}

app: App;

main :: () {
    if !supported() return;

    register_handler(*app, .POST, "/{path...}", (request: *Request, _data: *void) -> Response {
        return bad_peer(request);
    }, null);
    config: Tls.Tls_Config;
    config.certificate_chain_pem = SERVER_PEM;
    config.private_key_pem = SERVER_KEY;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    app.config.tls = *config;
    app.config.write_timeout = 0;
    run_server(*app, exercise);
    finish();
}
