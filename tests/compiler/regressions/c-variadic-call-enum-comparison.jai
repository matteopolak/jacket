// A C variadic call compared with an enum literal, passing a local of the calling procedure:
// checking the call to look for an overloaded `==` must not read that local's IR type.
#import "Basic";

libc :: #system_library "libc";

Written :: enum s32 {
    NONE :: 0;
    TWO :: 2;
}

format :: (buf: *u8, n: u64, fmt: *u8, args: ..Any) -> Written #foreign libc "snprintf";

main :: () {
    if OS == .WASM {
        print("ok\n");
        return;
    }

    a, b, c, d, e, f := 1, 2, 3, 4, 5, 6;
    buf: [16] u8;
    digits: [3] u8 = .[#char "4", #char "2", 0];

    assert(format(buf.data, buf.count, "%s", digits.data) == .TWO);
    assert(to_string(buf.data, 2) == "42");
    assert(a + b + c + d + e + f == 21);
}
