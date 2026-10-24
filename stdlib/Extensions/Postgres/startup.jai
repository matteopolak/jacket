// Opening a session: the transport, TLS, the startup message and authentication.

PROTOCOL_3_0 :: 196608; // 3 << 16
PROTOCOL_3_2 :: 196610; // PostgreSQL 18 and later; older servers answer with version 3.0
SSL_REQUEST :: 80877103;
CANCEL_REQUEST :: 80877102;

// AuthenticationRequest codes.
AUTH_OK :: 0;
AUTH_CLEARTEXT :: 3;
AUTH_MD5 :: 5;
AUTH_GSS :: 7;
AUTH_GSS_CONTINUE :: 8;
AUTH_SSPI :: 9;
AUTH_SASL :: 10;
AUTH_SASL_CONTINUE :: 11;
AUTH_SASL_FINAL :: 12;

open :: (c: *Connection, config: Config) -> Pg_Error {
    user := config.user;
    if !user user = environment("USER");
    if !user user = environment("USERNAME");
    if !user return make_error(.CONFIG, "no user name: set Config.user");

    database := ifx config.database then config.database else user;
    if contains_zero(user) || contains_zero(database) || contains_zero(config.options) || contains_zero(config.application_name) {
        return make_error(.CONFIG, "startup parameters cannot hold zero bytes");
    }

    c.host = copy_string(config.host);
    c.port = config.port;
    error := open_transport(c);
    if error.kind != .NONE return error;

    error = negotiate_tls(c, config);
    if error.kind != .NONE return error;

    begin_message(c, 0);
    put_u32(*c.out, PROTOCOL_3_2);
    put_cstring(*c.out, "user");
    put_cstring(*c.out, user);
    put_cstring(*c.out, "database");
    put_cstring(*c.out, database);

    if config.application_name {
        put_cstring(*c.out, "application_name");
        put_cstring(*c.out, config.application_name);
    }

    put_cstring(*c.out, "client_encoding");
    put_cstring(*c.out, "UTF8");
    put_cstring(*c.out, "DateStyle");
    put_cstring(*c.out, "ISO");
    put_cstring(*c.out, "IntervalStyle");
    put_cstring(*c.out, "postgres");

    if config.options {
        put_cstring(*c.out, "options");
        put_cstring(*c.out, config.options);
    }

    put_u8(*c.out, 0);
    end_message(c);
    c.protocol_minor = 2;
    error = send(c);
    if error.kind != .NONE return error;

    return authenticate(c, config, user);
}

open_transport :: (c: *Connection) -> Pg_Error {
    io: Io.Io_Error;

    if begins_with(c.host, "/") {
        c.unix_socket = true;
        path := tprint("%/.s.PGSQL.%", c.host, c.port);
        c.tcp, io = Io.connect_unix(path);
        if io != .NONE return io_failure(io, tprint("could not connect to %", path));
    } else {
        target := ifx contains(c.host, ":") then tprint("[%]:%", c.host, c.port) else tprint("%:%", c.host, c.port);
        c.tcp, io = Io.connect(target);
        if io != .NONE return io_failure(io, tprint("could not connect to %", target));
    }

    c.reader = Io.reader_of(c.tcp);
    c.writer = Io.writer_of(c.tcp);

    return .{};
}

negotiate_tls :: (c: *Connection, config: Config) -> Pg_Error {
    if config.ssl_mode == .DISABLE || c.unix_socket return .{};

    #if !TLS {
        if config.ssl_mode == .PREFER return .{};
        return make_error(.CONFIG, "sslmode needs TLS, and this program imports Extensions/Postgres with TLS = false");
    } else {
        if config.ssl_direct {
            if config.ssl_mode == .PREFER {
                return make_error(.CONFIG, "sslnegotiation=direct needs sslmode=require or stricter");
            }

            return start_tls(c, config);
        }

        begin_message(c, 0);
        put_u32(*c.out, SSL_REQUEST);
        end_message(c);
        error := send(c);
        if error.kind != .NONE return error;

        answer: [1] u8;
        _, io := Io.read_full(c.reader, answer);

        if io != .NONE {
            c.broken = true;
            return io_failure(io, "the server did not answer the TLS request");
        }

        if answer[0] == {
            case #char "S";
                return start_tls(c, config);
            case #char "N";
                if config.ssl_mode == .PREFER return .{};

                return make_error(.TLS, "the server does not accept TLS connections, and sslmode is %", config.ssl_mode);
        }

        c.broken = true;

        return make_error(.PROTOCOL, "the server answered the TLS request with %", answer[0]);
    }
}

#if TLS {
    start_tls :: (c: *Connection, config: Config) -> Pg_Error {
        c.tls_config.alpn = string.["postgresql"];
        c.tls_config.roots_pem = config.ssl_root_cert_pem;
        c.tls_config.verify = config.ssl_mode == .VERIFY_CA || config.ssl_mode == .VERIFY_FULL;
        tls_error := Tls.tls_config_init(*c.tls_config);
        if tls_error.kind != .NONE return make_error(.TLS, Tls.error_text(tls_error));

        c.tls_config_ready = true;

        name := ifx c.tls_config.verify then c.host else "";
        stream: *Tls.Tls_Stream;
        stream, tls_error = Tls.tls_connect(*c.tls_config, c.reader, c.writer, name);
        if tls_error.kind != .NONE {
            c.broken = true;
            if tls_error.kind == .IO return io_failure(tls_error.io, Tls.error_text(tls_error));
            return make_error(.TLS, Tls.error_text(tls_error));
        }

        c.tls = stream;
        c.tls_used = true;
        c.reader = Tls.reader_of(stream);
        c.writer = Tls.writer_of(stream);

        return .{};
    }
}

authenticate :: (c: *Connection, config: Config, user: string) -> Pg_Error {
    scram: Scram;

    defer free_scram(*scram);
    authenticated := false;

    while true {
        type, body, error := read_message(c);
        if error.kind != .NONE return error;
        if handle_async(c, type, body) continue;

        r := Cursor.{ data = body };

        if type == {
            case #char "R";
                code := get_u32(*r);

                if r.bad return protocol_error(c, "a short authentication request");

                if code == AUTH_OK {
                    if scram.started && !scram.verified {
                        return auth_error(c, "the server accepted the login without proving it knows the password");
                    }
                    if config.channel_binding == .REQUIRE && !scram.plus {
                        return auth_error(c, "channel_binding=require, but the server did not use SCRAM-SHA-256-PLUS");
                    }

                    authenticated = true;
                    c.channel_binding_used = scram.plus;
                    continue;
                }

                error = answer_authentication(c, config, user, code, *r, *scram);
                if error.kind != .NONE return error;

            case #char "v";
                // NegotiateProtocolVersion: the newest protocol version the server speaks for
                // major version 3, then the protocol options it did not recognize.
                version := get_u32(*r);
                if r.bad || version >> 16 != 3 || (version & 0xffff) > 2 {
                    return protocol_error(c, "an unexpected protocol version answer");
                }

                c.protocol_minor = version & 0xffff;

            case #char "K";
                c.backend_pid = get_s32(*r);
                key := rest(*r);
                if r.bad || key.count < 4 || key.count > 256 return protocol_error(c, "bad BackendKeyData");

                free(c.cancel_key.data);
                c.cancel_key = array_copy(key);

            case #char "E";
                c.broken = true;
                return parse_server_error(body);

            case #char "Z";
                if !authenticated return protocol_error(c, "ReadyForQuery before authentication finished");

                status := get_u8(*r);

                c.status = transaction_status(status);

                return .{};

            case;
                return protocol_error(c, tprint("unexpected message % while connecting", cast(string) u8.[type]));
        }
    }

    return .{};
}

answer_authentication :: (c: *Connection, config: Config, user: string, code: u32, r: *Cursor, scram: *Scram) -> Pg_Error {
    if config.channel_binding == .REQUIRE && (code == AUTH_CLEARTEXT || code == AUTH_MD5) {
        return auth_error(c, "channel_binding=require, but the server asked for a password without SCRAM");
    }

    if code == {
        case AUTH_CLEARTEXT;
            if !c.tls_used && !config.allow_cleartext_password {
                return auth_error(c, "the server asked for the password in clear text without TLS (set allow_cleartext_password to allow it)");
            }

            if !config.password return auth_error(c, "the server asked for a password, and none was given");

            begin_message(c, #char "p");
            put_cstring(*c.out, config.password);
            end_message(c);

            return send(c);

        case AUTH_MD5;
            salt := get_bytes(r, 4);

            if r.bad return protocol_error(c, "a short MD5 request");
            if !config.password return auth_error(c, "the server asked for a password, and none was given");

            inner := Md5.md5(join(config.password, user,, temp));

            defer free(inner);
            outer := Md5.md5(join(inner, cast(string) salt,, temp));

            defer free(outer);
            begin_message(c, #char "p");
            put_bytes(*c.out, "md5");
            put_cstring(*c.out, outer);
            end_message(c);

            return send(c);

        case AUTH_SASL;
            if !config.password return auth_error(c, "the server asked for a password, and none was given");

            mechanisms: [..] string;
            mechanisms.allocator = temp;

            while true {
                name := get_cstring(r);
                if r.bad || !name break;
                array_add(*mechanisms, name);
            }

            if r.bad return protocol_error(c, "a bad SASL mechanism list");

            return scram_start(c, config, scram, mechanisms);

        case AUTH_SASL_CONTINUE;
            return scram_continue(c, config, scram, rest(r));

        case AUTH_SASL_FINAL;
            return scram_finish(c, scram, rest(r));

        case AUTH_GSS; #through;
        case AUTH_GSS_CONTINUE; #through;
        case AUTH_SSPI;
            return auth_error(c, "the server asked for GSSAPI or SSPI authentication, which this client does not support");
    }

    return auth_error(c, tprint("the server asked for an unknown authentication method %", code));
}

transaction_status :: (byte: u8) -> Transaction_Status {
    if byte == #char "T" return .IN_TRANSACTION;
    if byte == #char "E" return .FAILED;
    return .IDLE;
}

protocol_error :: (c: *Connection, what: string) -> Pg_Error {
    c.broken = true;
    return make_error(.PROTOCOL, what);
}

auth_error :: (c: *Connection, what: string) -> Pg_Error {
    c.broken = true;
    return make_error(.AUTH, what);
}

contains_zero :: (text: string) -> bool {
    for text if it == 0 return true;
    return false;
}

#if OS == .WINDOWS {
    kernel32_pg :: #system_library "kernel32";
    GetEnvironmentVariableA :: (name: *u8, buffer: *u8, size: u32) -> u32 #foreign kernel32_pg;

    environment :: (name: string) -> string {
        buffer: [512] u8;
        count := GetEnvironmentVariableA(temp_c_string(name), buffer.data, buffer.count);
        if count == 0 || count >= buffer.count return "";
        return copy_string(string.{ count, buffer.data },, temp);
    }
} else {
    libc_pg :: #system_library "libc";
    getenv_pg :: (name: *u8) -> *u8 #foreign libc_pg "getenv";

    environment :: (name: string) -> string {
        value := getenv_pg(temp_c_string(name));
        if !value return "";
        return copy_string(to_string(value),, temp);
    }
}
