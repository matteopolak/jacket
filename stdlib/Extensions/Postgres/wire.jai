// Messages on the wire: a type byte (none for the first messages a client sends), a big-endian
// length that counts itself, and the body. Integers are big-endian throughout.

// Messages the server may send are never larger than this; a bigger length means the stream is
// out of step or hostile.
MAX_MESSAGE_BYTES :: 1 << 30;

// A message starts with its type byte and its four-byte length.
HEADER_BYTES :: 5;

// Writing: messages go into `connection.out` and leave with `send`.

// Starts a message in the output buffer, with its length left to fill in.
//
// - c: the connection whose output buffer gets the message.
// - type: the message type byte; 0 for none, as in the first messages a client sends.
begin_message :: (c: *Connection, type: u8) {
    if type array_add(*c.out, type);
    c.message_type = type;
    c.message_start = c.out.count;
    put_u32(*c.out, 0);
}

// Fills in the length of the message `begin_message` started.
//
// - c: the connection with a message begun.
end_message :: (c: *Connection) {
    length := c.out.count - c.message_start;

    write_u32(c.out.data + c.message_start, cast(u32) length);

    if c.on_trace && c.message_type {
        trace_message(c, true, c.message_type, .{ length - 4, c.out.data + c.message_start + 4 });
    }
}

// The messages of the extended protocol, as pipelines and `run_extended` queue them.

// Adds a Parse message to the output.
//
// - c: the connection whose output buffer gets the message.
// - name: the statement's name; empty for the unnamed statement.
// - sql: the statement text, with `$n` for parameters.
// - types: the OIDs of the parameter types, in order; may be empty.
put_parse :: (c: *Connection, name: string, sql: string, types: [] u32) {
    begin_message(c, #char "P");
    put_cstring(*c.out, name);
    put_cstring(*c.out, sql);
    put_u16(*c.out, cast(u16) types.count);
    for types put_u32(*c.out, it);

    end_message(c);
}

// Parameters go as text unless one needs another format; the format codes are left out when
// every parameter is text.
//
// - c: the connection whose output buffer gets the message.
// - portal: the portal to create; empty for the unnamed one.
// - statement: the prepared statement to bind; empty for the unnamed one.
// - params: the parameter values with their formats.
// - result_format: the format for every result column.
put_bind :: (c: *Connection, portal: string, statement: string, params: [] Param, result_format: Format) {
    all_text := true;
    for params if it.format != .TEXT all_text = false;

    begin_message(c, #char "B");
    put_cstring(*c.out, portal);
    put_cstring(*c.out, statement);

    if all_text {
        put_u16(*c.out, 0);
    } else {
        put_u16(*c.out, cast(u16) params.count);
        for params put_u16(*c.out, xx it.format);
    }

    put_u16(*c.out, cast(u16) params.count);

    for params {
        if it.null {
            put_u32(*c.out, 0xffff_ffff);
        } else {
            put_u32(*c.out, cast(u32) it.data.count);
            put_bytes(*c.out, it.data);
        }
    }

    put_u16(*c.out, 1);
    put_u16(*c.out, xx result_format);
    end_message(c);
}

// Adds a Describe message to the output.
//
// - c: the connection whose output buffer gets the message.
// - kind: `S` for a prepared statement, `P` for a portal.
// - name: its name; empty for the unnamed one.
put_describe :: (c: *Connection, kind: u8, name: string) {
    begin_message(c, #char "D");
    put_u8(*c.out, kind);
    put_cstring(*c.out, name);
    end_message(c);
}

// Adds a Close message to the output.
//
// - c: the connection whose output buffer gets the message.
// - kind: `S` for a prepared statement, `P` for a portal.
// - name: its name; empty for the unnamed one.
put_close :: (c: *Connection, kind: u8, name: string) {
    begin_message(c, #char "C");
    put_u8(*c.out, kind);
    put_cstring(*c.out, name);
    end_message(c);
}

// Adds an Execute message to the output.
//
// - c: the connection whose output buffer gets the message.
// - portal: the portal to run; empty for the unnamed one.
// - limit: the most rows to return; 0 for all of them.
put_execute :: (c: *Connection, portal: string, limit: u32) {
    begin_message(c, #char "E");
    put_cstring(*c.out, portal);
    put_u32(*c.out, limit);
    end_message(c);
}

// Adds a Sync message to the output.
//
// - c: the connection whose output buffer gets the message.
put_sync :: (c: *Connection) {
    begin_message(c, #char "S");
    end_message(c);
}

// Adds a Flush message to the output.
//
// - c: the connection whose output buffer gets the message.
put_flush :: (c: *Connection) {
    begin_message(c, #char "H");
    end_message(c);
}

// Writes the output buffer to the server.
//
// - c: the connection whose output buffer is written out and emptied.
send :: (c: *Connection) -> Pg_Error {
    if c.broken return make_error(.CLOSED, "the connection was broken by an earlier failure");
    if !c.out.count return .{};

    _, io := Io.write(c.writer, c.out);

    c.out.count = 0;

    if io != .NONE {
        c.broken = true;
        return io_failure(io, "could not send to the server");
    }

    return .{};
}

// - out: the buffer to append to.
// - value: the byte.
put_u8 :: (out: *[..] u8, value: u8) {
    array_add(out, value);
}

// - out: the buffer to append to.
// - value: the number, written big-endian.
put_u16 :: (out: *[..] u8, value: u16) {
    array_add(out, cast,trunc(u8) (value >> 8), cast,trunc(u8) value);
}

// - out: the buffer to append to.
// - value: the number, written big-endian.
put_u32 :: (out: *[..] u8, value: u32) {
    array_add(out, cast,trunc(u8) (value >> 24), cast,trunc(u8) (value >> 16), cast,trunc(u8) (value >> 8), cast,trunc(u8) value);
}

// - out: the buffer to append to.
// - value: the number, written big-endian.
put_u64 :: (out: *[..] u8, value: u64) {
    put_u32(out, cast,trunc(u32) (value >> 32));
    put_u32(out, cast,trunc(u32) value);
}

// Room for `total` items, growing by doubling so a long run of small appends copies little.
//
// - array: the array to make room in.
// - total: how many items it should be able to hold.
reserve_for :: (array: *[..] $T, total: s64) {
    if total <= array.allocated return;
    array_reserve(array, max(total, array.allocated * 2, 64));
}

// Appends bytes.
//
// - out: the buffer to append to.
// - bytes: what to append.
put_bytes :: (out: *[..] u8, bytes: [] u8) {
    if !bytes.count return;

    start := out.count;

    reserve_for(out, start + bytes.count);
    out.count = start + bytes.count;
    memcpy(out.data + start, bytes.data, bytes.count);
}

// Appends the bytes of a string.
//
// - out: the buffer to append to.
// - text: what to append, as it is.
put_bytes :: (out: *[..] u8, text: string) {
    put_bytes(out, cast([] u8) text);
}

// A C string: the text and a zero. Text holding a zero would end early, so callers check first.
//
// - out: the buffer to append to.
// - text: the text, which must hold no zero byte.
put_cstring :: (out: *[..] u8, text: string) {
    put_bytes(out, text);
    array_add(out, 0);
}

// - at: where to write four bytes.
// - value: the number, written big-endian.
write_u32 :: (at: *u8, value: u32) {
    at[0] = cast,trunc(u8) (value >> 24);
    at[1] = cast,trunc(u8) (value >> 16);
    at[2] = cast,trunc(u8) (value >> 8);
    at[3] = cast,trunc(u8) value;
}

// Reads a big-endian number.
//
// - at: where the two bytes start.
read_u16 :: (at: *u8) -> u16 {
    return ((cast(u16) at[0]) << 8) | at[1];
}

// Reads a big-endian number.
//
// - at: where the four bytes start.
read_u32 :: (at: *u8) -> u32 {
    return ((cast(u32) at[0]) << 24) | ((cast(u32) at[1]) << 16) | ((cast(u32) at[2]) << 8) | at[3];
}

// Reads a big-endian number.
//
// - at: where the eight bytes start.
read_u64 :: (at: *u8) -> u64 {
    return ((cast(u64) read_u32(at)) << 32) | read_u32(at + 4);
}

// Reading: `read_message` returns the next message; its body is valid until the next read.

// - c: the connection to read from.
read_message :: (c: *Connection) -> type: u8, body: [] u8, Pg_Error {
    if c.broken return 0, .[], make_error(.CLOSED, "the connection was broken by an earlier failure");

    error := fill(c, HEADER_BYTES);
    if error.kind != .NONE return 0, .[], error;

    start := c.input.data + c.input_start;
    type := start[0];
    length := cast(s64) read_u32(start + 1);

    if length < 4 || length > MAX_MESSAGE_BYTES {
        c.broken = true;
        return 0, .[], make_error(.PROTOCOL, "the server sent a message of % bytes", length);
    }

    error = fill(c, 1 + length);
    if error.kind != .NONE return 0, .[], error;

    body := [] u8.{ length - 4, c.input.data + c.input_start + HEADER_BYTES };

    c.input_start += 1 + length;

    if c.on_trace trace_message(c, false, type, body);

    return type, body, .{};
}

// Makes at least `count` unread bytes available.
//
// - c: the connection to read from.
// - count: how many unread bytes must be in its input buffer.
fill :: (c: *Connection, count: s64) -> Pg_Error {
    if c.input.count - c.input_start >= count return .{};

    // Move what is left to the front, then read more after it.
    left := c.input.count - c.input_start;

    if c.input_start > 0 {
        if left memcpy(c.input.data, c.input.data + c.input_start, left);
        c.input.count = left;
        c.input_start = 0;
    }

    want := max(count, 16 * 1024);
    if c.input.allocated < want array_reserve(*c.input, want);

    while c.input.count < count {
        room := [] u8.{ c.input.allocated - c.input.count, c.input.data + c.input.count };
        got, io := Io.read(c.reader, room);

        if io != .NONE {
            // A deadline or cancel loses no bytes, so the stream is still in step: the request
            // decides whether to recover it (`recover_interrupted`).
            if io != .TIMED_OUT && io != .CANCELED c.broken = true;

            // A fatal alert from the server (a client certificate it refused, say) is the reason.
            #if TLS {
                if c.tls && io == .CONNECTION_ABORTED {
                    failure := Tls.last_error(c.tls);
                    if failure.kind == .PROTOCOL && failure.alert > 0 return make_error(.TLS, tls_failure_text(failure));
                }
            }

            return io_failure(io, "could not read from the server");
        }

        if got == 0 {
            c.broken = true;
            return make_error(.CLOSED, "the server closed the connection");
        }

        c.input.count += got;
    }

    return .{};
}

// Reading inside a message body. A read past the end sets `bad` and returns zeros, so a parser
// checks `bad` once at the end.
Cursor :: struct {
    data: [] u8;
    at: s64;
    bad: bool;
}

// - r: the cursor, which moves past the byte.
get_u8 :: (r: *Cursor) -> u8 {
    if r.at + 1 > r.data.count {
        r.bad = true;
        return 0;
    }

    r.at += 1;

    return r.data[r.at - 1];
}

// - r: the cursor, which moves past the number.
get_u16 :: (r: *Cursor) -> u16 {
    if r.at + 2 > r.data.count {
        r.bad = true;
        return 0;
    }

    r.at += 2;

    return read_u16(r.data.data + r.at - 2);
}

// - r: the cursor, which moves past the number.
get_u32 :: (r: *Cursor) -> u32 {
    if r.at + 4 > r.data.count {
        r.bad = true;
        return 0;
    }

    r.at += 4;

    return read_u32(r.data.data + r.at - 4);
}

// - r: the cursor, which moves past the number.
get_s16 :: (r: *Cursor) -> s16 {
    return cast,no_check(s16) get_u16(r);
}

// - r: the cursor, which moves past the number.
get_s32 :: (r: *Cursor) -> s32 {
    return cast,no_check(s32) get_u32(r);
}

// - r: the cursor, which moves past the bytes.
// - count: how many bytes to take; the result points into the cursor's data.
get_bytes :: (r: *Cursor, count: s64) -> [] u8 {
    if count < 0 || r.at + count > r.data.count {
        r.bad = true;
        return .[];
    }

    r.at += count;

    return .{ count, r.data.data + r.at - count };
}

// - r: the cursor, which moves past the zero that ends the text.
get_cstring :: (r: *Cursor) -> string {
    start := r.at;

    while r.at < r.data.count && r.data[r.at] != 0 r.at += 1;

    if r.at >= r.data.count {
        r.bad = true;
        return "";
    }

    r.at += 1;

    return .{ r.at - 1 - start, r.data.data + start };
}

// The bytes left in the cursor's data.
//
// - r: the cursor, which moves to the end.
rest :: (r: *Cursor) -> [] u8 {
    return get_bytes(r, r.data.count - r.at);
}
