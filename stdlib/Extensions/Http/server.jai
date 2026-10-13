// The server: one listener per scheduler, one fiber per connection, requests read through the
// codec and answered one after another on the connection (pipelined requests are answered in
// order and their responses written together).
//
//     serve(*app);                       // production: every core, Ctrl-C drains connections
//
//     server, error := start(*app);      // inside Io.io_run: tests and programs that do more
//     ...
//     stop(server);
//     wait(server);

// One scheduler's listener and its connections.
Server :: struct {
    app: *App;
    listener: Io.Tcp_Listener;
    address: Io.Address; // where it listens (the real port when the config asked for 0)
    stopping: bool;

    // Connections that are open, for graceful shutdown.
    connections: *Connection;
    count: int;

    dates: Date_Cache;
    finished: Io.Wait_Group;
    served: s64; // requests answered, for tests and logs
}

#scope_module

Connection :: struct {
    next: *Connection;
    previous: *Connection;
    server: *Server;
    stream: Link;
    fiber: Io.Fiber_Handle;
    secure: bool; // the TLS handshake is still to be done (`config.tls`)
    idle: bool; // waiting for the first byte of a request: safe to cut off
}

#scope_export

// Starts listening on the running scheduler (call it inside `Io.io_run`, from a fiber) and
// returns at once; connections are served by fibers of this scheduler. Fails if the address
// cannot be bound or the routes conflict. `stop` and `wait` end it.
start :: (app: *App, reuse_port := false) -> *Server, Io.Io_Error {
    if app.setup_errors.count {
        for app.setup_errors print("Http: %\n", it, to_standard_error = true);

        return null, .INVALID;
    }

    if app.config.tls && !Tls.can_accept(app.config.tls) {
        print("Http: config.tls has no certificate and key, or Tls.tls_config_init was not called on it\n", to_standard_error = true);

        return null, .INVALID;
    }

    listener, error := Io.tcp_listen(app.config.address, reuse_port = reuse_port);

    if error != .NONE return null, error;

    server := New(Server);

    server.app = app;
    server.listener = listener;

    address, _ := Io.local_address(listener);

    server.address = address;

    Io.add(*server.finished, 1);

    Io.spawn(accept_loop, server);

    return server, .NONE;
}

// The `http://host:port` (`https://` with `config.tls`) the server is reachable at (a wildcard
// address shows as 127.0.0.1).
server_url :: (server: *Server) -> string {
    address := server.address;

    if address.family == .IPV4 && address.bytes[0] == 0 && address.bytes[1] == 0 && address.bytes[2] == 0 && address.bytes[3] == 0 {
        address.bytes[0] = 127;
        address.bytes[3] = 1;
    }

    return tprint("%://%", ifx server.app.config.tls then "https" else "http", Io.address_to_string(address));
}

// Starts the graceful shutdown: no new connections, idle ones close, requests in progress finish
// (up to `config.shutdown_timeout`, then the connections are cut). Returns at once; `wait`
// returns when it is done.
stop :: (server: *Server) {
    if server.stopping return;

    server.stopping = true;

    Io.close(server.listener);

    Io.spawn(drain, server);
}

// Blocks the calling fiber until the server has stopped and its connections are gone. The
// `Server` itself stays valid (its fibers may still be winding down) and costs a few hundred
// bytes; nothing needs it afterwards.
wait :: (server: *Server) {
    Io.wait(*server.finished);
}

#scope_module

accept_loop :: (server: *Server) {
    while !server.stopping {
        stream, error := Io.accept(server.listener);

        if error == .CLOSED || error == .CANCELED break;

        if error != .NONE {
            // Out of descriptors or memory: wait for connections to end.
            Io.sleep(20);

            continue;
        }

        adopt(server, tcp_link(stream), server.app.config.tls != null);
    }
}

// Serves `stream` on a new fiber of the running scheduler. With `secure` the fiber first does the
// TLS handshake (`config.tls`) on it.
adopt :: (server: *Server, stream: Link, secure := false) {
    connection := New(Connection);

    connection.server = server;
    connection.stream = stream;
    connection.secure = secure;
    connection.next = server.connections;

    if server.connections server.connections.previous = connection;

    server.connections = connection;
    server.count += 1;

    connection.fiber = Io.spawn(serve_connection, connection);
}

drain :: (server: *Server) {
    limit := now_ms() + server.app.config.shutdown_timeout;
    forced := false;

    while server.count > 0 {
        connection := server.connections;

        while connection {
            if connection.idle || forced Io.cancel(connection.fiber);

            connection = connection.next;
        }

        if !forced && now_ms() >= limit forced = true;

        Io.sleep(5);
    }

    Io.done(*server.finished);
}

now_ms :: () -> s64 {
    return to_milliseconds(current_time_monotonic());
}

// Where an HTTP/2 upgrade (`Upgrade: h2c`) or prior-knowledge connection would take over. HTTP/2
// is a later milestone, so nothing switches yet; the connection keeps speaking HTTP/1.
upgrade_connection :: (connection: *Connection, head: *Head) -> bool {
    return false;
}

// Runs the TLS handshake of a new connection, which has `header_timeout` to finish it, and returns
// the link that carries HTTP from here on (or, when the connection must be dropped, the link to
// close). The protocol is chosen here too: ALPN `http/1.1` (or none)
// speaks HTTP/1.1. `choose_protocol` is where `h2` would branch off to an HTTP/2 connection.
accept_tls :: (connection: *Connection) -> Link, bool {
    config := *connection.server.app.config;
    link := connection.stream;

    stream: *Tls.Tls_Stream;
    error: Tls.Tls_Error;

    Io.with_deadline(within(config.header_timeout), #code {
        stream, error = Tls.tls_accept(config.tls, link.reader, link.writer);
    });

    if error.kind != .NONE {
        if config.log_requests print("TLS handshake from % failed: %\n", Io.address_to_string(link.peer), Tls.error_text(error));

        return link, false;
    }

    secured: Link;

    secured.reader = Tls.reader_of(stream);
    secured.writer = Tls.writer_of(stream);
    secured.peer = link.peer;

    if !choose_protocol(Tls.alpn_protocol(stream)) {
        if config.log_requests print("TLS connection from % negotiated ALPN %, which this server does not speak\n", Io.address_to_string(link.peer), Tls.alpn_protocol(stream));

        return secured, false;
    }

    return secured, true;
}

// Whether the connection can speak the protocol ALPN chose. HTTP/1.1 is all there is; HTTP/2
// (milestone 11) turns `h2` into a different connection loop here.
choose_protocol :: (alpn: string) -> bool {
    return alpn.count == 0 || alpn == "http/1.1";
}

serve_connection :: (connection: *Connection) {
    server := connection.server;
    app := server.app;
    config := *app.config;
    stream := connection.stream;

    defer {
        // Leave the list, then close: a fiber cut off by shutdown can still close.
        if connection.previous connection.previous.next = connection.next;
        else server.connections = connection.next;

        if connection.next connection.next.previous = connection.previous;

        server.count -= 1;

        free(connection);
    }

    if connection.secure {
        secured, ok := accept_tls(connection);

        // A client that fails the handshake is dropped without an answer: there is no HTTP yet.
        // `secured` is then the link to close: the TCP one, or the TLS one when it is up.
        if !ok {
            close(secured);

            return;
        }

        stream = secured;
    }

    defer close(stream);

    peer := stream.peer;
    write_error: Io.Io_Error;

    limits: Limits;

    limits.max_line_size = config.max_line_size;
    limits.max_head_size = config.max_head_size;
    limits.max_headers = config.max_headers;
    limits.max_body_size = config.max_body_size;

    parser: Parser;

    parser_init(*parser, .REQUEST, limits);

    defer parser_free(*parser);

    // The read buffer comes from the scheduler's pool. A connection whose reader can wait for
    // readability (a TCP stream) holds none while it is idle or waiting for more of a head, and
    // takes one only when bytes are there; any other connection keeps one for its whole life.
    buffers := read_buffers();
    buffer_size := max(config.read_buffer_size, 512);
    can_wait := Io.can_wait_readable(stream.reader);
    buffer: [] u8;

    defer if buffer.data give_buffer(buffers, buffer);

    if !can_wait buffer = take_buffer(buffers, buffer_size);

    out: [..] u8;
    body_buffer: [..] u8;

    defer array_reset(*out);
    defer array_reset(*body_buffer);

    pending: [] u8;
    served_here := 0;
    head_started: s64 = 0;

    // When the previous request began, and whether the client is sending them back to back.
    last_started: s64 = 0;
    hot := false;

    while true {
        // ---- The head of the next request.
        event: Event;

        while true {
            if pending.count == 0 {
                // About to wait for the client: it gets what it is owed first.
                if out.count > 0 {
                    Io.with_deadline(within(config.write_timeout), #code {
                        _, write_error = Io.write(stream.writer, out);
                    });

                    if write_error != .NONE return;

                    out.count = 0;

                    trim(*out, RETAIN_OUTPUT);
                }

                idle := parser_idle(*parser);

                connection.idle = idle;

                timeout := config.idle_timeout;

                if !idle {
                    // The first bytes of this request came with the end of the previous one.
                    if head_started == 0 head_started = now_ms();

                    timeout = config.header_timeout;

                    if config.header_timeout > 0 {
                        timeout = head_started + config.header_timeout - now_ms();

                        if timeout <= 0 {
                            send_failure(server, stream, *out, http_error(408, "the request head was not received in time"));

                            return;
                        }
                    }
                }

                count: s64;
                error: Io.Io_Error;

                // Nothing is pending, so nothing points into the buffer.
                if can_wait && !hot && buffer.data {
                    give_buffer(buffers, buffer);
                    buffer = .[];
                }

                // A connection that is not busy also keeps none of its scratch memory (the
                // response buffer and the parser's storage grow again when it needs them).
                if can_wait && !hot {
                    trim(*out, 0);
                    trim(*body_buffer, 0);
                    parser_trim(*parser, 0);
                }

                if hot {
                    // A client that sent its last request a moment ago most likely sends the next
                    // one at once, and the read below completes without a second operation. It
                    // holds a buffer for that; if the client stays quiet for HOT_MS the buffer
                    // goes back and the wait continues without one.
                    if !buffer.data buffer = take_buffer(buffers, buffer_size);

                    limit := ifx timeout > 0 then min(timeout, HOT_MS) else HOT_MS;

                    Io.with_deadline(limit, #code {
                        count, error = Io.read(stream.reader, buffer);
                    });

                    if error == .TIMED_OUT && (timeout <= 0 || timeout > HOT_MS) {
                        give_buffer(buffers, buffer);
                        buffer = .[];

                        hot = false;
                        error = .NONE;

                        if timeout > 0 timeout -= HOT_MS;
                    }
                }

                if !hot && error == .NONE && count == 0 {
                    if can_wait {
                        Io.with_deadline(within(timeout), #code {
                            error = Io.wait_readable(stream.reader);
                        });

                        if error == .NONE {
                            buffer = take_buffer(buffers, buffer_size);

                            Io.with_deadline(within(timeout), #code {
                                count, error = Io.read(stream.reader, buffer);
                            });
                        }
                    } else {
                        Io.with_deadline(within(timeout), #code {
                            count, error = Io.read(stream.reader, buffer);
                        });
                    }
                }

                connection.idle = false;

                if error == .TIMED_OUT && !idle {
                    send_failure(server, stream, *out, http_error(408, "the request head was not received in time"));

                    return;
                }

                if error != .NONE || count == 0 return;

                if idle {
                    head_started = now_ms();

                    hot = can_wait && last_started != 0 && head_started - last_started < HOT_MS;
                }

                pending = .{ count, buffer.data };
            }

            used, next := parse(*parser, pending);

            pending = advance(pending, used);

            if next.kind == .NEED_MORE continue;

            event = next;

            break;
        }

        if event.kind == .ERROR {
            send_failure(server, stream, *out, parse_failure(*parser));

            return;
        }

        if event.kind != .HEAD return;

        head := *parser.head;

        if head.upgrade && upgrade_connection(connection, head) return;

        // ---- The body.
        if head.expect_continue {
            if out.count > 0 {
                Io.with_deadline(within(config.write_timeout), #code {
                    _, write_error = Io.write(stream.writer, out);
                });

                if write_error != .NONE return;

                out.count = 0;
            }

            write_continue(*out);

            Io.with_deadline(within(config.write_timeout), #code {
                _, write_error = Io.write(stream.writer, out);
            });

            if write_error != .NONE return;

            out.count = 0;
        }

        body_buffer.count = 0;

        body: [] u8;
        in_buffer := false;

        while true {
            used, next := parse(*parser, pending);

            pending = advance(pending, used);

            if next.kind == .BODY {
                if body_buffer.count == 0 && !in_buffer {
                    body = next.data;
                    in_buffer = true;
                } else {
                    if body_buffer.count == 0 append_bytes(*body_buffer, body.data, body.count);

                    append_bytes(*body_buffer, next.data.data, next.data.count);
                }

                continue;
            }

            if next.kind == .END break;

            if next.kind == .NEED_MORE {
                // The next read overwrites the buffer the body so far points into.
                if in_buffer && body_buffer.count == 0 && body.count > 0 append_bytes(*body_buffer, body.data, body.count);

                count: s64;
                error: Io.Io_Error;

                Io.with_deadline(within(config.body_timeout), #code {
                    count, error = Io.read(stream.reader, buffer);
                });

                if error == .TIMED_OUT {
                    send_failure(server, stream, *out, http_error(408, "the request body was not received in time"));

                    return;
                }

                if error != .NONE || count == 0 return;

                pending = .{ count, buffer.data };

                continue;
            }

            if next.kind == .ERROR {
                send_failure(server, stream, *out, parse_failure(*parser));

                return;
            }

            return;
        }

        if body_buffer.count > 0 body = body_buffer;

        // ---- The request and its response.
        started := now_ms();

        last_started = started;

        request: Request;

        request.method = head.method;
        request.method_text = head.method_text;
        request.target = head.target;
        request.version = head.version;
        request.headers = head.headers;
        request.body = body;
        request.peer = peer;

        split_target(*request);

        response := handle_request(app, *request);

        served_here += 1;
        server.served += 1;

        keep_alive := head.keep_alive && !server.stopping;

        if config.max_requests_per_connection > 0 && served_here >= config.max_requests_per_connection keep_alive = false;

        if !append_response(server, *out, *response, *request, keep_alive) return;

        if config.log_requests {
            print("% % % (% ms)\n", request.method_text, request.target, response.status, now_ms() - started);
        }

        // A big body goes out from where it is instead of being copied after the head.
        if response.body.count >= LARGE_BODY && request.method != .HEAD && body_allowed(response.status) {
            Io.with_deadline(within(config.write_timeout), #code {
                _, write_error = Io.write(stream.writer, out);
            });

            if write_error != .NONE return;

            out.count = 0;

            trim(*out, RETAIN_OUTPUT);

            Io.with_deadline(within(config.write_timeout), #code {
                _, write_error = Io.write(stream.writer, response.body);
            });

            if write_error != .NONE return;
        }

        reset_temporary_storage();

        // One big request must not make the connection heavy for the rest of its life.
        trim(*body_buffer, RETAIN_OUTPUT);
        parser_trim(*parser, RETAIN_HEAD);

        if !keep_alive {
            Io.with_deadline(within(config.write_timeout), #code {
                _, write_error = Io.write(stream.writer, out);
            });

            return;
        }

        head_started = 0;
    }
}

LARGE_BODY :: 64 * 1024;

// A connection whose last two requests began less than this many milliseconds apart keeps its read
// buffer while it waits for the next one, for at most this long. Waiting without a buffer costs a
// second operation per request (the readiness wait, then the read), which a client that is busy
// does not need.
HOT_MS :: 10;

// A connection keeps what it grew to between requests up to this much per array (a response
// buffer, the collected body, the parser's head storage); past it the memory is returned once the
// request is done, so one large request does not weigh on an idle connection.
RETAIN_OUTPUT :: 64 * 1024;
RETAIN_HEAD :: 16 * 1024;

// Frees `array` when it is empty and has grown past `limit` bytes.
trim :: (array: *[..] u8, limit: int) {
    if array.count == 0 && array.allocated > limit array_reset(array);
}

// ---------------------------------------------------------------------------------------------
// Read buffers

// The free read buffers of one scheduler. A connection takes one when bytes are waiting for it and
// gives it back when it has nothing pending, so a server with many quiet connections needs about as
// many buffers as it has requests in flight, not connections.
Read_Buffers :: struct {
    size: int; // of the buffers in `free`
    free: [..] *u8;
    held: int; // taken and not yet given back
}

// Buffers kept for reuse per scheduler; more are freed as they come back.
READ_BUFFERS_KEPT :: 64;

read_buffers_key: s32;

read_buffers :: () -> *Read_Buffers {
    pool := cast(*Read_Buffers) Io.scheduler_local(*read_buffers_key);

    if pool return pool;

    pool = New(Read_Buffers);

    Io.set_scheduler_local(*read_buffers_key, pool, drop_read_buffers);

    return pool;
}

drop_read_buffers :: (value: *void) {
    pool := cast(*Read_Buffers) value;

    for pool.free free(it);

    array_reset(*pool.free);
    free(pool);
}

// A buffer of `size` bytes. Its contents are whatever the last user left.
take_buffer :: (pool: *Read_Buffers, size: int) -> [] u8 {
    pool.held += 1;

    // Servers with different `read_buffer_size`s on one scheduler: the pool follows the one that
    // finds it empty.
    if pool.free.count == 0 pool.size = size;

    if pool.size == size && pool.free.count > 0 {
        data := pool.free[pool.free.count - 1];
        pool.free.count -= 1;

        return .{ size, data };
    }

    return .{ size, alloc(size) };
}

give_buffer :: (pool: *Read_Buffers, buffer: [] u8) {
    pool.held -= 1;

    if buffer.count == pool.size && pool.free.count < READ_BUFFERS_KEPT {
        array_add(*pool.free, buffer.data);

        return;
    }

    free(buffer.data);
}

body_allowed :: (status: int) -> bool {
    return status >= 200 && status != 204 && status != 304;
}

// Puts the head and body of `response` into `out`. A large body is left for the caller to send
// (the head only goes in); `append_response` then reports the body as pending through `out`'s
// framing: the caller checks `response.body.count >= LARGE_BODY`. False when nothing can be sent.
append_response :: (server: *Server, out: *[..] u8, response: *Response, request: *Request, keep_alive: bool) -> bool {
    options: Response_Options;

    options.version = request.version;
    options.request_method = request.method;
    options.keep_alive = keep_alive;
    options.date = http_date_now(*server.dates);
    options.server = server.app.config.server_name;

    mark := out.count;

    kind, ok := write_response_head(out, response.status, response.headers, response.body.count, options);

    if !ok {
        // A handler built a response the writer refuses (a bad header, a status outside 100-599).
        out.count = mark;

        failure := internal_error("the handler produced an invalid response");
        replacement := problem_response(request, failure);

        kind, ok = write_response_head(out, 500, replacement.headers, replacement.body.count, options);

        if !ok return false;

        response.* = replacement;
    }

    if kind == .LENGTH && response.body.count < LARGE_BODY {
        append_bytes(out, response.body.data, response.body.count);
    }

    return true;
}

// Fills in `path` and `query` from the target (an absolute-form target keeps only its path).
split_target :: (request: *Request) {
    target := request.target;

    if begins_with(target, "http://") || begins_with(target, "https://") {
        rest := slice(target, ifx begins_with(target, "http://") then 7 else 8, target.count - ifx begins_with(target, "http://") then 7 else 8);
        slash := find_index_from_left(rest, #char "/");

        target = ifx slash >= 0 then slice(rest, slash, rest.count - slash) else "/";
    }

    question := find_index_from_left(target, #char "?");

    if question >= 0 {
        request.path = slice(target, 0, question);
        request.query = slice(target, question + 1, target.count - question - 1);
    } else {
        request.path = target;
    }
}

// The error response for a message the codec refused.
parse_failure :: (parser: *Parser) -> Http_Error {
    status := error_status(parser.error.kind);

    return http_error(status, parser.error.message);
}

// Sends an error and closes (the connection cannot be trusted to carry another request).
send_failure :: (server: *Server, stream: Link, out: *[..] u8, error: Http_Error) {
    request: Request;

    request.app = server.app;
    request.version = .HTTP_1_1;

    response := error_response(*request, error);

    append_response(server, out, *response, *request, false);

    if response.body.count >= LARGE_BODY && body_allowed(response.status) append_bytes(out, response.body.data, response.body.count);

    Io.with_deadline(within(server.app.config.write_timeout), #code {
        Io.write(stream.writer, out.*);
    });

    out.count = 0;

    reset_temporary_storage();
}

// ---------------------------------------------------------------------------------------------
// serve

Serve_State :: struct {
    app: *App;
    ok: s32 = 1;
    announced: s32;
    threads: int;
}

#scope_export

// Runs the app until it is stopped: SIGINT or SIGTERM (when `config.handle_signals`), or
// `shutdown(*app)` from another thread. One scheduler per `config.threads` (default: the
// processor count), each with its own listener on the same port. Returns false if it could not
// start (the port is taken, two routes conflict), after saying why on standard error.
serve :: (app: *App) -> bool {
    if app.setup_errors.count {
        for app.setup_errors print("Http: %\n", it, to_standard_error = true);

        return false;
    }

    if app.config.handle_signals Io.catch_interrupts();

    state: Serve_State;

    state.app = app;
    state.threads = ifx app.config.threads > 0 then app.config.threads else Io.processor_count();

    Io.io_run_threads(state.threads, serve_thread, *state);

    return state.ok != 0;
}

serve :: (app: *App, address: string) -> bool {
    app.config.address = address;

    return serve(app);
}

#scope_module

serve_thread :: (state: *Serve_State) {
    server, error := start(state.app, reuse_port = state.threads > 1);

    if error != .NONE {
        if compare_and_swap(*state.ok, 1, 0) {
            print("Http: cannot listen on %: %\n", state.app.config.address, error, to_standard_error = true);
        }

        return;
    }

    if compare_and_swap(*state.announced, 0, 1) {
        print("Http: listening on % (% thread%)\n", server_url(server), state.threads, ifx state.threads == 1 then "" else "s");
    }

    Io.spawn(watch, server);

    wait(server);
}

// Polls for a reason to stop.
watch :: (server: *Server) {
    app := server.app;

    while !server.stopping {
        if atomic_load(*app.shutdown_flag) != 0 || (app.config.handle_signals && Io.interrupted()) {
            stop(server);

            break;
        }

        Io.sleep(50);
    }
}
