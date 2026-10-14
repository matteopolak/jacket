#load "../../stdlib/internal/bindings/graphics-math/module.jai";

probe :: #library "../../.work/native/graphics_macro_probe";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (value: bool) {
    if !value abort();
}

components :: (a: s32, b: s32, c: s32, d: s32) -> s32 #foreign probe "binding_components";
component :: (index: s32, mapping: s32) -> s32 #foreign probe "binding_component";
filter :: (a: s32, b: s32, c: s32, d: s32) -> s32 #foreign probe "binding_filter";
anisotropic :: (reduction: s32) -> s32 #foreign probe "binding_anisotropic";
filter_part :: (value: s32, part: s32) -> s32 #foreign probe "binding_filter_part";
native_is_comparison :: (value: s32) -> s32 #foreign probe "binding_is_comparison";
native_is_anisotropic :: (value: s32) -> s32 #foreign probe "binding_is_anisotropic";

main :: () {
    for a: 0..7 for b : 0..7 for c : 0..7 for d : 0..7 {
        value := encode_components(cast(s32) a, cast(s32) b, cast(s32) c, cast(s32) d);
        check(value == components(cast(s32) a, cast(s32) b, cast(s32) c, cast(s32) d));
        for i: 0..3 check(decode_component(cast(s32) i, value) == component(cast(s32) i, value));
    }

    for a: 0..3 for b : 0..3 for c : 0..3 for d : 0..3 {
        check(encode_filter(cast(s32) a, cast(s32) b, cast(s32) c, cast(s32) d) == filter(cast(s32) a, cast(s32) b, cast(s32) c, cast(s32) d));
    }

    for i: 0..511 {
        value := cast(s32) i;

        check(decode_minimum(value) == filter_part(value, 0));
        check(decode_magnification(value) == filter_part(value, 1));
        check(decode_mip(value) == filter_part(value, 2));
        check(decode_reduction(value) == filter_part(value, 3));
        check(is_comparison(value) == (native_is_comparison(value) != 0));
        check(is_anisotropic(value) == (native_is_anisotropic(value) != 0));
    }

    for i: 0..3 check(encode_anisotropic(cast(s32) i) == anisotropic(cast(s32) i));
}
