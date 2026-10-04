#import "Basic";

pick :: (small: bool, a: u8, b: s32) -> s32 {
    return ifx small then a else b;
}

main :: () {
    assert(pick(true, 200, -5) == 200);
    assert(pick(false, 200, -5) == -5);
    x: float64 = ifx true then cast(s32) 3 else 2.5;
    assert(x == 3);
    print("ok\n");
}
