// RFC 6455 framing and messages, with bounded RFC 7692 compression.
// The codec owns event bytes until the next parse; release it with ws_free.
Ws_Role :: enum {
    SERVER;
    CLIENT;
}

Ws_Opcode :: enum u8 {
    CONTINUATION :: 0;
    TEXT :: 1;
    BINARY :: 2;
    CLOSE :: 8;
    PING :: 9;
    PONG :: 10;
}

Ws_Error_Kind :: enum {
    NONE;
    PROTOCOL;
    UTF8;
    LIMIT;
    COMPRESSION;
    HANDSHAKE;
    TRANSPORT;
    TIMED_OUT;
    CLOSED;
}

Ws_Error :: struct {
    kind: Ws_Error_Kind;
    message: string;
    offset: s64;
}

Ws_Options :: struct {
    max_frame_size: s64 = 16 << 20;
    max_message_size: s64 = 64 << 20;
}

Ws_Deflate :: struct {
    enabled: bool;
    server_window_present: bool;
    client_window_present: bool;
    server_no_context_takeover: bool;
    client_no_context_takeover: bool;
    server_max_window_bits: int = 15;
    client_max_window_bits: int = 15;
}

Ws_Event_Kind :: enum {
    NEED_MORE;
    MESSAGE;
    PING;
    PONG;
    CLOSE;
    ERROR;
}

Ws_Event :: struct {
    kind: Ws_Event_Kind;
    opcode: Ws_Opcode;
    data: [] u8;
    code: u16;
    error: Ws_Error;
}

Ws_Codec :: struct {
    role: Ws_Role;
    options: Ws_Options;
    compression: Ws_Deflate;
    header: [14] u8;
    header_count: int;
    header_size: int = 2;
    length: s64;
    received: s64;
    offset: s64;
    opcode: Ws_Opcode;
    message_opcode: Ws_Opcode;
    final: bool;
    masked: bool;
    compressed: bool;
    fragmented: bool;
    delivered: bool;
    closed: bool;
    utf_remaining: int;
    utf_low: u8 = 0x80;
    utf_high: u8 = 0xBF;
    message: [..] u8;
    control: [..] u8;
    dictionary: [..] u8;
    error: Ws_Error;
}

// Parses any input chunk, stopping at one message or control event. Example: used, event := ws_parse(*codec, bytes).
ws_parse :: (c: *Ws_Codec, bytes: [] u8) -> used: s64, event: Ws_Event {
    if c.error.kind != .NONE return 0, .{ kind = .ERROR, error = c.error };
    if c.closed return 0, .{ kind = .CLOSE };

    if c.delivered {
        c.message.count = 0;
        c.delivered = false;
    }

    while used < bytes.count {
        if c.header_count < c.header_size {
            c.header[c.header_count] = bytes[used];
            c.header_count += 1;
            used += 1;
            c.offset += 1;
            if c.header_count == 2 {
                tag := c.header[1] & 127;

                c.header_size = 2 + (ifx tag == 126 then 2 else ifx tag == 127 then 8 else 0) + (ifx c.header[1] & 128 then 4 else 0);
                // Reject invalid short headers before waiting for extended lengths or keys.
                c.error = check_header(c);
                if c.error.kind != .NONE return used, .{ kind = .ERROR, error = c.error };
            }

            if c.header_count < c.header_size continue;

            c.error = begin_frame(c);
            if c.error.kind != .NONE return used, .{ kind = .ERROR, error = c.error };
        }

        control := cast(u8) c.opcode >= 8;
        into := *c.message;

        if control into = *c.control;

        count := min(bytes.count - used, c.length - c.received);
        start := into.count;

        array_reserve(into, start + count);

        for i: 0..count - 1 {
            byte := bytes[used + i];
            if c.masked byte ^= c.header[c.header_size - 4 + ((c.received + i) & 3)];

            into.data[start + i] = byte;

            if !control && !c.compressed && c.message_opcode == .TEXT && !utf_byte(c, byte) {
                c.error = .{ .UTF8, "invalid text UTF-8", c.offset + i };
                return used + i + 1, .{ kind = .ERROR, error = c.error };
            }
        }

        into.count += count;
        used += count;
        c.offset += count;
        c.received += count;
        if c.received != c.length continue;

        c.header_count = 0;
        c.header_size = 2;

        if control {
            event.opcode = c.opcode;
            event.data = c.control;
            if c.opcode == .PING event.kind = .PING;
            else if c.opcode == .PONG event.kind = .PONG;
            else {
                event.kind = .CLOSE;
                c.error = check_close(c.control, c.offset - c.length);
                if c.error.kind != .NONE return used, .{ kind = .ERROR, error = c.error };

                c.closed = true;
                if c.control.count >= 2 event.code = ((cast(u16) c.control[0]) << 8) | cast(u16) c.control[1];
            }

            return used, event;
        }

        if !c.final continue;

        c.fragmented = false;
        if c.compressed {
            window_bits := ifx c.role == .SERVER then c.compression.client_max_window_bits else c.compression.server_max_window_bits;
            output, ok, why := Deflate.inflate_sync(c.message, c.dictionary, c.options.max_message_size, window_bits);

            if !ok {
                c.error = .{ .COMPRESSION, why, c.offset - c.length };
                return used, .{ kind = .ERROR, error = c.error };
            }

            c.message.count = 0;
            append_data(*c.message, output);
            free(output.data);

            if c.message_opcode == .TEXT {
                for c.message if !utf_byte(c, it) {
                    c.error = .{ .UTF8, "invalid decompressed text UTF-8", c.offset };
                    return used, .{ kind = .ERROR, error = c.error };
                }
            }

            no_context := ifx c.role == .SERVER then c.compression.client_no_context_takeover else c.compression.server_no_context_takeover;
            if !no_context {
                bits := ifx c.role == .SERVER then c.compression.client_max_window_bits else c.compression.server_max_window_bits;
                retain_history(*c.dictionary, c.message, bits);
            }
        }

        if c.utf_remaining != 0 {
            c.error = .{ .UTF8, "incomplete text UTF-8", c.offset };
            return used, .{ kind = .ERROR, error = c.error };
        }

        c.delivered = true;

        return used, .{ kind = .MESSAGE, opcode = c.message_opcode, data = c.message };
    }

    return used, .{};
}

// Appends one frame with an explicit masking key. Example: ws_frame(*out, .TEXT, cast([] u8) "hi").
// Client transports must obtain a fresh unpredictable key for every call.
ws_frame :: (out: *[..] u8, opcode: Ws_Opcode, data: [] u8, final := true, masked := false, key: [4] u8 = .{}, compressed := false, options := Ws_Options.{}) -> Ws_Error {
    op := cast(u8) opcode;
    if op != 0 && op != 1 && op != 2 && op != 8 && op != 9 && op != 10 return .{ .PROTOCOL, "reserved opcode", 0 };
    if data.count > options.max_frame_size || options.max_frame_size < 0 return .{ .LIMIT, "frame exceeds limit", 0 };
    if op >= 8 && (!final || data.count > 125 || compressed) return .{ .PROTOCOL, "invalid control frame", 0 };
    if compressed && op != 1 && op != 2 return .{ .PROTOCOL, "invalid compression bit", 0 };

    if opcode == .CLOSE {
        error := check_close(data, 0);
        if error.kind != .NONE return error;
    }

    array_add(out, op | (ifx final then cast(u8) 128 else 0) | (ifx compressed then cast(u8) 64 else 0));
    mask: u8 = ifx masked then 128 else 0;
    if data.count < 126 array_add(out, mask | cast(u8) data.count);
    else {
        array_add(out, mask | (ifx data.count <= 65535 then cast(u8) 126 else 127));
        width := ifx data.count <= 65535 then 2 else 8;
        for < i: 0..width - 1 array_add(out, cast,trunc(u8) (cast(u64) data.count >> (i * 8)));
    }

    if masked for key array_add(out, it);

    start := out.count;

    append_data(out, data);
    if masked for i: 0..data.count - 1 out.data[start + i] ^= key[i & 3];

    return .{};
}

// Releases codec buffers and resets all state. Example: defer ws_free(*codec).
ws_free :: (c: *Ws_Codec) {
    array_free(c.message);
    array_free(c.control);
    array_free(c.dictionary);
    c.* = .{};
}

#scope_module
#import "Basic";
Deflate :: #import "Extensions/Deflate";

append_data :: (out: *[..] u8, data: [] u8) {
    if data.count == 0 return;
    array_reserve(out, out.count + data.count);
    memcpy(out.data + out.count, data.data, data.count);
    out.count += data.count;
}

check_header :: (c: *Ws_Codec) -> Ws_Error {
    first := c.header[0];
    op := first & 15;

    if first & 48 || (op != 0 && op != 1 && op != 2 && op != 8 && op != 9 && op != 10) return .{ .PROTOCOL, "reserved bits or opcode", c.offset - 2 };
    if (c.header[1] & 128 != 0) != (c.role == .SERVER) return .{ .PROTOCOL, "incorrect masking for role", c.offset - 1 };
    if op >= 8 && (!(first & 128) || (c.header[1] & 127) > 125 || first & 64) return .{ .PROTOCOL, "invalid control frame", c.offset - 2 };
    if first & 64 && (!c.compression.enabled || op == 0) return .{ .PROTOCOL, "unexpected compression bit", c.offset - 2 };
    if op < 8 && ((op == 0) != c.fragmented) return .{ .PROTOCOL, "invalid continuation sequence", c.offset - 2 };

    return .{};
}

begin_frame :: (c: *Ws_Codec) -> Ws_Error {
    tag := c.header[1] & 127;
    size: u64 = tag;

    if tag >= 126 {
        size = 0;
        width := ifx tag == 126 then 2 else 8;
        for i: 0..width - 1 size = (size << 8) | cast(u64) c.header[2 + i];
        if (tag == 126 && size < 126) || (tag == 127 && (size < 65536 || c.header[2] & 128)) return .{ .PROTOCOL, "noncanonical frame length", c.offset - c.header_size };
    }

    if c.options.max_frame_size < 0 || size > cast(u64) c.options.max_frame_size return .{ .LIMIT, "frame exceeds limit", c.offset - c.header_size };

    c.opcode = cast(Ws_Opcode) (c.header[0] & 15);
    c.final = (c.header[0] & 128) != 0;
    c.masked = (c.header[1] & 128) != 0;
    c.length = cast(s64) size;
    c.received = 0;
    if cast(u8) c.opcode >= 8 {
        c.control.count = 0;
        return .{};
    }

    if c.options.max_message_size < 0 || c.length > c.options.max_message_size - c.message.count return .{ .LIMIT, "message exceeds limit", c.offset - c.header_size };

    if c.opcode != .CONTINUATION {
        c.message_opcode = c.opcode;
        c.compressed = (c.header[0] & 64) != 0;
        c.utf_remaining = 0;
    }

    c.fragmented = !c.final;

    return .{};
}

utf_byte :: (c: *Ws_Codec, byte: u8) -> bool {
    if c.utf_remaining > 0 {
        if byte < c.utf_low || byte > c.utf_high return false;

        c.utf_remaining -= 1;
        c.utf_low = 0x80;
        c.utf_high = 0xBF;

        return true;
    }

    c.utf_low = 0x80;
    c.utf_high = 0xBF;
    if byte < 0x80 return true;

    if byte >= 0xC2 && byte <= 0xDF c.utf_remaining = 1;
    else if byte >= 0xE0 && byte <= 0xEF {
        c.utf_remaining = 2;
        if byte == 0xE0 c.utf_low = 0xA0;
        if byte == 0xED c.utf_high = 0x9F;
    } else if byte >= 0xF0 && byte <= 0xF4 {
        c.utf_remaining = 3;
        if byte == 0xF0 c.utf_low = 0x90;
        if byte == 0xF4 c.utf_high = 0x8F;
    } else return false;

    return true;
}

check_close :: (data: [] u8, offset: s64) -> Ws_Error {
    if data.count == 1 return .{ .PROTOCOL, "truncated close code", offset };
    if data.count == 0 return .{};

    code := ((cast(u16) data[0]) << 8) | cast(u16) data[1];
    if code < 1000 || code >= 5000 || (code >= 1016 && code < 3000) || code == 1004 || code == 1005 || code == 1006 || code == 1015 return .{ .PROTOCOL, "invalid close code", offset };

    c: Ws_Codec;
    for i: 2..data.count - 1 if !utf_byte(*c, data[i]) return .{ .UTF8, "invalid close reason", offset + i };
    if c.utf_remaining != 0 return .{ .UTF8, "incomplete close reason", offset + data.count };

    return .{};
}

#load "handshake.jai";

#if OS != .WASM {
    #load "connection.jai";
}

#scope_module

retain_history :: (history: *[..] u8, message: [] u8, bits: int) {
    window := (cast(s64) 1) << clamp(bits, 8, 15);
    keep := min(message.count, window);
    old := min(history.count, window - keep);

    for i: 0..old - 1 history.data[i] = history.data[i + history.count - old];

    history.count = old;
    append_data(history, .{ keep, message.data + message.count - keep });
}
