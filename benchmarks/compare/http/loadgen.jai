// Load generator of the HTTP server comparison (tools/bench/http_compare.py).
//
// Opens BENCH_CONNECTIONS keep-alive connections to 127.0.0.1:BENCH_PORT, spread over BENCH_THREADS
// threads with one fiber per connection. A fiber sends `GET /items/N` (N random in 1..1000), waits for
// the whole response, and sends the next request: closed-loop, one request in flight per connection.
// A server that closes the connection after a response (HTTP/1.0, `Connection: close`) is reconnected to,
// and the new connection counts in that request's latency. Requests in the first BENCH_WARMUP_MS are not
// recorded. With BENCH_HOLD_MS each connection makes one request and stays open that long, for the
// memory-per-connection runs. The last line on standard output is one JSON object with the totals and the latency percentiles.
#import "Basic";
#import "String";
#import "Thread";
#import "Sort";
#import "Extensions/Environment";
Io :: #import "Extensions/Io";

ITEMS :: 1000;
REQUEST_TIMEOUT_MS :: 5000;

Worker :: struct {
    thread: Thread;
    address: string;
    connections: s64;
    warmup_end: s64;
    end: s64;
    seed: u64;
    // More than 0: each connection makes one request, then stays open this long (idle-memory runs).
    hold_ms: s64;

    latencies: [..] s64;
    requests: s64;
    // Failures by kind: a connect that was refused or reset, one that did not finish within the
    // request timeout, a request that failed on a connected socket (reset, closed early, malformed
    // answer), and one that got no complete answer within the timeout.
    connect_errors: s64;
    connect_timeouts: s64;
    request_errors: s64;
    request_timeouts: s64;
    connect_why: string;
    request_why: string;
    reconnects: s64;
    body_bytes: s64;
    sample: string;
}

now_ns :: () -> s64 {
    nanoseconds := to_nanoseconds(current_time_monotonic());
    return nanoseconds;
}

env_int :: (name: string, fallback: int) -> int {
    text := environment_variable(name);
    if !text return fallback;

    value, ok := parse_int(*text);
    if !ok return fallback;

    return value;
}

next_random :: (w: *Worker) -> u64 {
    w.seed ^= w.seed << 13;
    w.seed ^= w.seed >> 7;
    w.seed ^= w.seed << 17;
    return w.seed;
}

lower :: (c: u8) -> u8 {
    if c >= #char "A" && c <= #char "Z" return c + 32;
    return c;
}

// Whether `head` has the header `name` (lower case, with the colon); its value goes to `value`.
header_value :: (head: string, name: string) -> string, bool {
    position := 0;

    while position < head.count {
        end := position;
        while end < head.count && head[end] != #char "\n" end += 1;

        line := string.{ end - position, head.data + position };

        if line.count > name.count {
            same := true;

            for 0..name.count - 1 {
                if lower(line[it]) != name[it] {
                    same = false;
                    break;
                }
            }

            if same return trim(string.{ line.count - name.count, line.data + name.count }), true;
        }

        position = end + 1;
    }

    return "", false;
}

// The result of one request on a connection.
Exchange :: struct {
    ok: bool;
    // Whether the connection must be replaced before the next request.
    closed: bool;
    body: string;
    // Why it failed, for the first failure of each kind that is reported on standard error.
    why: string;
}

exchange :: (conn: Io.Tcp_Stream, request: string, buffer: [] u8) -> Exchange {
    result: Exchange;

    _, error := Io.write(conn, request);

    if error != .NONE {
        result.why = tprint("write: %", error);
        return result;
    }

    filled := 0;
    head_end := -1;

    while head_end < 0 {
        if filled == buffer.count return result;

        count, error := Io.read(conn, array_view(buffer, filled, buffer.count - filled));
        if error != .NONE || count == 0 {
            result.why = tprint("read of the head after % bytes: %", filled, ifx error != .NONE then tprint("%", error) else "closed by the server");
            return result;
        }

        filled += count;

        for 3..filled - 1 {
            if buffer[it] == #char "\n" && buffer[it - 1] == #char "\r" && buffer[it - 2] == #char "\n" && buffer[it - 3] == #char "\r" {
                head_end = it + 1;
                break;
            }
        }
    }

    head := string.{ head_end, buffer.data };

    if head.count < 12 || head[9] != #char "2" {
        result.why = tprint("status line: %", string.{ min(head.count, 20), head.data });
        return result;
    }

    length_text, has_length := header_value(head, "content-length:");
    connection, has_connection := header_value(head, "connection:");
    http_1_0 := head.count > 8 && head[7] == #char "0";

    result.closed = http_1_0 || (has_connection && equal_nocase(connection, "close"));

    if has_length {
        length, parsed := parse_int(*length_text);
        if !parsed return result;

        while filled - head_end < length {
            if filled == buffer.count return result;

            count, error := Io.read(conn, array_view(buffer, filled, buffer.count - filled));
            if error != .NONE || count == 0 return result;

            filled += count;
        }
    } else {
        // The body ends where the connection does.
        result.closed = true;

        while true {
            if filled == buffer.count return result;

            count, error := Io.read(conn, array_view(buffer, filled, buffer.count - filled));
            if error != .NONE return result;
            if count == 0 break;

            filled += count;
        }
    }

    result.body = string.{ filled - head_end, buffer.data + head_end };
    result.ok = true;

    return result;
}

connect_with_deadline :: (address: string) -> Io.Tcp_Stream, Io.Io_Error {
    conn: Io.Tcp_Stream;
    error: Io.Io_Error;

    Io.with_deadline(REQUEST_TIMEOUT_MS, #code {
        conn, error = Io.connect(address);
    });

    return conn, error;
}

connection :: (w: *Worker) {
    buffer: [8192] u8;
    conn: Io.Tcp_Stream;
    connected := false;

    while now_ns() < w.end {
        reset_temporary_storage();

        started := now_ns();

        if !connected {
            error: Io.Io_Error;
            conn, error = connect_with_deadline(w.address);

            if error != .NONE {
                if error == .TIMED_OUT w.connect_timeouts += 1;
                else w.connect_errors += 1;

                if !w.connect_why w.connect_why = copy_string(tprint("%", error));

                Io.sleep(20);
                continue;
            }

            connected = true;
        }

        id := 1 + cast(int) (next_random(w) % ITEMS);
        request := tprint("GET /items/% HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept: application/json\r\n\r\n", id);

        result: Exchange;

        Io.with_deadline(REQUEST_TIMEOUT_MS, #code {
            result = exchange(conn, request, buffer);
        });

        finished := now_ns();

        if !result.ok {
            // The deadline cut the exchange short when it ran for the whole timeout.
            if finished - started >= (REQUEST_TIMEOUT_MS - 100) * 1_000_000 w.request_timeouts += 1;
            else w.request_errors += 1;

            if !w.request_why w.request_why = copy_string(result.why);

            Io.close(conn, true);
            connected = false;
            continue;
        }

        if finished >= w.warmup_end || w.hold_ms > 0 {
            array_add(*w.latencies, finished - started);
            w.requests += 1;
            w.body_bytes += result.body.count;
            if !w.sample w.sample = copy_string(result.body);
        }

        if w.hold_ms > 0 && !result.closed {
            Io.sleep(w.hold_ms);
            Io.close(conn, true);
            return;
        }

        if result.closed {
            Io.close(conn, true);
            connected = false;
            w.reconnects += 1;
        }
    }

    if connected Io.close(conn, true);
}

run_worker :: (w: *Worker) {
    group: Io.Wait_Group;

    for 1..w.connections {
        Io.add(*group);
        pair := New(Pair);
        pair.worker = w;
        pair.group = *group;

        Io.spawn((data: *Pair) {
            connection(data.worker);
            Io.done(data.group);
            free(data);
        }, pair);
    }

    Io.wait(*group);
}

Pair :: struct {
    worker: *Worker;
    group: *Io.Wait_Group;
}

percentile :: (sorted: [] s64, fraction: float64) -> s64 {
    if sorted.count == 0 return 0;
    index := cast(s64) (fraction * cast(float64) (sorted.count - 1));
    return sorted[index];
}

thread_main :: (thread: *Thread) -> s64 {
    w := cast(*Worker) thread.data;
    Io.io_run(run_worker, w);
    return 0;
}

main :: () {
    connections := env_int("BENCH_CONNECTIONS", 1);
    threads := env_int("BENCH_THREADS", 2);
    seconds := env_int("BENCH_SECONDS", 5);
    warmup_ms := env_int("BENCH_WARMUP_MS", 1000);
    address := tprint("127.0.0.1:%", env_int("BENCH_PORT", 8080));

    if threads > connections threads = connections;

    workers := NewArray(threads, Worker);
    start := now_ns();

    for *workers {
        it.address = address;
        it.connections = connections / threads + ifx it_index < connections % threads then 1 else 0;
        it.warmup_end = start + warmup_ms * 1_000_000;
        it.end = it.warmup_end + seconds * 1_000_000_000;
        it.hold_ms = env_int("BENCH_HOLD_MS", 0);
        it.seed = 0x9E3779B97F4A7C15 + cast(u64) it_index * 7919;
        thread_init(*it.thread, thread_main);
        it.thread.data = it;
        thread_start(*it.thread);
    }

    all: [..] s64;
    requests, reconnects, body_bytes: s64;
    connect_errors, connect_timeouts, request_errors, request_timeouts: s64;
    sample: string;
    connect_why, request_why: string;

    for *workers {
        thread_is_done(*it.thread, -1);
        thread_deinit(*it.thread);
        array_add(*all, ..it.latencies);
        requests += it.requests;
        connect_errors += it.connect_errors;
        connect_timeouts += it.connect_timeouts;
        request_errors += it.request_errors;
        request_timeouts += it.request_timeouts;
        reconnects += it.reconnects;
        body_bytes += it.body_bytes;
        if !sample sample = it.sample;
        if !connect_why connect_why = it.connect_why;
        if !request_why request_why = it.request_why;
    }

    quick_sort(all, (value: s64) -> s64 {
        return value;
    });

    print("sample: %\n", sample, to_standard_error = true);
    if connect_why print("first connect failure: %\n", connect_why, to_standard_error = true);
    if request_why print("first request failure: %\n", request_why, to_standard_error = true);

    print("{\"connections\":%,\"threads\":%,\"seconds\":%,\"requests\":%,\"connect_errors\":%,\"connect_timeouts\":%,\"request_errors\":%,\"request_timeouts\":%,\"reconnects\":%,\"requests_per_second\":%,\"p50_ns\":%,\"p99_ns\":%,\"p999_ns\":%,\"max_ns\":%,\"sample_bytes\":%}\n",
          connections, threads, seconds, requests, connect_errors, connect_timeouts, request_errors, request_timeouts, reconnects,
          cast(s64) (cast(float64) requests / cast(float64) seconds),
          percentile(all, 0.50), percentile(all, 0.99), percentile(all, 0.999), percentile(all, 1.0),
          sample.count);
}
