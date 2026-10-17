// The two halves of a connection, as the server's connection loop and the client's exchange see
// it: an `Io.Reader` and an `Io.Writer` over one TCP socket or one end of an in-memory pipe (see
// test_client.jai), and the address of the other side. All the reading and writing goes through
// `Io.read` and `Io.write`, so a transport of your own only has to produce a `Link`.

Link :: struct {
    reader: Io.Reader;
    writer: Io.Writer;
    // The other side's address. A pipe has none: it reports the loopback address with port 0.
    peer: Io.Address;
}

tcp_link :: (tcp: Io.Tcp_Stream) -> Link {
    link: Link;

    link.reader = Io.reader_of(tcp);
    link.writer = Io.writer_of(tcp);
    link.peer, _ = Io.peer_address(tcp);

    return link;
}

pipe_link :: (pipe: *Io.Pipe_Stream) -> Link {
    link: Link;

    link.reader = Io.reader_of(pipe);
    link.writer = Io.writer_of(pipe);
    link.peer = Io.address_v4(127, 0, 0, 1, 0);

    return link;
}

// Closes the connection: both halves share one socket or pipe end, so closing the reader closes
// it once. With `now` it does not wait (see `Io.close`).
close :: (link: Link, now := false) -> Io.Io_Error {
    return Io.close(link.reader, now);
} @Releases

// A `with_deadline` argument for a timeout setting in milliseconds where 0 or less means no limit.
within :: (timeout: s64) -> s64 {
    return ifx timeout > 0 then timeout else Io.NO_DEADLINE;
}
