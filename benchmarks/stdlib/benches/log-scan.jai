// Run with jacket bench --path benchmarks/stdlib.
#import "Extensions/Testing";
#import "Basic";
#import "String";
#import "Extensions/Byte_Search";

LINES :: 400_000;

METHODS :: string.["GET", "POST", "PUT", "DELETE"];
PATHS :: string.["/index.html", "/api/v1/users", "/static/app.js", "/api/v1/orders/17", "/health", "/login"];
STATUS :: int.[200, 200, 200, 204, 301, 404, 500];

make_log :: () -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
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

log_scan :: (b: *Bench) {
    log := make_log();

    defer free(log);
    set_bytes(b, log.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        classes: [6] int;
        lines, sent, errors := scan(black_box(log), *classes);

        black_box(lines);
        black_box(sent);
        black_box(errors);
        black_box(classes);
    }
} @bench
