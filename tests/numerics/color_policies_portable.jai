// Retains every Color_Interpolation policy assertion without the unrelated native GetRect hash check.
#import "Basic";
#import "Math";
C :: #import "Color_Interpolation";

main :: () {
    a := C.Color_Keyframe.{ .{ 0, 0, 0, 0 }, 0.25 };
    b := C.Color_Keyframe.{ .{ 1, 1, 1, 1 }, 0.75 };
    keys: [2] *C.Color_Keyframe = .[*a, *b];
    colors := C.make_colors(keys, .RGB);

    defer array_free(colors);
    linear := C.Interpolation.{ .RGB, .LINEAR, false };
    sample := C.interpolate_one_sample(0, 0.375, linear, keys, colors);

    assert(abs(sample.w - 0.25) < 0.000001);
    linear.geometry = .QUADRIC;
    sample = C.interpolate_one_sample(0, 0.375, linear, keys, colors);
    assert(abs(sample.w - 0.125) < 0.000001);
    linear.geometry = .HERMITE;
    sample = C.interpolate_one_sample(0, 0.375, linear, keys, colors);
    assert(abs(sample.w - 0.15625) < 0.000001);
    samples: [5] Vector4;

    linear.geometry = .LINEAR;
    C.generate_color_samples(keys, linear, samples);
    assert(samples[0].w == -0.5 && samples[4].w == 1.5);
    assert(samples[1].w == 0 && samples[3].w == 1);
    linear.wrapping = true;
    C.generate_color_samples(keys, linear, samples);
    assert(samples[0].w == 0.5 && samples[4].w == 0.5);
    a.t = 0.5;
    b.t = 0.5;
    linear.wrapping = false;
    C.generate_color_samples(keys, linear, samples);
    assert(samples[0].w == -0.5 && samples[2].w == 0 && samples[4].w == 0.5);
    empty: [] Vector4;

    C.generate_color_samples(keys, linear, empty);
    a.color_rgb = .{ .2, .4, .6, .8 };
    single: [1] *C.Color_Keyframe = .[*a];
    output: [3] Vector4;

    C.generate_color_samples(single, .{ .RGB, .HERMITE, true }, output);
    for sample: output assert(abs(sample.x - .2) < 0.000001 && sample.w == .8);

    rgb := xyz(.2, .4, .6);
    lab := C.linear_srgb_to_oklab(rgb);
    back := C.oklab_to_linear_srgb(lab);

    assert(distance(rgb, back) < 0.000001);
    low := C.oklab_to_linear_srgb(xyz(-1, 0, 0));

    assert(low.x == 0 && low.y == 0 && low.z == 0);
    print("portable color policies ok\n");
}
