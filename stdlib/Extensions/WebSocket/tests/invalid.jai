#import "Basic";
#import "Extensions/WebSocket";
#load "../../test-support/counting-allocator.jai";

invalid :: (wire: [] u8, kind: Ws_Error_Kind = .PROTOCOL) {
    codec: Ws_Codec;
    codec.role = .CLIENT;
    defer ws_free(*codec);
    _, event := ws_parse(*codec, wire);

    assert(event.kind == .ERROR && event.error.kind == kind);
    _, again := ws_parse(*codec, .[]);

    assert(again.kind == .ERROR && again.error.offset == event.error.offset);
}

main :: () {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);
        invalid(u8.[0xC1, 0]); // unnegotiated RSV1
        invalid(u8.[0xA1, 0]); // RSV2
        invalid(u8.[0x83, 0]); // reserved opcode
        invalid(u8.[0x89, 126]); // long control
        invalid(u8.[0x09, 0]); // fragmented control
        invalid(u8.[0x80, 0]); // stray continuation
        invalid(u8.[0x01, 0, 0x81, 0]); // missing continuation
        invalid(u8.[0x81, 126, 0, 125]); // noncanonical size
        invalid(u8.[0x82, 127, 0x80, 0, 0, 0, 0, 0, 0, 0]); // high length bit
        invalid(u8.[0x88, 1, 0]); // one-byte close
        invalid(u8.[0x88, 2, 3, 0xED]); // wire-only reserved close
        invalid(u8.[0x88, 3, 3, 0xE8, 0xFF], .UTF8);
        invalid(u8.[0x81, 3, 0xED, 0xA0, 0x80], .UTF8); // surrogate
        invalid(u8.[0x81, 4, 0xF4, 0x90, 0x80, 0x80], .UTF8); // above Unicode
        invalid(u8.[0x81, 1, 0xE2], .UTF8); // truncated scalar
        invalid(u8.[0x82, 127, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], .LIMIT);

        for first: 0..255 {
            for second: 0..255 {
                codec: Ws_Codec;
                codec.role = .CLIENT;
                codec.options = .{ max_frame_size = 256, max_message_size = 256 };
                wire := u8.[cast(u8) first, cast(u8) second, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

                ws_parse(*codec, wire);
                ws_free(*codec);
            }
        }
    }

    assert(tally.outstanding == 0);
}
