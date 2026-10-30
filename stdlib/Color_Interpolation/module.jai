// Fills an array with colors blended between keyframes, in RGB, HSV or Oklab.
//
// A `Color_Keyframe` pairs a color with a position `t` in [0, 1]; `generate_color_samples` spreads `n` output samples evenly over [0, 1] and blends the two keys around each. RGB and Oklab blending works on colors linearized with a plain gamma of 2.2 (`GAMMA`), not the exact sRGB curve, and encodes the result the same way; HSV blends hue, saturation and value directly, taking the shorter way around the hue circle. Outside the first and last key the nearest key's color holds, unless `wrapping` is set, which blends the last key into the first across the 1 to 0 seam.
//
// ```jai
// #import "Math";
// first := Color_Keyframe.{ .{ 0, 0, 0, 1 }, 0 };
// last := Color_Keyframe.{ .{ 1, 1, 1, 1 }, 1 };
// keys: [2] *Color_Keyframe = .[*first, *last];
// samples: [3] Vector4;
// generate_color_samples(keys, .{ .RGB, .LINEAR, false }, samples);
// assert(samples[0] == xyzw(0, 0, 0, 1) && samples[2] == xyzw(1, 1, 1, 1));
// assert(samples[1].x > 0.7 && samples[1].x < 0.75); // the midpoint, blended in linear light and encoded back
// ```

// The gamma, 2.2, used to turn the encoded colors of the keys into linear light for blending in the RGB and Oklab spaces and back again. It is a power law, a little different from the piecewise sRGB curve of the `Srgb` module.
GAMMA :: Impl.GAMMA;
// How `generate_color_samples` blends: the `color_space` the blend happens in (`.RGB`, `.HSV` or `.OKLAB`), the `geometry` that shapes the blend between two keys (`.LINEAR`, `.QUADRIC` for an ease in and out, `.HERMITE` for a smoothstep) and `wrapping`, which blends the last key into the first across the seam instead of holding the end colors. The default value is RGB, linear, not wrapping.
//
// ```jai
// settings := Interpolation.{ .OKLAB, .HERMITE, true };
// assert(settings.color_space == .OKLAB && settings.wrapping);
// assert(Interpolation.{}.geometry == .LINEAR && !Interpolation.{}.wrapping);
// ```
Interpolation :: Impl.Interpolation;
// One key of a gradient: `color_rgb` is a red, green, blue, alpha color with channels in [0, 1] (encoded like an ordinary image color, not linear light) and `t` its position in [0, 1]. A list of keys must be sorted by `t` ascending. Two keys at the same position make a hard step. Alpha is always blended linearly, whatever the color space.
Color_Keyframe :: Impl.Color_Keyframe;
// Converts a red, green, blue color (channels in [0, 1]) to hue in degrees [0, 360), saturation and value in [0, 1]. Takes a `Vector3` and returns a `Vector3`, or takes `r, g, b` and returns `h, s, v`. Greys (no difference between channels) give hue 0 and black gives saturation 0.
//
// ```jai
// #import "Math";
// assert(rgb_to_hsv(xyz(1, 0, 0)) == xyz(0, 1, 1));
// h, s, v := rgb_to_hsv(0, 0, 0.5);
// assert(h == 240 && s == 1 && v == 0.5);
// assert(rgb_to_hsv(xyz(0.5, 0.5, 0.5)) == xyz(0, 0, 0.5));
// ```
rgb_to_hsv :: Impl.rgb_to_hsv;
// Converts hue in degrees, saturation and value (both in [0, 1]) to red, green, blue. Takes a `Vector3` and returns a `Vector3`, or takes `h, s, v` and returns `r, g, b`. The hue wraps, so -60 and 300 are the same, and a saturation of 0 gives a grey of the given value.
//
// ```jai
// #import "Math";
// assert(hsv_to_rgb(xyz(120, 1, 1)) == xyz(0, 1, 0));
// r, g, b := hsv_to_rgb(0, 0, 0.25);
// assert(r == 0.25 && g == 0.25 && b == 0.25);
// assert(hsv_to_rgb(xyz(-60, 1, 1)) == hsv_to_rgb(xyz(300, 1, 1)));
// ```
hsv_to_rgb :: Impl.hsv_to_rgb;
// Converts a linear-light sRGB color to Oklab: `x` is lightness (0 for black, about 1 for white), `y` and `z` the green-red and blue-yellow axes. The input must be linear, not gamma-encoded; see `Srgb.srgb_to_linear`. The transform runs in `float64` and rounds to `float` at the end.
//
// ```jai
// #import "Math";
// white := linear_srgb_to_oklab(xyz(1, 1, 1));
// assert(abs(white.x - 1) < 0.001 && abs(white.y) < 0.001 && abs(white.z) < 0.001);
// ```
linear_srgb_to_oklab :: Impl.linear_srgb_to_oklab;
// Converts an Oklab color back to linear-light sRGB. Colors outside the sRGB gamut are clamped into [0, 1] per channel (a simple clip, not a gamut mapping), so a round trip of an out-of-gamut Oklab color does not return it.
//
// ```jai
// #import "Math";
// c := oklab_to_linear_srgb(linear_srgb_to_oklab(xyz(0.2, 0.5, 0.8)));
// assert(abs(c.x - 0.2) < 0.001 && abs(c.y - 0.5) < 0.001 && abs(c.z - 0.8) < 0.001);
// ```
oklab_to_linear_srgb :: Impl.oklab_to_linear_srgb;
// Raises each component of a `Vector3` to `power`: used to apply and remove `GAMMA`. A negative component with a non-whole power gives NaN.
pow :: Impl.pow;
// The quadratic ease in and out of `t` in [0, 1]: slow at both ends, 0.5 at the middle, 0 at 0 and 1 at 1. This is the `.QUADRIC` shape.
//
// ```jai
// assert(easeInOutQuad(0) == 0 && easeInOutQuad(1) == 1 && easeInOutQuad(0.5) == 0.5);
// assert(easeInOutQuad(0.25) == 0.125);
// ```
easeInOutQuad :: Impl.easeInOutQuad;
// Interpolates from `a` to `b` along a smoothstep `3t^2 - 2t^3` of `t`, so the change starts and ends gently. Takes `float` or `Vector3`; `t` is expected in [0, 1] and is not clamped.
//
// ```jai
// assert(hermite(0.0, 10.0, 0.5) == 5);
// assert(hermite(0.0, 10.0, 0.25) == 1.5625);
// ```
hermite :: Impl.hermite;
// Prepares the key colors for blending in `space`: gamma-decoded to linear light for RGB, Oklab-converted from that for Oklab, and as hue, saturation, value for HSV. Returns one `Vector3` per key (alpha is read from the keys later). The array is allocated with `context.allocator` and the caller frees it with `array_free`.
//
// `generate_color_samples` calls this for you; use it directly with `interpolate_one_sample` to compute single samples.
make_colors :: Impl.make_colors;
// Blends one sample at position `t`. `sorted` are the keys (by `t` ascending), `temp_colors` the result of `make_colors` for the same keys and space, and `i0` the index of the last key at or before `t`; use -1 for a `t` before the first key when `wrapping`. Returns the red, green, blue, alpha color, encoded again. With a single key that key's color is returned. Asserts when there are no keys or the two arrays differ in length.
//
// Keys at the same position make a step; outside the keys the nearest key's color holds, with the blend factor clamped to [0, 1].
interpolate_one_sample :: Impl.interpolate_one_sample;
// Fills `output_samples` with a gradient through the keys: sample `i` of `n` is at `t = i / (n - 1)`, so the first sample is at 0 and the last at 1. `sorted` must be sorted by `t` ascending. Nothing is written when there are no samples or no keys; a single sample gets the first key's color unchanged; an unsupported color space prints a message to standard error and writes nothing.
//
// The temporary colors are allocated with `context.allocator` and freed before returning; the caller owns `output_samples`.
//
// ```jai
// #import "Math";
// a := Color_Keyframe.{ .{ 1, 0, 0, 1 }, 0 };
// b := Color_Keyframe.{ .{ 0, 0, 1, 0 }, 1 };
// keys: [2] *Color_Keyframe = .[*a, *b];
// samples: [3] Vector4;
// generate_color_samples(keys, .{ .HSV, .LINEAR, false }, samples);
// assert(samples[0] == xyzw(1, 0, 0, 1)); // the key colors
// assert(abs(samples[1].w - 0.5) < 0.0001); // alpha blends linearly
// assert(samples[2].z > 0.99 && samples[2].w == 0);
// ```
generate_color_samples :: Impl.generate_color_samples;

#scope_file
Impl :: #import "internal/numerics/color";
