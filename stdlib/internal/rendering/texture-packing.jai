#scope_module
#load "gl-formats.jai";

GL_Image_Format :: struct {
    internal, external, scalar: u32;
    compressed := false;
    integer := false;
}

pack_bitmap :: (bitmap: *Bitmap) -> [] u8, bool {
    if bitmap.width <= 0 || bitmap.height <= 0 || bitmap.mipmap_count != 1 return .{}, false;
    if !bitmap.data.data return .{}, false;

    size, row_bytes := get_image_size(bitmap.format, bitmap.width, bitmap.height);
    rows := cast(int) bitmap.height;

    if get_image_bytes_per_texel(bitmap.format) == 0 rows = (rows + 3) / 4;
    if size > 0x7fffffff || bitmap.stride < row_bytes return .{}, false;
    if bitmap.data.count < (rows - 1) * bitmap.stride + row_bytes return .{}, false;

    result: [] u8 = .{ size, cast(*u8) Basic.alloc(size) };

    assert(result.data != null, "texture upload staging allocation failed");
    for row: 0..rows - 1 memcpy(result.data + row * row_bytes, bitmap.data.data + row * bitmap.stride, row_bytes);

    return result, true;
}
