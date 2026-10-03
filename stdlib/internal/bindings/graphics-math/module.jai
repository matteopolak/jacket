// Packed Direct3D arithmetic from the pinned public Microsoft SDK macros.
encode_components :: (a: s32, b: s32, c: s32, d: s32) -> s32 {
    return (a & 7) | ((b & 7) << 3) | ((c & 7) << 6) | ((d & 7) << 9) | 4096;
}

decode_component :: (index: s32, mapping: s32) -> s32 {
    return (mapping >> (3 * index)) & 7;
}

encode_filter :: (minimum: s32, magnification: s32, mip: s32, reduction: s32) -> s32 {
    return ((minimum & 3) << 4) | ((magnification & 3) << 2) | (mip & 3) | ((reduction & 3) << 7);
}

encode_anisotropic :: (reduction: s32) -> s32 {
    return 64 | encode_filter(1, 1, 1, reduction);
}

decode_minimum :: (filter: s32) -> s32 {
    return (filter >> 4) & 3;
}

decode_magnification :: (filter: s32) -> s32 {
    return (filter >> 2) & 3;
}

decode_mip :: (filter: s32) -> s32 {
    return filter & 3;
}

decode_reduction :: (filter: s32) -> s32 {
    return (filter >> 7) & 3;
}

is_comparison :: (filter: s32) -> bool {
    return decode_reduction(filter) == 1;
}

is_anisotropic :: (filter: s32) -> bool {
    return (filter & 64) != 0 && decode_minimum(filter) == 1 && decode_magnification(filter) == 1;
}
