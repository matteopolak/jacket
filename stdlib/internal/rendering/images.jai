// The size of one texel in bytes, or 0 for the block compressed formats.
//
// ```jai
// assert(get_image_bytes_per_texel(.RGBA16F) == 8);
// assert(get_image_bytes_per_texel(.BC1) == 0);
// ```
get_image_bytes_per_texel :: (tf: Texture_Format) -> int {
    if tf == .R8 return 1;
    if tf == .RG8 || tf == .B5G6R5 || tf == .R16 || tf == .R16F || tf == .R16I || tf == .D16 return 2;
    if tf == .RGB8 return 3;
    if tf == .RGBA8 || tf == .RGBX8 || tf == .BGRA8 || tf == .RGB10A2 || tf == .RG16F || tf == .R32F || tf == .RG11B10F || tf == .RGB9E5F || tf == .RG16I || tf == .R32I || tf == .D24S8 || tf == .D32F return 4;
    if tf == .RGBA16F || tf == .RG32F || tf == .RGBA16I || tf == .RG32I || tf == .D32FS8 return 8;
    if tf == .RGBA32F || tf == .RGBA32I return 16;

    return 0;
}

// The size of a 4 by 4 block of a compressed format in bytes. Asserts for formats that aren't block compressed.
get_image_bytes_per_block :: (tf: Texture_Format) -> int {
    if tf == .BC1 || tf == .BC4 || tf == .ETC_RGB || tf == .ETC_R || tf == .ETC_RGB_A1 return 8;
    if tf == .BC3 || tf == .BC5 || tf == .BC6 || tf == .BC7 || tf == .ETC_RGBA || tf == .ETC_RG return 16;
    assert(false, "texture format has no block representation");
    return 0;
}

// The bytes an image of this format and size takes, and the bytes in one row (or one row of blocks).
//
// ```jai
// size, stride := get_image_size(.RGBA8, 10, 5);
// assert(size == 200 && stride == 40);
//
// size, stride = get_image_size(.BC1, 10, 5);  // 3 x 2 blocks of 8 bytes
// assert(size == 48 && stride == 24);
// ```
get_image_size :: (tf: Texture_Format, w: int, h: int) -> int, stride: int {
    bytes := get_image_bytes_per_texel(tf);
    if bytes != 0 return w * h * bytes, w * bytes;

    bytes = get_image_bytes_per_block(tf);
    stride := ((w + 3) / 4) * bytes;

    return stride * ((h + 3) / 4), stride;
}

// Gives `bitmap` pixel storage of the given size and format, freeing what it held. The bitmap owns the
// memory: free it with `deinit`.
//
// ```jai
// bitmap: Bitmap;
// bitmap_alloc(*bitmap, 8, 8, .R8);
// defer deinit(*bitmap);
// assert(bitmap.data.count == 64);
// ```
bitmap_alloc :: (using bitmap: *Bitmap, w: s32, h: s32, _format: Texture_Format) {
    assert(w >= 0 && h >= 0);
    deinit(bitmap);
    size, row_bytes := get_image_size(_format, w, h);

    assert(size >= 0 && row_bytes <= 0x7fffffff);
    memory := cast(*u8) Basic.alloc(size);

    assert(memory != null || size == 0, "bitmap allocation failed");
    width = w;
    height = h;
    stride = cast(s32) row_bytes;
    format = _format;
    data = .{ size, memory };
    flags = .FREE_DATA_ON_DEINIT;
}

// Frees the bitmap's pixels if it owns them, and empties it.
deinit :: (bitmap: *Bitmap) {
    if bitmap.flags & .FREE_DATA_ON_DEINIT Basic.free(bitmap.data.data);
    bitmap.* = .{};
} @FreesArgument
