// Where and how to connect: a plain struct with defaults, or one parsed from a URL.

Ssl_Mode :: enum u8 {
    DISABLE; // never TLS
    PREFER; // TLS when the server offers it, no certificate check (the default, like libpq)
    REQUIRE; // TLS, no certificate check
    VERIFY_CA; // TLS, the chain must lead to a trusted root
    VERIFY_FULL; // TLS, the chain and the host name are checked
}

Channel_Binding :: enum u8 {
    DISABLE; // plain SCRAM-SHA-256 only
    PREFER; // SCRAM-SHA-256-PLUS over TLS when the server offers it
    REQUIRE; // fail unless SCRAM-SHA-256-PLUS is used
}

Config :: struct {
    // A host name or address, or a directory holding the server's Unix socket (starts with `/`).
    host := "localhost";
    port: u16 = 5432;
    user: string; // empty: $USER
    password: string;
    database: string; // empty: the user name
    application_name := "jai";
    options: string; // sent as the `options` startup parameter: "-c search_path=app"

    ssl_mode := Ssl_Mode.PREFER;
    // Start TLS at once, with ALPN `postgresql`, instead of asking first (PostgreSQL 17 and
    // later): saves a round trip. Needs `ssl_mode` REQUIRE or stricter.
    ssl_direct := false;
    ssl_root_cert_pem: string; // the roots to trust; empty: the system's
    channel_binding := Channel_Binding.PREFER;

    // The whole connect (TCP, TLS, authentication), in milliseconds; negative: no limit.
    connect_timeout := 10_000;
    // A server could ask for any number of SCRAM iterations to waste the client's time.
    max_scram_iterations := 1_000_000;
    // Send the password in clear text when the server asks, without TLS.
    allow_cleartext_password := false;

    owned: bool; // set by `parse_url`: the strings belong to the config (`free_config`)
}

// `postgres://user:password@host:port/database?sslmode=verify-full&application_name=api`.
// `postgresql://` works too. A Unix socket directory goes in `host`, percent-encoded or as a
// parameter: `postgres:///app?host=/var/run/postgresql`. Parameters: host, port, user, password,
// dbname, sslmode, sslnegotiation (`direct`), sslrootcert (a PEM file), channel_binding,
// application_name, options, connect_timeout (seconds).
parse_url :: (url: string) -> Config, Pg_Error {
    config: Config;
    config.owned = true;
    config.host = "";
    config.application_name = "";
    failed :: (config: *Config, format: string, args: ..Any) -> Config, Pg_Error #expand {
        free_config(config);
        return .{}, make_error(.CONFIG, format, ..args);
    }

    rest := url;
    scheme_found := false;

    for prefix: string.["postgresql://", "postgres://"] {
        if begins_with(rest, prefix) {
            rest = advance(rest, prefix.count);
            scheme_found = true;
            break;
        }
    }

    if !scheme_found return failed(*config, "a URL must start with postgres:// or postgresql://, not %", url);

    query := "";
    found, before, after := split_from_left(rest, #char "?");

    if found {
        rest = before;
        query = after;
    }

    path := "";

    found, before, after = split_from_left(rest, #char "/");
    if found {
        rest = before;
        path = after;
    }

    // user:password@host:port
    at := find_index_from_right(rest, #char "@");
    if at >= 0 {
        credentials := slice(rest, 0, at);

        rest = advance(rest, at + 1);
        found, name, secret := split_from_left(credentials, #char ":");
        ok: bool;
        config.user, ok = percent_decode(ifx found then name else credentials);
        if !ok return failed(*config, "bad percent-encoding in the user name");

        if found {
            config.password, ok = percent_decode(secret);
            if !ok return failed(*config, "bad percent-encoding in the password");
        }
    }

    host_text := rest;
    port_text := "";

    if begins_with(rest, "[") {
        close := find_index_from_left(rest, #char "]");
        if close < 0 return failed(*config, "an IPv6 host needs a closing ]");

        host_text = slice(rest, 1, close - 1);
        tail := advance(rest, close + 1);
        if begins_with(tail, ":") port_text = advance(tail, 1);
        else if tail return failed(*config, "unexpected text after the host: %", tail);
    } else {
        colon := find_index_from_right(rest, #char ":");
        if colon >= 0 {
            host_text = slice(rest, 0, colon);
            port_text = advance(rest, colon + 1);
        }
    }

    ok: bool;
    config.host, ok = percent_decode(host_text);
    if !ok return failed(*config, "bad percent-encoding in the host");

    if port_text {
        port, valid := parse_port(port_text);
        if !valid return failed(*config, "bad port: %", port_text);
        config.port = port;
    }

    config.database, ok = percent_decode(path);
    if !ok return failed(*config, "bad percent-encoding in the database name");

    while query {
        pair := query;

        query = "";
        found, before, after = split_from_left(pair, #char "&");
        if found {
            pair = before;
            query = after;
        }

        if !pair continue;

        has_value, key, encoded := split_from_left(pair, #char "=");
        if !has_value return failed(*config, "parameter % has no value", pair);

        value, decoded := percent_decode(encoded);
        if !decoded return failed(*config, "bad percent-encoding in parameter %", key);

        keep := false;

        if key == {
            case "host";
                free(config.host);
                config.host = value;
                keep = true;
            case "user";
                free(config.user);
                config.user = value;
                keep = true;
            case "password";
                free(config.password);
                config.password = value;
                keep = true;
            case "dbname";
                free(config.database);
                config.database = value;
                keep = true;
            case "application_name";
                free(config.application_name);
                config.application_name = value;
                keep = true;
            case "options";
                free(config.options);
                config.options = value;
                keep = true;
            case "port";
                port, valid := parse_port(value);
                if !valid {
                    free(value);
                    return failed(*config, "bad port: %", encoded);
                }

                config.port = port;
            case "sslmode";
                if value == {
                    case "disable"; config.ssl_mode = .DISABLE;
                    case "allow"; config.ssl_mode = .PREFER;
                    case "prefer"; config.ssl_mode = .PREFER;
                    case "require"; config.ssl_mode = .REQUIRE;
                    case "verify-ca"; config.ssl_mode = .VERIFY_CA;
                    case "verify-full"; config.ssl_mode = .VERIFY_FULL;
                    case;
                        free(value);
                        return failed(*config, "unknown sslmode %", encoded);
                }
            case "sslnegotiation";
                if value == "direct" config.ssl_direct = true;
                else if value != "postgres" {
                    free(value);
                    return failed(*config, "unknown sslnegotiation %", encoded);
                }
            case "channel_binding";
                if value == {
                    case "disable"; config.channel_binding = .DISABLE;
                    case "prefer"; config.channel_binding = .PREFER;
                    case "require"; config.channel_binding = .REQUIRE;
                    case;
                        free(value);
                        return failed(*config, "unknown channel_binding %", encoded);
                }
            case "sslrootcert";
                pem, read := read_entire_file(value, log_errors = false);
                if !read {
                    free(value);
                    return failed(*config, "cannot read sslrootcert %", encoded);
                }

                free(config.ssl_root_cert_pem);
                config.ssl_root_cert_pem = pem;
            case "connect_timeout";
                seconds, valid, remainder := string_to_int(value);
                if !valid || remainder {
                    free(value);
                    return failed(*config, "bad connect_timeout %", encoded);
                }

                config.connect_timeout = ifx seconds <= 0 then Io.NO_DEADLINE else seconds * 1000;
            case;
                free(value);
                return failed(*config, "unknown URL parameter %", key);
        }

        if !keep free(value);
    }

    if !config.host config.host = copy_string("localhost");
    if !config.application_name config.application_name = copy_string("jai");

    return config, .{};
}

// Frees the strings of a config from `parse_url`; does nothing for one the program built.
free_config :: (config: *Config) {
    if !config.owned return;

    free(config.host);
    free(config.user);
    free(config.password);
    free(config.database);
    free(config.application_name);
    free(config.options);
    free(config.ssl_root_cert_pem);
    config.* = .{};
}

#scope_module

parse_port :: (text: string) -> u16, bool {
    value, valid, remainder := string_to_int(text);
    if !valid || remainder || value < 1 || value > 65535 return 0, false;
    return cast(u16) value, true;
}

percent_decode :: (text: string) -> string, bool {
    out := alloc_string(text.count);
    count := 0;
    i := 0;

    while i < text.count {
        c := text[i];
        if c == #char "%" {
            if i + 2 >= text.count {
                free(out);
                return "", false;
            }

            high := hex_value(text[i + 1]);
            low := hex_value(text[i + 2]);

            if high < 0 || low < 0 {
                free(out);
                return "", false;
            }

            out[count] = cast(u8) (high * 16 + low);
            i += 3;
        } else {
            out[count] = c;
            i += 1;
        }

        count += 1;
    }

    out.count = count;

    return out, true;
}
