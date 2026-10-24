// LISTEN and NOTIFY: other sessions' notifications reach the connection that LISTENs.
//
//     Pg.listen(db, "jobs");
//     while true {
//         n, error := Pg.wait_for_notification(db);
//         if error.kind != .NONE break;
//         defer Pg.free_notification(*n);
//         print("% sent % on %\n", n.backend_pid, n.payload, n.channel);
//     }
//
// The server sends a notification at any time the session is not in a transaction, and this
// client reads it whenever it reads the connection: in the middle of a query, a pipeline or a
// wait. They go to `Connection.on_notification` when it is set, otherwise they queue on the
// connection until taken. A connection that LISTENs is best kept for it alone, off a pool.

Notification :: struct {
    channel: string;
    payload: string; // empty when NOTIFY gave none
    backend_pid: s32; // the session that sent it
}

free_notification :: (n: *Notification) {
    free(n.channel);
    free(n.payload);
    n.* = .{};
}

// The oldest queued notification, without waiting. `found` is false when none is queued. Free it
// with `free_notification`.
next_notification :: (c: *Connection) -> n: Notification, found: bool {
    if !c || !c.notifications.count return .{}, false;

    n := c.notifications[0];
    array_ordered_remove_by_index(*c.notifications, 0);

    return n, true;
}

// Frees every queued notification.
drop_notifications :: (c: *Connection) {
    for *c.notifications free_notification(it);

    c.notifications.count = 0;
}

// The oldest queued notification, or the next one to arrive: blocks the fiber until then, within
// any `Io.with_deadline` or `Io.cancel` (the error is `.TIMED_OUT` or `.CANCELED`, and the
// connection stays usable). Free the notification with `free_notification`. Nothing else may use
// the connection while a fiber waits on it. With `on_notification` set, notifications never queue,
// so there is nothing to wait for.
wait_for_notification :: (c: *Connection) -> Notification, Pg_Error {
    if !c return .{}, make_error(.USAGE, "no connection");

    n, found := next_notification(c);
    if found return n, .{};

    if c.pipeline.active return .{}, make_error(.USAGE, "a pipeline is active on the connection");
    if c.busy return .{}, make_error(.USAGE, "the connection is in use by another fiber");
    if c.broken return .{}, make_error(.CLOSED, "the connection was broken by an earlier failure");
    if !Io.in_scheduler() return .{}, make_error(.USAGE, "Postgres calls must run on a fiber: call them inside Io.io_run");

    c.busy = true;
    defer c.busy = false;

    while true {
        type, body, error := read_message(c);
        if error.kind != .NONE return .{}, error;

        if handle_async(c, type, body) {
            n, found = next_notification(c);
            if found return n, .{};

            continue;
        }

        if type == #char "E" {
            // The server ends an idle session with a FATAL (shutdown, idle timeout).
            c.broken = true;
            return .{}, parse_server_error(body);
        }

        return .{}, protocol_error(c, tprint("unexpected message % while waiting for a notification", cast(string) u8.[type]));
    }

    return .{}, .{};
}

// LISTEN on a channel. The name is quoted, so any text is a channel.
listen :: (c: *Connection, channel: string) -> Pg_Error {
    return execute_script(c, tprint("LISTEN %", quote_identifier(channel)));
}

unlisten :: (c: *Connection, channel: string) -> Pg_Error {
    return execute_script(c, tprint("UNLISTEN %", quote_identifier(channel)));
}

// Stops listening to every channel.
unlisten_all :: (c: *Connection) -> Pg_Error {
    return execute_script(c, "UNLISTEN *");
}

// NOTIFY a channel, with a payload of up to 7999 bytes. Delivered to the listeners when the
// transaction commits; inside none, at once. A connection also hears its own notifications.
notify :: (c: *Connection, channel: string, payload := "") -> Pg_Error {
    result, error := query_dynamic(c, "SELECT pg_notify($1, $2)", channel, payload);
    free_result(*result);

    return error;
}

// The text as an SQL identifier: in double quotes, with embedded quotes doubled.
quote_identifier :: (name: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "\"");

    for name {
        if it == #char "\"" append(*builder, "\"");
        append(*builder, string.{ 1, *it });
    }

    append(*builder, "\"");

    return builder_to_string(*builder,, temp);
}
