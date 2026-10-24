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
    // An address to connect to instead of looking `host` up. `host` is still the name the
    // server's certificate is checked against and the password file is searched by.
    hostaddr: string;
    // Every host of a connection string such as `host1:5432,host2:5433`, the first one included.
    // Empty for one host. This client connects to `host` and `port`, the first of them: it does
    // not try the others.
    hosts: [] Host_Port;
    user: string; // empty: $USER
    password: string;
    database: string; // empty: the user name
    application_name := "jai";
    options: string; // sent as the `options` startup parameter: "-c search_path=app"
    // More startup parameters, or replacements for the client's own (`client_encoding`,
    // `DateStyle`, `IntervalStyle`), matched by name without regard to case. Text-format dates,
    // times and intervals only decode with DateStyle ISO and IntervalStyle postgres.
    parameters: [] Startup_Parameter;
    // Send DateStyle ISO and IntervalStyle postgres. false leaves both to the server's settings
    // (as psql does); text-format dates, times and intervals may then fail to decode.
    send_styles := true;

    // The connection's `on_notice` and `user_data` from the start, so notices sent while logging
    // in (a login event trigger's, say) reach them too. Null keeps the default, which logs.
    on_notice: Notice_Proc;
    user_data: *void;

    // How `json` and `jsonb` columns decode into `Json(T)`: unknown and missing fields, duplicate
    // keys, depth and size limits. Each result starts with these, and its `json_options` can be
    // changed before `get`.
    json_options: Json_Options;

    ssl_mode := Ssl_Mode.PREFER;
    // Start TLS at once, with ALPN `postgresql`, instead of asking first (PostgreSQL 17 and
    // later): saves a round trip. Needs `ssl_mode` REQUIRE or stricter.
    ssl_direct := false;
    ssl_root_cert_pem: string; // the roots to trust; empty: the system's
    channel_binding := Channel_Binding.PREFER;

    // The protocol versions to use, like libpq's `min_protocol_version` and `max_protocol_version`.
    // The startup message asks for the maximum; a server that speaks less answers with the version
    // it does, which the connection uses unless it is below the minimum (then connecting fails).
    // The default is 3.0, as libpq's: `.LATEST` asks for 3.2, which PostgreSQL 18 and later speak.
    min_protocol_version := Protocol_Version.V3_0;
    max_protocol_version := Protocol_Version.V3_0;

    // The whole connect (TCP, TLS, authentication), in milliseconds; negative: no limit.
    connect_timeout := 10_000;
    // After a deadline or cancel ends a query: how long to spend asking the server to stop it and
    // reading what is left of its answer, in milliseconds. The connection breaks if that fails.
    cancel_timeout := 2_000;
    // A server could ask for any number of SCRAM iterations to waste the client's time.
    max_scram_iterations := 1_000_000;
    // Send the password in clear text when the server asks, without TLS.
    allow_cleartext_password := false;

    // The password file to look in when `password` is empty: lines of
    // `host:port:database:user:password`, `*` matching anything, `\` escaping `:` and `\`. Empty:
    // `$PGPASSFILE`, then `~/.pgpass`. A file others can read is ignored, as in libpq.
    passfile: string;
    // The authentication methods the server may use, as libpq's `require_auth`: a comma separated
    // list of `password`, `md5`, `gss`, `sspi`, `scram-sha-256` and `none`, or the same each with
    // a `!` in front to forbid them. Empty: any.
    require_auth: string;

    owned: bool; // set by `parse_url`: the strings belong to the config (`free_config`)
}

Startup_Parameter :: struct {
    name: string;
    value: string;
}

Host_Port :: struct {
    host: string;
    hostaddr: string;
    port: u16 = 5432;
}

// `postgres://user:password@host:port/database?sslmode=verify-full&application_name=api`.
// `postgresql://` works too. A Unix socket directory goes in `host`, percent-encoded or as a
// parameter: `postgres:///app?host=/var/run/postgresql`. It is `parse_conninfo` for a URI.
parse_url :: (url: string) -> Config, Pg_Error {
    if conninfo_uri_prefix(url) == 0 {
        return .{}, make_error(.CONFIG, "a URL must start with postgres:// or postgresql://, not %", url);
    }

    return parse_conninfo(url);
}

// libpq's connection strings: a URI (see `parse_url`) or `host=localhost port=5432 dbname=app`,
// where a value with spaces is quoted, `'a value'`, and `\` escapes the next character
// (`\'` and `\\` inside quotes). Parameters (keywords): host, hostaddr, port, user, password, passfile, dbname,
// sslmode, sslnegotiation (`direct`), sslrootcert (a PEM file, or `system`), channel_binding,
// application_name, fallback_application_name, options, connect_timeout (seconds), client_encoding,
// replication and require_auth. libpq's other keywords are accepted and ignored. Errors read like
// libpq's.
//
// With `use_environment` the keywords that were not given come from `PGHOST`, `PGPORT`,
// `PGUSER`, `PGPASSWORD`, `PGDATABASE` and the other variables libpq reads; otherwise only the
// text counts. The config owns its strings: `free_config`.
parse_conninfo :: (text: string, use_environment := false) -> Config, Pg_Error {
    info, error := parse_conninfo_options(text);
    if error.kind != .NONE return .{}, error;

    defer free_conninfo(*info);

    return config_from_conninfo(info, use_environment);
}

// The `Config` for the options of a connection string, as `parse_conninfo` makes it. Programs
// that combine several sources (a command line and a connection string) set the options
// themselves with `conninfo_set` first.
config_from_conninfo :: (info: Conninfo, use_environment := false) -> Config, Pg_Error {
    config: Config;
    config.owned = true;
    config.host = "";
    config.application_name = "";

    failed :: (config: *Config, format: string, args: ..Any) -> Config, Pg_Error #expand {
        free_config(config);
        return .{}, make_error(.CONFIG, format, ..args);
    }

    value :: (keyword: string) -> string #expand {
        return conninfo_value(info, keyword, use_environment);
    }

    config.user = copy_string(value("user"));
    config.password = copy_string(value("password"));
    config.passfile = copy_string(value("passfile"));
    config.database = copy_string(value("dbname"));
    config.options = copy_string(value("options"));
    config.require_auth = copy_string(value("require_auth"));

    application_name := value("application_name");
    if !application_name application_name = value("fallback_application_name");

    config.application_name = copy_string(application_name);

    // Hosts, addresses and ports line up by position; one port serves every host.
    host_names := split_list(value("host"));
    addresses := split_list(value("hostaddr"));
    ports := split_list(value("port"));
    count := max(max(host_names.count, addresses.count), 1);

    if host_names.count && addresses.count && host_names.count != addresses.count {
        return failed(*config, "could not match % host names to % hostaddr values", host_names.count, addresses.count);
    }

    if ports.count > 1 && ports.count != count {
        return failed(*config, "could not match % port numbers to % hosts", ports.count, count);
    }

    hosts: [..] Host_Port;

    for 0..count - 1 {
        entry: Host_Port;
        if it < host_names.count entry.host = copy_string(host_names[it]);
        if it < addresses.count entry.hostaddr = copy_string(addresses[it]);

        text := ifx ports.count == 1 then ports[0] else ifx it < ports.count then ports[it] else "";

        if text {
            port, valid := parse_port(text);
            if !valid {
                array_add(*hosts, entry);
                config.hosts = hosts;
                return failed(*config, "invalid port number: \"%\"", text);
            }

            entry.port = port;
        }

        array_add(*hosts, entry);
    }

    if hosts.count {
        config.host = copy_string(hosts[0].host);
        config.hostaddr = copy_string(hosts[0].hostaddr);
        config.port = hosts[0].port;
    }

    if hosts.count > 1 {
        config.hosts = hosts;
    } else {
        for hosts {
            free(it.host);
            free(it.hostaddr);
        }

        array_free(hosts);
    }

    root_certificate := value("sslrootcert");
    mode := value("sslmode");

    if !mode && root_certificate == "system" mode = "verify-full";

    if mode == {
        case "disable"; config.ssl_mode = .DISABLE;
        case "allow"; config.ssl_mode = .PREFER;
        case "prefer"; config.ssl_mode = .PREFER;
        case "require"; config.ssl_mode = .REQUIRE;
        case "verify-ca"; config.ssl_mode = .VERIFY_CA;
        case "verify-full"; config.ssl_mode = .VERIFY_FULL;
        case "";
        case;
            return failed(*config, "invalid sslmode value: \"%\"", mode);
    }

    negotiation := value("sslnegotiation");

    if negotiation == {
        case "direct"; config.ssl_direct = true;
        case "postgres";
        case "";
        case;
            return failed(*config, "invalid sslnegotiation value: \"%\"", negotiation);
    }

    binding := value("channel_binding");

    if binding == {
        case "disable"; config.channel_binding = .DISABLE;
        case "prefer"; config.channel_binding = .PREFER;
        case "require"; config.channel_binding = .REQUIRE;
        case "";
        case;
            return failed(*config, "invalid channel_binding value: \"%\"", binding);
    }

    if root_certificate && root_certificate != "system" && config.ssl_mode != .DISABLE {
        pem, read := read_entire_file(root_certificate, log_errors = false);
        if !read return failed(*config, "could not read root certificate file \"%\"", root_certificate);

        config.ssl_root_cert_pem = pem;
    }

    for keyword: string.["min_protocol_version", "max_protocol_version"] {
        text := value(keyword);
        version: Protocol_Version;

        if text == {
            case "3.0"; version = .V3_0;
            case "3.2"; version = .V3_2;
            case "latest"; version = .LATEST;
            case "";
                continue;
            case;
                return failed(*config, "invalid % value: \"%\"", keyword, text);
        }

        if it_index == 0 config.min_protocol_version = version;
        else config.max_protocol_version = version;
    }

    timeout := value("connect_timeout");

    if timeout {
        seconds, valid, remainder := string_to_int(timeout);
        if !valid || remainder return failed(*config, "invalid connect_timeout value: \"%\"", timeout);

        // libpq waits at least two seconds, and does not limit the wait at zero.
        config.connect_timeout = ifx seconds <= 0 then Io.NO_DEADLINE else max(seconds, 2) * 1000;
    }

    parameters: [..] Startup_Parameter;
    encoding := value("client_encoding");

    if encoding && encoding != "auto" {
        array_add(*parameters, .{ copy_string("client_encoding"), copy_string(encoding) });
    }

    replication := value("replication");

    if replication {
        array_add(*parameters, .{ copy_string("replication"), copy_string(replication) });
    }

    config.parameters = parameters;

    if !config.host config.host = copy_string("localhost");
    if !config.application_name config.application_name = copy_string("jai");

    return config, .{};
}

// A config that owns copies of its strings, for keeping beyond the original (`free_config`).
copy_config :: (config: Config) -> Config {
    copy := config;
    copy.owned = true;
    copy.host = copy_string(config.host);
    copy.hostaddr = copy_string(config.hostaddr);
    copy.hosts = array_copy(config.hosts);
    for *copy.hosts {
        it.host = copy_string(it.host);
        it.hostaddr = copy_string(it.hostaddr);
    }

    copy.passfile = copy_string(config.passfile);
    copy.require_auth = copy_string(config.require_auth);
    copy.user = copy_string(config.user);
    copy.password = copy_string(config.password);
    copy.database = copy_string(config.database);
    copy.application_name = copy_string(config.application_name);
    copy.options = copy_string(config.options);
    copy.ssl_root_cert_pem = copy_string(config.ssl_root_cert_pem);
    copy.parameters = array_copy(config.parameters);
    for *copy.parameters {
        it.name = copy_string(it.name);
        it.value = copy_string(it.value);
    }

    return copy;
}

// Frees the strings of a config from `parse_conninfo`, `parse_url` or `copy_config`; does nothing
// for one the program built.
free_config :: (config: *Config) {
    if !config.owned return;

    free(config.host);
    free(config.hostaddr);
    free(config.passfile);
    free(config.require_auth);

    for config.hosts {
        free(it.host);
        free(it.hostaddr);
    }

    array_free(config.hosts);
    free(config.user);
    free(config.password);
    free(config.database);
    free(config.application_name);
    free(config.options);
    free(config.ssl_root_cert_pem);

    for config.parameters {
        free(it.name);
        free(it.value);
    }

    array_free(config.parameters);
    config.* = .{};
}

#scope_module

// The value of a keyword: what the connection string set, else its environment variable.
conninfo_value :: (info: Conninfo, keyword: string, use_environment: bool) -> string {
    value, found := conninfo_get(info, keyword);
    if found return value;
    if !use_environment return "";

    index := conninfo_index(keyword);
    if index < 0 || !CONNINFO_ENVIRONMENT[index] return "";

    return environment(CONNINFO_ENVIRONMENT[index]);
}

// The environment variable libpq reads for each of CONNINFO_KEYWORDS, empty where it has none.
CONNINFO_ENVIRONMENT :: string.[
    "PGSERVICE", "PGUSER", "PGPASSWORD", "PGPASSFILE", "PGCHANNELBINDING", "PGCONNECT_TIMEOUT",
    "PGDATABASE", "PGHOST", "PGHOSTADDR", "PGPORT", "PGCLIENTENCODING", "PGOPTIONS", "PGAPPNAME",
    "", "", "", "", "", "", "PGSSLMODE", "PGSSLNEGOTIATION", "PGSSLCOMPRESSION", "PGSSLCERT",
    "PGSSLKEY", "", "PGSSLCERTMODE", "PGSSLROOTCERT", "PGSSLCRL", "PGSSLCRLDIR", "PGSSLSNI",
    "PGREQUIREPEER", "PGREQUIREAUTH", "PGMINPROTOCOLVERSION", "PGMAXPROTOCOLVERSION", "PGSSLMINPROTOCOLVERSION", "PGSSLMAXPROTOCOLVERSION",
    "PGGSSENCMODE", "PGKRBSRVNAME", "PGGSSLIB", "PGGSSDELEGATION", "", "PGTARGETSESSIONATTRS",
    "PGLOADBALANCEHOSTS",
];

// "a,b,,c" as four entries; empty text as none.
split_list :: (text: string) -> [] string {
    items: [..] string;
    items.allocator = temp;

    if !text return items;

    rest := text;

    while true {
        found, item, after := split_from_left(rest, #char ",");

        if !found {
            array_add(*items, rest);
            break;
        }

        array_add(*items, item);
        rest = after;
    }

    return items;
}

parse_port :: (text: string) -> u16, bool {
    value, valid, remainder := string_to_int(text);
    if !valid || remainder || value < 1 || value > 65535 return 0, false;
    return cast(u16) value, true;
}
