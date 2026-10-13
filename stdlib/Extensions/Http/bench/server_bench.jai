// Requests per second and latency of the HTTP server over loopback, with keep-alive connections: a
// hello-world text route and a JSON route (a struct in, a struct out). Every scheduler thread runs
// its own server on its own port and its own load clients, so N threads are N independent servers,
// which is what `serve` with `SO_REUSEPORT` gives when the kernel spreads connections evenly. The
// load generator is a few fibers that write a fixed request and read until the response is complete;
// it costs about as much as the server, so the numbers are a floor for the server on its own.
//
// Every request's round trip goes into a histogram (log-linear buckets, about 6% wide; the first two
// requests of a connection are left out), and the p50, p90, p99, p99.9 and the largest are printed
// per route. With no rate the load is closed: each connection sends its next request as soon as the
// last answer arrived, so a slow server is sent fewer requests and its tail is hidden (coordinated
// omission), and the latencies are those of a saturated server. With a rate (requests per second in
// all, over all threads) the load is open: each connection has a schedule, one request every
// `connections / rate` seconds, and a request's latency runs from when it was due, not from when the
// client got round to sending it, so a stall shows up in every request that waited behind it. A
// connection sleeps in whole milliseconds and sends up to a millisecond before its schedule says;
// the latency then runs from the moment the sleep was meant to end, so a timer that fires late
// counts as latency (the clients share the thread with the server, so a stall of the thread delays
// both). The open load uses enough connections per thread that each one waits a few milliseconds
// between requests.
//
//     jaic stdlib/Extensions/Http/bench/server_bench.jai -O2 -exe /tmp/http-bench
//     /tmp/http-bench                 # one thread
//     /tmp/http-bench 4               # four
//     /tmp/http-bench 4 5000          # four threads, 5 seconds per route
//     /tmp/http-bench 4 5000 150000   # the same at 150,000 requests per second (open load)
//     /tmp/http-bench 4 5000 150000 120000   # the JSON route at its own rate
#import "Basic";
#import "String";
#import "Extensions/Http";
#import "Atomics";
Io :: #import "Extensions/Io";

CLIENTS :: 32; // connections per thread, closed load

// An open load wants each connection to wait about this long between requests, so it opens as many
// connections as the rate needs for that (but not fewer than CLIENTS).
OPEN_GAP_NS :: 3_000_000;
MAX_CLIENTS :: 1024;

// Latencies in nanoseconds: exact below 16, then 16 buckets per power of two.
BUCKETS :: 1024;
WARMUP :: 2; // requests per connection that are not recorded

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
    rate: s64; // requests per second over all threads; 0 is the closed load
    requests: s64; // completed by all threads
    bad: s64;
    clients: s64; // connections per thread
    buckets: [BUCKETS] s64; // the threads' histograms added up
    maxima: [..] s64; // the slowest request of each thread
}

// One thread's latencies; its connections all run on that thread, so it needs no locking.
Histogram :: struct {
    counts: [BUCKETS] s64;
    largest: s64;
}

Load :: struct {
    address: Io.Address;
    request: string;
    deadline: s64;
    group: *Io.Wait_Group;
    round: *Round;
    histogram: *Histogram;
    // Open load: nanoseconds between this connection's requests, and when its first is due after
    // it is connected.
    period: s64;
    phase: s64;
}

now_ns :: () -> s64 {
    nanoseconds := to_nanoseconds(current_time_monotonic());

    return nanoseconds;
}

bucket_of :: (nanoseconds: s64) -> int {
    value := cast(u64) max(nanoseconds, 0);

    if value < 16 return cast(int) value;

    shift := 0;

    while (value >> shift) >= 32 shift += 1;

    return min((shift + 1) * 16 + cast(int) ((value >> shift) - 16), BUCKETS - 1);
}

// The largest value that falls into bucket `index`.
bucket_top :: (index: int) -> s64 {
    if index < 16 return index;

    shift := index / 16 - 1;
    sub := index % 16;

    return ((cast(s64) (16 + sub + 1)) << shift) - 1;
}

record :: (histogram: *Histogram, nanoseconds: s64) {
    histogram.counts[bucket_of(nanoseconds)] += 1;

    if nanoseconds > histogram.largest histogram.largest = nanoseconds;
}

// The latency below which `fraction` of the requests fell.
percentile :: (counts: [] s64, total: s64, fraction: float64) -> s64 {
    want := cast(s64) (fraction * cast(float64) total + 0.5);

    if want < 1 want = 1;

    seen := 0;

    for counts {
        seen += it;

        if seen >= want return bucket_top(it_index);
    }

    return 0;
}

// Where the head ends and how long the body is, once the head has arrived. `closes` says the server
// ends the connection after this response (it does after `max_requests_per_connection`, 10,000).
response_size :: (data: [] u8) -> (size: int, closes: bool) {
    text := string.{ data.count, data.data };
    end := find_index_from_left(text, "\r\n\r\n");

    if end < 0 return -1, false;

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

    return end + 4 + length, find_index_from_left(head, "Connection: close") >= 0;
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

    open := load.period > 0;
    due := now_ns() + load.phase; // when the next request of an open load is due

    while true {
        sent := now_ns();
        planned := sent; // when the request was meant to go out

        if open {
            // Whole milliseconds; the last part of the wait is not worth a spin.
            ahead := (due - sent) / 1_000_000;

            if ahead > 0 {
                planned = sent + ahead * 1_000_000;

                Io.sleep(ahead);

                sent = now_ns();
            } else if due < planned {
                planned = due;
            }
        }

        if sent >= load.deadline break;

        _, write_error := Io.write(stream, load.request);

        if write_error != .NONE {
            atomic_add(*load.round.bad, 1);

            return;
        }

        have := 0;
        reconnect := false;

        while true {
            count, read_error := Io.read(stream, .{ buffer.count - have, buffer.data + have });

            if read_error != .NONE || count == 0 {
                atomic_add(*load.round.bad, 1);

                return;
            }

            have += count;

            size, closes := response_size(.{ have, buffer.data });

            if size >= 0 && have >= size {
                reconnect = closes;

                break;
            }
        }

        if open due += load.period;

        if reconnect {
            Io.close(stream);

            stream, error = Io.connect(load.address);

            if error != .NONE {
                atomic_add(*load.round.bad, 1);

                return;
            }
        }

        if done >= WARMUP record(load.histogram, now_ns() - planned);

        done += 1;
    }

    atomic_add(*load.round.requests, done);
}

Run :: struct {
    app: *App;
    rounds: [] Round;
    threads: int;
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
        histogram: Histogram;

        clients := CLIENTS;
        period := 0;

        if round.rate > 0 {
            // Each thread carries its share of the rate.
            share := cast(float64) round.rate / cast(float64) run.threads;

            clients = clamp(cast(int) (share * cast(float64) OPEN_GAP_NS / 1.0e9) + 1, CLIENTS, MAX_CLIENTS);
            period = cast(s64) (cast(float64) clients * 1.0e9 / share);
        }

        round.clients = clients;

        Io.add(*group, clients);

        deadline := now_ns() + round.duration_ms * 1_000_000;

        for i: 0..clients - 1 {
            load := Load.{ server.address, round.request, deadline, *group, round, *histogram, period, 0 };

            // The first requests are spread over one period.
            if period > 0 load.phase = period * i / clients;

            Io.spawn(client, load);
        }

        Io.wait(*group);

        for histogram.counts {
            if it != 0 atomic_add(*round.buckets[it_index], it);
        }

        slot := atomic_add(*round.maxima.count, 1);
        round.maxima[slot] = histogram.largest;
    }

    stop(server);
    wait(server);
}

main :: () {
    args := get_command_line_arguments();
    threads := 1;
    duration := 2000;
    rate := 0;
    rate_json := -1;

    if args.count > 1 threads = parse_int(args[1]);
    if args.count > 2 duration = parse_int(args[2]);
    if args.count > 3 rate = parse_int(args[3]);
    if args.count > 4 rate_json = parse_int(args[4]);
    if rate_json < 0 rate_json = rate;

    if threads < 1 threads = 1;

    app: App;

    get(*app, "/hello", hello);
    post(*app, "/add", add);

    rounds: [2] Round;

    rounds[0].request = "GET /hello HTTP/1.1\r\nHost: bench\r\n\r\n";
    rounds[1].request = "POST /add HTTP/1.1\r\nHost: bench\r\nContent-Type: application/json\r\nContent-Length: 17\r\n\r\n{\"x\":20,\"y\":22}  ";

    rounds[0].rate = rate;
    rounds[1].rate = rate_json;

    for *rounds {
        it.duration_ms = duration;

        array_resize(*it.maxima, threads);
        it.maxima.count = 0;
    }

    run := Run.{ *app, .{ 2, rounds.data }, threads };

    if rate > 0 {
        print("% thread% x open load at % requests/s (JSON route %), % ms per route\n", threads, ifx threads == 1 then "" else "s", rate, rate_json, duration);
    } else {
        print("% thread% x % connections, % ms per route\n", threads, ifx threads == 1 then "" else "s", CLIENTS, duration);
    }

    Io.io_run_threads(threads, thread_main, *run);

    names := string.["hello (text)", "add (JSON in, JSON out)"];

    for round: rounds {
        per_second := cast(float64) round.requests * 1000.0 / cast(float64) round.duration_ms;

        print("%: % requests/s (% threads, % connections each, % failed)\n", names[it_index], cast(s64) per_second, threads, round.clients, round.bad);

        recorded := 0;
        for round.buckets recorded += it;

        largest := 0;
        for round.maxima largest = max(largest, it);

        fractions := float64.[0.5, 0.9, 0.99, 0.999];
        labels := string.["p50", "p90", "p99", "p99.9"];

        print("  latency (us):");

        for fraction: fractions {
            print(" % %", labels[it_index], format_microseconds(percentile(round.buckets, recorded, fraction)));
        }

        print(" max %\n", format_microseconds(largest));
    }
}

// Microseconds with one decimal.
format_microseconds :: (nanoseconds: s64) -> string {
    tenths := (nanoseconds + 50) / 100;

    return tprint("%.%", tenths / 10, tenths % 10);
}

parse_int :: (text: string) -> int {
    value := 0;

    for character: text {
        if character < #char "0" || character > #char "9" break;

        value = value * 10 + (character - #char "0");
    }

    return value;
}
