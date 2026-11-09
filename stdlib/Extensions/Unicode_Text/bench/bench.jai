// Compare with Python on the same repeated corpus; see docs/extensions/unicode-text.md.
#import "Basic";
#import "Extensions/Unicode_Text";

main :: () {
    source: String_Builder;

    defer free_buffers(*source);
    for 1..32768 append(*source, "Café Καλημέρα 한글 👩‍👩‍👧‍👦\n");

    text := builder_to_string(*source);

    defer free(text);
    checksum := 0;

    for mode: 0..1 {
        best: float64 = 1.0e30;

        for 1..5 {
            start := current_time_monotonic();

            if mode == 0 {
                b: String_Builder;

                defer free_buffers(*b);
                error := normalize(text, .NFC, *b);

                assert(error.kind == .NONE);
                out := builder_to_string(*b);

                checksum += out.count;
                free(out);
            } else {
                iter := segments(text, .GRAPHEME);

                while true {
                    _, end, more := next_segment(*iter);
                    if !more break;
                    checksum += end;
                }

                assert(iter.error.kind == .NONE);
            }

            elapsed, _ := to_float64_seconds(current_time_monotonic() - start);

            best = min(best, elapsed);
        }

        print("%: % MB/s (% bytes, % s best of 5)\n", ifx mode == 0 then "NFC" else "graphemes", (cast(float64) text.count) / best / 1.0e6, text.count, best);
    }

    print("checksum %\n", checksum);
}
