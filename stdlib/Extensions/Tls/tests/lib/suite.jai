// The Extensions/Tls tests, shared by the test programs next to this directory. Every test runs on
// a fiber inside `io_run`, on whichever backend that scheduler has. Nothing here depends on the
// network beyond loopback; the certificates are the test-only ones in ../data (see the script
// there). Run the programs from their own directory.
#import "Basic";
#import "File";
#import "String";
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";

LOOPBACK :: "127.0.0.1:0";

now_ms :: () -> s64 {
    return to_milliseconds(current_time_monotonic());
}

// The test certificates and keys, as string constants (data/certs.jai, made by data/make_certs.sh
// from the files beside it), so the program carries them and reads no file.
#load "../data/certs.jai";

load :: (name: string) -> string {
    for DATA if it.name == name return it.text;

    assert(false, "no test file %", name);

    return "";
}

pattern :: (index: int) -> u8 {
    return cast(u8) ((index * 31 + index / 251) & 0xff);
}

has :: (text: string, part: string) -> bool {
    found, _ := contains(text, part);

    return found;
}

// A config that proves itself with `certificate` (a file in data/, "" for none) and trusts the
// test CA (or `roots`).
make_config :: (certificate := "", roots := "ca", alpn: [] string = .[], min := Tls.Tls_Version.TLS_1_2, max := Tls.Tls_Version.TLS_1_3) -> Tls.Tls_Config {
    config: Tls.Tls_Config;

    if certificate.count {
        config.certificate_chain_pem = load(tprint("%.pem", certificate));
        config.private_key_pem = load(tprint("%.key", certificate));
    }

    if roots.count config.roots_pem = load(tprint("%.pem", roots));

    config.alpn = alpn;
    config.min_version = min;
    config.max_version = max;

    return config;
}

init :: (config: *Tls.Tls_Config) {
    error := Tls.tls_config_init(config);

    assert(error.kind == .NONE, "tls_config_init: %", Tls.error_text(error));
}

// ---------------------------------------------------------------------------------------------
// A connected pair

Handshake :: struct {
    stream: *Tls.Tls_Stream;
    error: Tls.Tls_Error;
    tcp: Io.Tcp_Stream;
}

Acceptor :: struct {
    listener: Io.Tcp_Listener;
    config: *Tls.Tls_Config;
    results: *Io.Channel(Handshake);
}

accept_and_handshake :: (job: *Acceptor) {
    result: Handshake;
    error: Io.Io_Error;

    result.tcp, error = Io.accept(job.listener);

    assert(error == .NONE, "accept: %", error);

    result.stream, result.error = Tls.tls_accept(job.config, Io.reader_of(result.tcp), Io.writer_of(result.tcp));

    Io.send(job.results, result);
}

Pair :: struct {
    client: Handshake;
    server: Handshake;
}

// Connects a client and a server on loopback and runs both handshakes. Either side may fail:
// look at the `error` of each. A failed side's TCP connection is closed.
connect_pair :: (server_config: *Tls.Tls_Config, client_config: *Tls.Tls_Config, name := "localhost") -> Pair {
    pair: Pair;

    job: Acceptor;
    error: Io.Io_Error;

    job.config = server_config;
    job.results = Io.make_channel(Handshake, 1);

    job.listener, error = Io.tcp_listen(LOOPBACK);

    assert(error == .NONE, "listen: %", error);

    address, _ := Io.local_address(job.listener);

    Io.spawn(accept_and_handshake, *job);

    pair.client.tcp, error = Io.connect(address);

    assert(error == .NONE, "connect: %", error);

    pair.client.stream, pair.client.error = Tls.tls_connect(client_config, Io.reader_of(pair.client.tcp), Io.writer_of(pair.client.tcp), name);

    // A client that failed leaves the server waiting for it: close, so the server's side ends too.
    if pair.client.error.kind != .NONE Io.close(pair.client.tcp);

    pair.server, _ = Io.recv(job.results);

    if pair.server.error.kind != .NONE Io.close(pair.server.tcp);

    Io.close(job.listener);
    Io.free_channel(job.results);

    return pair;
}

close_pair :: (pair: *Pair) {
    if pair.client.stream Tls.close(pair.client.stream);
    if pair.server.stream Tls.close(pair.server.stream);

    pair.client.stream = null;
    pair.server.stream = null;
}

expect_connected :: (pair: Pair) {
    assert(pair.client.error.kind == .NONE, "client: %", Tls.error_text(pair.client.error));
    assert(pair.server.error.kind == .NONE, "server: %", Tls.error_text(pair.server.error));
}

// Sends `text` from one stream and reads it from the other.
pass :: (from: *Tls.Tls_Stream, to: *Tls.Tls_Stream, text: string) {
    count, error := Tls.write(from, text);

    assert(error == .NONE && count == text.count, "write: % (% of %)", error, count, text.count);

    buffer: [256] u8;

    count, error = Io.read_full(Tls.reader_of(to), .{ text.count, buffer.data });

    assert(error == .NONE && count == text.count, "read: % (% of %)", error, count, text.count);
    assert(string.{ count, buffer.data } == text, "got the wrong bytes");
}

// ---------------------------------------------------------------------------------------------
// Handshakes

test_handshakes :: () {
    for certificate: string.["server", "rsa"] {
        for versions: 0..2 {
            min := ifx versions == 1 then Tls.Tls_Version.TLS_1_3 else Tls.Tls_Version.TLS_1_2;
            max := ifx versions == 0 then Tls.Tls_Version.TLS_1_2 else Tls.Tls_Version.TLS_1_3;
            expected := ifx versions == 0 then "TLSv1.2" else "TLSv1.3";

            server_config := make_config(certificate, "");
            client_config := make_config("", "ca", min = min, max = max);

            init(*server_config);
            init(*client_config);

            pair := connect_pair(*server_config, *client_config);

            expect_connected(pair);

            assert(Tls.version(pair.client.stream) == expected, "%: got %", expected, Tls.version(pair.client.stream));
            assert(Tls.version(pair.server.stream) == expected);
            assert(Tls.server_name(pair.server.stream) == "localhost", "sni: '%'", Tls.server_name(pair.server.stream));
            assert(Tls.alpn_protocol(pair.client.stream) == "");

            pass(pair.client.stream, pair.server.stream, "ping");
            pass(pair.server.stream, pair.client.stream, "pong");

            close_pair(*pair);

            Tls.tls_config_free(*server_config);
            Tls.tls_config_free(*client_config);
        }
    }
}

test_version_mismatch :: () {
    server_config := make_config("server", "", min = .TLS_1_3);
    client_config := make_config("", "ca", max = .TLS_1_2);

    init(*server_config);
    init(*client_config);

    pair := connect_pair(*server_config, *client_config);

    assert(pair.client.error.kind == .HANDSHAKE, "client: %", Tls.error_text(pair.client.error));
    assert(pair.server.error.kind != .NONE);
    assert(pair.client.stream == null && pair.server.stream == null);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

test_alpn :: () {
    server_config := make_config("server", "", string.["h2", "http/1.1"]);

    init(*server_config);

    // The client's first choice that the server also speaks.
    both := make_config("", "ca", string.["http/1.1", "spdy"]);
    other := make_config("", "ca", string.["spdy/3"]);
    none := make_config("", "ca");

    init(*both);
    init(*other);
    init(*none);

    pair := connect_pair(*server_config, *both);

    expect_connected(pair);

    assert(Tls.alpn_protocol(pair.client.stream) == "http/1.1", "client: '%'", Tls.alpn_protocol(pair.client.stream));
    assert(Tls.alpn_protocol(pair.server.stream) == "http/1.1");

    close_pair(*pair);

    // No protocol in common: the handshake fails on both sides.
    pair = connect_pair(*server_config, *other);

    assert(pair.client.error.kind == .HANDSHAKE, "client: %", Tls.error_text(pair.client.error));
    assert(pair.server.error.kind == .HANDSHAKE, "server: %", Tls.error_text(pair.server.error));

    // A client that offers nothing gets no ALPN.
    pair = connect_pair(*server_config, *none);

    expect_connected(pair);

    assert(Tls.alpn_protocol(pair.client.stream) == "");
    assert(Tls.alpn_protocol(pair.server.stream) == "");

    close_pair(*pair);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*both);
    Tls.tls_config_free(*other);
    Tls.tls_config_free(*none);
}

// ---------------------------------------------------------------------------------------------
// Data

Transfer :: struct {
    stream: *Tls.Tls_Stream;
    size: int;
    seed: int;
    errors: int;
    done: bool;
}

send_pattern :: (job: *Transfer) {
    data := cast(*u8) alloc(job.size);

    defer free(data);

    for 0..job.size - 1 data[it] = pattern(it + job.seed);

    count, error := Tls.write(job.stream, .{ job.size, data });

    if error != .NONE || count != job.size job.errors += 1;

    job.done = true;
}

receive_pattern :: (job: *Transfer) {
    data := cast(*u8) alloc(job.size);

    defer free(data);

    count, error := Io.read_full(Tls.reader_of(job.stream), .{ job.size, data });

    if error != .NONE || count != job.size {
        job.errors += 1;
    } else {
        for 0..job.size - 1 {
            if data[it] != pattern(it + job.seed) {
                job.errors += 1;

                break;
            }
        }
    }

    job.done = true;
}

wait_all :: (jobs: [] *bool, limit := 60_000) {
    started := now_ms();

    while true {
        assert(now_ms() - started < limit, "the transfers did not finish in % ms", limit);

        all := true;

        for jobs if !(<<it) all = false;

        if all break;

        Io.sleep(1);
    }
}

// More than a record and more than the socket buffers, both ways at once.
test_large_transfers :: () {
    for version: 0..1 {
        server_config := make_config("server", "", min = ifx version == 0 then Tls.Tls_Version.TLS_1_2 else Tls.Tls_Version.TLS_1_3);
        client_config := make_config("", "ca");

        init(*server_config);
        init(*client_config);

        pair := connect_pair(*server_config, *client_config);

        expect_connected(pair);

        SIZE :: 1_000_000;

        jobs: [4] Transfer;

        jobs[0] = .{ stream = pair.client.stream, size = SIZE, seed = 1 };
        jobs[1] = .{ stream = pair.server.stream, size = SIZE, seed = 1 };
        jobs[2] = .{ stream = pair.server.stream, size = SIZE + 77, seed = 5 };
        jobs[3] = .{ stream = pair.client.stream, size = SIZE + 77, seed = 5 };

        Io.spawn(send_pattern, *jobs[0]);
        Io.spawn(receive_pattern, *jobs[1]);
        Io.spawn(send_pattern, *jobs[2]);
        Io.spawn(receive_pattern, *jobs[3]);

        wait_all(.[*jobs[0].done, *jobs[1].done, *jobs[2].done, *jobs[3].done]);

        for jobs assert(it.errors == 0, "transfer % failed", it_index);

        close_pair(*pair);

        Tls.tls_config_free(*server_config);
        Tls.tls_config_free(*client_config);
    }
}

// Writes of every awkward size (a block, a record, a batch of records, one byte over each) read back
// in pieces of another size, so the partial blocks of the cipher and the read-ahead of the stream
// both meet the ends of their buffers.
test_awkward_sizes :: () {
    sizes := int.[1, 2, 15, 16, 17, 31, 33, 100, 127, 129, 4095, 8191, 8192, 8193, 16383, 16384, 16385, 65535, 65536, 65537, 150_001];
    pieces := int.[1, 7, 16, 1000, 8192, 20_000];

    for version: 0..1 {
        server_config := make_config("server", "", min = ifx version == 0 then Tls.Tls_Version.TLS_1_2 else Tls.Tls_Version.TLS_1_3);
        client_config := make_config("", "ca");

        init(*server_config);
        init(*client_config);

        pair := connect_pair(*server_config, *client_config);

        expect_connected(pair);

        suite := Tls.cipher_suite(pair.client.stream);

        assert(has(suite, "GCM") || has(suite, "CHACHA20"), "an AEAD suite: %", suite);
        assert(Tls.cipher_suite(pair.server.stream) == suite, "both ends agree on %", suite);

        for size: sizes {
            for from_client: 0..1 {
                sender := ifx from_client then pair.client.stream else pair.server.stream;
                receiver := ifx from_client then pair.server.stream else pair.client.stream;
                piece := pieces[(size + from_client) % pieces.count];

                job := Transfer.{ stream = sender, size = size, seed = size };

                Io.spawn(send_pattern, *job);

                data := cast(*u8) alloc(size);
                got := 0;

                while got < size {
                    count, error := Io.read_full(Tls.reader_of(receiver), .{ min(piece, size - got), data + got });

                    assert(error == .NONE && count > 0, "read % bytes of % (pieces of %): %", got, size, piece, error);

                    got += count;
                }

                for 0..size - 1 assert(data[it] == pattern(it + size), "byte % of % differs", it, size);

                free(data);

                wait_all(.[*job.done]);

                assert(job.errors == 0, "the write of % bytes failed", size);
            }
        }

        close_pair(*pair);

        Tls.tls_config_free(*server_config);
        Tls.tls_config_free(*client_config);
    }
}

// Socket buffers of a few KiB, so a megabyte cannot hide in them.
#if OS == .WINDOWS {
    ws2_32 :: #system_library "ws2_32";
    set_socket_option :: (socket: s64, level: s32, name: s32, value: *void, length: s32) -> s32 #foreign ws2_32 "setsockopt";

    SOL_SOCKET :: 0xffff;
    SO_SNDBUF :: 0x1001;
    SO_RCVBUF :: 0x1002;
} else {
    socket_library :: #system_library "libc";
    set_socket_option :: (socket: s32, level: s32, name: s32, value: *void, length: u32) -> s32 #foreign socket_library "setsockopt";

    #if OS == .MACOS {
        SOL_SOCKET :: 0xffff;
        SO_SNDBUF :: 0x1001;
        SO_RCVBUF :: 0x1002;
    } else {
        SOL_SOCKET :: 1;
        SO_SNDBUF :: 7;
        SO_RCVBUF :: 8;
    }
}

shrink_buffers :: (tcp: Io.Tcp_Stream) {
    size: s32 = 8192;

    #if OS == .WINDOWS {
        set_socket_option(tcp.handle.fd, SOL_SOCKET, SO_SNDBUF, *size, 4);
        set_socket_option(tcp.handle.fd, SOL_SOCKET, SO_RCVBUF, *size, 4);
    } else {
        set_socket_option(cast(s32) tcp.handle.fd, SOL_SOCKET, SO_SNDBUF, *size, 4);
        set_socket_option(cast(s32) tcp.handle.fd, SOL_SOCKET, SO_RCVBUF, *size, 4);
    }
}

// Both ends write a lot at once (200 KB, against socket buffers of a few KiB): each side has to keep reading
// while its own write waits for the peer, or both wait for each other for good. (With the socket
// buffers of a loopback connection, which take a megabyte, the large transfer above never shows
// this.)
test_duplex_with_small_socket_buffers :: () {
    for version: 0..1 {
        server_config := make_config("server", "", min = ifx version == 0 then Tls.Tls_Version.TLS_1_2 else Tls.Tls_Version.TLS_1_3);
        client_config := make_config("", "ca");

        init(*server_config);
        init(*client_config);

        pair := connect_pair(*server_config, *client_config);

        expect_connected(pair);

        shrink_buffers(pair.client.tcp);
        shrink_buffers(pair.server.tcp);

        SIZE :: 200_000;

        jobs: [4] Transfer;

        jobs[0] = .{ stream = pair.client.stream, size = SIZE, seed = 1 };
        jobs[1] = .{ stream = pair.server.stream, size = SIZE, seed = 1 };
        jobs[2] = .{ stream = pair.server.stream, size = SIZE + 77, seed = 5 };
        jobs[3] = .{ stream = pair.client.stream, size = SIZE + 77, seed = 5 };

        Io.spawn(send_pattern, *jobs[0]);
        Io.spawn(receive_pattern, *jobs[1]);
        Io.spawn(send_pattern, *jobs[2]);
        Io.spawn(receive_pattern, *jobs[3]);

        wait_all(.[*jobs[0].done, *jobs[1].done, *jobs[2].done, *jobs[3].done], 40_000);

        for jobs assert(it.errors == 0, "transfer % failed", it_index);

        close_pair(*pair);

        Tls.tls_config_free(*server_config);
        Tls.tls_config_free(*client_config);
    }
}

test_close_notify :: () {
    server_config := make_config("server", "");
    client_config := make_config("", "ca");

    init(*server_config);
    init(*client_config);

    pair := connect_pair(*server_config, *client_config);

    expect_connected(pair);

    pass(pair.client.stream, pair.server.stream, "last words");

    // The server closes properly: the client reads the end, once everything sent is read.
    Tls.write(pair.server.stream, "bye");
    Tls.close(pair.server.stream);

    pair.server.stream = null;

    buffer: [16] u8;

    count, error := Io.read_full(Tls.reader_of(pair.client.stream), .{ 3, buffer.data });

    assert(error == .NONE && string.{ count, buffer.data } == "bye");

    count, error = Tls.read(pair.client.stream, buffer);

    assert(error == .NONE && count == 0, "the end of the stream: % %", count, error);

    count, error = Tls.read(pair.client.stream, buffer);

    assert(error == .NONE && count == 0, "the end again");

    close_pair(*pair);

    // The server only drops the connection: that is not the end of the data.
    pair = connect_pair(*server_config, *client_config);

    expect_connected(pair);

    Io.close(pair.server.tcp);

    count, error = Tls.read(pair.client.stream, buffer);

    assert(error == .CONNECTION_RESET, "truncated: % %", count, error);
    assert(Tls.last_error(pair.client.stream).kind == .TRUNCATED, "kind: %", Tls.error_text(Tls.last_error(pair.client.stream)));
    assert(Tls.ended_without_close_notify(Tls.reader_of(pair.client.stream)), "the reader says the end was ragged");
    assert(!Tls.ended_without_close_notify(Io.reader_of(pair.client.tcp)), "a TCP reader is not a TLS one");

    // And it stays broken.
    count, error = Tls.read(pair.client.stream, buffer);

    assert(error == .CONNECTION_RESET);

    // The server stream's connection is already closed: only free the client side.
    Tls.close(pair.client.stream, true);
    Tls.close(pair.server.stream, true);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

// A writer that stops after a number of bytes and shuts the connection down: the peer sees a
// record that never ends.
Cutter :: struct {
    tcp: Io.Tcp_Stream;
    remaining := -1;
}

cutter_write :: (data: *void, bytes: [] u8) -> s64, Io.Io_Error {
    cutter := cast(*Cutter) data;

    if cutter.remaining < 0 return Io.write_some(cutter.tcp, bytes);

    if cutter.remaining == 0 {
        Io.shutdown_write(cutter.tcp);

        return 0, .BROKEN_PIPE;
    }

    take := min(cutter.remaining, bytes.count);
    count, error := Io.write_some(cutter.tcp, .{ take, bytes.data });

    cutter.remaining -= count;

    return count, error;
}

cutter_close :: (data: *void, now: bool) -> Io.Io_Error {
    cutter := cast(*Cutter) data;

    return Io.close(cutter.tcp, now);
}

test_cut_in_the_middle_of_a_record :: () {
    server_config := make_config("server", "");
    client_config := make_config("", "ca");

    init(*server_config);
    init(*client_config);

    listener, error := Io.tcp_listen(LOOPBACK);

    assert(error == .NONE);

    address, _ := Io.local_address(listener);

    cutter: Cutter;

    serve :: (state: *Cutting) {
        tcp, error := Io.accept(state.listener);

        assert(error == .NONE);

        state.cutter.tcp = tcp;

        writer := Io.Writer.{ data = state.cutter, write_proc = cutter_write, close_proc = cutter_close };

        stream, handshake_error := Tls.tls_accept(state.config, Io.reader_of(tcp), writer);

        assert(handshake_error.kind == .NONE, "%", Tls.error_text(handshake_error));

        // 12 bytes of the next record, which holds 200 bytes, then the end of the connection.
        state.cutter.remaining = 12;

        block: [200] u8;
        count, write_error := Tls.write(stream, block);

        assert(write_error != .NONE && count == 0, "cut write: % %", count, write_error);

        state.stream = stream;
        state.done = true;
    }

    state := Cutting.{ listener = listener, config = *server_config, cutter = *cutter };

    Io.spawn(serve, *state);

    tcp, connect_error := Io.connect(address);

    assert(connect_error == .NONE);

    stream, tls_error := Tls.tls_connect(*client_config, Io.reader_of(tcp), Io.writer_of(tcp), "localhost");

    assert(tls_error.kind == .NONE, "%", Tls.error_text(tls_error));

    buffer: [300] u8;
    count, read_error := Tls.read(stream, buffer);

    assert(read_error == .CONNECTION_RESET && count == 0, "cut record: % %", count, read_error);
    assert(Tls.last_error(stream).kind == .TRUNCATED, "%", Tls.error_text(Tls.last_error(stream)));

    while !state.done Io.sleep(1);

    Tls.close(stream, true);
    Tls.close(state.stream, true);
    Io.close(listener);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

Cutting :: struct {
    listener: Io.Tcp_Listener;
    config: *Tls.Tls_Config;
    cutter: *Cutter;
    stream: *Tls.Tls_Stream;
    done: bool;
}

// ---------------------------------------------------------------------------------------------
// Deadlines and cancellation

// Accepts a connection and says nothing.
silent_server :: (state: *Silent) {
    tcp, error := Io.accept(state.listener);

    assert(error == .NONE);

    state.accepted = true;

    while !state.release Io.sleep(1);

    Io.close(tcp);

    state.done = true;
}

Silent :: struct {
    listener: Io.Tcp_Listener;
    accepted: bool;
    release: bool;
    done: bool;
}

test_handshake_deadline :: () {
    client_config := make_config("", "ca");

    init(*client_config);

    state: Silent;
    error: Io.Io_Error;

    state.listener, error = Io.tcp_listen(LOOPBACK);

    address, _ := Io.local_address(state.listener);

    Io.spawn(silent_server, *state);

    tcp, connect_error := Io.connect(address);

    assert(connect_error == .NONE);

    stream: *Tls.Tls_Stream;
    tls_error: Tls.Tls_Error;

    started := now_ms();

    Io.with_deadline(100, #code {
        stream, tls_error = Tls.tls_connect(*client_config, Io.reader_of(tcp), Io.writer_of(tcp), "localhost");
    });

    elapsed := now_ms() - started;

    assert(stream == null);
    assert(tls_error.kind == .IO && tls_error.io == .TIMED_OUT, "handshake: %", Tls.error_text(tls_error));
    assert(elapsed >= 90 && elapsed < 2000, "took % ms", elapsed);

    Io.close(tcp);

    state.release = true;

    while !state.done Io.sleep(1);

    Io.close(state.listener);

    Tls.tls_config_free(*client_config);
}

Connecting :: struct {
    config: *Tls.Tls_Config;
    address: Io.Address;
    error: Tls.Tls_Error;
    finished: bool;
}

connect_and_handshake :: (job: *Connecting) {
    tcp, io_error := Io.connect(job.address);

    assert(io_error == .NONE);

    stream, error := Tls.tls_connect(job.config, Io.reader_of(tcp), Io.writer_of(tcp), "localhost");

    job.error = error;

    assert(stream == null);

    Io.close(tcp);

    job.finished = true;
}

test_handshake_cancel :: () {
    client_config := make_config("", "ca");

    init(*client_config);

    state: Silent;
    error: Io.Io_Error;

    state.listener, error = Io.tcp_listen(LOOPBACK);

    address, _ := Io.local_address(state.listener);

    Io.spawn(silent_server, *state);

    job := Connecting.{ config = *client_config, address = address };
    handle := Io.spawn(connect_and_handshake, *job);

    while !state.accepted Io.sleep(1);

    Io.sleep(20);

    assert(!job.finished, "the handshake ended by itself");

    Io.cancel(handle);

    while !job.finished Io.sleep(1);

    assert(job.error.kind == .IO && job.error.io == .CANCELED, "canceled handshake: %", Tls.error_text(job.error));

    state.release = true;

    while !state.done Io.sleep(1);

    Io.close(state.listener);

    Tls.tls_config_free(*client_config);
}

// ---------------------------------------------------------------------------------------------
// Verification

test_verification_failures :: () {
    client_config := make_config("", "ca");

    init(*client_config);

    // Signed by a CA the client does not trust.
    unknown := make_config("unknown", "");

    init(*unknown);

    pair := connect_pair(*unknown, *client_config);

    assert(pair.client.error.kind == .VERIFY, "unknown CA: %", Tls.error_text(pair.client.error));
    assert(has(Tls.detail(*pair.client.error), "trusted"), "message: %", Tls.detail(*pair.client.error));
    assert(pair.client.stream == null);

    // For another name.
    wrong := make_config("wrong_name", "");

    init(*wrong);

    pair = connect_pair(*wrong, *client_config);

    assert(pair.client.error.kind == .VERIFY, "wrong name: %", Tls.error_text(pair.client.error));
    assert(has(Tls.detail(*pair.client.error), "Common Name"), "message: %", Tls.detail(*pair.client.error));

    // The right certificate asked for under a name it is not valid for.
    good := make_config("server", "");

    init(*good);

    pair = connect_pair(*good, *client_config, "not-localhost.test");

    assert(pair.client.error.kind == .VERIFY, "asked name: %", Tls.error_text(pair.client.error));

    // Out of date.
    expired := make_config("expired", "");

    init(*expired);

    pair = connect_pair(*expired, *client_config);

    assert(pair.client.error.kind == .VERIFY, "expired: %", Tls.error_text(pair.client.error));
    assert(has(Tls.detail(*pair.client.error), "expired"), "message: %", Tls.detail(*pair.client.error));

    // No roots at all in the config means the system's, which do not know the test CA.
    system := make_config("", "");

    init(*system);

    pair = connect_pair(*good, *system);

    assert(pair.client.error.kind == .VERIFY || pair.client.error.kind == .CONFIG, "system roots: %", Tls.error_text(pair.client.error));

    // The right name and the right CA connect, by an IP address as well.
    pair = connect_pair(*good, *client_config, "127.0.0.1");

    expect_connected(pair);

    close_pair(*pair);

    Tls.tls_config_free(*client_config);
    Tls.tls_config_free(*unknown);
    Tls.tls_config_free(*wrong);
    Tls.tls_config_free(*good);
    Tls.tls_config_free(*expired);
    Tls.tls_config_free(*system);
}

test_verification_off :: () {
    unknown := make_config("unknown", "");
    client_config := make_config("", "");

    client_config.verify = false;

    init(*unknown);
    init(*client_config);

    pair := connect_pair(*unknown, *client_config, "whatever.test");

    expect_connected(pair);

    pass(pair.client.stream, pair.server.stream, "unchecked");

    close_pair(*pair);

    // A name is only needed to check one.
    pair = connect_pair(*unknown, *client_config, "");

    expect_connected(pair);

    close_pair(*pair);

    Tls.tls_config_free(*unknown);
    Tls.tls_config_free(*client_config);
}

test_client_certificates :: () {
    server_config := make_config("server", "ca");

    server_config.client_auth = true;

    init(*server_config);

    with_certificate := make_config("client", "ca");
    without := make_config("", "ca");
    wrong_ca := make_config("unknown", "ca");

    init(*with_certificate);
    init(*without);
    init(*wrong_ca);

    pair := connect_pair(*server_config, *with_certificate);

    expect_connected(pair);

    pass(pair.client.stream, pair.server.stream, "who am I");

    close_pair(*pair);

    // The server refuses a client with no certificate (in TLS 1.3 the client only learns on its
    // next read, so look at the server).
    pair = connect_pair(*server_config, *without);

    assert(pair.server.error.kind != .NONE, "the server accepted a client with no certificate");
    assert(pair.server.stream == null);

    close_pair(*pair);

    pair = connect_pair(*server_config, *wrong_ca);

    assert(pair.server.error.kind == .VERIFY, "untrusted client: %", Tls.error_text(pair.server.error));

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*with_certificate);
    Tls.tls_config_free(*without);
    Tls.tls_config_free(*wrong_ca);
}

Seen :: struct {
    calls: int;
    certificates: int;
    name: string;
    accept: bool;
}

record_chain :: (chain: [][] u8, name: string, data: *void) -> string {
    seen := cast(*Seen) data;

    seen.calls += 1;
    seen.certificates = chain.count;
    seen.name = name;

    for chain assert(it.count > 100 && it[0] == 0x30, "not DER");

    return ifx seen.accept then "" else "this test says no";
}

test_verify_proc :: () {
    server_config := make_config("server", "");

    init(*server_config);

    seen: Seen;

    // The procedure replaces the library's check: the test CA is not given to the config.
    client_config := make_config("", "");

    client_config.verify_proc = record_chain;
    client_config.verify_data = *seen;

    init(*client_config);

    seen.accept = true;

    pair := connect_pair(*server_config, *client_config);

    expect_connected(pair);

    assert(seen.calls == 1 && seen.certificates == 1 && seen.name == "localhost", "seen: % % '%'", seen.calls, seen.certificates, seen.name);

    pass(pair.client.stream, pair.server.stream, "accepted");

    close_pair(*pair);

    seen.accept = false;

    pair = connect_pair(*server_config, *client_config);

    assert(pair.client.error.kind == .VERIFY, "%", Tls.error_text(pair.client.error));
    assert(Tls.detail(*pair.client.error) == "this test says no");

    // The text does not point into the error: it survives the error being copied over.
    kept_detail := Tls.detail(*pair.client.error);
    kept_text := Tls.error_text(pair.client.error);

    pair.client.error = .{};
    assert(kept_detail == "this test says no" && Tls.error_text(pair.client.error) == "no error", "'%'", kept_detail);
    assert(kept_text == "certificate verification failed: this test says no", "'%'", kept_text);
    assert(pair.client.stream == null);
    assert(seen.calls == 2);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

// ---------------------------------------------------------------------------------------------
// Reading and waiting

test_wait_readable :: () {
    server_config := make_config("server", "");
    client_config := make_config("", "ca");

    init(*server_config);
    init(*client_config);

    pair := connect_pair(*server_config, *client_config);

    expect_connected(pair);

    reader := Tls.reader_of(pair.client.stream);

    assert(Io.can_wait_readable(reader), "a TCP-backed stream can wait");

    // Nothing arrives: the wait ends at its deadline and the stream is fine afterwards.
    started := now_ms();
    error := Io.Io_Error.NONE;

    Io.with_deadline(60, #code {
        error = Io.wait_readable(reader);
    });

    assert(error == .TIMED_OUT, "idle wait: %", error);
    assert(now_ms() - started >= 50);

    pass(pair.server.stream, pair.client.stream, "after the timeout");

    // Bytes the library holds already count as readable: send ten, read four, the rest is there.
    Tls.write(pair.server.stream, "0123456789");

    assert(Io.wait_readable(reader) == .NONE);

    buffer: [4] u8;
    count, read_error := Io.read(reader, buffer);

    assert(read_error == .NONE && count == 4);

    Io.with_deadline(0, #code {
        error = Io.wait_readable(reader);
    });

    assert(error == .NONE, "decrypted bytes are readable even past a deadline: %", error);

    // Six bytes are decrypted and waiting: that is readable without any new traffic.
    count, read_error = Io.read(reader, buffer);

    assert(read_error == .NONE && count == 4);
    assert(Io.wait_readable(reader) == .NONE);

    count, read_error = Io.read(reader, buffer);

    assert(read_error == .NONE && count == 2 && string.{ 2, buffer.data } == "89");

    // The peer ends the stream: that is readable too, and the read says it is the end.
    Tls.close(pair.server.stream);

    pair.server.stream = null;

    assert(Io.wait_readable(reader) == .NONE);

    count, read_error = Io.read(reader, buffer);

    assert(read_error == .NONE && count == 0);

    close_pair(*pair);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

Echo :: struct {
    stream: *Tls.Tls_Stream;
    delay: s64;
    done: bool;
}

// Reads one message, waits, answers.
slow_echo :: (job: *Echo) {
    buffer: [64] u8;
    count, error := Tls.read(job.stream, buffer);

    assert(error == .NONE && count > 0);

    Io.sleep(job.delay);

    Tls.write(job.stream, .{ count, buffer.data });

    job.done = true;
}

Reading :: struct {
    stream: *Tls.Tls_Stream;
    text: string;
    error: Io.Io_Error;
    done: bool;
}

read_one :: (job: *Reading) {
    buffer: [64] u8;
    count, error := Tls.read(job.stream, buffer);

    job.error = error;
    job.text = copy_string(string.{ count, buffer.data });
    job.done = true;
}

// One fiber waits to read while another writes on the same stream.
test_reading_and_writing_at_once :: () {
    server_config := make_config("server", "");
    client_config := make_config("", "ca");

    init(*server_config);
    init(*client_config);

    pair := connect_pair(*server_config, *client_config);

    expect_connected(pair);

    echo := Echo.{ stream = pair.server.stream, delay = 20 };

    Io.spawn(slow_echo, *echo);

    // The server answers a message with the same message, so this reader waits for the answer to
    // the writer's message.
    reading := Reading.{ stream = pair.client.stream };

    Io.spawn(read_one, *reading);
    Io.sleep(30);

    assert(!reading.done);

    count, error := Tls.write(pair.client.stream, "duplex");

    assert(error == .NONE && count == 6, "write while a read waits: % %", count, error);

    while !reading.done Io.sleep(1);

    assert(reading.error == .NONE && reading.text == "duplex", "read: % '%'", reading.error, reading.text);

    free(reading.text.data);

    while !echo.done Io.sleep(1);

    close_pair(*pair);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

// Closing a stream under a fiber that is waiting on it wakes the fiber, and nothing is used after.
test_close_under_a_reader :: () {
    server_config := make_config("server", "");
    client_config := make_config("", "ca");

    init(*server_config);
    init(*client_config);

    pair := connect_pair(*server_config, *client_config);

    expect_connected(pair);

    reading := Reading.{ stream = pair.client.stream };

    Io.spawn(read_one, *reading);
    Io.sleep(20);

    assert(!reading.done);

    Tls.close(pair.client.stream);

    pair.client.stream = null;

    while !reading.done Io.sleep(1);

    assert(reading.error == .CLOSED, "the waiting read: %", reading.error);

    free(reading.text.data);

    // The server sees the client's close_notify, then the end.
    buffer: [8] u8;
    count, read_error := Tls.read(pair.server.stream, buffer);

    assert(read_error == .NONE && count == 0, "server: % %", count, read_error);

    close_pair(*pair);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

Flooding :: struct {
    stream: *Tls.Tls_Stream;
    error: Io.Io_Error;
    done: bool;
}

// Writes until something stops it.
flood :: (job: *Flooding) {
    block: [8192] u8;

    while true {
        _, error := Tls.write(job.stream, block);

        if error != .NONE {
            job.error = error;

            break;
        }
    }

    job.done = true;
}

// A peer that does not read cannot hold up `close`: it is what ends the writer that is stuck.
test_close_with_a_peer_that_does_not_read :: () {
    server_config := make_config("server", "");
    client_config := make_config("", "ca");

    init(*server_config);
    init(*client_config);

    pair := connect_pair(*server_config, *client_config);

    expect_connected(pair);

    job := Flooding.{ stream = pair.client.stream };

    Io.spawn(flood, *job);

    // The socket buffers fill up, and the writer waits inside a send.
    Io.sleep(300);

    assert(!job.done, "the writer should be stuck");

    started := now_ms();

    Tls.close(pair.client.stream);

    pair.client.stream = null;

    elapsed := now_ms() - started;

    assert(elapsed < 900, "close took % ms with a peer that does not read", elapsed);

    while !job.done Io.sleep(1);

    assert(job.error == .CLOSED, "the stuck writer: %", job.error);

    close_pair(*pair);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

// ---------------------------------------------------------------------------------------------
// Many connections

Crowd :: struct {
    listener: Io.Tcp_Listener;
    server_config: *Tls.Tls_Config;
    client_config: *Tls.Tls_Config;
    address: Io.Address;
    count: int;
    served: int;
    failures: int;
    finished: int;
}

crowd_serve :: (crowd: *Crowd) {
    for 1..crowd.count {
        tcp, error := Io.accept(crowd.listener);

        assert(error == .NONE);

        Io.spawn((job: *Crowd_Job) {
            stream, tls_error := Tls.tls_accept(job.crowd.server_config, Io.reader_of(job.tcp), Io.writer_of(job.tcp));

            if tls_error.kind != .NONE {
                job.crowd.failures += 1;
                Io.close(job.tcp);
                free(job);

                return;
            }

            buffer: [2048] u8;
            count, read_error := Io.read_full(Tls.reader_of(stream), .{ 1500, buffer.data });

            if read_error != .NONE || count != 1500 job.crowd.failures += 1;
            else Tls.write(stream, .{ count, buffer.data });

            Tls.close(stream);

            job.crowd.served += 1;

            free(job);
        }, New_Job(crowd, tcp));
    }
}

Crowd_Job :: struct {
    crowd: *Crowd;
    tcp: Io.Tcp_Stream;
}

New_Job :: (crowd: *Crowd, tcp: Io.Tcp_Stream) -> *Crowd_Job {
    job := New(Crowd_Job);

    job.crowd = crowd;
    job.tcp = tcp;

    return job;
}

crowd_client :: (crowd: *Crowd) {
    tcp, error := Io.connect(crowd.address);

    assert(error == .NONE, "connect: %", error);

    stream, tls_error := Tls.tls_connect(crowd.client_config, Io.reader_of(tcp), Io.writer_of(tcp), "localhost");

    if tls_error.kind != .NONE {
        crowd.failures += 1;
        Io.close(tcp);
        crowd.finished += 1;

        return;
    }

    message: [1500] u8;

    for *message it.* = pattern(it_index);

    Tls.write(stream, message);

    reply: [1500] u8;
    count, read_error := Io.read_full(Tls.reader_of(stream), reply);

    if read_error != .NONE || count != 1500 || reply != message crowd.failures += 1;

    Tls.close(stream);

    crowd.finished += 1;
}

test_many_connections :: () {
    server_config := make_config("server", "");
    client_config := make_config("", "ca");

    init(*server_config);
    init(*client_config);

    crowd: Crowd;
    error: Io.Io_Error;

    crowd.server_config = *server_config;
    crowd.client_config = *client_config;
    crowd.count = 60;

    crowd.listener, error = Io.tcp_listen(LOOPBACK);
    crowd.address, _ = Io.local_address(crowd.listener);

    Io.spawn(crowd_serve, *crowd);

    for 1..crowd.count Io.spawn(crowd_client, *crowd);

    while crowd.finished < crowd.count || crowd.served + crowd.failures < crowd.count Io.sleep(2);

    assert(crowd.failures == 0, "% connections failed", crowd.failures);

    Io.close(crowd.listener);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

// ---------------------------------------------------------------------------------------------
// Over a pipe, which cannot say when it is readable

Piped :: struct {
    config: *Tls.Tls_Config;
    end: *Io.Pipe_Stream;
    stream: *Tls.Tls_Stream;
    error: Tls.Tls_Error;
    done: bool;
}

pipe_accept :: (job: *Piped) {
    job.stream, job.error = Tls.tls_accept(job.config, Io.reader_of(job.end), Io.writer_of(job.end));
    job.done = true;
}

test_over_a_pipe :: () {
    server_config := make_config("server", "");
    client_config := make_config("", "ca");

    init(*server_config);
    init(*client_config);

    client_end, server_end := Io.make_pipe();

    job := Piped.{ config = *server_config, end = server_end };

    Io.spawn(pipe_accept, *job);

    client, error := Tls.tls_connect(*client_config, Io.reader_of(client_end), Io.writer_of(client_end), "localhost");

    assert(error.kind == .NONE, "client: %", Tls.error_text(error));

    while !job.done Io.sleep(1);

    assert(job.error.kind == .NONE, "server: %", Tls.error_text(job.error));
    assert(!Io.can_wait_readable(Tls.reader_of(client)), "a pipe has no wait");

    pass(client, job.stream, "through a pipe");
    pass(job.stream, client, "and back");

    Tls.close(client);
    Tls.close(job.stream);

    Tls.tls_config_free(*server_config);
    Tls.tls_config_free(*client_config);
}

// ---------------------------------------------------------------------------------------------
// Configs

test_config_errors :: () {
    garbage := make_config("", "");

    garbage.roots_pem = "-----BEGIN CERTIFICATE-----\nnot a certificate\n-----END CERTIFICATE-----\n";

    error := Tls.tls_config_init(*garbage);

    assert(error.kind == .CONFIG, "garbage roots: %", Tls.error_text(error));

    config := make_config("server", "");

    config.private_key_pem = "";

    error = Tls.tls_config_init(*config);

    assert(error.kind == .CONFIG, "a certificate without a key: %", Tls.error_text(error));

    config = make_config("server", "");
    config.private_key_pem = "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\n";

    error = Tls.tls_config_init(*config);

    assert(error.kind == .CONFIG, "garbage key: %", Tls.error_text(error));

    // A key that belongs to another certificate is not caught by the parsers, and the handshake
    // fails instead: the config is accepted.
    config = make_config("server", "", min = .TLS_1_3, max = .TLS_1_2);

    error = Tls.tls_config_init(*config);

    assert(error.kind == .CONFIG, "empty version range: %", Tls.error_text(error));

    config = make_config("", "ca");
    config.alpn = string.[""];

    error = Tls.tls_config_init(*config);

    assert(error.kind == .CONFIG, "an empty ALPN name: %", Tls.error_text(error));

    // A client-only config cannot accept, and a config that was never initialised cannot connect.
    client := make_config("", "ca");

    init(*client);

    a, b := Io.make_pipe();

    stream, accept_error := Tls.tls_accept(*client, Io.reader_of(a), Io.writer_of(a));

    assert(stream == null && accept_error.kind == .CONFIG, "accept without a certificate: %", Tls.error_text(accept_error));

    stream, accept_error = Tls.tls_connect(*client, Io.reader_of(a), Io.writer_of(a), "");

    assert(stream == null && accept_error.kind == .CONFIG, "connect without a name: %", Tls.error_text(accept_error));

    fresh: Tls.Tls_Config;

    stream, accept_error = Tls.tls_connect(*fresh, Io.reader_of(a), Io.writer_of(a), "x");

    assert(stream == null && accept_error.kind == .CONFIG, "uninitialised: %", Tls.error_text(accept_error));

    Io.close(a);
    Io.close(b);

    Tls.tls_config_free(*client);
}

// Names each group as it starts, so a run that hangs says where (a timeout keeps the output).
progress :: (name: string) {
    print("  %\n", name);
}

run_all :: () {
    progress("test_handshakes");
    test_handshakes();
    progress("test_version_mismatch");
    test_version_mismatch();
    progress("test_alpn");
    test_alpn();
    progress("test_large_transfers");
    test_large_transfers();
    progress("test_awkward_sizes");
    test_awkward_sizes();
    progress("test_duplex_with_small_socket_buffers");
    test_duplex_with_small_socket_buffers();
    progress("test_close_notify");
    test_close_notify();
    progress("test_cut_in_the_middle_of_a_record");
    test_cut_in_the_middle_of_a_record();
    progress("test_handshake_deadline");
    test_handshake_deadline();
    progress("test_handshake_cancel");
    test_handshake_cancel();
    progress("test_verification_failures");
    test_verification_failures();
    progress("test_verification_off");
    test_verification_off();
    progress("test_client_certificates");
    test_client_certificates();
    progress("test_verify_proc");
    test_verify_proc();
    progress("test_wait_readable");
    test_wait_readable();
    progress("test_reading_and_writing_at_once");
    test_reading_and_writing_at_once();
    progress("test_close_under_a_reader");
    test_close_under_a_reader();
    progress("test_close_with_a_peer_that_does_not_read");
    test_close_with_a_peer_that_does_not_read();
    progress("test_many_connections");
    test_many_connections();
    progress("test_over_a_pipe");
    test_over_a_pipe();
    progress("test_config_errors");
    test_config_errors();
}

// Runs `group` on a scheduler of its own.
run_group :: (group: () -> ()) {
    Io.io_run(group);
}
