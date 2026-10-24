// A connection: one server session, used by one fiber at a time.

Transaction_Status :: enum u8 {
    IDLE; // outside a transaction block
    IN_TRANSACTION; // inside BEGIN ... COMMIT
    FAILED; // inside a transaction block that failed: only ROLLBACK works
}

Parameter :: struct {
    name: string;
    value: string;
}

// Called for each NoticeResponse (warnings, RAISE NOTICE). The notice is freed after the call.
Notice_Proc :: #type (connection: *Connection, notice: *Pg_Error);

Connection :: struct {
    status: Transaction_Status;
    backend_pid: s32;
    // From `server_version`: 170011 for 17.11, 150004 for 15.4.
    server_version: s64;
    // What the server reported with ParameterStatus: server_version, TimeZone, client_encoding,
    // standard_conforming_strings, ... (`parameter(connection, name)` looks one up).
    parameters: [..] Parameter;
    on_notice: Notice_Proc = log_notice;
    user_data: *void; // for the program: on_notice can find its state here
    // The minor protocol version in use: 2 (PostgreSQL 18 and later) or 0.
    protocol_minor: u32;
    tls_used: bool;
    // Authentication used SCRAM-SHA-256-PLUS: the password exchange is bound to this TLS session.
    channel_binding_used: bool;

    // Set when the connection can no longer be used: a failed read or write, a protocol error, or
    // a FATAL error from the server.
    broken: bool;
    busy: bool; // a request is in flight
    pool: *Pool; // the pool it belongs to, if any

    // Below: the client's own state.
    host: string;
    port: u16;
    unix_socket: bool;
    ssl_direct: bool;
    cancel_timeout: s64;
    opened_at: s64; // monotonic ms, for a pool's max_lifetime
    cancel_key: [] u8;
    tcp: Io.Tcp_Stream;
    reader: Io.Reader;
    writer: Io.Writer;
    out: [..] u8;
    message_start: s64;
    input: [..] u8;
    input_start: s64;
    #if TLS {
        tls: *Tls.Tls_Stream;
        tls_config: Tls.Tls_Config;
        tls_config_ready: bool;
    }
}

// Opens a connection and authenticates. On failure the connection is null and nothing needs
// closing. Must run on a fiber (`Io.io_run`); `config.connect_timeout` bounds the whole of it.
connect :: (config: Config) -> *Connection, Pg_Error {
    if !Io.in_scheduler() {
        return null, make_error(.CONFIG, "Postgres calls must run on a fiber: call them inside Io.io_run");
    }

    c := New(Connection);
    error: Pg_Error;

    Io.with_deadline(config.connect_timeout, #code {
        error = open(c, config);
    });

    if error.kind != .NONE {
        close(c, now = true);
        return null, error;
    }

    return c, .{};
}

// Ends the session (Terminate) and frees the connection. `now` skips the goodbye.
close :: (c: *Connection, now := false) {
    if !c return;

    if !now && !c.broken && c.reader.read_proc {
        begin_message(c, #char "X");
        end_message(c);
        Io.with_deadline(1000, #code {
            send(c);
        });
    }

    close_transport(c, now);

    for c.parameters {
        free(it.name);
        free(it.value);
    }

    array_free(c.parameters);
    array_free(c.out);
    array_free(c.input);
    free(c.cancel_key.data);
    free(c.host);
    free(c);
}

// A ParameterStatus value, or "" if the server did not report it.
parameter :: (c: *Connection, name: string) -> string {
    for c.parameters if it.name == name return it.value;
    return "";
}

log_notice :: (c: *Connection, notice: *Pg_Error) {
    log("postgres %: %", notice.severity, notice.message);
}

#scope_module

close_transport :: (c: *Connection, now: bool) {
    #if TLS {
        if c.tls {
            Tls.close(c.tls); // closes the TCP stream under it
            c.tls = null;
            c.reader = .{};
            c.writer = .{};
        }

        if c.tls_config_ready {
            Tls.tls_config_free(*c.tls_config);
            c.tls_config_ready = false;
        }
    }

    if c.reader.read_proc {
        Io.close(c.tcp, now);
        c.reader = .{};
        c.writer = .{};
    }
}

set_parameter :: (c: *Connection, name: string, value: string) {
    for *c.parameters {
        if it.name == name {
            free(it.value);
            it.value = copy_string(value);
            return;
        }
    }

    array_add(*c.parameters, .{ copy_string(name), copy_string(value) });
    if name == "server_version" c.server_version = parse_server_version(value);
}

VERSION_MAJOR :: 10_000;

// "17.11 (Homebrew)" is 170011; "9.6.24" would be 90624.
parse_server_version :: (text: string) -> s64 {
    major, ok, rest := string_to_int(text);
    if !ok return 0;

    minor := 0;

    if begins_with(rest, ".") {
        minor, ok, rest = string_to_int(advance(rest, 1));
        if major < 10 && begins_with(rest, ".") {
            patch: s64;
            patch, ok = string_to_int(advance(rest, 1));
            return major * VERSION_MAJOR + minor * 100 + patch;
        }
    }

    return major * VERSION_MAJOR + minor;
}

// Messages that may arrive at any time: ParameterStatus, NoticeResponse and (later)
// NotificationResponse. Returns true when the message was one of them.
handle_async :: (c: *Connection, type: u8, body: [] u8) -> bool {
    if type == {
        case #char "S";
            r := Cursor.{ data = body };
            name := get_cstring(*r);
            value := get_cstring(*r);

            if !r.bad set_parameter(c, name, value);

            return true;
        case #char "N";
            notice := parse_server_error(body);
            if c.on_notice c.on_notice(c, *notice);

            free_error(*notice);

            return true;
        case #char "A";
            return true; // LISTEN/NOTIFY: not routed yet
    }

    return false;
}
