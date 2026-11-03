// Postgres access for the HTTP handlers. A `Pg.Pool` belongs to one scheduler and the server runs
// one scheduler per thread, so the first request on each thread makes that thread's pool. A
// handler borrows a connection with `borrow`, gives it back with `Pg.release`, and everything it
// reads out of a `Pg.Result` is copied into temporary storage before the result is freed.

SCHEMA :: #string END
create table if not exists accounts (
    id bigserial primary key,
    name text not null unique,
    token text not null unique,
    created_at timestamptz not null default now()
);

create table if not exists matches (
    id bigserial primary key,
    account_id bigint not null references accounts (id),
    seed bigint not null,
    score bigint not null,
    kills bigint not null,
    duration_ms bigint not null,
    created_at timestamptz not null default now()
);

create index if not exists matches_by_score on matches (score desc, id);
create index if not exists matches_by_account on matches (account_id);
END

Thread_Pool :: struct {
    thread_index: u32;
    pool: *Pg.Pool;
}

Database :: struct {
    config: Pg.Config;
    settings: Pg.Pool_Config;
    lock: Mutex;
    pools: [..] Thread_Pool;
}

// Connects once, makes sure the tables exist and remembers how to make pools. Runs on a fiber.
database_open :: (db: *Database, url: string, pool_size: int) -> string {
    config, parse_error := Pg.parse_url(url);

    if parse_error.kind != .NONE return tprint("bad database url: %", Pg.error_text(parse_error));

    db.config = config;
    db.settings.max_size = pool_size;
    db.settings.min_idle = 0;

    init(*db.lock);

    connection, error := Pg.connect(config);

    if error.kind != .NONE return tprint("cannot connect to the database: %", Pg.error_text(error));

    defer Pg.close(connection);

    error = Pg.execute_script(connection, SCHEMA);

    if error.kind != .NONE return tprint("cannot create the schema: %", Pg.error_text(error));

    return "";
}

pool_of_thread :: (db: *Database) -> *Pg.Pool, Pg.Pg_Error {
    lock(*db.lock);
    defer unlock(*db.lock);

    for db.pools if it.thread_index == context.thread_index return it.pool, .{};

    pool, error := Pg.create_pool(db.config, db.settings);

    if error.kind != .NONE return null, error;

    array_add(*db.pools, .{ context.thread_index, pool });

    return pool, .{};
}

// A connection for the calling fiber, or the HTTP error to answer with.
borrow :: (db: *Database) -> *Pg.Connection, Http_Error {
    pool, pool_error := pool_of_thread(db);

    if pool_error.kind != .NONE return null, internal_error("database pool: %", Pg.error_text(pool_error));

    connection, error := Pg.acquire(pool);

    if error.kind == .TIMED_OUT return null, unavailable("all database connections are busy");
    if error.kind != .NONE return null, unavailable("database: %", Pg.error_text(error));

    return connection, .{};
}

give_back :: (db: *Database, connection: *Pg.Connection) {
    pool, _ := pool_of_thread(db);

    Pg.release(pool, connection);
}

database_failed :: (error: Pg.Pg_Error) -> Http_Error {
    return internal_error("database: %", Pg.error_text(error));
}

// One row of one `bigint` column, or `fallback` when the query returned no row.
scalar :: (connection: *Pg.Connection, sql: string, fallback: s64, args: ..Any) -> s64, Http_Error {
    result, error := Pg.query_dynamic(connection, sql, ..args);

    defer Pg.free_result(*result);

    if error.kind != .NONE return fallback, database_failed(error);
    if result.row_count == 0 return fallback, .{};

    value, decode_error := Pg.get(*result, 0, 0, s64);

    if decode_error.kind != .NONE return fallback, database_failed(decode_error);

    return value, .{};
}

#scope_file

#import "Thread";
