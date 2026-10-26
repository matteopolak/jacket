// Color_Interpolation policies: geometry, wrapping, degenerate keys and every color space.
#import "Basic";
#import "Math";
#import "Extensions/Testing";
C :: #import "Color_Interpolation";

Space :: C.Interpolation.color_space;
Geometry :: C.Interpolation.geometry;

Sample :: struct {
    geometry: Geometry;
    want: float32;
}

// The alpha halfway between keys at 0.25 and 0.75 that go from 0 to 1.
SAMPLES :: Sample.[
    .{ .LINEAR, 0.25 },
    .{ .QUADRIC, 0.125 },
    .{ .HERMITE, 0.15625 },
];

geometry_shapes_the_blend :: (geometry: Geometry, want: float32) {
    a := C.Color_Keyframe.{ .{ 0, 0, 0, 0 }, 0.25 };
    b := C.Color_Keyframe.{ .{ 1, 1, 1, 1 }, 0.75 };
    keys: [2] *C.Color_Keyframe = .[*a, *b];
    colors := C.make_colors(keys, .RGB);

    defer array_free(colors);
    policy := C.Interpolation.{ .RGB, geometry, false };
    sample := C.interpolate_one_sample(0, 0.375, policy, keys, colors);

    expect_near(sample.w, want, 0.000001);
} @test @cases(SAMPLES)

samples_hold_the_end_colors_outside_the_keys :: () {
    a := C.Color_Keyframe.{ .{ 0, 0, 0, 0 }, 0.25 };
    b := C.Color_Keyframe.{ .{ 1, 1, 1, 1 }, 0.75 };
    keys: [2] *C.Color_Keyframe = .[*a, *b];
    samples: [5] Vector4;

    C.generate_color_samples(keys, .{ .RGB, .LINEAR, false }, samples);
    expect(samples[0].w == 0 && samples[4].w == 1);
    expect(samples[1].w == 0 && samples[3].w == 1);

    C.generate_color_samples(keys, .{ .RGB, .LINEAR, true }, samples);
    expect(samples[0].w == 0.5 && samples[4].w == 0.5);
} @test

keys_at_the_same_place_make_a_step :: () {
    a := C.Color_Keyframe.{ .{ 0, 0, 0, 0 }, 0.5 };
    b := C.Color_Keyframe.{ .{ 1, 1, 1, 1 }, 0.5 };
    keys: [2] *C.Color_Keyframe = .[*a, *b];
    samples: [5] Vector4;
    empty: [] Vector4;

    C.generate_color_samples(keys, .{ .RGB, .LINEAR, false }, samples);
    expect(samples[0].w == 0 && samples[2].w == 0 && samples[4].w == 0.5);

    // No samples asked for, none written.
    C.generate_color_samples(keys, .{ .RGB, .LINEAR, false }, empty);
} @test

one_key_fills_every_sample :: () {
    a := C.Color_Keyframe.{ .{ 0.2, 0.4, 0.6, 0.8 }, 0.5 };
    single: [1] *C.Color_Keyframe = .[*a];
    output: [3] Vector4;

    C.generate_color_samples(single, .{ .RGB, .HERMITE, true }, output);

    for sample: output expect(abs(sample.x - 0.2) < 0.000001 && sample.w == 0.8);
} @test

oklab_round_trips :: () {
    rgb := xyz(0.2, 0.4, 0.6);
    back := C.oklab_to_linear_srgb(C.linear_srgb_to_oklab(rgb));

    expect(distance(rgb, back) < 0.000001);

    low := C.oklab_to_linear_srgb(xyz(-1, 0, 0));

    expect(low.x == 0 && low.y == 0 && low.z == 0);
} @test

// Outside the keys the end colors hold, in every color space.
end_colors_hold :: (space: Space) {
    a := C.Color_Keyframe.{ .{ 0.2, 0.4, 0.6, 0 }, 0.25 };
    b := C.Color_Keyframe.{ .{ 0.8, 0.5, 0.1, 1 }, 0.75 };
    keys: [2] *C.Color_Keyframe = .[*a, *b];
    edges: [5] Vector4;

    C.generate_color_samples(keys, .{ space, .LINEAR, false }, edges);

    expect(edges[0] == edges[1] && edges[3] == edges[4]);
    expect(edges[0].w == 0 && edges[4].w == 1);

    for edges expect(it.x == it.x && it.y == it.y && it.z == it.z, "a component is NaN");
} @test @cases(all_values(Space))
