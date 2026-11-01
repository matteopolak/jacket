// The TCP server: one fiber accepts connections and starts a fiber for each of them.
//
// All fibers of a scheduler share its one OS thread, and a fiber only gives way when it waits for
// the network. A request is executed from its first byte to its reply without waiting, so every
// request runs alone: many connections can read the same store, and the only writer is whichever
// fiber is running, with no lock anywhere. The price is that the disk is not waited for
// asynchronously, so a COMPACT (which rewrites the whole log) holds up every other client.

Server :: struct {
    store: *Store;
    listener: Io.Tcp_Listener;
    handlers: [..] Io.Fiber_Handle;
    stopping: bool;
    // Requests answered, for the tests and the shutdown message.
    served: s64;
}

Connection :: struct {
    server: *Server;
    stream: Io.Tcp_Stream;
}

// Accepts connections until `server_stop` (or a client's SHUTDOWN) closes the listener, then
// cancels the connections that are still open. Call it from a fiber, with `server.listener` open.
serve :: (server: *Server) {
    while !server.stopping {
        stream, error := Io.accept(server.listener);

        if error != .NONE break;

        array_add(*server.handlers, Io.spawn(connection_loop, Connection.{ server, stream }));
    }

    for server.handlers Io.cancel(it);

    array_reset(*server.handlers);
}

// Makes `serve` return. Safe to call from any fiber of the server's scheduler.
server_stop :: (server: *Server) {
    if server.stopping return;

    server.stopping = true;

    Io.close(server.listener);
}

#scope_file

// Reads requests from one client and answers each in turn until the client leaves, asks to be
// disconnected, or the server stops.
connection_loop :: (connection: Connection) {
    server := connection.server;
    reader := Line_Reader.{ stream = connection.stream };
    reply: String_Builder;

    defer free_line_reader(*reader);
    defer free_buffers(*reply);
    defer Io.close(connection.stream);

    while true {
        line, error := read_line(*reader);

        if error == .INVALID {
            write_line(connection.stream, "ERR the request line is too long");

            break;
        }

        if error != .NONE break;
        if line.count == 0 continue;

        outcome := Outcome.CONTINUE;
        request, problem := parse_request(line);

        if problem.count > 0 {
            print_to_builder(*reply, "ERR %\n", problem);
        } else {
            outcome = execute(server.store, request, *reply);
        }

        text := builder_to_string(*reply);
        _, written := Io.write(connection.stream, text);

        free(text);
        server.served += 1;

        if written != .NONE || outcome == .CLOSE break;

        if outcome == .SHUTDOWN {
            server_stop(server);

            break;
        }
    }
}
