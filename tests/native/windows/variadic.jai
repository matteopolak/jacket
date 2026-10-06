// Variadic C calls with floats and more arguments than registers, through `sprintf`, which
// msvcrt.dll exports on every Windows CPU (UCRT's `snprintf` is a header inline, so the
// interpreter cannot find it there). Windows on arm64 passes all of a variadic call's
// arguments, doubles included, in x0-x7 and then on the stack; Win64 duplicates the first
// four doubles in integer registers. Run natively by tools/windows_cross.py and with
// `jaic run` in .github/workflows/windows-native.yml.
#import "Basic";

libc :: #system_library "libc";
sprintf :: (buf: *u8, fmt: *u8, args: ..Any) -> s32 #foreign libc;

expect :: (got: s32, buf: [] u8, want: string) {
    text := to_string(buf.data, got);
    assert(text == want, "sprintf gave '%', expected '%'", text, want);
}

main :: () {
    buf: [256] u8;
    n := sprintf(buf.data, "%.2f", 1.25);
    expect(n, buf, "1.25");

    n = sprintf(buf.data, "%d %.1f %s %.3f", cast(s32) 7, 0.5, "mid".data, 2.125);
    expect(n, buf, "7 0.5 mid 2.125");

    // Eleven variadic arguments: the last ones are on the stack whichever registers they
    // would otherwise use.
    n = sprintf(buf.data, "%.1f %d %.1f %d %.1f %d %.1f %d %.1f %lld %s", 1.5, cast(s32) 2, 3.5, cast(s32) 4, 5.5, cast(s32) 6, 7.5, cast(s32) 8, 9.5, cast(s64) -10, "end".data);
    expect(n, buf, "1.5 2 3.5 4 5.5 6 7.5 8 9.5 -10 end");

    print("ok\n");
}
