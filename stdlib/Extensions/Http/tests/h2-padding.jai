// test: skip_on(.WASM, reason = "native HTTP/2 scheduler")
#load "../module.jai";

main :: () {
    entry :: (_: *void) {
        conn: H2_Conn;
        stream: H2_Stream;

        conn.role = .CLIENT;
        conn.next_stream = 3;
        conn.local.initial_window_size = 1024;
        stream.id = 1;
        stream.conn = *conn;
        stream.has_head = true;
        stream.refs = 2;
        stream.recv_window = 1024;
        array_add(*conn.streams, *stream);
        defer array_reset(*conn.streams);
        defer array_reset(*conn.out);
        defer array_reset(*stream.recv);
        bytes: [261] u8;
        bytes[0] = 255;

        for 0..99 {
            error := h2_on_data(*conn, .{ length = 261, type = .DATA, flags = H2_PADDED, stream = 1 }, bytes);

            assert(error == .NO_ERROR && !stream.reset_sent, "discarded padding must return stream flow-control credit");
            assert(stream.recv.count == 5);
            stream.recv.count = 0;
            h2_consumed(*stream, 5);
        }

        assert(stream.received == 500);
        assert(stream.recv_window > 0 && conn.recv_window > 0);
    }

    Io.io_run(entry, null);
}
