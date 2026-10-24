// The client against a scripted server on a local socket: no PostgreSQL needed. After a normal
// startup, each case answers one query with a reply that is valid, cut short at every byte,
// interleaved with unsolicited messages, or mutated from a seed. The client must give the right
// values for a valid reply and an error (never a crash or a hang) for a broken one. Every column
// type is decoded, so the mutations reach the binary codecs.
#import "Basic";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";

MUTATIONS :: 400;

// What the server sends for the next connection, after the startup.
reply: [..] u8;
port: u16;
notices := 0;

main :: () {
    Io.io_run(() {
        listener, error := Io.tcp_listen("127.0.0.1:0");

        assert(error == .NONE, "listen");

        address := Io.local_address(listener);

        port = address.port;

        Io.spawn(serve, listener);

        valid := valid_reply();

        // The valid reply decodes to the values it holds.
        set_reply(valid);
        result, query_error := run_query();

        assert(query_error.kind == .NONE, Pg.error_text(query_error));
        check_values(*result);
        Pg.free_result(*result);

        // Unsolicited messages between the others change nothing.
        noisy: [..] u8;
        noisy.allocator = temp;
        unsolicited(*noisy);
        array_add(*noisy, ..valid);
        set_reply(noisy);
        result, query_error = run_query();
        assert(query_error.kind == .NONE, Pg.error_text(query_error));
        assert(notices == 1, "the notice should reach on_notice");
        check_values(*result);
        Pg.free_result(*result);

        // Cut short at every byte, then the connection closes: an error each time.
        for cut: 0..valid.count - 1 {
            set_reply(array_view(valid, 0, cut));
            result, query_error = run_query();
            assert(query_error.kind != .NONE, tprint("a reply cut at % of % bytes was accepted", cut, valid.count));
            Pg.free_result(*result);
        }

        // Mutated replies: any outcome but a crash or a hang.
        state: u64 = 0x5EED;
        failed := 0;

        for 1..MUTATIONS {
            mutated: [..] u8;
            mutated.allocator = temp;
            array_add(*mutated, ..valid);
            mutate(*state, *mutated);
            set_reply(mutated);

            result, query_error = run_query();
            if query_error.kind != .NONE failed += 1;
            else decode_all(*result);

            Pg.free_result(*result);
            reset_temporary_storage();
        }

        assert(failed > 0, "no mutation was rejected");

        Io.close(listener);
        print("ok\n");
    });
}

set_reply :: (bytes: [] u8) {
    array_reset_keeping_memory(*reply);
    array_add(*reply, ..bytes);
}

run_query :: () -> Pg.Result, Pg.Pg_Error {
    config, config_error := Pg.parse_url(tprint("postgres://fake@127.0.0.1:%/fake?sslmode=disable&connect_timeout=5", port));

    assert(config_error.kind == .NONE, Pg.error_text(config_error));
    defer Pg.free_config(*config);

    config.on_notice = (c: *Pg.Connection, notice: *Pg.Pg_Error) {
        notices += 1;
    };

    db, error := Pg.connect(config);
    if error.kind != .NONE return .{}, error;

    defer Pg.close(db, now = true);

    result: Pg.Result;

    // A reply that leaves the client waiting would hang: the server closes, so it must not.
    Io.with_deadline(5_000, #code {
        result, error = Pg.query_binary(db, "select the columns");
    });

    assert(error.kind != .TIMED_OUT, "the client hung on a broken reply");

    return result, error;
}

COLUMNS :: Column.[
    .{ "i8", 20 },
    .{ "t", 25 },
    .{ "b", 16 },
    .{ "f8", 701 },
    .{ "n", 1700 },
    .{ "ts", 1184 },
    .{ "u", 2950 },
    .{ "by", 17 },
    .{ "a", 1007 }, // int4[]
    .{ "j", 3802 },
    .{ "nul", 25 },
];

Column :: struct {
    name: string;
    oid: u32;
}

check_values :: (result: *Pg.Result) {
    assert(result.row_count == 1, "one row");

    i8, e1 := Pg.get(result, 0, 0, s64);
    t, e2 := Pg.get(result, 0, 1, string);
    b, e3 := Pg.get(result, 0, 2, bool);
    f8, e4 := Pg.get(result, 0, 3, float64);
    n, e5 := Pg.get(result, 0, 4, Pg.Numeric);
    ts, e6 := Pg.get(result, 0, 5, Pg.Timestamp);
    u, e7 := Pg.get(result, 0, 6, Pg.Uuid);
    by, e8 := Pg.get(result, 0, 7, [] u8);
    a, e9 := Pg.get(result, 0, 8, [] s32);
    j, e10 := Pg.get(result, 0, 9, Pg.Json_Raw);
    nul, e11 := Pg.get(result, 0, 10, Pg.Nullable(string));

    for Pg.Pg_Error.[e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11] assert(it.kind == .NONE, tprint("column %: %", it_index, Pg.error_text(it)));

    assert(i8 == -42 && t == "héllo" && b && f8 == 2.5, "scalars");
    assert(n.text == "12.5", n.text);
    assert(Pg.unix_microseconds(ts) == 1_000_000, "timestamp");
    assert(Pg.uuid_text(u) == "00112233-4455-6677-8899-aabbccddeeff", Pg.uuid_text(u));
    assert(by.count == 3 && by[0] == 0 && by[2] == 0xFF, "bytea");
    assert(a.count == 2 && a[0] == 7 && a[1] == -1, "int4[]");
    assert(j.text == "{\"k\": 1}", j.text);
    assert(nul.null, "null");
}

// Every column read as its type, whatever the bytes became.
decode_all :: (result: *Pg.Result) {
    for row: 0..result.row_count - 1 {
        Pg.get(result, row, 0, s64);
        Pg.get(result, row, 1, string);
        Pg.get(result, row, 2, bool);
        Pg.get(result, row, 3, float64);
        Pg.get(result, row, 4, Pg.Numeric);
        Pg.get(result, row, 5, Pg.Timestamp);
        Pg.get(result, row, 6, Pg.Uuid);
        Pg.get(result, row, 7, [] u8);
        Pg.get(result, row, 8, [] s32);
        Pg.get(result, row, 9, Pg.Json_Raw);
        Pg.get(result, row, 10, Pg.Nullable(string));
    }
}

// ParseComplete, BindComplete, RowDescription, one DataRow, CommandComplete, ReadyForQuery.
valid_reply :: () -> [] u8 {
    out: [..] u8;

    message(*out, #char "1", .[]);
    message(*out, #char "2", .[]);

    body: [..] u8;
    body.allocator = temp;
    put16(*body, COLUMNS.count);

    for COLUMNS {
        put_cstring(*body, it.name);
        put32(*body, 0); // table
        put16(*body, 0); // column
        put32(*body, it.oid);
        put16(*body, -1); // size
        put32(*body, -1); // modifier
        put16(*body, 1); // binary
    }

    message(*out, #char "T", body);

    array_reset(*body);
    put16(*body, COLUMNS.count);

    value(*body, -42);
    value_bytes(*body, cast([] u8) "héllo");
    value_bytes(*body, .[1]);
    half: float64 = 2.5;

    value(*body, <<cast(*s64) *half);

    numeric: [..] u8;
    numeric.allocator = temp;
    for s16.[2, 0, 0, 1, 12, 5000] put16(*numeric, it); // 2 digits, weight 0, positive, scale 1: 12.5

    value_bytes(*body, numeric);

    value(*body, 1_000_000 - 946_684_800_000_000); // microseconds since 2000-01-01
    value_bytes(*body, .[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    value_bytes(*body, .[0, 1, 0xFF]);

    array_value: [..] u8;
    array_value.allocator = temp;
    for s32.[1, 0, 23, 2, 1] put32(*array_value, it); // one dimension, no nulls, int4, 2 items from 1

    for s32.[7, -1] {
        put32(*array_value, 4);
        put32(*array_value, it);
    }

    value_bytes(*body, array_value);

    jsonb: [..] u8;
    jsonb.allocator = temp;
    array_add(*jsonb, 1); // jsonb version
    array_add(*jsonb, ..cast([] u8) "{\"k\": 1}");
    value_bytes(*body, jsonb);

    put32(*body, -1); // null

    message(*out, #char "D", body);

    array_reset(*body);
    put_cstring(*body, "SELECT 1");
    message(*out, #char "C", body);
    message(*out, #char "Z", .[#char "I"]);

    return out;
}

// A notice, a notification and a parameter change.
unsolicited :: (out: *[..] u8) {
    body: [..] u8;
    body.allocator = temp;

    for string.["SNOTICE", "VNOTICE", "C00000", "Mjust saying"] put_cstring(*body, it);

    array_add(*body, 0);
    message(out, #char "N", body);

    array_reset(*body);
    put32(*body, 1234);
    put_cstring(*body, "channel");
    put_cstring(*body, "payload");
    message(out, #char "A", body);

    array_reset(*body);
    put_cstring(*body, "application_name");
    put_cstring(*body, "other");
    message(out, #char "S", body);
}

// One to four changes: a flipped bit, a random byte, a length or count set to an edge value, a
// cut, or a duplicated run.
mutate :: (state: *u64, bytes: *[..] u8) {
    for 1..1 + below(state, 4) {
        if bytes.count == 0 return;

        at := below(state, bytes.count);

        if below(state, 6) == {
            case 0; bytes.*[at] ^= cast(u8) (1 << below(state, 8));
            case 1; bytes.*[at] = cast(u8) below(state, 256);
            case 2;
                edges :: u32.[0, 1, 4, 0x7FFF_FFFF, 0x8000_0000, 0xFFFF_FFFF, 0xFFFF];
                edge := edges[below(state, edges.count)];

                for 0..3 if at + it < bytes.count bytes.*[at + it] = cast,trunc(u8) (edge >> (8 * (3 - it)));
            case 3; bytes.count = at;
            case 4;
                run := min(1 + below(state, 16), bytes.count - at);
                copy := array_copy(array_view(bytes.*, at, run),, temp);

                for copy array_insert_at(bytes, it, at + it_index);
            case 5; array_ordered_remove_by_index(bytes, at);
        }
    }
}

below :: (state: *u64, n: s64) -> s64 {
    x := state.*;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    state.* = x;

    return cast(s64) ((x * 0x2545F4914F6CDD1D) % cast(u64) n);
}

// One connection at a time: answer the startup, wait for the query's Sync, send the reply, close.
serve :: (listener: Io.Tcp_Listener) {
    while true {
        stream, error := Io.accept(listener);
        if error != .NONE return;

        answer(stream);
        Io.close(stream, now = true);
    }
}

answer :: (stream: Io.Tcp_Stream) {
    // The startup message: a length, then the rest.
    header: [4] u8;
    if !read_exactly(stream, header) return;

    rest := NewArray(read32(header) - 4, u8,, temp);
    if !read_exactly(stream, rest) return;

    startup: [..] u8;
    startup.allocator = temp;
    message(*startup, #char "R", .[0, 0, 0, 0]); // AuthenticationOk

    body: [..] u8;
    body.allocator = temp;

    for string.["server_version", "17.0", "client_encoding", "UTF8", "standard_conforming_strings", "on", "integer_datetimes", "on", "DateStyle", "ISO, MDY"] put_cstring(*body, it);

    // One ParameterStatus per pair.
    at := 0;
    pair := 0;

    while at < body.count {
        name_end := at;
        while body[name_end] != 0 name_end += 1;

        value_end := name_end + 1;
        while body[value_end] != 0 value_end += 1;

        message(*startup, #char "S", array_view(body, at, value_end + 1 - at));
        at = value_end + 1;
        pair += 1;
    }

    message(*startup, #char "K", .[0, 0, 0, 1, 0, 0, 0, 2]);
    message(*startup, #char "Z", .[#char "I"]);

    if !write_all(stream, startup) return;

    // Read messages up to the Sync.
    while true {
        type: [1] u8;
        if !read_exactly(stream, type) return;

        length: [4] u8;
        if !read_exactly(stream, length) return;

        skipped := NewArray(read32(length) - 4, u8,, temp);
        if !read_exactly(stream, skipped) return;

        if type[0] == #char "S" break;
        if type[0] == #char "X" return;
    }

    write_all(stream, reply);
}

read_exactly :: (stream: Io.Tcp_Stream, buffer: [] u8) -> bool {
    count, error := Io.read_full(stream, buffer);
    return error == .NONE && count == buffer.count;
}

write_all :: (stream: Io.Tcp_Stream, bytes: [] u8) -> bool {
    if !bytes.count return true;

    count, error := Io.write(stream, bytes);
    return error == .NONE && count == bytes.count;
}

message :: (out: *[..] u8, type: u8, body: [] u8) {
    array_add(out, type);
    put32(out, cast(s32) (body.count + 4));
    array_add(out, ..body);
}

value :: (out: *[..] u8, v: s64) {
    put32(out, 8);
    for 0..7 array_add(out, cast,trunc(u8) (v >> (8 * (7 - it))));
}

value_bytes :: (out: *[..] u8, bytes: [] u8) {
    put32(out, cast(s32) bytes.count);
    array_add(out, ..bytes);
}

put16 :: (out: *[..] u8, v: s64) {
    array_add(out, cast,trunc(u8) (v >> 8), cast,trunc(u8) v);
}

put32 :: (out: *[..] u8, v: s64) {
    for 0..3 array_add(out, cast,trunc(u8) (v >> (8 * (3 - it))));
}

put_cstring :: (out: *[..] u8, text: string) {
    array_add(out, ..cast([] u8) text);
    array_add(out, 0);
}

read32 :: (bytes: [] u8) -> s64 {
    return (cast(s64) bytes[0] << 24) | (cast(s64) bytes[1] << 16) | (cast(s64) bytes[2] << 8) | bytes[3];
}
