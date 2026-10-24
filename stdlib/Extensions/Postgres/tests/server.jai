// Runs against a live server started by tests/start_server.sh; skipped when JAI_PG_TEST_PORT is
// not set.
#import "Basic";
#import "String";
#import "File";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";

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
        step("test_parameters");
        test_parameters();
        step("test_errors");
        test_errors();
        step("test_scripts_and_status");
        test_scripts_and_status();
        step("test_deadline");
        test_deadline();
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
    assert(get(*result, 0, bool) == true);
    assert(get(*result, 1, bool) == false);
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

test_deadline :: () {
    db := connect_ok(config_for("postgres"));

    defer Pg.close(db);
    error: Pg.Pg_Error;

    Io.with_deadline(100, #code {
        result: Pg.Result;
        result, error = Pg.query_dynamic(db, "select pg_sleep(5)");
        Pg.free_result(*result);
    });
    assert(error.kind == .TIMED_OUT);
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
