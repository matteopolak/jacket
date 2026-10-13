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
// epoll also runs a test that makes the kernel refuse to watch a descriptor (Linux only).
//
// The default backend and the forced ones must pass the same tests.
#load "lib/suite.jai";

#if OS == .LINUX {
    libc :: #system_library "libc";

    c_readlink :: (path: *u8, buffer: *u8, size: u64) -> s64 #foreign libc "readlink";
    c_open :: (path: *u8, flags: s32) -> s32 #foreign libc "open";
    c_dup2 :: (from: s32, to: s32) -> s32 #foreign libc "dup2";
    c_close :: (fd: s32) -> s32 #foreign libc "close";

    // The scheduler's epoll descriptor, or -1.
    find_epoll_descriptor :: () -> s32 {
        link: [64] u8;

        for 3..255 {
            count := c_readlink(temp_c_string(tprint("/proc/self/fd/%", it)), link.data, link.count);
            if count <= 0 continue;

            if string.{ count, link.data } == "anon_inode:[eventpoll]" return cast(s32) it;
        }

        return -1;
    }

    // A read that has to wait puts its descriptor in the epoll set from `poll_submit`. If the
    // kernel refuses, that read ends with the error, in the fiber that asked: the fiber is running
    // while its own operation fails, so nothing may wake it, and it must not wait for a completion
    // that will never come. The epoll descriptor is replaced by /dev/null to make the kernel refuse,
    // which leaves this scheduler unable to wait, so the test ends right after.
    test_epoll_refuses_a_descriptor :: () {
        assert(Io.backend_name() == "epoll", "this test is for epoll");

        pair := make_pair();

        epoll_fd := find_epoll_descriptor();

        assert(epoll_fd >= 0, "the epoll descriptor");

        null_fd := c_open("/dev/null", 2);

        assert(null_fd >= 0 && c_dup2(null_fd, epoll_fd) == epoll_fd, "replace the epoll descriptor");
        c_close(null_fd);

        buffer: [8] u8;
        count, error := Io.read(pair.server, buffer);

        assert(count == 0 && error == .INVALID, "a read the kernel will not watch: % %", count, error);

        // The other waiters on that descriptor were not left behind either.
        close_pair(pair);
    }

    run_epoll_refusal :: () {
        test_epoll_refuses_a_descriptor();
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

            #if OS == .LINUX {
                if it == "epoll" {
                    Io.io_run(run_epoll_refusal);
                    print("epoll refusing a descriptor passed\n");
                }
            }
        }
    }

    assert(passed > 0, "no backend ran");
}
