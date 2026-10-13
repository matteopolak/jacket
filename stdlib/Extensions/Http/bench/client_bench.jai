// Requests per second of the HTTP client over loopback: fibers that `get` a small text route and
// read the body, against a server on the same scheduler (so the number is client and server
// together), plus the memory it takes to hold many unread responses at once (each holds its
// connection and, until it is read or closed, its read buffer).
//
//     jaic build stdlib/Extensions/Http/bench/client_bench.jai -O2 -o /tmp/http-client-bench
//     /tmp/http-client-bench              # 3 seconds
//     /tmp/http-client-bench 5000         # 5 seconds
//     /tmp/http-client-bench 0            # only the memory figure, in a fresh process
#import "Basic";
#import "String";
#import "Extensions/Http";
Io :: #import "Extensions/Io";

FIBERS :: 32;
OPEN :: 500; // responses held open for the memory figure

hello :: () -> string {
    return "Hello, World!";
}

now_ms :: () -> s64 {
    return to_milliseconds(current_time_monotonic());
}

Load :: struct {
    url: string;
    deadline: s64;
    done: int;
    bad: int;
    group: Io.Wait_Group;
}

worker :: (load: *Load) {
    defer Io.done(*load.group);

    api: Client;

    while now_ms() < load.deadline {
        response, error := get(*api, load.url);

        if error.kind != .NONE {
            load.bad += 1;

            continue;
        }

        text, _ := read_text(*response);

        if text != "Hello, World!" load.bad += 1;

        free(text.data);
        close(*response);

        load.done += 1;
    }
}

#if OS == .LINUX {
    // Resident set of this process in KiB, from /proc/self/status (-1 elsewhere).
    resident :: () -> int {
        file, error := Io.open_file("/proc/self/status", .READ);
        if error != .NONE return -1;

        defer Io.close(*file);

        data, read_error := Io.read_all(Io.reader_of(*file));
        if read_error != .NONE return -1;

        defer free(data.data);

        text := string.{ data.count, data.data };
        at := find_index_from_left(text, "VmRSS:");

        if at < 0 return -1;

        value := 0;

        for index: at + 6..text.count - 1 {
            character := text[index];

            if character == #char "\n" break;
            if character >= #char "0" && character <= #char "9" value = value * 10 + (character - #char "0");
        }

        return value;
    }
} else {
    resident :: () -> int {
        return -1;
    }
}

run :: (duration: *int) {
    app: App;

    app.config.address = "127.0.0.1:0";
    app.config.handle_signals = false;

    get(*app, "/hello", hello);

    server, error := start(*app);

    if error != .NONE {
        print("cannot listen: %\n", error);
        exit(1);
    }

    url := tprint("%/hello", server_url(server));

    if duration.* > 0 {
        load: Load;

        load.url = url;
        load.deadline = now_ms() + duration.*;

        Io.add(*load.group, FIBERS);

        for 1..FIBERS Io.spawn(worker, *load);

        Io.wait(*load.group);

        per_second := cast(float64) load.done * 1000.0 / cast(float64) duration.*;

        print("%: % requests/s (% failed)\n", Io.backend_name(), cast(s64) per_second, load.bad);
    }

    // Responses held open: one connection each, nothing read yet.
    api: Client;
    held: [..] Client_Response;

    before := resident();

    for 1..OPEN {
        response, failure := get(*api, url);

        if failure.kind == .NONE array_add(*held, response);
    }

    after := resident();

    if before >= 0 print("% responses held open: % KiB resident, % bytes each (client and server together)\n", held.count, after - before, (after - before) * 1024 / max(held.count, 1));

    for *held close(it);

    array_reset(*held);

    stop(server);
    wait(server);
}

main :: () {
    args := get_command_line_arguments();
    duration := 3000;

    if args.count > 1 {
        duration = 0;

        for character: args[1] duration = duration * 10 + (character - #char "0");
    }

    Io.io_run(run, *duration);
}
