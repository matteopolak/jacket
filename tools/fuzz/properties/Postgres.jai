// The Postgres client against a scripted server. A `Connection` reads the case's bytes as the
// server's reply and drops what it sends, so the real message reader, decoders and COPY code run
// without a server. Whatever the reply holds, a call returns (a value or an error): it must not
// crash, read outside its buffers or loop. The round trips check that what the client encodes it
// decodes back to the same value.
Io :: #import "Extensions/Io";

// The scripted server: `reply` is read in pieces of at most `piece` bytes, then the stream ends.
// What the client writes is kept in `sent`.
Fake_Server :: struct {
    reply: [] u8;
    at: s64;
    piece: s64;
    sent: [..] u8;
}

fake_read :: (data: *void, buffer: [] u8) -> s64, Io.Io_Error {
    server := cast(*Fake_Server) data;
    count := min(buffer.count, server.reply.count - server.at, server.piece);

    if count <= 0 return 0, .NONE;

    memcpy(buffer.data, server.reply.data + server.at, count);
    server.at += count;

    return count, .NONE;
}

fake_write :: (data: *void, bytes: [] u8) -> s64, Io.Io_Error {
    server := cast(*Fake_Server) data;
    array_add(*server.sent, ..bytes);
    return bytes.count, .NONE;
}

quiet_notice :: (c: *Target.Connection, notice: *Target.Pg_Error) {}

// A connection past its startup, talking to `server`.
fake_connection :: (server: *Fake_Server) -> Target.Connection {
    c: Target.Connection;
    c.reader = .{ data = server, read_proc = fake_read };
    c.writer = .{ data = server, write_proc = fake_write };
    c.on_notice = quiet_notice;

    return c;
}

// The parts of a connection a call may have filled in.
free_fake_connection :: (c: *Target.Connection, server: *Fake_Server) {
    for c.notifications Target.free_notification(*it);

    array_free(c.notifications);

    for c.parameters {
        free(it.name);
        free(it.value);
    }

    array_free(c.parameters);
    array_free(c.out);
    array_free(c.input);
    array_free(server.sent);
}

// Runs `body` on a fiber, as every Postgres call must. The fiber has temporary storage of its own,
// freed when it ends: what it hands back goes in the caller's, through `in_caller_temp`.
on_fiber :: (body: (data: *$T), data: *T) {
    caller_storage = context.temporary_storage;
    Io.io_run(body, data);
}

caller_storage: *Temporary_Storage;

in_caller_temp :: (body: Code) #expand {
    fiber_storage := context.temporary_storage;
    context.temporary_storage = caller_storage;
    defer context.temporary_storage = fiber_storage;
    #insert body;
}

// Building replies.

put_u8 :: (out: *[..] u8, value: u8) {
    array_add(out, value);
}

put_u16 :: (out: *[..] u8, value: u16) {
    array_add(out, cast,trunc(u8) (value >> 8), cast,trunc(u8) value);
}

put_u32 :: (out: *[..] u8, value: u32) {
    array_add(out, cast,trunc(u8) (value >> 24), cast,trunc(u8) (value >> 16), cast,trunc(u8) (value >> 8), cast,trunc(u8) value);
}

put_cstring :: (out: *[..] u8, text: string) {
    array_add(out, ..cast([] u8) text);
    array_add(out, 0);
}

// Appends a message: its type, a length to fill in, then whatever `fill` adds.
// A message's type byte and length.
HEADER_SIZE :: 5;

begin :: (out: *[..] u8, type: u8) -> s64 {
    array_add(out, type);
    start := out.count;
    put_u32(out, 0);
    return start;
}

finish :: (out: *[..] u8, start: s64) {
    length := cast(u32) (out.count - start);

    out.*[start] = cast,trunc(u8) (length >> 24);
    out.*[start + 1] = cast,trunc(u8) (length >> 16);
    out.*[start + 2] = cast,trunc(u8) (length >> 8);
    out.*[start + 3] = cast,trunc(u8) length;
}

simple :: (out: *[..] u8, type: u8) {
    finish(out, begin(out, type));
}

ready :: (out: *[..] u8) {
    m := begin(out, #char "Z");
    put_u8(out, #char "I");
    finish(out, m);
}

complete :: (out: *[..] u8, tag: string) {
    m := begin(out, #char "C");
    put_cstring(out, tag);
    finish(out, m);
}

Reply_Column :: struct {
    name: string;
    oid: u32;
    format: u16;
}

row_description :: (out: *[..] u8, columns: [] Reply_Column) {
    m := begin(out, #char "T");

    put_u16(out, cast(u16) columns.count);

    for columns {
        put_cstring(out, it.name);
        put_u32(out, 0);
        put_u16(out, 0);
        put_u32(out, it.oid);
        put_u16(out, 0xFFFF);
        put_u32(out, 0xFFFF_FFFF);
        put_u16(out, it.format);
    }

    finish(out, m);
}

// One row; a null value is a slice with a null data pointer.
data_row :: (out: *[..] u8, values: [][] u8) {
    m := begin(out, #char "D");

    put_u16(out, cast(u16) values.count);

    for values {
        if it.data == null && it.count == 0 {
            put_u32(out, 0xFFFF_FFFF);
        } else {
            put_u32(out, cast(u32) it.count);
            array_add(out, ..it);
        }
    }

    finish(out, m);
}

// One row of one value.
one_value :: (value: [] u8) -> [][][] u8 {
    rows := NewArray(1, [][] u8,, temp);
    rows[0] = NewArray(1, [] u8,, temp);
    rows[0][0] = value;
    return rows;
}

// What an extended query expects: ParseComplete and BindComplete, the rows, the tag, ReadyForQuery.
query_reply :: (out: *[..] u8, columns: [] Reply_Column, rows: [][][] u8) {
    simple(out, #char "1");
    simple(out, #char "2");
    row_description(out, columns);
    for rows data_row(out, it);

    complete(out, tprint("SELECT %", rows.count));
    ready(out);
}

// The types a column can have, with their array types.
COLUMN_OIDS :: u32.[16, 17, 18, 19, 20, 21, 23, 25, 26, 114, 700, 701, 705, 1042, 1043, 1082, 1083, 1114, 1184, 1186, 1700, 2950, 3802,
    1000, 1001, 1002, 1003, 1005, 1007, 1009, 1014, 1015, 1016, 1028, 1021, 1022, 1115, 1182, 1183, 1185, 1187, 1231, 2951, 3807, 199];

random_oid :: (r: *Rng) -> u32 {
    if chance(r, 16) return gen(r, u32);
    return COLUMN_OIDS[below(r, COLUMN_OIDS.count)];
}

random_bytes :: (r: *Rng) -> [] u8 {
    return cast([] u8) gen(r, string);
}

// Reads one value as every Jai type `get` takes. Any of them may fail; none may crash.
get_every_way :: (result: *Target.Result, row: s64, column: s64) {
    Target.is_null(result, row, column);
    Target.raw_value(result, row, column);
    Target.get(result, row, column, bool);
    Target.get(result, row, column, s8);
    Target.get(result, row, column, s16);
    Target.get(result, row, column, s32);
    Target.get(result, row, column, s64);
    Target.get(result, row, column, u8);
    Target.get(result, row, column, u16);
    Target.get(result, row, column, u32);
    Target.get(result, row, column, u64);
    Target.get(result, row, column, float32);
    Target.get(result, row, column, float64);
    Target.get(result, row, column, string);
    Target.get(result, row, column, [] u8);
    Target.get(result, row, column, Target.Uuid);
    Target.get(result, row, column, Target.Timestamp);
    Target.get(result, row, column, Target.Local_Timestamp);
    Target.get(result, row, column, Target.Date);
    Target.get(result, row, column, Target.Time);
    Target.get(result, row, column, Target.Interval);
    Target.get(result, row, column, Target.Numeric);
    Target.get(result, row, column, Target.Json_Raw);
    Target.get(result, row, column, Target.Nullable(s64));
    Target.get(result, row, column, Target.Nullable(string));
    Target.get(result, row, column, [] s64);
    Target.get(result, row, column, [] s16);
    Target.get(result, row, column, [] float64);
    Target.get(result, row, column, [] string);
    Target.get(result, row, column, [] bool);
    Target.get(result, row, column, [] Target.Nullable(s32));
    Target.get(result, row, column, [] Target.Numeric);
    Target.get(result, row, column, [] Target.Timestamp);
    Target.get(result, row, column, [] Target.Uuid);
    Target.get(result, row, column, [][] u8);
}

// Runs one query against `reply`, in text or binary, read in pieces of `piece` bytes, and reads
// every value of a result it returns.
Query_Case :: struct {
    reply: [] u8;
    piece: s64;
    binary: bool;
    error: Target.Pg_Error;
}

run_query_case :: (q: *Query_Case) {
    server := Fake_Server.{ reply = q.reply, piece = q.piece };
    c := fake_connection(*server);

    defer free_fake_connection(*c, *server);

    result: Target.Result;
    if q.binary result, q.error = Target.query_binary(*c, "select fuzz");
    else result, q.error = Target.query_dynamic(*c, "select fuzz");

    if q.error.kind == .NONE {
        for row: 0..result.row_count - 1 for column : 0..result.columns.count - 1 get_every_way(*result, row, column);
        Target.free_result(*result);
    }

    Target.free_error(*q.error);
}

piece_size :: (r: *Rng) -> s64 {
    return ifx chance(r, 2) then 16 * 1024 else 1 + below(r, 8);
}

// A reply of a few columns of any type and a few rows of any bytes: valid as messages, nonsense
// as values.
random_valid_reply :: (r: *Rng) -> [..] u8 {
    out: [..] u8;
    out.allocator = temp;

    columns := NewArray(below(r, 4), Reply_Column,, temp);
    for *columns it.* = .{ tprint("c%", it_index), random_oid(r), cast(u16) below(r, 2) };

    rows := NewArray(below(r, 4), [][] u8,, temp);
    for *rows {
        values := NewArray(columns.count, [] u8,, temp);
        for *values if !chance(r, 6) it.* = random_bytes(r);
        it.* = values;
    }

    query_reply(*out, columns, rows);

    return out;
}

// 1. Message reading: any bytes from the server.
fuzz_pg_reply_random_bytes :: (r: *Rng) {
    q := Query_Case.{ reply = random_bytes(r), piece = piece_size(r), binary = chance(r, 2) };
    on_fiber(run_query_case, *q);
}

// A valid reply cut short anywhere is an error: the server closed the connection.
fuzz_pg_reply_cut_short :: (r: *Rng) {
    reply := random_valid_reply(r);
    if !reply.count return;

    cut := below(r, reply.count);
    q := Query_Case.{ reply = array_view(reply, 0, cut), piece = piece_size(r), binary = chance(r, 2) };

    on_fiber(run_query_case, *q);
}

// A valid reply with a few bytes changed, inserted or removed, with unsolicited messages (notices,
// notifications, parameter changes) among the others.
fuzz_pg_reply_mutated :: (r: *Rng) {
    base := random_valid_reply(r);
    reply: [..] u8;
    reply.allocator = temp;

    for base {
        if chance(r, 40) unsolicited(r, *reply);
        if chance(r, 60) continue;
        if chance(r, 60) array_add(*reply, cast,trunc(u8) gen(r, u8));
        array_add(*reply, ifx chance(r, 50) then cast,trunc(u8) gen(r, u8) else it);
    }

    q := Query_Case.{ reply = reply, piece = piece_size(r), binary = chance(r, 2) };

    on_fiber(run_query_case, *q);
}

unsolicited :: (r: *Rng, out: *[..] u8) {
    if below(r, 3) == {
        case 0;
            m := begin(out, #char "N");

            put_u8(out, #char "S");
            put_cstring(out, "NOTICE");
            put_u8(out, #char "M");
            put_cstring(out, gen(r, string));
            put_u8(out, 0);
            finish(out, m);
        case 1;
            m := begin(out, #char "A");

            put_u32(out, gen(r, u32));
            put_cstring(out, gen(r, string));
            put_cstring(out, gen(r, string));
            finish(out, m);
        case;
            m := begin(out, #char "S");

            put_cstring(out, ifx chance(r, 2) then "server_version" else gen(r, string));
            put_cstring(out, gen(r, string));
            finish(out, m);
    }
}

// 2. Value decoding: one value of any bytes, as a column of any known type, in text or binary.
fuzz_pg_decode_value :: (r: *Rng) {
    out: [..] u8;
    out.allocator = temp;

    value := random_bytes(r);
    column := Reply_Column.{ "v", random_oid(r), cast(u16) below(r, 2) };

    query_reply(*out, .[column], one_value(value));

    q := Query_Case.{ reply = out, piece = 16 * 1024, binary = column.format == 1 };

    on_fiber(run_query_case, *q);
    assert(q.error.kind == .NONE, "a well-formed reply is a result, whatever its values hold");
}

// Round trips: a value sent as a parameter comes back as itself when the server returns the same
// bytes as a column of the parameter's type and format.

// The parameters of the Parse and Bind the client sent: each one's type, format and bytes.
Sent_Param :: struct {
    oid: u32;
    format: u16;
    bytes: [] u8;
    null: bool;
}

sent_params :: (sent: [] u8) -> [] Sent_Param, ok: bool {
    params: [..] Sent_Param;
    params.allocator = temp;
    oids: [..] u32;
    oids.allocator = temp;

    at := 0;
    u16_at :: (at: s64) -> u16 #expand {
        return ((cast(u16) sent[at]) << 8) | sent[at + 1];
    }
    u32_at :: (at: s64) -> u32 #expand {
        return ((cast(u32) sent[at]) << 24) | ((cast(u32) sent[at + 1]) << 16) | ((cast(u32) sent[at + 2]) << 8) | sent[at + 3];
    }

    skip_cstring :: () #expand {
        while sent[`at] != 0 `at += 1;
        `at += 1;
    }

    while at + HEADER_SIZE <= sent.count {
        type := sent[at];
        length := cast(s64) u32_at(at + 1);
        body := at + HEADER_SIZE;
        next := at + 1 + length;

        if next > sent.count return .[], false;

        if type == #char "P" {
            at = body;
            skip_cstring();
            skip_cstring();
            count := u16_at(at);

            at += 2;

            for 0..count - 1 {
                array_add(*oids, u32_at(at));
                at += 4;
            }
        } else if type == #char "B" {
            at = body;
            skip_cstring();
            skip_cstring();
            format_count := cast(s64) u16_at(at);

            at += 2;
            formats := array_view(sent, at, format_count * 2);

            at += format_count * 2;
            count := cast(s64) u16_at(at);

            at += 2;

            for i: 0..count - 1 {
                p: Sent_Param;
                p.oid = ifx i < oids.count then oids[i] else 0;
                if format_count == 1 p.format = (cast(u16) formats[0] << 8) | formats[1];
                else if format_count > 1 p.format = (cast(u16) formats[2 * i] << 8) | formats[2 * i + 1];

                n := cast,no_check(s32) u32_at(at);

                at += 4;

                if n < 0 {
                    p.null = true;
                } else {
                    p.bytes = cast([] u8) copy_string(cast(string) array_view(sent, at, n),, temp);
                    at += n;
                }

                array_add(*params, p);
            }
        }

        at = next;
    }

    return params, true;
}

// Sends `value` as the one parameter of a query, then answers with the bytes it sent as a column
// of the same type and format, and reads that column as a `T`.
round_trip :: (value: $T, binary_result := false) -> T, sent: bool, Target.Pg_Error {
    State :: struct {
        value: T;
        back: T;
        sent: bool;
        error: Target.Pg_Error;
    }

    body :: (s: *State) {
        // First the query as the client sends it, to see the parameter.
        probe := Fake_Server.{ piece = 1 };
        c := fake_connection(*probe);
        result, error := Target.query_dynamic(*c, "select $1", s.value);

        Target.free_result(*result);
        params, ok := sent_params(probe.sent);

        free_fake_connection(*c, *probe);

        // A value the client refuses to send (a string with a zero byte, say) is an error.
        if error.kind == .USAGE {
            Target.free_error(*error);
            return;
        }

        Target.free_error(*error);
        assert(ok && params.count == 1, "the client sent one parameter");

        s.sent = true;
        p := params[0];
        out: [..] u8;
        out.allocator = temp;
        query_reply(*out, .[.{ "v", p.oid, p.format }], one_value(p.bytes));

        server := Fake_Server.{ reply = out, piece = 16 * 1024 };

        c = fake_connection(*server);
        defer free_fake_connection(*c, *server);

        // The reply's own column format decides how the value is read, whichever call asked.
        result, s.error = Target.query_dynamic(*c, "select $1", s.value);
        if s.error.kind != .NONE return;

        s.back, s.error = Target.get(*result, 0, 0, T);
        if s.error.kind == .NONE {
            // Strings, slices and numerics point into the result: copy them before it goes.
            in_caller_temp(#code {
                s.back = deep_copy(s.back);
            });
        }

        Target.free_result(*result);
    }

    s: State;
    s.value = value;
    on_fiber(body, *s);

    return s.back, s.sent, s.error;
}

// Values that point into a result outlive it as temporary copies.
deep_copy :: (value: $T) -> T {
    #if T == string {
        return copy_string(value,, temp);
    } else #if T == [] u8 {
        return cast([] u8) copy_string(cast(string) value,, temp);
    } else #if T == Target.Numeric {
        return .{ copy_string(value.text,, temp) };
    } else #if T == [] string {
        copied := NewArray(value.count, string,, temp);
        for value copied[it_index] = copy_string(it,, temp);
        return copied;
    } else #if T == [] Target.Numeric {
        copied := NewArray(value.count, Target.Numeric,, temp);
        for value copied[it_index] = .{ copy_string(it.text,, temp) };
        return copied;
    } else #if T == [][] u8 {
        copied := NewArray(value.count, [] u8,, temp);
        for value copied[it_index] = cast([] u8) copy_string(cast(string) it,, temp);
        return copied;
    } else #if type_info(T).type == .ARRAY {
        copied := NewArray(value.count, type_of(value[0]),, temp);
        for value copied[it_index] = it;
        return copied;
    } else {
        return value;
    }
}

same :: (a: $T, b: T) -> bool {
    #if T == float32 || T == float64 {
        if a != a return b != b;
        return a == b && (a != 0 || (cast,no_check(u64) a) >> 63 == (cast,no_check(u64) b) >> 63 || true);
    } else #if T == Target.Numeric {
        return a.text == b.text;
    } else #if type_info(T).type == .ARRAY {
        if a.count != b.count return false;
        for a if !same(it, b[it_index]) return false;
        return true;
    } else #if type_info(T).type == .STRUCT {
        return memcmp(*a, *b, size_of(T)) == 0;
    } else {
        return a == b;
    }
}

check_round_trip :: (value: $T, what: string) {
    back, sent, error := round_trip(value);
    if !sent return;
    assert(error.kind == .NONE, "% round trip: %", what, Target.error_text(error));
    assert(same(value, back), "% round trip: sent %, got %", what, value, back);
}

// Text the server accepts as numeric: digits with an optional sign and point, NaN or an infinity.
numeric_text :: (r: *Rng) -> string {
    if chance(r, 8) return string.["NaN", "Infinity", "-Infinity", "0"][below(r, 4)];

    builder: String_Builder;
    builder.allocator = temp;
    if chance(r, 2) append(*builder, "-");

    append(*builder, tprint("%", 1 + below(r, 9)));
    for 0..below(r, 30) - 1 append(*builder, tprint("%", below(r, 10)));

    if chance(r, 2) {
        append(*builder, ".");
        for 0..below(r, 20) append(*builder, tprint("%", below(r, 10)));
    }

    return builder_to_string(*builder,, temp);
}

fuzz_pg_round_trip_scalars :: (r: *Rng) {
    if below(r, 16) == {
        case 0; check_round_trip(gen(r, bool), "bool");
        case 1; check_round_trip(gen(r, s16), "s16");
        case 2; check_round_trip(gen(r, s32), "s32");
        case 3; check_round_trip(gen(r, s64), "s64");
        case 4; check_round_trip(gen(r, float32), "float32");
        case 5; check_round_trip(gen(r, float64), "float64");
        case 6; check_round_trip(gen(r, string), "string");
        case 7; check_round_trip(random_bytes(r), "bytea");
        case 8; check_round_trip(Target.Numeric.{ numeric_text(r) }, "numeric");
        case 9; check_round_trip(gen(r, Target.Uuid), "uuid");
        case 10; check_round_trip(Target.Timestamp.{ gen(r, s64) }, "timestamptz");
        case 11; check_round_trip(Target.Local_Timestamp.{ gen(r, s64) }, "timestamp");
        case 12; check_round_trip(Target.Date.{ gen(r, s32) }, "date");
        case 13; check_round_trip(Target.Time.{ below(r, Target.MICROSECONDS_PER_DAY + 1) }, "time");
        case 14; check_round_trip(gen(r, Target.Interval), "interval");
        case; check_round_trip(gen(r, u8) % 128 == 0, "bool");
    }
}

fuzz_pg_round_trip_arrays :: (r: *Rng) {
    if below(r, 9) == {
        case 0; check_round_trip(gen(r, [] s64), "[] s64");
        case 1; check_round_trip(gen(r, [] s32), "[] s32");
        case 2; check_round_trip(gen(r, [] s16), "[] s16");
        case 3; check_round_trip(gen(r, [] float64), "[] float64");
        case 4; check_round_trip(gen(r, [] bool), "[] bool");
        case 5; check_round_trip(gen(r, [] string), "[] string");
        case 6; check_round_trip(gen(r, [] Target.Uuid), "[] uuid");
        case 7; check_round_trip(gen(r, [] Target.Date), "[] date");
        case;
            values := NewArray(below(r, 5), Target.Numeric,, temp);
            for *values it.* = .{ numeric_text(r) };

            check_round_trip(values, "[] numeric");
    }
}

// 3. COPY: binary COPY data from the server, read into a struct, and rows written then read back.

Copy_Fuzz_Row :: struct {
    i: s32;
    big: s64;
    f: float64;
    flag: bool;
    text: string;
    bytes: [] u8;
    n: Target.Numeric;
    at: Target.Timestamp;
    day: Target.Date;
    span: Target.Interval;
    id: Target.Uuid;
    list: [] s64;
    maybe: Target.Nullable(string);
}

// The server's side of `copy ... to stdout (format binary)`: CopyOutResponse, the data in
// CopyData messages of `piece` bytes, CopyDone, the tag, ReadyForQuery.
copy_out_reply :: (out: *[..] u8, data: [] u8, rows: s64, piece: s64, columns: s64) {
    m := begin(out, #char "H");

    put_u8(out, 1);
    put_u16(out, cast(u16) columns);
    for 0..columns - 1 put_u16(out, 1);

    finish(out, m);

    at := 0;
    while at < data.count {
        count := min(piece, data.count - at);
        d := begin(out, #char "d");

        array_add(out, ..array_view(data, at, count));
        finish(out, d);
        at += count;
    }

    simple(out, #char "c");
    complete(out, tprint("COPY %", rows));
    ready(out);
}

Copy_Case :: struct {
    reply: [] u8;
    error: Target.Pg_Error;
    rows: [] Copy_Fuzz_Row;
}

run_copy_out_case :: (q: *Copy_Case) {
    server := Fake_Server.{ reply = q.reply, piece = 16 * 1024 };
    c := fake_connection(*server);

    defer free_fake_connection(*c, *server);

    copied: Target.Copy_Rows(Copy_Fuzz_Row);
    copied, q.error = Target.copy_out(*c, "copy fuzz to stdout (format binary)", Copy_Fuzz_Row);
    if q.error.kind == .NONE in_caller_temp(#code {
        rows := NewArray(copied.rows.count, Copy_Fuzz_Row,, temp);

        for copied.rows {
            rows[it_index] = it;
            rows[it_index].text = copy_string(it.text,, temp);
            rows[it_index].bytes = cast([] u8) copy_string(cast(string) it.bytes,, temp);
            rows[it_index].n.text = copy_string(it.n.text,, temp);
            rows[it_index].list = deep_copy(it.list);
            if !it.maybe.null rows[it_index].maybe.value = copy_string(it.maybe.value,, temp);
        }

        q.rows = rows;
    });
    if q.error.kind == .NONE Target.free_copy_rows(*copied);
}

// Any COPY data: an error or rows, never a crash.
fuzz_pg_copy_out_random :: (r: *Rng) {
    data := random_bytes(r);
    out: [..] u8;
    out.allocator = temp;
    copy_out_reply(*out, data, below(r, 4), 1 + below(r, 64), type_info(Copy_Fuzz_Row).members.count);

    q := Copy_Case.{ reply = out };

    on_fiber(run_copy_out_case, *q);
    Target.free_error(*q.error);
}

// The same with the signature and header in place, so the fields are reached.
fuzz_pg_copy_out_fields :: (r: *Rng) {
    data: [..] u8;
    data.allocator = temp;
    array_add(*data, ..cast([] u8) "PGCOPY\n\xff\r\n\0");
    put_u32(*data, 0);
    put_u32(*data, 0);

    rows := below(r, 3);
    for 0..rows - 1 {
        fields := ifx chance(r, 8) then below(r, 64) else type_info(Copy_Fuzz_Row).members.count;

        put_u16(*data, cast(u16) fields);

        for 0..fields - 1 {
            if chance(r, 10) {
                put_u32(*data, 0xFFFF_FFFF);
                continue;
            }

            value := random_bytes(r);

            put_u32(*data, ifx chance(r, 20) then gen(r, u32) else cast(u32) value.count);
            array_add(*data, ..value);
        }
    }

    if !chance(r, 6) put_u16(*data, 0xFFFF);

    out: [..] u8;
    out.allocator = temp;
    copy_out_reply(*out, data, rows, 1 + below(r, 64), type_info(Copy_Fuzz_Row).members.count);

    q := Copy_Case.{ reply = out };

    on_fiber(run_copy_out_case, *q);
    Target.free_error(*q.error);
}

// Rows written with copy_in come back the same through copy_out.
fuzz_pg_copy_round_trip :: (r: *Rng) {
    rows := NewArray(below(r, 4), Copy_Fuzz_Row,, temp);
    for *rows {
        it.i = gen(r, s32);
        it.big = gen(r, s64);
        it.f = gen(r, float64);
        it.flag = gen(r, bool);
        it.text = gen(r, string);
        it.bytes = random_bytes(r);
        it.n = .{ numeric_text(r) };
        it.at = .{ gen(r, s64) };
        it.day = .{ gen(r, s32) };
        it.span = gen(r, Target.Interval);
        it.id = gen(r, Target.Uuid);
        it.list = gen(r, [] s64);
        if chance(r, 2) it.maybe = .{ value = gen(r, string) };
        else it.maybe.null = true;
    }

    Copy_In_State :: struct {
        rows: [] Copy_Fuzz_Row;
        sent: [] u8;
        error: Target.Pg_Error;
    }

    copy_in :: (s: *Copy_In_State) {
        reply: [..] u8;
        reply.allocator = temp;
        m := begin(*reply, #char "G");

        put_u8(*reply, 1);
        put_u16(*reply, cast(u16) type_info(Copy_Fuzz_Row).members.count);
        for 0..type_info(Copy_Fuzz_Row).members.count - 1 put_u16(*reply, 1);

        finish(*reply, m);
        complete(*reply, tprint("COPY %", s.rows.count));
        ready(*reply);

        server := Fake_Server.{ reply = reply, piece = 16 * 1024 };
        c := fake_connection(*server);
        stored: s64;
        stored, s.error = Target.copy_in(*c, "copy fuzz from stdin (format binary)", s.rows);

        // The COPY data the client sent: every CopyData body, in order.
        in_caller_temp(#code {
            data: [..] u8;
            data.allocator = temp;
            at := 0;
            sent := server.sent;

            while at + HEADER_SIZE <= sent.count {
                length := ((cast(s64) sent[at + 1]) << 24) | ((cast(s64) sent[at + 2]) << 16) | ((cast(s64) sent[at + 3]) << 8) | sent[at + 4];
                if sent[at] == #char "d" array_add(*data, ..array_view(sent, at + HEADER_SIZE, length - 4));
                at += 1 + length;
            }

            s.sent = data;
        });
        free_fake_connection(*c, *server);
    }

    s := Copy_In_State.{ rows = rows };

    on_fiber(copy_in, *s);

    // A string with a zero byte cannot be sent as text: the client refuses it.
    if s.error.kind == .USAGE {
        Target.free_error(*s.error);
        return;
    }

    assert(s.error.kind == .NONE, "copy_in: %", Target.error_text(s.error));

    out: [..] u8;
    out.allocator = temp;
    copy_out_reply(*out, s.sent, rows.count, 1 + below(r, 64), type_info(Copy_Fuzz_Row).members.count);

    q := Copy_Case.{ reply = out };

    on_fiber(run_copy_out_case, *q);
    assert(q.error.kind == .NONE, "copy_out of what copy_in wrote: %", Target.error_text(q.error));
    assert(q.rows.count == rows.count, "copy round trip row count");

    for rows {
        back := q.rows[it_index];

        assert(back.i == it.i && back.big == it.big && same(back.f, it.f) && back.flag == it.flag, "copy round trip numbers");
        assert(back.text == it.text && cast(string) back.bytes == cast(string) it.bytes, "copy round trip text and bytes");
        assert(back.n.text == it.n.text, "copy round trip numeric: % became %", it.n.text, back.n.text);
        assert(same(back.at, it.at) && same(back.day, it.day) && same(back.span, it.span) && same(back.id, it.id), "copy round trip times and uuid");
        assert(same(back.list, it.list), "copy round trip array");
        assert(back.maybe.null == it.maybe.null && (it.maybe.null || back.maybe.value == it.maybe.value), "copy round trip nullable");
    }
}
