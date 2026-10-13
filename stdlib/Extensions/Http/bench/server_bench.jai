// Requests per second of the HTTP server over loopback, with keep-alive connections: a hello-world
// text route and a JSON route (a struct in, a struct out). Every scheduler thread runs its own
// server on its own port and its own load clients, so N threads are N independent servers, which is
// what `serve` with `SO_REUSEPORT` gives when the kernel spreads connections evenly. The load
// generator is a few fibers that write a fixed request and read until the response is complete; it
// costs about as much as the server, so the numbers are a floor for the server on its own.
//
//     jaic build stdlib/Extensions/Http/bench/server_bench.jai -O2 -o /tmp/http-bench
//     /tmp/http-bench                 # one thread
//     /tmp/http-bench 4               # four
//     /tmp/http-bench 4 5000          # four threads, 5 seconds per route
#import "Basic";
#import "String";
#import "Extensions/Http";
#import "Atomics";
Io :: #import "Extensions/Io";

CLIENTS :: 32; // connections per thread

Point :: struct {
    x: int;
    y: int;
}

Sum :: struct {
    total: int;
    label: string;
}

hello :: () -> string {
    return "Hello, World!";
}

add :: (body: Json(Point)) -> Sum {
    return .{ body.x + body.y, "sum" };
}

Round :: struct {
    request: string;
    duration_ms: s64;
    requests: s64; // completed by all threads
    bad: s64;
}

Load :: struct {
    address: Io.Address;
    request: string;
    deadline: s64;
    group: *Io.Wait_Group;
    round: *Round;
}

now_ms :: () -> s64 {
    return to_milliseconds(current_time_monotonic());
}

// Where the head ends and how long the body is, once the head has arrived.
response_size :: (data: [] u8) -> int {
    text := string.{ data.count, data.data };
    end := find_index_from_left(text, "\r\n\r\n");

    if end < 0 return -1;

    head := slice(text, 0, end);
    length := 0;
    at := find_index_from_left(head, "Content-Length: ");

    if at >= 0 {
        rest := slice(head, at + 16, head.count - at - 16);
        digits := 0;

        while digits < rest.count && rest[digits] >= #char "0" && rest[digits] <= #char "9" {
            length = length * 10 + (rest[digits] - #char "0");
            digits += 1;
        }
    }

    return end + 4 + length;
}

client :: (load: Load) {
    defer Io.done(load.group);

    stream, error := Io.connect(load.address);

    if error != .NONE {
        atomic_add(*load.round.bad, 1);

        return;
    }

    defer Io.close(stream);

    buffer: [4096] u8;
    done := 0;

    while now_ms() < load.deadline {
        _, write_error := Io.write(stream, load.request);

        if write_error != .NONE {
            atomic_add(*load.round.bad, 1);

            return;
        }

        have := 0;

        while true {
            count, read_error := Io.read(stream, .{ buffer.count - have, buffer.data + have });

            if read_error != .NONE || count == 0 {
                atomic_add(*load.round.bad, 1);

                return;
            }

            have += count;

            size := response_size(.{ have, buffer.data });

            if size >= 0 && have >= size break;
        }

        done += 1;
    }

    atomic_add(*load.round.requests, done);
}

Run :: struct {
    app: *App;
    rounds: [] Round;
}

thread_main :: (run: *Run) {
    app := run.app;

    app.config.address = "127.0.0.1:0";
    app.config.handle_signals = false;

    server, error := start(app);

    if error != .NONE {
        print("cannot listen: %\n", error);
        exit(1);
    }

    for *round: run.rounds {
        group: Io.Wait_Group;

        Io.add(*group, CLIENTS);

        load := Load.{ server.address, round.request, now_ms() + round.duration_ms, *group, round };

        for 1..CLIENTS Io.spawn(client, load);

        Io.wait(*group);
    }

    stop(server);
    wait(server);
}

main :: () {
    args := get_command_line_arguments();
    threads := 1;
    duration := 2000;

    if args.count > 1 threads = parse_int(args[1]);
    if args.count > 2 duration = parse_int(args[2]);

    if threads < 1 threads = 1;

    app: App;

    get(*app, "/hello", hello);
    post(*app, "/add", add);

    rounds: [2] Round;

    rounds[0].request = "GET /hello HTTP/1.1\r\nHost: bench\r\n\r\n";
    rounds[1].request = "POST /add HTTP/1.1\r\nHost: bench\r\nContent-Type: application/json\r\nContent-Length: 17\r\n\r\n{\"x\":20,\"y\":22}  ";
    rounds[0].duration_ms = duration;
    rounds[1].duration_ms = duration;

    run := Run.{ *app, .{ 2, rounds.data } };

    print("% thread% x % connections, % ms per route\n", threads, ifx threads == 1 then "" else "s", CLIENTS, duration);

    Io.io_run_threads(threads, thread_main, *run);

    names := string.["hello (text)", "add (JSON in, JSON out)"];

    for rounds {
        per_second := cast(float64) it.requests * 1000.0 / cast(float64) it.duration_ms;

        print("%: % requests/s (% threads, % failed)\n", names[it_index], cast(s64) per_second, threads, it.bad);
    }
}

parse_int :: (text: string) -> int {
    value := 0;

    for character: text {
        if character < #char "0" || character > #char "9" break;

        value = value * 10 + (character - #char "0");
    }

    return value;
}
