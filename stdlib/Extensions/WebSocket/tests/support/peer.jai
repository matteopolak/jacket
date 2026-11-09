// Echo peer for external conformance, not a timing benchmark.
// Example: jaic stdlib/Extensions/WebSocket/tests/support/peer.jai -exe target/ws-peer
#import "File";
#load "echo.jai";

peer_args: [] string;

main :: () {
    peer_args = get_command_line_arguments();

    assert(peer_args.count >= 3);
    config.compression = peer_args.count > 3 && peer_args[3] == "deflate";
    config.protocols = string.["echo"];
    config.handshake_timeout = 5000;
    config.read_timeout = 5000;
    config.close_timeout = 1000;
    config.limits.max_frame_size = 4096;
    Io.io_run(() {
        if peer_args[1] == "server" {
            app: Http.App;

            defer Http.free_app(*app);
            app.config.address = "127.0.0.1:0";
            app.config.handle_signals = false;
            tls: Tls.Tls_Config;

            if peer_args.count > 4 {
                tls.certificate_chain_pem = fixture("server.pem");
                tls.private_key_pem = fixture("server.key");
                tls.alpn = string.["h2", "http/1.1"];
                assert(Tls.tls_config_init(*tls).kind == .NONE);
                app.config.tls = *tls;
            }

            defer if tls.native Tls.tls_config_free(*tls);
            Http.get(*app, "/", route);
            server, error := Http.start(*app);

            assert(error.kind == .NONE);
            defer Http.close_server(*server);
            assert(write_entire_file(peer_args[2], Io.address_to_string(server.address)));
            while true Io.sleep(1000);
        } else {
            roots: Tls.Tls_Config;
            roots.roots_pem = fixture("ca.pem");
            roots.alpn = string.["h2", "http/1.1"];
            assert(Tls.tls_config_init(*roots).kind == .NONE);
            defer Http.release_tls_config(*roots);
            api: Http.Client;
            api.tls = *roots;
            connection, error := ws_connect(peer_args[2], config, *api);

            assert(error.kind == .NONE, "%", error.message);
            defer ws_dispose(*connection);
            assert(connection.protocol == "echo");

            for message: string.["hello", "日本語😀", "", "repeat repeat repeat repeat repeat"] {
                assert(ws_write(*connection, cast([] u8) message).kind == .NONE);
                event := ws_read(*connection);
                assert(event.kind == .MESSAGE && cast(string) event.data == message);
            }

            big := alloc_string(100000);

            defer free(big);
            for *big it.* = #char "x";

            for 0..2 {
                assert(ws_write(*connection, cast([] u8) big, .BINARY).kind == .NONE);
                event := ws_read(*connection);
                assert(event.kind == .MESSAGE && cast(string) event.data == big);
            }

            assert(ws_write(*connection, cast([] u8) "ping", .PING).kind == .NONE);
            assert(ws_write(*connection, cast([] u8) "after ping").kind == .NONE);
            assert(cast(string) ws_read(*connection).data == "after ping");
            assert(ws_close(*connection, 1000, "done").kind == .NONE);
        }
    });
}
