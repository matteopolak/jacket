// test: skip_on(.WASM, reason = "needs Extensions/Io fibers, which wasm cannot run (no stack switching)")
// What each kind of query argument and COPY member sends, checked against a live server started by
// tests/start_server.sh; skipped when JAI_PG_TEST_PORT is not set. Every value goes in and comes
// back as the server's own text.
#import "Basic";
#import "String";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";
#import "Extensions/Json";
#import "Extensions/Environment";

Mood :: enum {
    sad;
    ok;
    happy;
}

Theme :: struct {
    theme: string;
}

// One column per way a COPY member can be sent: arrays of several element types, fixed and growing,
// with and without NULL elements, and json as `jsonb`, as `json` and as a typed value.
Copy_Shapes :: struct {
    ints: [] s64;
    small: [..] s16;
    fixed: [3] s32;
    names: [] string;
    maybe: [] Pg.Nullable(string);
    reals: [] float64;
    flags: [] bool;
    raw: [] u8;
    blobs: [][] u8;
    ids: [] Pg.Uuid;
    prices: [] Pg.Numeric;
    spans: [] Pg.Interval;
    empty: [] s64;
    doc: Pg.Json(Theme);
    plain: Pg.Json_Raw; @pg_json
    note: Pg.Nullable(Pg.Json_Raw);
    after: Pg.Nullable([] s32);
}

main :: () {
    port_text := environment_variable("JAI_PG_TEST_PORT");

    if !port_text {
        print("skipped: set JAI_PG_TEST_PORT (see tests/start_server.sh)\n");
        return;
    }

    value, valid := string_to_int(port_text);

    assert(valid, "bad JAI_PG_TEST_PORT");
    port = cast(u16) value;

    Io.io_run(() {
        config: Pg.Config;
        config.host = "127.0.0.1";
        config.port = port;
        config.user = "postgres";
        config.database = "postgres";
        config.ssl_mode = .DISABLE;

        db, error := Pg.connect(config);

        assert(error.kind == .NONE, "%", Pg.error_text(error));
        defer Pg.close(db);

        // Timestamps come back in the session's zone.
        error = Pg.execute_script(db, "set timezone = 'UTC'");
        assert(error.kind == .NONE, "%", Pg.error_text(error));

        test_scalars(db);
        test_arrays(db);
        test_refused(db);
        test_copy(db);
        print("ok\n");
    });
}

port: u16;

// The text of the only value a query returns.
text_of :: (db: *Pg.Connection, sql: string, args: [] Any) -> string {
    result, error := Pg.query_dynamic(db, sql, ..args);

    defer Pg.free_result(*result);
    assert(error.kind == .NONE, "%: %", sql, Pg.error_text(error));

    got, decode_error := Pg.get(*result, 0, 0, string);

    assert(decode_error.kind == .NONE, "%: %", sql, Pg.error_text(decode_error));

    return copy_string(got,, temp);
}

expect :: (db: *Pg.Connection, sql: string, wanted: string, args: ..Any) {
    got := text_of(db, sql, args);

    assert(got == wanted, "% gave % instead of %", sql, got, wanted);
}

test_scalars :: (db: *Pg.Connection) {
    a: s8 = -5;
    b: u8 = 250;
    c: s16 = -300;
    d: u16 = 65535;
    e: s32 = -2_000_000_000;
    f: u32 = 4_000_000_000;
    g: s64 = 1 << 40;
    h: u64 = 18_000_000_000_000_000_000;
    i: float32 = 1.5;
    j: float64 = -2.25;

    expect(db, "select $1::text || ',' || $2::text || ',' || $3::text || ',' || $4::text", "-5,250,-300,65535", a, b, c, d);
    expect(db, "select $1::text || ',' || $2::text || ',' || $3::text", "-2000000000,4000000000,1099511627776", e, f, g);
    expect(db, "select $1::numeric::text", "18000000000000000000", h);
    expect(db, "select $1::float8::text || ',' || $2::float8::text", "1.5,-2.25", i, j);
    expect(db, "select $1::text || $2::text", "tf", true, false);
    expect(db, "select $1::text", "café", "café");

    uuid, uuid_ok := Pg.parse_uuid("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11");

    assert(uuid_ok);
    expect(db, "select $1::uuid::text", "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11", uuid);
    expect(db, "select $1::timestamptz::text", "2030-01-02 03:04:05.000006+00", Pg.timestamp_from_civil(2030, 1, 2, 3, 4, 5, 6));
    expect(db, "select $1::date::text", "1999-12-31", Pg.date_from_civil(1999, 12, 31));
    expect(db, "select $1::interval::text", "1 mon 2 days 00:00:00.000003", Pg.Interval.{ months = 1, days = 2, microseconds = 3 });
    expect(db, "select ($1::numeric * 2)::text", "3.0", Pg.Numeric.{ "1.5" });

    bytes: [3] u8 = .[1, 0, 255];

    expect(db, "select $1::bytea::text", "\\x0100ff", bytes);
    expect(db, "select $1::timestamp::text || ',' || $2::time::text", "2000-01-02 00:00:00,01:00:00", Pg.Local_Timestamp.{ 86_400_000_000 }, Pg.Time.{ 3_600_000_000 });

    // A NULL, a value in a Nullable, and enums and json, which are sent with no type of their own.
    nothing: Pg.Nullable(s64);
    nothing.null = true;

    expect(db, "select ($1::int8 is null)::text || ($2::int8 + 1)::text", "true42", nothing, Pg.some(41));
    expect(db, "select $1::text", "happy", Mood.happy);
    expect(db, "select $1::jsonb->>'theme'", "solar", Pg.json(Theme.{ "solar" }));
    expect(db, "select $1::jsonb::text", "[1, 2]", Pg.Json_Raw.{ "[1, 2]" });
}

test_arrays :: (db: *Pg.Connection) {
    ints: [] s64 = .[3, 1, 2];
    small: [..] s16;

    array_add(*small, -4);
    array_add(*small, 9);
    fixed: [3] s32 = .[7, 8, 9];
    unsigned: [] u32 = .[4_000_000_000, 1];
    bytes: [] u8 = .[0, 1];
    names: [] string = .["a b", "c\"d", "NULL"];
    maybe: [] Pg.Nullable(string) = .[Pg.some("x"), .{ null = true }];
    reals: [] float64 = .[1.5, -0.25];
    singles: [] float32 = .[0.5];
    flags: [] bool = .[true, false];
    wide: [] u64 = .[5];
    uuid, uuid_ok := Pg.parse_uuid("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11");

    assert(uuid_ok);
    ids: [] Pg.Uuid = .[uuid];
    prices: [] Pg.Numeric = .[.{ "1.50" }, .{ "-2" }];
    spans: [] Pg.Interval = .[.{ months = 1, days = 2, microseconds = 3 }];
    stamps: [] Pg.Timestamp = .[Pg.timestamp_from_civil(2030, 1, 2, 3, 4, 5, 6)];
    locals: [] Pg.Local_Timestamp = .[.{ 0 }];
    dates: [] Pg.Date = .[Pg.date_from_civil(1999, 12, 31)];
    times: [] Pg.Time = .[.{ 3_600_000_000 }];
    blobs: [][] u8 = .[bytes, bytes];
    empty: [] s64;
    absent: [] Pg.Nullable(s32) = .[.{ null = true }, .{ null = true }];
    moods: [] Mood = .[.happy, .sad];
    docs: [] Pg.Json(Theme) = .[Pg.json(Theme.{ "a" }), Pg.json(Theme.{ "b" })];
    raws: [] Pg.Json_Raw = .[.{ "{\"k\": 1}" }];

    expect(db, "select $1::text", "{3,1,2}", ints);
    expect(db, "select $1::text", "{-4,9}", small);
    expect(db, "select $1::text", "{7,8,9}", fixed);
    expect(db, "select $1::text", "{4000000000,1}", unsigned);
    expect(db, "select $1::text", "{\"a b\",\"c\\\"d\",\"NULL\"}", names);
    expect(db, "select $1::text", "{x,NULL}", maybe);
    expect(db, "select $1::text", "{1.5,-0.25}", reals);
    expect(db, "select $1::text", "{0.5}", singles);
    expect(db, "select $1::text", "{t,f}", flags);
    expect(db, "select $1::text", "\\x0001", bytes);
    expect(db, "select $1::text", "{5}", wide);
    expect(db, "select $1::text", "{a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11}", ids);
    expect(db, "select $1::text", "{1.50,-2}", prices);
    expect(db, "select $1::text", "{\"1 mon 2 days 00:00:00.000003\"}", spans);
    expect(db, "select $1::text", "{\"2030-01-02 03:04:05.000006+00\"}", stamps);
    expect(db, "select $1::text", "{\"2000-01-01 00:00:00\"}", locals);
    expect(db, "select $1::text", "{1999-12-31}", dates);
    expect(db, "select $1::text", "{01:00:00}", times);
    expect(db, "select $1::text", "{\"\\\\x0001\",\"\\\\x0001\"}", blobs);
    expect(db, "select $1::int8[]::text", "{}", empty);
    expect(db, "select $1::int4[]::text", "{NULL,NULL}", absent);
    expect(db, "select $1::text", "{\"happy\",\"sad\"}", moods);
    expect(db, "select $1::jsonb[]::text", "{\"{\\\"theme\\\": \\\"a\\\"}\",\"{\\\"theme\\\": \\\"b\\\"}\"}", docs);
    expect(db, "select $1::jsonb[]::text", "{\"{\\\"k\\\": 1}\"}", raws);
    expect(db, "select array_length($1::int8[], 1)::text", "3", ints);
    expect(db, "select ($1::int2[])[1]::text", "-4", Pg.some(small));
}

// Arguments that cannot be sent say why and leave the connection usable.
test_refused :: (db: *Pg.Connection) {
    huge: [] u64 = .[18_000_000_000_000_000_000];
    nested: [][] s32 = .[.[1], .[2]];
    stray_mood := cast(Mood) 9;
    stray: [] Mood = .[cast(Mood) 9];
    zero: [] string = .["a\0b"];

    refuse(db, "select $1::text", "does not fit in int8", huge);
    refuse(db, "select $1::text", "multi-dimensional", nested);
    refuse(db, "select $1::text", "none of its members", stray_mood);
    refuse(db, "select $1::text", "none of its enum's members", stray);
    refuse(db, "select $1::text", "zero byte", "a\0b");
    refuse(db, "select $1::text", "zero byte", zero);
    expect(db, "select 1::text", "1");
}

refuse :: (db: *Pg.Connection, sql: string, fragment: string, args: ..Any) {
    result, error := Pg.query_dynamic(db, sql, ..args);

    Pg.free_result(*result);
    assert(error.kind == .USAGE && contains(error.message, fragment), "% gave % instead of a usage error naming %", sql, Pg.error_text(error), fragment);
    Pg.free_error(*error);
}

test_copy :: (db: *Pg.Connection) {
    error := Pg.execute_script(db, #string SQL
create temp table shapes (ints int8[], small int2[], fixed int4[], names text[], maybe text[], reals float8[], flags bool[], raw bytea, blobs bytea[], ids uuid[], prices numeric[], spans interval[], empty int8[], doc jsonb, plain json, note jsonb, after int4[]);
SQL
    );

    assert(error.kind == .NONE, "%", Pg.error_text(error));

    uuid, uuid_ok := Pg.parse_uuid("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11");

    assert(uuid_ok);

    row: Copy_Shapes;
    row.ints = .[3, 1, 2];
    array_add(*row.small, -4);
    array_add(*row.small, 9);
    row.fixed = .[7, 8, 9];
    row.names = .["a b", "c\"d"];
    row.maybe = .[Pg.some("x"), .{ null = true }];
    row.reals = .[1.5, -0.25];
    row.flags = .[true, false];
    row.raw = .[1, 0, 255];
    row.blobs = .[.[0, 1], .[]];
    row.ids = .[uuid];
    row.prices = .[.{ "1.50" }];
    row.spans = .[.{ months = 1 }];
    row.doc = Pg.json(Theme.{ "solar" });
    row.plain = .{ "{\"k\":  1}" };
    row.note.null = true;
    after: [] s32 = .[5, 6];

    row.after = Pg.some(after);

    rows: [1] Copy_Shapes;
    rows[0] = row;

    stored, copy_error := Pg.copy_in(db, "copy shapes from stdin (format binary)", rows);

    assert(copy_error.kind == .NONE, "%", Pg.error_text(copy_error));
    assert(stored == 1);

    expect(db, "select ints::text || small::text || fixed::text || names::text from shapes", "{3,1,2}{-4,9}{7,8,9}{\"a b\",\"c\\\"d\"}");
    expect(db, "select maybe::text || reals::text || flags::text || raw::text from shapes", "{x,NULL}{1.5,-0.25}{t,f}\\x0100ff");
    expect(db, "select blobs::text || ids::text || prices::text || spans::text || empty::text from shapes", "{\"\\\\x0001\",\"\\\\x\"}{a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11}{1.50}{\"1 mon\"}{}");
    expect(db, "select doc::text || plain::text || (note is null)::text || after::text from shapes", "{\"theme\": \"solar\"}{\"k\":  1}true{5,6}");
}
