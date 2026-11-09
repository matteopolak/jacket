#import "Basic";
#import "Extensions/WebSocket";
#load "../../test-support/counting-allocator.jai";

main :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);

        for masked: 0..1 {
            for size: s64.[126, 65536] {
                codec: Ws_Codec;
                codec.role = .CLIENT;
                defer ws_free(*codec);
                data := NewArray(size, u8);

                defer free(data.data);
                wire: [..] u8;

                defer array_free(wire);
                error := ws_frame(*wire, .BINARY, data);

                assert(error.kind == .NONE);
                used, event := ws_parse(*codec, wire);

                assert(used == wire.count && event.kind == .MESSAGE && event.data.count == size);
            }

            codec: Ws_Codec;
            codec.role = ifx masked == 1 then .SERVER else .CLIENT;
            defer ws_free(*codec);
            wire: [..] u8;

            defer array_free(wire);
            error := ws_frame(*wire, .TEXT, cast([] u8) "hello", true, masked == 1, .[1, 2, 3, 4]);

            assert(error.kind == .NONE);

            for wire {
                one := u8.[it];
                used, event := ws_parse(*codec, one);
                assert(used == 1 && event.error.kind == .NONE);
                if it_index == wire.count - 1 assert(event.kind == .MESSAGE && cast(string) event.data == "hello");
            }
        }

        codec: Ws_Codec;

        defer ws_free(*codec);
        _, event := ws_parse(*codec, u8.[0x81, 0]);

        assert(event.error.kind == .PROTOCOL);
        ws_free(*codec);
        codec.role = .CLIENT;
        _, event = ws_parse(*codec, u8.[0x01, 1, 0xE2, 0x89, 0, 0x80, 2, 0x82, 0xAC]);
        assert(event.kind == .PING);
        _, event = ws_parse(*codec, u8.[0x80, 2, 0x82, 0xAC]);
        assert(event.kind == .MESSAGE && event.data.count == 3);
        ws_free(*codec);
        codec.role = .CLIENT;
        _, event = ws_parse(*codec, u8.[0x81, 2, 0xC0, 0x80]);
        assert(event.error.kind == .UTF8);
        ws_free(*codec);
        codec.role = .CLIENT;
        _, event = ws_parse(*codec, u8.[0x88, 2, 3, 0xED]);
        assert(event.error.kind == .PROTOCOL);
        ws_free(*codec);
        codec.role = .CLIENT;
        _, event = ws_parse(*codec, u8.[0x88, 2, 3, 0xE8]);
        assert(event.kind == .CLOSE && event.code == 1000);
        ws_free(*codec);
        codec.role = .CLIENT;
        codec.options.max_frame_size = 2;
        _, event = ws_parse(*codec, u8.[0x82, 3]);
        assert(event.error.kind == .LIMIT);
    }

    assert(tally.outstanding == 0);
    print("codec and ownership passed\n");
}
