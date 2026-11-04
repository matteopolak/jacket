#import "Basic";
G :: #import "Extensions/Grpc";

main :: () {
    framed, error := G.frame_message(u8.[1, 2, 3]);

    assert(error == .NONE);
    defer free(framed.data);

    for split: 0..framed.count {
        decoder: G.Envelope_Decoder;

        defer G.free_decoder(*decoder);
        used, failure := G.feed_envelope(*decoder, .{ split, framed.data });

        assert(used == split && failure == .NONE);
        used, failure = G.feed_envelope(*decoder, .{ framed.count - split, framed.data + split });
        assert(used == framed.count - split && failure == .NONE && decoder.ready);
        message := G.take_message(*decoder);

        assert(message.count == 3 && message[0] == 1 && message[2] == 3);
        free(message.data);
        assert(G.finish_envelopes(*decoder) == .NONE);
    }

    for size: 1..framed.count - 1 {
        decoder: G.Envelope_Decoder;

        defer G.free_decoder(*decoder);
        _, failure := G.feed_envelope(*decoder, .{ size, framed.data });

        assert(failure == .NONE);
        assert(G.finish_envelopes(*decoder) == .TRUNCATED);
    }

    decoder: G.Envelope_Decoder;

    defer G.free_decoder(*decoder);
    decoder.max_message = 2;
    used, failure := G.feed_envelope(*decoder, framed);

    assert(used == 5 && failure == .TOO_LARGE && decoder.payload.count == 0);
    decoder2: G.Envelope_Decoder;

    defer G.free_decoder(*decoder2);
    _, failure = G.feed_envelope(*decoder2, u8.[1, 0, 0, 0, 0]);
    assert(failure == .UNSUPPORTED_COMPRESSION);
    decoder3: G.Envelope_Decoder;

    defer G.free_decoder(*decoder3);
    _, failure = G.feed_envelope(*decoder3, u8.[2, 0, 0, 0, 0]);
    assert(failure == .INVALID_FLAG);
    decoder4: G.Envelope_Decoder;

    defer G.free_decoder(*decoder4);
    used, failure = G.feed_envelope(*decoder4, u8.[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert(used == 5 && failure == .NONE && decoder4.ready);
    empty := G.take_message(*decoder4);

    assert(empty.count == 0);
    print("ok\n");
}
