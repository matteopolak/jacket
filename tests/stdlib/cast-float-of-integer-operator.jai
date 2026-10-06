// rules: cast.29 cast.31
// The operands of `&`, `|`, `^` and `%` under a float cast keep their integer types and only the
// result is converted (jaigen seeds 62030 and 62063 failed with "operator BitAnd is not defined
// for float32"). The same holds when the float type comes from a declaration.
#import "Basic";

main :: () {
    w: u16 = 15;
    v: u16 = 0xff;
    c := w > 3;
    a := cast(float32) ((ifx c then cast(s16) 6 else cast(s16) 0) & (cast(s16) 7700 ^ cast(s16) 32767));
    assert(a == 2);
    b := cast(float64) ((ifx c then cast(s32) 7 else cast(s32) 1) % cast(s32) 4);
    assert(b == 3);
    // The untyped `0` takes `w`'s type, as it would without the cast.
    d := cast(float32) ((0 - w) & v);
    assert(d == 241);
    e := cast(float32) (3 | 4);
    assert(e == 7);
    // A float declaration does not reach the `ifx` branches either.
    h: float32 = (ifx c then cast(s16) 6 else cast(s16) 0) & cast(s16) 3;
    assert(h == 2);
    print("ok\n");
}
