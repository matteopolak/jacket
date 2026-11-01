#import "Basic";
#import "File";
#import "Extensions/Testing";
#import,dir "../chat";

protocol_grammar :: (t: *Test) {
    for string.["NICK Ada", "JOIN room-1", "SAY hello world", "DM Bob private words", "LEAVE", "HISTORY", "QUIT", "PING", "EMAIL ada@example.test"] {
        request, valid := parse_request(it);
        expect(valid, it);
        expect(request.verb.count > 0);
    }

    for string.["", "nick Ada", "NICK", "NICK a b", "JOIN ../bad", "SAY ", "SAY bad\ttext", "DM Bob", "QUIT now", "PING ", "EMAIL x\r\nTo:bad", "NICK \x1b"] {
        _, valid := parse_request(it);
        expect(!valid, it);
    }

    request, valid := parse_request("DM Bob spaces stay in body");

    expect(valid);
    expect_equal(request.target, "Bob");
    expect_equal(request.text, "spaces stay in body");
} @test

rate_limit_and_clock_reversal :: (t: *Test) {
    limiter: Limiter;
    for 1..20 expect(allow(*limiter, 0, 2));

    expect(!allow(*limiter, 0, 2));
    expect(!allow(*limiter, 0.25, 2));
    expect(allow(*limiter, 0.5, 2));
    expect(!allow(*limiter, -1, 2));
    expect(allow(*limiter, 100, 2));
    expect_equal(limiter.tokens, cast(float64) 19);
} @test

mentions_have_boundaries :: (t: *Test) {
    expect(mentions("hi @Ada!", "Ada"));
    expect(mentions("@Ada", "Ada"));
    expect(!mentions("@Adaline", "Ada"));
    expect(!mentions("mail@Ada", "Ada"));
    expect(!mentions("@ada", "Ada"));
    expect(!mentions("", "Ada"));
} @test

history_is_bounded_and_rooms_are_isolated :: (t: *Test) {
    model := Model.{ history_limit = 2 };

    defer model_free(*model);
    lobby := room_find(*model, "lobby");
    other := room_find(*model, "other");

    expect(lobby == room_find(*model, "lobby"));
    expect(!room_find(*model, "missing", false));
    remember(*model, lobby, "A", "one");
    remember(*model, lobby, "B", "two");
    remember(*model, lobby, "C", "three");
    expect_equal(lobby.history.count, 2);
    expect_equal(lobby.history[0].text, "two");
    expect_equal(lobby.history[1].nick, "C");
    expect_equal(other.history.count, 0);
} @test

persist_restart_and_trim_torn_tail :: (t: *Test) {
    path := tprint("%/history.log", temp_dir(t));
    model := Model.{ history_limit = 2 };

    require(model_open(*model, path));
    room := room_find(*model, "lobby");

    require(record_message(*model, room, "Ada", "one"));
    require(record_message(*model, room, "Bob", "two"));
    require(record_message(*model, room, "Ada", "three"));
    model_free(*model);
    data, read := read_entire_file(path);

    require(read);
    defer free(data);
    require(write_entire_file(path, tprint("%lobby\tBob\tcut", data)));
    model.history_limit = 2;
    require(model_open(*model, path));
    room = room_find(*model, "lobby", false);
    require(room != null);
    expect_equal(room.history.count, 2);
    expect_equal(room.history[0].text, "two");
    require(record_message(*model, room, "Bob", "after recovery"));
    model_free(*model);
    model.history_limit = 2;
    require(model_open(*model, path));
    defer model_free(*model);
    room = room_find(*model, "lobby", false);
    expect_equal(room.history[1].text, "after recovery");
} @test

corrupt_complete_record_is_not_silently_dropped :: (t: *Test) {
    path := tprint("%/corrupt.log", temp_dir(t));

    require(write_entire_file(path, "lobby\tAda\tmessage\tbad-checksum\n"));
    model: Model;

    defer model_free(*model);
    expect(!model_open(*model, path));
    text, ok := read_entire_file(path);

    defer free(text);
    expect(ok);
    expect_equal(text, "lobby\tAda\tmessage\tbad-checksum\n");
} @test

room_limit :: (t: *Test) {
    model: Model;
    defer model_free(*model);
    for 1..MAX_ROOMS expect(room_find(*model, tprint("room%", it)) != null);
    expect(room_find(*model, "overflow") == null);
} @test

peer_events :: (peer: *Peer) -> string {
    output: String_Builder;

    while true {
        line, found := Io.try_recv(peer.output);
        if !found break;
        print_to_builder(*output, "%\n", line);
        free(line);
    }

    result := builder_to_string(*output);

    free_buffers(*output);

    return result;
}

issue :: (peer: *Peer, line: string) {
    request, valid := parse_request(line);
    assert(valid);
    execute(peer, request);
}

room_commands_route_events_and_enforce_nicks :: (t: *Test) {
    Io.io_run((t: *Test) {
        server: Server;
        server.options.rate = 100000;
        defer model_free(*server.model);
        a := Peer.{ server = *server, output = Io.make_channel(string, 128) };
        b := Peer.{ server = *server, output = Io.make_channel(string, 128) };
        c := Peer.{ server = *server, output = Io.make_channel(string, 128) };

        server.peers[0] = *a;
        server.peers[1] = *b;
        server.peers[2] = *c;

        defer {
            free(peer_events(*a));
            free(peer_events(*b));
            free(peer_events(*c));
            Io.free_channel(a.output);
            Io.free_channel(b.output);
            Io.free_channel(c.output);

            for server.users {
                free(it.nick);
                free(it);
            }

            array_free(server.users);
        }

        issue(*a, "NICK Alice");
        issue(*b, "NICK Bob");
        issue(*c, "NICK Bob");
        duplicate := peer_events(*c);

        defer free(duplicate);
        expect_equal(duplicate, "ERR nickname-in-use\n");
        issue(*a, "JOIN alpha");
        issue(*b, "JOIN beta");
        free(peer_events(*a));
        free(peer_events(*b));
        expect(a.room != b.room);
        issue(*a, "DM Bob secret");
        private := peer_events(*b);

        defer free(private);
        expect_equal(private, "PRIVATE Alice secret\n");
        issue(*b, "JOIN alpha");
        expect(a.room == b.room);
        free(peer_events(*a));
        free(peer_events(*b));
        issue(*a, "LEAVE");
        expect(a.room == null);
        left := peer_events(*b);

        defer free(left);
        expect_equal(left, "LEFT alpha Alice\n");
    }, t);
} @test
