// Where and how to connect: a plain struct with defaults, or one parsed from a URL.

Ssl_Mode :: enum u8 {
    DISABLE; // never TLS
    PREFER; // TLS when the server offers it, no certificate check (the default, like libpq)
    REQUIRE; // TLS, no certificate check
    VERIFY_CA; // TLS, the chain must lead to a trusted root
    VERIFY_FULL; // TLS, the chain and the host name are checked
    ALLOW; // a plain connection first; TLS (no certificate check) only when the server refuses that
}

// Which of several servers `connect` accepts, as libpq's `target_session_attrs`.
Target_Session_Attrs :: enum u8 {
    ANY;
    READ_WRITE; // transactions are not read-only (a primary, by default)
    READ_ONLY;
    PRIMARY; // not in hot standby
    STANDBY; // in hot standby
    PREFER_STANDBY; // a standby if any host is one, else any
}

// The order `connect` tries the hosts in, as libpq's `load_balance_hosts`.
Load_Balance_Hosts :: enum u8 {
    DISABLE; // as listed
    RANDOM; // shuffled for each connection
}

Ssl_Cert_Mode :: enum u8 {
    DISABLE; // never send a client certificate
    ALLOW; // send the certificate in `ssl_cert_pem` when there is one (the default)
    REQUIRE; // fail unless a certificate was sent
}

// The TLS versions libpq names in `ssl_min_protocol_version` and `ssl_max_protocol_version`. The
// TLS library behind this client speaks 1.2 and 1.3 only, so a range that ends below 1.2 cannot
// be used, and a minimum below it is raised to 1.2.
Ssl_Protocol_Version :: enum u8 {
    TLS_1_0;
    TLS_1_1;
    TLS_1_2;
    TLS_1_3;
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
    // Empty for one host (`host` and `port`). With several, `connect` tries them in turn until one
    // accepts the login and suits `target_session_attrs`; `host`, `hostaddr` and `port` are then
    // only the first entry's copy.
    hosts: [] Host_Port;
    target_session_attrs := Target_Session_Attrs.ANY;
    load_balance_hosts := Load_Balance_Hosts.DISABLE;
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
    // The roots to trust, as PEM. With `ssl_mode` VERIFY_CA or VERIFY_FULL, empty means the system's
    // roots. With REQUIRE or PREFER, roots given are checked all the same (the chain, not the
    // name, as libpq does with a root.crt), and empty means no check. The chain must end in a
    // self-signed certificate from this list.
    ssl_root_cert_pem: string;
    // Certificate revocation lists (PEM, any number, or one DER) for the chain: if the server's
    // certificate is listed, or none of them was issued by its issuer, the connection fails.
    ssl_crl_pem: string;
    // The client certificate chain (PEM, the client's certificate first) and its private key (PEM or
    // DER), for servers that ask for one. `ssl_key_password` decrypts an encrypted key. Both
    // `ssl_cert_pem` and `ssl_key` are needed to send a certificate; `ssl_key_file` is only the
    // name error messages use for the key.
    ssl_cert_pem: string;
    ssl_key: string;
    ssl_key_password: string;
    ssl_key_file: string;
    ssl_cert_mode := Ssl_Cert_Mode.ALLOW;
    // Send the host name as SNI, unless it is an IP address or a socket directory.
    ssl_sni := true;
    ssl_min_protocol_version := Ssl_Protocol_Version.TLS_1_2;
    ssl_max_protocol_version := Ssl_Protocol_Version.TLS_1_3;
    channel_binding := Channel_Binding.PREFER;

    // The protocol versions to use, like libpq's `min_protocol_version` and `max_protocol_version`.
    // The startup message asks for the maximum; a server that speaks less answers with the version
    // it does, which the connection uses unless it is below the minimum (then connecting fails).
    // The default is 3.0, as libpq's: `.LATEST` asks for 3.2, which PostgreSQL 18 and later speak.
    min_protocol_version := Protocol_Version.V3_0;
    max_protocol_version := Protocol_Version.V3_0;

    // Named prepared statements, kept per connection for typed queries (`Pg.query`, `execute`, ...):
    // the first run of a query parses it, later runs only bind. `.OFF` uses unnamed statements
    // only, for poolers that do not track named ones (PgBouncer before 1.21). The cache holds
    // `statement_cache_size` statements; the least recently used is closed to make room.
    statement_cache := Statement_Cache_Mode.ON;
    statement_cache_size := 256;
    // The first time a connection prepares a statement, ask the server what it returns and fail
    // with an error if its columns are not the ones the query was compiled against, instead of
    // reading the rows wrongly after a migration under a running program. Costs one round trip
    // per statement and connection.
    verify_statements := false;
    // The server's `statement_timeout` for the session, in milliseconds; 0 keeps the server's own.
    // Sent as a startup parameter, so a pooler that rejects unknown parameters needs
    // `options=-c statement_timeout=...` or a role setting instead.
    statement_timeout := 0;

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
// sslmode, sslnegotiation (`direct`), sslrootcert (a PEM file, or `system`), sslcert, sslkey,
// sslpassword, sslcertmode, sslcrl, sslcrldir, sslsni, ssl_min_protocol_version,
// ssl_max_protocol_version, channel_binding,
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
        case "allow"; config.ssl_mode = .ALLOW;
        case "prefer"; config.ssl_mode = .PREFER;
        case "require"; config.ssl_mode = .REQUIRE;
        case "verify-ca"; config.ssl_mode = .VERIFY_CA;
        case "verify-full"; config.ssl_mode = .VERIFY_FULL;
        case "";
        case;
            return failed(*config, "invalid sslmode value: \"%\"", mode);
    }

    attrs := value("target_session_attrs");

    if attrs == {
        case "any"; config.target_session_attrs = .ANY;
        case "read-write"; config.target_session_attrs = .READ_WRITE;
        case "read-only"; config.target_session_attrs = .READ_ONLY;
        case "primary"; config.target_session_attrs = .PRIMARY;
        case "standby"; config.target_session_attrs = .STANDBY;
        case "prefer-standby"; config.target_session_attrs = .PREFER_STANDBY;
        case "";
        case;
            return failed(*config, "invalid target_session_attrs value: \"%\"", attrs);
    }

    balance := value("load_balance_hosts");

    if balance == {
        case "disable"; config.load_balance_hosts = .DISABLE;
        case "random"; config.load_balance_hosts = .RANDOM;
        case "";
        case;
            return failed(*config, "invalid load_balance_hosts value: \"%\"", balance);
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

    if config.ssl_mode != .DISABLE {
        problem := load_ssl_files(*config, info, use_environment, root_certificate);
        if problem return failed(*config, "%", problem);
    }

    ssl_minimum := value("ssl_min_protocol_version");
    ssl_maximum := value("ssl_max_protocol_version");

    if ssl_minimum {
        version, valid := parse_ssl_protocol(ssl_minimum);
        if !valid return failed(*config, "invalid \"ssl_min_protocol_version\" value: \"%\"", ssl_minimum);

        config.ssl_min_protocol_version = version;
    }

    if ssl_maximum {
        version, valid := parse_ssl_protocol(ssl_maximum);
        if !valid return failed(*config, "invalid \"ssl_max_protocol_version\" value: \"%\"", ssl_maximum);

        config.ssl_max_protocol_version = version;
    }

    if config.ssl_min_protocol_version > config.ssl_max_protocol_version {
        return failed(*config, "invalid SSL protocol version range");
    }

    sni := value("sslsni");

    if sni == {
        case "1"; config.ssl_sni = true;
        case "0"; config.ssl_sni = false;
        case "";
        case;
            return failed(*config, "invalid sslsni value: \"%\"", sni);
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
    copy.ssl_crl_pem = copy_string(config.ssl_crl_pem);
    copy.ssl_cert_pem = copy_string(config.ssl_cert_pem);
    copy.ssl_key = copy_string(config.ssl_key);
    copy.ssl_key_password = copy_string(config.ssl_key_password);
    copy.ssl_key_file = copy_string(config.ssl_key_file);
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
    free(config.ssl_crl_pem);
    free(config.ssl_cert_pem);
    free(config.ssl_key);
    free(config.ssl_key_password);
    free(config.ssl_key_file);

    for config.parameters {
        free(it.name);
        free(it.value);
    }

    array_free(config.parameters);
    config.* = .{};
}

#scope_module

parse_ssl_protocol :: (text: string) -> Ssl_Protocol_Version, bool {
    lower := to_lower_copy(text,, temp);

    if lower == {
        case "tlsv1"; return .TLS_1_0, true;
        case "tlsv1.0"; return .TLS_1_0, true;
        case "tlsv1.1"; return .TLS_1_1, true;
        case "tlsv1.2"; return .TLS_1_2, true;
        case "tlsv1.3"; return .TLS_1_3, true;
    }

    return .TLS_1_2, false;
}

// The directory libpq keeps a user's TLS files in (`~/.postgresql`, `%APPDATA%\postgresql`), or "".
ssl_home_directory :: () -> string {
    #if OS == .WINDOWS {
        base := environment_variable("APPDATA");
        if !base return "";

        return tprint("%/postgresql", base);
    } else {
        home := environment_variable("HOME");
        if !home return "";

        return tprint("%/.postgresql", home);
    }
}

is_regular_file :: (path: string) -> bool {
    if !Typed_Files.file_exists(path) return false;

    is_dir, ok := Typed_Files.is_directory(path);

    return ok && !is_dir;
}

// Reads the files the TLS settings of a connection string name, as libpq does: a root certificate
// that is missing is an error only when the server is to be verified and one was asked for;
// a client certificate or revocation list that is missing is no error. Returns the complaint, or "".
load_ssl_files :: (config: *Config, info: Conninfo, use_environment: bool, root_certificate: string) -> string {
    home := ssl_home_directory();
    verifying := config.ssl_mode == .VERIFY_CA || config.ssl_mode == .VERIFY_FULL;

    // The roots: a file, the system's (with the overrides OpenSSL honours), or none.
    if root_certificate == "system" {
        if config.ssl_mode != .VERIFY_FULL {
            return tprint("weak sslmode \"%\" may not be used with sslrootcert=system (use \"verify-full\")", ssl_mode_name(config.ssl_mode));
        }

        pem := system_roots_override();
        if pem config.ssl_root_cert_pem = pem;
    } else {
        explicit := root_certificate.count > 0;
        path := ifx explicit then root_certificate else ifx home.count then tprint("%/root.crt", home) else "";

        if path && !is_regular_file(path) {
            if explicit && verifying {
                return tprint("root certificate file \"%\" does not exist\nEither provide the file, use the system's trusted roots with sslrootcert=system, or change sslmode to disable server certificate verification.", path);
            }
        } else if path {
            pem, read := read_entire_file(path, log_errors = false);
            if !read return tprint("could not read root certificate file \"%\"", path);

            config.ssl_root_cert_pem = pem;
        }
    }

    // Revocation lists, from a file and a directory of files.
    crl_file := conninfo_value(info, "sslcrl", use_environment);
    if !crl_file && home.count crl_file = tprint("%/root.crl", home);

    crl_directory := conninfo_value(info, "sslcrldir", use_environment);
    lists: String_Builder;

    defer free_buffers(*lists);

    if crl_file && is_regular_file(crl_file) {
        text, read := read_entire_file(crl_file, log_errors = false);
        if !read return tprint("could not read certificate revocation list file \"%\"", crl_file);

        add_crls(*lists, text);
        free(text);
    }

    if crl_directory {
        is_dir, ok := Typed_Files.is_directory(crl_directory);

        if ok && is_dir {
            for Typed_Files.file_list(crl_directory,, temp) {
                text, read := read_entire_file(it, log_errors = false);
                if !read continue;

                add_crls(*lists, text);
                free(text);
            }
        }
    }

    config.ssl_crl_pem = builder_to_string(*lists);

    // The client certificate and its key.
    cert_mode := conninfo_value(info, "sslcertmode", use_environment);

    if cert_mode == {
        case "disable"; config.ssl_cert_mode = .DISABLE;
        case "allow"; config.ssl_cert_mode = .ALLOW;
        case "require"; config.ssl_cert_mode = .REQUIRE;
        case "";
        case;
            return tprint("invalid sslcertmode value: \"%\"", cert_mode);
    }

    config.ssl_key_password = copy_string(conninfo_value(info, "sslpassword", use_environment));

    if config.ssl_cert_mode == .DISABLE return "";

    cert_file := conninfo_value(info, "sslcert", use_environment);
    if !cert_file && home.count cert_file = tprint("%/postgresql.crt", home);

    if !cert_file || !is_regular_file(cert_file) return "";

    certificate, read := read_entire_file(cert_file, log_errors = false);
    if !read return tprint("could not read certificate file \"%\"", cert_file);

    config.ssl_cert_pem = certificate;

    key_file := conninfo_value(info, "sslkey", use_environment);
    if !key_file && home.count key_file = tprint("%/postgresql.key", home);

    if !key_file || !is_regular_file(key_file) {
        return tprint("certificate present, but not private key file \"%\"", key_file);
    }

    #if OS != .WINDOWS {
        status: POSIX.stat_t;

        if POSIX.stat(temp_c_string(key_file), *status) == 0 {
            owner := POSIX.geteuid();

            // The server's own rule: a key only its owner may read, or root's group as well.
            if status.st_uid == owner {
                if (status.st_mode & 0x3F) != 0 return private_key_permissions(key_file);
            } else if status.st_uid == 0 {
                if (status.st_mode & (0x10 | 0x08 | 0x07)) != 0 return private_key_permissions(key_file);
            } else if (status.st_mode & 0x3F) != 0 {
                return private_key_permissions(key_file);
            }
        }
    }

    key, key_read := read_entire_file(key_file, log_errors = false);
    if !key_read return tprint("could not read private key file \"%\"", key_file);

    config.ssl_key = key;
    config.ssl_key_file = copy_string(key_file);

    return "";
}

// The PEM lists in a file, ready to be put one after the other.
add_crls :: (lists: *String_Builder, text: string) {
    if !contains(text, "-----BEGIN") return;

    append(lists, trim_right(text, " \t\r\n"));
    append(lists, "\n");
}

private_key_permissions :: (path: string) -> string {
    return tprint("private key file \"%\" has group or world access; file must have permissions u=rw (0600) or less if owned by the current user, or permissions u=rw,g=r (0640) or less if owned by root", path);
}

ssl_mode_name :: (mode: Ssl_Mode) -> string {
    if mode == {
        case .DISABLE; return "disable";
        case .PREFER; return "prefer";
        case .ALLOW; return "allow";
        case .REQUIRE; return "require";
        case .VERIFY_CA; return "verify-ca";
        case .VERIFY_FULL; return "verify-full";
    }

    return "";
}

// `sslrootcert=system` means the system's roots, which OpenSSL lets $SSL_CERT_FILE and
// $SSL_CERT_DIR replace: the PEM of those, or "" for the system's own.
system_roots_override :: () -> string {
    file := environment_variable("SSL_CERT_FILE");

    if file {
        text, read := read_entire_file(file, log_errors = false);
        if read return text;
    }

    directory := environment_variable("SSL_CERT_DIR");
    if !directory return "";

    all: String_Builder;

    defer free_buffers(*all);

    for part: split_list(directory) {
        is_dir, ok := Typed_Files.is_directory(part);
        if !ok || !is_dir continue;

        for Typed_Files.file_list(part,, temp) {
            text, read := read_entire_file(it, log_errors = false);
            if !read continue;

            append(*all, text);
            append(*all, "\n");
            free(text);
        }
    }

    return builder_to_string(*all);
}

// The value of a keyword: what the connection string set, else its environment variable.
conninfo_value :: (info: Conninfo, keyword: string, use_environment: bool) -> string {
    value, found := conninfo_get(info, keyword);
    if found return value;
    if !use_environment return "";

    index := conninfo_index(keyword);
    if index < 0 || !CONNINFO_ENVIRONMENT[index] return "";

    return environment_variable(CONNINFO_ENVIRONMENT[index]);
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
