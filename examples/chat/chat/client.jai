Colors :: #import "Print_Color";
#import "Sort";

Client_Options :: struct {
    address: string = "127.0.0.1:7979";
    roots: string;
    nick: string = "guest";
    room: string = "lobby";
    script: string;
}

// Reads all asynchronous events preceding the command's terminal OK/ERR response.
call :: (link: *Link, command: string, echo := true, expect_error := false) -> bool {
    if !send_line(link, command) return false;

    while true {
        line, error := Io.read_line(*link.reader);
        if error != .NONE || link.reader.unterminated return false;
        if echo print("%\n", line);
        if begins_with(line, "ERR ") return expect_error;
        if begins_with(line, "OK ") return !expect_error;
    }

    return false;
}

client_open :: (link: *Link, options: Client_Options) -> bool {
    if !link_connect(link, options.address, options.roots) return false;
    ready, error := Io.read_line(*link.reader);
    if error != .NONE || ready != "READY chat/1" return false;
    return call(link, tprint("NICK %", options.nick), false) && call(link, tprint("JOIN %", options.room), false);
} @TempAllocates

script_run :: (link: *Link, path: string) -> bool {
    text, opened := read_entire_file(path);
    if !opened return false;

    defer free(text);
    lines := split(text, "\n",, temp);
    for lines {
        line := it;
        if ends_with(line, "\r") line.count -= 1;
        if line.count == 0 continue;

        if begins_with(line, "WAIT ") {
            delay, valid := string_to_int(slice(line, 5, line.count - 5));
            if !valid || delay < 0 || delay > 10000 return false;

            Io.with_deadline(delay, #code {
                while true {
                    event, error := Io.read_line(*link.reader);
                    if error == .TIMED_OUT break;
                    if error != .NONE return false;
                    print("%\n", event);
                }
            });
        } else {
            // EXPECT-ERROR is local to scripts and asserts a rejected command.
            expect_error := begins_with(line, "EXPECT-ERROR ");
            if expect_error line = slice(line, 13, line.count - 13);
            if !call(link, line, true, expect_error) return false;
            if line == "QUIT" return true;
        }
    }

    return call(link, "QUIT");
}

Terminal :: struct {
    link: *Link;
    input: Io.Io_File;
    input_fiber: Io.Fiber_Handle;
    done: Io.Wait_Group;
    input_finished: bool;
}

redraw_input :: (terminal: *Terminal) {
    print("\x1b[24;1H\x1b[2K> ");
}

terminal_input :: (terminal: *Terminal) {
    defer Io.done(*terminal.done);
    defer terminal.input_finished = true;
    input := Io.make_line_reader(Io.reader_of(*terminal.input), 1024);

    defer Io.free_line_reader(*input);

    while true {
        line, error := Io.read_line(*input);
        if error != .NONE break;

        if line.count > 0 {
            command := ifx line[0] == #char "/" then slice(line, 1, line.count - 1) else tprint("SAY %", line);
            if !send_line(terminal.link, command) break;
        }

        redraw_input(terminal);
        reset_temporary_storage();
    }
}

terminal_run :: (link: *Link) {
    #if OS == .WINDOWS {
        print("chat: interactive terminal needs POSIX; use --script\n");
        exit(1);
    } else {
        terminal := Terminal.{ link = link };
        error: Io.Io_Error;
        terminal.input, error = Io.open_stream_file("/dev/stdin");
        if error != .NONE {
            print("chat: no terminal; use --script\n");
            exit(1);
        }

        defer Io.close(*terminal.input);
        print("\x1b[2J\x1b[1;22r\x1b[23;1Hchat: /JOIN room /DM nick text /HISTORY /QUIT");
        defer print("\x1b[r\x1b[24;1H\n");
        Io.add(*terminal.done);
        terminal.input_fiber = Io.spawn(terminal_input, *terminal);
        redraw_input(*terminal);

        while !Io.interrupted() && !terminal.input_finished {
            line: string;
            received: Io.Io_Error;

            Io.with_deadline(200, #code {
                line, received = Io.read_line(*link.reader);
            });
            if received == .TIMED_OUT continue;
            if received != .NONE || line == "OK QUIT" break;

            // The third word of MSG/HISTORY is the nick. Hash it to a stable palette slot.
            parts := split(line, " ",, temp);
            hash: u32;

            if parts.count >= 3 for parts[2] hash = hash * 31 + cast(u32) it;

            palette := Colors.Console_Color.[.CYAN, .GREEN, .YELLOW, .MAGENTA, .BLUE, .WHITE];

            print("\x1b7\x1b[22;1H\n");
            Colors.print_color("%", line, color = palette[hash % 6]);
            print("\x1b8");
            reset_temporary_storage();
        }

        Io.cancel(terminal.input_fiber);
        Io.wait(*terminal.done);
    }
}

run_client :: (options: Client_Options) {
    link: Link;

    defer link_close(*link);
    connected := false;

    Io.with_deadline(5000, #code {
        connected = client_open(*link, options);
    });

    if !connected {
        print("chat: cannot connect or register\n");
        exit(1);
    }

    if options.script.count > 0 {
        passed := false;

        Io.with_deadline(30000, #code {
            passed = script_run(*link, options.script);
        });
        if !passed exit(1);
    } else terminal_run(*link);
}

Load_Options :: struct {
    address: string = "127.0.0.1:7979";
    roots: string;
    clients: s64 = 4;
    rounds: s64 = 10;
}

Load :: struct {
    options: Load_Options;
    group: Io.Wait_Group;
    samples: [..] float64;
    failed: s64;
    run: s64;
}

Load_Work :: struct {
    load: *Load;
    index: s64;
}

load_worker :: (work: Load_Work) {
    defer Io.done(*work.load.group);
    Io.with_deadline(10000, #code {
        load_worker_body(work);
    });
}

load_worker_body :: (work: Load_Work) {
    load := work.load;
    link: Link;

    defer link_close(*link);
    options := Client_Options.{ address = load.options.address, roots = load.options.roots, nick = tprint("load%_%", load.run, work.index), room = "load" };
    if !client_open(*link, options) {
        load.failed += 1;
        return;
    }

    for 1..load.options.rounds {
        began := seconds_since_init();

        if !call(*link, tprint("SAY sample%", it), false) {
            load.failed += 1;
            return;
        }

        array_add(*load.samples, (seconds_since_init() - began) * 1000);
    }

    if !call(*link, "QUIT", false) load.failed += 1;
}

run_load :: (options: Load_Options) {
    load := Load.{ options = options, run = to_milliseconds(current_time_monotonic()) % 1000000 };

    defer array_free(load.samples);
    began := seconds_since_init();

    Io.with_deadline(15000, #code {
        for 0..options.clients - 1 {
            Io.add(*load.group);
            Io.spawn(load_worker, Load_Work.{ *load, it });
        }

        Io.wait(*load.group);
    });
    // Each worker has its own deadline; always join before freeing their shared recorder.
    Io.wait(*load.group);
    elapsed := seconds_since_init() - began;

    quick_sort(load.samples, (a, b) => ifx a < b then -1 else ifx a > b then 1 else 0);

    if load.samples.count == 0 {
        print("chat: no load samples\n");
        exit(1);
    }

    p50 := (load.samples.count - 1) / 2;
    p99 := (load.samples.count - 1) * 99 / 100;

    print("messages=% messages/s=% p50=%ms p99=%ms failures=%\n", load.samples.count, cast(s64) (cast(float64) load.samples.count / elapsed), load.samples[p50], load.samples[p99], load.failed);
    if load.failed > 0 exit(1);
}
