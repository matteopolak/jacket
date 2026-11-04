// test: skip_on(.WASM, reason = "HTTP/2 fibers require native Io")
// Test the exact suspension point without depending on socket capacity or timing.
#load "../module.jai";

Trailer_Test_State :: struct {
    conn: H2_Conn;
    stream: H2_Stream;
    frame: Stream_Frame;
    done: Io.Wait_Group;
    returned: bool;
    result: bool;
}

finish_blocked :: (state: *Trailer_Test_State) {
    state.result = finish_stream(*state.frame.stream, Header_Field.[.{ name = "grpc-status", value = "0" }]);
    state.returned = true;
    Io.done(*state.done);
}

main :: () {
    entry :: (_: *void) {
        // Releasing before the body source observes EOF is not completion,
        // even when the reader has already seen remote END_STREAM.
        conn: H2_Conn;
        stream: H2_Stream;
        source: Body_Source;
        conn.refs = 2;
        stream.refs = 2;
        stream.conn = *conn;
        stream.local_ended = true;
        stream.remote_ended = true;
        source.h2 = *stream;
        source.held = true;
        h2_source_release(*source);
        assert(!source.h2_complete && !source.h2 && !source.held);

        state: Trailer_Test_State;
        state.stream.conn = *state.conn;
        state.frame.h2 = *state.stream;
        array_reserve(*state.conn.out, H2_QUEUE_LIMIT + 1);
        state.conn.out.count = H2_QUEUE_LIMIT + 1;
        defer array_reset(*state.conn.out);
        Io.add(*state.done, 1);
        Io.spawn(finish_blocked, *state);
        // The child runs until the full output queue parks it.
        Io.yield();
        assert(!state.returned);
        state.stream.reset_by_peer = true;
        state.conn.out.count = 0;
        Io.notify_all(*state.conn.drained);
        Io.wait(*state.done);
        assert(state.returned && !state.result && state.conn.out.count == 0);
    }
    Io.io_run(entry, null);
    print("ok\n");
}
