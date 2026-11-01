// The line protocol: parsing, the replies of each command against a store, and a round trip over
// real TCP connections between fibers of one scheduler.

#import "Basic";
#import "String";
#import "Base64";
#import "Extensions/Testing";
Io :: #import "Extensions/Io";
#import,dir "../store";
#import,dir "../protocol";

#import,dir "support";

Parse_Case :: struct {
    line: string;
    verb: Verb;
    key: string;
    value: string;
}

PARSES :: Parse_Case.[
    .{ "GET name", .GET, "name", "" },
    .{ "get name", .GET, "name", "" },
    .{ "SET name Ada Lovelace", .SET, "name", "Ada Lovelace" },
    .{ "SET name", .SET, "name", "" },
    .{ "SET64 name QWRh", .SET64, "name", "QWRh" },
    .{ "DEL name", .DEL, "name", "" },
    .{ "SCAN", .SCAN, "", "" },
    .{ "SCAN user:", .SCAN, "user:", "" },
    .{ "COMPACT", .COMPACT, "", "" },
    .{ "STATS", .STATS, "", "" },
    .{ "QUIT", .QUIT, "", "" },
    .{ "SHUTDOWN", .SHUTDOWN, "", "" },
];

parses_requests :: (line: string, verb: Verb, key: string, value: string) {
    request, problem := parse_request(line);

    expect_equal(problem, "");
    expect_equal(request.verb, verb);
    expect_equal(request.key, key);
    expect_equal(request.value, value);
} @test @cases(PARSES)

Bad_Request :: struct {
    line: string;
    problem: string;
}

BAD_REQUESTS :: Bad_Request.[
    .{ "", "unknown command ''" },
    .{ "FETCH x", "unknown command 'FETCH'" },
    .{ "GET", "GET needs a key" },
    .{ "SET", "SET needs a key" },
    .{ "STATS now", "STATS takes no arguments" },
    .{ "GET a b", "GET takes one key" },
];

rejects_bad_requests :: (line: string, problem: string) {
    _, got := parse_request(line);

    expect_equal(got, problem);
} @test @cases(BAD_REQUESTS)

// Runs one request line against `store` and returns the reply text.
ask :: (store: *Store, line: string) -> string {
    request, problem := parse_request(line);

    if problem.count > 0 return tprint("ERR %\n", problem);

    reply: String_Builder;
    reply.allocator = temp;

    execute(store, request, *reply);

    return builder_to_string(*reply,, temp);
} @TempAllocates

commands_answer_as_documented :: (t: *Test) {
    store: Store;

    store_open(*store, log_path(t));
    defer store_close(*store);

    expect_equal(ask(*store, "SET user:1 Ada"), "OK\n");
    expect_equal(ask(*store, "SET user:2 Grace Hopper"), "OK\n");
    expect_equal(ask(*store, "SET other x"), "OK\n");
    expect_equal(ask(*store, "GET user:2"), "VALUE Grace Hopper\n");
    expect_equal(ask(*store, "GET nobody"), "MISSING\n");
    expect_equal(ask(*store, "SCAN user:"), "ROW user:1 Ada\nROW user:2 Grace Hopper\nEND\n");
    expect_equal(ask(*store, "DEL user:1"), "OK\n");
    expect_equal(ask(*store, "DEL user:1"), "MISSING\n");
    expect_equal(ask(*store, "SCAN user:"), "ROW user:2 Grace Hopper\nEND\n");
    expect_equal(ask(*store, "STATS"), tprint("STATS keys=2 records=4 log_bytes=% live_bytes=%\n", store.log_size, store.live_bytes));
    expect_equal(ask(*store, "QUIT"), "OK\n");
} @test

compact_reports_both_sizes :: (t: *Test) {
    store: Store;

    store_open(*store, log_path(t));
    defer store_close(*store);

    ask(*store, "SET k 1");
    ask(*store, "SET k 2");

    before := store.log_size;
    reply := ask(*store, "COMPACT");

    expect_equal(reply, tprint("OK % %\n", before, store.log_size));
    expect(store.log_size < before);
} @test

// A value with a line break would break the framing, so it travels as Base64.
binary_values_use_base64 :: (t: *Test) {
    store: Store;

    store_open(*store, log_path(t));
    defer store_close(*store);

    encoded := base64_encode("two\nlines",, temp);

    expect_equal(ask(*store, tprint("SET64 note %", encoded)), "OK\n");
    expect_equal(ask(*store, "GET note"), tprint("VALUE64 %\n", encoded));
    expect_equal(ask(*store, "SCAN"), tprint("ROW64 note %\nEND\n", encoded));
    expect_equal(ask(*store, "SET64 bad !!!"), "ERR the value is not Base64\n");

    value, _ := store_get(*store, "note");

    expect_equal(value, "two\nlines");
} @test

Session :: struct {
    store: *Store;
    server: Server;
    failures: s64;
    clients_done: s64;
}

// Sends `line` and checks the reply, as the lines it is made of.
expect_reply :: (client: *Client, line: string, want: ..string) -> bool {
    reply, error := call(client, line);

    defer free_reply(reply);

    ok := error == .NONE && reply.count == want.count;

    if ok {
        for want ok = ok && reply[it_index] == it;
    }

    if !ok {
        log_error("% -> % lines (error %), wanted %", line, reply.count, error, want.count);
    }

    return ok;
}

// The scripted client: every kind of request over one connection.
script_client :: (session: *Session) {
    address, _ := Io.local_address(session.server.listener);
    client, error := connect_client(Io.address_to_string(address));

    if error != .NONE {
        session.failures += 1;

        return;
    }

    defer close_client(*client);

    ok := true;

    ok = expect_reply(*client, "SET colour blue", "OK") && ok;
    ok = expect_reply(*client, "GET colour", "VALUE blue") && ok;
    ok = expect_reply(*client, "SET color gray", "OK") && ok;
    ok = expect_reply(*client, "SCAN col", "ROW color gray", "ROW colour blue", "END") && ok;
    ok = expect_reply(*client, "DEL color", "OK") && ok;
    ok = expect_reply(*client, "GET color", "MISSING") && ok;
    ok = expect_reply(*client, "NOPE", "ERR unknown command 'NOPE'") && ok;
    ok = expect_reply(*client, "QUIT", "OK") && ok;

    if !ok session.failures += 1;
}

// A client that writes and reads its own keys while others do the same.
busy_client :: (session: *Session, number: s64) {
    address, _ := Io.local_address(session.server.listener);
    client, error := connect_client(Io.address_to_string(address));

    if error != .NONE {
        session.failures += 1;

        return;
    }

    defer close_client(*client);

    for round: 0..19 {
        key := tprint("client%-%", number, round);
        value := tprint("value %", round * number);

        ok := expect_reply(*client, tprint("SET % %", key, value), "OK");
        ok = expect_reply(*client, tprint("GET %", key), tprint("VALUE %", value)) && ok;

        if !ok session.failures += 1;

        // Give the other clients a turn between requests.
        Io.yield();
    }

    session.clients_done += 1;
}

run_session :: (session: *Session) {
    error: Io.Io_Error;

    session.server.listener, error = Io.tcp_listen("127.0.0.1:0");
    session.server.store = session.store;

    require(error == .NONE, "listening on an ephemeral port");

    Io.spawn(serve, *session.server);

    script_client(session);

    for 1..6 Io.spawn((busy: Busy) {
        busy_client(busy.session, busy.number);
    }, Busy.{ session, it });

    // The clients are fibers too: wait until all six are through, then stop the server.
    while session.clients_done < 6 Io.sleep(1);

    address, _ := Io.local_address(session.server.listener);
    closer, _ := connect_client(Io.address_to_string(address));

    expect_reply(*closer, "SHUTDOWN", "OK");
    close_client(*closer);
}

Busy :: struct {
    session: *Session;
    number: s64;
}

the_server_answers_many_clients_over_tcp :: (t: *Test) {
    session: Session;
    store: Store;

    store_open(*store, log_path(t));
    defer store_close(*store);

    session.store = *store;

    Io.io_run(run_session, *session);

    expect_equal(session.failures, 0);
    expect_equal(session.clients_done, 6);

    // The script's keys and all 6 * 20 of the busy clients' keys went through the one store.
    expect_equal(store.index.count, 1 + 6 * 20);

    value, found := store_get(*store, "client3-19");

    expect(found && value == "value 57");
} @test

// What the server acknowledged is in the log: a second store opened on the same file sees it.
acknowledged_writes_survive_a_restart :: (t: *Test) {
    session: Session;
    store: Store;

    store_open(*store, log_path(t));
    session.store = *store;

    Io.io_run(run_session, *session);
    store_close(*store);

    reopened := store_open(*store, log_path(t));

    defer store_close(*store);

    expect_equal(reopened.status, Open_Status.OK);

    value, found := store_get(*store, "colour");

    expect(found && value == "blue");
    expect_equal(store.index.count, 1 + 6 * 20);
} @test

Bulk :: struct {
    store: *Store;
    server: Server;
    rows: s64;
    value_ok: bool;
}

BIG_VALUE_SIZE :: 600 * 1000;

bulk_session :: (bulk: *Bulk) {
    error: Io.Io_Error;

    bulk.server.listener, error = Io.tcp_listen("127.0.0.1:0");
    bulk.server.store = bulk.store;

    require(error == .NONE, "listening on an ephemeral port");

    Io.spawn(serve, *bulk.server);

    address, _ := Io.local_address(bulk.server.listener);
    client, connect_error := connect_client(Io.address_to_string(address));

    require(connect_error == .NONE, "connecting");

    defer close_client(*client);

    rows, _ := call(*client, "SCAN item");

    bulk.rows = rows.count - 1;
    free_reply(rows);

    // A value of 600 kB, written and read back through the protocol in one line each way.
    big := alloc_string(BIG_VALUE_SIZE);

    defer free(big);

    for *big it.* = cast(u8) (it_index % 251);

    encoded := base64_encode(big);

    defer free(encoded);

    request := join("SET64 big", encoded, separator = " ");

    defer free(request);

    set_reply, _ := call(*client, request);

    free_reply(set_reply);

    reply, _ := call(*client, "GET big");

    defer free_reply(reply);

    bulk.value_ok = reply.count == 1 && reply[0].count == "VALUE64 ".count + encoded.count && ends_with(reply[0], encoded);

    shutdown_reply, _ := call(*client, "SHUTDOWN");

    free_reply(shutdown_reply);
}

// Long replies and long requests: a scan with more rows than a fiber's temporary storage could
// hold, and a value that is many times bigger than the read buffer.
large_replies_and_requests_work :: (t: *Test) {
    bulk: Bulk;
    store: Store;

    store_open(*store, log_path(t));
    defer store_close(*store);

    for 0..999 store_put(*store, tprint("item%", it), tprint("value of item %", it));

    bulk.store = *store;

    Io.io_run(bulk_session, *bulk);

    expect_equal(bulk.rows, 1000);
    expect(bulk.value_ok, "the 600 kB value came back whole");

    value, found := store_get(*store, "big");

    expect(found && value.count == BIG_VALUE_SIZE);
} @test

cut_off_session :: (bulk: *Bulk) {
    error: Io.Io_Error;

    bulk.server.listener, error = Io.tcp_listen("127.0.0.1:0");
    bulk.server.store = bulk.store;

    require(error == .NONE, "listening on an ephemeral port");

    Io.spawn(serve, *bulk.server);

    local, _ := Io.local_address(bulk.server.listener);
    address := Io.address_to_string(local);

    // A complete request, then half of one with no line ending, then the client is gone.
    leaver, leave_error := connect_client(address);

    require(leave_error == .NONE, "connecting");

    Io.write(leaver.stream, "SET whole 1\nSET partial value");

    // A clean end of the stream (a closed socket may be reported as a reset instead).
    Io.shutdown_write(leaver.stream);
    Io.sleep(20);
    close_client(*leaver);

    // Let the server see the end of that connection before asking what it stored.
    Io.sleep(20);

    asker, ask_error := connect_client(address);

    require(ask_error == .NONE, "connecting");

    defer close_client(*asker);

    first := expect_reply(*asker, "GET whole", "VALUE 1");
    second := expect_reply(*asker, "GET partial", "MISSING");

    bulk.value_ok = first && second;

    expect_reply(*asker, "SHUTDOWN", "OK");
}

// A request cut off before its line ending is not run.
an_unfinished_request_is_ignored :: (t: *Test) {
    bulk: Bulk;
    store: Store;

    store_open(*store, log_path(t));
    defer store_close(*store);

    bulk.store = *store;

    Io.io_run(cut_off_session, *bulk);

    expect(bulk.value_ok, "the whole request ran and the cut one did not");

    _, found := store_get(*store, "partial");

    expect(!found, "the cut request left nothing in the store");
} @test
