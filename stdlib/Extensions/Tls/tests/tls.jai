// Extensions/Tls over loopback: handshakes (TLS 1.2 and 1.3, EC and RSA certificates), ALPN, SNI,
// data both ways, close_notify and truncation, deadlines and cancellation, certificate checks,
// client certificates, a custom verifier, waiting for input, and many connections at once. Run once
// per backend the system offers (JAI_IO_BACKEND), from this directory.
#load "support/suite.jai";

#if OS == .WINDOWS {
    kernel32 :: #system_library "kernel32";
    SetEnvironmentVariableA :: (name: *u8, value: *u8) -> s32 #foreign kernel32;

    set_env :: (name: string, value: string) {
        SetEnvironmentVariableA(temp_c_string(name), temp_c_string(value));
    }
} else {
    libc :: #system_library "libc";
    setenv :: (name: *u8, value: *u8, overwrite: s32) -> s32 #foreign libc;

    set_env :: (name: string, value: string) {
        setenv(temp_c_string(name), temp_c_string(value), 1);
    }
}

backend_found: string;
reason_found: string;

probe_backend :: () {
    backend_found = Io.backend_name();
    reason_found = Io.backend_reason();
}

main :: () {
    #if OS == .LINUX names := string.["io_uring", "epoll"];
    else #if OS == .WINDOWS names := string.["iocp", "ioring"];
    else names := string.["kqueue"];

    passed := 0;

    for names {
        set_env("JAI_IO_BACKEND", it);

        Io.io_run(probe_backend);

        if backend_found != it {
            print("skipped %: the scheduler got % instead (%)\n", it, backend_found, reason_found);

            continue;
        }

        started := now_ms();

        run_group(run_all);
        passed += 1;

        print("% passed (% ms)\n", it, now_ms() - started);
    }

    assert(passed > 0, "no backend ran");
}
