// A stand-in for libpq's libpq_uri_regress test program, built on this Postgres client's
// connection string parser, so that PostgreSQL's src/interfaces/libpq/t/001_uri.pl exercises
// the client instead of libpq. It parses its one argument and prints the options that differ
// from their defaults, then whether the connection would use a Unix socket or the network.
// Errors go to standard error, prefixed with the program name, and the exit status is 1.
//
// The output format follows what 001_uri.pl expects: see docs/postgres/conformance.md.
//
//     jaic uri_regress.jai -exe libpq_uri_regress

#import "Basic";
#import "String";
POSIX :: #import "POSIX";
Pg :: #import "Extensions/Postgres";

write_fd :: (fd: s32, text: string) {
    at := 0;

    while at < text.count {
        n := POSIX.write(fd, text.data + at, cast(u64) (text.count - at));
        if n < 0 return;

        at += n;
    }
}

environment :: (name: string) -> string {
    value := POSIX.getenv(temp_c_string(name));
    if !value return "";
    return to_string(value);
}

main :: () {
    args := get_command_line_arguments();

    if args.count != 2 {
        write_fd(2, "Usage: libpq_uri_regress <connection-string>\n");
        exit(1);
    }

    info, error := Pg.parse_conninfo_options(args[1]);

    if error.kind != .NONE {
        write_fd(2, tprint("libpq_uri_regress: %\n", error.message));
        exit(1);
    }

    // sslmode starts as verify-full when the system's roots are in use.
    default_sslmode := ifx environment("PGSSLROOTCERT") == "system" then "verify-full" else "prefer";

    builder: String_Builder;
    first := true;

    for keyword: Pg.CONNINFO_KEYWORDS {
        value, found := Pg.conninfo_get(info, keyword);
        if !found continue;
        if keyword == "port" && value == "5432" continue;
        if keyword == "sslmode" && value == default_sslmode continue;

        if !first append(*builder, " ");

        print_to_builder(*builder, "%='%'", keyword, value);
        first = false;
    }

    host, _ := Pg.conninfo_get(info, "host");
    _, has_address := Pg.conninfo_get(info, "hostaddr");

    // The first host decides, as in libpq.
    first_host := host;
    found_comma, before, _ := split_from_left(host, #char ",");

    if found_comma first_host = before;

    local := !has_address && (!first_host || begins_with(first_host, "/"));

    if !first append(*builder, " ");

    append(*builder, ifx local then "(local)" else "(inet)");
    append(*builder, "\n");

    write_fd(1, builder_to_string(*builder));
}
