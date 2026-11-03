// A PostgreSQL client over Io fibers.
// @experimental
//
// ```jai,no_run,skip_on(wasm)
// Io :: #import "Extensions/Io";
// Pg :: #import "Extensions/Postgres";
//
// main :: () {
//     Io.io_run(() {
//         config, error := Pg.parse_url("postgres://app@localhost/app_dev");
//         db:, error = Pg.connect(config);
//         if error.kind != .NONE {
//             print("%\n", Pg.error_text(error));
//             return;
//         }
//         defer Pg.close(db);
//
//         result:, error = Pg.query_dynamic(db, "SELECT id, name FROM users WHERE id > $1", 10);
//         defer Pg.free_result(*result);
//         for row: 0..result.row_count - 1 {
//             id := Pg.get(*result, row, 0, s64);
//             name := Pg.get(*result, row, 1, string);
//         }
//     });
// }
// ```
//
// Every call that talks to the server runs on a fiber inside `Io.io_run` and blocks only that
// fiber. `Io.with_deadline` bounds any of them.
//
// Set the module parameters when importing:
//
// ```jai,skip_on(wasm)
// Pg :: #import "Extensions/Postgres"(MIGRATIONS = "sql/migrations", SCHEMA = "sql/schema.jai", QUERY_CACHE = "sql/cache", MIN_SERVER_VERSION = 15);
// ```

#module_parameters(
    // Connects over TLS, which needs Extensions/Tls (mbedTLS). Set `false` to leave it out of programs that only use Unix sockets or cleartext TCP.
    TLS := true,
    // The migrations folder, relative to the project root.
    MIGRATIONS := "db/migrations",
    // The schema file, relative to the project root.
    SCHEMA := "db/schema.jai",
    // The folder of the typed-query cache, relative to the project root.
    QUERY_CACHE := "db/queries",
    // The oldest server major version accepted; an older server fails every describe.
    MIN_SERVER_VERSION := 0
);

#assert OS != .WASM "Extensions/Postgres runs on fibers over Io, and WebAssembly has neither";

#load "error.jai";
#load "config.jai";
#load "conninfo.jai";
#load "saslprep.jai";
#load "auth.jai";
#load "connection.jai";
#load "query.jai";
#load "statements.jai";
#load "copy.jai";
#load "codecs.jai";
#load "types.jai";
#load "pool.jai";
#load "pipeline.jai";
#load "notify.jai";
#load "listener.jai";
#load "trace.jai";
#load "migrate.jai";
#load "managed.jai";

#scope_module

#import "Basic";
#import "String";
#import "Math";
#import "File";
Io :: #import "Extensions/Io";
Json_Module :: #import "Extensions/Json";

#load "json_options.jai";

#if TLS {
    Tls :: #import "Extensions/Tls";
}

Sha :: #import "internal/crypto/sha";
Hmac :: #import "internal/crypto/hmac";
Random :: #import "internal/crypto/random";
Md5 :: #import "md5";

#if OS != .WINDOWS {
    POSIX :: #import "POSIX";
}

Base64 :: #import "Base64";
System :: #import "System";
#import "Extensions/Environment";

#load "wire.jai";
#load "startup.jai";
#load "tls_name.jai";
#load "scram.jai";
#load "sql_scan.jai";
#load "entry.jai";
#load "describe.jai";
#load "schema_databases.jai";
#load "resolve.jai";
#load "dotenv.jai";
#load "orm.jai";
#load "shape.jai";
#load "typed.jai";
#load "schema.jai";
#load "generate.jai";
#load "diff.jai";
