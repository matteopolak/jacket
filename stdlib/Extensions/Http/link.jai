// The two halves of a connection, as the server's connection loop and the client's exchange see
// it: an `Io.Reader` and an `Io.Writer` over one TCP socket or one end of an in-memory pipe (see
// test_client.jai), and the address of the other side. All the reading and writing goes through
// `Io.read` and `Io.write`, so a transport of your own only has to produce a `Link`.

// The connection as the server loop and the client exchange see it: the `reader` and `writer` halves and the address of the
// `peer`. Both halves share one socket or pipe end, so `close` on the link (or on either half) ends the connection once.
// A transport of your own only has to produce a `Link`.
//
// ```jai,skip_on(wasm)
// Io :: #import "Extensions/Io";
//
// main :: () {
//     Io.io_run(() {
//         near, far := Io.make_pipe();
//         link := pipe_link(near);
//         other := pipe_link(far);
//
//         assert(Io.address_to_string(link.peer) == "127.0.0.1:0"); // a pipe has no peer: loopback, port 0
//         assert(!link.h2);
//
//         close(link);
//         close(other);
//     });
// }
// ```
Link :: struct {
    reader: Io.Reader;
    writer: Io.Writer;
    // The other side's address. A pipe has none: it reports the loopback address with port 0.
    peer: Io.Address;
    // True when TLS agreed on HTTP/2 (ALPN `h2`) for this connection.
    h2: bool;
}

// A `Link` over an accepted or connected TCP stream. `peer` is the stream's remote address, or empty if it cannot be read.
// The link does not copy the stream: close the link (not the stream as well) when done.
//
// ```jai,skip_on(wasm)
// Io :: #import "Extensions/Io";
//
// main :: () {
//     Io.io_run(() {
//         listener, _ := Io.tcp_listen("127.0.0.1:0");
//         defer Io.close(listener);
//
//         address, _ := Io.local_address(listener);
//         client_stream, _ := Io.connect(address);
//         server_stream, _ := Io.accept(listener);
//
//         client := tcp_link(client_stream);
//         server := tcp_link(server_stream);
//         assert(Io.write(client.writer, "ping") == 4);
//
//         buffer: [8] u8;
//         count, error := Io.read(server.reader, buffer);
//         assert(error == .NONE && string.{ count, buffer.data } == "ping");
//
//         close(client); // the link, not the stream as well
//         close(server);
//     });
// }
// ```
tcp_link :: (tcp: Io.Tcp_Stream) -> Link {
    link: Link;

    link.reader = Io.reader_of(tcp);
    link.writer = Io.writer_of(tcp);
    link.peer, _ = Io.peer_address(tcp);

    return link;
}

// A `Link` over one end of an in-memory pipe from `Io.make_pipe`; the test client connects an `App` this way. `peer` is the
// loopback address with port 0.
//
// ```jai,skip_on(wasm)
// Io :: #import "Extensions/Io";
//
// main :: () {
//     Io.io_run(() {
//         near, far := Io.make_pipe();
//         client := pipe_link(near);
//         server := pipe_link(far);
//
//         Io.write(client.writer, "ping");
//
//         buffer: [8] u8;
//         count, error := Io.read(server.reader, buffer);
//         assert(error == .NONE && string.{ count, buffer.data } == "ping");
//
//         close(client);
//         close(server);
//     });
// }
// ```
pipe_link :: (pipe: *Io.Pipe_Stream) -> Link {
    link: Link;

    link.reader = Io.reader_of(pipe);
    link.writer = Io.writer_of(pipe);
    link.peer = Io.address_v4(127, 0, 0, 1, 0);

    return link;
}

// Closes the connection: both halves share one socket or pipe end, so closing the reader closes
// it once. With `now` it does not wait (see `Io.close`).
//
// ```jai,skip_on(wasm)
// Io :: #import "Extensions/Io";
//
// main :: () {
//     Io.io_run(() {
//         near, far := Io.make_pipe();
//         link := pipe_link(near);
//         other := pipe_link(far);
//
//         assert(close(link) == .NONE);
//
//         buffer: [8] u8;
//         count, _ := Io.read(other.reader, buffer); // the other end sees the end of the stream
//         assert(count == 0);
//
//         close(other);
//     });
// }
// ```
close :: (link: Link, now := false) -> Io.Io_Error {
    return Io.close(link.reader, now);
} @FreesArgument

// A `with_deadline` argument for a timeout setting in milliseconds where 0 or less means no limit.
//
// ```jai,skip_on(wasm)
// Io :: #import "Extensions/Io";
//
// assert(within(250) == 250);
// assert(within(0) == Io.NO_DEADLINE); // 0 or less: no limit
// assert(within(-1) == Io.NO_DEADLINE);
// ```
within :: (timeout: s64) -> s64 {
    return ifx timeout > 0 then timeout else Io.NO_DEADLINE;
}
