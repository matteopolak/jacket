// Build_Options.cast_bounds_check: a cast to a narrower integer type checks that the value fits.
// .FATAL (the default) stops the program, .NONFATAL reports on stderr and continues with the low
// bits, .OFF keeps the low bits silently. `trunc` and `no_check` skip the check, and casts to a
// type at least as wide are never checked. Programs that must fail are in
// tests/corpus/negative/cast-check-*.jai.
// rules: cast.2 cast.32 cast.33 cast.34
#import "Basic";
#import "Compiler";

build_with :: (body: string, mode: type_of(Build_Options.cast_bounds_check)) {
    w := compiler_create_workspace();
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.cast_bounds_check = mode;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(tprint("#import \"Basic\";\nwork :: () {\n%\n}\n#run work();\n", body), w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
}

run_cases :: () {
    // cast.33: off, and nonfatal (a warning on stderr), keep the low bits; a failing assert would
    // fail the workspace's compile and so this test.
    narrowing := "w := 300; c := cast(u8) w; assert(c == 44);";
    signed := "n: s32 = -1; c := cast(u16) n; assert(c == 65535);";
    // cast.34: `xx` to a narrower type is checked like `cast`.
    implicit := "n := 70000; c: s16 = xx n; assert(c == 4464);";
    for body: string.[narrowing, signed, implicit] {
        build_with(body, .OFF);
        build_with(body, .NONFATAL);
    }
}

#run run_cases();

main :: () {
    // Values that fit pass.
    w := 200;
    assert(cast(u8) w == 200);
    m := -128;
    assert(cast(s8) m == -128);
    k: u32 = 65535;
    assert(cast(u16) k == 65535);

    // cast.2: `trunc` and `no_check` keep the low bits without a check, and so does `xx,trunc`.
    big := 300;
    assert(cast,trunc(u8) big == 44);
    assert(cast,no_check(u8) big == 44);
    low: u8 = xx,trunc big;
    assert(low == 44);
    assert(big.(u8, trunc) == 44);

    // cast.32: a type at least as wide only extends or reinterprets the bits.
    minus_one: s8 = -1;
    assert(cast(u64) minus_one == 0xffff_ffff_ffff_ffff);
    all_ones: u64 = 0xffff_ffff_ffff_fff4;
    assert(cast(s64) all_ones == -12);
    negative: s32 = -20;
    assert(cast(u32) negative == 0xffff_ffec);

    print("ok\n");
}
