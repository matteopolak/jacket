// Run with jacket bench --path stdlib/Extensions/Tls.
#import "Extensions/Testing";
#import "Basic";
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";

#load "../tests/fixtures/certs.jai";

// The text of the test file called `name`.
//
// - name: a file name from the fixtures, such as "ca.pem".
load :: (name: string) -> string {
    for DATA if it.name == name return it.text;

    assert(false, "no test file %", name);

    return "";
}

CHUNK :: 64 * 1024;

// A connected pair, TLS or plain

Pair :: struct {
    sender: Io.Writer;
    receiver: Io.Reader;
    // What to close afterwards.
    streams: [2] *Tls.Tls_Stream;
    tcps: [2] Io.Tcp_Stream;
}

Acceptor :: struct {
    listener: Io.Tcp_Listener;
    config: *Tls.Tls_Config;
    tls: bool;
    tcp: Io.Tcp_Stream;
    stream: *Tls.Tls_Stream;
    error: Tls.Tls_Error;
    done: *Io.Channel(bool);
}

// Accepts one connection (and finishes the TLS handshake when `job.tls`).
//
// - job: the listener, config and result slots to use.
accept_one :: (job: *Acceptor) {
    error: Io.Io_Error;

    job.tcp, error = Io.accept(job.listener);

    assert(error == .NONE, "accept: %", error);

    if job.tls {
        job.stream, job.error = Tls.tls_accept(job.config, Io.reader_of(job.tcp), Io.writer_of(job.tcp));

        assert(job.error.kind == .NONE, "server handshake: %", Tls.error_text(job.error));
    }

    Io.send(job.done, true);
}

// Connects a client to a server over a fresh listener. `client_sends` says which end writes.
//
// - tls: wrap the connection in TLS, or leave it plain.
// - server_config: the server's config.
// - client_config: the client's config.
// - client_sends: true if the client writes and the server reads, false for the reverse.
connect :: (tls: bool, server_config: *Tls.Tls_Config, client_config: *Tls.Tls_Config, client_sends: bool) -> Pair {
    pair: Pair;
    job: Acceptor;
    error: Io.Io_Error;

    job.config = server_config;
    job.tls = tls;
    job.done = Io.make_channel(bool, 1);

    job.listener, error = Io.tcp_listen("127.0.0.1:0");

    assert(error == .NONE, "listen: %", error);

    address, _ := Io.local_address(job.listener);

    Io.spawn(accept_one, *job);

    pair.tcps[0], error = Io.connect(address);

    assert(error == .NONE, "connect: %", error);

    client_stream: *Tls.Tls_Stream;

    if tls {
        tls_error: Tls.Tls_Error;

        client_stream, tls_error = Tls.tls_connect(client_config, Io.reader_of(pair.tcps[0]), Io.writer_of(pair.tcps[0]), "localhost");

        assert(tls_error.kind == .NONE, "client handshake: %", Tls.error_text(tls_error));
    }

    Io.recv(job.done);
    Io.close(job.listener);
    Io.free_channel(job.done);

    pair.tcps[1] = job.tcp;
    pair.streams[0] = client_stream;
    pair.streams[1] = job.stream;

    client_writer := ifx tls then Tls.writer_of(client_stream) else Io.writer_of(pair.tcps[0]);
    client_reader := ifx tls then Tls.reader_of(client_stream) else Io.reader_of(pair.tcps[0]);
    server_writer := ifx tls then Tls.writer_of(job.stream) else Io.writer_of(job.tcp);
    server_reader := ifx tls then Tls.reader_of(job.stream) else Io.reader_of(job.tcp);

    if client_sends {
        pair.sender = client_writer;
        pair.receiver = server_reader;
    } else {
        pair.sender = server_writer;
        pair.receiver = client_reader;
    }

    return pair;
}

// Closes both ends of the pair.
//
// - pair: the pair to close.
close_pair :: (pair: *Pair) {
    for pair.streams if it Tls.close(it);

    // The Tls close closed the TCP streams under TLS ones; a plain pair closes its own.
    if !pair.streams[0] {
        Io.close(pair.tcps[0]);
        Io.close(pair.tcps[1]);
    }
} @FreesArgument

// One transfer

Transfer :: struct {
    writer: Io.Writer;
    bytes: int;
    buffer: [] u8;
    done: *Io.Channel(bool);
}

// Writes `job.bytes` bytes to `job.writer` in chunks, then signals `job.done`.
//
// - job: where to write, how much, and the channel to signal.
send_bytes :: (job: *Transfer) {
    buffer := job.buffer;

    sent := 0;

    while sent < job.bytes {
        count := min(CHUNK, job.bytes - sent);

        _, error := Io.write(job.writer, black_box(([] u8).{ count, buffer.data }));

        assert(error == .NONE, "send: %", error);

        sent += count;
    }

    Io.send(job.done, true);
}

// Handshakes

Shaker :: struct {
    listener: Io.Tcp_Listener;
    config: *Tls.Tls_Config;
    count: int;
    done: *Io.Channel(bool);
}

// Accepts `job.count` connections and closes each after its handshake.
//
// - job: the listener, config and count.
shake_server :: (job: *Shaker) {
    defer Io.send(job.done, true);

    for 1..job.count {
        tcp, error := Io.accept(job.listener);

        assert(error == .NONE, "accept: %", error);

        stream, tls_error := Tls.tls_accept(job.config, Io.reader_of(tcp), Io.writer_of(tcp));

        // The client asserts on its own side; a server that is canceled at the end just leaves.
        if tls_error.kind != .NONE return;

        Tls.close(stream);
    }
}

tls_case :: (b: *Bench, variant: int) {
    state := Tls_Bench.{ b, variant };

    Io.io_run((state: *Tls_Bench) {
        b := state.b;
        server_config: Tls.Tls_Config;

        server_config.certificate_chain_pem = load("server.pem");
        server_config.private_key_pem = load("server.key");

        client_config: Tls.Tls_Config;

        client_config.roots_pem = load("ca.pem");

        error := Tls.tls_config_init(*server_config);

        assert(error.kind == .NONE, "server config: %", Tls.error_text(error));

        error = Tls.tls_config_init(*client_config);

        assert(error.kind == .NONE, "client config: %", Tls.error_text(error));

        defer Tls.tls_config_free(*server_config);
        defer Tls.tls_config_free(*client_config);
        pair: Pair;
        if state.variant != 3 pair = connect(state.variant != 0, *server_config, *client_config, state.variant != 2);

        defer if state.variant != 3 close_pair(*pair);
        bytes := 1024 * 1024;
        if state.variant != 3 set_bytes(b, bytes);
        else set_items(b, 1);

        if state.variant == 3 {
            listener, failure := Io.tcp_listen("127.0.0.1:0");

            assert(failure == .NONE);
            defer Io.close(listener);
            address, _ := Io.local_address(listener);
            job := Shaker.{ listener, *server_config, b.iterations, Io.make_channel(bool, 1) };

            defer Io.free_channel(job.done);
            Io.spawn(shake_server, *job);
            reset_timer(b);

            for 0..b.iterations - 1 {
                tcp, connect_error := Io.connect(black_box(address));

                assert(connect_error == .NONE);
                stream, handshake_error := Tls.tls_connect(*client_config, Io.reader_of(tcp), Io.writer_of(tcp), "localhost");

                assert(handshake_error.kind == .NONE);
                black_box(stream);
                Tls.close(stream);
            }

            Io.recv(job.done);
        } else {
            outgoing := NewArray(CHUNK, u8);
            incoming := NewArray(CHUNK, u8);

            defer array_free(outgoing);
            defer array_free(incoming);
            for *outgoing it.* = cast(u8) ((it_index * 7) & 0xff);

            job := Transfer.{ pair.sender, b.iterations * bytes, outgoing, Io.make_channel(bool, 1) };

            defer Io.free_channel(job.done);
            Io.spawn(send_bytes, *job);
            reset_timer(b);

            for 0..b.iterations - 1 {
                received := 0;
                while received < bytes {
                    count, failure := Io.read(black_box(pair.receiver), .{ min(incoming.count, bytes - received), incoming.data });
                    assert(failure == .NONE && count > 0);
                    received += count;
                }

                black_box(incoming);
                black_box(received);
            }

            Io.recv(job.done);
        }
    }, *state);
}

Tls_Bench :: struct {
    b: *Bench;
    variant: int;
}

plain_tcp :: (b: *Bench) {
    tls_case(b, 0);
} @bench

tls_upload :: (b: *Bench) {
    tls_case(b, 1);
} @bench

tls_download :: (b: *Bench) {
    tls_case(b, 2);
} @bench

tls_handshakes :: (b: *Bench) {
    tls_case(b, 3);
} @bench
