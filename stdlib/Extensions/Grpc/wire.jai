// Per-message compression uses the algorithm negotiated by grpc-encoding.
Compression :: enum u8 {
    IDENTITY;
    GZIP;
}

// Envelope validation failures. Declared-size rejection precedes payload allocation.
Wire_Error :: enum u8 {
    NONE;
    TOO_LARGE;
    INVALID_FLAG;
    UNSUPPORTED_COMPRESSION;
    INVALID_COMPRESSION;
    TRUNCATED;
}

// One in-progress message, with no background message queue. A ready payload is
// transferred with `take_message`; release an abandoned decoder with `free_decoder`.
Envelope_Decoder :: struct {
    max_message := 4 * 1024 * 1024;
    max_wire_message := 4 * 1024 * 1024;
    compression: Compression;
    compressed: bool;
    ready: bool;
    failure: Wire_Error;
    prefix: [5] u8;
    prefix_used: int;
    length: s64;
    payload: [..] u8;
}

// Encodes an envelope into caller-owned memory, freed with `free(bytes.data)`.
//
// ```jai
// bytes, failure := frame_message(u8.[1, 2]);
// assert(failure == .NONE && bytes.count == 7);
// free(bytes.data);
// ```
frame_message :: (message: [] u8, max_message := 4 * 1024 * 1024, compression := Compression.IDENTITY, max_wire_message := 4 * 1024 * 1024) -> [] u8, Wire_Error {
    if message.count < 0 || message.count > max_message || message.count > 0xffffffff return .[], .TOO_LARGE;

    packed: [] u8;

    defer if packed.data free(packed.data);
    payload := message;

    if compression == .GZIP {
        packed = Deflate.deflate(message, .GZIP);
        payload = packed;
    }

    if payload.count > max_wire_message || payload.count > 0xffffffff return .[], .TOO_LARGE;

    out := NewArray(payload.count + 5, u8);
    out[0] = ifx compression == .GZIP then 1 else 0;
    for i: 0..3 out[i + 1] = cast(u8) ((cast(u64) payload.count >> ((3 - i) * 8)) & 255);
    if payload.count > 0 memcpy(out.data + 5, payload.data, payload.count);

    return out, .NONE;
}

// Consumes at most one message, returning the consumed byte count. Callers keep
// unconsumed input for the next message. The size limit is checked after the prefix,
// before reserving payload memory; errors remain sticky until the decoder is freed.
//
// ```jai
// decoder: Envelope_Decoder;
// defer free_decoder(*decoder);
// used, failure := feed_envelope(*decoder, u8.[0, 0, 0, 0, 0]);
// assert(used == 5 && failure == .NONE && decoder.ready);
// ```
feed_envelope :: (decoder: *Envelope_Decoder, bytes: [] u8) -> used: int, failure: Wire_Error {
    if decoder.failure != .NONE return 0, decoder.failure;
    if decoder.ready return 0, .NONE;

    at := 0;
    while decoder.prefix_used < 5 && at < bytes.count {
        decoder.prefix[decoder.prefix_used] = bytes[at];
        decoder.prefix_used += 1;
        at += 1;
    }

    if decoder.prefix_used < 5 return at, .NONE;

    flag := decoder.prefix[0];
    if flag > 1 || (flag == 1 && decoder.compression == .IDENTITY) {
        decoder.failure = ifx flag == 1 then Wire_Error.UNSUPPORTED_COMPRESSION else Wire_Error.INVALID_FLAG;
        return at, decoder.failure;
    }

    length: s64;
    for i: 1..4 length = (length << 8) | cast(s64) decoder.prefix[i];

    if length > decoder.max_wire_message || decoder.max_wire_message < 0 || decoder.max_message < 0 || (flag == 0 && length > decoder.max_message) {
        decoder.failure = .TOO_LARGE;
        return at, decoder.failure;
    }

    decoder.length = length;
    count := min(length - decoder.payload.count, bytes.count - at);
    if count > 0 {
        array_reserve(*decoder.payload, decoder.payload.count + count);
        memcpy(decoder.payload.data + decoder.payload.count, bytes.data + at, count);
        decoder.payload.count += count;
        at += count;
    }

    decoder.ready = decoder.payload.count == length;
    decoder.compressed = flag == 1;
    if decoder.ready && decoder.compressed {
        decoded, ok, error := Deflate.inflate_complete(decoder.payload, .GZIP, decoder.max_message);
        if !ok {
            decoder.ready = false;
            decoder.failure = ifx error == "output exceeds the limit" then Wire_Error.TOO_LARGE else Wire_Error.INVALID_COMPRESSION;
            return at, decoder.failure;
        }

        defer free(decoded.data);
        decoder.payload.count = 0;
        array_reserve(*decoder.payload, decoded.count);
        if decoded.count memcpy(decoder.payload.data, decoded.data, decoded.count);

        decoder.payload.count = decoded.count;
    }

    return at, .NONE;
}

// Transfers a complete payload to the caller and starts the next envelope.
// Returns empty if no message is ready; inspect `ready` before calling to
// distinguish that case from a legitimate zero-byte message.
//
// ```jai,ignore
// if decoder.ready { bytes := take_message(*decoder); defer free(bytes.data); }
// ```
take_message :: (decoder: *Envelope_Decoder) -> [] u8 {
    if !decoder.ready return .[];

    bytes: [] u8 = .{ decoder.payload.count, decoder.payload.data };

    decoder.payload = .{};
    decoder.prefix_used = 0;
    decoder.length = 0;
    decoder.ready = false;

    return bytes;
}

// Validates byte EOF. A partial prefix or payload is an error, even if a prior
// message was valid. This only validates framing; RPC completion requires status.
//
// ```jai
// decoder: Envelope_Decoder;
// assert(finish_envelopes(*decoder) == .NONE);
// ```
finish_envelopes :: (decoder: *Envelope_Decoder) -> Wire_Error {
    if decoder.failure != .NONE return decoder.failure;
    if decoder.prefix_used != 0 && !decoder.ready return .TRUNCATED;
    return .NONE;
}

// Releases partial or unread payload storage. Safe to call repeatedly.
//
// ```jai
// decoder: Envelope_Decoder;
// free_decoder(*decoder);
// ```
free_decoder :: (decoder: *Envelope_Decoder) {
    array_reset(*decoder.payload);
    decoder.* = .{};
}
