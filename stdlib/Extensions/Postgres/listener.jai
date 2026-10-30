// A dedicated, self-healing LISTEN on a pool's database. `listen(pool, "jobs")` takes a connection
// out of the pool for as long as the listener lives, LISTENs on it, and reads notifications on a
// fiber of its own into a queue. If the connection fails (the server restarts, the backend is
// terminated, the network drops) the listener returns it to the pool, takes another, LISTENs
// again and carries on. Postgres keeps no notifications for a session that is not connected, so
// those sent while the listener is reconnecting are lost; code that must not miss any should read
// its table again after `reconnects` changes.
//
// ```jai,ignore
// listener, error := Pg.listen(pool, "jobs", "mail");
// while true {
//     n, error := Pg.receive(listener);
//     if error.kind != .NONE break;
//     defer Pg.free_notification(*n);
//     print("% on %\n", n.payload, n.channel);
// }
// Pg.close_listener(listener);
// ```
Listener :: struct {
    // Notifications received and not yet taken, oldest first (`receive`). The reader waits when
    // `LISTENER_QUEUE` of them are queued.
    queue: [..] Notification;
    arrived: Io.Condition;
    taken: Io.Condition;
    closed: bool; // no more will arrive
    // How many times the connection was replaced after a failure.
    reconnects: s64;

    pool: *Pool;
    channels: [..] string;
    connection: *Connection;
    fiber: Io.Fiber_Handle;
    closing: bool;
    finished: bool;
    ended: Io.Condition;
}

LISTENER_QUEUE :: 256; // notifications held before the reader waits for the program
LISTENER_RETRY_FIRST :: 100; // ms before the first reconnect attempt, doubling
LISTENER_RETRY_LAST :: 5_000;

listen :: (pool: *Pool, channels: ..string) -> *Listener, Pg_Error {
    if !pool return null, make_error(.USAGE, "no pool");
    if !channels.count return null, make_error(.USAGE, "listen needs at least one channel");

    c, error := acquire(pool);
    if error.kind != .NONE return null, error;

    error = listen_all(c, channels);

    if error.kind != .NONE {
        // It may be listening to some of them: do not hand it back to the pool.
        c.broken = true;
        release(pool, c);

        return null, error;
    }

    listener := New(Listener);
    listener.pool = pool;
    listener.connection = c;

    for channels array_add(*listener.channels, copy_string(it));

    listener.fiber = Io.spawn(read_notifications, listener);

    return listener, .{};
}

// The next notification, in order. Blocks the fiber until one arrives, within any `Io.with_deadline`
// or `Io.cancel` (`.TIMED_OUT` or `.CANCELED`); `.CLOSED` once the listener is closed and what it
// held is taken. Free the notification with `free_notification`.
receive :: (listener: *Listener) -> Notification, Pg_Error {
    while !listener.queue.count {
        if listener.closed return .{}, make_error(.CLOSED, "the listener is closed");

        io := Io.wait(*listener.arrived);
        if io != .NONE return .{}, io_failure(io, "the wait for a notification ended");
    }

    n := listener.queue[0];

    array_ordered_remove_by_index(*listener.queue, 0);
    Io.notify_one(*listener.taken);

    return n, .{};
}

// Stops listening: the connection goes back to the pool without its subscriptions, and the
// listener is freed. Notifications not taken yet are dropped.
close_listener :: (listener: *Listener) {
    if !listener return;

    listener.closing = true;
    Io.cancel(listener.fiber);

    // The reader fiber ends by itself once canceled, after handing the connection back.
    while !listener.finished {
        if Io.wait(*listener.ended) != .NONE break;
    }

    for *listener.queue free_notification(it);

    array_free(listener.queue);
    for listener.channels free(it);

    array_free(listener.channels);
    free(listener);
}

#scope_module

listen_all :: (c: *Connection, channels: [] string) -> Pg_Error {
    for channels {
        error := listen(c, it);
        if error.kind != .NONE return error;
    }

    return .{};
}

// The listener's fiber: reads notifications into a queue, replaces the connection when it fails.
read_notifications :: (listener: *Listener) {
    defer {
        give_back(listener);
        listener.closed = true;
        listener.finished = true;
        Io.notify_all(*listener.arrived);
        Io.notify_all(*listener.ended);
    }

    while !listener.closing {
        n, error := wait_for_notification(listener.connection);

        if error.kind == .NONE {
            while listener.queue.count >= LISTENER_QUEUE && !listener.closing {
                if Io.wait(*listener.taken) != .NONE break;
            }

            if listener.closing {
                free_notification(*n);
                return;
            }

            array_add(*listener.queue, n);
            Io.notify_one(*listener.arrived);

            continue;
        }

        canceled := error.kind == .CANCELED;

        free_error(*error);

        if canceled || listener.closing || !reconnect(listener) return;
    }
}

// Returns the connection to the pool: unsubscribed when it still works, dropped when it does not.
give_back :: (listener: *Listener) {
    c := listener.connection;
    if !c return;

    listener.connection = null;

    if !c.broken {
        Io.with_grace(c.cancel_timeout, #code {
            error := unlisten_all(c);
            if error.kind != .NONE c.broken = true;
            free_error(*error);
        });
    }

    release(listener.pool, c);
}

// A new connection with the same subscriptions, tried with a growing pause until it works.
// False when the listener was closed first.
reconnect :: (listener: *Listener) -> bool {
    give_back(listener);

    delay := LISTENER_RETRY_FIRST;

    while !listener.closing {
        c, error := acquire(listener.pool);

        if error.kind == .NONE {
            error = listen_all(c, listener.channels);

            if error.kind == .NONE {
                listener.connection = c;
                listener.reconnects += 1;

                return true;
            }

            c.broken = true;
            release(listener.pool, c);
        }

        canceled := error.kind == .CANCELED;

        free_error(*error);

        if canceled return false;
        if Io.sleep(delay) != .NONE return false;

        delay = min(delay * 2, LISTENER_RETRY_LAST);
    }

    return false;
}
