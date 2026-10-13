// A client wired to an `App` instead of the network, for end-to-end tests. It is the same
// `Client` as the real one: every call (`get`, `post`, `send`, `read_json`...) works on it and returns
// the same types, and relative URLs (`"/notes/1"`) are what you write.
//
//     app: App;
//     get(*app, "/notes/{id}", get_note);
//
//     api := test_client(*app);
//
//     response := get(api, "/notes/1");
//     expect_status(response, 200);
//
// Each request goes through a connection made of two ends of an in-memory pipe (`Io.make_pipe`):
// the app's real connection loop reads it, so the codec, framing, keep-alive, middleware,
// routing, extractors, error mapping and the response writer all run, only the socket is
// missing. Everything happens on one scheduler thread in an order that depends on nothing but
// the code, so a test gives the same result every time.
//
// Outside `Io.io_run` every call runs a short scheduler of its own (see `send`), so a test needs
// no setup; inside one (a test that spawns fibers, or uses the real client against another
// server) the same calls run on the calling fiber and keep their connections between calls.
// `over_tcp` serves the app on a free loopback port instead, for tests where the network path
// matters.

Test_Options :: struct {
    // Serve the app on a loopback socket (a free port, started on first use) instead of in
    // memory. This sets `app.config.address` and turns `handle_signals` off.
    over_tcp := false;

    // Follow redirects like the real client. Off, a 3xx is the response, so it can be checked
    // (`expect_redirect`); `send(api, .{ url = ..., max_redirects = 5 })` follows for one request.
    follow_redirects := false;

    // Milliseconds a request may take before it fails with `.TIMED_OUT` (the client's `timeout`).
    timeout := 10_000;
}

// A client for `app`. The app must stay alive and unchanged in its routes while the client is
// used; its `state` pointer is yours to inspect between requests. The client belongs to the test
// run (`run_tests` frees it afterwards); `free_test_client` releases one sooner.
test_client :: (app: *App, options := Test_Options.{}) -> *Client {
    target := New(Test_Target);

    target.app = app;
    target.over_tcp = options.over_tcp;

    if options.over_tcp {
        app.config.address = "127.0.0.1:0";
        app.config.handle_signals = false;
    }

    client := New(Client);

    client.timeout = options.timeout;
    client.max_redirects = ifx options.follow_redirects then 10 else 0;
    client.transport.data = target;
    client.transport.base = test_base;
    client.transport.open = test_open;

    array_add(*test_clients, client);

    return client;
}

free_test_client :: (client: *Client) {
    for test_clients {
        if it == client {
            remove it;

            break;
        }
    }

    free(client.transport.data);

    for client.headers {
        free(it.name.data);
        free(it.value.data);
    }

    array_reset(*client.headers);
    free(client);
}

#scope_module

Test_Target :: struct {
    app: *App;
    over_tcp: bool;
}

// The server side of one test client on one scheduler: the connections of a memory client, or the
// listener of a TCP one.
Test_Host :: struct {
    target: *Test_Target;
    server: *Server;
}

Test_Hosts :: struct {
    list: [..] *Test_Host;
}

test_hosts_key: s32;

// Every client made by `test_client` and not yet freed.
test_clients: [..] *Client;

free_test_clients :: () {
    while test_clients.count > 0 free_test_client(test_clients[test_clients.count - 1]);
}

test_host :: (target: *Test_Target) -> *Test_Host, Io.Io_Error {
    hosts := cast(*Test_Hosts) Io.scheduler_local(*test_hosts_key);

    if !hosts {
        hosts = New(Test_Hosts);

        Io.set_scheduler_local(*test_hosts_key, hosts, drop_test_hosts);
    }

    for hosts.list if it.target == target return it, .NONE;

    host := New(Test_Host);

    host.target = target;

    if target.over_tcp {
        server, error := start(target.app);

        if error != .NONE {
            free(host);

            return null, error;
        }

        host.server = server;
    } else {
        if target.app.setup_errors.count {
            for target.app.setup_errors print("Http: %\n", it, to_standard_error = true);

            free(host);

            return null, .INVALID;
        }

        host.server = New(Server);
        host.server.app = target.app;
    }

    array_add(*hosts.list, host);

    return host, .NONE;
}

// The scheduler is over: its connection fibers have ended, so only the listener is left.
drop_test_hosts :: (value: *void) {
    hosts := cast(*Test_Hosts) value;

    for hosts.list {
        if it.target.over_tcp {
            it.server.stopping = true;

            Io.close(it.server.listener, now = true);
        }

        free(it.server);
        free(it);
    }

    array_reset(*hosts.list);
    free(hosts);
}

test_base :: (data: *void) -> string {
    target := cast(*Test_Target) data;

    if !target.over_tcp return "http://localhost";

    host, error := test_host(target);

    if error != .NONE return "";

    return server_url(host.server);
}

test_open :: (data: *void, url: Url) -> Link, Io.Io_Error {
    target := cast(*Test_Target) data;

    host, error := test_host(target);

    if error != .NONE return .{}, error;

    if target.over_tcp {
        stream, connect_error := Io.connect(host.server.address);

        if connect_error != .NONE return .{}, connect_error;

        return tcp_link(stream), .NONE;
    }

    near, far := Io.make_pipe();

    adopt(host.server, pipe_link(far));

    return pipe_link(near), .NONE;
}
