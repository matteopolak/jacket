// Extensions/Tls against OpenSSL on loopback: `openssl s_client` connecting to our server, and our
// client connecting to `openssl s_server`. Nothing leaves the machine. Skipped (and says so) when
// there is no `openssl` command, or on Windows. Run from this directory.
#load "lib/suite.jai";

Process :: #import "Process";

#if OS == .WINDOWS {
    main :: () {
        print("skipped: the OpenSSL interop tests run on macOS and Linux\n");
    }
} else {

    openssl_present :: () -> bool {
        result, output := Process.run_command("openssl", "version", capture_and_return_output = true);

        // LibreSSL (macOS's /usr/bin/openssl) has other s_server and s_client options and an older TLS.
        return result.type == .EXITED && result.exit_code == 0 && begins_with(output, "OpenSSL 3");
    }

    // A child process whose output is polled, so the scheduler keeps running while it works.
    Child :: struct {
        process: Process.Process;
        output: [..] u8;
        errors: [..] u8;
    }

    start_child :: (child: *Child, args: ..string) {
        ok := Process.create_process(*child.process, ..args, capture_and_return_output = true);

        assert(ok, "could not start %", args[0]);
    }

    // Reads what the child wrote until it ends or `limit` milliseconds pass.
    drain_child :: (child: *Child, limit: s64) -> bool {
        started := now_ms();
        out: [4096] u8;
        err: [4096] u8;

        while !child.process.output.eof || !child.process.error.eof {
            if now_ms() - started > limit return false;

            ok, out_count, err_count := Process.read_from_process(*child.process, out, err, 0);

            if !ok break;

            for 0..out_count - 1 array_add(*child.output, out[it]);
            for 0..err_count - 1 array_add(*child.errors, err[it]);

            if out_count == 0 && err_count == 0 Io.sleep(5);
        }

        return true;
    }

    child_text :: (child: *Child) -> string {
        return string.{ child.output.count, child.output.data };
    }

    stop_child :: (child: *Child) {
        Process.kill_process(*child.process);
        Process.deinit(*child.process);
        array_free(child.output);
        array_free(child.errors);
    }

    Serving :: struct {
        listener: Io.Tcp_Listener;
        config: *Tls.Tls_Config;
        alpn: string;
        name: string;
        version: string;
        suite: string;
        error: Tls.Tls_Error;
        done: bool;
    }

    // What the next `check_server` expects in the name of the cipher suite ("" for anything).
    expected_suite := "";

    // Accepts one connection, reads the request line, answers and closes.
    serve_one :: (state: *Serving) {
        tcp, io_error := Io.accept(state.listener);

        assert(io_error == .NONE);

        stream, error := Tls.tls_accept(state.config, Io.reader_of(tcp), Io.writer_of(tcp));

        state.error = error;

        if error.kind != .NONE {
            Io.close(tcp);
            state.done = true;

            return;
        }

        state.alpn = copy_string(Tls.alpn_protocol(stream));
        state.name = copy_string(Tls.server_name(stream));
        state.version = copy_string(Tls.version(stream));
        state.suite = copy_string(Tls.cipher_suite(stream));

        buffer: [1024] u8;
        count, read_error := Tls.read(stream, buffer);

        assert(read_error == .NONE && count > 0, "request: % %", count, read_error);

        Tls.write(stream, "got it: ");
        Tls.write(stream, string.{ count, buffer.data });
        Tls.close(stream);

        state.done = true;
    }

    // Our server, OpenSSL's client: `extra` are s_client options.
    check_server :: (config: *Tls.Tls_Config, version_flag: string, expected_version: string, extra: ..string) {
        state := Serving.{ config = config };
        error: Io.Io_Error;

        state.listener, error = Io.tcp_listen(LOOPBACK);

        address, _ := Io.local_address(state.listener);

        Io.spawn(serve_one, *state);

        child: Child;

        args: [..] string;

        array_add(*args, "openssl", "s_client", "-quiet", "-connect", tprint("127.0.0.1:%", address.port), "-CAfile", "interop-ca.pem", "-verify_return_error", "-servername", "localhost", "-verify_hostname", "localhost");

        if version_flag.count array_add(*args, version_flag);

        for extra array_add(*args, it);

        start_child(*child, ..args);

        request := "hello openssl\n";

        Process.write_to_process(*child.process, .{ request.count, request.data });

        // The server answers and closes; s_client then ends on its own.
        finished := drain_child(*child, 10_000);

        errors := string.{ child.errors.count, child.errors.data };

        assert(finished, "s_client did not finish: %", errors);
        assert(has(child_text(*child), "got it: hello openssl"), "s_client printed '%' (errors '%')", child_text(*child), errors);

        while !state.done Io.sleep(1);

        assert(state.error.kind == .NONE, "%", Tls.error_text(state.error));
        assert(state.version == expected_version, "version %", state.version);
        assert(state.name == "localhost", "sni '%'", state.name);
        assert(expected_suite.count == 0 || has(state.suite, expected_suite), "cipher suite '%', expected '%'", state.suite, expected_suite);

        expected_suite = "";

        // The alpn the caller asked for, if any.
        alpn_wanted := "";

        for extra if it == "-alpn" && it_index + 1 < extra.count alpn_wanted = extra[it_index + 1];

        assert(state.alpn == alpn_wanted, "alpn '%'", state.alpn);

        free(state.alpn.data);
        free(state.name.data);
        free(state.version.data);
        free(state.suite.data);

        stop_child(*child);
        array_free(args);
        Io.close(state.listener);
    }

    test_openssl_client :: () {
        server_config := make_config("server", "", string.["h2", "http/1.1"]);

        init(*server_config);

        check_server(*server_config, "", "TLSv1.3");
        check_server(*server_config, "-tls1_2", "TLSv1.2");
        check_server(*server_config, "", "TLSv1.3", "-alpn", "http/1.1");

        rsa_config := make_config("rsa", "");

        init(*rsa_config);

        check_server(*rsa_config, "", "TLSv1.3");
        check_server(*rsa_config, "-tls1_2", "TLSv1.2");

        // A client that offers one cipher gets it, whichever the server would prefer.
        expected_suite = "AES-128-GCM";
        check_server(*server_config, "", "TLSv1.3", "-ciphersuites", "TLS_AES_128_GCM_SHA256");
        expected_suite = "AES-256-GCM";
        check_server(*server_config, "", "TLSv1.3", "-ciphersuites", "TLS_AES_256_GCM_SHA384");
        expected_suite = "CHACHA20-POLY1305";
        check_server(*server_config, "", "TLSv1.3", "-ciphersuites", "TLS_CHACHA20_POLY1305_SHA256");
        expected_suite = "ECDSA-WITH-AES-128-GCM";
        check_server(*server_config, "-tls1_2", "TLSv1.2", "-cipher", "ECDHE-ECDSA-AES128-GCM-SHA256");
        expected_suite = "ECDSA-WITH-AES-256-GCM";
        check_server(*server_config, "-tls1_2", "TLSv1.2", "-cipher", "ECDHE-ECDSA-AES256-GCM-SHA384");
        expected_suite = "ECDSA-WITH-CHACHA20-POLY1305";
        check_server(*server_config, "-tls1_2", "TLSv1.2", "-cipher", "ECDHE-ECDSA-CHACHA20-POLY1305");
        expected_suite = "RSA-WITH-AES-128-GCM";
        check_server(*rsa_config, "-tls1_2", "TLSv1.2", "-cipher", "ECDHE-RSA-AES128-GCM-SHA256");
        expected_suite = "RSA-WITH-CHACHA20-POLY1305";
        check_server(*rsa_config, "-tls1_2", "TLSv1.2", "-cipher", "ECDHE-RSA-CHACHA20-POLY1305");

        // OpenSSL presents a client certificate that our server checks.
        mutual := make_config("server", "ca");

        mutual.client_auth = true;

        init(*mutual);

        check_server(*mutual, "", "TLSv1.3", "-cert", "interop-client.pem", "-key", "interop-client.key");
        check_server(*mutual, "-tls1_2", "TLSv1.2", "-cert", "interop-client.pem", "-key", "interop-client.key");

        Tls.tls_config_free(*server_config);
        Tls.tls_config_free(*rsa_config);
        Tls.tls_config_free(*mutual);
    }

    // Our client against `openssl s_server -www`, which answers any GET with a status page.
    // Returns why it did not work, or "". The port is picked by listening and closing, so another
    // process (the stress runs start many copies of this test) can take it before s_server does:
    // `test_openssl_server` then tries again.
    try_client :: (client_config: *Tls.Tls_Config, version_flag: string, expected_version: string, certificate: string) -> string {
        probe, error := Io.tcp_listen(LOOPBACK);
        address, _ := Io.local_address(probe);

        Io.close(probe);

        child: Child;

        args: [..] string;

        defer array_free(args);

        array_add(*args, "openssl", "s_server", "-accept", tprint("127.0.0.1:%", address.port), "-cert", tprint("interop-%.pem", certificate), "-key", tprint("interop-%.key", certificate), "-www", "-quiet", "-alpn", "http/1.1");

        if version_flag.count array_add(*args, version_flag);

        start_child(*child, ..args);

        defer stop_child(*child);

        // Wait for it to listen, and for it to be the one that does: a child that ended did not.
        tcp: Io.Tcp_Stream;

        for 1..300 {
            tcp, error = Io.connect(address);

            if error == .NONE break;

            Io.sleep(10);
        }

        if error != .NONE return tprint("s_server never listened: %", error);

        Io.sleep(30);

        _, status := Process.get_process_result(*child.process, 0);

        if status.type != .STILL_RUNNING {
            Io.close(tcp);

            return "s_server ended: the port was taken";
        }

        stream, tls_error := Tls.tls_connect(client_config, Io.reader_of(tcp), Io.writer_of(tcp), "localhost");

        if tls_error.kind != .NONE {
            Io.close(tcp);

            return tprint("handshake with s_server: %", Tls.error_text(tls_error));
        }

        defer Tls.close(stream);

        if Tls.version(stream) != expected_version return tprint("version %", Tls.version(stream));
        if Tls.alpn_protocol(stream) != "http/1.1" return tprint("alpn '%'", Tls.alpn_protocol(stream));

        Tls.write(stream, "GET / HTTP/1.0\r\n\r\n");

        reply, read_error := Io.read_all(Tls.reader_of(stream));

        defer free(reply.data);

        if read_error != .NONE && read_error != .CONNECTION_RESET return tprint("reading the page: %", read_error);

        page := string.{ reply.count, reply.data };

        if !begins_with(page, "HTTP/1.0 200 ok") return tprint("page: '%' (read: %, tls: %)", page, read_error, Tls.error_text(Tls.last_error(stream)));
        if !has(page, "Secure Renegotiation") && !has(page, "New, TLS") return "the page does not look like s_server's";

        return "";
    }

    check_client :: (client_config: *Tls.Tls_Config, version_flag: string, expected_version: string, certificate: string) {
        problem := "";

        for 1..5 {
            problem = try_client(client_config, version_flag, expected_version, certificate);

            if problem.count == 0 return;
        }

        // What s_server serves differs between OpenSSL builds; the handshake above it is ours.
        if begins_with(problem, "page") || begins_with(problem, "reading the page") || begins_with(problem, "the page") {
            print("skipped s_server %: %\n", version_flag, problem);

            return;
        }

        assert(false, "%", problem);
    }

    test_openssl_server :: () {
        client_config := make_config("", "ca", string.["http/1.1"]);

        init(*client_config);

        check_client(*client_config, "", "TLSv1.3", "server");
        check_client(*client_config, "-tls1_2", "TLSv1.2", "server");
        check_client(*client_config, "", "TLSv1.3", "rsa");
        check_client(*client_config, "-tls1_2", "TLSv1.2", "rsa");

        Tls.tls_config_free(*client_config);
    }

    // OpenSSL reads its certificates from files: the ones the test program carries, written beside it.
    FILES :: string.["ca.pem", "client.pem", "client.key", "server.pem", "server.key", "rsa.pem", "rsa.key"];

    write_files :: () {
        for FILES {
            ok := write_entire_file(tprint("interop-%", it), load(it));

            assert(ok, "could not write interop-%", it);
        }
    }

    remove_files :: () {
        for FILES file_delete(tprint("interop-%", it));
    }

    run_interop :: () {
        write_files();

        defer remove_files();

        test_openssl_client();
        test_openssl_server();
    }

    main :: () {
        if !openssl_present() {
            print("skipped: no openssl command\n");

            return;
        }

        run_group(run_interop);

        print("ok\n");
    }

}
