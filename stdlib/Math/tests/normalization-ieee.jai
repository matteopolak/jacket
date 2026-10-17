#import "Basic";
M :: #import "Math";

Bits :: union {
    f: float32;
    u: u32;
}

main :: () {
    patterns: [6] u32 = .[1, 3, 0x007fffff, 0x00800000, 0x3f800000, 0x7f7fffff];
    for pattern: patterns {
        bits: Bits;
        bits.u = pattern;
        v := M.xy(bits.f, bits.f);
        n := M.normalize(*v);

        assert(n > 0);
        assert(M.abs(cast(float64) v.x - 0.7071067811865476) < 0.00000002, "normalization must not divide by a rounded/overflowed binary32 length");
        assert(M.abs(cast(float64) v.y - 0.7071067811865476) < 0.00000002);
    }

    v3 := M.unit_vector(M.xyz(M.FLOAT32_MAX, M.FLOAT32_MAX, M.FLOAT32_MAX));

    assert(M.abs(cast(float64) v3.x - 0.5773502691896258) < 0.00000002);
    v4 := M.normalize(M.xyzw(M.FLOAT32_MAX, M.FLOAT32_MAX, M.FLOAT32_MAX, M.FLOAT32_MAX));

    assert(v4.x == 0.5 && v4.y == 0.5 && v4.z == 0.5 && v4.w == 0.5);
    q := M.Quaternion.{ M.FLOAT32_MAX, M.FLOAT32_MAX, M.FLOAT32_MAX, M.FLOAT32_MAX };
    original := M.normalize_or_identity(*q);

    assert(M.isinf(original) && q.x == 0.5 && q.w == 0.5);
    tiny: Bits;
    tiny.u = 1;
    v := M.xy(tiny.f, tiny.f);
    fallback := M.xy(7, 8);
    n := M.normalize(*v, epsilon = tiny.f, fallback = fallback);

    assert(n == tiny.f && v.x == 7 && v.y == 8, "epsilon compares the returned binary32 length");
    negative_zero: Bits;
    negative_zero.u = 0x80000000;
    v = M.unit_vector(M.xy(negative_zero.f, negative_zero.f));
    negative_zero.f = v.x;
    assert(negative_zero.u == 0x80000000, "unit_vector must leave a zero vector unchanged");
    assert(M.isinf(M.length(M.xy(M.FLOAT32_INFINITY, 1))));
    assert(M.isnan(M.length(M.xy(M.FLOAT32_INFINITY, M.FLOAT32_NAN))));
    print("IEEE normalization ok\n");
}
