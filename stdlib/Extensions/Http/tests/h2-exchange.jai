// test: skip_on(.WASM, reason = "native RPC/HTTP scheduler")
#load "support/server_support.jai";
Tls :: #import "Extensions/Tls";
#load "../../Tls/tests/fixtures/certs.jai";

load :: (name: string) -> string {
    for DATA if it.name == name return it.text;
    assert(false);
    return "";
}

raw_echo :: (request: *Request) -> Response {
    exchange, ok := accept_http2_exchange(request, Header_Field.[.{ name = "content-type", value = "application/octet-stream" }]);

    assert(ok);
    defer close_exchange(*exchange);
    assert(exchange_write(*exchange, bytes_of("ready")).kind == .NONE);
    buffer: [8192] u8;

    while true {
        count, error := exchange_read(*exchange, buffer);
        assert(error.kind == .NONE);
        if count == 0 break;
        assert(exchange_write(*exchange, .{ count, buffer.data }).kind == .NONE);
    }

    assert(exchange_finish(*exchange, Header_Field.[.{ name = "grpc-status", value = "0" }]));

    return status_only(200);
}

Upload :: struct {
    exchange: *Http2_Exchange;
    done: Io.Wait_Group;
}

upload :: (job: *Upload) {
    defer Io.done(*job.done);
    bytes := NewArray(600 * 1024 + 3, u8);

    defer free(bytes.data);
    for *bytes it.* = cast(u8) (it_index % 251);

    assert(exchange_write(job.exchange, bytes, true).kind == .NONE);
}

exercise :: (base: string, server: *Server) {
    config: Tls.Tls_Config;
    config.roots_pem = load("ca.pem");
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    channel: Http2_Channel;
    channel.base_url = base;
    channel.tls = *config;
    defer close_http2_channel(*channel);
    exchange, failure := open_http2_exchange(*channel, "/echo", .[]);

    assert(failure.kind == .NONE && channel.active_exchanges == 1);
    defer close_exchange(*exchange);
    status, _, head_error := exchange_head(*exchange);

    assert(status == 200 && head_error.kind == .NONE);
    prefix: [5] u8;
    at := 0;

    while at < 5 {
        count, error := exchange_read(*exchange, .{ 5 - at, prefix.data + at });
        assert(error.kind == .NONE && count > 0);
        at += count;
    }

    assert(string.{ 5, prefix.data } == "ready");
    // The server answered before we sent any request DATA.
    job: Upload;
    job.exchange = *exchange;
    Io.add(*job.done, 1);
    Io.spawn(upload, *job);
    buffer: [4093] u8;
    received := 0;

    while true {
        count, error := exchange_read(*exchange, buffer);

        assert(error.kind == .NONE);
        if count == 0 break;
        for i: 0..count - 1 assert(buffer[i] == cast(u8) ((received + i) % 251));

        received += count;
    }

    Io.wait(*job.done);
    assert(received == 600 * 1024 + 3);
    fields, complete := exchange_trailers(*exchange);

    assert(complete && fields.count == 1 && fields[0].value == "0");
}

fallback_invocations := 0;

fallback_handler :: (_request: *Request) -> Response {
    fallback_invocations += 1;
    return text("HTTP/1 fallback must not run");
}

reject_fallback :: (base: string, _server: *Server) {
    config: Tls.Tls_Config;
    config.roots_pem = load("ca.pem");
    config.alpn = string.["h2", "http/1.1"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    channel: Http2_Channel;
    channel.base_url = base;
    channel.tls = *config;
    channel.max_connections = 1;
    defer close_http2_channel(*channel);

    for 0..2 {
        exchange, error := open_http2_exchange(*channel, "/echo", .[]);
        defer free_error(*error);
        assert(!exchange.state && error.kind == .TLS && channel.active_exchanges == 0);
    }

    assert(fallback_invocations == 0, "required H2 does not send or replay a request over HTTP/1");
}

app: App;

main :: () {
    if !supported() return;

    post(*app, "/echo", raw_echo);
    config: Tls.Tls_Config;
    config.certificate_chain_pem = load("server.pem");
    config.private_key_pem = load("server.key");
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    app.config.tls = *config;
    run_server(*app, exercise);
    fallback_config: Tls.Tls_Config;
    fallback_config.certificate_chain_pem = load("server.pem");
    fallback_config.private_key_pem = load("server.key");
    fallback_config.alpn = string.["http/1.1"];
    assert(Tls.tls_config_init(*fallback_config).kind == .NONE);
    defer Tls.tls_config_free(*fallback_config);
    fallback_app: App;

    defer free_app(*fallback_app);
    fallback_app.config.tls = *fallback_config;
    post(*fallback_app, "/echo", fallback_handler);
    run_server(*fallback_app, reject_fallback);
    finish();
}
