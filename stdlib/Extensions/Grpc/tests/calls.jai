// test: skip_on(.WASM, reason = "native RPC/HTTP scheduler")
#load "../../Http/tests/support/server_support.jai";
Tls :: #import "Extensions/Tls";
G :: #import "Extensions/Grpc";
#load "../../Tls/tests/fixtures/certs.jai";

load :: (name: string) -> string {
    for DATA if it.name == name return it.text;
    assert(false);
    return "";
}

METHOD :: G.Method.{ path = "/fixture.Echo/Bidi", client_streaming = true, server_streaming = true };

handle_rpc :: (call: *G.Call, data: *void) -> G.Status {
    while true {
        message, kind := G.receive_bytes(call);
        if kind == .END break;
        if kind == .ERROR return .{ code = .INTERNAL };

        defer if message.data free(message.data);
        status := G.send_bytes(call, message);
        if status.code != .OK return status;
    }

    return .{};
}

handler :: (request: *Request) -> Response {
    return G.serve_call(request, METHOD, handle_rpc, null);
}

exercise :: (base: string, server: *Server) {
    config: Tls.Tls_Config;
    config.roots_pem = load("ca.pem");
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    channel: G.Channel;
    channel.transport.base_url = base;
    channel.transport.tls = *config;
    defer G.close_channel(*channel);
    call, status := G.open_call(*channel, METHOD);

    assert(status.code == .OK);
    defer G.close(*call);

    for value: string.["hello", "", "world"] {
        sent := G.send_bytes(*call, bytes_of(value));

        assert(sent.code == .OK);
        message, kind := G.receive_bytes(*call);

        assert(kind == .MESSAGE && string.{ message.count, message.data } == value);
        if message.data free(message.data);
    }

    half := G.close_send(*call);

    assert(half.code == .OK);
    final := G.finish(*call);

    defer G.free_status(*final);
    assert(final.code == .OK);
    again := G.finish(*call);

    defer G.free_status(*again);
    assert(again.code == .OK);
}

app: App;

main :: () {
    if !supported() return;

    post(*app, METHOD.path, handler);
    config: Tls.Tls_Config;
    config.certificate_chain_pem = load("server.pem");
    config.private_key_pem = load("server.key");
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    app.config.tls = *config;
    app.config.write_timeout = 0;
    run_server(*app, exercise);
    finish();
}
