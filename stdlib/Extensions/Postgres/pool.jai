// A pool of connections for the fibers of one scheduler, and transactions on pooled or plain
// connections.

Pool_Config :: struct {
    max_size := 10;
    // Kept open even when idle; opened in the background, so the first borrowers need not wait.
    min_idle := 0;
    // An idle connection older than this is closed (ms; negative: never).
    idle_timeout := 10 * 60_000;
    // A connection is closed after this long, less up to a tenth so that a fleet started together
    // does not reconnect together (ms; negative: never).
    max_lifetime := 30 * 60_000;
    // A connection idle for longer is checked with an empty Sync before it is handed out (ms;
    // negative: never).
    health_check_after := 30_000;
    // How long `acquire` waits for a free connection (ms; negative: no limit).
    acquire_timeout := 30_000;
}

// Belongs to one scheduler: create it, and use it, on the fibers of one `io_run` (with
// `io_run_threads`, one pool per thread). No locks are involved.
Pool :: struct {
    config: Config;
    settings: Pool_Config;
    // Idle connections, the most recently used last: they are handed out first, so the rest go
    // stale and are closed by `idle_timeout` when the load drops.
    idle: [..] Idle_Connection;
    // Connections open or being opened, idle or borrowed: at most `max_size`.
    open: s64;
    // A connection came back or a slot freed up.
    changed: Io.Condition;
    closed: bool;
    maintaining: bool; // the background fiber runs: the pool is not freed under it
    thread_index: u32;
    maintainer: Io.Fiber_Handle;
}

Idle_Connection :: struct {
    connection: *Connection;
    since: s64; // milliseconds, monotonic
}

// The config is copied. Nothing connects until the first `acquire`, or `min_idle` asks for it.
create_pool :: (config: Config, settings := Pool_Config.{}) -> *Pool, Pg_Error {
    if !Io.in_scheduler() return null, make_error(.USAGE, "Postgres calls must run on a fiber: call them inside Io.io_run");
    if settings.max_size < 1 return null, make_error(.CONFIG, "max_size must be at least 1");
    if settings.min_idle > settings.max_size return null, make_error(.CONFIG, "min_idle is more than max_size");

    pool := New(Pool);
    pool.config = copy_config(config);
    pool.settings = settings;
    pool.thread_index = context.thread_index;
    pool.maintaining = true;
    pool.maintainer = Io.spawn(maintain, pool);

    return pool, .{};
}

// Closes the idle connections and frees the pool once every borrowed one is back.
close_pool :: (pool: *Pool) {
    if !pool || pool.closed return;

    pool.closed = true;
    Io.cancel(pool.maintainer);

    for pool.idle close(it.connection);

    pool.open -= pool.idle.count;
    pool.idle.count = 0;
    Io.notify_all(*pool.changed);

    if pool.open == 0 && !pool.maintaining free_pool(pool);
}

// A connection for this fiber alone, until `release`. Waits for one when `max_size` are in use,
// up to `acquire_timeout` (and any `Io.with_deadline` around the call).
acquire :: (pool: *Pool) -> *Connection, Pg_Error {
    if !pool return null, make_error(.USAGE, "no pool");
    if !Io.in_scheduler() return null, make_error(.USAGE, "Postgres calls must run on a fiber: call them inside Io.io_run");

    assert(pool.thread_index == context.thread_index, "Postgres: a pool is used from the thread that created it");

    c: *Connection;
    error: Pg_Error;

    Io.with_deadline(pool.settings.acquire_timeout, #code {
        c, error = take_connection(pool);
    });

    return c, error;
}

// Gives a connection back. One left inside a transaction is rolled back first; a broken one, or
// one past `max_lifetime`, is closed instead.
release :: (pool: *Pool, c: *Connection) {
    if !c return;

    assert(c.pool == pool, "Postgres: released a connection to a pool it does not come from");
    assert(!c.busy, "Postgres: released a connection while a request is running on it");

    if c.pipeline.active {
        // Statements or results left unread leave the session out of step.
        error := pipeline_exit(*c.pipeline);
        if error.kind != .NONE c.broken = true;

        free_error(*error);
    }

    drop_notifications(c);

    if !c.broken && c.status != .IDLE {
        // The borrower's deadline or cancel may have ended the transaction's work: finish anyway.
        Io.with_grace(c.cancel_timeout, #code {
            error := execute_script(c, "rollback");
            if error.kind != .NONE c.broken = true;
            free_error(*error);
        });
    }

    if pool.closed || c.broken || c.status != .IDLE || expired(pool, c) {
        drop(pool, c);
        return;
    }

    array_add(*pool.idle, .{ c, now_milliseconds() });
    Io.notify_one(*pool.changed);
}

// A round trip that runs nothing: an empty Sync. Fails if the connection is dead.
ping :: (c: *Connection) -> Pg_Error {
    error := begin_request(c, "");
    if error.kind != .NONE return error;

    defer c.busy = false;
    begin_message(c, #char "S");
    end_message(c);
    error = send(c);
    if error.kind != .NONE return error;

    while true {
        type, body, read_error := read_message(c);
        if read_error.kind != .NONE {
            recover_interrupted(c, read_error);
            return read_error;
        }

        if handle_async(c, type, body) continue;

        if type == #char "Z" {
            c.status = transaction_status(ifx body.count then body[0] else 0);
            return .{};
        }

        if type != #char "E" return protocol_error(c, tprint("unexpected message % after Sync", cast(string) u8.[type]));
    }

    return .{};
}

Isolation :: enum u8 {
    DEFAULT; // the server's default_transaction_isolation
    READ_COMMITTED;
    REPEATABLE_READ;
    SERIALIZABLE;
}

// Runs `body` in a transaction on a connection from the pool. The transaction commits when `body`
// returns no error, and rolls back when it returns one (which comes out of `transaction`) or a
// statement in it failed. A serialization failure or deadlock (40001, 40P01) runs `body` again, up
// to `retries` more times, so `body` must be safe to repeat. `data` is passed through, as Jai
// procedures capture nothing.
//
//     Transfer :: struct { from: s64; to: s64; amount: s64; }
//     error := Pg.transaction(pool, Transfer.{ 1, 2, 10 }, (tx: *Pg.Connection, t: Transfer) -> Pg.Pg_Error {
//         _, error := Pg.query_dynamic(tx, "update accounts set balance = balance - $1 where id = $2", t.amount, t.from);
//         if error.kind != .NONE return error;
//         _, error = Pg.query_dynamic(tx, "update accounts set balance = balance + $1 where id = $2", t.amount, t.to);
//         return error;
//     }, .SERIALIZABLE, retries = 3);
transaction :: (pool: *Pool, data: $T, body: (tx: *Connection, data: T) -> Pg_Error, isolation := Isolation.DEFAULT, retries := 0, read_only := false) -> Pg_Error {
    c, error := acquire(pool);
    if error.kind != .NONE return error;

    defer release(pool, c);

    return transaction(c, data, body, isolation, retries, read_only);
}

transaction :: (pool: *Pool, body: (tx: *Connection) -> Pg_Error, isolation := Isolation.DEFAULT, retries := 0, read_only := false) -> Pg_Error {
    return transaction(pool, body, (tx: *Connection, body: (tx: *Connection) -> Pg_Error) -> Pg_Error {
        return body(tx);
    }, isolation, retries, read_only);
}

// The same on a connection of your own.
transaction :: (c: *Connection, data: $T, body: (tx: *Connection, data: T) -> Pg_Error, isolation := Isolation.DEFAULT, retries := 0, read_only := false) -> Pg_Error {
    for attempt: 0..retries {
        error := begin_transaction(c, isolation, read_only);
        if error.kind != .NONE return error;

        error = body(c, data);

        result, retry := finish_transaction(c, *error, attempt < retries);
        if !retry return result;
    }

    return .{};
}

transaction :: (c: *Connection, body: (tx: *Connection) -> Pg_Error, isolation := Isolation.DEFAULT, retries := 0, read_only := false) -> Pg_Error {
    return transaction(c, body, (tx: *Connection, body: (tx: *Connection) -> Pg_Error) -> Pg_Error {
        return body(tx);
    }, isolation, retries, read_only);
}

#scope_module

take_connection :: (pool: *Pool) -> *Connection, Pg_Error {
    while true {
        if pool.closed return null, make_error(.USAGE, "the pool is closed");

        while pool.idle.count {
            entry := pop(*pool.idle);
            c := entry.connection;

            if expired(pool, c) || idle_too_long(pool, entry) {
                drop(pool, c);
                continue;
            }

            check_after := pool.settings.health_check_after;
            if check_after >= 0 && now_milliseconds() - entry.since > check_after {
                error := ping(c);
                if error.kind == .TIMED_OUT || error.kind == .CANCELED {
                    // The borrower ran out of time, not the connection.
                    array_add(*pool.idle, entry);
                    return null, error;
                }

                if error.kind != .NONE {
                    free_error(*error);
                    drop(pool, c);
                    continue;
                }
            }

            return c, .{};
        }

        if pool.open < pool.settings.max_size {
            pool.open += 1;
            c, error := connect(pool.config);

            if error.kind != .NONE {
                slot_freed(pool); // free for another try
                return null, error;
            }

            c.pool = pool;
            c.opened_at = now_milliseconds();

            return c, .{};
        }

        io := Io.wait(*pool.changed);
        if io == .TIMED_OUT return null, make_error(.TIMED_OUT, "no connection was free within acquire_timeout", io);
        if io != .NONE return null, io_failure(io, "the wait for a connection ended");
    }

    return null, .{};
}

drop :: (pool: *Pool, c: *Connection) {
    close(c, now = c.broken);
    slot_freed(pool);
}

// One connection fewer. The last one out of a closed pool frees it.
slot_freed :: (pool: *Pool) {
    pool.open -= 1;

    if pool.closed {
        if pool.open == 0 && !pool.maintaining free_pool(pool);
        return;
    }

    Io.notify_one(*pool.changed);
}

free_pool :: (pool: *Pool) {
    array_free(pool.idle);
    free_config(*pool.config);
    free(pool);
}

LIFETIME_JITTER_DIVISOR :: 10;

// Past `max_lifetime`, less a jitter of up to a tenth: the backend's pid spreads connections
// opened together.
expired :: (pool: *Pool, c: *Connection) -> bool {
    lifetime := pool.settings.max_lifetime;
    if lifetime < 0 return false;

    jitter := ifx lifetime >= LIFETIME_JITTER_DIVISOR then (cast(s64) c.backend_pid) % (lifetime / LIFETIME_JITTER_DIVISOR) else 0;

    return now_milliseconds() - c.opened_at >= lifetime - jitter;
}

idle_too_long :: (pool: *Pool, entry: Idle_Connection) -> bool {
    timeout := pool.settings.idle_timeout;
    return timeout >= 0 && now_milliseconds() - entry.since >= timeout;
}

// Keeps `min_idle` connections open and closes the ones past their time, while the pool lives.
MAINTAIN_INTERVAL :: 1_000;

maintain :: (pool: *Pool) {
    defer {
        pool.maintaining = false;
        if pool.closed && pool.open == 0 free_pool(pool);
    }

    while !pool.closed {
        // Oldest first: the front of `idle` has waited longest.
        i := 0;
        while i < pool.idle.count && pool.idle.count > pool.settings.min_idle {
            entry := pool.idle[i];

            if expired(pool, entry.connection) || idle_too_long(pool, entry) {
                array_ordered_remove_by_index(*pool.idle, i);
                drop(pool, entry.connection);
                continue;
            }

            i += 1;
        }

        while !pool.closed && pool.idle.count < pool.settings.min_idle && pool.open < pool.settings.max_size {
            pool.open += 1;
            c, error := connect(pool.config);

            if error.kind != .NONE {
                free_error(*error);
                slot_freed(pool);
                break; // try again later
            }

            if pool.closed {
                close(c);
                slot_freed(pool);
                break;
            }

            c.pool = pool;
            c.opened_at = now_milliseconds();
            // Oldest place: borrowers take the warm ones first.
            array_insert_at(*pool.idle, .{ c, now_milliseconds() }, 0);
            Io.notify_one(*pool.changed);
        }

        if Io.sleep(MAINTAIN_INTERVAL) != .NONE return; // canceled by close_pool
    }
}

begin_transaction :: (c: *Connection, isolation: Isolation, read_only: bool) -> Pg_Error {
    if !c return make_error(.USAGE, "no connection");
    if c.status != .IDLE return make_error(.USAGE, "the connection is already in a transaction");

    sql := "begin";

    if isolation == {
        case .READ_COMMITTED; sql = "begin isolation level read committed";
        case .REPEATABLE_READ; sql = "begin isolation level repeatable read";
        case .SERIALIZABLE; sql = "begin isolation level serializable";
    }

    if read_only sql = tprint("% read only", sql);

    return execute_script(c, sql);
}

// Commits, or rolls back after a failure. `retry` asks for another run of the body.
finish_transaction :: (c: *Connection, failure: *Pg_Error, can_retry: bool) -> Pg_Error, retry: bool {
    if failure.kind == .NONE && c.status == .IN_TRANSACTION {
        error := execute_script(c, "commit");
        if error.kind == .NONE return .{}, false;

        failure.* = error;
    }

    if failure.kind == .NONE && c.status == .IDLE {
        // The body committed or rolled back by itself.
        return .{}, false;
    }

    if c.status != .IDLE && !c.broken {
        Io.with_grace(c.cancel_timeout, #code {
            error := execute_script(c, "rollback");
            if error.kind != .NONE c.broken = true;
            free_error(*error);
        });
    }

    if failure.kind == .NONE {
        failure.* = make_error(.SERVER, "a statement in the transaction failed, so it was rolled back");
        failure.code = .IN_FAILED_SQL_TRANSACTION;
    }

    if can_retry && !c.broken && (failure.code == .SERIALIZATION_FAILURE || failure.code == .DEADLOCK_DETECTED) {
        free_error(failure);
        failure.* = .{};
        return .{}, true;
    }

    result := failure.*;

    failure.* = .{};

    return result, false;
}

now_milliseconds :: () -> s64 {
    return to_milliseconds(current_time_monotonic());
}
