get_bytes_for_image :: (comp: s32, size: s32) -> (image_bytes: s32, mask_bytes: s32, total_bytes: s32) {
    image_bytes: s32 = 2 * size * size * comp + 40;
    mask_bytes: s32 = (2 * size * size) / 8;
    return image_bytes, mask_bytes, image_bytes + 2 * mask_bytes;
}

get_total_bytes_for_size :: (comp: s32, size: s32) -> s32 {
    _image_bytes, _mask_bytes, total_bytes := get_bytes_for_image(comp, size);
    return total_bytes;
}

create_ico_file :: (x: s32, y: s32, comp: s32, image_data: *u8) -> string {
    if x <= 0 || y <= 0 || (comp != 3 && comp != 4) || !image_data {
        B.log_error("ICO requires readable tightly packed BGR/BGRA pixels with positive dimensions");
        return "";
    }

    if x > 0x7fffffff / comp {
        B.log_error("ICO source row exceeds the native resize stride limit");
        return "";
    }

    sizes: [5] s32 = .[256, 128, 48, 32, 16];
    total: s64 = 86;

    for size: sizes total += get_total_bytes_for_size(comp, size);

    result := B.alloc_string(total);
    if !result.data return "";

    memset(result.data, 0, result.count);
    Wire.store_u16(result.data, 2, 1);
    Wire.store_u16(result.data, 4, 5);
    cursor: s64 = 86;
    alpha: s32 = -1;

    if comp == 4 alpha = 3;

    for size, index: sizes {
        length := get_total_bytes_for_size(comp, size);
        entry := 6 + 16 * index;
        result.data[entry] = cast,truncate(u8) size;
        result.data[entry + 1] = cast,truncate(u8) size;
        Wire.store_u16(result.data, entry + 4, 1);
        Wire.store_u16(result.data, entry + 6, cast(u16) (comp * 8));
        Wire.store_u32(result.data, entry + 8, cast(u32) length);
        Wire.store_u32(result.data, entry + 12, cast(u32) cursor);
        Wire.store_u32(result.data, cursor, 40);
        Wire.store_u32(result.data, cursor + 4, cast(u32) size);
        Wire.store_u32(result.data, cursor + 8, cast(u32) (2 * size));
        Wire.store_u16(result.data, cursor + 12, 1);
        Wire.store_u16(result.data, cursor + 14, cast(u16) (comp * 8));
        // Each resize starts from the caller's original pixels, never another frame.
        resized := Resize.stbir_resize_uint8_srgb(image_data, x, y, x * comp, result.data + cursor + 40, size, size, size * comp, comp, alpha, .{});
        if resized == 0 {
            B.free(result);
            B.log_error("ICO image resizing failed");
            return "";
        }

        cursor += length;
    }

    return result;
}

#scope_file
B :: #import "Basic";
Wire :: #import "internal/resource-wire/integers";
Resize :: #import "stb_image_resize";
