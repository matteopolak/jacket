// A connection: one server session, used by one fiber at a time.

// A minor version of protocol 3, for `Config.min_protocol_version` and `max_protocol_version`.
// Version 3.2 (PostgreSQL 18 and later) allows cancel keys of up to 256 bytes, where 3.0 has four.
Protocol_Version :: enum u8 {
    V3_0;
    V3_2;
    LATEST; // the newest this client speaks: 3.2
}

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

// Called for each NotificationResponse (a NOTIFY on a channel the connection LISTENs to), in the
// middle of whatever request is reading. The notification is freed after the call. Without one,
// notifications queue on the connection (`next_notification`, `wait_for_notification`).
Notification_Proc :: #type (connection: *Connection, notification: *Notification);

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
    on_notification: Notification_Proc;
    // Notifications received and not yet taken, oldest first, when `on_notification` is not set.
    notifications: [..] Notification;
    on_trace: Trace_Proc; // `set_trace`
    trace_data: *void;
    // The minor protocol version in use: 2 (PostgreSQL 18 and later) or 0.
    protocol_minor: u32;
    protocol_requested: u32; // the minor version the startup message asked for
    protocol_required: u32; // the lowest minor version `min_protocol_version` accepts
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
    // The name the server's certificate is for (the host, not the address connected to), the name
    // sent as SNI ("" for none), whether verify-full checks the name, and whether the handshake
    // sent a client certificate.
    tls_name: string;
    tls_sni: string;
    tls_check_name: bool;
    tls_cert_sent: bool;
    port: u16;
    unix_socket: bool;
    ssl_direct: bool;
    cancel_timeout: s64;
    opened_at: s64; // monotonic ms, for a pool's max_lifetime
    cancel_key: [] u8;
    json_options: Json_Options; // copied to each result
    tcp: Io.Tcp_Stream;
    reader: Io.Reader;
    writer: Io.Writer;
    out: [..] u8;
    message_start: s64;
    message_type: u8; // of the message being written; 0 for the startup messages
    trace_regress: bool;
    pipeline: Pipeline; // pipeline mode, when `pipeline.active`
    statements: Statement_Cache; // prepared statements, by query (statements.jai)
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
    if config.on_notice c.on_notice = config.on_notice;

    c.user_data = config.user_data;
    c.json_options = config.json_options;
    c.statements.mode = config.statement_cache;
    c.statements.capacity = config.statement_cache_size;
    c.statements.verify = config.verify_statements;
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

    free_pipeline(*c.pipeline);
    free_statement_cache(*c.statements);
    for *c.notifications free_notification(it);

    array_free(c.notifications);

    array_free(c.parameters);
    array_free(c.out);
    array_free(c.input);
    free(c.cancel_key.data);
    free(c.host);
    free(c.tls_name);
    free(c.tls_sni);
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

minor_of :: (version: Protocol_Version) -> u32 {
    if version == .V3_0 return 0;
    return 2;
}

// Reads the protocol range from the config. An older server answers a request for a newer
// protocol with NegotiateProtocolVersion; `min_protocol_version` is how far down it may go.
set_protocol_versions :: (c: *Connection, config: Config) -> Pg_Error {
    c.protocol_requested = minor_of(config.max_protocol_version);
    c.protocol_required = minor_of(config.min_protocol_version);

    if c.protocol_required > c.protocol_requested {
        return make_error(.CONFIG, "min_protocol_version is newer than max_protocol_version");
    }

    return .{};
}

// NegotiateProtocolVersion: the newest minor version the server speaks for major version 3, then
// the count and names of the protocol options it did not recognize (this client sends none).
negotiate_protocol_version :: (c: *Connection, r: *Cursor) -> Pg_Error {
    version := get_u32(r);

    if r.bad || version >> 16 != 3 || (version & 0xffff) > c.protocol_requested {
        return protocol_error(c, "an unexpected protocol version answer");
    }

    minor := version & 0xffff;

    if minor < c.protocol_required {
        c.broken = true;
        return make_error(.PROTOCOL, "the server speaks protocol version 3.%, and min_protocol_version asks for 3.%", minor, c.protocol_required);
    }

    c.protocol_minor = minor;

    return .{};
}

// Version 3.0 has four-byte cancel keys; 3.2 allows up to 256 bytes.
valid_cancel_key :: (c: *Connection, key: [] u8) -> bool {
    if c.protocol_minor < 2 return key.count == 4;
    return key.count >= 4 && key.count <= 256;
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

// Messages that may arrive at any time: ParameterStatus, NoticeResponse and NotificationResponse.
// Returns true when the message was one of them.
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
            r := Cursor.{ data = body };
            notification: Notification;

            notification.backend_pid = get_s32(*r);
            notification.channel = copy_string(get_cstring(*r));
            notification.payload = copy_string(get_cstring(*r));

            if r.bad {
                free_notification(*notification);
                return true; // a malformed one is dropped, like an unknown notice field
            }

            if c.on_notification {
                c.on_notification(c, *notification);
                free_notification(*notification);
            } else {
                array_add(*c.notifications, notification);
            }

            return true;
    }

    return false;
}
