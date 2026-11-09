#import "Basic";
#import "String";
#import "Extensions/Testing";
#import "Extensions/Vt";

#import,dir "support";

all_chunk_boundaries :: () {
    streams := string.["hello界é\x1b[?38:2::255;1m\x1b(0", "\x1b]2;a;b\x07\x1bP1$qabc\x1b\\", "\xf0\x9f\x8e\x89\xd8\x1b012", "\x1b[1 $bad\x1bP1:2q\x00x\x18A\x1b_ignored\x1b\\"];
    for bytes: streams {
        whole := trace(bytes, 0);

        defer free(whole);

        for split: 0..bytes.count {
            chunked := trace(bytes, split);
            defer free(chunked);
            expect_equal(chunked, whole);
        }
    }
} @test

bounded_and_recovery :: () {
    p: Vt_Parser;
    p.options.max_params = 2;
    p.options.max_osc_bytes = 3;
    c: Capture;
    error := vt_feed(*p, "\x1b[1;2;3mA", record, *c);

    expect_equal(error.kind, Vt_Error_Kind.LIMIT);
    expect_equal(error.offset, cast(u64) 5);
    expect_equal(p.state, Vt_State.GROUND);
    error = vt_feed(*p, "\x1b]abcd\x07B", record, *c);
    expect_equal(error.kind, Vt_Error_Kind.LIMIT);
    text := builder_to_string(*c.text);

    defer free(text);
    expect_equal(text, "PRINT:65[]?0|PRINT:66[]?0|");
} @test

utf8_replacement_and_finish :: () {
    text := trace("\xef\xbcZ\xd8\x1b0\xf0\x9f", 2);
    defer free(text);
    expect_equal(text, "PRINT:65533[]?0|PRINT:90[]?0|PRINT:65533[]?0|ESC_DISPATCH:48[]?0|PRINT:65533[]?0|");
} @test

c1_is_opt_in_and_utf8_continuations_win :: () {
    text := trace("\xc2\x9b\x9b2m", 1, .{ c1 = true });

    defer free(text);
    expect_equal(text, "PRINT:155[]?0|CSI_DISPATCH:109/2:false[]?0|");
    disabled := trace("\x9b2m", 0);

    defer free(disabled);
    expect_equal(disabled, "PRINT:65533[]?0|PRINT:50[]?0|PRINT:109[]?0|");
} @test
