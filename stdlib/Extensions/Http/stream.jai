// The byte stream under a connection: a TCP socket or one end of an in-memory pipe. The server's
// connection loop and the client's exchange read and write through these procedures, so the same
// code serves a real socket and the test client's memory connection (see testing.jai).

Stream_Kind :: enum u8 {
    NONE;
    TCP;
    PIPE;
}

Stream :: struct {
    kind: Stream_Kind;
    tcp: Io.Tcp_Stream;
    pipe: *Io.Pipe_Stream;
}

tcp_stream :: (tcp: Io.Tcp_Stream) -> Stream {
    stream: Stream;

    stream.kind = .TCP;
    stream.tcp = tcp;

    return stream;
}

pipe_stream :: (pipe: *Io.Pipe_Stream) -> Stream {
    stream: Stream;

    stream.kind = .PIPE;
    stream.pipe = pipe;

    return stream;
}

// Waits for bytes and reads what is there; 0 means the other side closed.
stream_read :: (stream: Stream, buffer: [] u8) -> s64, Io.Io_Error {
    if stream.kind == .PIPE return Io.read(stream.pipe, buffer);
    if stream.kind == .TCP return Io.read(stream.tcp, buffer);

    return 0, .INVALID;
}

// Sends all of `data`.
stream_write :: (stream: Stream, data: [] u8) -> s64, Io.Io_Error {
    if stream.kind == .PIPE return Io.write(stream.pipe, data);
    if stream.kind == .TCP return Io.write(stream.tcp, data);

    return 0, .INVALID;
}

stream_close :: (stream: Stream) {
    if stream.kind == .PIPE Io.close(stream.pipe);
    else if stream.kind == .TCP Io.close(stream.tcp);
}

// Closes without waiting, for code that cannot block (a scheduler-local drop).
stream_close_now :: (stream: Stream) {
    if stream.kind == .PIPE Io.close(stream.pipe);
    else if stream.kind == .TCP Io.close_now(stream.tcp);
}

// The other side's address. A pipe has none: it reports the loopback address with port 0.
stream_peer :: (stream: Stream) -> Io.Address {
    if stream.kind == .TCP {
        address, _ := Io.peer_address(stream.tcp);

        return address;
    }

    return Io.address_v4(127, 0, 0, 1, 0);
}
