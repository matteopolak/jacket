// Throughput of a TLS stream over loopback: one-way bulk transfers in both directions (client to
// server, server to client) and handshakes per second. Both ends share one scheduler thread, so the
// numbers include both ends' encryption and decryption (and the fiber switches between them): a
// single stream's rate on one core, not the machine's network. A plain TCP transfer of the same
// size is printed first for reference.
//
//     jaic build stdlib/Extensions/Tls/bench/bench.jai -O2 -o /tmp/tls-bench
//     /tmp/tls-bench                  # 512 MiB per transfer, 5 rounds, 300 handshakes
//     /tmp/tls-bench 256 7 500        # MiB per transfer, rounds, handshakes
//     JAI_IO_BACKEND=epoll /tmp/tls-bench
//
// Every round runs plain, client-to-server and server-to-client in turn (interleaved, so drift
// affects them alike); the report is the median and the best of the rounds.
#import "Basic";
#import "Sort";
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";

#load "../tests/data/certs.jai";

load :: (name: string) -> string {
    for DATA if it.name == name return it.text;

    assert(false, "no test file %", name);

    return "";
}

CHUNK :: 64 * 1024;

now_ms :: () -> float64 {
    return to_float64_seconds(current_time_monotonic()) * 1000.0;
}

Options :: struct {
    mebibytes := 512;
    rounds := 5;
    handshakes := 300;
}

options: Options;

// ---------------------------------------------------------------------------------------------
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

close_pair :: (pair: *Pair) {
    for pair.streams if it Tls.close(it);

    // The Tls close closed the TCP streams under TLS ones; a plain pair closes its own.
    if !pair.streams[0] {
        Io.close(pair.tcps[0]);
        Io.close(pair.tcps[1]);
    }
}

// ---------------------------------------------------------------------------------------------
// One transfer

Transfer :: struct {
    writer: Io.Writer;
    bytes: int;
    done: *Io.Channel(bool);
}

send_bytes :: (job: *Transfer) {
    buffer := alloc_array(CHUNK);

    defer free(buffer.data);

    for *buffer it.* = cast(u8) ((it_index * 7) & 0xff);

    sent := 0;

    while sent < job.bytes {
        count := min(CHUNK, job.bytes - sent);

        _, error := Io.write(job.writer, .{ count, buffer.data });

        assert(error == .NONE, "send: %", error);

        sent += count;
    }

    Io.send(job.done, true);
}

alloc_array :: (count: int) -> [] u8 {
    data := cast(*u8) alloc(count);

    return .{ count, data };
}

// Moves `bytes` through the pair and returns the MiB/s the receiver saw.
transfer :: (pair: Pair, bytes: int) -> float64 {
    job := Transfer.{ writer = pair.sender, bytes = bytes, done = Io.make_channel(bool, 1) };

    buffer := alloc_array(CHUNK);

    defer free(buffer.data);

    started := now_ms();

    Io.spawn(send_bytes, *job);

    received := 0;

    while received < bytes {
        count, error := Io.read(pair.receiver, buffer);

        assert(error == .NONE && count > 0, "receive: % after % bytes", error, received);

        received += count;
    }

    elapsed := now_ms() - started;

    Io.recv(job.done);
    Io.free_channel(job.done);

    return (cast(float64) bytes / (1024.0 * 1024.0)) / (elapsed / 1000.0);
}

// ---------------------------------------------------------------------------------------------
// Handshakes

Shaker :: struct {
    listener: Io.Tcp_Listener;
    config: *Tls.Tls_Config;
    count: int;
}

shake_server :: (job: *Shaker) {
    for 1..job.count {
        tcp, error := Io.accept(job.listener);

        assert(error == .NONE, "accept: %", error);

        stream, tls_error := Tls.tls_accept(job.config, Io.reader_of(tcp), Io.writer_of(tcp));

        // The client asserts on its own side; a server that is canceled at the end just leaves.
        if tls_error.kind != .NONE return;

        Tls.close(stream);
    }
}

handshakes_per_second :: (server_config: *Tls.Tls_Config, client_config: *Tls.Tls_Config, count: int) -> float64 {
    job := Shaker.{ config = server_config, count = count };
    error: Io.Io_Error;

    job.listener, error = Io.tcp_listen("127.0.0.1:0");

    assert(error == .NONE, "listen: %", error);

    address, _ := Io.local_address(job.listener);

    Io.spawn(shake_server, *job);

    started := now_ms();

    for 1..count {
        tcp, connect_error := Io.connect(address);

        assert(connect_error == .NONE, "connect: %", connect_error);

        stream, tls_error := Tls.tls_connect(client_config, Io.reader_of(tcp), Io.writer_of(tcp), "localhost");

        assert(tls_error.kind == .NONE, "client handshake: %", Tls.error_text(tls_error));

        Tls.close(stream);
    }

    elapsed := now_ms() - started;

    Io.close(job.listener);

    return cast(float64) count / (elapsed / 1000.0);
}

// ---------------------------------------------------------------------------------------------

median :: (values: [] float64) -> float64 {
    sorted := NewArray(values.count, float64,, temp);

    for values sorted[it_index] = it;

    quick_sort(sorted, (a, b) => ifx a < b then -1 else ifx a > b then 1 else 0);

    return sorted[sorted.count / 2];
}

best :: (values: [] float64) -> float64 {
    result := values[0];

    for values if it > result result = it;

    return result;
}

report :: (name: string, values: [] float64, unit: string) {
    print("%  median % %, best % %\n", name, formatFloat(median(values), 0, 1), unit, formatFloat(best(values), 0, 1), unit);
}

run :: () {
    server_config: Tls.Tls_Config;

    server_config.certificate_chain_pem = load("server.pem");
    server_config.private_key_pem = load("server.key");

    client_config: Tls.Tls_Config;

    client_config.roots_pem = load("ca.pem");

    error := Tls.tls_config_init(*server_config);

    assert(error.kind == .NONE, "server config: %", Tls.error_text(error));

    error = Tls.tls_config_init(*client_config);

    assert(error.kind == .NONE, "client config: %", Tls.error_text(error));

    bytes := options.mebibytes * 1024 * 1024;

    plain: [..] float64;
    upload: [..] float64;
    download: [..] float64;
    shakes: [..] float64;

    // One warm-up round that is not counted.
    for round: 0..options.rounds {
        for variant: 0..3 {
            if variant == 3 {
                rate := handshakes_per_second(*server_config, *client_config, options.handshakes);

                if round > 0 array_add(*shakes, rate);

                continue;
            }

            pair := connect(variant != 0, *server_config, *client_config, variant != 2);

            if round == 0 && variant == 1 print("cipher suite %\n", Tls.cipher_suite(pair.streams[0]));

            rate := transfer(pair, bytes);

            close_pair(*pair);

            if round == 0 continue;

            if variant == 0 array_add(*plain, rate);
            else if variant == 1 array_add(*upload, rate);
            else array_add(*download, rate);
        }
    }

    print("% MiB per transfer, % rounds\n", options.mebibytes, options.rounds);
    report("plain TCP           ", plain, "MiB/s");
    report("TLS client -> server", upload, "MiB/s");
    report("TLS server -> client", download, "MiB/s");
    report("handshakes          ", shakes, "/s");

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

main :: () {
    arguments := get_command_line_arguments();

    if arguments.count > 1 options.mebibytes = number_or(arguments[1], options.mebibytes);
    if arguments.count > 2 options.rounds = number_or(arguments[2], options.rounds);
    if arguments.count > 3 options.handshakes = number_or(arguments[3], options.handshakes);

    Io.io_run(run);
}

number_or :: (text: string, fallback: int) -> int {
    value, ok := to_integer(text);

    return ifx ok then value else fallback;
}
