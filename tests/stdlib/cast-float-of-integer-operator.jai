// rules: cast.29
// A float cast's target type does not reach the operands of `&`, `|`, `^` and `%`, which floats
// lack: the operands keep their integer types and the result is converted (jaigen seeds 62030 and
// 62063 failed with "operator BitAnd is not defined for float32").
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
    // `-` is defined on floats, so there the target still types the literal.
    g := cast(float32) (0 - w);
    assert(g == -15);
    print("ok\n");
}
