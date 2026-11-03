// Conversion between sRGB-encoded and linear-light color values.
//
// sRGB spends more of its range on dark values than linear light does, so blend, filter and light in linear space and encode on output.
//
// [srgb_to_linear] and [linear_to_srgb] are the exact curves, each for a `float`, a `Vector3` and a `Vector4` (alpha is left alone). The `_fast` forms use a table, and the `_pow2` and `_sqrt` forms a gamma of 2.0; both are cheaper and less accurate.
//
// ```jai
// #import "Math";
// encoded := xyz(0.5, 0.5, 0.5);
// linear := srgb_to_linear(encoded);
// assert(abs(linear.x - 0.214) < 0.001);
// back := linear_to_srgb(linear);
// assert(abs(back.x - 0.5) < 0.00001);
// ```

#load "../internal/rendering/srgb.jai";

#scope_module
#import "Math";
