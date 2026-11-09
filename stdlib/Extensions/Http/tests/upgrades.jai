// test: skip_on(.WASM, reason = "upgrades require fibers")
#import "Basic";
#import "Extensions/Http";
Io :: #import "Extensions/Io";

handoff :: (link: Link, pending: [] u8, data: *void) {
    if pending.count Io.write(link.writer, pending);

    Io.write(link.writer, "ready");
    buffer: [32] u8;
    count, error := Io.read(link.reader, buffer);

    assert(error == .NONE);
    Io.write(link.writer, string.{ count, buffer.data });
}

route :: (request: *Request) -> Response {
    return upgrade_response("example", handoff);
}

main :: () {
    value, found := find_header(Header_Field.[.{ name = "Upgrade", value = "example" }], "Upgrade");

    assert(found && value == "example");
    Io.io_run(() {
        bogus: Client;
        bogus.transport.open = fake_open;
        invalid, invalid_link, extra, failure := client_upgrade(*bogus, "http://localhost/", .[
            .{ name = "Connection", value = "Upgrade" }, .{ name = "Upgrade", value = "example" }
        ]);

        assert(failure.kind != .NONE && !invalid_link.reader.read_proc && extra.count == 0);
        close(*invalid);
        free_error(*failure);
        app: App;

        get(*app, "/", route);
        api := test_client(*app);
        response, link, pending, error := client_upgrade(api, "http://localhost/", .[
            .{ name = "Connection", value = "Upgrade" }, .{ name = "Upgrade", value = "example" }
        ]);

        assert(error.kind == .NONE && response.status == 101, "%", error_text(error));
        defer close(*response);
        defer close(link);
        defer free(pending.data);
        Io.write(link.writer, "echo");
        builder: String_Builder;

        append(*builder, cast(string) pending);
        buffer: [32] u8;
        total := pending.count;

        while total < 9 {
            count, err := Io.read(link.reader, buffer);
            assert(err == .NONE && count > 0);
            append(*builder, string.{ count, buffer.data });
            total += count;
        }

        result := builder_to_string(*builder);

        assert(result == "readyecho");
        free(result);
        free(*builder);
    });
}

fake_switch :: (pipe: *Io.Pipe_Stream) {
    defer Io.close(pipe);
    buffer: [4096] u8;
    Io.read(pipe, buffer);
    Io.write(pipe, "HTTP/1.1 101 Switching Protocols\r\n\r\n");
}

fake_open :: (data: *void, url: Url) -> Link, Io.Io_Error {
    near, far := Io.make_pipe();
    Io.spawn(fake_switch, far);
    return pipe_link(near), .NONE;
}
