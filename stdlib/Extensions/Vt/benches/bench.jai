// Run with jacket bench --path stdlib/Extensions/Vt.
#import "Extensions/Testing";
#import "Basic";
#import "Extensions/Vt";

count_action :: (data: *void, action: Vt_Action) {
    total := cast(*u64) data;
    total.* += 1;
}

mixed_stream :: (b: *Bench) {
    builder: String_Builder;

    defer free_buffers(*builder);
    for 0..16383 append(*builder, "building package: hello world\r\n\x1b[1;32mPASS\x1b[0m\x1b[12;40Hnext\x1b[2D");

    text := builder_to_string(*builder);

    defer free(text);
    set_bytes(b, text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        total: u64;
        p: Vt_Parser;
        error := vt_feed(*p, black_box(text), count_action, *total);

        assert(error.kind == .NONE);
        black_box(total);
    }
} @bench
