// HTTP/2 frames (RFC 9113): reading the nine-byte frame header and each frame's payload, and
// appending frames to a buffer. The codec has no IO; `h2.jai` runs connections on top of it.
//
// ```jai,ignore
// out: [..] u8;
// defer array_reset(*out);
//
// h2_put_window_update(*out, 3, 1000);
//
// header := h2_decode_header(.{ H2_FRAME_HEADER_SIZE, out.data });
// assert(header.type == .WINDOW_UPDATE && header.stream == 3 && header.length == 4);
// ```

#scope_module

// What the client sends first on every HTTP/2 connection.
H2_PREFACE :: "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

// Bytes before each frame's payload.
H2_FRAME_HEADER_SIZE :: 9;

// The largest payload a peer may send until its SETTINGS say otherwise, and the largest one can ask for.
H2_DEFAULT_MAX_FRAME_SIZE :: 16384;
H2_LARGEST_FRAME_SIZE :: 16777215;

// The flow-control window of a new connection or stream, and the most a window can hold.
H2_DEFAULT_WINDOW :: 65535;
H2_LARGEST_WINDOW :: 0x7fffffff;

H2_END_STREAM: u8 : 0x1; // DATA, HEADERS
H2_ACK: u8 : 0x1; // SETTINGS, PING
H2_END_HEADERS: u8 : 0x4; // HEADERS, CONTINUATION
H2_PADDED: u8 : 0x8; // DATA, HEADERS
H2_PRIORITY: u8 : 0x20; // HEADERS

// The frame types in the order of their numbers; `UNKNOWN` stands for any type this module does not know (such frames are skipped).
H2_Frame_Type :: enum u8 {
    DATA;
    HEADERS;
    PRIORITY;
    RST_STREAM;
    SETTINGS;
    PUSH_PROMISE;
    PING;
    GOAWAY;
    WINDOW_UPDATE;
    CONTINUATION;
    UNKNOWN;
}

// The error codes of RFC 9113 section 7, which are also their numbers.
H2_Error :: enum u32 {
    NO_ERROR;
    PROTOCOL_ERROR;
    INTERNAL_ERROR;
    FLOW_CONTROL_ERROR;
    SETTINGS_TIMEOUT;
    STREAM_CLOSED;
    FRAME_SIZE_ERROR;
    REFUSED_STREAM;
    CANCEL;
    COMPRESSION_ERROR;
    CONNECT_ERROR;
    ENHANCE_YOUR_CALM;
    INADEQUATE_SECURITY;
    HTTP_1_1_REQUIRED;
}

// The settings parameters in the order of their numbers (`UNKNOWN` is number 0, which no peer sends; unknown ones are ignored).
H2_Setting :: enum u16 {
    UNKNOWN;
    HEADER_TABLE_SIZE;
    ENABLE_PUSH;
    MAX_CONCURRENT_STREAMS;
    INITIAL_WINDOW_SIZE;
    MAX_FRAME_SIZE;
    MAX_HEADER_LIST_SIZE;
}

// What a peer has told us about itself, with the protocol's defaults. A limit of -1 is "no limit".
H2_Settings :: struct {
    header_table_size: s64 = 4096;
    enable_push: bool = true;
    max_concurrent_streams: s64 = -1;
    initial_window_size: s64 = H2_DEFAULT_WINDOW;
    max_frame_size: s64 = H2_DEFAULT_MAX_FRAME_SIZE;
    max_header_list_size: s64 = -1;
}

// One parameter of a SETTINGS frame.
H2_Setting_Entry :: struct {
    id: u16;
    value: u32;
}

// The fixed part of a frame. `type_byte` is the number on the wire, which `type` maps to `UNKNOWN` when it is not one of ours.
H2_Frame_Header :: struct {
    length: int;
    type: H2_Frame_Type;
    type_byte: u8;
    flags: u8;
    stream: u32;
}

// Reads the first nine bytes of `bytes` (at least that many must be there).
h2_decode_header :: (bytes: [] u8) -> H2_Frame_Header {
    header: H2_Frame_Header;

    header.length = ((cast(int) bytes[0]) << 16) | ((cast(int) bytes[1]) << 8) | (cast(int) bytes[2]);
    header.type_byte = bytes[3];
    header.type = ifx bytes[3] < cast(u8) H2_Frame_Type.UNKNOWN then cast(H2_Frame_Type) bytes[3] else .UNKNOWN;
    header.flags = bytes[4];
    header.stream = h2_read_u32(bytes, 5) & 0x7fffffff;

    return header;
}

// The big-endian number at `offset`.
h2_read_u32 :: (bytes: [] u8, offset: int) -> u32 {
    return ((cast(u32) bytes[offset]) << 24) | ((cast(u32) bytes[offset + 1]) << 16) | ((cast(u32) bytes[offset + 2]) << 8) | (cast(u32) bytes[offset + 3]);
}

h2_put_u32 :: (out: *[..] u8, value: u32) {
    bytes: [4] u8;

    bytes[0] = cast(u8) ((value >> 24) & 0xff);
    bytes[1] = cast(u8) ((value >> 16) & 0xff);
    bytes[2] = cast(u8) ((value >> 8) & 0xff);
    bytes[3] = cast(u8) (value & 0xff);

    append_bytes(out, bytes.data, 4);
}

// Appends a frame header.
h2_put_frame_header :: (out: *[..] u8, length: int, type: H2_Frame_Type, flags: u8, stream: u32) {
    bytes: [H2_FRAME_HEADER_SIZE] u8;

    bytes[0] = cast(u8) ((length >> 16) & 0xff);
    bytes[1] = cast(u8) ((length >> 8) & 0xff);
    bytes[2] = cast(u8) (length & 0xff);
    bytes[3] = cast(u8) type;
    bytes[4] = flags;

    append_bytes(out, bytes.data, 5);
    h2_put_u32(out, stream & 0x7fffffff);
}

// Appends a DATA frame with `data` (at most the peer's frame size) and no padding.
h2_put_data :: (out: *[..] u8, stream: u32, data: [] u8, end_stream: bool) {
    h2_put_frame_header(out, data.count, .DATA, ifx end_stream then H2_END_STREAM else 0, stream);
    append_bytes(out, data.data, data.count);
}

// Appends a header block as a HEADERS frame, followed by CONTINUATION frames when the block is longer than `max_frame_size`.
h2_put_headers :: (out: *[..] u8, stream: u32, block: [] u8, end_stream: bool, max_frame_size: int) {
    first := min(block.count, max_frame_size);
    flags: u8 = ifx end_stream then H2_END_STREAM else 0;

    if first == block.count flags |= H2_END_HEADERS;

    h2_put_frame_header(out, first, .HEADERS, flags, stream);
    append_bytes(out, block.data, first);

    at := first;

    while at < block.count {
        count := min(block.count - at, max_frame_size);

        h2_put_frame_header(out, count, .CONTINUATION, ifx at + count == block.count then H2_END_HEADERS else 0, stream);
        append_bytes(out, block.data + at, count);

        at += count;
    }
}

// Appends RST_STREAM.
h2_put_rst_stream :: (out: *[..] u8, stream: u32, code: H2_Error) {
    h2_put_frame_header(out, 4, .RST_STREAM, 0, stream);
    h2_put_u32(out, cast(u32) code);
}

// Appends SETTINGS with the given parameters.
h2_put_settings :: (out: *[..] u8, entries: [] H2_Setting_Entry) {
    h2_put_frame_header(out, entries.count * 6, .SETTINGS, 0, 0);

    for entries {
        bytes: [2] u8;

        bytes[0] = cast(u8) ((it.id >> 8) & 0xff);
        bytes[1] = cast(u8) (it.id & 0xff);

        append_bytes(out, bytes.data, 2);
        h2_put_u32(out, it.value);
    }
}

// Appends the empty SETTINGS frame that acknowledges the peer's.
h2_put_settings_ack :: (out: *[..] u8) {
    h2_put_frame_header(out, 0, .SETTINGS, H2_ACK, 0);
}

// Appends PING with eight bytes of `data`; a reply carries the same bytes with `ack`.
h2_put_ping :: (out: *[..] u8, data: [] u8, ack: bool) {
    h2_put_frame_header(out, 8, .PING, ifx ack then H2_ACK else 0, 0);
    append_bytes(out, data.data, 8);
}

// Appends GOAWAY: no stream above `last_stream` was or will be processed.
h2_put_goaway :: (out: *[..] u8, last_stream: u32, code: H2_Error, debug := "") {
    h2_put_frame_header(out, 8 + debug.count, .GOAWAY, 0, 0);
    h2_put_u32(out, last_stream);
    h2_put_u32(out, cast(u32) code);
    append_bytes(out, debug.data, debug.count);
}

// Appends WINDOW_UPDATE: the peer may send `increment` more bytes on `stream` (0 is the connection).
h2_put_window_update :: (out: *[..] u8, stream: u32, increment: u32) {
    h2_put_frame_header(out, 4, .WINDOW_UPDATE, 0, stream);
    h2_put_u32(out, increment);
}

// The data of a DATA frame without its padding. A pad length that leaves no room is a protocol error.
h2_data_payload :: (flags: u8, payload: [] u8) -> [] u8, H2_Error {
    if (flags & H2_PADDED) == 0 return payload, .NO_ERROR;

    if payload.count < 1 return .[], .FRAME_SIZE_ERROR;

    pad := cast(int) payload[0];

    if pad > payload.count - 1 return .[], .PROTOCOL_ERROR;

    return .{ payload.count - 1 - pad, payload.data + 1 }, .NO_ERROR;
}

// The header block of a HEADERS frame, without padding and the priority fields (parsed and ignored).
h2_headers_payload :: (flags: u8, payload: [] u8) -> [] u8, H2_Error {
    block := payload;
    pad := 0;

    if (flags & H2_PADDED) != 0 {
        if block.count < 1 return .[], .FRAME_SIZE_ERROR;

        pad = cast(int) block[0];
        block = .{ block.count - 1, block.data + 1 };
    }

    if (flags & H2_PRIORITY) != 0 {
        priority_size :: 5;

        if block.count < priority_size return .[], .FRAME_SIZE_ERROR;

        block = .{ block.count - priority_size, block.data + priority_size };
    }

    if pad > block.count return .[], .PROTOCOL_ERROR;

    return .{ block.count - pad, block.data }, .NO_ERROR;
}

// Applies one SETTINGS parameter, refusing the values RFC 9113 section 6.5.2 forbids. Unknown parameters are ignored.
h2_apply_setting :: (settings: *H2_Settings, id: u16, value: u32) -> H2_Error {
    if id == {
        case cast(u16) H2_Setting.HEADER_TABLE_SIZE;
            settings.header_table_size = value;

        case cast(u16) H2_Setting.ENABLE_PUSH;
            if value > 1 return .PROTOCOL_ERROR;

            settings.enable_push = value == 1;

        case cast(u16) H2_Setting.MAX_CONCURRENT_STREAMS;
            settings.max_concurrent_streams = value;

        case cast(u16) H2_Setting.INITIAL_WINDOW_SIZE;
            if value > H2_LARGEST_WINDOW return .FLOW_CONTROL_ERROR;

            settings.initial_window_size = value;

        case cast(u16) H2_Setting.MAX_FRAME_SIZE;
            if value < H2_DEFAULT_MAX_FRAME_SIZE || value > H2_LARGEST_FRAME_SIZE return .PROTOCOL_ERROR;

            settings.max_frame_size = value;

        case cast(u16) H2_Setting.MAX_HEADER_LIST_SIZE;
            settings.max_header_list_size = value;
    }

    return .NO_ERROR;
}

// Applies every parameter of a SETTINGS payload, in order. A length that is not a multiple of six is a frame size error.
h2_read_settings :: (payload: [] u8, settings: *H2_Settings) -> H2_Error {
    entry_size :: 6;

    if payload.count % entry_size != 0 return .FRAME_SIZE_ERROR;

    at := 0;

    while at < payload.count {
        id := ((cast(u16) payload[at]) << 8) | (cast(u16) payload[at + 1]);
        error := h2_apply_setting(settings, id, h2_read_u32(payload, at + 2));

        if error != .NO_ERROR return error;

        at += entry_size;
    }

    return .NO_ERROR;
}

// The fields of a GOAWAY payload: the last stream the sender handled, its error code and the debug text.
h2_read_goaway :: (payload: [] u8) -> last_stream: u32, code: u32, debug: string, error: H2_Error {
    if payload.count < 8 return 0, 0, "", .FRAME_SIZE_ERROR;

    return h2_read_u32(payload, 0) & 0x7fffffff, h2_read_u32(payload, 4), .{ payload.count - 8, payload.data + 8 }, .NO_ERROR;
}
