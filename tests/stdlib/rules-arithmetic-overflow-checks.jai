// Checks the claims of docs/language/arithmetic-overflow-checks.md not already pinned by
// arithmetic-overflow-check.jai: what the check leaves alone, and the compound operators.
// rules: overflow.1 overflow.7 overflow.8
#import "Basic";
#import "Compiler";

// Compile and run `body` inside `work` in a workspace with .FATAL checks; a reported overflow or a
// failing assert fails the whole compile.
run_fatal :: (prefix: string, body: string) {
    source := tprint("#import \"Basic\";\n%\nwork :: () {\n%\n}\n#run work();\n", prefix, body);
    w := compiler_create_workspace();
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.arithmetic_overflow_check = .FATAL;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(source, w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
}

#run {
    // overflow.8: constant folding still wraps.
    run_fatal("A : u8 : 250;\nB : u8 : 10;", "z := A + B; assert(z == 4);");
    // overflow.8: unary minus is not checked.
    run_fatal("", "x: s8 = -128; y := -x; assert(y == -128);");
    // overflow.8: shifts are not checked.
    run_fatal("", "x: u8 = 200; y := x << 1; assert(y == 144); s: s32 = 0x4000_0000; t := s << 1; assert(t < 0);");
    // overflow.8: pointer arithmetic is not checked.
    run_fatal("", "p := cast(*u8) 0xffff_ffff_ffff_fff0; q := p + 0x20; assert(cast(u64) q == 0x10);");
    // overflow.8: floats are not checked.
    run_fatal("", "f: float32 = 3e38; g := f * 10; assert(g > 3e38);");
    // overflow.8: indexing arithmetic is not checked.
    run_fatal("", "a := int.[5, 6, 7]; i: u8 = 2; assert(a[i] == 7);");

    // overflow.1: compound operators are checked like the plain ones when they fit.
    run_fatal("", "x: u8 = 200; x += 55; x -= 255; x *= 1; y: s16 = -100; y *= 300; assert(x == 0 && y == -30000);");
}

main :: () {
    // overflow.7: the program's own workspace keeps the default (.OFF), so arithmetic wraps.
    x: u8 = 250;
    x += 10;
    assert(x == 4, "wraps without the check");
    y: s8 = 127;
    y *= 2;
    assert(y == -2, "s8 multiply wraps");
    print("ok\n");
}
