// The describe service: a compile-time service (see docs/extensions/service.md) that answers
// "what does this statement take and return?" from a live database. The compiler starts it with
// `jaic -interp` the first time typed query code asks, and keeps it running, so one connection
// serves every query of a build (or of an editing session).
//
// A request is the JSON of a `Describe_Request`; the answer is the JSON `describe_answer` writes.
// A database that cannot be reached is `.UNAVAILABLE`, which the caller answers from its cache.
// See docs/postgres/typed-queries.md.

#import "Basic";
#import "String";
Io :: #import "Extensions/Io";
Json :: #import "Extensions/Json";
Service :: #import "Extensions/Service";
Pg :: #import "Extensions/Postgres";

main :: () {
    Io.io_run(() {
        Service.serve(answer);
    });
}

// A connection kept between requests, and the backoff for reaching it again after a failure.
// `database` empty is the URL's own database; otherwise a schema database on the same server.
Held :: struct {
    url: string;
    database: string;
    db: *Pg.Connection;
    failed_until: s64; // monotonic ms: until then the last failure is the answer
    backoff: s64;
    failure: string;
}

held: [..] Held;

RECONNECT_FIRST :: 1_000;
RECONNECT_MAX :: 30_000;

schemas: Pg.Schema_Databases;

answer :: (request_text: string) -> Service.Status, string {
    request, error := Json.from_json(Pg.Describe_Request, request_text, .{ missing_fields_default = true },, temp);
    if error.kind != .NONE return .FAILED, tprint("malformed request: %", error.message);

    if request.op == "ping" return .OK, "pong";

    if !request.url && request.managed {
        // An empty managed server has no schema: only the migrations can give it one.
        if !request.migrations return .FAILED, "managed mode needs migrations: add SQL files to db/migrations, or set DATABASE_URL";

        url, failure := managed_url(request.managed_version);
        if !url return .UNAVAILABLE, failure;

        request.url = url;
    }

    if !request.url return .UNAVAILABLE, "no database: set JACKET_DATABASE_URL or DATABASE_URL, or JACKET_PG_MANAGED=1 (or managed = true under [postgres] in jacket.toml)";

    // A connection that went stale between requests is replaced once.
    for attempt: 0..1 {
        db, failure := connection_for(request.url, "");

        if !db {
            // The managed server may have been stopped: start it again for the next request.
            if request.url == managed.url managed.ready = false;

            return .UNAVAILABLE, failure;
        }

        // It parsed when the connection was made.
        config, config_error := Pg.parse_url(request.url);
        if config_error.kind != .NONE return .UNAVAILABLE, Pg.error_text(config_error);

        target, problem := Pg.schema_target(*schemas, db, config, request.migrations);
        if problem && db.broken {
            drop_connection(request.url, "");
            if attempt == 1 return .UNAVAILABLE, problem;
            continue;
        }

        if problem return .FAILED, problem;

        if target.built {
            for held if it.database != target.database && begins_with(it.database, "jacket_") drop_connection(it.url, it.database);
            Pg.prune_schema_databases(db);
        }

        answered := db;

        if target.database {
            answered, failure = connection_for(request.url, target.database);
            if !answered return .UNAVAILABLE, failure;
        }

        request.setup = target.setup;
        reply: string;

        if request.op == "fingerprints" reply = Pg.fingerprints_answer(answered, request);
        else if request.op == "describe" reply = Pg.describe_answer(answered, request);
        else return .FAILED, tprint("unknown operation \"%\"", request.op);

        if answered != db && answered.broken {
            drop_connection(request.url, target.database);
            if attempt == 1 return .UNAVAILABLE, "the database connection broke while describing";
            continue;
        }

        if !db.broken return .OK, reply;

        drop_connection(request.url, "");
        if attempt == 1 return .UNAVAILABLE, "the database connection broke while describing";
    }

    return .FAILED, "unreachable";
}

// The managed server this service process owns (managed.jai). It is started by the first request
// that needs it and then reused; `start_managed` takes a lock, so two services (two compiles, or
// a compile and the editor) never start the same data folder at once. The server keeps running
// when the service exits, so the next build finds it ready.
Managed_State :: struct {
    ready: bool;
    url: string;
    failed_until: s64; // monotonic ms: until then the last failure is the answer
    failure: string;
}

managed: Managed_State;

managed_url :: (version: string) -> url: string, failure: string {
    if managed.ready return managed.url, "";

    if now_milliseconds() < managed.failed_until return "", managed.failure;

    settings := Pg.managed_settings_from_environment();
    if version settings.version = version;

    server, error := Pg.start_managed(settings);

    if error {
        managed.failure = copy_string(tprint("cannot start the managed PostgreSQL server: %", error));
        managed.failed_until = now_milliseconds() + RECONNECT_MAX;

        return "", managed.failure;
    }

    managed.ready = true;
    managed.url = copy_string(server.url);

    return managed.url, "";
}

now_milliseconds :: () -> s64 {
    return to_milliseconds(current_time_monotonic());
}

slot_for :: (url: string, database: string) -> *Held {
    for *held if it.url == url && it.database == database return it;

    array_add(*held, .{ url = copy_string(url), database = copy_string(database) });

    return *held[held.count - 1];
}

drop_connection :: (url: string, database: string) {
    slot := slot_for(url, database);

    if slot.db {
        Pg.close(slot.db, now = true);
        slot.db = null;
    }
}

connection_for :: (url: string, database: string) -> *Pg.Connection, string {
    slot := slot_for(url, database);
    if slot.db && !slot.db.broken return slot.db, "";

    if slot.db {
        Pg.close(slot.db, now = true);
        slot.db = null;
    }

    // Do not try again before the backoff is over: the editor asks on every keystroke.
    if now_milliseconds() < slot.failed_until return null, slot.failure;

    config, config_error := Pg.parse_url(url);

    if config_error.kind != .NONE {
        slot.failure = copy_string(tprint("the database URL is not usable: %", config_error.message));
        slot.backoff = RECONNECT_MAX;
        slot.failed_until = now_milliseconds() + slot.backoff;

        return null, slot.failure;
    }

    if database config.database = slot.database;

    db, error := Pg.connect(config);

    if error.kind != .NONE {
        slot.backoff = ifx slot.backoff then min(slot.backoff * 2, RECONNECT_MAX) else RECONNECT_FIRST;
        slot.failed_until = now_milliseconds() + slot.backoff;
        slot.failure = copy_string(tprint("could not connect to %:%: %", config.host, config.port, Pg.error_text(error)));

        return null, slot.failure;
    }

    slot.db = db;
    slot.backoff = 0;
    slot.failed_until = 0;

    return db, "";
}
