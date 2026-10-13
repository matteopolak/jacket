// Memory of the HTTP server while it holds many idle keep-alive connections. The server runs in a
// child process (so the clients' own memory is not counted); the parent opens N connections, sends
// one request on each, keeps them open and idle, and reads the child's resident set from
// /proc/<pid>/status before and after. Linux only: elsewhere it prints that it skipped.
//
//     jaic stdlib/Extensions/Http/bench/idle_bench.jai -O2 -exe /tmp/http-idle
//     ulimit -n 65536                   # both ends of every connection are descriptors
//     /tmp/http-idle 1000               # N connections (default 1000)
//     /tmp/http-idle 10000 http         # also a final request on every connection, to show it still works
#import "Basic";
#import "String";
#import "Extensions/Http";
Io :: #import "Extensions/Io";

#if OS == .LINUX {
    libc :: #system_library "libc";

    c_fork :: () -> s32 #foreign libc "fork";
    c_pipe :: (fds: *s32) -> s32 #foreign libc "pipe";
    c_read :: (fd: s32, buffer: *void, length: u64) -> s64 #foreign libc "read";
    c_write :: (fd: s32, buffer: *void, length: u64) -> s64 #foreign libc "write";
    c_kill :: (pid: s32, signal: s32) -> s32 #foreign libc "kill";
    c_waitpid :: (pid: s32, status: *s32, options: s32) -> s32 #foreign libc "waitpid";
    c_exit :: (status: s32) #foreign libc "_exit";
}

WORKERS :: 64; // connections being opened at once

hello :: () -> string {
    return "Hello, World!";
}

Run :: struct {
    address: Io.Address;
    target: int;
    opened: int;
    failed: int;
    held: [..] Io.Tcp_Stream;
    group: Io.Wait_Group;
}

REQUEST :: "GET /hello HTTP/1.1\r\nHost: bench\r\n\r\n";

opener :: (run: *Run) {
    defer Io.done(*run.group);

    buffer: [1024] u8;

    while run.opened < run.target {
        run.opened += 1;

        stream, error := Io.connect(run.address);

        if error != .NONE {
            run.failed += 1;

            continue;
        }

        _, error = Io.write(stream, REQUEST);

        count: s64;

        if error == .NONE count, error = Io.read(stream, buffer);

        if error != .NONE || count == 0 {
            run.failed += 1;
            Io.close(stream);

            continue;
        }

        array_add(*run.held, stream);
    }
}

// The "VmRSS" and "VmHWM" lines of /proc/<pid>/status, in KiB.
resident :: (pid: s32) -> rss: int, peak: int {
    // Not `read_file`: a /proc file reports no size.
    file, error := Io.open_file(tprint("/proc/%/status", pid), .READ);
    if error != .NONE return -1, -1;

    defer Io.close(*file);

    data, read_error := Io.read_all(Io.reader_of(*file));
    if read_error != .NONE return -1, -1;

    defer free(data.data);

    text := string.{ data.count, data.data };
    rss := -1;
    peak := -1;

    while text.count > 0 {
        line := text;
        newline := find_index_from_left(text, "\n");

        if newline >= 0 {
            line = slice(text, 0, newline);
            text = slice(text, newline + 1, text.count - newline - 1);
        } else {
            text = "";
        }

        if begins_with(line, "VmRSS:") rss = number_in(line);
        if begins_with(line, "VmHWM:") peak = number_in(line);
    }

    return rss, peak;
}

// The resident KiB of the mappings of exactly `size_kib` KiB in /proc/<pid>/smaps: with Io's default
// `STACK_BYTES` (256 KiB, 16 KiB of it guard) these are the fibers' stacks.
resident_mappings :: (pid: s32, size_kib: int) -> int {
    file, error := Io.open_file(tprint("/proc/%/smaps", pid), .READ);
    if error != .NONE return -1;

    defer Io.close(*file);

    data, read_error := Io.read_all(Io.reader_of(*file));
    if read_error != .NONE return -1;

    defer free(data.data);

    text := string.{ data.count, data.data };
    total := 0;
    matching := false;

    while text.count > 0 {
        line := text;
        newline := find_index_from_left(text, "\n");

        if newline >= 0 {
            line = slice(text, 0, newline);
            text = slice(text, newline + 1, text.count - newline - 1);
        } else {
            text = "";
        }

        if begins_with(line, "Size:") matching = number_in(line) == size_kib;
        if matching && begins_with(line, "Rss:") total += number_in(line);
    }

    return total;
}

number_in :: (line: string) -> int {
    value := 0;

    for character: line {
        if character >= #char "0" && character <= #char "9" value = value * 10 + (character - #char "0");
    }

    return value;
}

parent :: (pid: s32, port: u16, target: int, check: bool) {
    run: Run;

    run.address = Io.address_v4(127, 0, 0, 1, port);
    run.target = target;

    before_rss, _ := resident(pid);

    Io.add(*run.group, WORKERS);

    for 1..WORKERS Io.spawn(opener, *run);

    Io.wait(*run.group);

    // Let the server's fibers settle into their idle wait.
    Io.sleep(500);

    rss, peak := resident(pid);

    print("backend % | % idle connections (% failed)\n", Io.backend_name(), run.held.count, run.failed);
    print("server RSS: % KiB before, % KiB after, peak % KiB: % bytes per connection\n", before_rss, rss, peak, ifx run.held.count > 0 then (rss - before_rss) * 1024 / run.held.count else 0);

    stacks := resident_mappings(pid, 240);

    print("  of which in fiber stacks: % KiB, % bytes per connection\n", stacks, ifx run.held.count > 0 then stacks * 1024 / run.held.count else 0);

    if check {
        buffer: [1024] u8;
        bad := 0;

        for run.held {
            _, error := Io.write(it, REQUEST);
            count: s64;

            if error == .NONE count, error = Io.read(it, buffer);
            if error != .NONE || count == 0 bad += 1;
        }

        print("a second request on every connection: % failed\n", bad);
    }

    for run.held Io.close(it);
}

main :: () {
    #if OS != .LINUX {
        print("skipped: this bench reads /proc\n");
    } else {
        args := get_command_line_arguments();
        target := 1000;
        check := false;

        if args.count > 1 target = parse_int(args[1]);
        if args.count > 2 check = args[2] == "http";

        fds: [2] s32;

        if c_pipe(fds.data) < 0 {
            print("pipe failed\n");
            exit(1);
        }

        pid := c_fork();

        if pid == 0 {
            // The server: reports its port, then serves until it is killed.
            app: App;

            app.config.address = "127.0.0.1:0";
            app.config.handle_signals = false;

            get(*app, "/hello", hello);

            child := Child.{ *app, fds[1] };

            Io.io_run((child: *Child) {
                server, error := start(child.app);

                if error != .NONE {
                    print("cannot listen: %\n", error);
                    exit(1);
                }

                port := server.address.port;

                c_write(child.fd, *port, 2);

                wait(server);
            }, *child);

            c_exit(0);
        }

        port: u16;

        if c_read(fds[0], *port, 2) != 2 {
            print("the server did not start\n");
            exit(1);
        }

        state := Parent_State.{ pid, port, target, check };

        Io.io_run((state: *Parent_State) {
            parent(state.pid, state.port, state.target, state.check);
        }, *state);

        c_kill(pid, 9);
        c_waitpid(pid, null, 0);
    }
}

Child :: struct {
    app: *App;
    fd: s32;
}

Parent_State :: struct {
    pid: s32;
    port: u16;
    target: int;
    check: bool;
}

parse_int :: (text: string) -> int {
    value := 0;

    for character: text {
        if character < #char "0" || character > #char "9" break;

        value = value * 10 + (character - #char "0");
    }

    return value;
}
