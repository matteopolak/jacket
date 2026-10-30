// A struct whose layout depends on the OS cannot cross from compile-time code (compiled for the
// host) into a program for another OS: `jaic -check -os windows` on a Mac reports the two sizes.

Odd :: struct {
    a: s32;
    #if OS == .WINDOWS {
        b: s64;
    }
    c: s32;
}

make_odd :: () -> Odd {
    odd: Odd;
    odd.a = 1;
    odd.c = 3;
    return odd;
}

ODD :: #run make_odd();

main :: () {}
