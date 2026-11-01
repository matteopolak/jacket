// One shape for plaintext and TLS. TLS owns the TCP socket after a successful handshake.
Link :: struct {
    tcp: Io.Tcp_Stream;
    tls: *Tls.Tls_Stream;
    config: Tls.Tls_Config;
    owns_config: bool;
    reader: Io.Line_Reader;
    writer: Io.Writer;
    connected: bool;
}

link_prepare :: (link: *Link) {
    source := Io.reader_of(link.tcp);

    link.writer = Io.writer_of(link.tcp);
    if link.tls {
        source = Tls.reader_of(link.tls);
        link.writer = Tls.writer_of(link.tls);
    }

    link.reader = Io.make_line_reader(source, MAX_LINE);
    link.connected = true;
}

link_connect :: (link: *Link, address: string, roots: string) -> bool {
    tcp, error := Io.connect(address);
    if error != .NONE return false;

    link.tcp = tcp;

    if roots.count > 0 {
        pem, ok := read_entire_file(roots);
        if !ok {
            Io.close(tcp);
            return false;
        }

        defer free(pem);
        link.config = .{ roots_pem = pem };
        failure := Tls.tls_config_init(*link.config);

        if failure.kind != .NONE {
            print("TLS config: %\n", Tls.error_text(failure));
            Io.close(tcp);
            return false;
        }

        stream, tls_error := Tls.tls_connect(*link.config, Io.reader_of(tcp), Io.writer_of(tcp), "localhost");
        if tls_error.kind != .NONE {
            print("TLS: %\n", Tls.error_text(tls_error), to_standard_error = true);
            Tls.tls_config_free(*link.config);
            Io.close(tcp);
            return false;
        }

        link.tls = stream;
        link.owns_config = true;
    }

    link_prepare(link);

    return true;
}

link_close :: (link: *Link) {
    if !link.connected return;
    if link.tls Tls.close(link.tls);
    else Io.close(link.tcp);

    if link.owns_config Tls.tls_config_free(*link.config);

    Io.free_line_reader(*link.reader);
    link.connected = false;
}

send_line :: (link: *Link, line: string) -> bool {
    _, error := Io.write(link.writer, tprint("%\n", line));
    return error == .NONE;
} @TempAllocates
