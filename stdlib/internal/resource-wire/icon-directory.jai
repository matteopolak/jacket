// Private checked wire views; payload ownership remains with the input file.
Image_View :: struct {
    directory_fields: string;
    bytes: string;
}

Directory :: struct {
    images: [] Image_View;
}

inspect :: (data: string) -> bool, Directory {
    if !Wire.contains_extent(data, 0, HEADER_SIZE) return false, .{};
    if Wire.load_u16(data, 0) != 0 || Wire.load_u16(data, 2) != 1 return false, .{};

    count := cast(s64) Wire.load_u16(data, 4);
    if count == 0 || !Wire.contains_extent(data, HEADER_SIZE, count * FILE_ENTRY_SIZE) return false, .{};

    // Validate the entire directory before allocating or forming payload views.
    directory_end := HEADER_SIZE + count * FILE_ENTRY_SIZE;

    for index: 0..count - 1 {
        entry := HEADER_SIZE + index * FILE_ENTRY_SIZE;
        size := cast(s64) Wire.load_u32(data, entry + 8);
        offset := cast(s64) Wire.load_u32(data, entry + SHARED_ENTRY_SIZE);
        if data[entry + 3] != 0 || size == 0 || offset < directory_end || !Wire.contains_extent(data, offset, size) return false, .{};
    }

    result: Directory;
    result.images = .{ count, cast(*Image_View) B.alloc(count * size_of(Image_View)) };
    if !result.images.data return false, .{};

    for *image, index: result.images {
        entry := HEADER_SIZE + index * FILE_ENTRY_SIZE;
        size := cast(s64) Wire.load_u32(data, entry + 8);
        offset := cast(s64) Wire.load_u32(data, entry + SHARED_ENTRY_SIZE);

        image.directory_fields = .{ SHARED_ENTRY_SIZE, data.data + entry };
        image.bytes = .{ size, data.data + offset };
    }

    return true, result;
}

release :: (directory: *Directory) {
    B.array_free(directory.images);
    directory.* = .{};
} @FreesArgument

// Policy is supplied by the native caller; no public compatibility IDs are guessed.
make_group :: (directory: Directory, first_image_id: u16) -> bool, string {
    count := directory.images.count;
    if count <= 0 || count > 65536 - cast(s64) first_image_id return false, "";

    result := B.alloc_string(HEADER_SIZE + GROUP_ENTRY_SIZE * count);
    if !result.data return false, "";

    memset(result.data, 0, result.count);
    Wire.store_u16(result.data, 2, 1);
    Wire.store_u16(result.data, 4, cast(u16) count);

    for image, index: directory.images {
        memcpy(result.data + HEADER_SIZE + GROUP_ENTRY_SIZE * index, image.directory_fields.data, SHARED_ENTRY_SIZE);
        Wire.store_u16(result.data, HEADER_SIZE + SHARED_ENTRY_SIZE + GROUP_ENTRY_SIZE * index, cast(u16) (first_image_id + index));
    }

    return true, result;
}

#scope_file
// An icon file entry ends in a 4-byte payload offset; a group entry ends in a 2-byte resource ID.
// The first 12 bytes of both are the same.
HEADER_SIZE :: 6;
FILE_ENTRY_SIZE :: 16;
GROUP_ENTRY_SIZE :: 14;
SHARED_ENTRY_SIZE :: 12;

B :: #import "Basic";
Wire :: #import "internal/resource-wire/integers";
