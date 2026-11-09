#import "Basic";
#import "Extensions/WebSocket";
D :: #import "Extensions/Deflate";
#load "../../test-support/counting-allocator.jai";

main :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);
        packed := D.deflate_sync(cast([] u8) "hello hello hello");

        defer free(packed.data);

        for split: 0..packed.count {
            codec: Ws_Codec;
            codec.role = .CLIENT;
            codec.compression.enabled = true;
            defer ws_free(*codec);
            wire: [..] u8;

            defer array_free(wire);
            assert(ws_frame(*wire, .TEXT, .{ split, packed.data }, false, false, .{}, true).kind == .NONE);
            assert(ws_frame(*wire, .PING, cast([] u8) "control").kind == .NONE);
            assert(ws_frame(*wire, .CONTINUATION, .{ packed.count - split, packed.data + split }).kind == .NONE);
            used, event := ws_parse(*codec, wire);

            assert(event.kind == .PING);
            _, event = ws_parse(*codec, .{ wire.count - used, wire.data + used });
            assert(event.kind == .MESSAGE && cast(string) event.data == "hello hello hello");
            wire.count = 0;
            // The second zlib message refers to the previous message's history.
            assert(ws_frame(*wire, .TEXT, u8.[0xCA, 0x40, 0x17, 0, 0], true, false, .{}, true).kind == .NONE);
            _, event = ws_parse(*codec, wire);
            assert(event.kind == .MESSAGE && cast(string) event.data == "hello hello hello");
        }

        for no_context: 0..1 {
            codec: Ws_Codec;
            codec.role = .CLIENT;
            codec.compression.enabled = true;
            codec.compression.server_no_context_takeover = no_context == 1;
            defer ws_free(*codec);
            wire: [..] u8;

            defer array_free(wire);
            ws_frame(*wire, .TEXT, packed, true, false, .{}, true);
            _, event := ws_parse(*codec, wire);

            assert(event.kind == .MESSAGE);
            wire.count = 0;
            ws_frame(*wire, .TEXT, u8.[0xCA, 0x40, 0x17, 0, 0], true, false, .{}, true);
            _, event = ws_parse(*codec, wire);
            assert((event.kind == .MESSAGE) == (no_context == 0));
        }

        codec: Ws_Codec;
        codec.role = .CLIENT;
        codec.compression.enabled = true;
        codec.options.max_message_size = 16;
        defer ws_free(*codec);
        source := NewArray(4096, u8);

        defer free(source.data);
        bomb := D.deflate_sync(source);

        defer free(bomb.data);
        wire: [..] u8;

        defer array_free(wire);
        ws_frame(*wire, .BINARY, bomb, true, false, .{}, true);
        _, event := ws_parse(*codec, wire);

        assert(event.kind == .ERROR);

        // Failed inflation and every truncated/corrupted stream release allocations.
        for i: 0..packed.count {
            plain, ok := D.inflate_sync(.{ i, packed.data }, .[], 64);
            if ok free(plain.data);
        }
    }

    assert(tally.outstanding == 0);
}
