// Srgb: conversion between sRGB-encoded and linear-light color values.
//
// sRGB is the encoding of most images and displays: it spends more of its range on dark values than linear light does, so blend, filter and light in linear space and encode on output. Each conversion comes as a `float` form, a `Vector3` (red, green, blue) form and a `Vector4` form that leaves alpha as it is. The exact curves are `srgb_to_linear` and `linear_to_srgb`; the `_fast` forms use a table and the `_pow2` and `_sqrt` forms a gamma of 2.0, both cheaper and less accurate. Values outside [0, 1] are not clamped.
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
