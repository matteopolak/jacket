// Run with jacket bench --path stdlib/Extensions/WebSocket.
#import "Extensions/Testing";
#load "../tests/support/echo.jai";

websocket_case :: (b: *Bench, compression: bool, secure: bool, input: int) {
    state := Ws_Bench.{ b, compression, secure, input };

    Io.io_run((state: *Ws_Bench) {
        b := state.b;

        config = .{};
        config.compression = state.compression;
        config.protocols = string.["echo"];
        config.handshake_timeout = 5000;
        config.read_timeout = 5000;
        config.close_timeout = 1000;
        config.limits.max_frame_size = 4096;
        app: Http.App;

        defer Http.free_app(*app);
        app.config.address = "127.0.0.1:0";
        app.config.handle_signals = false;
        tls: Tls.Tls_Config;
        roots: Tls.Tls_Config;

        if state.secure {
            tls.certificate_chain_pem = fixture("server.pem");
            tls.private_key_pem = fixture("server.key");
            assert(Tls.tls_config_init(*tls).kind == .NONE);
            roots.roots_pem = fixture("ca.pem");
            assert(Tls.tls_config_init(*roots).kind == .NONE);
            app.config.tls = *tls;
        }

        defer if tls.native Tls.tls_config_free(*tls);
        defer if roots.native Http.release_tls_config(*roots);
        Http.get(*app, "/", route);
        server, error := Http.start(*app);

        assert(error.kind == .NONE);

        defer {
            Http.stop(server);
            Http.wait(server);
        }

        api: Http.Client;
        if state.secure api.tls = *roots;

        url := tprint("%://localhost:%/", ifx state.secure then "wss" else "ws", server.address.port);
        connection, failure := ws_connect(url, config, *api);

        assert(failure.kind == .NONE, "%", failure.message);
        defer ws_dispose(*connection);
        big := alloc_string(100000);

        defer free(big);
        for *big it.* = #char "x";

        message := string.["hello", "日本語😀", "", "repeat repeat repeat repeat repeat", big, "after ping"][state.input];

        set_bytes(b, message.count);
        reset_timer(b);

        for 0..b.iterations - 1 {
            if state.input == 5 assert(ws_write(*connection, cast([] u8) black_box("ping"), .PING).kind == .NONE);

            opcode: Ws_Opcode = ifx state.input == 4 then .BINARY else .TEXT;

            assert(ws_write(*connection, cast([] u8) black_box(message), opcode).kind == .NONE);
            event := ws_read(*connection);

            assert(event.kind == .MESSAGE && cast(string) event.data == message);
            black_box(event);
        }

        assert(ws_close(*connection, 1000, "done").kind == .NONE);
    }, *state);
}

Ws_Bench :: struct {
    b: *Bench;
    compression: bool;
    secure: bool;
    input: int;
}

websocket_benches :: () -> string {
    out: String_Builder;

    for compression: 0..1 for secure : 0..1 for input : 0..5 {
        name := string.["hello", "unicode", "empty", "repeat", "binary", "ping"][input];
        print(*out, "%_%_% :: (b: *Bench) { websocket_case(b, %, %, %); } @bench\n", name,
            ifx compression == 1 then "deflate" else "plain", ifx secure == 1 then "tls" else "tcp", compression == 1, secure == 1, input);
    }

    return builder_to_string(*out);
}

#insert #run websocket_benches();
