// Private checked wire views; payload ownership remains with the input file.
Image_View :: struct {
    directory_fields: string;
    bytes: string;
}

Directory :: struct {
    images: [] Image_View;
}

inspect :: (data: string) -> bool, Directory {
    if !Wire.contains_extent(data, 0, 6) return false, .{};
    if Wire.load_u16(data, 0) != 0 || Wire.load_u16(data, 2) != 1 return false, .{};

    count := cast(s64) Wire.load_u16(data, 4);
    if count == 0 || !Wire.contains_extent(data, 6, count * 16) return false, .{};

    // Validate the entire directory before allocating or forming payload views.
    directory_end := 6 + count * 16;

    for index: 0..count - 1 {
        entry := 6 + index * 16;
        size := cast(s64) Wire.load_u32(data, entry + 8);
        offset := cast(s64) Wire.load_u32(data, entry + 12);
        if data[entry + 3] != 0 || size == 0 || offset < directory_end || !Wire.contains_extent(data, offset, size) return false, .{};
    }

    result: Directory;
    result.images = .{ count, cast(*Image_View) B.alloc(count * size_of(Image_View)) };
    if !result.images.data return false, .{};

    for *image, index: result.images {
        entry := 6 + index * 16;
        size := cast(s64) Wire.load_u32(data, entry + 8);
        offset := cast(s64) Wire.load_u32(data, entry + 12);

        image.directory_fields = .{ 12, data.data + entry };
        image.bytes = .{ size, data.data + offset };
    }

    return true, result;
}

release :: (directory: *Directory) {
    B.array_free(directory.images);
    directory.* = .{};
}

// Policy is supplied by the native caller; no public compatibility IDs are guessed.
make_group :: (directory: Directory, first_image_id: u16) -> bool, string {
    count := directory.images.count;
    if count <= 0 || count > 65536 - cast(s64) first_image_id return false, "";

    result := B.alloc_string(6 + 14 * count);
    if !result.data return false, "";

    memset(result.data, 0, result.count);
    Wire.store_u16(result.data, 2, 1);
    Wire.store_u16(result.data, 4, cast(u16) count);

    for image, index: directory.images {
        memcpy(result.data + 6 + 14 * index, image.directory_fields.data, 12);
        Wire.store_u16(result.data, 18 + 14 * index, cast(u16) (first_image_id + index));
    }

    return true, result;
}

#scope_file
B :: #import "Basic";
Wire :: #import "internal/resource-wire/integers";
