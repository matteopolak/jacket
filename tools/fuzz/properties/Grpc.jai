// Fragmentation preserves arbitrary message bytes without a decoded message queue.
fuzz_grpc_envelopes :: (r: *Rng) {
    input := gen(r, string);
    framed, error := Target.frame_message(.{ input.count, input.data });

    assert(error == .NONE);
    defer free(framed.data);
    decoder: Target.Envelope_Decoder;
    decoder.max_message = input.count;
    defer Target.free_decoder(*decoder);
    at := 0;
    while at < framed.count {
        count := min(framed.count - at, 1 + cast(int) (gen(r, u8) % 17));
        used, failure := Target.feed_envelope(*decoder, .{ count, framed.data + at });

        assert(failure == .NONE && used == count);
        at += used;
        assert(decoder.payload.allocated <= input.count);
    }

    assert(decoder.ready);
    bytes := Target.take_message(*decoder);

    defer if bytes.data free(bytes.data);
    assert(string.{ bytes.count, bytes.data } == input);
    assert(Target.finish_envelopes(*decoder) == .NONE);
}

// Declared-size errors precede allocation; arbitrary incomplete frames stay bounded.
fuzz_grpc_damaged :: (r: *Rng) {
    input := gen(r, string);
    decoder: Target.Envelope_Decoder;
    decoder.max_message = 65536;
    defer Target.free_decoder(*decoder);
    used, failure := Target.feed_envelope(*decoder, .{ input.count, input.data });

    assert(used >= 0 && used <= input.count && decoder.payload.allocated <= 65536);
    if failure == .TOO_LARGE assert(decoder.payload.count == 0);

    if decoder.ready {
        bytes := Target.take_message(*decoder);
        if bytes.data free(bytes.data);
    }
}

// Binary metadata round-trips arbitrary octets and retains repeated names.
fuzz_grpc_metadata :: (r: *Rng) {
    input := gen(r, string);
    fields, valid := Target.encode_metadata(Target.Metadata_Entry.[.{ "data-bin", input }, .{ "data-bin", input }]);

    assert(valid);
    metadata, decoded := Target.decode_metadata(fields);

    defer Target.free_metadata(*metadata);
    assert(decoded && metadata.count == 2);
    assert(metadata[0].value == input && metadata[1].value == input);
}
