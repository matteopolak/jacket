// A client for the protocol: connect, send a request line, read the reply lines.

Client :: struct {
    stream: Io.Tcp_Stream;
    reader: Line_Reader;
}

// Connects to `address` such as "127.0.0.1:7878". Call it from a fiber.
connect_client :: (address: string) -> Client, Io.Io_Error {
    client: Client;
    error: Io.Io_Error;

    client.stream, error = Io.connect(address);
    client.reader.stream = client.stream;

    return client, error;
}

close_client :: (client: *Client) {
    Io.close(client.stream);
    free_line_reader(*client.reader);
}

// Sends one request and returns its reply: one line, or for SCAN every row up to END. The lines
// are copies; release them with `free_reply`. Any transport error comes back with the lines that
// arrived before it.
call :: (client: *Client, request: string) -> [] string, Io.Io_Error {
    lines: [..] string;

    error := write_line(client.stream, request);

    if error != .NONE return lines, error;

    _, word, _ := split_from_left(request, #char " ");
    multiple := equal_nocase(word, "SCAN");

    while true {
        line, read_error := read_line(*client.reader);

        if read_error != .NONE return lines, read_error;

        array_add(*lines, copy_string(line));

        if !multiple || line == "END" || begins_with(line, "ERR") break;
    }

    return lines, .NONE;
}

free_reply :: (lines: [] string) {
    for lines free(it);

    free(lines.data);
}
