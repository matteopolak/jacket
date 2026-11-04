// test: skip_on(.WASM, reason = "native RPC/HTTP scheduler")
// Test the exact suspension point without depending on socket capacity or timing.
#load "../module.jai";

Trailer_Test_State :: struct {
    conn: H2_Conn;
    stream: H2_Stream;
    frame: Stream_Frame;
    done: Io.Wait_Group;
    returned: bool;
    result: bool;
    data_send: bool;
}

finish_blocked :: (state: *Trailer_Test_State) {
    if state.data_send state.result = h2_send_data(*state.stream, u8.[1], false);
    else state.result = finish_stream(*state.frame.stream, Header_Field.[.{ name = "grpc-status", value = "0" }]);
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

        for data_send: bool.[false, true] {
            state: Trailer_Test_State;
            state.data_send = data_send;
            state.stream.id = 1;
            state.stream.refs = 2;
            state.stream.conn = *state.conn;
            state.stream.send_window = 65535;
            state.conn.open_local = 1;
            array_add(*state.conn.streams, *state.stream);
            defer array_reset(*state.conn.streams);
            state.frame.h2 = *state.stream;
            array_reserve(*state.conn.spare, H2_QUEUE_LIMIT + 1);
            state.conn.spare.count = H2_QUEUE_LIMIT + 1;
            defer array_reset(*state.conn.spare);
            defer array_reset(*state.conn.out);
            Io.add(*state.done, 1);
            Io.spawn(finish_blocked, *state);
            Io.yield();
            assert(!state.returned);
            h2_reset_stream(*state.conn, *state.stream, .CANCEL);
            queued_after_reset := state.conn.out.count;

            // Only cancellation changes; the writer stays held and queue stays full.
            Io.with_deadline(1000, #code {
                Io.wait(*state.done);
            });
            assert(state.returned && !state.result, "queue-stalled operation wakes on stream cancellation");
            assert(state.conn.out.count == queued_after_reset && !state.conn.dead, "no DATA/HEADERS after reset and connection survives");
        }
    }

    Io.io_run(entry, null);
    print("ok\n");
}
