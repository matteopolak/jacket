// srgb_to_linear and linear_to_srgb match the piecewise sRGB formula at fixed points, for scalars and vectors.
#import "Basic";
#import "Math";
S :: #import "Srgb";

near :: (a: float64, b: float64, tolerance: float64) {
    assert(abs(a - b) <= tolerance);
}

main :: () {
    // Expected values from the sRGB formula with 9 digits.
    near(S.srgb_to_linear(0.5), 0.214041140, 0.000001);
    near(S.srgb_to_linear(0.25), 0.050876088, 0.000001);
    near(S.srgb_to_linear(0.75), 0.522521554, 0.000001);
    near(S.srgb_to_linear(0.1), 0.010022826, 0.000001);
    near(S.linear_to_srgb(0.5), 0.735356983, 0.000001);
    near(S.linear_to_srgb(0.25), 0.537098730, 0.000001);
    near(S.linear_to_srgb(0.2), 0.484529204, 0.000001);

    // Linear segments: below 0.04045 sRGB is c / 12.92, below 0.0031308 linear is 12.92 * c.
    near(S.srgb_to_linear(0.02), cast(float) (0.02 / 12.92), 0.0000001);
    near(S.linear_to_srgb(0.001), 0.01292, 0.0000001);

    // Endpoints.
    assert(S.srgb_to_linear(0.0) == 0);
    near(S.srgb_to_linear(1.0), 1, 0.000001);
    assert(S.linear_to_srgb(0.0) == 0);
    near(S.linear_to_srgb(1.0), 1, 0.000001);

    // Both curves increase with their input.
    prev_linear: float = -1;
    prev_encoded: float = -1;

    for i: 0..1000 {
        x := cast(float) i / 1000;

        assert(S.srgb_to_linear(x) > prev_linear || i == 0);
        assert(S.linear_to_srgb(x) > prev_encoded || i == 0);
        prev_linear = S.srgb_to_linear(x);
        prev_encoded = S.linear_to_srgb(x);
    }

    // Vector forms apply the curve to each of x, y and z.
    encoded := Vector3.{ 0.1, 0.25, 0.75 };
    linear := S.srgb_to_linear(encoded);

    near(linear.x, 0.010022826, 0.000001);
    near(linear.y, 0.050876088, 0.000001);
    near(linear.z, 0.522521554, 0.000001);

    back := S.linear_to_srgb(Vector3.{ 0.5, 0.25, 0.2 });

    near(back.x, 0.735356983, 0.000001);
    near(back.y, 0.537098730, 0.000001);
    near(back.z, 0.484529204, 0.000001);

    // Vector4 converts the first three channels and passes w through unchanged.
    rgba := Vector4.{ 0.5, 0.25, 0.75, 0.37 };
    out := S.srgb_to_linear(rgba);

    near(out.x, 0.214041140, 0.000001);
    near(out.y, 0.050876088, 0.000001);
    near(out.z, 0.522521554, 0.000001);
    assert(out.w == 0.37);

    print("ok\n");
}
