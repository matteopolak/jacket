// test: skip_on(.WASM, reason = "needs Http and Io fibers")
#import "Basic";
#import "Extensions/WebSocket";
Http :: #import "Extensions/Http";
Io :: #import "Extensions/Io";

mode: int;

mock_server :: (pipe: *Io.Pipe_Stream) {
    defer Io.close(pipe);
    parser: Http.Parser;

    Http.parser_init(*parser, .REQUEST);
    defer Http.parser_free(*parser);
    buffer: [4096] u8;

    while true {
        count, error := Io.read(pipe, buffer);
        if error != .NONE || count == 0 return;
        _, event := Http.parse(*parser, .{ count, buffer.data });
        if event.kind == .HEAD break;
    }

    if mode == 4 {
        Io.sleep(1000);
        return;
    }

    key, _ := Http.find_header(parser.head.headers, "Sec-WebSocket-Key");
    accept := ws_accept_key(key);

    defer free(accept);
    if mode == 1 accept.data[0] = #char "!";

    protocol := "Sec-WebSocket-Protocol: echo\r\n";
    if mode == 2 protocol = "Sec-WebSocket-Protocol: echo\r\nSec-WebSocket-Protocol: echo\r\n";

    extension := "";
    if mode == 3 extension = "Sec-WebSocket-Extensions: unknown\r\n";

    wire: [..] u8;

    defer array_free(wire);
    head := tprint("HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: %\r\n%%\r\n", accept, protocol, extension);
    for cast([] u8) head array_add(*wire, it);
    if mode == 6 ws_frame(*wire, .PING, cast([] u8) "ping");

    ws_frame(*wire, .TEXT, cast([] u8) "early");
    Io.write(pipe, wire);

    if mode == 6 {
        codec: Ws_Codec;

        defer ws_free(*codec);
        count, error := Io.read(pipe, buffer);

        assert(error == .NONE);
        _, event := ws_parse(*codec, .{ count, buffer.data });

        assert(event.kind == .PONG && cast(string) event.data == "ping");

        return;
    }

    while true {
        count, error := Io.read(pipe, buffer);
        if count == 0 || error != .NONE return;
    }
}

mock_open :: (data: *void, url: Http.Url) -> Http.Link, Io.Io_Error {
    near, far := Io.make_pipe();
    Io.spawn(mock_server, far);
    return Http.pipe_link(near), .NONE;
}

main :: () {
    Io.io_run(() {
        api: Http.Client;
        api.transport.open = mock_open;

        for i: 0..6 {
            mode = i;
            connection, error := ws_connect("ws://localhost/", .{ protocols = string.["echo"], compression = false, handshake_timeout = 10, close_timeout = 10 }, *api);

            if i >= 1 && i <= 4 {
                assert(error.kind != .NONE, "mode % accepted invalid handshake", i);
                if i == 4 assert(error.kind == .TIMED_OUT);
            } else {
                assert(error.kind == .NONE);
                event := ws_read(*connection);

                assert(event.kind == .MESSAGE && cast(string) event.data == "early");
                if i == 6 ws_dispose(*connection);
                else assert(ws_close(*connection).kind == .TIMED_OUT);
            }
        }
    });
}
