#import "Basic";
S :: #import "Srgb";
#import "Math";

near :: (a: float, b: float, tolerance: float) {
    assert(abs(a - b) < tolerance);
}

main :: () {
    near(S.srgb_to_linear(0.0), 0, 0.000001);
    near(S.srgb_to_linear(0.04045), 0.0031308, 0.000001);
    near(S.srgb_to_linear(0.5), 0.21404114, 0.000001);
    near(S.linear_to_srgb(0.0031308), 0.0404499, 0.000001);
    near(S.linear_to_srgb(1.0), 1, 0.000001);

    for i: 0..1000 {
        x := cast(float) i / 1000;
        near(S.linear_to_srgb(S.srgb_to_linear(x)), x, 0.000002);
        near(S.srgb_to_linear_fast(x), S.srgb_to_linear(x), 0.002);
        near(S.linear_to_srgb_fast(x), S.linear_to_srgb(x), 0.003);
    }

    c := Vector4.{ 0.1, 0.5, 0.9, 0.37 };

    assert(S.srgb_to_linear(c).w == c.w);
    assert(S.srgb_to_linear_fast(c).w == c.w);
    assert(S.srgb_to_linear_pow2(c).w == c.w);
    assert(S.linear_to_srgb(c).w == c.w);
    assert(S.linear_to_srgb_fast(c).w == c.w);
    assert(S.linear_to_srgb_sqrt(c).w == c.w);
    print("ok\n");
}
