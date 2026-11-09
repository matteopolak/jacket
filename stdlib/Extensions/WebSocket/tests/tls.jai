// test: skip_on(.WASM, reason = "needs TLS and Io fibers")
#import "Basic";
#import "Extensions/WebSocket";
Http :: #import "Extensions/Http";
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";
#load "../../Tls/tests/fixtures/certs.jai";

fixture :: (name: string) -> string {
    for DATA if it.name == name return it.text;
    return "";
}

echo :: (connection: *Ws_Connection, data: *void) {
    event := ws_read(connection);
    assert(event.kind == .MESSAGE);
    assert(ws_write(connection, event.data, event.opcode).kind == .NONE);
    assert(ws_read(connection).kind == .CLOSE);
}

route :: (request: *Http.Request) -> Http.Response {
    return ws_upgrade(request, echo);
}

main :: () {
    Io.io_run(() {
        tls: Tls.Tls_Config;
        tls.certificate_chain_pem = fixture("server.pem");
        tls.private_key_pem = fixture("server.key");
        tls.alpn = string.["h2", "http/1.1"];
        assert(Tls.tls_config_init(*tls).kind == .NONE);
        defer Tls.tls_config_free(*tls);
        roots: Tls.Tls_Config;
        roots.roots_pem = fixture("ca.pem");
        roots.alpn = string.["h2", "http/1.1"];
        assert(Tls.tls_config_init(*roots).kind == .NONE);
        defer Http.release_tls_config(*roots);
        app: Http.App;
        app.config.address = "127.0.0.1:0";
        app.config.handle_signals = false;
        app.config.tls = *tls;
        Http.get(*app, "/", route);
        server, failure := Http.start(*app);

        assert(failure.kind == .NONE);
        defer Http.close_server(*server);
        api: Http.Client;
        api.tls = *roots;
        connection, error := ws_connect(tprint("wss://%/", Io.address_to_string(server.address)), .{ handshake_timeout = 500 }, *api);

        assert(error.kind == .NONE, "%", error.message);
        assert(ws_write(*connection, cast([] u8) "secure").kind == .NONE);
        event := ws_read(*connection);

        assert(event.kind == .MESSAGE && cast(string) event.data == "secure");
        assert(ws_close(*connection).kind == .NONE);
    });
}
