// Runs every Extensions/Io engine test once per backend the system can offer, forcing each with
// JAI_IO_BACKEND: io_uring and epoll on Linux, kqueue on macOS, and iocp and ioring on Windows. A
// backend the system refuses (io_uring in a container, on an old kernel, or with io_uring_disabled;
// ioring before Windows 11 22H2) is skipped with the engine's reason.
//
// The readiness backends (epoll, kqueue) also run the file tests twice more: once with every file
// operation on the worker thread pool (JAI_IO_FILE_INLINE=0) and once with the pool off
// (JAI_IO_FILE_THREADS=0), so both paths of `poll_submit` are exercised whatever the default
// threshold is.
//
// The default backend and the forced ones must pass the same tests.
#load "lib/suite.jai";

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

        if it == "epoll" || it == "kqueue" {
            set_env("JAI_IO_FILE_INLINE", "0");
            started = now_ms();

            run_group(run_files);
            print("% with every file operation on the pool passed (% ms)\n", it, now_ms() - started);

            set_env("JAI_IO_FILE_INLINE", "");
            set_env("JAI_IO_FILE_THREADS", "0");
            started = now_ms();

            run_group(run_files);
            print("% with the file pool off passed (% ms)\n", it, now_ms() - started);

            set_env("JAI_IO_FILE_THREADS", "");
        }
    }

    assert(passed > 0, "no backend ran");
}
