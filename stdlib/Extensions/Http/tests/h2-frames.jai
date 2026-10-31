// The HTTP/2 frame codec: headers, every frame the library writes, padding and priority fields,
// SETTINGS values and the errors RFC 9113 gives for malformed frames.
//
//     jaic stdlib/Extensions/Http/tests/h2-frames.jai -interp

#load "support/support.jai";

header_of :: (out: [..] u8) -> H2_Frame_Header {
    return h2_decode_header(.{ out.count, out.data });
}

frame_header :: () {
    out: [..] u8;

    defer array_reset(*out);

    h2_put_frame_header(*out, 0x123456, .HEADERS, H2_END_STREAM | H2_END_HEADERS, 0x7fffffff);

    check(out.count == H2_FRAME_HEADER_SIZE, "a frame header takes nine bytes");

    header := header_of(out);

    check(header.length == 0x123456 && header.type == .HEADERS && header.flags == 5 && header.stream == 0x7fffffff, "a frame header reads back");

    // The reserved bit of the stream id is dropped.
    out[5] |= 0x80;

    check(header_of(out).stream == 0x7fffffff, "the reserved bit is ignored");

    out[3] = 77;
    header = header_of(out);

    check(header.type == .UNKNOWN && header.type_byte == 77, "an unknown type is kept as its number");

    for type: H2_Frame_Type.DATA..H2_Frame_Type.CONTINUATION {
        out[3] = cast(u8) type;

        check(header_of(out).type == type, tprint("type % maps to its name", cast(int) type));
    }
}

data_and_padding :: () {
    out: [..] u8;

    defer array_reset(*out);

    h2_put_data(*out, 1, bytes_of("hello"), true);

    header := header_of(out);

    check(header.type == .DATA && header.length == 5 && header.stream == 1 && (header.flags & H2_END_STREAM) != 0, "DATA with END_STREAM");

    payload: [] u8 = .{ 5, out.data + H2_FRAME_HEADER_SIZE };
    data, error := h2_data_payload(header.flags, payload);

    check(error == .NO_ERROR && string.{ data.count, data.data } == "hello", "unpadded data");

    // Padded: one byte of pad length, the data, then the padding.
    padded: [8] u8;

    padded[0] = 2;
    padded[1] = #char "a";
    padded[2] = #char "b";
    padded[3] = #char "c";

    data, error = h2_data_payload(H2_PADDED, .{ 6, padded.data });

    check(error == .NO_ERROR && string.{ data.count, data.data } == "abc", "padding is taken off");

    data, error = h2_data_payload(H2_PADDED, .{ 3, padded.data });

    check(error == .NO_ERROR && data.count == 0, "padding that takes the rest leaves no data");

    padded[0] = 3;
    _, error = h2_data_payload(H2_PADDED, .{ 3, padded.data });

    check(error == .PROTOCOL_ERROR, "padding longer than the frame");

    _, error = h2_data_payload(H2_PADDED, .[]);

    check(error == .FRAME_SIZE_ERROR, "a padded frame with no pad length");
}

headers_and_continuation :: () {
    block := NewArray(100, u8,, temp);

    for *block it.* = cast(u8) (#char "a" + it_index % 26);

    out: [..] u8;

    defer array_reset(*out);

    h2_put_headers(*out, 5, block, true, 100);

    header := header_of(out);

    check(header.type == .HEADERS && header.length == 100 && header.flags == (H2_END_STREAM | H2_END_HEADERS), "a block that fits is one frame");

    out.count = 0;
    h2_put_headers(*out, 5, block, false, 40);

    first := header_of(out);
    second := h2_decode_header(.{ out.count - 49, out.data + 49 });
    third := h2_decode_header(.{ out.count - 98, out.data + 98 });

    check(first.type == .HEADERS && first.length == 40 && first.flags == 0, "the first piece is HEADERS without END_HEADERS");
    check(second.type == .CONTINUATION && second.length == 40 && second.flags == 0 && second.stream == 5, "the middle piece is CONTINUATION");
    check(third.type == .CONTINUATION && third.length == 20 && third.flags == H2_END_HEADERS, "the last piece ends the block");
    check(out.count == 9 * 3 + 100, "the pieces add up to the block");

    // Padding and priority fields are not part of the block.
    payload: [] u8 = .[2, 0, 0, 0, 0, 5, 10, 20, 30, 0, 0];
    inner, error := h2_headers_payload(H2_PADDED | H2_PRIORITY, payload);

    check(error == .NO_ERROR && inner.count == 3 && inner[0] == 10 && inner[2] == 30, "padding and priority are skipped");

    _, error = h2_headers_payload(H2_PRIORITY, .[1, 2, 3]);

    check(error == .FRAME_SIZE_ERROR, "a priority field that is cut short");

    _, error = h2_headers_payload(H2_PADDED, .[9, 1]);

    check(error == .PROTOCOL_ERROR, "padding longer than the block");
}

control_frames :: () {
    out: [..] u8;

    defer array_reset(*out);

    h2_put_rst_stream(*out, 7, .CANCEL);

    header := header_of(out);

    check(header.type == .RST_STREAM && header.length == 4 && header.stream == 7 && h2_read_u32(out, 9) == 8, "RST_STREAM carries its code");

    out.count = 0;
    h2_put_window_update(*out, 0, 1000);

    header = header_of(out);

    check(header.type == .WINDOW_UPDATE && header.stream == 0 && h2_read_u32(out, 9) == 1000, "WINDOW_UPDATE");

    out.count = 0;
    h2_put_ping(*out, .[1, 2, 3, 4, 5, 6, 7, 8], true);

    header = header_of(out);

    check(header.type == .PING && header.length == 8 && (header.flags & H2_ACK) != 0 && out[16] == 8, "PING ack");

    out.count = 0;
    h2_put_goaway(*out, 9, .ENHANCE_YOUR_CALM, "slow down");

    header = header_of(out);

    last, code, debug, error := h2_read_goaway(.{ header.length, out.data + 9 });

    check(header.type == .GOAWAY && error == .NO_ERROR && last == 9 && code == 11 && debug == "slow down", "GOAWAY carries the last stream, the code and the debug text");

    _, _, _, error = h2_read_goaway(.[1, 2, 3]);

    check(error == .FRAME_SIZE_ERROR, "a GOAWAY that is too short");

    out.count = 0;
    h2_put_settings_ack(*out);

    header = header_of(out);

    check(header.type == .SETTINGS && header.length == 0 && (header.flags & H2_ACK) != 0, "SETTINGS ack");
}

settings :: () {
    out: [..] u8;

    defer array_reset(*out);

    entries := H2_Setting_Entry.[
        .{ cast(u16) H2_Setting.MAX_CONCURRENT_STREAMS, 50 },
        .{ cast(u16) H2_Setting.INITIAL_WINDOW_SIZE, 1 << 20 },
        .{ cast(u16) H2_Setting.ENABLE_PUSH, 0 },
        .{ cast(u16) H2_Setting.MAX_FRAME_SIZE, 32768 },
        .{ 99, 5 },
    ];

    h2_put_settings(*out, entries);

    header := header_of(out);

    check(header.type == .SETTINGS && header.length == 30, "SETTINGS takes six bytes a parameter");

    values: H2_Settings;
    error := h2_read_settings(.{ header.length, out.data + 9 }, *values);

    check(error == .NO_ERROR, "the parameters are accepted");
    check(values.max_concurrent_streams == 50 && values.initial_window_size == 1 << 20 && !values.enable_push && values.max_frame_size == 32768, "and applied");
    check(values.header_table_size == 4096 && values.max_header_list_size == -1, "the rest keeps its default");

    fresh: H2_Settings;

    check(fresh.enable_push && fresh.initial_window_size == 65535 && fresh.max_frame_size == 16384 && fresh.max_concurrent_streams == -1, "the defaults");

    bad: H2_Settings;

    check(h2_apply_setting(*bad, cast(u16) H2_Setting.ENABLE_PUSH, 2) == .PROTOCOL_ERROR, "ENABLE_PUSH above 1");
    check(h2_apply_setting(*bad, cast(u16) H2_Setting.INITIAL_WINDOW_SIZE, 0x80000000) == .FLOW_CONTROL_ERROR, "a window above 2^31 - 1");
    check(h2_apply_setting(*bad, cast(u16) H2_Setting.MAX_FRAME_SIZE, 16383) == .PROTOCOL_ERROR, "a frame size below 16384");
    check(h2_apply_setting(*bad, cast(u16) H2_Setting.MAX_FRAME_SIZE, 16777216) == .PROTOCOL_ERROR, "a frame size above 2^24 - 1");
    check(h2_read_settings(.[0, 3, 0, 0, 0], *bad) == .FRAME_SIZE_ERROR, "a payload that is not a multiple of six");
}

main :: () {
    frame_header();
    data_and_padding();
    headers_and_continuation();
    control_frames();
    settings();

    finish();
}
