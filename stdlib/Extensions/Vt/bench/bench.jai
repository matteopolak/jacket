// Generate and optionally record a mixed stream, then measure callbacks per codepoint/command.
// Example: jaic -O2 stdlib/Extensions/Vt/bench/bench.jai -exe /tmp/vt-bench
//          /tmp/vt-bench /tmp/vt-mixed.bin
#import "Basic";
#import "File";
#import "Extensions/Vt";

count_action :: (data: *void, action: Vt_Action) {
    total := cast(*u64) data;
    total.* += 1;
}

main :: () {
    b: String_Builder;
    for 0..16383 append(*b, "building package: hello world\r\n\x1b[1;32mPASS\x1b[0m\x1b[12;40Hnext\x1b[2D");

    text := builder_to_string(*b);

    defer free(text);
    args := get_command_line_arguments();
    if args.count > 1 assert(write_entire_file(args[1], text));

    best: float64 = 1.0e30;
    total: u64;

    for 0..4 {
        total = 0;
        p: Vt_Parser;
        start := current_time_monotonic();

        for 0..7 {
            error := vt_feed(*p, text, count_action, *total);
            assert(error.kind == .NONE);
        }

        best = min(best, to_float64_seconds(current_time_monotonic() - start));
    }

    print("Vt: % MB/s, % bytes, % actions/pass (best of 5, 8 passes)\n", cast(float64) text.count * 8 / best / 1000000, text.count, total / 8);
}
