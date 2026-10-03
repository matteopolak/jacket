bitmap_build_mipmaps :: (bitmap: *Bitmap) {
    assert(bitmap.width > 0 && bitmap.height > 0 && bitmap.data.data != null);
    assert(bitmap.format == .R8 || bitmap.format == .RG8 || bitmap.format == .RGB8 || bitmap.format == .RGBA8, "mip generation requires an 8-bit R/RG/RGB/RGBA bitmap");
    channels := get_image_bytes_per_texel(bitmap.format);

    assert(bitmap.stride >= bitmap.width * channels);
    assert(bitmap.data.count >= cast(int) (bitmap.height - 1) * bitmap.stride + bitmap.width * channels);
    width := bitmap.width;
    height := bitmap.height;
    total: int;
    levels: s32 = 1;

    while true {
        total += mip_stride(width, channels) * height;
        if width == 1 && height == 1 break;

        width = Basic.max(cast(s32) 1, width / 2);
        height = Basic.max(cast(s32) 1, height / 2);
        levels += 1;
    }

    data := cast(*u8) Basic.alloc(total);

    assert(data != null, "mipmap storage allocation failed");
    memset(data, 0, total);
    base_stride := mip_stride(bitmap.width, channels);
    for row: 0..bitmap.height - 1 memcpy(data + row * base_stride, bitmap.data.data + row * bitmap.stride, bitmap.width * channels);

    width = bitmap.width;
    height = bitmap.height;
    offset: int;

    for 1..levels - 1 {
        stride := mip_stride(width, channels);
        next_width := Basic.max(cast(s32) 1, width / 2);
        next_height := Basic.max(cast(s32) 1, height / 2);
        next_stride := mip_stride(next_width, channels);
        next_offset := offset + stride * height;
        alpha: s32 = -1;

        if channels == 4 alpha = 3;

        space := Resize.stbir_colorspace.COLORSPACE_LINEAR;
        if bitmap.flags & .IS_SRGB space = .COLORSPACE_SRGB;

        result := Resize.stbir_resize_uint8_generic(data + offset, width, height, cast(s32) stride, data + next_offset, next_width, next_height, cast(s32) next_stride, cast(s32) channels, alpha, 0, .CLAMP, .DEFAULT, space, null);

        assert(result != 0, "native mip resampling failed");
        offset = next_offset;
        width = next_width;
        height = next_height;
    }

    if bitmap.flags & .FREE_DATA_ON_DEINIT Basic.free(bitmap.data.data);

    bitmap.data = .{ total, data };
    bitmap.stride = cast(s32) base_stride;
    bitmap.mipmap_count = levels;
    bitmap.flags |= .FREE_DATA_ON_DEINIT;
}

#scope_module
mip_stride :: (width: int, channels: int) -> int {
    return (width * channels + 3) & ~cast(int) 3;
}
