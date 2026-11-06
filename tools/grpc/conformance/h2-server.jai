// TLS HTTP/2 target for the pinned h2spec suite. Supply ready-file and stop-file paths.
// A streaming drain keeps body flow control real without collecting request bodies.
#import "Basic";
#import "File";
#import "File_Utilities";
Http :: #import "Extensions/Http";
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";
#load "../../../stdlib/Extensions/Tls/tests/fixtures/certs.jai";

root :: (body: *Http.Body) -> string, Http.Http_Error {
    buffer: [4096] u8;

    while true {
        count, error := Http.read(body, buffer);
        if error.kind != .NONE return "", error;
        if count == 0 break;
    }

    return "h2spec", .{};
}

Control :: struct {
    ready: string;
    stop: string;
    app: *Http.App;
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
    assert(args.count == 3, "supply ready-file and stop-file paths");
    config: Tls.Tls_Config;
    config.certificate_chain_pem = SERVER_PEM;
    config.private_key_pem = SERVER_KEY;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    app: Http.App;

    defer Http.free_app(*app);
    app.config.address = "127.0.0.1:0";
    app.config.handle_signals = false;
    app.config.tls = *config;
    app.config.write_timeout = 5000;
    Http.get(*app, "/", root);
    Http.post(*app, "/", root);
    control := Control.{ args[1], args[2], *app };

    Io.io_run(serve, *control);
}
