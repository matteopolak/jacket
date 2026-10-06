// `cast(bool)` of a number, enum or pointer tests for non-zero: the result is a real
// `true`, so `!` and `==` treat it like any other bool (it used to keep the low byte, and
// `!cast(bool) 2` was true).
// rules: cast.7 cast.22
// jailint: allow-file(bool_comparison) -- this test checks the exact value a cast to bool gives.
#import "Basic";

Flags :: enum_flags {
    A;
    B;
}

main :: () {
    two := 2;
    b := cast(bool) two;
    assert(b && !(!b) && b == true);
    assert(!cast(bool) 0 == true);
    big: u32 = 0x100;
    assert(cast(bool) big);
    flags := Flags.B;
    on := cast(bool) flags;
    assert(!(!on));
    words: [2] u64;
    p := *words[1];
    assert(cast(bool) p == true);
    half: float = 0.5;
    assert(cast(bool) half);
    print("ok\n");
}
