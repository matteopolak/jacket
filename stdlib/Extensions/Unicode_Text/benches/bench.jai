// Run with jacket bench --path stdlib/Extensions/Unicode_Text.
#import "Extensions/Testing";
#import "Basic";
#import "Extensions/Unicode_Text";

nfc :: (b: *Bench) {
    source: String_Builder;

    defer free_buffers(*source);
    for 1..32768 append(*source, "Café Καλημέρα 한글 👩‍👩‍👧‍👦\n");

    text := builder_to_string(*source);

    defer free(text);
    set_bytes(b, text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        bld: String_Builder;

        defer free_buffers(*bld);
        error := normalize(black_box(text), .NFC, *bld);

        assert(error.kind == .NONE);
        out := builder_to_string(*bld);

        black_box(out);
        free(out);
    }
} @bench

graphemes :: (b: *Bench) {
    source: String_Builder;

    defer free_buffers(*source);
    for 1..32768 append(*source, "Café Καλημέρα 한글 👩‍👩‍👧‍👦\n");

    text := builder_to_string(*source);

    defer free(text);
    set_bytes(b, text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        iter := segments(black_box(text), .GRAPHEME);

        while true {
            start, end, more := next_segment(*iter);
            if !more break;
            black_box(start);
            black_box(end);
        }

        assert(iter.error.kind == .NONE);
    }
} @bench
