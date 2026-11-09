// test: skip_on(.WASM, reason = "needs Io fibers")
#import "Basic";
#import "Extensions/WebSocket";
Http :: #import "Extensions/Http";
Io :: #import "Extensions/Io";

echo :: (connection: *Ws_Connection, data: *void) {
    event := ws_read(connection);

    assert(event.kind == .MESSAGE && cast(string) event.data == "hello");
    error := ws_write(connection, event.data, event.opcode);

    assert(error.kind == .NONE);
    event = ws_read(connection);
    assert(event.kind == .CLOSE && event.code == 1000);
}

route :: (request: *Http.Request) -> Http.Response {
    return ws_upgrade(request, echo);
}

main :: () {
    Io.io_run(() {
        app: Http.App;

        Http.get(*app, "/", route);
        api := Http.test_client(*app);
        connection, error := ws_connect("ws://localhost/", .{}, api);

        assert(error.kind == .NONE);
        error = ws_write(*connection, cast([] u8) "hello", .TEXT);
        assert(error.kind == .NONE);
        event := ws_read(*connection);

        assert(event.kind == .MESSAGE && cast(string) event.data == "hello");
        error = ws_close(*connection);
        assert(error.kind == .NONE, "%: %", error.kind, error.message);
    });
}
