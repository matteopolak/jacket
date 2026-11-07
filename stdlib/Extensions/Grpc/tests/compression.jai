#import "Basic";
G :: #import "Extensions/Grpc";

main :: () {
    input := u8.[1, 2, 3, 1, 2, 3];
    framed, error := G.frame_message(input, compression = .GZIP);

    assert(error == .NONE && framed[0] == 1);
    defer free(framed.data);

    for split: 0..framed.count {
        decoder: G.Envelope_Decoder;
        decoder.compression = .GZIP;
        defer G.free_decoder(*decoder);
        used, failure := G.feed_envelope(*decoder, .{ split, framed.data });

        assert(used == split && failure == .NONE);
        _, failure = G.feed_envelope(*decoder, .{ framed.count - split, framed.data + split });
        assert(failure == .NONE && decoder.ready && decoder.compressed);
        output := G.take_message(*decoder);

        defer if output.data free(output.data);
        assert(output.count == input.count);
        for input assert(output[it_index] == it);

        // A following identity message is valid on the same negotiated gzip stream.
        _, failure = G.feed_envelope(*decoder, u8.[0, 0, 0, 0, 0]);
        assert(failure == .NONE && decoder.ready && !decoder.compressed);
        empty := G.take_message(*decoder);
        if empty.data free(empty.data);
    }

    decoder: G.Envelope_Decoder;
    decoder.compression = .GZIP;
    decoder.max_message = 2;
    defer G.free_decoder(*decoder);
    _, failure := G.feed_envelope(*decoder, framed);

    assert(failure == .TOO_LARGE && !decoder.ready);
    invalid: G.Envelope_Decoder;
    invalid.compression = .GZIP;
    defer G.free_decoder(*invalid);
    _, failure = G.feed_envelope(*invalid, u8.[1, 0, 0, 0, 1, 7]);
    assert(failure == .INVALID_COMPRESSION && !invalid.ready);
    bounded: G.Envelope_Decoder;
    bounded.compression = .GZIP;
    bounded.max_wire_message = 2;
    defer G.free_decoder(*bounded);
    used: int;
    used, failure = G.feed_envelope(*bounded, framed);
    assert(used == 5 && failure == .TOO_LARGE && bounded.payload.count == 0);
    print("ok\n");
}
