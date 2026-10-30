// The backend of the game: accounts, matches and a leaderboard over a JSON API, stored in Postgres.
//
//     jaic examples/game2d/server/main.jai -exe game2d-server
//     DATABASE_URL=postgres://postgres@127.0.0.1/postgres ./game2d-server --listen 127.0.0.1:8080
//     ./game2d-server --tls-cert cert.pem --tls-key key.pem     # HTTPS
//
// The routes and documents are in ../shared, the handlers in routes.jai and the SQL in database.jai.

#import "Basic";
#import "String";
#import "File";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";
Tls :: #import "Extensions/Tls";
Args :: #import "Extensions/Args";
#import "Extensions/Http";
#import,dir "../shared";

#load "database.jai";
#load "tokens.jai";
#load "routes.jai";

Cli :: struct {
    listen: string = "127.0.0.1:8080"; @short=l @value=ADDRESS
    database: string; @short=d @env=DATABASE_URL @value=URL
    threads: int = 4; @short=t @range(1,64)
    pool_size: int = 8; @range(1,100)
    tls_cert: string; @value=FILE
    tls_key: string; @value=FILE
    log_requests: bool;
}

HELP :: Args.Help(Cli).{
    about = "The game2d backend: accounts, matches and leaderboards over JSON.",
    version = "0.1.0",
    listen = "Address to listen on",
    database = "Postgres URL (or set DATABASE_URL)",
    threads = "Scheduler threads serving requests",
    pool_size = "Database connections per thread",
    tls_cert = "PEM certificate chain: serve HTTPS (with --tls-key)",
    tls_key = "PEM private key for --tls-cert",
    log_requests = "Print one line per request",
};

Setup :: struct {
    server: *Game_Server;
    url: string;
    pool_size: int;
    failure: string;
}

// Runs once on a fiber: connects, creates the tables.
prepare :: (setup: *Setup) {
    setup.failure = copy_string(database_open(*setup.server.db, setup.url, setup.pool_size));
}

fail :: (message: string) {
    print("game2d-server: %\n", message);
    exit(1);
}

main :: () {
    cli, _ := Args.parse(Cli, HELP);

    if !cli.database fail("no database: pass --database or set DATABASE_URL");
    if (cli.tls_cert != "") != (cli.tls_key != "") fail("--tls-cert and --tls-key go together");

    server: Game_Server;

    if !token_source_init(*server.tokens) fail("cannot read /dev/urandom");

    setup := Setup.{ *server, cli.database, cli.pool_size, "" };

    Io.io_run(prepare, *setup);

    if setup.failure fail(setup.failure);

    app: App;

    server_routes(*app, *server);

    app.config.address = cli.listen;
    app.config.threads = cli.threads;
    app.config.log_requests = cli.log_requests;

    tls: Tls.Tls_Config;

    if cli.tls_cert != "" {
        chain, chain_read := read_entire_file(cli.tls_cert);
        key, key_read := read_entire_file(cli.tls_key);

        if !chain_read || !key_read fail("cannot read the certificate or the key");

        tls.certificate_chain_pem = chain;
        tls.private_key_pem = key;
        tls.alpn = string.["http/1.1"];

        tls_error := Tls.tls_config_init(*tls);

        if tls_error.kind != .NONE fail(tprint("tls: %", Tls.error_text(tls_error)));

        app.config.tls = *tls;
    }

    scheme := ifx cli.tls_cert != "" then "https" else "http";

    print("game2d-server listening on %://% (% threads)\n", scheme, cli.listen, cli.threads);

    error := serve(*app);

    if error.kind != .NONE fail(error_text(error));
}
