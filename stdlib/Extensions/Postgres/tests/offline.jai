// The parts of the client that need no server: URLs, SQLSTATEs, UUIDs and calendar arithmetic.
#import "Basic";
Pg :: #import "Extensions/Postgres";

main :: () {
    test_urls();
    test_sqlstates();
    test_uuids();
    test_calendar();
    test_numerics();
    print("ok\n");
}

test_urls :: () {
    config, error := Pg.parse_url("postgres://app:p%40ss@db.example.com:6543/app_dev?sslmode=verify-full&application_name=api&connect_timeout=3");

    assert(error.kind == .NONE);
    assert(config.user == "app" && config.password == "p@ss");
    assert(config.host == "db.example.com" && config.port == 6543 && config.database == "app_dev");
    assert(config.ssl_mode == .VERIFY_FULL && config.application_name == "api" && config.connect_timeout == 3000);
    Pg.free_config(*config);

    config, error = Pg.parse_url("postgresql://[::1]/x?channel_binding=require&sslnegotiation=direct&sslmode=require");
    assert(error.kind == .NONE);
    assert(config.host == "::1" && config.port == 5432 && config.channel_binding == .REQUIRE && config.ssl_direct);
    assert(config.application_name == "jai" && config.user == "");
    Pg.free_config(*config);

    config, error = Pg.parse_url("postgres:///app?host=%2Fvar%2Frun%2Fpostgresql&user=me");
    assert(error.kind == .NONE);
    assert(config.host == "/var/run/postgresql" && config.user == "me" && config.database == "app");
    Pg.free_config(*config);

    config, error = Pg.parse_url("postgres://localhost");
    assert(error.kind == .NONE && config.host == "localhost" && config.database == "");
    Pg.free_config(*config);

    bad :: string.[
        "mysql://localhost", "postgres://h:0/d", "postgres://h:70000/d", "postgres://h:x/d",
        "postgres://[::1/d", "postgres://h/d?sslmode=sometimes", "postgres://h/d?nope=1",
        "postgres://h/d?port", "postgres://h/d%2", "postgres://h/d%zz",
        "postgres://h/d?sslrootcert=/no/such/file.pem",
    ];
    for bad {
        config, error = Pg.parse_url(it);
        if error.kind != .CONFIG {
            print("% parsed\n", it);
            assert(false);
        }

        Pg.free_error(*error);
    }
}

test_sqlstates :: () {
    assert(Pg.sqlstate_of("23505") == .UNIQUE_VIOLATION);
    assert(Pg.sqlstate_text(.UNIQUE_VIOLATION) == "23505");
    assert(Pg.sqlstate_class(.UNIQUE_VIOLATION) == .INTEGRITY_CONSTRAINT_VIOLATION);
    assert(Pg.sqlstate_class(.SERIALIZATION_FAILURE) == Pg.sqlstate_of("40000"));
    // Codes the enum does not name still round-trip.
    assert(Pg.sqlstate_text(Pg.sqlstate_of("XX123")) == "XX123");
}

test_uuids :: () {
    u, ok := Pg.parse_uuid("A0EEBC99-9C0B-4EF8-BB6D-6BB9BD380A11");

    assert(ok && u.bytes[0] == 0xa0 && u.bytes[15] == 0x11);
    assert(Pg.uuid_text(u) == "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11");
    u, ok = Pg.parse_uuid("a0eebc999c0b4ef8bb6d6bb9bd380a11");
    assert(ok);
    _, ok = Pg.parse_uuid("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a1");
    assert(!ok);
    _, ok = Pg.parse_uuid("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11ff");
    assert(!ok);
    _, ok = Pg.parse_uuid("g0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11");
    assert(!ok);
}

test_calendar :: () {
    assert(Pg.date_from_civil(2000, 1, 1).days == 0);
    assert(Pg.date_from_civil(1970, 1, 1).days == -10_957);
    assert(Pg.date_from_civil(2024, 2, 29).days == 8_825);
    year, month, day := Pg.civil_from_date(.{ 8_825 });

    assert(year == 2024 && month == 2 && day == 29);

    // Every day for a few centuries either side round-trips, including year 0 (1 BC).
    for days: -800_000..800_000 {
        if days % 997 continue;
        y, m, d := Pg.civil_from_date(.{ cast(s32) days });
        assert(Pg.date_from_civil(y, m, d).days == days);
        assert(m >= 1 && m <= 12 && d >= 1 && d <= 31);
    }

    t := Pg.timestamp_from_unix_microseconds(0);

    assert(t.microseconds == -946_684_800_000_000 && Pg.unix_microseconds(t) == 0);
    assert(Pg.timestamp_from_civil(2000, 1, 2, 0, 0, 1).microseconds == 86_401_000_000);
    assert(Pg.now().microseconds > Pg.timestamp_from_civil(2020, 1, 1).microseconds);
}

test_numerics :: () {
    n, ok := Pg.numeric_to_s64(.{ "-123.99" });

    assert(ok && n == -123);
    n, ok = Pg.numeric_to_s64(.{ "99999999999999999999" });
    assert(!ok);
    n, ok = Pg.numeric_to_s64(.{ "NaN" });
    assert(!ok);
    f: float64;
    f, ok = Pg.numeric_to_float64(.{ "1.25" });
    assert(ok && f == 1.25);
    f, ok = Pg.numeric_to_float64(.{ "-Infinity" });
    assert(ok && f < 0 && f * 0 != f * 0);
    assert(Pg.or_else(Pg.Nullable(s64).{ null = true }, 7) == 7);
    assert(Pg.or_else(Pg.some(3), 7) == 3);
}
