// A psql stand-in built on this Postgres client, so that PostgreSQL's own test suites (which
// drive the server through psql) exercise the client instead of libpq. It reproduces psql's
// observable behaviour for what those suites use: see docs/postgres/conformance.md.
//
// The shim follows psql's sources (its catalog queries and messages are PostgreSQL's), which are
// under the PostgreSQL License: see COPYRIGHT.postgresql in this directory.
//
//     jaic psql.jai -exe psql && pg_regress --bindir=<this directory> ...

#import "Basic";
#import "String";
#import "File";
POSIX :: #import "POSIX";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";

#load "scan.jai";
#load "print.jai";
#load "describe.jai";

EXIT_SUCCESS :: 0;
EXIT_FAILURE :: 1;
EXIT_BADCONN :: 2;
EXIT_USER :: 3;

Verbosity :: enum u8 {
    DEFAULT;
    TERSE;
    VERBOSE;
    SQLSTATE;
}

Show_Context :: enum u8 {
    NEVER;
    ERRORS;
    ALWAYS;
}

Echo :: enum u8 {
    NONE;
    ERRORS;
    QUERIES;
    ALL;
}

Error_Rollback :: enum u8 {
    OFF;
    ON;
    INTERACTIVE;
}

// What psql calls `pset`: settings, the connection and the input being read.
State :: struct {
    db: *Pg.Connection;
    config: Pg.Config;
    parameters: [..] Pg.Startup_Parameter;

    vars: Variables;
    popt: Print_Options;

    quiet: bool;
    echo: Echo;
    on_error_stop: bool;
    autocommit := true;
    on_error_rollback: Error_Rollback;
    verbosity: Verbosity;
    show_context := Show_Context.ERRORS;
    show_all_results := true;
    hide_tableam: bool;
    hide_toast_compression: bool;

    // The input being read: a whole file or stdin, consumed line by line. COPY FROM STDIN data
    // comes from the same place.
    input: string;
    input_at: s64;
    lineno: s64;
    input_name: string; // the script file being read; empty for standard input and -c
    stdin_open: bool; // the input is standard input, with more to read (fill_input)
    interactive: bool; // standard input and output are a terminal: prompts are shown
    continuation: bool; // the next line continues a query
    password_prompt: string; // asked for by \password instead of the usual prompt

    // Set by \g-family commands for the next query only.
    gset_prefix: string;
    gset_active: bool;
    gexec: bool;
    gdesc: bool;
    fetch_count: s64;
    crosstab: bool;
    crosstab_args: [4] string;
    bind_params: [..] string;
    bind_active: bool;
    gfile: string;
    gpipe: string; // \g | command: gfile holds the output until the command reads it
    saved_popt: Print_Options;
    restore_popt: bool;

    timing: bool;
    restricted: bool; // \restrict is in force
    restrict_key: string;
    single_transaction: bool;

    last_error: Pg.Pg_Error; // for \errverbose
    last_error_query: string;
}

psql: State;

environment :: (name: string) -> string {
    value := POSIX.getenv(temp_c_string(name));
    if !value return "";
    return to_string(value);
}

errno_value :: () -> s64 {
    return cast(s64) POSIX.__errno_location().*;
}

exit_code := EXIT_SUCCESS;

main :: () {
    Io.io_run(() {
        // SIGINT cancels the running statement, as in psql.
        Io.catch_interrupts();
        Io.spawn(watch_interrupts);
        exit_code = run(get_command_line_arguments());
    });

    exit(cast(s32) exit_code);
}

// Turns SIGINT into a cancel request for the statement being run. The signal only sets a flag,
// which this fiber polls.
watch_interrupts :: () {
    while Io.sleep(10) == .NONE {
        if !Io.interrupted() continue;

        Io.clear_interrupt();
        if psql.db && psql.db.busy Pg.cancel_query(psql.db);
    }
}

// Startup

Startup_Action :: struct {
    is_file: bool;
    text: string; // a -c command, or a -f file name
}

run :: (args: [] string) -> s64 {
    actions: [..] Startup_Action;
    dbname := environment("PGDATABASE");
    user := environment("PGUSER");
    host := environment("PGHOST");
    port := environment("PGPORT");
    positional := 0;
    pending_vars: [..] string;

    init_variables();

    i := 1;

    next_value :: () -> string #expand {
        if i + 1 >= args.count {
            print_error(tprint("option requires an argument -- '%'", args[i]));
            `return EXIT_FAILURE;
        }

        i += 1;

        return args[i];
    }

    while i < args.count {
        arg := args[i];

        if begins_with(arg, "--") {
            if arg == {
                case "--no-psqlrc";
                case "--help"; return print_help("options");
                case "--version"; return print_version();
                case "--single-transaction"; psql.single_transaction = true;
                case "--quiet"; set_variable("QUIET", "on");
                case "--echo-all"; set_variable("ECHO", "all");
                case "--echo-queries"; set_variable("ECHO", "queries");
                case "--no-align"; psql.popt.format = .UNALIGNED;
                case "--tuples-only"; psql.popt.tuples_only = true;
                case "--no-password";
                case "--csv"; psql.popt.format = .CSV;
                case;
                    if begins_with(arg, "--help=") return print_help(advance(arg, 7));
                    else if begins_with(arg, "--set=") array_add(*pending_vars, advance(arg, 6));
                    else if begins_with(arg, "--variable=") array_add(*pending_vars, advance(arg, 11));
                    else if begins_with(arg, "--dbname=") dbname = advance(arg, 9);
                    else if begins_with(arg, "--host=") host = advance(arg, 7);
                    else if begins_with(arg, "--port=") port = advance(arg, 7);
                    else if begins_with(arg, "--username=") user = advance(arg, 11);
                    else if begins_with(arg, "--command=") array_add(*actions, .{ text = advance(arg, 10) });
                    else if begins_with(arg, "--file=") array_add(*actions, .{ is_file = true, text = advance(arg, 7) });
                    else {
                        print_error(tprint("unrecognized option '%'", arg));
                        return EXIT_FAILURE;
                    }
            }
        } else if arg == "-V" {
            return print_version();
        } else if arg == "-?" {
            return print_help("options");
        } else if begins_with(arg, "-") && arg.count > 1 {
            // Short options may be bundled: -XAtq.
            j := 1;
            while j < arg.count {
                flag := arg[j];

                j += 1;
                rest := advance(arg, j);
                takes_value := false;
                value: string;

                if flag == #char "c" || flag == #char "d" || flag == #char "f" || flag == #char "h" || flag == #char "p" || flag == #char "U" || flag == #char "v" || flag == #char "P" || flag == #char "F" || flag == #char "R" {
                    takes_value = true;
                    if rest value = rest;
                    else value = next_value();
                    j = arg.count;
                }

                if flag == {
                    case #char "X";
                    case #char "1"; psql.single_transaction = true;
                    case #char "a"; set_variable("ECHO", "all");
                    case #char "e"; set_variable("ECHO", "queries");
                    case #char "b"; set_variable("ECHO", "errors");
                    case #char "q"; set_variable("QUIET", "on");
                    case #char "A"; psql.popt.format = .UNALIGNED;
                    case #char "t"; psql.popt.tuples_only = true;
                    case #char "x"; psql.popt.expanded = .ON;
                    case #char "w";
                    case #char "W";
                    case #char "n";
                    case #char "c"; array_add(*actions, .{ text = value });
                    case #char "f"; array_add(*actions, .{ is_file = true, text = value });
                    case #char "d"; dbname = value;
                    case #char "h"; host = value;
                    case #char "p"; port = value;
                    case #char "U"; user = value;
                    case #char "v"; array_add(*pending_vars, value);
                    case #char "P"; apply_pset_option(value);
                    case #char "F"; psql.popt.field_separator = value;
                    case #char "R"; psql.popt.record_separator = value;
                    case;
                        print_error(tprint("invalid option -- '%'", cast(string) u8.[flag]));
                        return EXIT_FAILURE;
                }
            }
        } else {
            if positional == 0 dbname = arg;
            else if positional == 1 user = arg;
            positional += 1;
        }

        i += 1;
    }

    for pending_vars {
        found, name, value := split_from_left(it, #char "=");
        if !found {
            unset_variable(it);
        } else if !set_variable(name, value) {
            return EXIT_FAILURE;
        }
    }

    connection_string := "";

    if dbname && Pg.is_conninfo(dbname) {
        connection_string = dbname;
        dbname = "";
    }

    error := connect_to(connection_string, dbname, user, host, port, keep_previous = false);
    if error return EXIT_BADCONN;

    if !actions.count {
        psql.interactive = POSIX.isatty(0) == 1 && POSIX.isatty(1) == 1;
        return main_loop("", from_stdin = true);
    }

    if psql.single_transaction {
        error := Pg.execute_script(psql.db, "START TRANSACTION");

        if error.kind != .NONE {
            print_error_raw(build_error_message(error, "", true));
            Pg.free_error(*error);
            return EXIT_FAILURE;
        }
    }

    result := EXIT_SUCCESS;

    for actions {
        if it.is_file && it.text == "-" {
            result = main_loop("", input_name = "<stdin>", from_stdin = true);
        } else if it.is_file {
            text, ok := read_entire_file(it.text, log_errors = false);
            if !ok {
                print_error(tprint("%: No such file or directory", it.text));
                return EXIT_FAILURE;
            }

            result = main_loop(text, input_name = canonical_path(it.text));
        } else if begins_with(it.text, "\\") {
            result = main_loop(it.text, one_command = true);
        } else {
            if psql.echo == .ALL || psql.echo == .QUERIES {
                write_fd(1, it.text);
                write_fd(1, "\n");
            }

            ok := send_query(it.text, -1);

            result = ifx ok then EXIT_SUCCESS else EXIT_USER;
            if !ok && !psql.db return EXIT_BADCONN;
        }

        // Without ON_ERROR_STOP the next action runs after a failure and the last one decides.
        if result != EXIT_SUCCESS && psql.on_error_stop break;
    }

    if psql.single_transaction && psql.db {
        command := ifx psql.on_error_stop && result != EXIT_SUCCESS then "ROLLBACK" else "COMMIT";
        error := Pg.execute_script(psql.db, command);

        if error.kind != .NONE {
            print_error_raw(build_error_message(error, "", true));
            Pg.free_error(*error);
            return EXIT_FAILURE;
        }
    }

    return result;
}

init_variables :: () {
    defaults :: string.[
        "AUTOCOMMIT", "on",
        "COMP_KEYWORD_CASE", "preserve-upper",
        "ECHO", "none",
        "ECHO_HIDDEN", "off",
        "FETCH_COUNT", "0",
        "HIDE_TABLEAM", "off",
        "HIDE_TOAST_COMPRESSION", "off",
        "HISTCONTROL", "none",
        "HISTSIZE", "500",
        "IGNOREEOF", "0",
        "LAST_ERROR_MESSAGE", "",
        "LAST_ERROR_SQLSTATE", "00000",
        "ON_ERROR_ROLLBACK", "off",
        "ON_ERROR_STOP", "off",
        "PROMPT1", "%/%R%x%# ",
        "PROMPT2", "%/%R%x%# ",
        "PROMPT3", ">> ",
        "QUIET", "off",
        "SHOW_ALL_RESULTS", "on",
        "SHOW_CONTEXT", "errors",
        "SINGLELINE", "off",
        "SINGLESTEP", "off",
        "VERBOSITY", "default",
        "WATCH_INTERVAL", "2",
    ];

    i := 0;

    while i < defaults.count {
        variables_set(*psql.vars, defaults[i], defaults[i + 1]);
        i += 2;
    }
}

// Reads standard input until the unread part holds a whole line, or it ends. A query is sent as
// soon as its line has arrived, not when the input is complete.
fill_input :: () {
    if !psql.stdin_open return;

    chunk: [65536] u8;

    while true {
        end := psql.input_at;
        while end < psql.input.count && psql.input[end] != #char "\n" end += 1;
        if end < psql.input.count return;

        show_prompt();

        n := POSIX.read(0, chunk.data, chunk.count);

        if n < 0 {
            if errno_value() == POSIX.EINTR continue;
            psql.stdin_open = false;
            return;
        }

        if n == 0 {
            psql.stdin_open = false;
            return;
        }

        // The old buffer stays: lines already handed out point into it.
        psql.input = join(psql.input, string.{ n, chunk.data });
    }
}

// The prompt psql shows before it waits for a line of a terminal.
show_prompt :: () {
    if !psql.interactive return;

    text := psql.password_prompt;

    if !text {
        template, _ := variables_get(*psql.vars, ifx psql.continuation then "PROMPT2" else "PROMPT1");
        text = expand_prompt(template);
    }

    write_fd(1, text);
}

// The parts of a prompt template that psql's tests use: %/ %n %# %R %x and %%.
expand_prompt :: (template: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    i := 0;

    while i < template.count {
        c := template[i];

        i += 1;

        if c != #char "%" || i >= template.count {
            append(*builder, string.{ 1, *template[i - 1] });
            continue;
        }

        code := template[i];

        i += 1;

        if code == {
            case #char "/"; append(*builder, psql.config.database);
            case #char "n"; append(*builder, psql.config.user);
            case #char "#"; append(*builder, ifx Pg.parameter(psql.db, "is_superuser") == "on" then "#" else ">");
            case #char "R"; append(*builder, ifx psql.continuation then "-" else "=");
            case #char "x";
                if psql.db {
                    if psql.db.status == .IN_TRANSACTION append(*builder, "*");
                    else if psql.db.status == .FAILED append(*builder, "!");
                }
            case #char "%"; append(*builder, "%");
        }
    }

    return builder_to_string(*builder,, temp);
}

print_version :: () -> s64 {
    write_fd(1, "psql (PostgreSQL) 17.0 (jacket psql shim)\n");
    return EXIT_SUCCESS;
}

// The options and meta-commands this psql stand-in understands.
print_help :: (topic: string) -> s64 {
    if topic == {
        case "options"; #through;
        case "";
            write_fd(1, "psql is the PostgreSQL interactive terminal, here a stand-in built on the Jai client.\n\nUsage:\n  psql [OPTION]... [DBNAME [USERNAME]]\n\nGeneral options:\n  -c, --command=COMMAND    run only single command (SQL or internal) and exit\n  -d, --dbname=DBNAME      database name to connect to, or a connection string\n  -f, --file=FILENAME      execute commands from file, then exit\n  -1, --single-transaction execute as a single transaction\n  -V, --version            output version information, then exit\n  -X, --no-psqlrc          do not read startup file\n  -?, --help[=options]     show this help, then exit\n      --help=commands      list backslash commands, then exit\n      --help=variables     list special variables, then exit\n\nInput and output options:\n  -a, -b, -e, -q, -v NAME=VALUE\n\nOutput format options:\n  -A, -t, -x, -P, -F, -R, --csv\n\nConnection options:\n  -h, --host=HOSTNAME      database server host or socket directory\n  -p, --port=PORT          database server port\n  -U, --username=USERNAME  database user name\n  -w, --no-password        never prompt for password\n");
        case "commands";
            write_fd(1, "Backslash commands understood:\n  \\c, \\copy, \\d..., \\e (not supported), \\echo, \\encoding, \\errverbose, \\g, \\gdesc, \\gexec, \\gset, \\i, \\if, \\o, \\password, \\pset, \\q, \\restrict, \\set, \\timing, \\unrestrict, \\watch, \\!\n");
        case "variables";
            write_fd(1, "Special variables:\n  AUTOCOMMIT, COMP_KEYWORD_CASE, DBNAME, ECHO, ENCODING, ERROR, FETCH_COUNT, HOST, ON_ERROR_ROLLBACK, ON_ERROR_STOP, PORT, PROMPT1, PROMPT2, QUIET, ROW_COUNT, SHOW_ALL_RESULTS, SQLSTATE, USER, VERBOSITY\n");
        case;
            print_error(tprint("unrecognized help topic \"%\"", topic));
            return EXIT_FAILURE;
    }

    return EXIT_SUCCESS;
}

// Connecting

// Connects like psql does: the options of the command line (-h, -p, -U, -d), a connection string
// in place of the database name, which overrides them, and libpq's environment variables for
// whatever neither gave.
connect_to :: (connection_string: string, dbname: string, user: string, host: string, port: string, keep_previous: bool) -> failed: bool {
    info, _ := Pg.parse_conninfo_options("");

    defer Pg.free_conninfo(*info);

    if host Pg.conninfo_set(*info, "host", host);
    if port Pg.conninfo_set(*info, "port", port);
    if user Pg.conninfo_set(*info, "user", user);
    if dbname Pg.conninfo_set(*info, "dbname", dbname);

    if connection_string {
        parsed, error := Pg.parse_conninfo_options(connection_string);

        if error.kind != .NONE {
            print_error_raw(tprint("psql: error: %\n", error.message));
            Pg.free_error(*error);
            return true;
        }

        for Pg.CONNINFO_KEYWORDS {
            value, found := Pg.conninfo_get(parsed, it);
            if found Pg.conninfo_set(*info, it, value);
        }

        Pg.free_conninfo(*parsed);
    }

    Pg.conninfo_set(*info, "fallback_application_name", "psql");

    // Without a host, psql uses the Unix socket in the directory the server was built for.
    _, has_host := Pg.conninfo_get(info, "host");
    _, has_address := Pg.conninfo_get(info, "hostaddr");

    if !has_host && !has_address && !environment("PGHOST") && !environment("PGHOSTADDR") {
        Pg.conninfo_set(*info, "host", "/tmp");
    }

    config, config_error := Pg.config_from_conninfo(info, use_environment = true);

    if config_error.kind != .NONE {
        print_error_raw(tprint("psql: error: %\n", config_error.message));
        Pg.free_error(*config_error);
        return true;
    }

    _, has_timeout := Pg.conninfo_get(info, "connect_timeout");
    if !has_timeout && !environment("PGCONNECT_TIMEOUT") config.connect_timeout = -1;

    // libpq sends passwords in clear text whatever the transport.
    config.allow_cleartext_password = true;

    if !config.user config.user = environment("USER");
    if !config.database config.database = config.user;

    // The settings libpq takes from the environment as startup parameters, and the connection
    // string's own (client_encoding, replication).
    array_reset(*psql.parameters);

    ENVIRONMENT_PARAMETERS :: Pg.Startup_Parameter.[
        .{ "PGDATESTYLE", "DateStyle" },
        .{ "PGTZ", "TimeZone" },
        .{ "PGGEQO", "geqo" },
    ];

    for ENVIRONMENT_PARAMETERS {
        value := environment(it.name);
        if value array_add(*psql.parameters, .{ it.value, value });
    }

    has_encoding := false;

    for config.parameters {
        array_add(*psql.parameters, it);
        if it.name == "client_encoding" has_encoding = true;
    }

    if !has_encoding {
        encoding := environment("PGCLIENTENCODING");
        if !encoding encoding = locale_encoding();

        array_add(*psql.parameters, .{ "client_encoding", encoding });
    }

    config.parameters = psql.parameters;
    config.send_styles = false;
    config.on_notice = notice_processor;

    db, error := Pg.connect(config);
    if error.kind != .NONE {
        message := connection_error_text(config, error);

        if keep_previous && psql.db {
            print_error_raw(tprint("%\n", message));
            print_error_raw("Previous connection kept\n");
        } else {
            print_error_raw(tprint("psql: error: %\n", message));
        }

        Pg.free_error(*error);

        return true;
    }

    if psql.db Pg.close(psql.db);

    psql.db = db;
    psql.db.on_notice = notice_processor;
    psql.config = config;

    sync_connection_variables();

    return false;
}

// psql's client_encoding `auto`: the locale's character set.
locale_encoding :: () -> string {
    for string.["LC_ALL", "LC_CTYPE", "LANG"] {
        value := environment(it);
        if !value continue;

        lower := to_lower_copy(value,, temp);
        if contains(lower, "utf-8") || contains(lower, "utf8") return "UTF8";

        return "SQL_ASCII";
    }

    return "SQL_ASCII";
}

// What libpq makes of a failed connection: where it tried, then why. Mistakes in the settings
// have no place to mention.
connection_error_text :: (config: Pg.Config, error: Pg.Pg_Error) -> string {
    if error.kind == .CONFIG return error.message;

    where: string;

    if begins_with(config.host, "/") {
        where = tprint("connection to server on socket \"%/.s.PGSQL.%\" failed", config.host, config.port);
    } else if config.hostaddr && config.hostaddr != config.host {
        where = tprint("connection to server at \"%\" (%), port % failed", config.host, config.hostaddr, config.port);
    } else {
        where = tprint("connection to server at \"%\", port % failed", config.host, config.port);
    }

    if error.kind == .SERVER {
        return tprint("%: %", where, trim_right(build_error_message(error, "", false), "\n"));
    }

    if error.kind == .AUTH && begins_with(error.message, "the server asked for a password, and none was given") {
        return tprint("%: fe_sendauth: no password supplied", where);
    }

    return tprint("%: %", where, error.message);
}

sync_connection_variables :: () {
    c := psql.db;

    variables_set(*psql.vars, "DBNAME", psql.config.database);
    variables_set(*psql.vars, "USER", psql.config.user);
    variables_set(*psql.vars, "HOST", psql.config.host);
    variables_set(*psql.vars, "PORT", tprint("%", psql.config.port));
    variables_set(*psql.vars, "ENCODING", Pg.parameter(c, "client_encoding"));

    server_version := Pg.parameter(c, "server_version");
    space := find_index_from_left(server_version, #char " ");

    if space >= 0 server_version.count = space;

    variables_set(*psql.vars, "SERVER_VERSION_NAME", server_version);
    variables_set(*psql.vars, "SERVER_VERSION_NUM", tprint("%", c.server_version));
    variables_set(*psql.vars, "VERSION_NAME", server_version);
    variables_set(*psql.vars, "VERSION_NUM", tprint("%", c.server_version));
    variables_set(*psql.vars, "VERSION", tprint("PostgreSQL % (jacket psql shim)", server_version));
}

standard_strings :: () -> bool {
    if !psql.db return true;
    return Pg.parameter(psql.db, "standard_conforming_strings") != "off";
}

// Output. stdout and stderr are written straight through, unbuffered, so that their order in a
// file both are redirected to matches psql's.

write_fd :: (fd: s32, text: string) {
    at := 0;
    while at < text.count {
        n := POSIX.write(fd, text.data + at, cast(u64) (text.count - at));
        if n < 0 {
            if errno_value() == POSIX.EINTR continue;
            return;
        }

        at += n;
    }
}

write_out :: (text: string) {
    if psql.gfile {
        // \g file: appended to the file opened for this query.
        file_out(text);
        return;
    }

    if out_file_open {
        file_write(*out_file, text);
        return;
    }

    write_fd(1, text);
}

// \o: query output goes here until the next \o.
out_file: File;
out_file_open: bool;

g_output: File;
g_output_open: bool;

file_out :: (text: string) {
    if !g_output_open {
        g_output, g_output_open = file_open(psql.gfile, for_writing = true, keep_existing_content = false);
        if !g_output_open {
            print_error(tprint("%: could not open file", psql.gfile));
            psql.gfile = "";
            write_fd(1, text);
            return;
        }
    }

    file_write(*g_output, text);
}

close_g_output :: () {
    if g_output_open file_close(*g_output);

    // \g | command: the command reads what the query printed.
    if psql.gpipe && g_output_open {
        POSIX.system(temp_c_string(tprint("(%) < '%'", psql.gpipe, psql.gfile)));
        file_delete(psql.gfile);
    }

    psql.gpipe = "";
    g_output_open = false;
    psql.gfile = "";
}

// pg_log_error and friends. psql logs tersely: no program name or level prefix.
// Messages name the file and line while a script file runs, as psql's do, and the client's own
// errors and warnings then say which they are.
print_error :: (text: string) {
    print_error_raw(tprint("%1%2\n", ifx psql.input_name then "error: " else "", text));
}

print_warning :: (text: string) {
    print_error_raw(tprint("%1%2\n", ifx psql.input_name then "warning: " else "", text));
}

print_error_raw :: (text: string) {
    if psql.input_name write_fd(2, tprint("psql:%:%: ", psql.input_name, psql.lineno));
    write_fd(2, text);
}

// Server messages, laid out as libpq does

notice_processor :: (c: *Pg.Connection, notice: *Pg.Pg_Error) {
    print_error_raw(build_error_message(notice.*, current_query, false));
}

current_query: string; // the text of the query in flight, for error cursors

build_error_message :: (error: Pg.Pg_Error, query: string, is_error: bool) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    verbosity := psql.verbosity;

    if error.kind != .SERVER {
        // A client-side failure has no fields to lay out.
        print_to_builder(*builder, "%\n", error.message);
        return builder_to_string(*builder,, temp);
    }

    if error.severity print_to_builder(*builder, "%:  ", error.severity);

    if verbosity == .SQLSTATE {
        if error.code != .NONE {
            print_to_builder(*builder, "%\n", Pg.sqlstate_text(error.code));
            return builder_to_string(*builder,, temp);
        }

        verbosity = .TERSE;
    }

    if verbosity == .VERBOSE && error.code != .NONE print_to_builder(*builder, "%: ", Pg.sqlstate_text(error.code));

    append(*builder, error.message);

    cursor_query: string;
    cursor_position := 0;

    if error.position > 0 {
        if verbosity != .TERSE && query {
            cursor_query = query;
            cursor_position = error.position;
        } else {
            print_to_builder(*builder, " at character %", error.position);
        }
    } else if error.internal_position > 0 {
        if verbosity != .TERSE && error.internal_query {
            cursor_query = error.internal_query;
            cursor_position = error.internal_position;
        } else {
            print_to_builder(*builder, " at character %", error.internal_position);
        }
    }

    append(*builder, "\n");

    if verbosity != .TERSE {
        if cursor_query report_error_position(*builder, cursor_query, cursor_position);
        if error.detail print_to_builder(*builder, "DETAIL:  %\n", error.detail);
        if error.hint print_to_builder(*builder, "HINT:  %\n", error.hint);
        if error.internal_query print_to_builder(*builder, "QUERY:  %\n", error.internal_query);

        if psql.show_context == .ALWAYS || (psql.show_context == .ERRORS && is_error) {
            if error.where print_to_builder(*builder, "CONTEXT:  %\n", error.where);
        }
    }

    if verbosity == .VERBOSE {
        if error.schema print_to_builder(*builder, "SCHEMA NAME:  %\n", error.schema);
        if error.table print_to_builder(*builder, "TABLE NAME:  %\n", error.table);
        if error.column print_to_builder(*builder, "COLUMN NAME:  %\n", error.column);
        if error.data_type print_to_builder(*builder, "DATATYPE NAME:  %\n", error.data_type);
        if error.constraint print_to_builder(*builder, "CONSTRAINT NAME:  %\n", error.constraint);

        if error.routine || error.file || error.line {
            append(*builder, "LOCATION:  ");
            if error.routine print_to_builder(*builder, "%, ", error.routine);
            if error.file && error.line print_to_builder(*builder, "%:%", error.file, error.line);
            append(*builder, "\n");
        }
    }

    return builder_to_string(*builder,, temp);
}

// The `LINE n: ...` line and the caret under the character the error points at. Positions count
// characters; screen columns count wide characters twice. A line longer than 60 columns is cut
// around the position, with `...` where text was dropped.
report_error_position :: (builder: *String_Builder, query: string, position: s64) {
    DISPLAY_SIZE :: 60;
    MIN_RIGHT_CUT :: 10;

    location := position - 1;

    if location < 0 return;

    // Per character: its byte offset and starting screen column. Tabs show as spaces.
    text := copy_string(query,, temp);
    offsets: [..] s64;
    columns: [..] s64;

    offsets.allocator = temp;
    columns.allocator = temp;

    byte := 0;
    column := 0;
    line_number := 1;
    first := 0;
    last := -1;
    count := 0;

    while byte < text.count {
        c := text[byte];

        array_add(*offsets, byte);
        array_add(*columns, column);

        if c == #char "\t" {
            text[byte] = #char " ";
        } else if c == #char "\r" || c == #char "\n" {
            if count < location {
                if c == #char "\r" || count == 0 || text[offsets[count - 1]] != #char "\r" line_number += 1;
                first = count + 1;
            } else {
                last = count;
                break;
            }
        }

        length := utf8_length(text, byte);
        width := char_width(slice(text, byte, length));

        column += width;
        byte += length;
        count += 1;
    }

    if last < 0 {
        last = count;
        array_add(*offsets, byte);
        array_add(*columns, column);
    }

    if location > count return;

    cut_left := false;
    cut_right := false;

    if columns[last] - columns[first] > DISPLAY_SIZE {
        if columns[first] + DISPLAY_SIZE >= columns[location] + MIN_RIGHT_CUT {
            while columns[last] - columns[first] > DISPLAY_SIZE last -= 1;
            cut_right = true;
        } else {
            while columns[location] + MIN_RIGHT_CUT < columns[last] {
                last -= 1;
                cut_right = true;
            }

            while columns[last] - columns[first] > DISPLAY_SIZE {
                first += 1;
                cut_left = true;
            }
        }
    }

    prefix := tprint("LINE %: ", line_number);
    if cut_left prefix = tprint("%...", prefix);

    append(builder, prefix);
    append(builder, slice(text, offsets[first], offsets[last] - offsets[first]));
    if cut_right append(builder, "...");

    append(builder, "\n");

    indent := prefix.count + columns[location] - columns[first];
    for 1..indent append(builder, " ");

    append(builder, "^\n");
}

utf8_length :: (text: string, at: s64) -> s64 {
    c := text[at];
    length := 1;

    if c >= 0xf0 length = 4;
    else if c >= 0xe0 length = 3;
    else if c >= 0xc0 length = 2;

    return min(length, text.count - at);
}

// Screen columns of one character, as libpq counts them for the cursor: control characters and
// zero-width ones count 1.
char_width :: (c: string) -> s64 {
    if c.count == 1 return 1;
    width := display_width(c);
    return ifx width <= 0 then 1 else width;
}

// The main loop: lines become statements and backslash commands (psql's MainLoop)

If_State :: enum u8 {
    TRUE; // in a branch that runs
    FALSE; // in a branch that does not run; a later \elif or \else may
    IGNORED; // an earlier branch ran, or the whole block is in a dead branch
    ELSE_TRUE;
    ELSE_FALSE;
}

If_Level :: struct {
    state: If_State;
    saved_length: s64;
    saved_lex: Lex_State;
}

Command_Status :: enum u8 {
    SKIP_LINE; // done; the rest of the line goes on
    SEND; // send the query buffer
    ERROR;
    TERMINATE; // \q
}

Loop :: struct {
    scanner: Scanner;
    query: [..] u8;
    previous: string; // the last query sent, for \g with an empty buffer
    levels: [..] If_Level;
}

// The path with repeated and trailing slashes and `.` components removed, as psql shows it.
canonical_path :: (path: string) -> string {
    parts: [..] string;
    parts.allocator = temp;
    for split(path, "/",, temp) if it && it != "." array_add(*parts, it);

    joined := join(..parts, "/",, temp);
    if begins_with(path, "/") return tprint("/%", joined);
    if !joined return ".";

    return joined;
}

branch_active :: (l: *Loop) -> bool {
    if !l.levels.count return true;
    state := l.levels[l.levels.count - 1].state;
    return state == .TRUE || state == .ELSE_TRUE;
}

main_loop :: (input: string, one_command := false, input_name := "", from_stdin := false) -> s64 {
    saved_input := psql.input;
    saved_at := psql.input_at;
    saved_lineno := psql.lineno;
    saved_name := psql.input_name;
    saved_stdin := psql.stdin_open;

    psql.input = input;
    psql.input_at = 0;
    psql.lineno = 0;
    psql.input_name = input_name;
    psql.stdin_open = from_stdin;

    defer {
        psql.stdin_open = saved_stdin;
        psql.input = saved_input;
        psql.input_at = saved_at;
        psql.lineno = saved_lineno;
        psql.input_name = saved_name;
    }

    l: Loop;
    l.scanner.lookup = variables_lookup;
    l.scanner.hook_data = *psql.vars;
    l.scanner.report = scanner_message;

    result := EXIT_SUCCESS;
    status := Command_Status.SKIP_LINE;
    success := true;

    while true {
        psql.continuation = l.query.count > 0 || scan_in_quote(*l.scanner);

        line, more := next_line();
        if !more break;

        psql.lineno += 1;

        if psql.lineno == 1 && begins_with(line, "\xef\xbb\xbf") line = advance(line, 3);

        if psql.lineno == 1 && !one_command && begins_with(line, "PGDMP") {
            write_out("The input is a PostgreSQL custom-format dump.\nUse the pg_restore command-line client to restore this dump to a database.\n\n");
            result = EXIT_FAILURE;
            break;
        }

        if line.count == 0 && !scan_in_quote(*l.scanner) continue;

        if psql.echo == .ALL && !one_command {
            write_fd(1, line);
            write_fd(1, "\n");
        }

        added_newline := -1;

        if l.query.count > 0 {
            array_add(*l.query, #char "\n");
            added_newline = l.query.count;
        }

        die_on_error := psql.on_error_stop;

        l.scanner.std_strings = standard_strings();
        scan_setup(*l.scanner, line);
        success = true;

        while success || !die_on_error {
            l.scanner.active = branch_active(*l);
            scan_result := scan(*l.scanner, *l.query);

            if scan_result == .SEMICOLON {
                if branch_active(*l) {
                    copies := scan_count_copy_from_stdin(*l.scanner);
                    sql := copy_string(buffer_string(l.query));

                    success = send_query(sql, copies);
                    status = ifx success then Command_Status.SEND else .ERROR;
                    l.previous = sql;
                    l.query.count = 0;
                    scan_reset(*l.scanner);
                    added_newline = -1;
                } else {
                    success = true;
                    status = .SEND;
                }
            } else if scan_result == .BACKSLASH {
                if l.query.count == added_newline l.query.count -= 1;

                added_newline = -1;

                status = handle_slash_command(*l);
                success = status != .ERROR;

                if status == .SEND {
                    sql := copy_string(ifx l.query.count then buffer_string(l.query) else l.previous);

                    success = send_query(sql, -1);
                    l.previous = sql;
                    l.query.count = 0;
                    scan_reset(*l.scanner);
                } else if status == .TERMINATE {
                    break;
                }
            }

            if scan_result == .INCOMPLETE || scan_result == .EOL break;
        }

        scan_finish(*l.scanner);

        if status == .TERMINATE {
            result = EXIT_SUCCESS;
            break;
        }

        if !success && die_on_error {
            result = EXIT_USER;
            break;
        }

        if !psql.db {
            result = EXIT_BADCONN;
            break;
        }
    }

    if one_command && !success && result == EXIT_SUCCESS result = EXIT_FAILURE;

    if l.query.count > 0 && result == EXIT_SUCCESS && status != .TERMINATE {
        if branch_active(*l) {
            success = send_query(copy_string(buffer_string(l.query)), scan_count_copy_from_stdin(*l.scanner));
            if !success && psql.on_error_stop result = EXIT_USER;
            else if !psql.db result = EXIT_BADCONN;
        }
    }

    if status != .TERMINATE && result != EXIT_USER && l.levels.count {
        print_error("reached EOF without finding closing \\endif(s)");
        if psql.on_error_stop result = EXIT_USER;
    }

    return result;
}

scanner_message :: (kind: Message_Kind, text: string, data: *void) {
    print_error(text);
}

// The next input line without its line ending; false at the end of the input.
next_line :: () -> string, bool {
    fill_input();
    if psql.input_at >= psql.input.count return "", false;

    start := psql.input_at;
    end := start;

    while end < psql.input.count && psql.input[end] != #char "\n" end += 1;

    psql.input_at = ifx end < psql.input.count then end + 1 else end;
    line := slice(psql.input, start, end - start);

    // psql strips a trailing \r as well (pg_strip_crlf).
    while line.count && (line[line.count - 1] == #char "\r") line.count -= 1;

    return line, true;
}

// The next raw input line with its newline, for COPY data.
next_raw_line :: () -> string, bool {
    fill_input();
    if psql.input_at >= psql.input.count return "", false;

    start := psql.input_at;
    end := start;

    while end < psql.input.count && psql.input[end] != #char "\n" end += 1;
    if end < psql.input.count end += 1;

    psql.input_at = end;

    return slice(psql.input, start, end - start), true;
}

// Sending a query and showing its results (psql's SendQuery and ExecQueryAndProcessResults)

Query_Hooks :: struct {
    #as using hooks: Pg.Script_Hooks;

    // A result is shown only once the next one (or the end) has arrived, as psql does: notices
    // raised while the next statement runs come out before it.
    pending: Pg.Result;
    has_pending: bool;
    pending_is_copy_out: bool;

    // FETCH_COUNT: rows printed so far of the statement being received in pieces, and of the
    // pending one, whose table is then finished instead of printed whole.
    chunk_prior: s64;
    chunk_started: bool;
    pending_prior: s64;
    pending_continued: bool;

    any_result: bool;
    last_command: string;
    copies_expected: s64;
    copies_started: s64;
    copy_out_running: bool;
    copy_in_done: bool;
    success: bool;
    gfile_failed: bool;
}

send_query :: (sql: string, copies_expected: s64) -> bool {
    if !psql.db {
        print_error("You are currently not connected to a database.");
        reset_query_flags();
        return false;
    }

    if psql.echo == .QUERIES {
        write_fd(1, sql);
        write_fd(1, "\n");
    }

    ok := false;
    savepoint := false;

    if psql.db.status == .IDLE && !psql.autocommit && !command_no_begin(sql) {
        error := Pg.execute_script(psql.db, "BEGIN");
        if error.kind != .NONE {
            print_error_raw(build_error_message(error, "BEGIN", true));
            Pg.free_error(*error);
            reset_query_flags();
            return false;
        }
    }

    if psql.db.status == .IN_TRANSACTION && psql.on_error_rollback == .ON {
        error := Pg.execute_script(psql.db, "SAVEPOINT pg_psql_temporary_savepoint");
        if error.kind != .NONE {
            print_error_raw(build_error_message(error, "", true));
            Pg.free_error(*error);
            reset_query_flags();
            return false;
        }

        savepoint = true;
    }

    savepoint_gone := false;

    started := current_time_monotonic();

    ok = execute_and_show(sql, copies_expected, *savepoint_gone);

    if psql.timing print_timing(to_microseconds(current_time_monotonic() - started));
    if psql.db show_notifications();

    if !ok && psql.echo == .ERRORS print_error(tprint("STATEMENT:  %", sql));

    if savepoint && psql.db {
        command: string;

        if psql.db.status == {
            case .FAILED; command = "ROLLBACK TO pg_psql_temporary_savepoint";
            case .IN_TRANSACTION; if !savepoint_gone command = "RELEASE pg_psql_temporary_savepoint";
        }

        if command {
            error := Pg.execute_script(psql.db, command);
            if error.kind != .NONE {
                print_error_raw(build_error_message(error, "", true));
                Pg.free_error(*error);
                ok = false;
            }
        }
    }

    reset_query_flags();

    return ok;
}

reset_query_flags :: () {
    close_g_output();

    if psql.restore_popt {
        psql.popt = psql.saved_popt;
        psql.restore_popt = false;
    }

    psql.gset_prefix = "";
    psql.gset_active = false;
    psql.gexec = false;
    psql.gdesc = false;
    psql.crosstab = false;
    for *psql.crosstab_args {
        free(it.*);
        it.* = "";
    }

    for psql.bind_params free(it);

    array_reset(*psql.bind_params);
    psql.bind_active = false;
}

execute_and_show :: (sql: string, copies_expected: s64, savepoint_gone: *bool) -> bool {
    hooks: Query_Hooks;
    hooks.on_result = on_result;
    hooks.on_copy = on_copy;
    hooks.copy_in = copy_in;
    hooks.copy_out = copy_out;
    hooks.success = true;
    hooks.copies_expected = copies_expected;

    // psql 17 prints FETCH_COUNT rows at a time as they arrive (only the aligned layouts here).
    format := psql.popt.format;
    if psql.fetch_count > 0 && !psql.gset_active && !psql.gexec && !psql.crosstab && (format == .ALIGNED || format == .WRAPPED) {
        hooks.chunk_rows = psql.fetch_count;
    }

    if copies_expected < 0 {
        // Count them the way psql does for a query that did not come through the main loop.
        s: Scanner;
        s.std_strings = standard_strings();
        scan_setup(*s, sql);
        scratch: [..] u8;
        scratch.allocator = temp;
        scan(*s, *scratch);
        hooks.copies_expected = scan_count_copy_from_stdin(*s);
        scan_finish(*s);
        scan_destroy(*s);
    }

    if psql.gdesc return describe_query(sql);

    current_query = sql;
    error: Pg.Pg_Error;

    if psql.bind_active {
        // \bind: the extended protocol, with the values as untyped text parameters.
        args := NewArray(psql.bind_params.count, Any,, temp);
        for psql.bind_params args[it_index] = psql.bind_params[it_index];

        result: Pg.Result;
        result, error = Pg.query_dynamic(psql.db, sql, ..args);
        if error.kind == .NONE on_result(*hooks, *result);
    } else {
        error = Pg.run_script(psql.db, sql, *hooks);
    }

    current_query = "";

    if error.kind != .NONE {
        // The error arrives after the result before it, which therefore was not the last.
        flush_pending(*hooks, last = false);

        print_error_raw(build_error_message(error, sql, true));
        set_result_variables_failed(error);

        Pg.free_error(*psql.last_error);
        psql.last_error = error;
        psql.last_error_query = sql;
        hooks.success = false;

        if psql.db.broken || error.kind == .CLOSED || error.kind == .IO {
            closed :: "server closed the connection unexpectedly\n\tThis probably means the server terminated abnormally\n\tbefore or while processing the request.";
            // After a FATAL error libpq appends this to that error's text, so it has no prefix.
            if error.kind == .SERVER print_error_raw(tprint("%\n", closed));
            else print_error(closed);

            print_error("connection to server was lost");
            Pg.close(psql.db, now = true);
            psql.db = null;
            if !psql.interactive exit(EXIT_BADCONN);

            return false;
        }
    } else {
        flush_pending(*hooks, last = true);
        variables_set(*psql.vars, "ENCODING", Pg.parameter(psql.db, "client_encoding"));

        if !hooks.any_result {
            // An empty query: nothing came back but it did not fail.
            variables_set(*psql.vars, "ERROR", "false");
            variables_set(*psql.vars, "SQLSTATE", "00000");
            variables_set(*psql.vars, "ROW_COUNT", "0");
        }
    }

    // A COMMIT AND CHAIN, RELEASE, ROLLBACK or SAVEPOINT of the user's own leaves the transaction
    // open but takes ON_ERROR_ROLLBACK's savepoint with it (or must not lose theirs).
    last := hooks.last_command;

    savepoint_gone.* = last == "COMMIT" || last == "SAVEPOINT" || last == "RELEASE" || last == "ROLLBACK";

    // COPY FROM STDIN that never started (it failed first): its data lines are still in the
    // input and must be skipped.
    while hooks.copies_started < hooks.copies_expected {
        hooks.copies_started += 1;

        while true {
            line, more := next_raw_line();
            if !more break;
            if line == "\\.\n" || line == "\\.\r\n" || line == "\\." break;
        }
    }

    return hooks.success;
}

print_timing :: (microseconds: s64) {
    pad :: (value: s64, width: s64) -> string {
        text := tprint("%", value);
        while text.count < width text = tprint("0%", text);
        return text;
    }

    text := tprint("Time: %.% ms", microseconds / 1000, pad(microseconds % 1000, 3));

    if microseconds >= 1_000_000 {
        total := microseconds / 1000;
        text = tprint("% (%:%.%)", text, pad(total / 60000, 2), pad((total / 1000) % 60, 2), pad(total % 1000, 3));
    }

    write_out(tprint("%\n", text));
}

// Notifications that arrived with the last statement.
show_notifications :: () {
    while true {
        n, found := Pg.next_notification(psql.db);
        if !found break;

        if n.payload {
            write_out(tprint("Asynchronous notification \"%\" with payload \"%\" received from server process with PID %.\n", n.channel, n.payload, n.backend_pid));
        } else {
            write_out(tprint("Asynchronous notification \"%\" received from server process with PID %.\n", n.channel, n.backend_pid));
        }

        Pg.free_notification(*n);
    }
}

// \watch [interval] [i=SECONDS] [c=COUNT] [m=MIN_ROWS]: runs the query again and again.
watch_command :: (l: *Loop, s: *Scanner) -> Command_Status {
    interval: float64 = 2.0;
    count := -1;
    min_rows := -1;
    have_interval, have_count, have_min := false, false, false;

    while true {
        arg, found := next_arg(s);
        if !found break;

        name, value := "", arg;
        eq := find_index_from_left(arg, #char "=");

        if eq >= 0 {
            name = slice(arg, 0, eq);
            value = advance(arg, eq + 1);
        }

        if !name || name == "i" || name == "interval" {
            if have_interval {
                print_error("interval value is specified more than once");
                return .ERROR;
            }

            number, valid := parse_watch_number(value);
            if !valid || number < 0 || number > 1.0e9 {
                print_error(tprint("incorrect interval value \"%\"", value));
                return .ERROR;
            }

            interval = number;
            have_interval = true;
        } else if name == "c" || name == "count" {
            if have_count {
                print_error("iteration count is specified more than once");
                return .ERROR;
            }

            number, valid, rest := string_to_int(value);
            if !valid || rest || number <= 0 {
                print_error(tprint("incorrect iteration count \"%\"", value));
                return .ERROR;
            }

            count = number;
            have_count = true;
        } else if name == "m" || name == "min_rows" {
            if have_min {
                print_error("minimum row count specified more than once");
                return .ERROR;
            }

            number, valid, rest := string_to_int(value);
            if !valid || rest || number < 0 {
                print_error(tprint("incorrect minimum row count \"%\"", value));
                return .ERROR;
            }

            min_rows = number;
            have_min = true;
        } else {
            print_error(tprint("unrecognized parameter \"%\"", name));
            return .ERROR;
        }
    }

    sql := copy_string(ifx l.query.count then buffer_string(l.query) else l.previous);

    l.previous = sql;
    l.query.count = 0;
    scan_reset(*l.scanner);

    while send_query(sql, -1) {
        if count > 0 {
            count -= 1;
            if count == 0 break;
        }

        rows, _ := variables_get(*psql.vars, "ROW_COUNT");
        number, valid := string_to_int(rows);

        if min_rows >= 0 && valid && number < min_rows break;

        Io.sleep(cast(s64) (interval * 1000));
    }

    return .SKIP_LINE;
}

// A non-negative decimal number such as 0.01 or 2: the whole text must be one.
parse_watch_number :: (text: string) -> float64, bool {
    if !text return 0, false;

    value := 0.0;
    scale := 0.0;
    digits := 0;

    for text {
        if it == #char "." {
            if scale != 0 return 0, false;
            scale = 1;
        } else if it >= #char "0" && it <= #char "9" {
            digits += 1;

            if scale == 0 {
                value = value * 10 + (it - #char "0");
            } else {
                scale /= 10;
                value += (it - #char "0") * scale;
            }
        } else {
            return 0, false;
        }
    }

    return value, digits > 0;
}

// \password: asks twice for a new password and sets it, hashed here so that the server never sees
// it. On a terminal the prompts are shown; the typing is not hidden.
password_command :: (s: *Scanner) -> Command_Status {
    user, has_user := next_arg(s, .SQL_ID);
    if !has_user user = psql.config.user;

    ask :: (prompt: string) -> string {
        psql.password_prompt = prompt;
        line, _ := next_raw_line();
        psql.password_prompt = "";

        return trim_right(line, "\r\n");
    }

    first := ask(tprint("Enter new password for user \"%\": ", user));
    second := ask("Enter it again: ");

    if first != second {
        print_error("Passwords didn't match.");
        return .ERROR;
    }

    secret: string;
    result, error := Pg.query_dynamic(psql.db, "SHOW password_encryption");

    if error.kind == .NONE {
        method := Pg.get(*result, 0, 0, string);

        if method == "md5" {
            secret = Pg.md5_secret(first, user);
        } else {
            iterations, ok := string_to_int(Pg.parameter(psql.db, "scram_iterations"));
            secret = Pg.scram_secret(first, ifx ok then iterations else 4096);
        }

        Pg.free_result(*result);
        error = Pg.execute_script(psql.db, tprint("ALTER USER % PASSWORD %", escape_identifier(user), escape_literal(secret)));
    }

    if error.kind != .NONE {
        print_error_raw(build_error_message(error, "", true));
        Pg.free_error(*error);
        return .ERROR;
    }

    return .SKIP_LINE;
}

// \!: runs the line with the shell and records how it ended.
shell_command :: (line: string) -> Command_Status {
    if out_file_open file_flush(*out_file);

    status := POSIX.system(temp_c_string(line));
    exit := ifx status == -1 then -1 else (status >> 8) & 0xff;

    variables_set(*psql.vars, "SHELL_ERROR", ifx status == 0 then "false" else "true");
    variables_set(*psql.vars, "SHELL_EXIT_CODE", tprint("%", exit));

    if status != 0 {
        print_error("\\!: failed");
        return .ERROR;
    }

    return .SKIP_LINE;
}

// \gdesc: the columns the query would return, without running it (psql's DescribeQuery).
describe_query :: (sql: string) -> bool {
    info, error := Pg.describe_statement(psql.db, sql);
    if error.kind != .NONE {
        print_error_raw(build_error_message(error, sql, true));
        set_result_variables_failed(error);
        Pg.free_error(*psql.last_error);
        psql.last_error = error;
        psql.last_error_query = sql;

        return false;
    }

    defer Pg.free_statement_info(*info);

    if !info.columns.count {
        write_out("The command has no result, or the result has no columns.\n");
        variables_set(*psql.vars, "ERROR", "false");
        variables_set(*psql.vars, "SQLSTATE", "00000");
        variables_set(*psql.vars, "ROW_COUNT", "0");

        return true;
    }

    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "SELECT name AS \"Column\", pg_catalog.format_type(tp, tpm) AS \"Type\"\nFROM (VALUES ");

    for info.columns {
        if it_index > 0 append(*builder, ",");
        print_to_builder(*builder, "(%, '%'::pg_catalog.oid, %)", escape_literal(it.name), it.type_oid, it.type_modifier);
    }

    append(*builder, ") s(name, tp, tpm)");
    query := builder_to_string(*builder,, temp);

    hooks: Query_Hooks;
    hooks.on_result = on_result;
    hooks.success = true;

    current_query = query;
    error = Pg.run_script(psql.db, query, *hooks);
    current_query = "";

    if error.kind != .NONE {
        print_error_raw(build_error_message(error, query, true));
        set_result_variables_failed(error);
        Pg.free_error(*error);
        return false;
    }

    flush_pending(*hooks, last = true);

    return hooks.success;
}

on_result :: (h: *Pg.Script_Hooks, result: *Pg.Result) {
    hooks := cast(*Query_Hooks) h;

    flush_pending(hooks, last = false);

    if result.partial {
        hooks.any_result = true;
        if hooks.success print_rows(result, !hooks.chunk_started, false, hooks.chunk_prior);

        hooks.chunk_started = true;
        hooks.chunk_prior += result.row_count;

        return;
    }

    hooks.pending_continued = hooks.chunk_started;
    hooks.pending_prior = hooks.chunk_prior;
    hooks.chunk_started = false;
    hooks.chunk_prior = 0;

    hooks.any_result = true;
    hooks.last_command = copy_string(result.command,, temp);
    hooks.pending = result.*;
    hooks.has_pending = true;
    hooks.pending_is_copy_out = hooks.copy_out_running;
    hooks.copy_out_running = false;
    result.* = .{};
}

on_copy :: (h: *Pg.Script_Hooks, copy: *Pg.Copy_Info) {
    hooks := cast(*Query_Hooks) h;

    flush_pending(hooks, last = false);

    if copy.incoming {
        hooks.copies_started += 1;
        hooks.copy_in_done = false;
    } else {
        hooks.copy_out_running = true;
    }
}

copy_in :: (h: *Pg.Script_Hooks, copy: *Pg.Copy_Info) -> string, bool, string {
    hooks := cast(*Query_Hooks) h;
    if hooks.copy_in_done return "", true, "";

    if copy_file_data_active {
        // \copy ... from 'file': the whole file at once.
        hooks.copy_in_done = true;
        return copy_file_data, false, "";
    }

    line, more := next_raw_line();
    if !more return "", true, "";

    psql.lineno += 1;
    if line == "\\.\n" || line == "\\.\r\n" || line == "\\." hooks.copy_in_done = true;

    return line, false, "";
}

copy_out :: (h: *Pg.Script_Hooks, copy: *Pg.Copy_Info, data: [] u8) {
    if copy_file_out_active {
        file_write(*copy_file_out, data.data, data.count);
        return;
    }

    write_out(cast(string) data);
}

// Where \copy reads or writes when it names a file.
copy_file_data: string;
copy_file_data_active: bool;
copy_file_out: File;
copy_file_out_active: bool;

// psql's strtokx: the next token of `text` from `at`. A token is one `delim` character, a quoted
// string (quotes kept), or a run up to whitespace, a `delim` or a `quote` character. A whitespace
// character ending a token is consumed with it.
Tokenizer :: struct {
    text: string;
    at: s64;
}

next_token :: (t: *Tokenizer, delim: string, quote: string, escape: u8 = 0, e_strings := false) -> string, bool {
    WHITESPACE :: " \t\n\r";

    while t.at < t.text.count && is_any(t.text[t.at], WHITESPACE) t.at += 1;
    if t.at >= t.text.count return "", false;

    start := t.at;
    c := t.text[start];

    finish :: (end: s64) -> string #expand {
        t.at = end;
        if end < t.text.count && is_any(t.text[end], WHITESPACE) t.at = end + 1;
        return slice(t.text, start, end - start);
    }

    if is_any(c, delim) return finish(start + 1), true;

    p := start;
    active_quote := quote;
    active_escape := escape;

    if e_strings && (c == #char "E" || c == #char "e") && p + 1 < t.text.count && t.text[p + 1] == #char "'" {
        active_quote = "'";
        active_escape = #char "\\";
        p += 1;
    }

    if is_any(t.text[p], active_quote) {
        this_quote := t.text[p];

        p += 1;

        while p < t.text.count {
            if active_escape && t.text[p] == active_escape && p + 1 < t.text.count {
                p += 2;
            } else if t.text[p] == this_quote && p + 1 < t.text.count && t.text[p + 1] == this_quote {
                p += 2;
            } else if t.text[p] == this_quote {
                p += 1;
                break;
            } else {
                p += 1;
            }
        }

        return finish(p), true;
    }

    while p < t.text.count && !is_any(t.text[p], WHITESPACE) && !is_any(t.text[p], delim) && !is_any(t.text[p], quote) p += 1;

    return finish(p), true;
}

is_any :: (c: u8, set: string) -> bool {
    for set if it == c return true;
    return false;
}

// \lo_import, \lo_export and \lo_unlink, done with the server's large-object functions. psql runs
// them in a transaction of their own when none is open; a single statement is one already.
large_object_command :: (s: *Scanner, name: string) -> Command_Status {
    first, has_first := next_arg(s);
    second, has_second := next_arg(s);

    if !has_first || (name == "lo_export" && !has_second) {
        print_error(tprint("\\%: missing required argument", name));
        return .ERROR;
    }

    if !psql.db {
        print_error("You are currently not connected to a database.");
        return .ERROR;
    }

    message: string;

    if name == "lo_import" {
        data, ok := read_entire_file(first, log_errors = false);
        if !ok {
            print_error(tprint("could not open file \"%\": No such file or directory", first));
            return .ERROR;
        }

        result, error := Pg.query_dynamic(psql.db, "select lo_from_bytea(0, $1)", cast([] u8) data);

        free(data);

        if error.kind != .NONE {
            print_error_raw(build_error_message(error, "", true));
            Pg.free_error(*error);
            return .ERROR;
        }

        bytes, _ := Pg.raw_value(*result, 0, 0);
        oid := copy_string(cast(string) bytes,, temp);

        Pg.free_result(*result);

        if has_second {
            comment := tprint("COMMENT ON LARGE OBJECT % IS %", oid, escape_literal(second));
            if !send_query(comment, 0) return .ERROR;
        }

        variables_set(*psql.vars, "LASTOID", oid);
        message = tprint("lo_import %", oid);
    } else if name == "lo_export" {
        result, error := Pg.query_binary(psql.db, "select lo_get($1::oid)", first);
        if error.kind != .NONE {
            print_error_raw(build_error_message(error, "", true));
            Pg.free_error(*error);
            return .ERROR;
        }

        bytes, _ := Pg.raw_value(*result, 0, 0);
        ok := write_entire_file(second, bytes.data, bytes.count);

        Pg.free_result(*result);

        if !ok {
            print_error(tprint("could not open file \"%\"", second));
            return .ERROR;
        }

        message = "lo_export";
    } else {
        result, error := Pg.query_dynamic(psql.db, "select lo_unlink($1::oid)", first);
        if error.kind != .NONE {
            print_error_raw(build_error_message(error, "", true));
            Pg.free_error(*error);
            return .ERROR;
        }

        Pg.free_result(*result);
        message = tprint("lo_unlink %", first);
    }

    if !psql.quiet write_out(tprint("%\n", message));

    return .SKIP_LINE;
}

// \copy: a COPY ... FROM STDIN or TO STDOUT whose data is a file or psql's own input and output.
copy_command :: (args: string) -> bool {
    t := Tokenizer.{ text = args };
    before: String_Builder;
    before.allocator = temp;
    token: string;
    ok: bool;

    fail :: () #expand {
        if ok print_error(tprint("\\copy: parse error at \"%\"", token));
        else print_error("\\copy: parse error at end of line");
        `return false;
    }

    if !args {
        print_error("\\copy: arguments required");
        return false;
    }

    token, ok = next_token(*t, ".,()", "\"");
    if !ok fail();

    if equal_nocase(token, "binary") {
        append(*before, token);
        token, ok = next_token(*t, ".,()", "\"");
        if !ok fail();
    }

    if token[0] == #char "(" {
        parens := 1;
        escape: u8 = ifx standard_strings() then cast(u8) 0 else #char "\\";

        while parens > 0 {
            append(*before, " ");
            append(*before, token);
            token, ok = next_token(*t, "()", "\"'", escape, e_strings = true);
            if !ok fail();
            if token[0] == #char "(" parens += 1;
            else if token[0] == #char ")" parens -= 1;
        }
    }

    append(*before, " ");
    append(*before, token);
    token, ok = next_token(*t, ".,()", "\"");
    if !ok fail();

    if token[0] == #char "." {
        append(*before, token);
        token, ok = next_token(*t, ".,()", "\"");
        if !ok fail();

        append(*before, token);
        token, ok = next_token(*t, ".,()", "\"");
        if !ok fail();
    }

    if token[0] == #char "(" {
        while true {
            append(*before, " ");
            append(*before, token);
            token, ok = next_token(*t, "()", "\"");
            if !ok fail();
            if token[0] == #char ")" break;
        }

        append(*before, " ");
        append(*before, token);
        token, ok = next_token(*t, ".,()", "\"");
        if !ok fail();
    }

    from: bool;
    if equal_nocase(token, "from") from = true;
    else if equal_nocase(token, "to") from = false;
    else fail();

    token, ok = next_token(*t, ";", "'");
    if !ok fail();

    file: string;

    if equal_nocase(token, "program") {
        print_error("\\copy: PROGRAM is not supported by this shim");
        return false;
    } else if !(equal_nocase(token, "stdin") || equal_nocase(token, "stdout") || equal_nocase(token, "pstdin") || equal_nocase(token, "pstdout")) {
        file = token;
        if file.count >= 2 && file[0] == #char "'" && file[file.count - 1] == #char "'" {
            file = replace(slice(file, 1, file.count - 2), "''", "'",, temp);
        }
    }

    after := slice(t.text, t.at, t.text.count - t.at);

    if file && from {
        data, read_ok := read_entire_file(file, log_errors = false);
        if !read_ok {
            print_error(tprint("%: No such file or directory", file));
            return false;
        }

        copy_file_data = data;
        copy_file_data_active = true;
    } else if file {
        copy_file_out, copy_file_out_active = file_open(file, for_writing = true, keep_existing_content = false);
        if !copy_file_out_active {
            print_error(tprint("%: could not open file", file));
            return false;
        }
    }

    defer {
        if copy_file_data_active free(copy_file_data);
        copy_file_data_active = false;
        if copy_file_out_active file_close(*copy_file_out);
        copy_file_out_active = false;
    }

    direction := ifx from then " FROM STDIN " else " TO STDOUT ";
    sql := tprint("COPY %1%2%3", builder_to_string(*before,, temp), direction, after);

    return send_query(sql, ifx from && !file then 1 else 0);
}

flush_pending :: (hooks: *Query_Hooks, last: bool) {
    if !hooks.has_pending return;

    result := *hooks.pending;

    defer {
        Pg.free_result(result);
        hooks.has_pending = false;
    }

    if hooks.success {
        print_continued = hooks.pending_continued;
        print_prior = hooks.pending_prior;
        if !show_result(result, last, hooks.pending_is_copy_out) hooks.success = false;

        print_continued = false;
        print_prior = 0;
    }

    if last set_result_variables(result);
}

show_result :: (result: *Pg.Result, last: bool, copy_out: bool) -> bool {
    success := true;

    if result.returns_rows {
        if last && psql.gset_active {
            success = store_query_tuple(result);
        } else if last && psql.gexec {
            success = exec_query_tuples(result);
        } else if last && psql.crosstab {
            success = print_crosstab(result);
        } else if last || psql.show_all_results {
            print_rows(result, !print_continued, true, print_prior);
        }

        if last || psql.show_all_results {
            tag := result.command;
            if begins_with(tag, "INSERT") || begins_with(tag, "UPDATE") || begins_with(tag, "DELETE") || begins_with(tag, "MERGE") {
                print_status(result);
            }
        }
    } else if !copy_out {
        if last || psql.show_all_results print_status(result);
    }

    return success;
}

print_status :: (result: *Pg.Result) {
    if psql.quiet return;
    write_out(result.command);
    write_out("\n");
}

// The rest of a table whose first rows were printed as they arrived.
print_continued: bool;
print_prior: s64;

print_rows :: (result: *Pg.Result, start := true, stop := true, prior: s64 = 0) {
    columns := result.columns.count;
    headers := NewArray(columns, string,, temp);
    oids := NewArray(columns, u32,, temp);

    for result.columns {
        headers[it_index] = it.name;
        oids[it_index] = it.type_oid;
    }

    cells := NewArray(result.row_count * columns, string,, temp);
    nulls := NewArray(result.row_count * columns, bool,, temp);

    for row: 0..result.row_count - 1 {
        for column: 0..columns - 1 {
            bytes, is_null := Pg.raw_value(result, row, column);
            cells[row * columns + column] = cast(string) bytes;
            nulls[row * columns + column] = is_null;
        }
    }

    builder: String_Builder;

    table: Table;
    table.headers = headers;
    table.cells = cells;
    table.nulls = nulls;
    table.rows = result.row_count;
    table.default_footer = true;
    table.start_table = start;
    table.stop_table = stop;
    table.prior_records = prior;

    table.align = NewArray(columns, u8,, temp);
    for oids table.align[it_index] = column_alignment(it);

    if !start && stop && result.row_count > 0 {
        // psql gets the last rows of a chunked result as one more piece, then finishes the table
        // with an empty one, whose widths come from the headers alone.
        table.stop_table = false;
        print_table(*builder, table, psql.popt);

        table.cells = .[];
        table.nulls = .[];
        table.rows = 0;
        table.prior_records = prior + result.row_count;
        table.stop_table = true;
    }

    print_table(*builder, table, psql.popt);

    text := builder_to_string(*builder);

    write_out(text);
    free(text);
}

// The variables psql attaches behaviour to; \gset leaves them alone.
has_hook :: (name: string) -> bool {
    HOOKED :: string.[
        "AUTOCOMMIT", "ON_ERROR_STOP", "QUIET", "SINGLELINE", "SINGLESTEP", "FETCH_COUNT",
        "HISTFILE", "HISTSIZE", "IGNOREEOF", "ECHO", "ECHO_HIDDEN", "ON_ERROR_ROLLBACK",
        "COMP_KEYWORD_CASE", "HISTCONTROL", "PROMPT1", "PROMPT2", "PROMPT3", "VERBOSITY",
        "SHOW_ALL_RESULTS", "SHOW_CONTEXT", "HIDE_TOAST_COMPRESSION", "HIDE_TABLEAM",
        "WATCH_INTERVAL",
    ];

    for HOOKED if it == name return true;

    return false;
}

// \crosstabview: the result pivoted, one column's values down the side, another's across the top
// and a third's in the cells. The arguments name the columns by number or name: vertical,
// horizontal, data, and a column whose integers order the horizontal header.
print_crosstab :: (result: *Pg.Result) -> bool {
    MAX_COLUMNS :: 1600;

    fail :: (message: string) -> bool {
        print_error(tprint("\\crosstabview: %", message));
        return false;
    }

    columns := result.columns.count;

    if columns < 3 return fail("query must return at least three columns");

    // -1 for an argument not given.
    index: [4] s64;

    for arg, arg_index: psql.crosstab_args {
        index[arg_index] = -1;
        if !arg continue;

        if is_all_digits(arg) {
            number := string_to_int(arg);
            if number < 1 || number > columns return fail(tprint("column number % is out of range 1..%", number, columns));
            index[arg_index] = number - 1;
            continue;
        }

        name := sql_identifier(arg);

        for result.columns {
            if it.name != name continue;
            if index[arg_index] >= 0 return fail(tprint("ambiguous column name: \"%\"", name));
            index[arg_index] = it_index;
        }

        if index[arg_index] < 0 return fail(tprint("column name not found: \"%\"", name));
    }

    vertical := ifx index[0] >= 0 then index[0] else 0;
    horizontal := index[1];

    if horizontal < 0 horizontal = ifx vertical == 0 then 1 else 0;
    if horizontal == vertical return fail("vertical and horizontal headers must be different columns");

    data := index[2];
    if data < 0 {
        if columns != 3 return fail("data column must be specified when query returns more than three columns");
        data = 3 - vertical - horizontal;
    }

    sort := index[3];

    Header :: struct {
        value: string;
        is_null: bool;
        sort: s64;
    }

    find :: (headers: [] Header, value: string, is_null: bool) -> s64 {
        for headers if it.is_null == is_null && (is_null || it.value == value) return it_index;
        return -1;
    }

    across: [..] Header;
    down: [..] Header;

    across.allocator = temp;
    down.allocator = temp;

    // Which row of the result fills each cell, -1 when none does.
    cell_rows: [..] s64;
    cell_rows.allocator = temp;

    for row: 0..result.row_count - 1 {
        h, h_null := Pg.raw_value(result, row, horizontal);
        if find(across, cast(string) h, h_null) < 0 {
            if across.count >= MAX_COLUMNS return fail(tprint("maximum number of columns (%) exceeded", MAX_COLUMNS));

            header := Header.{ value = cast(string) h, is_null = h_null };

            if sort >= 0 {
                key, key_null := Pg.raw_value(result, row, sort);
                if !key_null header.sort = string_to_int(cast(string) key);
            }

            array_add(*across, header);
        }

        v, v_null := Pg.raw_value(result, row, vertical);
        if find(down, cast(string) v, v_null) < 0 array_add(*down, .{ value = cast(string) v, is_null = v_null });
    }

    if sort >= 0 {
        // Stable, so equal keys keep the order they first appeared in.
        for i: 1..across.count - 1 {
            j := i;
            while j > 0 && across[j - 1].sort > across[j].sort {
                across[j - 1], across[j] = across[j], across[j - 1];
                j -= 1;
            }
        }
    }

    width := across.count + 1;

    array_resize(*cell_rows, down.count * across.count);
    for *cell_rows it.* = -1;

    for row: 0..result.row_count - 1 {
        h, h_null := Pg.raw_value(result, row, horizontal);
        v, v_null := Pg.raw_value(result, row, vertical);
        x := find(across, cast(string) h, h_null);
        y := find(down, cast(string) v, v_null);

        slot := *cell_rows[y * across.count + x];

        if slot.* >= 0 {
            row_text := ifx v_null then psql.popt.null_display else cast(string) v;
            column_text := ifx h_null then psql.popt.null_display else cast(string) h;
            return fail(tprint("query result contains multiple data values for row \"%\", column \"%\"", row_text, column_text));
        }

        slot.* = row;
    }

    table: Table;
    table.default_footer = true;
    table.rows = down.count;

    headers := NewArray(width, string,, temp);
    headers[0] = result.columns[vertical].name;
    for across headers[it_index + 1] = ifx it.is_null then psql.popt.null_display else it.value;

    table.headers = headers;

    align := NewArray(width, u8,, temp);
    align[0] = column_alignment(result.columns[vertical].type_oid);
    data_align := column_alignment(result.columns[data].type_oid);
    for 1..width - 1 align[it] = data_align;

    table.align = align;

    cells := NewArray(down.count * width, string,, temp);
    nulls := NewArray(down.count * width, bool,, temp);

    for y: 0..down.count - 1 {
        cells[y * width] = down[y].value;
        nulls[y * width] = down[y].is_null;

        for x: 0..across.count - 1 {
            row := cell_rows[y * across.count + x];
            if row < 0 continue;

            bytes, is_null := Pg.raw_value(result, row, data);

            cells[y * width + x + 1] = cast(string) bytes;
            nulls[y * width + x + 1] = is_null;
        }
    }

    table.cells = cells;
    table.nulls = nulls;

    builder: String_Builder;

    print_table(*builder, table, psql.popt);
    text := builder_to_string(*builder);

    write_out(text);
    free(text);

    return true;
}

is_all_digits :: (text: string) -> bool {
    for cast([] u8) text if it < #char "0" || it > #char "9" return false;
    return text.count > 0;
}

// \gset: the one row's columns become variables.
store_query_tuple :: (result: *Pg.Result) -> bool {
    if result.row_count < 1 {
        print_error("no rows returned for \\gset");
        return false;
    }

    if result.row_count > 1 {
        print_error("more than one row returned for \\gset");
        return false;
    }

    for result.columns {
        name := tprint("%1%2", psql.gset_prefix, it.name);
        if has_hook(name) {
            print_warning(tprint("attempt to \\gset into specially treated variable \"%\" ignored", name));
            continue;
        }

        bytes, is_null := Pg.raw_value(result, 0, it_index);

        if is_null {
            unset_variable(name);
        } else if !set_variable(name, cast(string) bytes) {
            return false;
        }
    }

    return true;
}

// \gexec: every non-null value is run as a query.
exec_query_tuples :: (result: *Pg.Result) -> bool {
    psql.gexec = false;
    defer psql.gexec = true;

    queries: [..] string;
    queries.allocator = temp;

    for row: 0..result.row_count - 1 {
        for column: 0..result.columns.count - 1 {
            bytes, is_null := Pg.raw_value(result, row, column);
            if !is_null array_add(*queries, copy_string(cast(string) bytes,, temp));
        }
    }

    success := true;

    // The outer query is finished on the wire, but its result is still being shown: release the
    // connection for the nested queries.
    for queries {
        if psql.echo == .ALL {
            write_fd(1, it);
            write_fd(1, "\n");
        }

        saved_gfile := psql.gfile;

        psql.gfile = "";
        ok := nested_send(it);

        psql.gfile = saved_gfile;

        if !ok {
            success = false;
            if psql.on_error_stop break;
        }
    }

    return success;
}

// Runs a query while another's results are being shown (\gexec), keeping the outer flags.
nested_send :: (sql: string) -> bool {
    saved_prefix := psql.gset_prefix;
    saved_gset := psql.gset_active;
    saved_gexec := psql.gexec;
    saved_restore := psql.restore_popt;
    saved_popt := psql.saved_popt;
    saved_query := current_query;

    psql.gset_active = false;
    psql.gexec = false;
    psql.restore_popt = false;

    was_busy := psql.db.busy;

    psql.db.busy = false;
    savepoint_gone: bool;
    ok := execute_and_show(sql, -1, *savepoint_gone);

    if psql.db psql.db.busy = was_busy;

    psql.gset_prefix = saved_prefix;
    psql.gset_active = saved_gset;
    psql.gexec = saved_gexec;
    psql.restore_popt = saved_restore;
    psql.saved_popt = saved_popt;
    current_query = saved_query;

    return ok;
}

set_result_variables :: (result: *Pg.Result) {
    variables_set(*psql.vars, "ERROR", "false");
    variables_set(*psql.vars, "SQLSTATE", "00000");

    count := "0";
    tag := result.command;

    for string.["SELECT", "INSERT", "UPDATE", "DELETE", "MERGE", "MOVE", "FETCH", "COPY"] {
        if begins_with(tag, it) {
            count = tprint("%", result.rows_affected);
            break;
        }
    }

    variables_set(*psql.vars, "ROW_COUNT", count);
}

set_result_variables_failed :: (error: Pg.Pg_Error) {
    code := ifx error.kind == .SERVER then Pg.sqlstate_text(error.code) else "";

    variables_set(*psql.vars, "ERROR", "true");
    variables_set(*psql.vars, "SQLSTATE", code);
    variables_set(*psql.vars, "ROW_COUNT", "0");
    variables_set(*psql.vars, "LAST_ERROR_SQLSTATE", code);
    variables_set(*psql.vars, "LAST_ERROR_MESSAGE", error.message);
}

// Statements that must not run inside a transaction block, or that start or end one: with
// AUTOCOMMIT off psql does not send BEGIN before them.
command_no_begin :: (query: string) -> bool {
    q := skip_white_space(query);
    word := first_word(q);

    lower := to_lower_copy(word,, temp);

    if lower == {
        case "abort"; #through;
        case "begin"; #through;
        case "start"; #through;
        case "commit"; #through;
        case "end"; #through;
        case "rollback"; #through;
        case "vacuum";
            return true;
        case "prepare";
            next := to_lower_copy(first_word(skip_white_space(advance(q, word.count))),, temp);

            return next == "transaction";
        case "cluster";
            rest := skip_white_space(advance(q, word.count));

            return !(rest.count && is_alpha(rest[0]));
        case "create";
            rest := skip_white_space(advance(q, word.count));
            next := to_lower_copy(first_word(rest),, temp);

            if next == "database" || next == "tablespace" return true;

            if next == "unique" {
                rest = skip_white_space(advance(rest, next.count));
                next = to_lower_copy(first_word(rest),, temp);
            }

            if next == "index" {
                rest = skip_white_space(advance(rest, next.count));
                return to_lower_copy(first_word(rest),, temp) == "concurrently";
            }

            return false;
        case "alter";
            next := to_lower_copy(first_word(skip_white_space(advance(q, word.count))),, temp);

            return next == "system";
        case "drop"; #through;
        case "reindex";
            rest := skip_white_space(advance(q, word.count));
            next := to_lower_copy(first_word(rest),, temp);

            if next == "database" || next == "system" || next == "tablespace" return true;

            if next == "index" || next == "table" {
                rest = skip_white_space(advance(rest, next.count));
                return to_lower_copy(first_word(rest),, temp) == "concurrently";
            }

            return false;
        case "discard";
            next := to_lower_copy(first_word(skip_white_space(advance(q, word.count))),, temp);

            return next == "all";
    }

    return false;
}

skip_white_space :: (text: string) -> string {
    depth := 0;
    q := text;

    while q.count {
        if is_space(q[0]) {
            q = advance(q, 1);
        } else if begins_with(q, "/*") {
            depth += 1;
            q = advance(q, 2);
        } else if depth > 0 && begins_with(q, "*/") {
            depth -= 1;
            q = advance(q, 2);
        } else if depth == 0 && begins_with(q, "--") {
            newline := find_index_from_left(q, #char "\n");
            if newline < 0 return "";
            q = advance(q, newline + 1);
        } else if depth > 0 {
            q = advance(q, 1);
        } else {
            break;
        }
    }

    return q;
}

first_word :: (text: string) -> string {
    n := 0;
    while n < text.count && is_alpha(text[n]) n += 1;
    return slice(text, 0, n);
}

// Variables with side effects (psql's variable hooks)

// psql's boolean spellings: unique prefixes of true/false/yes/no, on/off (two letters at least),
// 1 and 0, in any case.
parse_bool :: (text: string, name: string) -> value: bool, ok: bool {
    t := to_lower_copy(text,, temp);

    if t.count {
        if begins_with("true", t) return true, true;
        if begins_with("false", t) return false, true;
        if begins_with("yes", t) return true, true;
        if begins_with("no", t) return false, true;
        if t.count >= 2 && begins_with("on", t) return true, true;
        if t.count >= 2 && begins_with("off", t) return false, true;
        if t == "1" return true, true;
        if t == "0" return false, true;
    }

    if name print_error(tprint("unrecognized value \"%\" for \"%\": Boolean expected", text, name));

    return false, false;
}

enum_error :: (value: string, name: string, choices: string) {
    print_error(tprint("unrecognized value \"%\" for \"%\"\nAvailable values are: %.", value, name, choices));
}

// Sets a variable, applying psql's checks and side effects for the special ones. False (with a
// message printed) when the value is not acceptable; the variable then keeps its old value.
set_variable :: (name: string, value: string) -> bool {
    return assign_variable(name, value, is_null = false);
}

unset_variable :: (name: string) -> bool {
    return assign_variable(name, "", is_null = true);
}

// psql's SetVariable with the special variables' substitute and assign hooks. Unsetting a special
// variable puts its default back; `\set` without a value turns a boolean one on.
assign_variable :: (name: string, value: string, is_null: bool) -> bool {
    if !valid_variable_name(name) {
        if is_null return true;
        print_error(tprint("invalid variable name: \"%\"", name));
        return false;
    }

    BOOLEANS :: string.[
        "AUTOCOMMIT", "ON_ERROR_STOP", "QUIET", "SINGLELINE", "SINGLESTEP", "ECHO_HIDDEN",
        "ON_ERROR_ROLLBACK", "SHOW_ALL_RESULTS", "HIDE_TOAST_COMPRESSION", "HIDE_TABLEAM",
    ];
    DEFAULTS :: string.[
        "FETCH_COUNT", "0",
        "HISTSIZE", "500",
        "IGNOREEOF", "0",
        "ECHO", "none",
        "COMP_KEYWORD_CASE", "preserve-upper",
        "HISTCONTROL", "none",
        "VERBOSITY", "default",
        "SHOW_CONTEXT", "errors",
    ];

    v := value;
    clear := is_null;
    special := false;

    for BOOLEANS if it == name {
        special = true;
        if clear v = "off";
        else if !v v = "on";
        clear = false;
    }

    i := 0;
    while i < DEFAULTS.count {
        if DEFAULTS[i] == name {
            special = true;
            if clear v = DEFAULTS[i + 1];
            clear = false;
        }

        i += 2;
    }

    if name == "IGNOREEOF" && !clear {
        _, ok, rest := string_to_int(v);
        if !ok || rest v = "10";
    }

    if !clear && !apply_variable(name, v) return false;

    if clear variables_unset(*psql.vars, name);
    else variables_set(*psql.vars, name, v);

    return true;
}

apply_variable :: (name: string, value: string) -> bool {
    lower := to_lower_copy(value,, temp);

    boolean :: (target: *bool) -> bool #expand {
        v, ok := parse_bool(value, name);
        if !ok `return false;
        target.* = v;
        return true;
    }

    integer :: () -> bool #expand {
        _, ok, rest := string_to_int(value);
        if !ok || rest {
            print_error(tprint("invalid value \"%\" for \"%\": integer expected", value, name));
            `return false;
        }

        return true;
    }

    if name == {
        case "AUTOCOMMIT"; boolean(*psql.autocommit);
        case "ON_ERROR_STOP"; boolean(*psql.on_error_stop);
        case "QUIET"; boolean(*psql.quiet);
        case "SHOW_ALL_RESULTS"; boolean(*psql.show_all_results);
        case "HIDE_TABLEAM"; boolean(*psql.hide_tableam);
        case "HIDE_TOAST_COMPRESSION"; boolean(*psql.hide_toast_compression);
        case "SINGLELINE"; #through;
        case "SINGLESTEP";
            ignored: bool;

            boolean(*ignored);
        case "ECHO_HIDDEN";
            if lower != "noexec" {
                _, ok := parse_bool(value, "");
                if !ok {
                    enum_error(value, name, "on, off, noexec");
                    return false;
                }
            }
        case "ECHO";
            if lower == {
                case "none"; psql.echo = .NONE;
                case "errors"; psql.echo = .ERRORS;
                case "queries"; psql.echo = .QUERIES;
                case "all"; psql.echo = .ALL;
                case;
                    enum_error(value, name, "none, errors, queries, all");
                    return false;
            }
        case "VERBOSITY";
            if lower == {
                case "default"; psql.verbosity = .DEFAULT;
                case "verbose"; psql.verbosity = .VERBOSE;
                case "terse"; psql.verbosity = .TERSE;
                case "sqlstate"; psql.verbosity = .SQLSTATE;
                case;
                    enum_error(value, name, "default, verbose, terse, sqlstate");
                    return false;
            }
        case "SHOW_CONTEXT";
            if lower == {
                case "never"; psql.show_context = .NEVER;
                case "errors"; psql.show_context = .ERRORS;
                case "always"; psql.show_context = .ALWAYS;
                case;
                    enum_error(value, name, "never, errors, always");
                    return false;
            }
        case "COMP_KEYWORD_CASE";
            if lower != "preserve-upper" && lower != "preserve-lower" && lower != "upper" && lower != "lower" {
                enum_error(value, name, "lower, upper, preserve-lower, preserve-upper");
                return false;
            }
        case "HISTCONTROL";
            if lower != "ignorespace" && lower != "ignoredups" && lower != "ignoreboth" && lower != "none" {
                enum_error(value, name, "none, ignorespace, ignoredups, ignoreboth");
                return false;
            }
        case "ON_ERROR_ROLLBACK";
            if lower == "interactive" {
                psql.on_error_rollback = .INTERACTIVE;
            } else {
                v, ok := parse_bool(value, "");
                if !ok {
                    enum_error(value, name, "on, off, interactive");
                    return false;
                }

                psql.on_error_rollback = ifx v then Error_Rollback.ON else .OFF;
            }
        case "FETCH_COUNT";
            integer();
            psql.fetch_count = string_to_int(value);
        case "HISTSIZE"; integer();
        case "IGNOREEOF"; integer();
    }

    return true;
}

// Backslash commands (psql's command.c)

handle_slash_command :: (l: *Loop) -> Command_Status {
    s := *l.scanner;
    name := copy_string(scan_slash_command(s),, temp);
    active := branch_active(l);

    status: Command_Status;

    if psql.restricted && name != "unrestrict" {
        print_error("backslash commands are restricted; only \\unrestrict is allowed");
        status = .ERROR;
    } else if is_conditional(name) {
        status = conditional_command(l, name);
    } else if active {
        s.active = true;
        status = run_command(l, name);
    } else if known_command(name) {
        // Commands whose argument is the rest of the line skip all of it.
        if takes_whole_line(name) scan_slash_option(s, .WHOLE_LINE);
        else if takes_file_or_pipe(name) scan_slash_option(s, .FILE_OR_PIPE);
        status = .SKIP_LINE;
    } else {
        // psql reports a command it does not know even where nothing runs.
        print_error(tprint("invalid command \\%", name));
        status = .ERROR;
    }

    if status == .ERROR {
        // Whatever is left of the line after a failed command is thrown away.
        s.active = false;
        scan_slash_option(s, .WHOLE_LINE);
    } else {
        // Arguments the command did not read are ignored, with a warning when it ran.
        s.active = false;

        while true {
            arg, found := scan_slash_option(s, .NORMAL);
            if !found break;
            if active && !is_conditional(name) print_warning(tprint("\\%: extra argument \"%\" ignored", name, arg));
        }
    }

    s.active = branch_active(l);
    scan_slash_command_end(s);

    return status;
}

// The commands psql knows (a `d` or `lo_` command with an unknown letter is only found out when
// it runs).
known_command :: (name: string) -> bool {
    KNOWN :: string.[
        "a", "bind", "C", "c", "connect", "cd", "conninfo", "copy", "copyright", "crosstabview",
        "e", "edit", "ef", "ev", "echo", "qecho", "warn", "elif", "else", "endif", "encoding",
        "errverbose", "f", "g", "gx", "gdesc", "getenv", "gexec", "gset", "h", "help", "H", "html",
        "i", "include", "ir", "include_relative", "if", "l", "list", "l+", "list+", "o", "out",
        "p", "print", "password", "prompt", "pset", "q", "quit", "r", "reset", "restrict", "s",
        "set", "setenv", "sf", "sf+", "sv", "sv+", "t", "T", "timing", "unrestrict", "unset", "w",
        "write", "watch", "x", "z", "zS", "!", "?",
    ];

    if begins_with(name, "d") || begins_with(name, "lo_") return true;
    for KNOWN if it == name return true;

    return false;
}

takes_whole_line :: (name: string) -> bool {
    WHOLE :: string.["!", "copy", "ef", "ev", "h", "help", "sf", "sf+", "sv", "sv+"];
    for WHOLE if it == name return true;
    return false;
}

takes_file_or_pipe :: (name: string) -> bool {
    FILE :: string.["g", "gx", "o", "out", "w", "write"];
    for FILE if it == name return true;
    return false;
}

is_conditional :: (name: string) -> bool {
    return name == "if" || name == "elif" || name == "else" || name == "endif";
}

save_branch :: (l: *Loop, level: *If_Level) {
    level.saved_length = l.query.count;
    level.saved_lex = scan_save_lex_state(*l.scanner);
}

discard_branch :: (l: *Loop, level: *If_Level) {
    l.query.count = level.saved_length;
    scan_restore_lex_state(*l.scanner, level.saved_lex);
}

// \if takes all its arguments, joined with spaces, as one boolean.
boolean_expression :: (s: *Scanner, name: string) -> bool {
    parts: [..] string;
    parts.allocator = temp;

    while true {
        arg, found := scan_slash_option(s, .NORMAL);
        if !found break;
        array_add(*parts, copy_string(arg,, temp));
    }

    text := join(..parts, " ",, temp);
    value, ok := parse_bool(text, tprint("\\% expression", name));

    return value && ok;
}

conditional_command :: (l: *Loop, name: string) -> Command_Status {
    s := *l.scanner;
    active := branch_active(l);

    if name == "if" {
        level := If_Level.{ state = .IGNORED };

        if active {
            s.active = true;
            level.state = ifx boolean_expression(s, name) then If_State.TRUE else .FALSE;
        }

        save_branch(l, *level);
        array_add(*l.levels, level);

        return .SKIP_LINE;
    }

    if !l.levels.count {
        print_error(tprint("\\%: no matching \\if", name));
        return .ERROR;
    }

    level := *l.levels[l.levels.count - 1];

    if name == "elif" {
        if level.state == {
            case .TRUE;
                save_branch(l, level);
                level.state = .IGNORED;
            case .FALSE;
                discard_branch(l, level);
                s.active = true;
                level.state = ifx boolean_expression(s, name) then If_State.TRUE else .FALSE;
            case .IGNORED;
                discard_branch(l, level);
            case;
                print_error("\\elif: cannot occur after \\else");
                return .ERROR;
        }
    } else if name == "else" {
        if level.state == {
            case .TRUE;
                save_branch(l, level);
                level.state = .ELSE_FALSE;
            case .FALSE;
                discard_branch(l, level);
                level.state = .ELSE_TRUE;
            case .IGNORED;
                discard_branch(l, level);
                level.state = .ELSE_FALSE;
            case;
                print_error("\\else: cannot occur after \\else");
                return .ERROR;
        }
    } else {
        if level.state == .FALSE || level.state == .IGNORED || level.state == .ELSE_FALSE discard_branch(l, level);
        l.levels.count -= 1;
    }

    return .SKIP_LINE;
}

next_arg :: (s: *Scanner, kind := Option_Kind.NORMAL, semicolon := false) -> string, bool {
    arg, found := scan_slash_option(s, kind, semicolon);
    return copy_string(arg,, temp), found;
}

run_command :: (l: *Loop, name: string) -> Command_Status {
    s := *l.scanner;

    if name == {
        case "echo"; #through;
        case "qecho"; #through;
        case "warn";
            return echo_command(s, name);

        case "set";
            var, found := next_arg(s);
            if !found {
                for psql.vars.entries write_out(tprint("% = '%'\n", it.name, it.value));
                return .SKIP_LINE;
            }

            parts: [..] string;
            parts.allocator = temp;

            while true {
                arg, more := next_arg(s);
                if !more break;
                array_add(*parts, arg);
            }

            if !set_variable(var, join(..parts, "",, temp)) return .ERROR;

            return .SKIP_LINE;

        case "unset";
            var, found := next_arg(s);
            if !found {
                print_error("\\unset: missing required argument");
                return .ERROR;
            }

            unset_variable(var);

            return .SKIP_LINE;

        case "getenv";
            var, found := next_arg(s);
            env, found2 := next_arg(s);

            if !found || !found2 {
                print_error("\\getenv: missing required argument");
                return .ERROR;
            }

            value := environment(env);
            if value && !set_variable(var, value) return .ERROR;

            return .SKIP_LINE;

        case "setenv";
            var, found := next_arg(s);
            value, has_value := next_arg(s);

            if !found {
                print_error("\\setenv: missing required argument");
                return .ERROR;
            }

            if has_value POSIX.setenv(temp_c_string(var), temp_c_string(value), 1);
            else POSIX.unsetenv(temp_c_string(var));

            return .SKIP_LINE;

        case "q"; #through;
        case "quit";
            return .TERMINATE;

        case "r"; #through;
        case "reset";
            l.query.count = 0;
            scan_reset(s);
            if !psql.quiet write_out("Query buffer reset (cleared).\n");

            return .SKIP_LINE;

        case "p"; #through;
        case "print";
            text := ifx l.query.count then buffer_string(l.query) else l.previous;
            if text write_out(tprint("%\n", text));
            else if !psql.quiet write_out("Query buffer is empty.\n");

            return .SKIP_LINE;

        case "g"; #through;
        case "gx";
            arg, found := next_arg(s, .FILE_OR_PIPE);
            if found && begins_with(arg, "(") {
                // \g (option=value ...): print settings for this query only.
                if !g_options(s, advance(arg, 1), name) return .ERROR;
                arg, found = next_arg(s, .FILE_OR_PIPE);
            }

            if found && begins_with(arg, "|") {
                psql.gpipe = copy_string(advance(arg, 1));
                psql.gfile = tprint("%/psql-pipe-%", ifx environment("TMPDIR") then environment("TMPDIR") else "/tmp", POSIX.getpid());
            } else if found {
                psql.gfile = arg;
            }

            if name == "gx" {
                if !psql.restore_popt {
                    psql.saved_popt = psql.popt;
                    psql.restore_popt = true;
                }

                psql.popt.expanded = .ON;
            }

            return .SEND;

        case "gset";
            prefix, found := next_arg(s);

            psql.gset_prefix = ifx found then copy_string(prefix) else "";
            psql.gset_active = true;

            return .SEND;

        case "gexec";
            psql.gexec = true;
            return .SEND;

        case "crosstabview";
            for *psql.crosstab_args {
                arg, found := next_arg(s);
                if found it.* = copy_string(arg);
            }

            psql.crosstab = true;

            return .SEND;

        case "gdesc";
            psql.gdesc = true;
            return .SEND;

        case "bind";
            array_reset(*psql.bind_params);

            while true {
                arg, found := next_arg(s);
                if !found break;
                array_add(*psql.bind_params, copy_string(arg));
            }

            psql.bind_active = true;

            return .SKIP_LINE;

        case "H"; #through;
        case "html";
            if psql.popt.format != .HTML return pset_command("format", "html", true);

            return pset_command("format", "aligned", true);

        case "T";
            value, found := next_arg(s);

            return pset_command("tableattr", value, found);

        case "x";
            value, found := next_arg(s);

            return pset_command("expanded", value, found);

        case "t";
            value, found := next_arg(s);

            return pset_command("tuples_only", value, found);

        case "a";
            if psql.popt.format != .ALIGNED return pset_command("format", "aligned", true);

            return pset_command("format", "unaligned", true);

        case "f";
            value, found := next_arg(s);

            return pset_command("fieldsep", value, found);

        case "C";
            value, found := next_arg(s);

            return pset_command("title", value, found);

        case "pset";
            option, found := next_arg(s);
            if !found {
                show_all_pset();
                return .SKIP_LINE;
            }

            value, has_value := next_arg(s);

            return pset_command(option, value, has_value);

        case "c"; #through;
        case "connect";
            return connect_command(s);

        case "errverbose";
            if psql.last_error.kind == .NONE {
                write_out("There is no previous error.\n");
                return .SKIP_LINE;
            }

            saved := psql.verbosity;
            saved_context := psql.show_context;

            psql.verbosity = .VERBOSE;
            psql.show_context = .ALWAYS;
            print_error(trim_right(build_error_message(psql.last_error, psql.last_error_query, true), "\n"));
            psql.verbosity = saved;
            psql.show_context = saved_context;

            return .SKIP_LINE;

        case "password";
            return password_command(s);

        case "encoding";
            value, found := next_arg(s);
            if !found {
                write_out(tprint("%\n", Pg.parameter(psql.db, "client_encoding")));
                return .SKIP_LINE;
            }

            sql := tprint("set client_encoding to %", escape_literal(value));
            error := Pg.execute_script(psql.db, sql);

            if error.kind != .NONE {
                print_error(tprint("%: invalid encoding name or conversion procedure not found", value));
                Pg.free_error(*error);
                return .ERROR;
            }

            variables_set(*psql.vars, "ENCODING", Pg.parameter(psql.db, "client_encoding"));

            return .SKIP_LINE;

        case "lo_import"; #through;
        case "lo_export"; #through;
        case "lo_unlink";
            return large_object_command(s, name);

        case "!";
            line, _ := next_arg(s, .WHOLE_LINE);

            return shell_command(line);

        case "copy";
            args, _ := next_arg(s, .WHOLE_LINE);
            if !copy_command(args) return .ERROR;

            return .SKIP_LINE;

        case "timing";
            value, found := next_arg(s);

            if !found {
                psql.timing = !psql.timing;
            } else if value == "on" || value == "true" || value == "1" {
                psql.timing = true;
            } else if value == "off" || value == "false" || value == "0" {
                psql.timing = false;
            } else {
                print_error(tprint("unrecognized value \"%\" for \"\\timing\": Boolean expected", value));
                return .ERROR;
            }

            write_out(ifx psql.timing then "Timing is on.\n" else "Timing is off.\n");

            return .SKIP_LINE;

        case "copyright";
            write_out("PostgreSQL Database Management System\n(formerly known as Postgres, then as Postgres95)\n\nPortions Copyright (c) 1996-2024, PostgreSQL Global Development Group\n\nPortions Copyright (c) 1994, The Regents of the University of California\n\nPermission to use, copy, modify, and distribute this software and its\ndocumentation for any purpose, without fee, and without a written agreement\nis hereby granted, provided that the above copyright notice and this\nparagraph and the following two paragraphs appear in all copies.\n");
            return .SKIP_LINE;

        case "restrict";
            key, found := next_arg(s, .WHOLE_LINE);

            if !found || !key {
                print_error("\\restrict: missing required argument");
                return .ERROR;
            }

            psql.restricted = true;
            psql.restrict_key = copy_string(key);

            return .SKIP_LINE;

        case "unrestrict";
            key, _ := next_arg(s, .WHOLE_LINE);

            if !psql.restricted {
                print_error("\\unrestrict: not currently in restricted mode");
                return .ERROR;
            }

            if key != psql.restrict_key {
                print_error("\\unrestrict: wrong key");
                return .ERROR;
            }

            psql.restricted = false;

            return .SKIP_LINE;

        case "watch";
            return watch_command(l, s);

        case "o"; #through;
        case "out";
            return out_command(s);

        case "i"; #through;
        case "include"; #through;
        case "ir"; #through;
        case "include_relative";
            return include_command(s, name);
    }

    if begins_with(name, "d") || begins_with(name, "s") || begins_with(name, "lo_list") || name == "z" || name == "l" || name == "l+" || name == "list" || name == "list+" {
        status, handled := describe_command(l, name);
        if handled return status;
    }

    print_error(tprint("invalid command \\%", name));

    return .ERROR;
}

// \echo, \qecho and \warn: the arguments joined by spaces; a leading unquoted -n drops the newline.
echo_command :: (s: *Scanner, name: string) -> Command_Status {
    parts: [..] string;
    parts.allocator = temp;
    newline := true;
    first := true;

    while true {
        arg, found, quote := scan_slash_option(s, .NORMAL);
        if !found break;

        if first && arg == "-n" && quote == 0 {
            newline = false;
        } else {
            array_add(*parts, copy_string(arg,, temp));
        }

        first = false;
    }

    text := join(..parts, " ",, temp);
    if newline text = tprint("%\n", text);

    if name == "warn" write_fd(2, text);
    else if name == "qecho" write_out(text);
    else write_fd(1, text);

    return .SKIP_LINE;
}

// \o: later query output goes to the file, or back to stdout without one.
out_command :: (s: *Scanner) -> Command_Status {
    file, found := next_arg(s, .FILE_OR_PIPE);

    if out_file_open file_close(*out_file);

    out_file_open = false;

    if found && file {
        if begins_with(file, "|") {
            print_error("\\o: pipes are not supported by this shim");
            return .ERROR;
        }

        out_file, out_file_open = file_open(file, for_writing = true, keep_existing_content = false);
        if !out_file_open {
            print_error(tprint("%: could not open file", file));
            return .ERROR;
        }
    }

    return .SKIP_LINE;
}

// \i and \ir: runs a file's commands as if typed here.
include_command :: (s: *Scanner, name: string) -> Command_Status {
    file, found := next_arg(s);
    if !found {
        print_error(tprint("\\%: missing required argument", name));
        return .ERROR;
    }

    text, ok := read_entire_file(file, log_errors = false);
    if !ok {
        print_error(tprint("%: No such file or directory", file));
        return .ERROR;
    }

    result := main_loop(text, input_name = canonical_path(file));
    if result == EXIT_USER return .ERROR;

    return .SKIP_LINE;
}

// Runs one query for a backslash command (psql's PSQLexec): its error is printed as psql prints
// it and null returned. Free the result with `free_query_result`.
psql_exec :: (sql: string) -> *Pg.Result {
    Exec_Hooks :: struct {
        #as using hooks: Pg.Script_Hooks;
        result: *Pg.Result;
    }

    hooks: Exec_Hooks;

    hooks.on_result = (h: *Pg.Script_Hooks, result: *Pg.Result) {
        e := cast(*Exec_Hooks) h;
        if e.result free_query_result(e.result);

        e.result = New(Pg.Result);
        e.result.* = result.*;
        result.* = .{};
    };

    if !psql.db {
        print_error("You are currently not connected to a database.");
        return null;
    }

    error := Pg.run_script(psql.db, sql, *hooks);
    if error.kind != .NONE {
        print_error_raw(build_error_message(error, sql, true));
        Pg.free_error(*error);
        if hooks.result free_query_result(hooks.result);
        return null;
    }

    return hooks.result;
}

free_query_result :: (result: *Pg.Result) {
    Pg.free_result(result);
    free(result);
}

// The options of `\g (name=value ...)`, one argument each, up to the one ending in `)`. The
// settings before them come back after the query.
g_options :: (s: *Scanner, first: string, command: string) -> bool {
    option := first;
    is_first := true;
    success := true;

    while true {
        if !is_first {
            arg, found := next_arg(s);
            if !found {
                print_error(tprint("\\%: missing right parenthesis", command));
                success = false;
                break;
            }

            option = arg;
        }

        is_first = false;
        closed := ends_with(option, ")");
        if closed option.count -= 1;

        if option {
            has_value, key, value := split_from_left(option, #char "=");
            if !has_value key = option;

            if !psql.restore_popt {
                psql.saved_popt = psql.popt;
                psql.restore_popt = true;
            }

            success = pset_apply(key, value, has_value, quiet = true) && success;
        }

        if closed break;
    }

    if !success && psql.restore_popt {
        psql.popt = psql.saved_popt;
        psql.restore_popt = false;
    }

    return success;
}

connect_command :: (s: *Scanner) -> Command_Status {
    dbname, has_db := next_arg(s, .SQL_ID_KEEP_CASE);
    user, has_user := next_arg(s, .SQL_ID_KEEP_CASE);
    host, has_host := next_arg(s, .SQL_ID_KEEP_CASE);
    port, has_port := next_arg(s, .SQL_ID_KEEP_CASE);

    keep :: (value: string, found: bool, current: string) -> string {
        if !found || value == "-" return current;
        return value;
    }

    connection_string := "";

    if has_db && Pg.is_conninfo(dbname) {
        connection_string = dbname;
        has_db = false;
    }

    new_db := keep(dbname, has_db, psql.config.database);
    new_user := keep(user, has_user, psql.config.user);
    new_host := keep(host, has_host, psql.config.host);
    new_port := keep(port, has_port, tprint("%", psql.config.port));

    if connect_to(connection_string, copy_string(new_db), copy_string(new_user), copy_string(new_host), copy_string(new_port), keep_previous = true) {
        return .ERROR;
    }

    if !psql.quiet {
        if begins_with(psql.config.host, "/") {
            write_out(tprint("You are now connected to database \"%\" as user \"%\" via socket in \"%\" at port \"%\".\n", psql.config.database, psql.config.user, psql.config.host, psql.config.port));
        } else {
            write_out(tprint("You are now connected to database \"%\" as user \"%\" on host \"%\" at port \"%\".\n", psql.config.database, psql.config.user, psql.config.host, psql.config.port));
        }
    }

    return .SKIP_LINE;
}

// \pset

apply_pset_option :: (text: string) {
    found, option, value := split_from_left(text, #char "=");
    pset_apply(option, value, found, quiet = true);
}

pset_command :: (option: string, value: string, has_value: bool) -> Command_Status {
    if !pset_apply(option, value, has_value, psql.quiet) return .ERROR;
    return .SKIP_LINE;
}

// One \pset option. Without a value a boolean option toggles. Prints the new setting unless quiet.
// `\pset` alone: every setting, one per line.
show_all_pset :: () {
    o := psql.popt;

    quoted :: (text: string) -> string {
        builder: String_Builder;
        builder.allocator = temp;
        append(*builder, "'");

        for text {
            if it == #char "'" append(*builder, "\\'");
            else if it == #char "\\" append(*builder, "\\\\");
            else if it == #char "\n" append(*builder, "\\n");
            else if it == #char "\r" append(*builder, "\\r");
            else if it == #char "\t" append(*builder, "\\t");
            else append(*builder, it);
        }

        append(*builder, "'");

        return builder_to_string(*builder,, temp);
    }

    on_off :: (b: bool) -> string {
        return ifx b then "on" else "off";
    }

    expanded := ifx o.expanded == .AUTO then "auto" else on_off(o.expanded == .ON);
    linestyle := "ascii";

    if o.linestyle == .OLD_ASCII linestyle = "old-ascii";
    if o.linestyle == .UNICODE linestyle = "unicode";

    rows := string.[
        "border", tprint("%", o.border),
        "columns", tprint("%", o.columns),
        "csv_fieldsep", quoted(o.csv_field_separator),
        "expanded", expanded,
        "fieldsep", ifx o.field_separator_zero then "zero" else quoted(o.field_separator),
        "fieldsep_zero", on_off(o.field_separator_zero),
        "footer", on_off(o.footer),
        "format", format_name(o.format),
        "linestyle", linestyle,
        "null", quoted(o.null_display),
        "numericlocale", "off",
        "pager", "1",
        "pager_min_lines", "0",
        "recordsep", ifx o.record_separator_zero then "zero" else quoted(o.record_separator),
        "recordsep_zero", on_off(o.record_separator_zero),
        "tableattr", ifx o.table_attr then quoted(o.table_attr) else "",
        "title", ifx o.title then quoted(o.title) else "",
        "tuples_only", on_off(o.tuples_only),
        "unicode_border_linestyle", "single",
        "unicode_column_linestyle", "single",
        "unicode_header_linestyle", "single",
        "xheader_width", "full",
    ];

    i := 0;

    while i < rows.count {
        name := rows[i];
        line := tprint("%1%2 %3\n", name, repeat_char(#char " ", 24 - name.count), rows[i + 1]);
        write_out(line);
        i += 2;
    }
}

repeat_char :: (c: u8, n: s64) -> string {
    if n <= 0 return "";
    text := talloc_string(n);
    for 0..n - 1 text[it] = c;
    return text;
}

pset_apply :: (option: string, value: string, has_value: bool, quiet: bool) -> bool {
    o := *psql.popt;

    toggle :: (current: bool, name: string) -> bool, bool #expand {
        if !has_value return !current, true;
        v, ok := parse_bool(value, name);
        return v, ok;
    }

    if option == {
        case "format";
            if !pset_format(value, has_value, quiet) return false;

        case "border";
            if has_value {
                n, ok := string_to_int(value);
                o.border = xx ifx ok then n else 0;
            }

            if !quiet write_out(tprint("Border style is %.\n", o.border));

        case "expanded"; #through;
        case "x";
            if !pset_expanded(value, has_value, quiet) return false;

        case "null";
            if has_value o.null_display = copy_string(value);
            if !quiet write_out(tprint("Null display is \"%\".\n", o.null_display));

        case "fieldsep";
            if has_value {
                o.field_separator = copy_string(value);
                o.field_separator_zero = false;
            }

            if !quiet {
                if o.field_separator_zero write_out("Field separator is zero byte.\n");
                else write_out(tprint("Field separator is \"%\".\n", o.field_separator));
            }

        case "fieldsep_zero";
            o.field_separator_zero = true;
            if !quiet write_out("Field separator is zero byte.\n");

        case "recordsep_zero";
            o.record_separator_zero = true;
            if !quiet write_out("Record separator is zero byte.\n");

        case "tableattr"; #through;
        case "T";
            o.table_attr = ifx has_value then copy_string(value) else "";

            if !quiet {
                if has_value write_out(tprint("Table attributes are \"%\".\n", o.table_attr));
                else write_out("Table attributes unset.\n");
            }

        case "recordsep";
            if has_value {
                o.record_separator = copy_string(value);
                o.record_separator_zero = false;
            }

            if !quiet {
                if o.record_separator_zero write_out("Record separator is zero byte.\n");
                else if o.record_separator == "\n" write_out("Record separator is <newline>.\n");
                else write_out(tprint("Record separator is \"%\".\n", o.record_separator));
            }

        case "csv_fieldsep";
            if !pset_csv_fieldsep(value, has_value, quiet) return false;

        case "tuples_only"; #through;
        case "t";
            v, ok := toggle(o.tuples_only, "tuples_only");
            if !ok return false;

            o.tuples_only = v;
            if !quiet write_out(ifx v then "Tuples only is on.\n" else "Tuples only is off.\n");

        case "footer";
            v, ok := toggle(o.footer, "footer");
            if !ok return false;

            o.footer = v;
            if !quiet write_out(ifx v then "Default footer is on.\n" else "Default footer is off.\n");

        case "title"; #through;
        case "C";
            o.title = ifx has_value then copy_string(value) else "";

            if !quiet {
                if o.title write_out(tprint("Title is \"%\".\n", o.title));
                else write_out("Title is unset.\n");
            }

        case "columns";
            if has_value {
                n, ok := string_to_int(value);
                o.columns = xx ifx ok then n else 0;
            }

            if !quiet {
                if o.columns write_out(tprint("Target width is %.\n", o.columns));
                else write_out("Target width is unset.\n");
            }

        case "linestyle";
            if !pset_linestyle(value, has_value, quiet) return false;

        case "pager"; #through;
        case "pager_min_lines"; #through;
        case "numericlocale"; #through;
        case "xheader_width"; #through;
        case "unicode_border_linestyle"; #through;
        case "unicode_column_linestyle"; #through;
        case "unicode_header_linestyle";
        // Not used in a way that changes output in the suites the shim runs.

        case;
            print_error(tprint("\\pset: unknown option: %", option));
            return false;
    }

    return true;
}

// The output format, by name or unambiguous prefix.
pset_format :: (value: string, has_value: bool, quiet: bool) -> bool {
    o := *psql.popt;

    if !has_value {
        if !quiet write_out(tprint("Output format is %.\n", format_name(o.format)));
        return true;
    }

    FORMATS :: struct {
        name: string;
        format: Print_Format;
    } .[
        .{ "aligned", .ALIGNED }, .{ "asciidoc", .ASCIIDOC }, .{ "csv", .CSV },
        .{ "html", .HTML }, .{ "latex", .LATEX }, .{ "troff-ms", .TROFF_MS },
        .{ "unaligned", .UNALIGNED }, .{ "wrapped", .WRAPPED },
    ];

    lower := to_lower_copy(value,, temp);
    match := -1;

    for FORMATS {
        if !begins_with(it.name, lower) continue;

        if match >= 0 {
            print_error(tprint("\\pset: ambiguous abbreviation \"%\" matches both \"%\" and \"%\"", value, FORMATS[match].name, it.name));
            return false;
        }

        match = it_index;
    }

    if match >= 0 {
        o.format = FORMATS[match].format;
    } else if begins_with("latex-longtable", lower) {
        o.format = .LATEX_LONGTABLE;
    } else {
        print_error("\\pset: allowed formats are aligned, asciidoc, csv, html, latex, latex-longtable, troff-ms, unaligned, wrapped");
        return false;
    }

    if !quiet write_out(tprint("Output format is %.\n", format_name(o.format)));

    return true;
}

// Expanded display: on, off or auto; no value toggles on and off.
pset_expanded :: (value: string, has_value: bool, quiet: bool) -> bool {
    o := *psql.popt;

    if has_value {
        if to_lower_copy(value,, temp) == "auto" {
            o.expanded = .AUTO;
        } else {
            v, ok := parse_bool(value, "expanded");
            if !ok return false;
            o.expanded = ifx v then Expanded.ON else .OFF;
        }
    } else {
        o.expanded = ifx o.expanded == .OFF then Expanded.ON else .OFF;
    }

    if !quiet {
        if o.expanded == {
            case .ON; write_out("Expanded display is on.\n");
            case .OFF; write_out("Expanded display is off.\n");
            case .AUTO; write_out("Expanded display is used automatically.\n");
        }
    }

    return true;
}

// The CSV field separator: one byte, not a quote or line break.
pset_csv_fieldsep :: (value: string, has_value: bool, quiet: bool) -> bool {
    o := *psql.popt;

    if has_value {
        if value.count != 1 {
            print_error("\\pset: csv_fieldsep must be a single one-byte character");
            return false;
        }

        if value == "\"" || value == "\n" || value == "\r" {
            print_error("\\pset: csv_fieldsep cannot be a double quote, a newline, or a carriage return");
            return false;
        }

        o.csv_field_separator = copy_string(value);
    }

    if !quiet write_out(tprint("Field separator for CSV is \"%\".\n", o.csv_field_separator));

    return true;
}

// The line style, by name or prefix.
pset_linestyle :: (value: string, has_value: bool, quiet: bool) -> bool {
    o := *psql.popt;

    if has_value {
        lower := to_lower_copy(value,, temp);
        if begins_with("ascii", lower) && lower.count o.linestyle = .ASCII;
        else if begins_with("old-ascii", lower) && lower.count o.linestyle = .OLD_ASCII;
        else if begins_with("unicode", lower) && lower.count o.linestyle = .UNICODE;
        else {
            print_error("\\pset: allowed line styles are ascii, old-ascii, unicode");
            return false;
        }
    }

    if !quiet {
        if o.linestyle == {
            case .ASCII; write_out("Line style is ascii.\n");
            case .OLD_ASCII; write_out("Line style is old-ascii.\n");
            case .UNICODE; write_out("Line style is unicode.\n");
        }
    }

    return true;
}

format_name :: (format: Print_Format) -> string {
    if format == {
        case .ALIGNED; return "aligned";
        case .UNALIGNED; return "unaligned";
        case .WRAPPED; return "wrapped";
        case .CSV; return "csv";
        case .HTML; return "html";
        case .ASCIIDOC; return "asciidoc";
        case .LATEX; return "latex";
        case .LATEX_LONGTABLE; return "latex-longtable";
        case .TROFF_MS; return "troff-ms";
    }

    return "aligned";
}
