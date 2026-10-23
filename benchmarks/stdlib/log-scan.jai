// A log scanner: splits a generated access log into lines and fields with Byte_Search, counts
// the requests per status class and the bytes sent, and finds the lines that mention an error.
// The search procedures pick their 16- or 32-byte version once, on first use.
//
//     jaic benchmarks/stdlib/log-scan.jai -O2 -exe log-scan-bench && ./log-scan-bench
#import "Basic";
#import "String";
#import "Extensions/Byte_Search";
#import "Extensions/SIMD";

LINES :: 400_000;
ROUNDS :: 7;

METHODS :: string.["GET", "POST", "PUT", "DELETE"];
PATHS :: string.["/index.html", "/api/v1/users", "/static/app.js", "/api/v1/orders/17", "/health", "/login"];
STATUS :: int.[200, 200, 200, 204, 301, 404, 500];

make_log :: () -> string {
    builder: String_Builder;
    state: u32 = 11;

    for 0..LINES - 1 {
        state = state * 1664525 + 1013904223;
        r := state >> 8;

        print_to_builder(*builder, "10.%.%.% - - [22/Oct/2026:23:%:%] \"% % HTTP/1.1\" % %",
            r % 250, (r >> 3) % 250, (r >> 5) % 250, 10 + r % 50, 10 + (r >> 2) % 50,
            METHODS[r % METHODS.count], PATHS[(r >> 4) % PATHS.count], STATUS[(r >> 6) % STATUS.count], r % 90000);

        // A few lines carry a long message, so the scan has long stretches to skip.
        if r % 17 == 0 append(*builder, " error: upstream timed out while reading the response header from the backend");

        append(*builder, "\n");
    }

    return builder_to_string(*builder);
}

// Returns the number of lines, the bytes sent, the lines that mention an error and the number of
// lines per status class (stored in `classes`).
scan :: (log: string, classes: *[6] int) -> int, int, int {
    lines, sent, errors := 0, 0, 0;
    position := 0;

    while position < log.count {
        end := find(log, #char "\n", position);
        if end < 0 end = log.count;

        line := string.{ end - position, log.data + position };

        position = end + 1;
        lines += 1;

        // The status and size follow the closing quote of the request.
        quote := find_last(line, #char "\"");
        if quote < 0 continue;

        status_start := quote + 2;
        space := find(line, #char " ", status_start);

        if space < 0 continue;

        field := string.{ space - status_start, line.data + status_start };
        status, ok := parse_int(*field);

        if ok && status >= 100 && status < 600 {
            (<<classes)[status / 100] += 1;
        }

        rest := string.{ line.count - space - 1, line.data + space + 1 };
        size_end := find(rest, #char " ");

        if size_end < 0 size_end = rest.count;

        field = string.{ size_end, rest.data };
        size, size_ok := parse_int(*field);
        if size_ok sent += size;

        if find_index_from_left(line, "error:") >= 0 errors += 1;
    }

    return lines, sent, errors;
}

now :: () -> float64 {
    return to_float64_seconds(current_time_monotonic());
}

main :: () {
    log := make_log();
    classes: [6] int;
    best: float64 = 1e18;
    lines, sent, errors: int;

    for 1..ROUNDS {
        for *classes it.* = 0;

        started := now();

        lines, sent, errors = scan(log, *classes);
        elapsed := now() - started;
        if elapsed < best best = elapsed;
    }

    print("% lines, % bytes in the log\n", lines, log.count);
    print("sent %, errors %, classes % % % %\n", sent, errors, classes[2], classes[3], classes[4], classes[5]);
    print("%: % MB/s\n", best_vector_tier(), formatFloat(cast(float64) log.count / best / 1e6, trailing_width = 0));
}
