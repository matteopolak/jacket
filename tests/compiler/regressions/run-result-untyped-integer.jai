// An integer a `#run` computes by executing code is an untyped integer constant, like a
// literal: it takes the type of the other operand or the target when its value fits.
// Typed constants and `#run` of a constant expression keep their type.
#import "Basic";

get_size :: () -> s64 #compile_time {
    largest: s64 = 16;
    return largest;
}

five :: () -> s64 {
    return 5;
}

five_u64 :: () -> u64 {
    return 5;
}

small :: () -> u8 {
    return 9;
}

LARGEST :: #run get_size();
C :: #run five();
D :: #run five_u64();
E :: C * 2;
S :: #run small();

main :: () {
    memory: *void = alloc(16);
    start := memory;
    slot: u64 = 3;

    memory += (slot - 1) * LARGEST;
    assert(cast(*u8) memory == cast(*u8) start + 32);
    assert(LARGEST == 16);

    a: u64 = C;
    b: u8 = C;
    c: u8 = D;

    assert(a == 5 && b == 5 && c == 5);

    narrow: u32 = 3;
    x := C * narrow;
    y := C + narrow;

    assert(type_of(x) == u32 && x == 15);
    assert(type_of(y) == u32 && y == 8);

    // Derived constants stay untyped.
    d: u8 = E;
    z := narrow + E;

    assert(d == 10 && type_of(z) == u32 && z == 13);

    // The default type is the type the run produced.
    s := S;

    assert(type_of(s) == u8 && s == 9);

    print("ok\n");
}
