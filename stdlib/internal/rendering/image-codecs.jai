bitmap_load :: (bitmap: *Bitmap, filepath: string) -> bool {
    filename := Basic.to_c_string(filepath);

    defer Basic.free(filename);
    w, h, channels: s32;
    pixels := Image.stbi_load(filename, *w, *h, *channels, 0);

    if !pixels return false;

    defer Image.stbi_image_free(pixels);
    adopt_decoded_pixels(bitmap, pixels, w, h, channels);

    return true;
}

bitmap_load_from_memory :: (bitmap: *Bitmap, memory: [] u8) -> bool {
    if memory.count <= 0 || memory.count > 0x7fffffff || !memory.data return false;

    w, h, channels: s32;
    pixels := Image.stbi_load_from_memory(memory.data, cast(s32) memory.count, *w, *h, *channels, 0);

    if !pixels return false;

    defer Image.stbi_image_free(pixels);
    adopt_decoded_pixels(bitmap, pixels, w, h, channels);

    return true;
}

bitmap_save :: (bitmap: *Bitmap, filepath: string, save_alpha := false) -> bool {
    source_channels := get_image_bytes_per_texel(bitmap.format);

    if bitmap.width <= 0 || bitmap.height <= 0 || !bitmap.data.data return false;
    if bitmap.format != .RGB8 && bitmap.format != .RGBA8 return false;

    minimum_stride := cast(int) bitmap.width * source_channels;
    if bitmap.stride < minimum_stride return false;

    required := cast(int) (bitmap.height - 1) * bitmap.stride + minimum_stride;
    if bitmap.data.count < required return false;

    channels := source_channels;
    if !save_alpha && source_channels == 4 channels = 3;

    row_bytes := cast(int) bitmap.width * channels;
    size := row_bytes * bitmap.height;

    if row_bytes > 0x7fffffff return false;

    pixels := cast(*u8) Basic.alloc(size);

    assert(pixels != null, "image encoding allocation failed");
    defer Basic.free(pixels);

    for row: 0..bitmap.height - 1 {
        for column: 0..bitmap.width - 1 {
            source := bitmap.data.data + row * bitmap.stride + column * source_channels;
            dest := pixels + row * row_bytes + column * channels;

            memcpy(dest, source, channels);
        }
    }

    filename := Basic.to_c_string(filepath);

    defer Basic.free(filename);
    if has_suffix(filepath, ".png") return Image_Write.stbi_write_png(filename, bitmap.width, bitmap.height, cast(s32) channels, pixels, cast(s32) row_bytes) != 0;
    if has_suffix(filepath, ".bmp") return Image_Write.stbi_write_bmp(filename, bitmap.width, bitmap.height, cast(s32) channels, pixels) != 0;
    if has_suffix(filepath, ".tga") return Image_Write.stbi_write_tga(filename, bitmap.width, bitmap.height, cast(s32) channels, pixels) != 0;
    if has_suffix(filepath, ".jpg") || has_suffix(filepath, ".jpeg") return Image_Write.stbi_write_jpg(filename, bitmap.width, bitmap.height, cast(s32) channels, pixels, 80) != 0;

    return false;
}

#scope_module
adopt_decoded_pixels :: (bitmap: *Bitmap, pixels: *u8, width: s32, height: s32, channels: s32) {
    formats := Texture_Format.[.R8, .RG8, .RGB8, .RGBA8];
    assert(channels >= 1 && channels <= 4, "native image decoder returned invalid channel count");
    bitmap_alloc(bitmap, width, height, formats[channels - 1]);
    memcpy(bitmap.data.data, pixels, bitmap.data.count);
}

has_suffix :: (text: string, suffix: string) -> bool {
    if suffix.count > text.count return false;

    for c, i: suffix {
        value := text[text.count - suffix.count + i];
        if value >= #char "A" && value <= #char "Z" value += #char "a" - #char "A";
        if value != c return false;
    }

    return true;
}
