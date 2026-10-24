// The parts of the client that need no server: URLs, SQLSTATEs, UUIDs and calendar arithmetic.
#import "Basic";
#import "String";
#import "File";
POSIX :: #import "POSIX";
Pg :: #import "Extensions/Postgres";

chmod_private :: (path: string) {
    POSIX.chmod(temp_c_string(path), 0x180); // 0600
}

chmod_open :: (path: string) {
    POSIX.chmod(temp_c_string(path), 0x1A4); // 0644
}

main :: () {
    test_urls();
    test_conninfo();
    test_require_auth();
    test_saslprep();
    test_passfile();
    test_ssl_options();
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
        "postgres://h/d?sslmode=verify-ca&sslrootcert=/no/such/file.pem",
        "postgres://h/d?sslmode=verify-full&sslrootcert=/no/such/file.pem",
        "postgres://h/d?sslmode=require&sslrootcert=system",
        "postgres://h/d?sslcertmode=sometimes", "postgres://h/d?sslsni=2",
        "postgres://h/d?ssl_min_protocol_version=TLSv9",
        "postgres://h/d?ssl_min_protocol_version=TLSv1.3&ssl_max_protocol_version=TLSv1.2",
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

test_conninfo :: () {
    config, error := Pg.parse_conninfo("host=db1,db2 port=5433,5434 user='a b' dbname=app password=a\\ b require_auth=scram-sha-256 sslmode=require application_name=x");

    assert(error.kind == .NONE);
    assert(config.host == "db1" && config.port == 5433 && config.hosts.count == 2);
    assert(config.hosts[1].host == "db2" && config.hosts[1].port == 5434);
    assert(config.user == "a b" && config.password == "a b" && config.database == "app");
    assert(config.require_auth == "scram-sha-256" && config.ssl_mode == .REQUIRE && config.application_name == "x");
    Pg.free_config(*config);

    config, error = Pg.parse_conninfo("postgresql://u:p%40ss@h1:1,[::1]:2/db?hostaddr=1.2.3.4,5.6.7.8&sslmode=disable");
    assert(error.kind == .NONE);
    assert(config.hosts.count == 2 && config.hosts[1].host == "::1" && config.hosts[1].port == 2);
    assert(config.hostaddr == "1.2.3.4" && config.password == "p@ss" && config.ssl_mode == .DISABLE);
    Pg.free_config(*config);

    config, error = Pg.parse_conninfo("postgres://h/d?min_protocol_version=3.0&max_protocol_version=latest");
    assert(error.kind == .NONE && config.min_protocol_version == .V3_0 && config.max_protocol_version == .LATEST);
    Pg.free_config(*config);

    config, error = Pg.parse_conninfo("sslrootcert=system");
    assert(error.kind == .NONE && config.ssl_mode == .VERIFY_FULL && config.ssl_root_cert_pem == "");
    Pg.free_config(*config);

    // libpq's error texts.
    errors :: string.[
        "postgre://", "missing \"=\" after \"postgre://\" in connection info string",
        "postgres://[::1", "end of string reached when looking for matching \"]\" in IPv6 host address in URI: \"postgres://[::1\"",
        "postgres://[::1]z", "unexpected character \"z\" at position 17 in URI (expected \":\" or \"/\"): \"postgres://[::1]z\"",
        "postgresql://host?key=key=value", "extra key/value separator \"=\" in URI query parameter: \"key\"",
        "postgresql://host?zzz", "missing key/value separator \"=\" in URI query parameter: \"zzz\"",
        "postgresql://a%00b", "forbidden value %00 in percent-encoded value: \"a%00b\"",
        "postgresql://%1", "invalid percent-encoded token: \"%1\"",
        "postgresql://host/db?u%7aer=x", "invalid URI query parameter: \"uzer\"",
        "host=a b", "missing \"=\" after \"b\" in connection info string",
        "host='a", "unterminated quoted string in connection info string",
        "nope=1", "invalid connection option \"nope\"",
        "port=x", "invalid port number: \"x\"",
        "host=a,b port=1,2,3", "could not match 3 port numbers to 2 hosts",
        "max_protocol_version=4.0", "invalid max_protocol_version value: \"4.0\"",
        "sslmode=maybe", "invalid sslmode value: \"maybe\"",
    ];

    for 0..errors.count / 2 - 1 {
        config, error = Pg.parse_conninfo(errors[it * 2]);

        if error.kind != .CONFIG || error.message != errors[it * 2 + 1] {
            print("% gave %\n", errors[it * 2], Pg.error_text(error));
            assert(false);
        }

        Pg.free_error(*error);
    }

    info, _ := Pg.parse_conninfo_options("postgres://h?options=-c%20x%3D1&host=ignored");
    value, found := Pg.conninfo_get(info, "options");

    assert(found && value == "-c x=1");
    value, found = Pg.conninfo_get(info, "host");
    assert(found && value == "h");
    _, found = Pg.conninfo_get(info, "port");
    assert(!found);
    Pg.free_conninfo(*info);
}

test_require_auth :: () {
    policy, error := Pg.parse_require_auth("");

    assert(error.kind == .NONE && !policy.restricted);

    policy, error = Pg.parse_require_auth("password,md5");
    assert(error.kind == .NONE && policy.restricted && policy.allowed == (Pg.Auth_Method.PASSWORD | .MD5));

    policy, error = Pg.parse_require_auth("!none");
    assert(error.kind == .NONE && !(policy.allowed & .NONE) && (policy.allowed & .SCRAM_SHA_256));

    bad :: string.[
        "scram-sha-256,!md5", "negative require_auth method \"!md5\" cannot be mixed with non-negative methods",
        "!password,!none,scram-sha-256", "require_auth method \"scram-sha-256\" cannot be mixed with negative methods",
        "password,md5,password", "require_auth method \"password\" is specified more than once",
        "!none,!md5,!none", "require_auth method \"!none\" is specified more than once",
        "none,abcdefg", "invalid require_auth value: \"abcdefg\"",
    ];

    for 0..bad.count / 2 - 1 {
        policy, error = Pg.parse_require_auth(bad[it * 2]);
        assert(error.kind == .CONFIG && error.message == bad[it * 2 + 1]);
        Pg.free_error(*error);
    }
}

test_saslprep :: () {
    // The examples of RFC 4013, section 3.
    prepared, ok := Pg.saslprep("I\xc2\xadX");

    assert(ok && prepared == "IX");
    prepared, ok = Pg.saslprep("\xe2\x85\xa8");
    assert(ok && prepared == "IX");
    prepared, ok = Pg.saslprep("\xc2\xaa");
    assert(ok && prepared == "a");
    prepared, ok = Pg.saslprep("user");
    assert(ok && prepared == "user");
    // Spaces become plain spaces.
    prepared, ok = Pg.saslprep("a\xc2\xa0b");
    assert(ok && prepared == "a b");
    // A prohibited character, a bidirectional violation and invalid UTF-8 leave the password as it is.
    _, ok = Pg.saslprep("a\xc2\x85b");
    assert(!ok);
    _, ok = Pg.saslprep("foo\xd8\xa71bar");
    assert(!ok);
    _, ok = Pg.saslprep("\xff");
    assert(!ok);
    // A password made only of right-to-left letters, with a digit between them, is fine.
    prepared, ok = Pg.saslprep("\xd8\xa71\xd8\xa7");
    assert(ok && prepared == "\xd8\xa71\xd8\xa7");

    secret := Pg.scram_secret("pass", 1024);

    assert(begins_with(secret, "SCRAM-SHA-256$1024:"));
    assert(Pg.md5_secret("pass", "user").count == 35);
}

test_passfile :: () {
    path := "passfile-test.pgpass";
    text := "# a comment\nother:5432:*:*:no\n*:5432:app:me:with\\:colon:ignored\nlocalhost:*:*:*:fallback\n";

    assert(write_entire_file(path, text));
    chmod_private(path);

    config: Pg.Config;
    config.passfile = path;
    assert(Pg.passfile_password(config, "db", 5432, "app", "me") == "with:colon");
    assert(Pg.passfile_password(config, "db", 5433, "app", "me") == "");
    assert(Pg.passfile_password(config, "/tmp", 5432, "x", "y") == "fallback");

    chmod_open(path);
    assert(Pg.passfile_password(config, "db", 5432, "app", "me") == "");
    file_delete(path);
}

test_ssl_options :: () {
    // A root certificate that is missing matters only when the server is to be verified.
    config, error := Pg.parse_conninfo("host=h sslmode=require sslrootcert=/no/such/file.pem sslcert=/no/such.crt");

    assert(error.kind == .NONE && config.ssl_root_cert_pem == "" && config.ssl_cert_pem == "");
    assert(config.ssl_cert_mode == .ALLOW && config.ssl_sni);
    Pg.free_config(*config);

    config, error = Pg.parse_conninfo("host=h sslsni=0 sslcertmode=require ssl_min_protocol_version=TLSv1.3 sslpassword=pw");
    assert(error.kind == .NONE && !config.ssl_sni && config.ssl_cert_mode == .REQUIRE);
    assert(config.ssl_min_protocol_version == .TLS_1_3 && config.ssl_max_protocol_version == .TLS_1_3 && config.ssl_key_password == "pw");
    Pg.free_config(*config);

    // A certificate needs its key, and a key only its owner may read.
    assert(write_entire_file("ssl-options.crt", "certificate\n"));
    assert(write_entire_file("ssl-options.key", "key\n"));

    config, error = Pg.parse_conninfo("host=h sslcert=ssl-options.crt sslkey=missing.key");
    assert(error.kind == .CONFIG && contains(error.message, "certificate present, but not private key file \"missing.key\""));
    Pg.free_error(*error);

    chmod_open("ssl-options.key");
    config, error = Pg.parse_conninfo("host=h sslcert=ssl-options.crt sslkey=ssl-options.key");
    assert(error.kind == .CONFIG && contains(error.message, "has group or world access"));
    Pg.free_error(*error);

    chmod_private("ssl-options.key");
    config, error = Pg.parse_conninfo("host=h sslcert=ssl-options.crt sslkey=ssl-options.key sslcertmode=disable");
    assert(error.kind == .NONE && config.ssl_cert_pem == "");
    Pg.free_config(*config);

    config, error = Pg.parse_conninfo("host=h sslcert=ssl-options.crt sslkey=ssl-options.key");
    assert(error.kind == .NONE && config.ssl_cert_pem == "certificate\n" && config.ssl_key == "key\n");
    Pg.free_config(*config);

    file_delete("ssl-options.crt");
    file_delete("ssl-options.key");
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
