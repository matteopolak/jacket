// Converting to linear and back in either order returns the input within 2e-6, for scalars, Vector3 and Vector4.
#import "Basic";
#import "Math";
S :: #import "Srgb";

near :: (a: float, b: float, tolerance: float) {
    assert(abs(a - b) <= tolerance);
}

main :: () {
    for i: 0..1000 {
        x := cast(float) i / 1000;

        near(S.srgb_to_linear(S.linear_to_srgb(x)), x, 0.000002);
        near(S.linear_to_srgb(S.srgb_to_linear(x)), x, 0.000002);

        c := Vector3.{ x, 1 - x, x / 2 };
        rgb := S.linear_to_srgb(S.srgb_to_linear(c));

        near(rgb.x, c.x, 0.000002);
        near(rgb.y, c.y, 0.000002);
        near(rgb.z, c.z, 0.000002);

        c4 := Vector4.{ x, 1 - x, x / 2, 0.5 };
        back := S.linear_to_srgb(S.srgb_to_linear(c4));

        near(back.x, c4.x, 0.000002);
        near(back.y, c4.y, 0.000002);
        near(back.z, c4.z, 0.000002);
        assert(back.w == 0.5);
    }

    print("ok\n");
}
