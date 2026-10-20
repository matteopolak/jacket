// An ifx takes the type of an arm that has one; two untyped arms are an s64 that only a signed
// or floating-point target accepts.
#import "Basic";

pick :: (wide: bool) -> u64 {
    return ifx wide then cast(u64) 1024 else 1000;
}

main :: () {
    wide := true;
    narrow := false;

    typed: u64 = 1000;

    assert(typed == 1000);

    a: u64 = ifx wide then cast(u64) 1024 else 1000;
    b: u64 = ifx narrow then 1 else typed;
    c: s32 = ifx wide then 7 else 9;
    d: float = ifx narrow then 1 else 2;
    e: u8 = ifx wide then cast(u8) 255 else 0;

    small: u8 = 7;
    f: u64 = ifx wide then small else 1000;

    assert(f == 7);
    assert(a == 1024 && b == 1000 && c == 7 && d == 2 && e == 255);
    assert(pick(true) == 1024 && pick(false) == 1000);

    print("ok\n");
}
