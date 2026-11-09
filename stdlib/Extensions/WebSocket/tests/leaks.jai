// test: skip_on(.WASM, reason = "checks fiber connection writes")
#import "Basic";
#import "Extensions/WebSocket";
Io :: #import "Extensions/Io";
#load "../../test-support/counting-allocator.jai";

Sink :: struct {
    codec: Ws_Codec;
    messages: int;
}

consume :: (data: *void, bytes: [] u8) -> s64, Io.Io_Error {
    sink := cast(*Sink) data;
    used, event := ws_parse(*sink.codec, bytes);

    assert(used == bytes.count && event.kind == .MESSAGE);
    assert(cast(string) event.data == "hello hello hello");
    sink.messages += 1;

    return bytes.count, .NONE;
}

main :: () {
    Io.io_run(() {
        tally: Tally;

        {
            push_allocator(tally_proc, *tally);
            sink: Sink;
            sink.codec.compression.enabled = true;
            defer ws_free(*sink.codec);
            connection: Ws_Connection;
            connection.codec.role = .CLIENT;
            connection.codec.compression.enabled = true;
            connection.link.writer = .{ data = *sink, write_proc = consume };
            for 0..9 assert(ws_write(*connection, cast([] u8) "hello hello hello").kind == .NONE);

            assert(sink.messages == 10 && connection.dictionary.count > 0);
            ws_dispose(*connection);
            ws_dispose(*connection);
        }

        assert(tally.outstanding == 0);
    });
}
