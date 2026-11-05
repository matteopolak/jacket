// test: skip_on(.WASM, reason = "native HTTP/2 scheduler")
// RFC 9113 minimal closed-stream processing and deprecated PRIORITY handling.
#load "../module.jai";

main :: () {
    entry :: (_: *void) {
        conn: H2_Conn;
        conn.role = .CLIENT;
        conn.next_stream = 5;
        conn.recv_window = 64;
        data: [5] u8;
        error := h2_on_data(*conn, .{ length = 5, type = .DATA, stream = 1 }, data);

        assert(error == .NO_ERROR && conn.recv_window == 59 && conn.recv_unacked == 5, "discarded closed-stream DATA still consumes connection credit");
        assert(conn.out.count == 0, "closed DATA can be minimally processed without a new stream reset");
        too_much: [60] u8;

        error = h2_on_data(*conn, .{ length = 60, type = .DATA, stream = 1 }, too_much);
        assert(error == .FLOW_CONTROL_ERROR, "closed streams cannot bypass connection flow control");
        conn.max_header_list = 512;
        hpack_decoder_init(*conn.decoder);
        defer hpack_decoder_free(*conn.decoder);
        encoder: Hpack_Encoder;

        hpack_encoder_init(*encoder);
        defer hpack_encoder_free(*encoder);
        block: [..] u8;

        defer array_reset(*block);
        hpack_encode_start(*encoder, *block);
        hpack_encode_field(*encoder, *block, ":status", "200");
        hpack_encode_field(*encoder, *block, "x-closed", "decoded-before-discard");
        assert(h2_headers_complete(*conn, 1, block, true) == .NO_ERROR);
        sibling: H2_Stream;
        sibling.id = 3;
        sibling.conn = *conn;
        sibling.refs = 2;
        array_add(*conn.streams, *sibling);
        defer array_reset(*conn.streams);
        defer hpack_block_free(*sibling.head);
        block.count = 0;
        hpack_encode_start(*encoder, *block);
        hpack_encode_field(*encoder, *block, ":status", "200");
        hpack_encode_field(*encoder, *block, "x-closed", "decoded-before-discard");
        assert(block.count < 8, "healthy sibling references the closed block's dynamic insertion");
        assert(h2_headers_complete(*conn, 3, block, false) == .NO_ERROR);
        assert(sibling.has_head && sibling.head.fields.count == 2 && sibling.head.fields[1].value == "decoded-before-discard");
        priority: [5] u8;
        priority[3] = 7;
        assert(h2_on_priority(*conn, .{ type = .PRIORITY, stream = 7 }, priority) == .NO_ERROR, "deprecated self-priority is ignored");
        assert(h2_on_priority(*conn, .{ type = .PRIORITY, stream = 0 }, priority) == .PROTOCOL_ERROR, "PRIORITY still forbids stream zero");
        error = h2_on_priority(*conn, .{ type = .PRIORITY, stream = 7 }, .{ 4, priority.data });
        assert(error == .NO_ERROR && conn.out.count == H2_FRAME_HEADER_SIZE + 4);
        header := h2_decode_header(.{ H2_FRAME_HEADER_SIZE, conn.out.data });

        assert(header.type == .RST_STREAM && header.stream == 7 && h2_read_u32(.{ 4, conn.out.data + H2_FRAME_HEADER_SIZE }, 0) == cast(u32) H2_Error.FRAME_SIZE_ERROR, "malformed PRIORITY length remains a stream error");
        defer array_reset(*conn.out);
    }

    Io.io_run(entry, null);
}
