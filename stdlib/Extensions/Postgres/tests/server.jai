// Runs against a live server started by tests/start_server.sh; skipped when JAI_PG_TEST_PORT is
// not set.
#import "Basic";
#import "String";
#import "File";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";
#import "Extensions/Json";

FIXTURES :: #run join(#filepath, "../../Tls/tests/fixtures/");

port: u16;
socket_dir: string;

main :: () {
    port_text := environment("JAI_PG_TEST_PORT");
    if !port_text {
        print("skipped: set JAI_PG_TEST_PORT (see tests/start_server.sh)\n");
        return;
    }

    value, valid := string_to_int(port_text);

    assert(valid, "bad JAI_PG_TEST_PORT");
    port = cast(u16) value;
    socket_dir = environment("JAI_PG_TEST_SOCKET_DIR");

    Io.io_run(() {
        step("test_authentication");
        test_authentication();
        step("test_tls");
        test_tls();
        step("test_unix_socket");
        test_unix_socket();
        step("test_values");
        test_values(.TEXT);
        step("test_values");
        test_values(.BINARY);
        step("test_arrays");
        test_arrays();
        step("test_json(.TEXT)");
        test_json(.TEXT);
        step("test_json(.BINARY)");
        test_json(.BINARY);
        step("test_parameters");
        test_parameters();
        step("test_errors");
        test_errors();
        step("test_scripts_and_status");
        test_scripts_and_status();
        step("test_run_script");
        test_run_script();
        step("test_startup_parameters");
        test_startup_parameters();
        step("test_notices_and_describe");
        test_notices_and_describe();
        step("test_deadline");
        test_deadline();
        step("test_pool");
        test_pool();
        step("test_transactions");
        test_transactions();
        step("test_pipeline");
        test_pipeline();
        step("test_pipeline_deadline");
        test_pipeline_deadline();
        step("test_notifications");
        test_notifications();
        step("test_protocol_versions");
        test_protocol_versions();
    });
    print("ok\n");
}

step :: (name: string) {
    if environment("JAI_PG_TEST_VERBOSE") print("%\n", name);
}

config_for :: (user: string, password := "") -> Pg.Config {
    config: Pg.Config;
    config.host = "127.0.0.1";
    config.port = port;
    config.user = user;
    config.password = password;
    config.database = "postgres";
    config.ssl_mode = .DISABLE;

    return config;
}

connect_ok :: (config: Pg.Config) -> *Pg.Connection {
    db, error := Pg.connect(config);
    if error.kind != .NONE {
        print("connect as % failed: %\n", config.user, Pg.error_text(error));
        assert(false);
    }

    return db;
}

connect_fails :: (config: Pg.Config, kind: Pg.Error_Kind, code := Pg.Sqlstate.SUCCESSFUL_COMPLETION) -> Pg.Pg_Error {
    db, error := Pg.connect(config);
    if db || error.kind != kind || (code != .SUCCESSFUL_COMPLETION && error.code != code) {
        print("connect as % gave % (%), wanted % %\n", config.user, error.kind, Pg.error_text(error), kind, code);
        assert(false);
    }

    return error;
}

scalar :: (db: *Pg.Connection, sql: string, $T: Type) -> T {
    result, error := Pg.query_dynamic(db, sql);

    defer Pg.free_result(*result);
    assert_ok(error, sql);
    value, decode_error := Pg.get(*result, 0, 0, T);

    assert_ok(decode_error, sql);
    #if T == string return copy_string(value,, temp);
    else return value;
}

assert_ok :: (error: Pg.Pg_Error, what: string, location := #caller_location) {
    if error.kind == .NONE return;
    print("%:%: %: %\n", location.fully_pathed_filename, location.line_number, what, Pg.error_text(error));
    assert(false);
}

test_authentication :: () {
    db := connect_ok(config_for("postgres"));

    assert(db.status == .IDLE);
    assert(db.server_version >= 150000);
    assert(Pg.parameter(db, "client_encoding") == "UTF8");
    assert(db.backend_pid > 0);
    Pg.close(db);

    db = connect_ok(config_for("jai_md5", "md5 secret"));
    assert(scalar(db, "select current_user", string) == "jai_md5");
    Pg.close(db);

    db = connect_ok(config_for("jai_scram", "scram secret"));
    assert(scalar(db, "select current_user", string) == "jai_scram");
    assert(!db.channel_binding_used);
    Pg.close(db);

    connect_fails(config_for("jai_scram", "wrong"), .SERVER, .INVALID_PASSWORD);
    connect_fails(config_for("jai_md5", "wrong"), .SERVER, .INVALID_PASSWORD);
    connect_fails(config_for("jai_scram"), .AUTH);

    // Cleartext passwords need TLS or an explicit opt-in.
    connect_fails(config_for("jai_clear", "clear secret"), .AUTH);
    config := config_for("jai_clear", "clear secret");
    config.allow_cleartext_password = true;
    db = connect_ok(config);
    Pg.close(db);

    // channel_binding=require refuses anything but SCRAM-SHA-256-PLUS.
    config = config_for("jai_scram", "scram secret");
    config.channel_binding = .REQUIRE;
    connect_fails(config, .AUTH);

    connect_fails(config_for("no_such_role"), .SERVER, .INVALID_AUTHORIZATION_SPECIFICATION);
}

test_tls :: () {
    ca, read := read_entire_file(tprint("%ca.pem", FIXTURES));

    assert(read);
    defer free(ca);

    config := config_for("jai_tls_scram", "tls secret");
    config.ssl_mode = .REQUIRE;
    db := connect_ok(config);

    assert(db.tls_used && db.channel_binding_used);
    assert(scalar(db, "select ssl from pg_stat_ssl where pid = pg_backend_pid()", bool));
    Pg.close(db);

    // The fixture certificate names localhost and chains to ca.pem.
    config.host = "localhost";
    config.ssl_mode = .VERIFY_FULL;
    config.ssl_root_cert_pem = ca;
    config.channel_binding = .REQUIRE;
    db = connect_ok(config);
    Pg.close(db);

    // Without the right root the certificate is refused.
    config.ssl_root_cert_pem = "";
    connect_fails(config, .TLS);

    // Direct TLS (PostgreSQL 17 and later) skips the SSLRequest round trip.
    config.ssl_root_cert_pem = ca;
    config.ssl_direct = true;
    direct, error := Pg.connect(config);
    if error.kind == .NONE {
        assert(direct.tls_used);
        Pg.close(direct);
    } else {
        // Servers before 17 do not speak it.
        Pg.free_error(*error);
    }

    // The role is hostssl only.
    config = config_for("jai_tls_scram", "tls secret");
    connect_fails(config, .SERVER);

    // PREFER uses TLS when offered; trust auth over TLS works too.
    config = config_for("postgres");
    config.ssl_mode = .PREFER;
    db = connect_ok(config);
    assert(db.tls_used);
    Pg.close(db);
}

test_unix_socket :: () {
    if !socket_dir || OS == .WINDOWS return;

    config := config_for("postgres");
    config.host = socket_dir;
    config.ssl_mode = .REQUIRE; // ignored over a Unix socket
    db := connect_ok(config);

    assert(!db.tls_used);
    assert(scalar(db, "select 1 + 1", s64) == 2);
    Pg.close(db);
}

test_values :: (format: Pg.Format) {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);

    run :: (sql: string) -> Pg.Result #expand {
        return query_in(db, sql, format);
    }

    result := run(#string SQL
        select true, false, 32767::int2, (-2147483648)::int4, 9223372036854775807::int8,
               1.5::float4, -2.25e300::float8, 'héllo'::text, 'x'::varchar(3), '\x00ff10'::bytea,
               'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'::uuid, 4294967295::oid, '{"a": [1, 2]}'::jsonb
    SQL);

    defer Pg.free_result(*result);
    assert(result.columns.count == 13 && result.row_count == 1);
    assert(result.columns[2].type_oid == xx Pg.Oid.INT2);
    assert(get(*result, 0, bool));
    assert(!get(*result, 1, bool));
    assert(get(*result, 2, s16) == 32767);
    assert(get(*result, 2, s64) == 32767);
    assert(get(*result, 3, s32) == -2147483648);
    assert(get(*result, 4, s64) == 9223372036854775807);
    assert(get(*result, 5, float32) == 1.5);
    assert(get(*result, 6, float64) == -2.25e300);
    assert(get(*result, 7, string) == "héllo");
    assert(get(*result, 8, string) == "x");
    bytes := get(*result, 9, [] u8);

    assert(bytes.count == 3 && bytes[0] == 0 && bytes[1] == 0xff && bytes[2] == 0x10);
    assert(Pg.uuid_text(get(*result, 10, Pg.Uuid)) == "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11");
    assert(get(*result, 11, u32) == 4294967295);
    assert(get(*result, 12, string) == "{\"a\": [1, 2]}");

    // A value that does not fit the requested type is an error, not a truncation.
    _, error := Pg.get(*result, 0, 2, s8);

    assert(error.kind == .DECODE);
    Pg.free_error(*error);

    result2 := run(#string SQL
        select '2024-02-29 13:45:10.123456+00'::timestamptz, '1999-12-31 23:59:59'::timestamp,
               '2000-01-01'::date, '0044-03-15 BC'::date, '23:59:59.5'::time,
               '1 year 2 mons -3 days 04:05:06.7'::interval, '-12345678901234567890.0012300'::numeric,
               'NaN'::numeric, 'infinity'::timestamptz, '-infinity'::date, null::int4,
               '2024-02-29 13:45:10+05:30'::timestamptz, 0::numeric, 0.00010::numeric, 10000::numeric
    SQL);

    defer Pg.free_result(*result2);
    t := get(*result2, 0, Pg.Timestamp);

    assert(t.microseconds == Pg.timestamp_from_civil(2024, 2, 29, 13, 45, 10, 123456).microseconds);
    local := get(*result2, 1, Pg.Local_Timestamp);

    assert(local.microseconds == -1_000_000);
    assert(get(*result2, 2, Pg.Date).days == 0);
    year, month, day := Pg.civil_from_date(get(*result2, 3, Pg.Date));

    assert(year == -43 && month == 3 && day == 15);
    assert(get(*result2, 4, Pg.Time).microseconds == 86_399_500_000);
    interval := get(*result2, 5, Pg.Interval);

    assert(interval.months == 14 && interval.days == -3 && interval.microseconds == 14_706_700_000);
    assert(get(*result2, 6, Pg.Numeric).text == "-12345678901234567890.0012300");
    assert(get(*result2, 7, Pg.Numeric).text == "NaN");
    assert(get(*result2, 8, Pg.Timestamp).microseconds == Pg.TIMESTAMP_INFINITY);
    assert(get(*result2, 9, Pg.Date).days == Pg.DATE_MINUS_INFINITY);
    assert(get(*result2, 10, Pg.Nullable(s32)).null);
    _, error = Pg.get(*result2, 0, 10, s32);
    assert(error.kind == .DECODE);
    Pg.free_error(*error);
    assert(get(*result2, 11, Pg.Timestamp).microseconds == Pg.timestamp_from_civil(2024, 2, 29, 8, 15, 10).microseconds);
    assert(get(*result2, 12, Pg.Numeric).text == "0");
    assert(get(*result2, 13, Pg.Numeric).text == "0.00010");
    assert(get(*result2, 14, Pg.Numeric).text == "10000");
    n, ok := Pg.numeric_to_s64(get(*result2, 6, Pg.Numeric));

    assert(!ok);
    n, ok = Pg.numeric_to_s64(get(*result2, 14, Pg.Numeric));
    assert(ok && n == 10000);
}

query_in :: (db: *Pg.Connection, sql: string, format: Pg.Format) -> Pg.Result {
    result: Pg.Result;
    error: Pg.Pg_Error;

    if format == .TEXT result, error = Pg.query_dynamic(db, sql);
    else result, error = Pg.query_binary(db, sql);

    assert_ok(error, sql);

    return result;
}

get :: (result: *Pg.Result, column: s64, $T: Type, location := #caller_location) -> T {
    value, error := Pg.get(result, 0, column, T);
    assert_ok(error, tprint("column %", column), location);
    return value;
}

Settings :: struct {
    theme: string;
    font_size := 14; @json(default)
    beta: Json_Maybe(bool);
    tags: [..] string; @json(default)
}

test_json :: (format: Pg.Format) {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);

    sql := #string SQL
        select '{"theme": "dark", "beta": null, "tags": ["a", "b\"c"]}'::jsonb,
               '{"theme": "light", "font_size": 12}'::json,
               null::jsonb,
               array['{"theme": "x"}', '{"theme": "y", "beta": true}']::jsonb[],
               array['{"theme": "z"}']::json[],
               '{"b": 1, "a": [true, null]}'::jsonb,
               'null'::jsonb
SQL;
    result := query_in(db, sql, format);

    defer Pg.free_result(*result);

    first := get(*result, 0, Pg.Json(Settings));

    assert(first.value.theme == "dark" && first.value.font_size == 14);
    assert(first.value.beta.state == .NULL);
    assert(first.value.tags.count == 2 && first.value.tags[1] == "b\"c");

    // Strings without escapes point into the result.
    theme := first.value.theme.data;

    assert(theme >= result.data.data && theme < result.data.data + result.data.count);

    second := get(*result, 1, Pg.Json(Settings));

    assert(second.value.theme == "light" && second.value.font_size == 12 && second.value.beta.state == .MISSING);

    // SQL NULL is the column's nullability, not the document's.
    missing := get(*result, 2, Pg.Nullable(Pg.Json(Settings)));

    assert(missing.null);

    list := get(*result, 3, [] Pg.Json(Settings));

    assert(list.count == 2 && list[0].value.theme == "x" && list[1].value.beta.state == .SET && list[1].value.beta.value);

    texts := get(*result, 4, [] Pg.Json(Settings));

    assert(texts.count == 1 && texts[0].value.theme == "z");

    raw := get(*result, 5, Pg.Json_Raw);

    assert(raw.text == "{\"a\": [true, null], \"b\": 1}", raw.text);

    tree := get(*result, 5, Pg.Json(Json_Value));

    assert(json_count(*tree.value) == 2);

    null_document := get(*result, 6, Pg.Json(Json_Maybe(Settings)));

    assert(null_document.value.state == .NULL);

    // Errors name the column and the path inside the document.
    _, error := Pg.get(*result, 0, 1, [] Pg.Json(Settings));

    assert(error.kind == .DECODE);
    Pg.free_error(*error);

    _, error = Pg.get(*result, 0, 5, Pg.Json(Settings));
    assert(error.kind == .DECODE && contains(error.message, "column \"jsonb\""), error.message);
    Pg.free_error(*error);

    // Unknown fields are ignored unless the result's options deny them.
    only := get(*result, 0, Pg.Json(Theme_Only));

    assert(only.value.theme == "dark");

    result.json_options.deny_unknown_fields = true;
    _, error = Pg.get(*result, 0, 3, [] Pg.Json(Theme_Only));
    assert(error.kind == .DECODE && contains(error.message, "element 2: at $"), error.message);
    Pg.free_error(*error);
    result.json_options = .{};

    // A json value keeps duplicate keys; the last wins unless asked otherwise.
    duplicate := query_in(db, "select '{\"theme\": \"a\", \"theme\": \"b\"}'::json", format);

    defer Pg.free_result(*duplicate);

    duplicate.json_options.duplicate_keys = .LAST;
    last := get(*duplicate, 0, Pg.Json(Settings));

    assert(last.value.theme == "b");

    duplicate.json_options.duplicate_keys = .ERROR;
    _, error = Pg.get(*duplicate, 0, 0, Pg.Json(Settings));
    assert(error.kind == .DECODE && contains(error.message, "theme"), error.message);
    Pg.free_error(*error);

    // Parameters: json(value) is sent as text for json or jsonb; arrays go as jsonb[].
    settings: Settings;
    settings.theme = "solar";
    settings.beta.state = .SET;
    settings.beta.value = true;

    echo, echo_error := Pg.query_dynamic(db, "select $1::jsonb->>'theme', $1::json->'beta', $2::jsonb[], $3::jsonb", Pg.json(settings), .[Pg.json(settings)], Pg.Json_Raw.{ "[1, 2]" });

    assert_ok(echo_error, "json parameters");
    defer Pg.free_result(*echo);
    assert(get(*echo, 0, string) == "solar");
    assert(get(*echo, 1, string) == "true");

    back := get(*echo, 2, [] Pg.Json(Settings));

    assert(back.count == 1 && back[0].value.theme == "solar" && back[0].value.beta.value);
    assert(get(*echo, 3, string) == "[1, 2]");
}

Theme_Only :: struct {
    theme: string;
}

test_arrays :: () {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);

    for format: Pg.Format.[.TEXT, .BINARY] {
        sql := #string SQL
            select array[1, 2, null]::int4[], array['a', 'b c', 'd"e', 'f\g', null, 'NULL']::text[],
                   '{}'::int8[], '[0:1]={7,8}'::int2[], array['00000000-0000-0000-0000-000000000001'::uuid]
        SQL;
        result := query_in(db, sql, format);

        defer Pg.free_result(*result);
        error: Pg.Pg_Error;
        ints := get(*result, 0, [] Pg.Nullable(s32));

        assert(ints.count == 3 && ints[0].value == 1 && ints[1].value == 2 && ints[2].null);
        texts := get(*result, 1, [] Pg.Nullable(string));

        assert(texts.count == 6);
        assert(texts[0].value == "a" && texts[1].value == "b c" && texts[2].value == "d\"e" && texts[3].value == "f\\g");
        assert(texts[4].null && !texts[5].null && texts[5].value == "NULL");
        assert(get(*result, 2, [] s64).count == 0);
        shifted := get(*result, 3, [] s16);

        assert(shifted.count == 2 && shifted[0] == 7 && shifted[1] == 8);
        uuids := get(*result, 4, [] Pg.Uuid);

        assert(uuids.count == 1 && uuids[0].bytes[15] == 1);
        _, error = Pg.get(*result, 0, 0, [] s32);
        assert(error.kind == .DECODE); // holds a NULL
        Pg.free_error(*error);
    }
}

test_parameters :: () {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);

    bytes: [3] u8 = .[1, 0, 255];
    uuid, uuid_ok := Pg.parse_uuid("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11");

    assert(uuid_ok);
    when := Pg.timestamp_from_civil(2030, 1, 2, 3, 4, 5, 6);
    nothing: Pg.Nullable(s64);
    nothing.null = true;
    big: u64 = 18_000_000_000_000_000_000;
    ids: [] s64 = .[3, 1, 2];
    names: [] Pg.Nullable(string) = .[Pg.some("x"), .{ null = true }];

    result, error := Pg.query_dynamic(db, #string SQL
        select $1::int8 + 1, $2 || '!', $3::bool, $4::float8 * 2, $5::bytea, $6::uuid, $7::timestamptz,
               $8::int8 is null, $9::numeric, $10::text, array_length($11::int8[], 1), $12::text[],
               $13::numeric * 2, $14::date, $15::interval
    SQL, 41, "hey", true, 1.25, bytes, uuid, when, nothing, big, Pg.some("set"), ids, names,
        Pg.Numeric.{ "1.5" }, Pg.date_from_civil(1999, 12, 31), Pg.Interval.{ months = 1, days = 2, microseconds = 3 });

    assert_ok(error, "parameters");
    defer Pg.free_result(*result);
    assert(get(*result, 0, s64) == 42);
    assert(get(*result, 1, string) == "hey!");
    assert(get(*result, 2, bool));
    assert(get(*result, 3, float64) == 2.5);
    got := get(*result, 4, [] u8);

    assert(got.count == 3 && got[2] == 255);
    assert(Pg.uuid_text(get(*result, 5, Pg.Uuid)) == Pg.uuid_text(uuid));
    assert(get(*result, 6, Pg.Timestamp).microseconds == when.microseconds);
    assert(get(*result, 7, bool));
    assert(get(*result, 8, Pg.Numeric).text == "18000000000000000000");
    assert(get(*result, 9, string) == "set");
    assert(get(*result, 10, s32) == 3);
    assert(get(*result, 11, string) == "{x,NULL}");
    assert(get(*result, 12, Pg.Numeric).text == "3.0");
    assert(get(*result, 13, string) == "1999-12-31");
    assert(get(*result, 14, string) == "1 mon 2 days 00:00:00.000003");

    // Parameters typed only by context.
    result2: Pg.Result;
    result2, error = Pg.query_dynamic(db, "select $1 = 'abc', $2 + 5", "abc", 10);
    assert_ok(error, "inferred parameters");
    defer Pg.free_result(*result2);
    assert(get(*result2, 0, bool) && get(*result2, 1, s64) == 15);

    _, error = Pg.query_dynamic(db, "select $1::text", "zero\0byte");
    assert(error.kind == .USAGE);
    Pg.free_error(*error);
}

test_errors :: () {
    {
        // A FATAL error right before the server closes is the error, not the close.
        doomed := connect_ok(config_for("postgres"));

        defer Pg.close(doomed);

        for sql: string.["select pg_terminate_backend(pg_backend_pid())", "select 1; select pg_terminate_backend(pg_backend_pid())"] {
            if it_index == 1 doomed = connect_ok(config_for("postgres"));

            error: Pg.Pg_Error;

            if it_index == 0 {
                result: Pg.Result;
                result, error = Pg.query_dynamic(doomed, sql);
                Pg.free_result(*result);
            } else {
                error = Pg.execute_script(doomed, sql);
            }

            assert(error.kind == .SERVER && error.code == .ADMIN_SHUTDOWN && error.severity == "FATAL", "%", Pg.error_text(error));
            assert(doomed.broken);
            Pg.free_error(*error);
            if it_index == 0 Pg.close(doomed);
        }
    }

    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);
    assert_ok(Pg.execute_script(db, "create temp table people (id int primary key, name text not null)"), "create");

    result, error := Pg.query_dynamic(db, "insert into people values ($1, $2), ($1, $2)", 1, "a");

    assert(error.kind == .SERVER && error.code == .UNIQUE_VIOLATION);
    assert(error.table == "people" && error.constraint == "people_pkey" && error.severity == "ERROR");
    assert(Pg.sqlstate_class(error.code) == .INTEGRITY_CONSTRAINT_VIOLATION);
    Pg.free_error(*error);
    assert(db.status == .IDLE && !db.broken);

    result, error = Pg.query_dynamic(db, "selec 1");
    assert(error.code == .SYNTAX_ERROR && error.position == 1);
    Pg.free_error(*error);

    result, error = Pg.query_dynamic(db, "insert into people values (1, null)");
    assert(error.code == .NOT_NULL_VIOLATION && error.column == "name");
    Pg.free_error(*error);

    // The connection is still usable after errors.
    result, error = Pg.query_dynamic(db, "insert into people values (1, 'a'), (2, 'b') returning id");
    assert_ok(error, "insert");
    assert(result.command == "INSERT 0 2" && result.rows_affected == 2 && result.row_count == 2);
    Pg.free_result(*result);

    result, error = Pg.query_dynamic(db, "update people set name = upper(name)");
    assert_ok(error, "update");
    assert(result.rows_affected == 2 && result.columns.count == 0);
    Pg.free_result(*result);

    // An empty query.
    result, error = Pg.query_dynamic(db, "");
    assert_ok(error, "empty");
    assert(result.row_count == 0);
    Pg.free_result(*result);

    // Notices go to on_notice.
    notices := 0;

    db.on_notice = (c: *Pg.Connection, notice: *Pg.Pg_Error) {
        (cast(*int) c.user_data).* += 1;
        assert(notice.message == "hello from plpgsql" && notice.severity == "NOTICE");
    };

    db.user_data = *notices;
    assert_ok(Pg.execute_script(db, "do $$ begin raise notice 'hello from plpgsql'; end $$"), "notice");
    assert(notices == 1);
}

test_scripts_and_status :: () {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);
    assert_ok(Pg.execute_script(db, "create temp table t (x int); insert into t values (1), (2); select * from t"), "script");
    assert(scalar(db, "select count(*) from t", s64) == 2);

    assert_ok(Pg.execute_script(db, "begin"), "begin");
    assert(db.status == .IN_TRANSACTION);
    error := Pg.execute_script(db, "select 1/0");

    assert(error.code == .DIVISION_BY_ZERO);
    Pg.free_error(*error);
    assert(db.status == .FAILED);
    _, error2 := Pg.query_dynamic(db, "select 1");

    assert(error2.code == .IN_FAILED_SQL_TRANSACTION);
    Pg.free_error(*error2);
    assert_ok(Pg.execute_script(db, "rollback"), "rollback");
    assert(db.status == .IDLE);

    // A script runs as one implicit transaction: an error undoes the statements before it.
    error = Pg.execute_script(db, "insert into t values (3); select 1/0; insert into t values (4)");
    assert(error.code == .DIVISION_BY_ZERO);
    Pg.free_error(*error);
    assert(scalar(db, "select count(*) from t", s64) == 2);
}

Script_Log :: struct {
    #as using hooks: Pg.Script_Hooks;
    lines: [..] string;
    copy_rows: [] string;
    copy_sent: s64;
    copied: String_Builder;
}

log_copy :: (hooks: *Pg.Script_Hooks, copy: *Pg.Copy_Info) {
    log := cast(*Script_Log) hooks;
    array_add(*log.lines, tprint("copy % %", ifx copy.incoming then "in" else "out", copy.format));
}

log_result :: (hooks: *Pg.Script_Hooks, result: *Pg.Result) {
    log := cast(*Script_Log) hooks;
    line := tprint("% returns_rows=% columns=% rows=%", result.command, result.returns_rows, result.columns.count, result.row_count);

    for row: 0..result.row_count - 1 {
        for column: 0..result.columns.count - 1 {
            bytes, is_null := Pg.raw_value(result, row, column);
            line = tprint("% %", line, ifx is_null then "NULL" else cast(string) bytes);
        }
    }

    array_add(*log.lines, line);
}

feed_copy :: (hooks: *Pg.Script_Hooks, copy: *Pg.Copy_Info) -> string, bool, string {
    log := cast(*Script_Log) hooks;
    if log.copy_sent == log.copy_rows.count return "", true, "";

    log.copy_sent += 1;
    row := log.copy_rows[log.copy_sent - 1];
    if row == "FAIL" return "", false, "stopped by the test";

    return row, false, "";
}

collect_copy :: (hooks: *Pg.Script_Hooks, copy: *Pg.Copy_Info, data: [] u8) {
    log := cast(*Script_Log) hooks;
    append(*log.copied, cast(string) data);
}

test_run_script :: () {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);

    log: Script_Log;
    log.on_result = log_result;
    log.on_copy = log_copy;
    log.copy_in = feed_copy;
    log.copy_out = collect_copy;
    log.copy_rows = .["1\tone\n2\t", "two\n", "3\t\\N\n"];

    sql := #string SQL
create temp table s (a int, b text);
select 1 as x, null::text as y union all select 2, 'b';
;
select;
copy s from stdin;
copy s to stdout;
select count(*) from s;
SQL;

    assert_ok(Pg.run_script(db, sql, *log), "script");
    expected := string.[
        "CREATE TABLE returns_rows=false columns=0 rows=0",
        "SELECT 2 returns_rows=true columns=2 rows=2 1 NULL 2 b",
        "SELECT 1 returns_rows=true columns=0 rows=1",
        "copy in TEXT",
        "COPY 3 returns_rows=false columns=0 rows=0",
        "copy out TEXT",
        "COPY 3 returns_rows=false columns=0 rows=0",
        "SELECT 1 returns_rows=true columns=1 rows=1 3",
    ];
    for expected {
        if it_index >= log.lines.count || log.lines[it_index] != it {
            print("run_script line %: got \"%\", wanted \"%\"\n", it_index, ifx it_index < log.lines.count then log.lines[it_index] else "(none)", it);
            assert(false);
        }
    }

    assert(log.lines.count == expected.count);
    assert(builder_to_string(*log.copied,, temp) == "1\tone\n2\ttwo\n3\t\\N\n");

    // Results before an error are reported; the error comes back after them.
    log.lines.count = 0;
    error := Pg.run_script(db, "select 7; select 1/0; select 8", *log);

    assert(error.code == .DIVISION_BY_ZERO);
    Pg.free_error(*error);
    assert(log.lines.count == 1 && log.lines[0] == "SELECT 1 returns_rows=true columns=1 rows=1 7");

    // A copy_in failure aborts the COPY, and the connection stays usable.
    log.copy_rows = .["4\tfour\n", "FAIL"];
    log.copy_sent = 0;
    error = Pg.run_script(db, "copy s from stdin", *log);
    assert(error.code == .QUERY_CANCELED, Pg.error_text(error));
    assert(contains(error.message, "stopped by the test"));
    Pg.free_error(*error);
    assert(scalar(db, "select count(*) from s", s64) == 3);

    // A bad row fails the COPY; the rows after it are ignored.
    log.copy_rows = .["5\tfive\n", "x\tbad\n", "6\tsix\n"];
    log.copy_sent = 0;
    error = Pg.run_script(db, "copy s from stdin", *log);
    assert(error.code == .INVALID_TEXT_REPRESENTATION, Pg.error_text(error));
    Pg.free_error(*error);
    assert(scalar(db, "select count(*) from s", s64) == 3);

    // chunk_rows hands rows over in pieces as they arrive; the last call has the tag.
    log.lines.count = 0;
    log.chunk_rows = 2;
    assert_ok(Pg.run_script(db, "select g from generate_series(1, 5) g; select 9", *log), "chunks");
    log.chunk_rows = 0;
    chunks := string.[
        " returns_rows=true columns=1 rows=2 1 2",
        " returns_rows=true columns=1 rows=2 3 4",
        "SELECT 5 returns_rows=true columns=1 rows=1 5",
        "SELECT 1 returns_rows=true columns=1 rows=1 9",
    ];

    assert(log.lines.count == chunks.count);
    for chunks assert(log.lines[it_index] == it, log.lines[it_index]);

    // Without hooks COPY FROM STDIN fails cleanly.
    error = Pg.execute_script(db, "copy s from stdin");
    assert(error.code == .QUERY_CANCELED);
    Pg.free_error(*error);
    assert(scalar(db, "select 5", s64) == 5);
}

test_startup_parameters :: () {
    config := config_for("postgres");
    parameters := Pg.Startup_Parameter.[
        .{ "datestyle", "Postgres, MDY" },
        .{ "TimeZone", "America/Los_Angeles" },
        .{ "geqo", "off" },
    ];

    config.parameters = parameters;
    db := connect_ok(config);

    defer Pg.close(db);
    assert(scalar(db, "show datestyle", string) == "Postgres, MDY");
    assert(scalar(db, "show timezone", string) == "America/Los_Angeles");
    assert(scalar(db, "show geqo", string) == "off");
    assert(scalar(db, "show intervalstyle", string) == "postgres");

    config.parameters = Pg.Startup_Parameter.[.{ "", "x" }];
    connect_fails(config, .CONFIG);

    // The client's DateStyle and IntervalStyle win over the server's unless send_styles is off.
    config.parameters = Pg.Startup_Parameter.[.{ "options", "-c intervalstyle=sql_standard -c datestyle=SQL,DMY" }];
    styled := connect_ok(config);

    assert(scalar(styled, "show intervalstyle", string) == "postgres");
    assert(scalar(styled, "show datestyle", string) == "ISO, DMY");
    Pg.close(styled);

    config.send_styles = false;
    unstyled := connect_ok(config);

    assert(scalar(unstyled, "show intervalstyle", string) == "sql_standard");
    assert(scalar(unstyled, "show datestyle", string) == "SQL, DMY");
    Pg.close(unstyled);
}

Notice_Log :: struct {
    messages: [..] string;
}

record_notice :: (connection: *Pg.Connection, notice: *Pg.Pg_Error) {
    log := cast(*Notice_Log) connection.user_data;
    array_add(*log.messages, copy_string(notice.message,, temp));
}

test_notices_and_describe :: () {
    log: Notice_Log;
    log.messages.allocator = temp;

    config := config_for("postgres");
    config.on_notice = record_notice;
    config.user_data = *log;
    db := connect_ok(config);

    defer Pg.close(db);
    assert_ok(Pg.execute_script(db, "do $$ begin raise notice 'hello %', 7; end $$"), "notice");
    assert(log.messages.count == 1 && log.messages[0] == "hello 7");

    // A query that matches no rows still says it returns rows.
    result, error := Pg.query_dynamic(db, "select 1 where false");

    assert_ok(error, "empty select");
    assert(result.returns_rows && result.row_count == 0);
    Pg.free_result(*result);

    info: Pg.Statement_Info;
    info, error = Pg.describe_statement(db, "select $1::int4 as a, 'x'::varchar(5) as b");
    assert_ok(error, "describe");
    assert(info.parameter_types.count == 1 && info.parameter_types[0] == 23);
    assert(info.columns.count == 2);
    assert(info.columns[0].name == "a" && info.columns[0].type_oid == 23);
    assert(info.columns[1].name == "b" && info.columns[1].type_oid == 1043 && info.columns[1].type_modifier == 9);
    Pg.free_statement_info(*info);

    info, error = Pg.describe_statement(db, "create temp table d (x int)");
    assert_ok(error, "describe utility");
    assert(info.parameter_types.count == 0 && info.columns.count == 0);
    Pg.free_statement_info(*info);

    // Describing runs nothing, and a bad statement leaves the connection usable.
    info, error = Pg.describe_statement(db, "select * from no_such_table");
    assert(error.code == .UNDEFINED_TABLE, Pg.error_text(error));
    Pg.free_error(*error);
    assert(db.status == .IDLE);
    assert(scalar(db, "select count(*) from pg_class where relname = 'd' and relpersistence = 't'", s64) == 0);
}

test_deadline :: () {
    // A deadline sends CancelRequest and drains, so the connection stays usable, plain and over
    // TLS (verify-full and direct TLS reuse the main connection's TLS settings).
    ca, read := read_entire_file(tprint("%ca.pem", FIXTURES));

    assert(read);
    defer free(ca);

    tls := config_for("jai_tls_scram", "tls secret");
    tls.host = "localhost";
    tls.ssl_mode = .VERIFY_FULL;
    tls.ssl_root_cert_pem = ca;

    direct := tls;
    direct.ssl_direct = true;

    for Pg.Config.[config_for("postgres"), tls, direct] {
        db, error := Pg.connect(it);
        if error.kind != .NONE {
            assert(it.ssl_direct, "connect failed: %", Pg.error_text(error)); // before PostgreSQL 17
            Pg.free_error(*error);
            continue;
        }

        defer Pg.close(db);
        expect_timeout_recovers(db);
    }

    // Canceled inside a transaction: the transaction failed, and ROLLBACK clears it.
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);

    assert(Pg.execute_script(db, "begin").kind == .NONE);
    expect_timeout_recovers(db, .FAILED);
    assert(Pg.execute_script(db, "rollback").kind == .NONE);
    assert(db.status == .IDLE);

    // Io.cancel from another fiber works the same way, and execute_script recovers too.
    canceled_db = db;
    worker := Io.spawn(cancel_worker);

    Io.sleep(100);
    Io.cancel(worker);
    while !worker_done Io.sleep(10);

    assert(worker_error == .CANCELED, "the worker got %", worker_error);
    assert(scalar(db, "select 7", s64) == 7);
}

canceled_db: *Pg.Connection;
worker_error: Pg.Error_Kind;
worker_done: bool;

cancel_worker :: () {
    error := Pg.execute_script(canceled_db, "select pg_sleep(5); select pg_sleep(5)");
    worker_error = error.kind;
    worker_done = true;
    Pg.free_error(*error);
}

expect_timeout_recovers :: (db: *Pg.Connection, status := Pg.Transaction_Status.IDLE) {
    error: Pg.Pg_Error;
    started := current_time_monotonic();

    Io.with_deadline(100, #code {
        result: Pg.Result;
        result, error = Pg.query_dynamic(db, "select pg_sleep(5)");
        Pg.free_result(*result);
    });

    elapsed := to_milliseconds(current_time_monotonic() - started);

    assert(error.kind == .TIMED_OUT, "got %", Pg.error_text(error));
    Pg.free_error(*error);
    assert(!db.broken, "the connection broke");
    assert(elapsed < 3000, "the cancel took % ms", elapsed);
    assert(db.status == status, "status %", db.status);

    if status == .IDLE assert(scalar(db, "select 41 + 1", s64) == 42);
}

test_pool :: () {
    settings: Pg.Pool_Config;
    settings.max_size = 2;
    settings.acquire_timeout = 100;

    pool, error := Pg.create_pool(config_for("postgres"), settings);

    assert_ok(error, "create_pool");

    // Two at most; a third borrower times out.
    a := acquire_ok(pool);
    b := acquire_ok(pool);

    assert(a != b);

    c: *Pg.Connection;
    c, error = Pg.acquire(pool);
    assert(!c && error.kind == .TIMED_OUT, "got %", Pg.error_text(error));
    Pg.free_error(*error);

    // A waiting borrower gets the connection that comes back.
    waiting_pool = pool;
    Io.spawn(pool_waiter);
    Io.yield();
    assert(!waiter_got);
    Pg.release(pool, b);
    Io.yield();
    assert(waiter_got == b, "the waiter did not get the released connection");
    Pg.release(pool, waiter_got);

    // Newest idle first.
    Pg.release(pool, a);
    c = acquire_ok(pool);
    assert(c == a);

    // Left in a transaction: rolled back before it goes back.
    // (Separate calls: a BEGIN in a script pulls the statements before it into the transaction.)
    assert(Pg.execute_script(c, "create temp table pool_check (x int)").kind == .NONE);
    assert(Pg.execute_script(c, "begin; insert into pool_check values (1)").kind == .NONE);
    assert(c.status == .IN_TRANSACTION);
    Pg.release(pool, c);
    c = acquire_ok(pool);
    assert(c == a && c.status == .IDLE);
    assert(scalar(c, "select count(*) from pool_check", s64) == 0);

    // A dead connection fails its health check and is replaced.
    dead_pid := c.backend_pid;
    killer := acquire_ok(pool);

    assert(killer != c);
    Pg.release(pool, c);
    assert(scalar(killer, tprint("select pg_terminate_backend(%)", dead_pid), bool));
    Pg.release(pool, killer);
    pool.settings.health_check_after = 0;
    Io.sleep(50);
    // Both idle now; the newest (killer) comes first, then the dead one is checked and dropped.
    first := acquire_ok(pool);
    second := acquire_ok(pool);

    assert(first.backend_pid != dead_pid && second.backend_pid != dead_pid);
    assert(scalar(second, "select 1", s64) == 1);
    Pg.release(pool, first);
    Pg.release(pool, second);
    pool.settings.health_check_after = 30_000;

    // Past max_lifetime: closed, not reused.
    pool.settings.max_lifetime = 0;
    c = acquire_ok(pool);
    old_pid := c.backend_pid;

    Pg.release(pool, c);
    assert(pool.idle.count == 0 && pool.open == 0, "idle % open %", pool.idle.count, pool.open); // all expired
    pool.settings.max_lifetime = -1;

    // Closing with a borrowed connection: the pool goes when it comes back.
    c = acquire_ok(pool);
    assert(c.backend_pid != old_pid);
    Pg.close_pool(pool);
    _, error = Pg.acquire(pool);
    assert(error.kind == .USAGE);
    Pg.free_error(*error);
    Pg.release(pool, c);

    // min_idle connections open in the background.
    settings = .{};
    settings.min_idle = 2;
    pool, error = Pg.create_pool(config_for("postgres"), settings);
    assert_ok(error, "create_pool");

    for 1..100 {
        if pool.idle.count == 2 break;
        Io.sleep(20);
    }

    assert(pool.idle.count == 2 && pool.open == 2, "idle % open %", pool.idle.count, pool.open);
    Pg.close_pool(pool);
}

waiting_pool: *Pg.Pool;
waiter_got: *Pg.Connection;

pool_waiter :: () {
    c, error := Pg.acquire(waiting_pool);
    assert_ok(error, "acquire while waiting");
    waiter_got = c;
}

acquire_ok :: (pool: *Pg.Pool) -> *Pg.Connection {
    c, error := Pg.acquire(pool);
    assert_ok(error, "acquire");

    return c;
}

attempts: int;

test_transactions :: () {
    pool, error := Pg.create_pool(config_for("postgres"));

    assert_ok(error, "create_pool");
    defer Pg.close_pool(pool);

    setup := acquire_ok(pool);

    assert_ok(Pg.execute_script(setup, "drop table if exists tx_check; create table tx_check (x int)"), "setup");
    Pg.release(pool, setup);

    // Commits when the body returns no error.
    error = Pg.transaction(pool, 1, insert_value);
    assert_ok(error, "commit");

    // An error from the body rolls back and comes out.
    error = Pg.transaction(pool, (tx: *Pg.Connection) -> Pg.Pg_Error {
        _, failure := Pg.query_dynamic(tx, "insert into tx_check values (2)");
        assert(failure.kind == .NONE);
        return Pg.make_error(.USAGE, "changed my mind");
    });
    assert(error.kind == .USAGE);
    Pg.free_error(*error);

    // An ignored failed statement rolls back too.
    error = Pg.transaction(pool, (tx: *Pg.Connection) -> Pg.Pg_Error {
        _, failure := Pg.query_dynamic(tx, "insert into tx_check values (3)");
        _, failure = Pg.query_dynamic(tx, "select 1/0");
        Pg.free_error(*failure);
        return .{};
    }, .REPEATABLE_READ);
    assert(error.kind == .SERVER && error.code == .IN_FAILED_SQL_TRANSACTION, "got %", Pg.error_text(error));
    Pg.free_error(*error);

    // A serialization failure runs the body again.
    attempts = 0;
    error = Pg.transaction(pool, (tx: *Pg.Connection) -> Pg.Pg_Error {
        attempts += 1;
        _, failure := Pg.query_dynamic(tx, "insert into tx_check values (4)");
        if attempts == 1 && failure.kind == .NONE {
            _, failure = Pg.query_dynamic(tx, "do $$ begin raise exception 'retry me' using errcode = '40001'; end $$");
        }

        return failure;
    }, .SERIALIZABLE, retries = 2);
    assert_ok(error, "retried");
    assert(attempts == 2);

    // Without retries it comes out.
    error = Pg.transaction(pool, (tx: *Pg.Connection) -> Pg.Pg_Error {
        _, failure := Pg.query_dynamic(tx, "do $$ begin raise exception 'no retry' using errcode = '40001'; end $$");
        return failure;
    }, read_only = true);
    assert(error.code == .SERIALIZATION_FAILURE);
    Pg.free_error(*error);

    // On a plain connection.
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);
    error = Pg.transaction(db, 5, insert_value);
    assert_ok(error, "plain connection");
    assert(db.status == .IDLE);

    assert(scalar(db, "select string_agg(x::text, ',' order by x) from tx_check", string) == "1,4,5");
    assert_ok(Pg.execute_script(db, "drop table tx_check"), "cleanup");
}

insert_value :: (tx: *Pg.Connection, value: int) -> Pg.Pg_Error {
    _, error := Pg.query_dynamic(tx, "insert into tx_check values ($1)", value);
    return error;
}

// Pipelines: typed statements, an error in the middle, prepared statements, and recovery.
test_pipeline :: () {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);
    assert_ok(Pg.execute_script(db, "create temp table pipe_t(a int, b text)"), "create");

    p, error := Pg.pipeline_enter(db);

    assert_ok(error, "enter");

    // Statements go out together, and the answers come back in order.
    assert_ok(Pg.pipeline_query(p, "insert into pipe_t values ($1, $2)", 1, "one"), "queue");
    assert_ok(Pg.pipeline_query(p, "insert into pipe_t values ($1, $2)", 2, "two"), "queue");
    assert_ok(Pg.pipeline_query(p, "select a, b from pipe_t order by a"), "queue");
    assert_ok(Pg.pipeline_query_binary(p, "select a from pipe_t order by a desc"), "queue");
    assert_ok(Pg.pipeline_sync(p), "sync");
    assert(Pg.pipeline_pending(p) == 5);

    item := next_item(p, .RESULT);

    assert(item.result.rows_affected == 1 && item.result.command == "INSERT 0 1");
    Pg.free_pipeline_result(*item);

    item = next_item(p, .RESULT);
    Pg.free_pipeline_result(*item);

    item = next_item(p, .RESULT);
    assert(item.result.row_count == 2 && item.result.columns.count == 2);

    b, get_error := Pg.get(*item.result, 1, "b", string);

    assert_ok(get_error, "get");
    assert(b == "two");
    Pg.free_pipeline_result(*item);

    item = next_item(p, .RESULT);
    a, binary_error := Pg.get(*item.result, 0, 0, s64);

    assert_ok(binary_error, "binary get");
    assert(a == 2 && item.result.columns[0].format == .BINARY);
    Pg.free_pipeline_result(*item);

    item = next_item(p, .SYNC);
    Pg.free_pipeline_result(*item);
    assert(db.status == .IDLE);

    // A prepared statement, run twice with different arguments, and one that fails.
    assert_ok(Pg.pipeline_prepare(p, "pipe_get", "select b from pipe_t where a = $1", u32.[23]), "prepare");
    assert_ok(Pg.pipeline_execute_prepared(p, "pipe_get", 1), "execute");
    assert_ok(Pg.pipeline_execute_prepared(p, "pipe_get", 2), "execute");
    assert_ok(Pg.pipeline_query(p, "select 1/$1::int", 0), "queue");
    assert_ok(Pg.pipeline_query(p, "select 2"), "queue");
    assert_ok(Pg.pipeline_sync(p), "sync");

    item = next_item(p, .RESULT);
    Pg.free_pipeline_result(*item);

    for string.["one", "two"] {
        item = next_item(p, .RESULT);
        value, value_error := Pg.get(*item.result, 0, 0, string);

        assert_ok(value_error, "get");
        assert(value == it);
        Pg.free_pipeline_result(*item);
    }

    item = next_item(p, .ERROR);
    assert(item.error.code == .DIVISION_BY_ZERO);
    Pg.free_pipeline_result(*item);

    item = next_item(p, .ABORTED);
    Pg.free_pipeline_result(*item);
    item = next_item(p, .SYNC);
    Pg.free_pipeline_result(*item);

    // Back to normal after the exit.
    assert_ok(Pg.pipeline_exit(p), "exit");
    assert(scalar(db, "select 5", s64) == 5);

    // A pipeline inside a transaction sees the transaction's state at each sync.
    assert_ok(Pg.execute_script(db, "begin"), "begin");
    p, error = Pg.pipeline_enter(db);
    assert_ok(error, "enter");
    assert_ok(Pg.pipeline_query(p, "select 1/0"), "queue");
    assert_ok(Pg.pipeline_sync(p), "sync");

    item = next_item(p, .ERROR);
    Pg.free_pipeline_result(*item);
    item = next_item(p, .SYNC);
    Pg.free_pipeline_result(*item);

    assert(db.status == .FAILED);
    assert_ok(Pg.pipeline_exit(p), "exit");
    assert_ok(Pg.execute_script(db, "rollback"), "rollback");

    // Prepared statements outlive the transaction; describe and close them without a pipeline.
    info, describe_error := Pg.describe_prepared(db, "pipe_get");

    assert_ok(describe_error, "describe_prepared");
    assert(info.parameter_types.count == 1 && info.parameter_types[0] == 23 && info.columns.count == 1);
    Pg.free_statement_info(*info);
    assert_ok(Pg.close_prepared(db, "pipe_get"), "close_prepared");
}

next_item :: (p: *Pg.Pipeline, kind: Pg.Pipeline_Kind, location := #caller_location) -> Pg.Pipeline_Result {
    item, error := Pg.pipeline_next_result(p);

    assert_ok(error, "next result", location);

    if item.kind != kind {
        print("%:%: expected a % result, got % (%)\n", location.fully_pathed_filename, location.line_number, kind, item.kind, Pg.error_text(item.error));
        assert(false);
    }

    return item;
}

// A deadline in the middle of a pipeline loses nothing; abandoning one cancels it.
test_pipeline_deadline :: () {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);

    p, error := Pg.pipeline_enter(db);

    assert_ok(error, "enter");
    assert_ok(Pg.pipeline_query(p, "select pg_sleep(0.4), 7"), "queue");
    assert_ok(Pg.pipeline_query(p, "select 8"), "queue");
    assert_ok(Pg.pipeline_sync(p), "sync");

    // The first wait times out; asking again gets the same statement's result.
    item: Pg.Pipeline_Result;

    Io.with_deadline(100, #code {
        item, error = Pg.pipeline_next_result(p);
    });

    assert(error.kind == .TIMED_OUT, "got %", Pg.error_text(error));
    Pg.free_error(*error);
    assert(!db.broken);

    item = next_item(p, .RESULT);
    value, get_error := Pg.get(*item.result, 0, 1, s64);

    assert_ok(get_error, "get");
    assert(value == 7);
    Pg.free_pipeline_result(*item);

    item = next_item(p, .RESULT);
    Pg.free_pipeline_result(*item);
    item = next_item(p, .SYNC);
    Pg.free_pipeline_result(*item);
    assert_ok(Pg.pipeline_exit(p), "exit");

    // Abandoned in the middle of a long statement: the cancel ends it, the connection stays.
    p, error = Pg.pipeline_enter(db);
    assert_ok(error, "enter");
    assert_ok(Pg.pipeline_query(p, "select pg_sleep(5)"), "queue");
    assert_ok(Pg.pipeline_query(p, "select 1"), "queue");
    assert_ok(Pg.pipeline_flush(p), "flush");

    started := current_time_monotonic();

    Io.with_deadline(100, #code {
        item, error = Pg.pipeline_next_result(p);
    });

    assert(error.kind == .TIMED_OUT, "got %", Pg.error_text(error));
    Pg.free_error(*error);

    Pg.pipeline_abandon(p);

    assert(!db.broken, "the connection broke");
    assert(!p.active);
    assert(to_milliseconds(current_time_monotonic() - started) < 3000, "the cancel was slow");
    assert(scalar(db, "select 3", s64) == 3);
}

// Notifications: a wait, a callback, one in the middle of a query, and the wait's deadline.
test_notifications :: () {
    listener := connect_ok(config_for("postgres"));
    sender := connect_ok(config_for("postgres"));

    defer Pg.close(listener);
    defer Pg.close(sender);

    assert_ok(Pg.listen(listener, "pg_test_jobs"), "listen");
    assert_ok(Pg.listen(listener, "odd \"channel\" name"), "listen to a quoted name");

    assert_ok(Pg.notify(sender, "pg_test_jobs", "first"), "notify");
    assert_ok(Pg.notify(sender, "odd \"channel\" name"), "notify");
    assert_ok(Pg.notify(sender, "not_listened", "lost"), "notify");

    n, error := Pg.wait_for_notification(listener);

    assert_ok(error, "wait");
    assert(n.channel == "pg_test_jobs" && n.payload == "first" && n.backend_pid == sender.backend_pid);
    Pg.free_notification(*n);

    n, error = Pg.wait_for_notification(listener);
    assert_ok(error, "wait");
    assert(n.channel == "odd \"channel\" name" && n.payload == "");
    Pg.free_notification(*n);

    // Nothing is left, so a wait with a deadline times out, and the connection stays usable.
    Io.with_deadline(100, #code {
        n, error = Pg.wait_for_notification(listener);
    });

    assert(error.kind == .TIMED_OUT, "got %", Pg.error_text(error));
    Pg.free_error(*error);
    assert(!listener.broken && !listener.busy);

    // One that arrives while a query runs is queued.
    assert_ok(Pg.notify(sender, "pg_test_jobs", "second"), "notify");
    assert(scalar(listener, "select 1 from pg_sleep(0.1)", s64) == 1);

    found: bool;
    n, found = Pg.next_notification(listener);
    assert(found && n.payload == "second");
    Pg.free_notification(*n);
    _, found = Pg.next_notification(listener);
    assert(!found);

    // A connection hears its own notifications, at the end of the statement that sent them.
    assert_ok(Pg.notify(listener, "pg_test_jobs", "mine"), "notify");
    n, error = Pg.wait_for_notification(listener);
    assert_ok(error, "wait");
    assert(n.payload == "mine" && n.backend_pid == listener.backend_pid);
    Pg.free_notification(*n);

    // A callback takes them in place of the queue.
    seen_payload = "";

    listener.on_notification = (c: *Pg.Connection, notification: *Pg.Notification) {
        seen_payload = copy_string(notification.payload);
    };

    assert_ok(Pg.notify(sender, "pg_test_jobs", "called"), "notify");
    assert(scalar(listener, "select 2 from pg_sleep(0.1)", s64) == 2);
    assert(seen_payload == "called");
    assert(listener.notifications.count == 0);
    listener.on_notification = null;

    // Another fiber sends while this one waits.
    waiting_sender = sender;
    sender_done = false;
    Io.spawn(send_later);

    n, error = Pg.wait_for_notification(listener);
    assert_ok(error, "wait");
    assert(n.payload == "later");
    Pg.free_notification(*n);

    while !sender_done Io.sleep(10);

    // Canceling a waiting fiber ends the wait.
    waiting_listener = listener;
    waiter_error = .NONE;
    waiter_done = false;
    waiter := Io.spawn(wait_forever);

    Io.sleep(100);
    Io.cancel(waiter);
    while !waiter_done Io.sleep(10);

    assert(waiter_error == .CANCELED, "the waiter got %", waiter_error);
    assert(!listener.broken && !listener.busy);
    assert(scalar(listener, "select 9", s64) == 9);

    assert_ok(Pg.unlisten(listener, "pg_test_jobs"), "unlisten");
    assert_ok(Pg.unlisten_all(listener), "unlisten all");
}

seen_payload: string;
waiting_sender: *Pg.Connection;
sender_done: bool;
waiting_listener: *Pg.Connection;
waiter_error: Pg.Error_Kind;
waiter_done: bool;

send_later :: () {
    Io.sleep(100);
    assert_ok(Pg.notify(waiting_sender, "pg_test_jobs", "later"), "notify from a fiber");
    sender_done = true;
}

wait_forever :: () {
    n, error := Pg.wait_for_notification(waiting_listener);

    waiter_error = error.kind;
    waiter_done = true;
    Pg.free_notification(*n);
    Pg.free_error(*error);
}

// Protocol 3.0 by default; 3.2 when asked for and the server speaks it, and a clear failure when
// it is required and the server does not.
test_protocol_versions :: () {
    plain := connect_ok(config_for("postgres"));

    defer Pg.close(plain);
    assert(plain.protocol_minor == 0 && plain.cancel_key.count == 4);

    modern := plain.server_version >= 180000;

    config := config_for("postgres");
    config.max_protocol_version = .LATEST;

    db := connect_ok(config);

    defer Pg.close(db);

    // A server before 18 answers NegotiateProtocolVersion, and the connection falls back to 3.0.
    assert(db.protocol_minor == ifx modern then 2 else 0, "protocol 3.%", db.protocol_minor);
    assert(db.cancel_key.count == ifx modern then 32 else 4, "a % byte cancel key", db.cancel_key.count);
    assert(scalar(db, "select 1", s64) == 1);
    expect_timeout_recovers(db);

    config.min_protocol_version = .LATEST;

    if modern {
        required := connect_ok(config);

        assert(required.protocol_minor == 2);
        Pg.close(required);
    } else {
        error := connect_fails(config, .PROTOCOL);
        assert(contains(error.message, "protocol version 3.0"), "got %", error.message);
        Pg.free_error(*error);
    }

    config.max_protocol_version = .V3_0;
    error := connect_fails(config, .CONFIG);

    Pg.free_error(*error);
}

#if OS == .WINDOWS {
    environment :: (name: string) -> string {
        return "";
    }
} else {
    libc :: #system_library "libc";
    getenv :: (name: *u8) -> *u8 #foreign libc;

    environment :: (name: string) -> string {
        value := getenv(temp_c_string(name));
        if !value return "";
        return to_string(value);
    }
}
