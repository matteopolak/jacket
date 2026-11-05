// test: skip_on(.WASM, reason = "native RPC scopes")
#load "../../Http/tests/support/server_support.jai";
Tls :: #import "Extensions/Tls";
G :: #import "Extensions/Grpc";
#load "../../Tls/tests/fixtures/certs.jai";

METHOD :: G.Method.{ path = "/fixture.Echo/Scope", client_streaming = true, server_streaming = true };

Probe :: struct {
    changed: Io.Condition;
    entered: bool;
    released: bool;
    shielded: bool;
    early: bool;
    error: Io.Io_Error;
    done: Io.Wait_Group;
    wait_returned: bool;
}

probe: Probe;

blocked :: (_call: *G.Call, _data: *void) -> G.Status {
    probe.entered = true;
    Io.notify_all(*probe.changed);
    defer Io.done(*probe.done);

    if probe.early {
        while !probe.released assert(Io.wait(*probe.changed) == .NONE);
        return .{ code = .INVALID_ARGUMENT };
    }

    if probe.shielded {
        Io.with_grace(Io.NO_DEADLINE, #code {
            while !probe.released assert(Io.wait(*probe.changed) == .NONE);
        });
    } else probe.error = Io.wait(*probe.changed);

    return .{ code = ifx probe.error == .TIMED_OUT then G.Status_Code.DEADLINE_EXCEEDED else G.Status_Code.CANCELLED };
}

route :: (request: *Request) -> Response {
    return G.serve_call(request, METHOD, blocked, null);
}

Early_Send :: struct {
    call: *G.Call;
    done: Io.Wait_Group;
    returned: bool;
}

early_send :: (job: *Early_Send) {
    defer Io.done(*job.done);
    bytes := NewArray(600 * 1024, u8);

    defer free(bytes.data);
    status := G.send_bytes(job.call, bytes);

    defer G.free_status(*status);
    assert(status.code != .OK);
    job.returned = true;
}

wait_server :: (server: *Server) {
    wait(server);
    probe.wait_returned = true;
}

exercise :: (base: string, server: *Server) {
    config: Tls.Tls_Config;
    config.roots_pem = CA_PEM;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    channel: Http2_Channel;
    channel.base_url = base;
    channel.tls = *config;
    defer close_http2_channel(*channel);

    {
        rpc_channel: G.Channel;
        rpc_channel.transport.base_url = base;
        rpc_channel.transport.tls = *config;
        defer G.close_channel(*rpc_channel);
        probe = .{};
        probe.early = true;
        Io.add(*probe.done, 1);
        call, opened := G.open_call(*rpc_channel, METHOD);

        assert(opened.code == .OK);
        defer G.close(*call);
        while !probe.entered Io.wait(*probe.changed);

        job := Early_Send.{ call = *call };

        Io.add(*job.done, 1);
        Io.spawn(early_send, *job);
        Io.yield();
        assert(!job.returned, "upload remains blocked at the small peer window");
        probe.released = true;
        Io.notify_all(*probe.changed);
        Io.with_deadline(1000, #code {
            assert(Io.wait(*job.done) == .NONE);
        });
        final := G.finish(*call);

        defer G.free_status(*final);
        assert(final.code == .INVALID_ARGUMENT, "an early final status survives stopping a blocked upload");
        assert(job.returned);
    }

    for canceled: bool.[false, true] {
        probe = .{};
        Io.add(*probe.done, 1);
        fields: [..] Header_Field;
        fields.allocator = temp;
        array_add(*fields, .{ name = "content-type", value = "application/grpc+proto" });
        if !canceled array_add(*fields, .{ name = "grpc-timeout", value = "20m" });

        exchange, opened := open_http2_exchange(*channel, METHOD.path, fields);

        assert(opened.kind == .NONE);
        Io.with_deadline(1000, #code {
            // Response fields stay deferred while the handler has not sent.
            // The explicit handler gate establishes the cancellation ordering.
            while !probe.entered assert(Io.wait(*probe.changed) == .NONE);
            if canceled exchange_cancel(*exchange);

            assert(Io.wait(*probe.done) == .NONE, "incoming deadline/reset wakes a handler blocked outside RPC helpers");
            assert(probe.error == ifx canceled then Io.Io_Error.CANCELED else Io.Io_Error.TIMED_OUT);

            if !canceled {
                _, _, head_error := exchange_head(*exchange);

                assert(head_error.kind == .NONE);
                buffer: [1] u8;
                count, read_error := exchange_read(*exchange, buffer);

                assert(count == 0 && read_error.kind == .NONE);
                trailers, complete := exchange_trailers(*exchange);

                assert(complete);
                status := G.parse_status(trailers, 200);

                defer G.free_status(*status);
                assert(status.code == .DEADLINE_EXCEEDED);
            }
        });
        close_exchange(*exchange);
    }

    probe = .{};
    probe.shielded = true;
    Io.add(*probe.done, 1);
    exchange, opened := open_http2_exchange(*channel, METHOD.path, Header_Field.[.{ name = "content-type", value = "application/grpc+proto" }]);

    assert(opened.kind == .NONE);
    // Shutdown must join a handler even before deferred headers are committed.
    while !probe.entered Io.wait(*probe.changed);

    stop(server);
    Io.spawn(wait_server, server);
    // Wait beyond both shutdown grace periods while shielded cleanup stays held.
    assert(Io.sleep(20) == .NONE);
    assert(!probe.wait_returned, "server must retain handler state through shielded cleanup");
    probe.released = true;
    Io.notify_all(*probe.changed);
    wait(server);
    assert(probe.done.count == 0);
    close_exchange(*exchange);
}

main :: () {
    config: Tls.Tls_Config;
    config.certificate_chain_pem = SERVER_PEM;
    config.private_key_pem = SERVER_KEY;
    config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*config).kind == .NONE);
    defer Tls.tls_config_free(*config);
    app: App;
    app.config.tls = *config;
    app.config.write_timeout = 0;
    app.config.shutdown_timeout = 1;
    app.config.h2_window = 16384;
    defer free_app(*app);
    post(*app, METHOD.path, route);
    run_server(*app, exercise);
    print("ok\n");
}
