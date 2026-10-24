// Authentication policy the application sets before connecting: libpq's `require_auth`, and the
// password file that supplies a password when the config has none.

Auth_Method :: enum_flags u32 {
    PASSWORD; // the server asked for the password in clear text
    MD5;
    GSS;
    SSPI;
    SCRAM_SHA_256; // SASL
    NONE; // the server accepted the login without asking for anything
}

// The methods `require_auth` allows. `restricted` is false when it is empty: anything goes.
Auth_Policy :: struct {
    restricted: bool;
    allowed: Auth_Method;
}

// Reads a `require_auth` value (libpq's syntax: see `Config.require_auth`). The errors are
// libpq's.
parse_require_auth :: (text: string) -> Auth_Policy, Pg_Error {
    policy: Auth_Policy;
    if !text return policy, .{};

    policy.restricted = true;
    listed: Auth_Method;
    negative := false;
    seen: [..] string;
    seen.allocator = temp;

    for token: split_list(text) {
        forbidden := begins_with(token, "!");
        name := ifx forbidden then advance(token, 1) else token;

        if it_index == 0 {
            negative = forbidden;
        } else if forbidden != negative {
            if negative return .{}, make_error(.CONFIG, "require_auth method \"%\" cannot be mixed with negative methods", token);
            return .{}, make_error(.CONFIG, "negative require_auth method \"%\" cannot be mixed with non-negative methods", token);
        }

        for seen if it == token return .{}, make_error(.CONFIG, "require_auth method \"%\" is specified more than once", token);

        array_add(*seen, token);

        method: Auth_Method;

        if name == {
            case "password"; method = .PASSWORD;
            case "md5"; method = .MD5;
            case "gss"; method = .GSS;
            case "sspi"; method = .SSPI;
            case "scram-sha-256"; method = .SCRAM_SHA_256;
            case "none"; method = .NONE;
            case; return .{}, make_error(.CONFIG, "invalid require_auth value: \"%\"", name);
        }

        listed |= method;
    }

    ALL :: Auth_Method.PASSWORD | .MD5 | .GSS | .SSPI | .SCRAM_SHA_256 | .NONE;

    policy.allowed = ifx negative then ALL & ~listed else listed;

    return policy, .{};
}

// Fails the login when the server's way of authenticating is not one `require_auth` allows.
// `code` is the AuthenticationRequest the server sent.
check_auth_method :: (c: *Connection, config: Config, policy: Auth_Policy, code: u32) -> Pg_Error {
    if !policy.restricted return .{};

    method: Auth_Method;
    reason: string;

    if code == {
        case AUTH_OK;
            method = .NONE;
            reason = "server did not complete authentication";
        case AUTH_CLEARTEXT;
            method = .PASSWORD;
            reason = "server requested a cleartext password";
        case AUTH_MD5;
            method = .MD5;
            reason = "server requested a hashed password";
        case AUTH_GSS; #through;
        case AUTH_GSS_CONTINUE;
            method = .GSS;
            reason = "server requested GSSAPI authentication";
        case AUTH_SSPI;
            method = .SSPI;
            reason = "server requested SSPI authentication";
        case AUTH_SASL; #through;
        case AUTH_SASL_CONTINUE; #through;
        case AUTH_SASL_FINAL;
            method = .SCRAM_SHA_256;
            reason = "server requested SASL authentication";
        case;
            return .{};
    }

    if policy.allowed & method return .{};

    return auth_error(c, tprint("authentication method requirement \"%\" failed: %", config.require_auth, reason));
}

// The password for a login from the password file, or an empty string. `host` is the name
// connected to; a socket directory or no name searches for `localhost`.
passfile_password :: (config: Config, host: string, port: u16, database: string, user: string) -> string {
    path := config.passfile;

    if !path path = environment_variable("PGPASSFILE");

    if !path {
        #if OS == .WINDOWS {
            base := environment_variable("APPDATA");
            if !base return "";

            path = tprint("%/postgresql/pgpass.conf", base);
        } else {
            home := environment_variable("HOME");
            if !home return "";

            path = tprint("%/.pgpass", home);
        }
    }

    #if OS != .WINDOWS {
        // A file other users can read is ignored.
        status: POSIX.stat_t;
        if POSIX.stat(temp_c_string(path), *status) != 0 return "";
        if (status.st_mode & 0x3F) != 0 return "";
    }

    contents, read := read_entire_file(path, log_errors = false);
    if !read return "";

    defer free(contents);

    name := ifx !host || begins_with(host, "/") then "localhost" else host;
    wanted := string.[name, tprint("%", port), database, user];
    rest := contents;

    while rest {
        found, line, after := split_from_left(rest, #char "\n");

        if !found {
            line = rest;
            after = "";
        }

        rest = after;

        while line.count && (line[line.count - 1] == #char "\r") line.count -= 1;

        if !line || line[0] == #char "#" continue;

        password, matched := passfile_line(line, wanted);
        if matched return copy_string(password,, temp);
    }

    return "";
}

#scope_module

// The password of a line `host:port:database:user:password` whose first four fields match.
passfile_line :: (line: string, wanted: [] string) -> string, bool {
    at := 0;

    for wanted {
        field_start := at;
        text: [..] u8;
        text.allocator = temp;

        while at < line.count && line[at] != #char ":" {
            if line[at] == #char "\\" && at + 1 < line.count at += 1;

            array_add(*text, line[at]);
            at += 1;
        }

        if at >= line.count return "", false;

        raw := slice(line, field_start, at - field_start);

        at += 1;

        if raw != "*" && cast(string) text != it return "", false;
    }

    // The password ends at the next colon that is not escaped: the rest of the line is ignored.
    password: [..] u8;
    password.allocator = temp;

    while at < line.count && line[at] != #char ":" {
        if line[at] == #char "\\" && at + 1 < line.count at += 1;

        array_add(*password, line[at]);
        at += 1;
    }

    return cast(string) password, true;
}
