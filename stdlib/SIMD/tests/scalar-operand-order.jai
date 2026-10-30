#import "Basic";
S :: #import "SIMD";

// Scalar and vector in both orders, for every lane type: `-` and `/` are not commutative.
exercise :: ($T: Type) {
    a := S.set(4, cast(T) 0);
    a.data = .[2, 3, 4, 6];

    for index: 0..3 {
        v := a.data[index];

        assert((a - cast(T) 1).data[index] == v - 1, "vector - scalar");
        assert((cast(T) 12 - a).data[index] == 12 - v, "scalar - vector");
        assert((a / cast(T) 2).data[index] == v / 2, "vector / scalar");
        assert((cast(T) 12 / a).data[index] == 12 / v, "scalar / vector");
        assert((a + cast(T) 1).data[index] == v + 1 && (cast(T) 1 + a).data[index] == v + 1, "add");
        assert((a * cast(T) 2).data[index] == v * 2 && (cast(T) 2 * a).data[index] == v * 2, "multiply");
    }
}

main :: () {
    exercise(u8);
    exercise(s8);
    exercise(u16);
    exercise(s16);
    exercise(u32);
    exercise(s32);
    exercise(u64);
    exercise(s64);
    exercise(float32);
    exercise(float64);

    a := S.set(4, cast(s32) 3);
    b := cast(s32) 10 - a;
    c := cast(s32) 12 / a;

    assert(b.data[0] == 7 && c.data[3] == 4);
    print("ok\n");
}
