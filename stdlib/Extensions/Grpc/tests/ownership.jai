// test: skip_on(.WASM, reason = "native RPC allocations")
#import "Basic"()(MEMORY_DEBUGGER = true, VISUALIZE_MEMORY_DEBUGGER = false);
H :: #import "Extensions/Http";
G :: #import "Extensions/Grpc";
Io :: #import "Extensions/Io";
Tls :: #import "Extensions/Tls";
#load "../../Tls/tests/fixtures/certs.jai";

METHOD :: G.Method.{ path = "/fixture.Echo/Owned", client_streaming = true, server_streaming = true };

handle :: (call: *G.Call, _data: *void) -> G.Status {
    while true {
        bytes, kind := G.receive_bytes(call);
        if kind == .END break;
        if kind != .MESSAGE return .{ code = .INTERNAL };

        defer if bytes.data free(bytes.data);
        sent := G.send_bytes(call, bytes);
        if sent.code != .OK return sent;
    }

    status: G.Status;

    array_add(*status.metadata, .{ "x-owned", "retained after close" },, temp);

    return status;
}

route :: (request: *H.Request) -> H.Response {
    return G.serve_call(request, METHOD, handle, null);
}

app: H.App;
client_config: Tls.Tls_Config;

main :: () {
    server_config: Tls.Tls_Config;
    server_config.certificate_chain_pem = SERVER_PEM;
    server_config.private_key_pem = SERVER_KEY;
    server_config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*server_config).kind == .NONE);
    defer Tls.tls_config_free(*server_config);
    client_config.roots_pem = CA_PEM;
    client_config.alpn = string.["h2"];
    assert(Tls.tls_config_init(*client_config).kind == .NONE);
    defer Tls.tls_config_free(*client_config);
    app.config.tls = *server_config;
    app.config.write_timeout = 0;
    app.config.address = "127.0.0.1:0";
    app.config.handle_signals = false;
    H.post(*app, METHOD.path, route);
    defer H.free_app(*app);
    allocations_before_this_call_will_not_be_considered_leaks();

    entry :: (_data: *void) {
        server, error := H.start(*app);

        assert(error.kind == .NONE);
        channel: G.Channel;
        channel.transport.base_url = H.server_url(server);
        channel.transport.tls = *client_config;

        for 0..3 {
            call, opened := G.open_call(*channel, METHOD);

            assert(opened.code == .OK);
            sent := G.send_bytes(*call, u8.[1, 2, 3]);

            assert(sent.code == .OK);
            half := G.close_send(*call);

            assert(half.code == .OK);
            bytes, kind := G.receive_bytes(*call);

            assert(kind == .MESSAGE);
            final := G.finish(*call);

            assert(final.code == .OK);
            G.close(*call);
            assert(bytes.count == 3 && bytes[2] == 3);
            assert(final.metadata.count == 1 && final.metadata[0].value == "retained after close");
            free(bytes.data);
            G.free_status(*final);
        }

        {
            call, opened := G.open_call(*channel, METHOD);

            assert(opened.code == .OK);
            G.cancel(*call);
            G.cancel(*call);
            G.close(*call);
        }

        {
            expired, status := G.open_call(*channel, METHOD, .{ deadline_ms = to_milliseconds(current_time_monotonic()) - 1 });
            assert(!expired.state && status.code == .DEADLINE_EXCEEDED);
            G.free_status(*status);
        }

        assert(channel.active_calls == 0 && channel.transport.active_exchanges == 0);
        G.close_channel(*channel);
        H.stop(server);
        H.wait(server);
        H.close_server(*server);
        H.close_server(*server);
    }

    Io.io_run(entry, null);
    report := make_leak_report(.{ scan_global_memory_to_remove_leaks = false, scan_leaks_to_combine = false });
    count: s64;
    bytes: s64;

    for report.sorted_summaries {
        count += it.count;
        bytes += it.bytes;
        print("allocation count % bytes %\n", it.count, it.bytes);

        if it.alloc_site_trace for node: it.alloc_site_trace.nodes {
            if node.info print("%:% %\n", node.info.location.fully_pathed_filename, node.line_number, node.info.name);
        }
    }

    print("call cleanup leaks % bytes %\n", count, bytes);
    deinit(*report);
    assert(count == 0 && bytes == 0);
    print("ok\n");
}
