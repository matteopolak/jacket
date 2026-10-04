// Build_Options.arithmetic_overflow_check: with the option on, `+`, `-` and `*` on integers that do not fit
// their type are reported (.FATAL stops the program, .NONFATAL reports on stderr and continues with the
// wrapped value); `#no_aoc` on a procedure, a block, or a loop / `if` turns the check off again. The default
// (.OFF) wraps silently. Each case compiles a small program in its own workspace and runs it with `#run`.
// Programs that must fail (.FATAL overflow) are in tests/corpus/negative/aoc-fatal-*.jai.
#import "Basic";
#import "Compiler";

// A program whose `work` procedure runs `body`, after `prefix` (top-level declarations).
program :: (prefix: string, body: string) -> string {
    return tprint("#import \"Basic\";\n%\nwork :: () {\n%\n}\n#run work();\n", prefix, body);
}

// Compile and run `source` in a workspace with the given overflow mode; a failing `assert` or a fatal
// overflow would fail that workspace's compile and so the whole test.
build_with :: (source: string, mode: type_of(Build_Options.arithmetic_overflow_check)) {
    w := compiler_create_workspace();
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.arithmetic_overflow_check = mode;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(source, w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
}

run_cases :: () {
    FATAL :: Build_Options.arithmetic_overflow_check.FATAL;
    NONFATAL :: Build_Options.arithmetic_overflow_check.NONFATAL;
    OFF :: Build_Options.arithmetic_overflow_check.OFF;

    u8_add   := program("", "x: u8 = 250; y: u8 = 10; z := x + y; assert(z == 4);");
    s8_sub   := program("", "x: s8 = -120; y: s8 = 20; z := x - y; assert(z == 116);");
    s32_mul  := program("", "x: s32 = 100000; z := x * x; assert(z == 1410065408);");
    s64_add  := program("", "x: s64 = 0x7fff_ffff_ffff_ffff; y := 1; z := x + y; assert(z < 0);");
    u64_sub  := program("", "x: u64 = 0; y: u64 = 1; z := x - y; assert(z == 0xffff_ffff_ffff_ffff);");
    compound := program("", "x: u16 = 65535; x += 1; assert(x == 0);");
    in_range := program("", "x: u8 = 250; y: u8 = 5; assert(x + y == 255); a: s8 = -128; b: s8 = 1; assert(a + b == -127); assert(a * 1 == -128); assert(b - 2 == -1); u: u64 = 0xffff_ffff_ffff_ffff; assert(u - 1 == 0xffff_ffff_ffff_fffe); assert(u * 1 == u);");

    // Off by default and when asked: arithmetic wraps.
    for source: string.[u8_add, s8_sub, s32_mul, s64_add, u64_sub, compound, in_range] {
        build_with(source, OFF);
    }

    // Values that fit never trip the check.
    build_with(in_range, FATAL);
    build_with(in_range, NONFATAL);

    // Nonfatal: reported on stderr, the program continues and the result wraps.
    for source: string.[u8_add, s8_sub, s32_mul, s64_add, u64_sub, compound] {
        build_with(source, NONFATAL);
    }

    // #no_aoc in every place #no_abc works.
    on_proc  := program("wrap :: (x: u8, y: u8) -> u8 #no_aoc { return x + y; }", "assert(wrap(250, 10) == 4);");
    on_block := program("", "x: u8 = 250; y: u8 = 10; z: u8; #no_aoc { z = x + y; } assert(z == 4);");
    on_for   := program("", "x: u8 = 250; for 1..3 #no_aoc { x += 10; } assert(x == 24);");
    on_while := program("", "x: u8 = 250; i := 0; while i < 2 #no_aoc { x += 10; i += 1; } assert(x == 14);");
    on_if    := program("", "x: u8 = 250; if x > 0 #no_aoc { x += 10; } assert(x == 4);");
    for source: string.[on_proc, on_block, on_for, on_while, on_if] {
        build_with(source, FATAL);
    }
}

#run run_cases();

main :: () {
    // Ordinary arithmetic wraps in a program that did not turn the option on.
    x: u8 = 250;
    y: u8 = 10;
    if x + y != 4 exit(1);
    print("ok\n");
}
