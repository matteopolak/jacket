// libpq's connection strings: a `key=value` list or a `postgresql://` URI, parsed into the
// options it sets. The error texts are libpq's. `parse_conninfo` (config.jai) turns the options
// into a `Config`.

// Every keyword libpq accepts, in the order it lists them.
CONNINFO_KEYWORDS :: string.[
    "service", "user", "password", "passfile", "channel_binding", "connect_timeout", "dbname",
    "host", "hostaddr", "port", "client_encoding", "options", "application_name",
    "fallback_application_name", "keepalives", "keepalives_idle", "keepalives_interval",
    "keepalives_count", "tcp_user_timeout", "sslmode", "sslnegotiation", "sslcompression",
    "sslcert", "sslkey", "sslpassword", "sslcertmode", "sslrootcert", "sslcrl", "sslcrldir",
    "sslkeylogfile",
    "sslsni", "requirepeer", "require_auth", "min_protocol_version", "max_protocol_version",
    "ssl_min_protocol_version",
    "ssl_max_protocol_version", "gssencmode", "krbsrvname", "gsslib", "gssdelegation",
    "replication", "target_session_attrs", "load_balance_hosts",
];

// The options a connection string set. A keyword that was not given has no value; one given
// empty has an empty value, which is not the same.
Conninfo :: struct {
    values: [] string;
    is_set: [] bool;
}

// Parses a URI (`postgresql://` or `postgres://`) or a `key=value` list. The strings in the
// result belong to it: `free_conninfo`.
parse_conninfo_options :: (text: string) -> Conninfo, Pg_Error {
    info: Conninfo;
    info.values = NewArray(CONNINFO_KEYWORDS.count, string);
    info.is_set = NewArray(CONNINFO_KEYWORDS.count, bool);

    error: Pg_Error;

    if conninfo_uri_prefix(text) > 0 error = parse_uri_options(*info, text);
    else error = parse_keyword_options(*info, text);

    if error.kind != .NONE {
        free_conninfo(*info);
        return .{}, error;
    }

    return info, .{};
}

free_conninfo :: (info: *Conninfo) {
    for info.values free(it);

    array_free(info.values);
    array_free(info.is_set);
    info.* = .{};
}

// Whether `text` is something a program should hand to `parse_conninfo`: a URI or a list with
// an `=` in it. psql and libpq use this to tell a connection string from a database name.
is_conninfo :: (text: string) -> bool {
    return conninfo_uri_prefix(text) > 0 || contains(text, "=");
}

// The value of `keyword` and whether it was set.
conninfo_get :: (info: Conninfo, keyword: string) -> string, bool {
    index := conninfo_index(keyword);
    if index < 0 || !info.is_set[index] return "", false;

    return info.values[index], true;
}

// Sets `keyword` to a copy of `value`, replacing what was there. False for a keyword libpq does
// not know.
conninfo_set :: (info: *Conninfo, keyword: string, value: string) -> bool {
    index := conninfo_index(keyword);
    if index < 0 return false;

    free(info.values[index]);
    info.values[index] = copy_string(value);
    info.is_set[index] = true;

    return true;
}

#scope_module

conninfo_index :: (keyword: string) -> s64 {
    for CONNINFO_KEYWORDS if it == keyword return it_index;
    return -1;
}

conninfo_uri_prefix :: (text: string) -> s64 {
    if begins_with(text, "postgresql://") return 13;
    if begins_with(text, "postgres://") return 11;
    return 0;
}

// A percent-encoded value. libpq reports the whole value when one escape is wrong.
uri_decode :: (text: string) -> string, Pg_Error {
    out: [..] u8;
    i := 0;

    while i < text.count {
        c := text[i];

        if c != #char "%" {
            array_add(*out, c);
            i += 1;
            continue;
        }

        high, low := -1, -1;

        if i + 2 < text.count {
            high = hex_value(text[i + 1]);
            low = hex_value(text[i + 2]);
        }

        if high < 0 || low < 0 {
            array_free(out);
            return "", make_error(.CONFIG, "invalid percent-encoded token: \"%\"", text);
        }

        byte := cast(u8) (high * 16 + low);

        if byte == 0 {
            array_free(out);
            return "", make_error(.CONFIG, "forbidden value % in percent-encoded value: \"%\"", "%00", text);
        }

        array_add(*out, byte);
        i += 3;
    }

    return cast(string) out, .{};
}

// Stores a percent-encoded value.
set_decoded :: (info: *Conninfo, keyword: string, encoded: string) -> Pg_Error {
    value, error := uri_decode(encoded);
    if error.kind != .NONE return error;

    defer free(value);
    conninfo_set(info, keyword, value);

    return .{};
}

parse_uri_options :: (info: *Conninfo, uri: string) -> Pg_Error {
    count := uri.count;
    start := conninfo_uri_prefix(uri);
    at := start;

    while at < count && uri[at] != #char "@" && uri[at] != #char "/" at += 1;

    if at < count && uri[at] == #char "@" {
        // user@ or user:password@
        end := start;
        while uri[end] != #char ":" && uri[end] != #char "@" end += 1;

        user := slice(uri, start, end - start);
        password: string;

        if uri[end] == #char ":" {
            password_start := end + 1;
            end = password_start;
            while uri[end] != #char "@" end += 1;

            password = slice(uri, password_start, end - password_start);
        }

        if user {
            error := set_decoded(info, "user", user);
            if error.kind != .NONE return error;
        }

        if password {
            error := set_decoded(info, "password", password);
            if error.kind != .NONE return error;
        }

        start = end + 1;
    }

    at = start;
    hosts: [..] string;
    ports: [..] string;

    hosts.allocator = temp;
    ports.allocator = temp;
    terminator: u8;

    // One or more hosts, each with an optional port, separated by commas.
    while true {
        host: string;

        if at < count && uri[at] == #char "[" {
            host_start := at + 1;

            at = host_start;
            while at < count && uri[at] != #char "]" at += 1;

            if at >= count {
                return make_error(.CONFIG, "end of string reached when looking for matching \"]\" in IPv6 host address in URI: \"%\"", uri);
            }

            if at == host_start {
                return make_error(.CONFIG, "IPv6 host address may not be empty in URI: \"%\"", uri);
            }

            host = slice(uri, host_start, at - host_start);
            at += 1;

            if at < count && uri[at] != #char ":" && uri[at] != #char "/" && uri[at] != #char "?" && uri[at] != #char "," {
                return make_error(.CONFIG, "unexpected character \"%\" at position % in URI (expected \":\" or \"/\"): \"%\"", string.{ 1, *uri[at] }, at + 1, uri);
            }
        } else {
            host_start := at;
            while at < count && uri[at] != #char ":" && uri[at] != #char "/" && uri[at] != #char "?" && uri[at] != #char "," at += 1;

            host = slice(uri, host_start, at - host_start);
        }

        terminator = ifx at < count then uri[at] else 0;
        port: string;

        if terminator == #char ":" {
            at += 1;
            port_start := at;

            while at < count && uri[at] != #char "/" && uri[at] != #char "?" && uri[at] != #char "," at += 1;

            port = slice(uri, port_start, at - port_start);
            terminator = ifx at < count then uri[at] else 0;
        }

        array_add(*hosts, host);
        array_add(*ports, port);

        if terminator != #char "," break;

        at += 1;
    }

    if terminator != 0 && terminator != #char "?" {
        at += 1;
        database_start := at;

        while at < count && uri[at] != #char "?" at += 1;

        database := slice(uri, database_start, at - database_start);

        terminator = ifx at < count then uri[at] else 0;

        if database {
            error := set_decoded(info, "dbname", database);
            if error.kind != .NONE return error;
        }
    }

    // `?host=` and `?port=` win over the host and port of the authority, as in libpq:
    // `postgres://localhost/db?host=/tmp` uses the socket in /tmp.
    host_list := comma_separated(hosts);
    port_list := comma_separated(ports);

    if host_list {
        error := set_decoded(info, "host", host_list);
        if error.kind != .NONE return error;
    }

    if port_list {
        error := set_decoded(info, "port", port_list);
        if error.kind != .NONE return error;
    }

    if terminator != 0 {
        error := parse_uri_parameters(info, advance(uri, at + 1));
        if error.kind != .NONE return error;
    }

    return .{};
}

comma_separated :: (items: [] string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for items {
        if it_index > 0 append(*builder, ",");
        append(*builder, it);
    }

    return builder_to_string(*builder,, temp);
}

// Spaces around a query parameter's name or value are dropped; one inside is an error (PostgreSQL
// 18), because a space belongs in the URI as %20.
uri_trim_spaces :: (text: string) -> string, Pg_Error {
    first := 0;
    last := text.count;

    while first < last && text[first] == #char " " first += 1;
    while last > first && text[last - 1] == #char " " last -= 1;

    trimmed := slice(text, first, last - first);

    if contains(trimmed, " ") {
        return "", make_error(.CONFIG, "unexpected spaces found in \"%\", use percent-encoded spaces (\%20) instead", text);
    }

    return trimmed, .{};
}

parse_uri_parameters :: (info: *Conninfo, parameters: string) -> Pg_Error {
    rest := parameters;

    while rest {
        keyword_end := 0;
        value_start := -1;
        at := 0;
        next := rest.count;

        while at < rest.count {
            c := rest[at];

            if c == #char "=" {
                if value_start >= 0 {
                    return make_error(.CONFIG, "extra key/value separator \"=\" in URI query parameter: \"%\"", slice(rest, 0, keyword_end));
                }

                keyword_end = at;
                value_start = at + 1;
            } else if c == #char "&" {
                next = at + 1;
                break;
            }

            at += 1;
        }

        value_end := ifx next < rest.count then next - 1 else rest.count;

        if value_start < 0 {
            return make_error(.CONFIG, "missing key/value separator \"=\" in URI query parameter: \"%\"", slice(rest, 0, value_end));
        }

        raw_keyword, error := uri_trim_spaces(slice(rest, 0, keyword_end));
        if error.kind != .NONE return error;

        raw_value:, error = uri_trim_spaces(slice(rest, value_start, value_end - value_start));

        if error.kind != .NONE return error;

        keyword:, error = uri_decode(raw_keyword);

        if error.kind != .NONE return error;

        defer free(keyword);

        value:, error = uri_decode(raw_value);

        if error.kind != .NONE return error;

        defer free(value);

        // The JDBC spelling of sslmode=require.
        if keyword == "ssl" && value == "true" {
            conninfo_set(info, "sslmode", "require");
        } else if !conninfo_set(info, keyword, value) {
            return make_error(.CONFIG, "invalid URI query parameter: \"%\"", keyword);
        }

        rest = advance(rest, next);
    }

    return .{};
}

parse_keyword_options :: (info: *Conninfo, text: string) -> Pg_Error {
    at := 0;
    count := text.count;

    while true {
        while at < count && is_space(text[at]) at += 1;
        if at >= count break;

        name_start := at;

        while at < count && text[at] != #char "=" && !is_space(text[at]) at += 1;

        name := slice(text, name_start, at - name_start);

        while at < count && is_space(text[at]) at += 1;

        if at >= count || text[at] != #char "=" {
            return make_error(.CONFIG, "missing \"=\" after \"%\" in connection info string", name);
        }

        at += 1;

        while at < count && is_space(text[at]) at += 1;

        value: [..] u8;
        value.allocator = temp;

        if at < count && text[at] == #char "'" {
            at += 1;

            while true {
                if at >= count return make_error(.CONFIG, "unterminated quoted string in connection info string");

                c := text[at];

                if c == #char "\\" {
                    at += 1;

                    if at < count {
                        array_add(*value, text[at]);
                        at += 1;
                    }
                } else if c == #char "'" {
                    at += 1;
                    break;
                } else {
                    array_add(*value, c);
                    at += 1;
                }
            }
        } else {
            while at < count && !is_space(text[at]) {
                if text[at] == #char "\\" {
                    at += 1;
                    if at >= count break;
                }

                array_add(*value, text[at]);
                at += 1;
            }
        }

        if !conninfo_set(info, name, cast(string) value) {
            return make_error(.CONFIG, "invalid connection option \"%\"", name);
        }
    }

    return .{};
}
