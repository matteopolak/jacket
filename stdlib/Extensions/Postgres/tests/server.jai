// Runs against a live server started by tests/start_server.sh; skipped when JAI_PG_TEST_PORT is
// not set.
#import "Basic";
#import "String";
#import "File";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";
#import "Extensions/Json";
#import "Extensions/Environment";

#if OS != .WINDOWS {
    POSIX :: #import "POSIX";
}

FIXTURES :: #run join(#filepath, "../../Tls/tests/fixtures/");

port: u16;
socket_dir: string;

main :: () {
    port_text := environment_variable("JAI_PG_TEST_PORT");
    if !port_text {
        print("skipped: set JAI_PG_TEST_PORT (see tests/start_server.sh)\n");
        return;
    }

    value, valid := string_to_int(port_text);

    assert(valid, "bad JAI_PG_TEST_PORT");
    port = cast(u16) value;
    socket_dir = environment_variable("JAI_PG_TEST_SOCKET_DIR");

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
        step("test_statement_cache");
        test_statement_cache();
        step("test_statement_timeout");
        test_statement_timeout();
        step("test_copy_out");
        test_copy_out();
        step("test_listener");
        test_listener();
        step("test_multiple_hosts");
        test_multiple_hosts();
    });
    print("ok\n");
}

step :: (name: string) {
    if environment_variable("JAI_PG_TEST_VERBOSE") print("%\n", name);
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

    // A second login with the same password and salt does not run PBKDF2 again.
    runs := Pg.scram_pbkdf2_runs();
    again := connect_ok(config_for("jai_scram", "scram secret"));

    assert(Pg.scram_pbkdf2_runs() == runs, "a second SCRAM login ran PBKDF2 again");
    Pg.close(again);
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

    // sslkeylogfile: each handshake appends its secrets to a file only the owner can read.
    config.ssl_direct = false;
    config.ssl_key_log_file = "pg-key-log-test.txt";
    file_delete(config.ssl_key_log_file);
    defer file_delete("pg-key-log-test.txt");

    db = connect_ok(config);
    Pg.close(db);
    config.ssl_max_protocol_version = .TLS_1_2;
    db = connect_ok(config);
    Pg.close(db);
    config.ssl_max_protocol_version = .TLS_1_3;

    key_log, logged := read_entire_file(config.ssl_key_log_file);

    assert(logged, "the key log file was not written");
    assert(contains(key_log, "CLIENT_TRAFFIC_SECRET_0 ") && contains(key_log, "\nCLIENT_RANDOM "), "%", key_log);
    free(key_log);

    #if OS != .WINDOWS {
        status: POSIX.stat_t;
        assert(POSIX.stat(temp_c_string(config.ssl_key_log_file), *status) == 0);
        assert((status.st_mode & 0x3F) == 0, "the key log file can be read by others");
    }

    // A file that cannot be opened does not stop the connection.
    config.ssl_key_log_file = "no-such-directory/key-log.txt";
    db = connect_ok(config);
    Pg.close(db);
    config.ssl_key_log_file = "";

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

    result.json_options.unknown_fields = .DENY;
    _, error = Pg.get(*result, 0, 3, [] Pg.Json(Theme_Only));
    assert(error.kind == .DECODE && contains(error.message, "element 2: at $"), error.message);
    Pg.free_error(*error);
    result.json_options = .{};

    // Strictness has three layers: the connection's, a `Json_Columns` entry's (the `OPTIONS` of
    // the column's type) and the query's, the most specific last.
    Strict :: Pg.Json(Theme_Only, Pg.pack_json_options(.{ unknown_fields = .DENY }));
    Relaxed :: Pg.Json(Theme_Only, Pg.pack_json_options(.{ unknown_fields = .IGNORE }));

    _, error = Pg.get(*result, 0, 0, Strict);
    assert(error.kind == .DECODE && contains(error.message, "at $"), error.message);
    Pg.free_error(*error);

    result.json_query.unknown_fields = .IGNORE;
    strict := get(*result, 0, Strict);

    assert(strict.value.theme == "dark");
    result.json_query = .{};

    result.json_options.unknown_fields = .DENY;
    relaxed := get(*result, 0, Relaxed);

    assert(relaxed.value.theme == "dark");
    result.json_options = .{};

    // A missing field is an error unless the layer says it keeps its initializer.
    empty := query_in(db, "select '{}'::json", format);

    defer Pg.free_result(*empty);

    _, error = Pg.get(*empty, 0, 0, Pg.Json(Theme_Only));
    assert(error.kind == .DECODE, "a missing field decoded");
    Pg.free_error(*error);

    defaulted := get(*empty, 0, Pg.Json(Theme_Only, Pg.pack_json_options(.{ missing_fields = .DEFAULT })));

    assert(defaulted.value.theme == "");

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

            assert(error.kind == .SERVER && error.code == .ADMIN_SHUTDOWN && Pg.error_fields(error).severity == "FATAL", "%", Pg.error_text(error));
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
    fields := Pg.error_fields(error);

    assert(fields.table == "people" && fields.constraint == "people_pkey" && fields.severity == "ERROR");
    assert(Pg.sqlstate_class(error.code) == .INTEGRITY_CONSTRAINT_VIOLATION);
    Pg.free_error(*error);
    assert(db.status == .IDLE && !db.broken);

    result, error = Pg.query_dynamic(db, "selec 1");
    assert(error.code == .SYNTAX_ERROR && Pg.error_fields(error).position == 1);
    Pg.free_error(*error);

    result, error = Pg.query_dynamic(db, "insert into people values (1, null)");
    assert(error.code == .NOT_NULL_VIOLATION && Pg.error_fields(error).column == "name");
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
        assert(notice.message == "hello from plpgsql" && Pg.error_fields(notice.*).severity == "NOTICE");
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
    wait_for_exit(dead_pid);
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

// Waits until the server has finished a terminated backend. A backend on its way out does not
// answer, and keeps its socket open while it removes its temporary tables (slow on a busy server),
// so a health check of its connection would run into the borrower's deadline.
wait_for_exit :: (pid: s32) {
    watcher := connect_ok(config_for("postgres"));

    defer Pg.close(watcher);

    for 1..500 {
        if scalar(watcher, tprint("select count(*) from pg_stat_activity where pid = %", pid), s64) == 0 {
            Io.sleep(20); // the socket closes just after the backend leaves the list
            return;
        }

        Io.sleep(10);
    }

    assert(false, "backend % did not exit", pid);
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

    // An exit refused for unread results reads nothing: each sync point is still there to read.
    p, error = Pg.pipeline_enter(db);
    assert_ok(error, "enter");
    assert_ok(Pg.pipeline_query(p, "select 1"), "queue");
    assert_ok(Pg.pipeline_sync(p), "sync");
    assert_ok(Pg.pipeline_query(p, "select 2"), "queue");
    assert_ok(Pg.pipeline_sync(p), "sync");

    item = next_item(p, .RESULT);
    Pg.free_pipeline_result(*item);
    assert(Pg.pipeline_exit(p).kind == .USAGE, "exit with a result unread");

    item = next_item(p, .SYNC);
    Pg.free_pipeline_result(*item);
    item = next_item(p, .RESULT);
    Pg.free_pipeline_result(*item);
    item = next_item(p, .SYNC);
    Pg.free_pipeline_result(*item);
    assert_ok(Pg.pipeline_exit(p), "exit");

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

    // Naming 3.2 outright behaves like asking for the latest, cancel key and all.
    named := config_for("postgres");
    named.max_protocol_version = .V3_2;

    pinned := connect_ok(named);

    defer Pg.close(pinned);

    assert(pinned.protocol_minor == ifx modern then 2 else 0, "protocol 3.%", pinned.protocol_minor);
    assert(pinned.cancel_key.count == ifx modern then 32 else 4, "a % byte cancel key", pinned.cancel_key.count);
    expect_timeout_recovers(pinned);

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

parses_sent: s64;

count_parses :: (c: *Pg.Connection, line: string) {
    if contains(line, "Parse\t") parses_sent += 1;
}

prepared_names :: (db: *Pg.Connection) -> [] string {
    result, error := Pg.query_dynamic(db, "select name from pg_prepared_statements where name like 'j%' order by name");

    assert_ok(error, "pg_prepared_statements");

    names: [..] string;
    names.allocator = temp;

    for 0..result.row_count - 1 {
        name, _ := Pg.get(*result, it, 0, string);
        array_add(*names, copy_string(name,, temp));
    }

    Pg.free_result(*result);

    return names;
}

cached_scalar :: (db: *Pg.Connection, sql: string, arg: s64, plan := Pg.Plan.AUTO) -> s64 {
    result, error := Pg.query_cached(db, sql, arg, plan = plan);

    defer Pg.free_result(*result);
    assert_ok(error, sql);

    value, decode_error := Pg.get(*result, 0, 0, s64);

    assert_ok(decode_error, sql);

    return value;
}

// Statements are parsed once per connection and bound afterwards; the cache is bounded, can be
// turned off, and recovers from a statement that went stale or was deallocated.
test_statement_cache :: () {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);

    sql := "select $1::int8 + 1";
    types := u32.[0];

    Pg.set_trace(db, count_parses);
    defer Pg.set_trace(db, null);

    parses_sent = 0;
    assert(cached_scalar(db, sql, 41) == 42);
    assert(cached_scalar(db, sql, 1) == 2);
    assert(cached_scalar(db, sql, 2) == 3);
    assert(parses_sent == 1, "% Parse messages for three runs", parses_sent);
    assert(Pg.statement_cache_count(db) == 1);

    names := prepared_names(db);

    assert(names.count == 1 && names[0] == Pg.statement_name(sql, types), "prepared %", names);

    // A custom plan uses the unnamed statement: nothing is cached for it.
    parses_sent = 0;
    assert(cached_scalar(db, "select $1::int8 * 2", 4, .CUSTOM) == 8);
    assert(cached_scalar(db, "select $1::int8 * 2", 5, .CUSTOM) == 10);
    assert(parses_sent == 2 && Pg.statement_cache_count(db) == 1 && prepared_names(db).count == 1);

    // A parse error caches nothing and leaves the connection usable.
    result, error := Pg.query_cached(db, "selec 1");

    assert(error.kind == .SERVER && error.code == .SYNTAX_ERROR);
    Pg.free_error(*error);
    Pg.free_result(*result);
    assert(Pg.statement_cache_count(db) == 1);
    assert(cached_scalar(db, sql, 9) == 10);

    // Statements without rows, and rows without columns, keep their shape on later runs.
    assert_ok(Pg.execute_script(db, "create temp table cached_t (a int8)"), "create");

    for 0..1 {
        insert, insert_error := Pg.query_cached(db, "insert into cached_t values ($1::int8)", 7 + it);
        assert_ok(insert_error, "insert");
        assert(insert.rows_affected == 1 && !insert.returns_rows);
        Pg.free_result(*insert);
    }

    for 0..1 {
        empty, empty_error := Pg.query_cached(db, "select");
        assert_ok(empty_error, "select");
        assert(empty.returns_rows && empty.columns.count == 0 && empty.row_count == 1);
        Pg.free_result(*empty);
    }

    // Binary results from a cached statement.
    for 0..1 {
        binary, binary_error := Pg.query_cached(db, "select a, a * 2 as twice from cached_t where a = $1::int8", 8, binary = true);

        assert_ok(binary_error, "binary");
        assert(binary.columns.count == 2 && binary.columns[0].format == .BINARY && binary.columns[1].name == "twice");

        twice, _ := Pg.get(*binary, 0, 1, s64);

        assert(twice == 16);
        Pg.free_result(*binary);
    }

    // The server forgets the statements (DEALLOCATE ALL, a pooler's reset): the next run prepares it again.
    assert_ok(Pg.execute_script(db, "deallocate all"), "deallocate");
    parses_sent = 0;
    assert(cached_scalar(db, sql, 99) == 100);
    assert(parses_sent == 1, "% Parse messages after deallocate", parses_sent);

    // A table that changed shape under `select *`: the server refuses the old plan, and the client prepares it again.
    assert_ok(Pg.execute_script(db, "create temp table shaped (a int8); insert into shaped values (1)"), "create shaped");
    first, first_error := Pg.query_cached(db, "select * from shaped");

    assert_ok(first_error, "shaped");
    assert(first.columns.count == 1);
    Pg.free_result(*first);
    assert_ok(Pg.execute_script(db, "alter table shaped add column b int8 default 5"), "alter");
    second, second_error := Pg.query_cached(db, "select * from shaped");

    assert_ok(second_error, "shaped after alter");
    assert(second.columns.count == 2 && second.columns[1].name == "b");
    Pg.free_result(*second);

    // Inside a transaction the same error aborts it: it is reported, and the next transaction is fine.
    assert_ok(Pg.execute_script(db, "begin"), "begin");
    assert_ok(Pg.execute_script(db, "deallocate all"), "deallocate in a transaction");
    result, error = Pg.query_cached(db, "select * from shaped");
    assert(error.kind == .SERVER && error.code == .INVALID_SQL_STATEMENT_NAME, "got %", Pg.error_text(error));
    Pg.free_error(*error);
    Pg.free_result(*result);
    assert_ok(Pg.execute_script(db, "rollback"), "rollback");
    third, third_error := Pg.query_cached(db, "select * from shaped");

    assert_ok(third_error, "shaped after rollback");
    Pg.free_result(*third);

    // Bounded: the least recently used statement is closed on the server.
    small := config_for("postgres");
    small.statement_cache_size = 2;

    limited := connect_ok(small);

    defer Pg.close(limited);

    one := "select $1::int8 + 1";
    two := "select $1::int8 + 2";
    three := "select $1::int8 + 3";

    assert(cached_scalar(limited, one, 1) == 2);
    assert(cached_scalar(limited, two, 1) == 3);
    assert(cached_scalar(limited, one, 2) == 3);
    assert(cached_scalar(limited, three, 1) == 4);
    assert(Pg.statement_cache_count(limited) == 2);

    held := prepared_names(limited);

    assert(held.count == 2);
    assert(held[0] == Pg.statement_name(one, types) || held[1] == Pg.statement_name(one, types));
    assert(held[0] != Pg.statement_name(two, types) && held[1] != Pg.statement_name(two, types), "the least recently used statement stayed: %", held);

    // Off: unnamed statements only.
    off := config_for("postgres");
    off.statement_cache = .OFF;

    plain := connect_ok(off);

    defer Pg.close(plain);

    assert(cached_scalar(plain, sql, 1) == 2);
    assert(cached_scalar(plain, sql, 2) == 3);
    assert(Pg.statement_cache_count(plain) == 0 && prepared_names(plain).count == 0);

    // The same text with another parameter type is another statement.
    before := Pg.statement_cache_count(db);

    for 0..1 {
        typed, typed_error := Pg.query_cached(db, "select $1 || 'x'", "a");
        assert_ok(typed_error, "text parameter");
        Pg.free_result(*typed);
    }

    assert(Pg.statement_cache_count(db) == before + 1);
}

// Several hosts: the next one after a failure, a random order, the kind of server wanted, and
// `sslmode=allow`.
test_multiple_hosts :: () {
    // The first host refuses connections.
    config := config_for("postgres");
    config.hosts = Pg.Host_Port.[.{ "127.0.0.1", "", 1 }, .{ "127.0.0.1", "", port }];

    db := connect_ok(config);

    assert(scalar(db, "select 1", s64) == 1);
    Pg.close(db);

    // All of them do: the last error comes back.
    config.hosts = Pg.Host_Port.[.{ "127.0.0.1", "", 1 }, .{ "127.0.0.1", "", 2 }];
    error := connect_fails(config, .IO);

    Pg.free_error(*error);

    // In the listed order unless asked otherwise; shuffled with load_balance_hosts=random. The
    // TCP host and the socket directory are told apart by whether the session has a server address.
    if socket_dir && OS != .WINDOWS {
        config = config_for("postgres");
        config.hosts = Pg.Host_Port.[.{ "127.0.0.1", "", port }, .{ socket_dir, "", port }];

        for 1..10 {
            db = connect_ok(config);
            assert(scalar(db, "select inet_server_addr() is not null", bool), "the first host was not used first");
            Pg.close(db);
        }

        config.load_balance_hosts = .RANDOM;
        tcp, socket := 0, 0;

        for 1..60 {
            db = connect_ok(config);

            if scalar(db, "select inet_server_addr() is not null", bool) tcp += 1;
            else socket += 1;

            Pg.close(db);
        }

        assert(tcp > 0 && socket > 0, "random order used one host only: % tcp, % socket", tcp, socket);
    }

    // target_session_attrs: a session that is read-only by default, as on a replica.
    config = config_for("postgres");
    config.options = "-c default_transaction_read_only=on";

    for attrs: Pg.Target_Session_Attrs.[.ANY, .READ_ONLY, .PRIMARY, .PREFER_STANDBY] {
        config.target_session_attrs = attrs;
        db = connect_ok(config);
        Pg.close(db);
    }

    for attrs: Pg.Target_Session_Attrs.[.READ_WRITE, .STANDBY] {
        config.target_session_attrs = attrs;
        error = connect_fails(config, .HOST_REJECTED);
        assert(contains(error.message, "session is read-only") || contains(error.message, "not in hot standby"), "got %", error.message);
        Pg.free_error(*error);
    }

    config = config_for("postgres");

    for attrs: Pg.Target_Session_Attrs.[.ANY, .READ_WRITE, .PRIMARY, .PREFER_STANDBY] {
        config.target_session_attrs = attrs;
        db = connect_ok(config);
        Pg.close(db);
    }

    for attrs: Pg.Target_Session_Attrs.[.READ_ONLY, .STANDBY] {
        config.target_session_attrs = attrs;
        error = connect_fails(config, .HOST_REJECTED);
        Pg.free_error(*error);
    }

    // The next host is tried when one has the wrong role.
    config.target_session_attrs = .READ_WRITE;
    config.hosts = Pg.Host_Port.[.{ "127.0.0.1", "", 1 }, .{ "127.0.0.1", "", port }];
    db = connect_ok(config);
    Pg.close(db);

    // Connection strings.
    parsed, parse_error := Pg.parse_conninfo("host=a,b target_session_attrs=prefer-standby load_balance_hosts=random sslmode=allow");

    assert_ok(parse_error, "parse_conninfo");
    assert(parsed.target_session_attrs == .PREFER_STANDBY && parsed.load_balance_hosts == .RANDOM && parsed.ssl_mode == .ALLOW);
    Pg.free_config(*parsed);

    _, parse_error = Pg.parse_conninfo("target_session_attrs=bogus");
    assert(parse_error.kind == .CONFIG && contains(parse_error.message, "invalid target_session_attrs value: \"bogus\""), "got %", Pg.error_text(parse_error));
    Pg.free_error(*parse_error);

    _, parse_error = Pg.parse_conninfo("load_balance_hosts=sometimes");
    assert(parse_error.kind == .CONFIG && contains(parse_error.message, "invalid load_balance_hosts value"), "got %", Pg.error_text(parse_error));
    Pg.free_error(*parse_error);

    // sslmode=allow: plain first, TLS when the server refuses that.
    config = config_for("postgres");
    config.ssl_mode = .ALLOW;
    db = connect_ok(config);
    assert(!db.tls_used);
    Pg.close(db);

    config = config_for("jai_tls_scram", "tls secret");
    config.ssl_mode = .ALLOW;
    db = connect_ok(config);
    assert(db.tls_used);
    Pg.close(db);

    // A wrong password is the server's answer on both attempts.
    config.password = "wrong";
    error = connect_fails(config, .SERVER);
    Pg.free_error(*error);
}

receive_within :: (listener: *Pg.Listener, milliseconds: s64) -> Pg.Notification, Pg.Pg_Error {
    n: Pg.Notification;
    error: Pg.Pg_Error;

    Io.with_deadline(milliseconds, #code {
        n, error = Pg.receive(listener);
    });

    return n, error;
}

// A listener takes a connection out of the pool, delivers notifications in order, listens again on a
// new connection after the old one is killed, and gives the connection back clean.
test_listener :: () {
    pool, pool_error := Pg.create_pool(config_for("postgres"), .{ max_size = 3 });

    assert_ok(pool_error, "create_pool");
    defer Pg.close_pool(pool);

    sender := connect_ok(config_for("postgres"));

    defer Pg.close(sender);

    listener, error := Pg.listen(pool, "pg_gap_a", "pg_gap_b");

    assert_ok(error, "listen");
    assert(pool.open == 1 && pool.idle.count == 0, "the listener holds a pooled connection");

    assert_ok(Pg.notify(sender, "pg_gap_a", "one"), "notify");
    assert_ok(Pg.notify(sender, "pg_gap_b", "two"), "notify");
    assert_ok(Pg.notify(sender, "pg_gap_c", "unheard"), "notify");

    n, receive_error := receive_within(listener, 2000);

    assert_ok(receive_error, "receive");
    assert(n.channel == "pg_gap_a" && n.payload == "one");
    Pg.free_notification(*n);

    n, receive_error = receive_within(listener, 2000);
    assert_ok(receive_error, "receive");
    assert(n.channel == "pg_gap_b" && n.payload == "two");
    Pg.free_notification(*n);

    // Nothing else is waiting.
    n, receive_error = receive_within(listener, 100);
    assert(receive_error.kind == .TIMED_OUT, "got %", Pg.error_text(receive_error));
    Pg.free_error(*receive_error);

    killed := listener.connection.backend_pid;

    assert(scalar(sender, tprint("select pg_terminate_backend(%)", killed), bool));

    waited := 0;
    while listener.reconnects == 0 && waited < 5000 {
        Io.sleep(20);
        waited += 20;
    }

    assert(listener.reconnects == 1, "the listener did not reconnect");
    assert(listener.connection.backend_pid != killed);

    assert_ok(Pg.notify(sender, "pg_gap_b", "after"), "notify after the reconnect");

    n, receive_error = receive_within(listener, 2000);
    assert_ok(receive_error, "receive after the reconnect");
    assert(n.channel == "pg_gap_b" && n.payload == "after");
    Pg.free_notification(*n);

    Pg.close_listener(listener);
    assert(pool.open == 1 && pool.idle.count == 1, "the connection went back to the pool");

    c, acquire_error := Pg.acquire(pool);

    assert_ok(acquire_error, "acquire");
    assert(scalar(c, "select count(*) from pg_listening_channels()", s64) == 0, "the pooled connection still listens");
    Pg.release(pool, c);
}

Copy_Row :: struct {
    id: s64;
    name: string;
    score: Pg.Nullable(s32);
    tags: [] string;
    raw: [] u8;
    price: Pg.Numeric;
    at: Pg.Timestamp;
    doc: Pg.Json_Raw;
    ratio: float64;
    flag: bool;
}

Copy_Pieces :: struct {
    text: [..] u8;
    calls: s64;
}

gather_copy :: (data: [] u8, user_data: *void) {
    pieces := cast(*Copy_Pieces) user_data;
    pieces.calls += 1;
    for data array_add(*pieces.text, it);
}

// COPY ... TO STDOUT, streamed to a callback in any format and decoded from binary into structs.
test_copy_out :: () {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);

    assert_ok(Pg.execute_script(db, #string SQL
create temp table copied (id int8, name text, score int4, tags text[], raw bytea, price numeric, at timestamptz, doc jsonb, ratio float8, flag bool);
insert into copied values
    (1, 'alice', 7, '{a,b}', '\xdeadbeef', 12.50, '2024-05-06 07:08:09+00', '{"k": [1, 2]}', 0.25, true),
    (2, 'bob "q"', null, '{}', '', -0.001, '2000-01-01 00:00:00+00', 'null', -1.5, false),
    (3, 'café', 0, '{}', '\x00', 0, '1999-12-31 23:59:59.999999+00', '{}', 1e300, true)
SQL
), "create");

    // A text stream, a piece per row.
    pieces: Copy_Pieces;
    count, error := Pg.copy_out(db, "copy (select id, name from copied order by id) to stdout", gather_copy, *pieces);

    assert_ok(error, "copy_out text");
    assert(count == 3 && pieces.calls == 3, "% rows in % pieces", count, pieces.calls);
    assert(cast(string) pieces.text == "1\talice\n2\tbob \"q\"\n3\tcafé\n", "got %", cast(string) pieces.text);

    csv: Copy_Pieces;

    count, error = Pg.copy_out(db, "copy (select id, name from copied order by id) to stdout (format csv, header)", gather_copy, *csv);

    assert_ok(error, "copy_out csv");
    assert(count == 3 && begins_with(cast(string) csv.text, "id,name\n1,alice\n2,\"bob \"\"q\"\"\"\n"), "got %", cast(string) csv.text);

    // The raw binary stream starts with the COPY signature.
    binary: Copy_Pieces;

    count, error = Pg.copy_out(db, "copy (select id from copied) to stdout (format binary)", gather_copy, *binary);

    assert_ok(error, "copy_out binary stream");
    assert(count == 3 && begins_with(cast(string) binary.text, "PGCOPY\n\xff\r\n\0"));

    // Statements that are not COPY TO STDOUT are refused.
    _, error = Pg.copy_out(db, "select 1", gather_copy, *binary);
    assert(error.kind == .USAGE, "got %", Pg.error_text(error));
    Pg.free_error(*error);

    _, error = Pg.copy_out(db, "copy copied from stdin", gather_copy, *binary);
    assert(error.kind == .SERVER || error.kind == .USAGE);
    Pg.free_error(*error);
    assert(scalar(db, "select 1", s64) == 1);

    // Rows.
    rows, rows_error := Pg.copy_out(db, "copy (select * from copied order by id) to stdout (format binary)", Copy_Row);

    assert_ok(rows_error, "copy_out rows");
    assert(rows.rows.count == 3);

    first := rows.rows[0];

    assert(first.id == 1 && first.name == "alice" && first.score.value == 7 && !first.score.null);
    assert(first.tags.count == 2 && first.tags[1] == "b");
    assert(first.raw.count == 4 && first.raw[0] == 0xde && first.raw[3] == 0xef);
    assert(first.price.text == "12.50", "got %", first.price.text);
    assert(first.at.microseconds == 768294489000000, "got %", first.at.microseconds);
    assert(contains(first.doc.text, "\"k\""));
    assert(first.ratio == 0.25 && first.flag);

    second := rows.rows[1];

    assert(second.name == "bob \"q\"" && second.score.null && second.tags.count == 0 && second.raw.count == 0);
    assert(second.price.text == "-0.001" && second.at.microseconds == 0 && second.doc.text == "null" && !second.flag);

    third := rows.rows[2];

    assert(third.name == "café" && !third.score.null && third.score.value == 0 && third.tags.count == 0);
    assert(third.at.microseconds == -1 && third.ratio == 1e300);
    Pg.free_copy_rows(*rows);

    // A subset of the members, in the statement's order.
    Small :: struct {
        name: string;
        id: s64;
    }

    small, small_error := Pg.copy_out(db, "copy (select name, id from copied order by id) to stdout (format binary)", Small, .["name", "id"]);

    assert_ok(small_error, "copy_out some columns");
    assert(small.rows.count == 3 && small.rows[2].id == 3 && small.rows[0].name == "alice");
    Pg.free_copy_rows(*small);

    // The inverse of copy_in: what goes in comes out.
    rows, rows_error = Pg.copy_out(db, "copy (select * from copied order by id) to stdout (format binary)", Copy_Row);
    assert_ok(rows_error, "copy_out for the round trip");
    assert_ok(Pg.execute_script(db, "create temp table copied_again (like copied)"), "create again");

    stored, copy_error := Pg.copy_in(db, "copy copied_again from stdin (format binary)", rows.rows);

    assert_ok(copy_error, "copy_in the rows back");
    assert(stored == 3);
    assert(scalar(db, "select count(*) from (select * from copied except select * from copied_again) d", s64) == 0);
    assert(scalar(db, "select count(*) from copied_again", s64) == 3);
    Pg.free_copy_rows(*rows);

    // Mistakes are reported, and the connection stays usable.
    Wrong :: struct {
        id: s64;
        name: s32;
    }

    _, wrong_error := Pg.copy_out(db, "copy (select id, name from copied) to stdout (format binary)", Wrong);

    assert(wrong_error.kind == .DECODE, "got %", Pg.error_text(wrong_error));
    Pg.free_error(*wrong_error);

    Narrow :: struct {
        id: s64;
        score: s32;
    }

    _, null_error := Pg.copy_out(db, "copy (select id, score from copied) to stdout (format binary)", Narrow);

    assert(null_error.kind == .DECODE && contains(null_error.message, "NULL"), "got %", Pg.error_text(null_error));
    Pg.free_error(*null_error);

    _, text_error := Pg.copy_out(db, "copy (select id, name from copied) to stdout", Small);

    assert(text_error.kind == .USAGE, "got %", Pg.error_text(text_error));
    Pg.free_error(*text_error);

    _, count_error := Pg.copy_out(db, "copy (select id from copied) to stdout (format binary)", Small);

    assert(count_error.kind == .USAGE, "got %", Pg.error_text(count_error));
    Pg.free_error(*count_error);
    assert(scalar(db, "select 1", s64) == 1);

    // A pool lends a connection for the call.
    pool, pool_error := Pg.create_pool(config_for("postgres"));

    assert_ok(pool_error, "create_pool");
    defer Pg.close_pool(pool);

    via_pool, via_error := Pg.copy_out(pool, "copy (select 5::int8, 'x'::text) to stdout (format binary)", Small, .["id", "name"]);

    assert_ok(via_error, "copy_out through a pool");
    assert(via_pool.rows.count == 1 && via_pool.rows[0].id == 5);
    Pg.free_copy_rows(*via_pool);
}

// `statement_timeout` is a startup parameter, and `options` carries it through a connection string.
test_statement_timeout :: () {
    config := config_for("postgres");
    config.statement_timeout = 250;

    db := connect_ok(config);

    defer Pg.close(db);

    assert(scalar(db, "show statement_timeout", string) == "250ms");

    result, error := Pg.query_dynamic(db, "select pg_sleep(2)");

    assert(error.kind == .SERVER && error.code == .QUERY_CANCELED, "got %", Pg.error_text(error));
    Pg.free_error(*error);
    Pg.free_result(*result);
    assert(scalar(db, "select 1", s64) == 1);

    url := tprint("postgres://postgres@127.0.0.1:%/postgres?sslmode=disable&options=-c\%20statement_timeout\%3D300", port);
    parsed, url_error := Pg.parse_url(url);

    assert_ok(url_error, "parse_url");

    defer Pg.free_config(*parsed);

    via_options := connect_ok(parsed);

    defer Pg.close(via_options);

    assert(scalar(via_options, "show statement_timeout", string) == "300ms");
}
