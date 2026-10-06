// `ifx` with no target type: a narrower integer then-value widens to the else-value's type.
#import "Basic";

Entry :: struct {
    score: s16;
}

evaluate :: (n: int) -> int {
    return n * 100000;
}

main :: () {
    entry := Entry.{ -12 };
    hit := false;
    a := ifx hit then entry.score else evaluate(3);
    #assert type_of(a) == int;
    assert(a == 300000);
    hit = true;
    b := ifx hit then entry.score else evaluate(3);
    assert(b == -12);
    small: u8 = 7;
    c := ifx hit then small else cast(s32) -5;
    #assert type_of(c) == s32;
    assert(c == 7);
    print("ok\n");
}
