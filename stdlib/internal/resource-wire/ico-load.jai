create_ico_file_from_bitmap_filename :: (bitmap_filename: string) -> string {
    bytes, read_ok := Files.read_entire_file(bitmap_filename, log_errors = false);
    if !read_ok {
        B.log_error("Unable to read bitmap '%'", bitmap_filename);
        return "";
    }

    defer B.free(bytes);

    if bytes.count > 0x7fffffff {
        B.log_error("Bitmap exceeds the image decoder's input limit: '%'", bitmap_filename);
        return "";
    }

    width: s32;
    height: s32;
    components: s32;
    pixels := Stb.stbi_load_from_memory(bytes.data, cast(s32) bytes.count, *width, *height, *components, 4);

    if !pixels {
        B.log_error("Unable to decode bitmap '%'", bitmap_filename);
        return "";
    }

    defer Stb.stbi_image_free(pixels);

    if (components != 3 && components != 4) || width <= 0 || height <= 0 || cast(s64) width > 0x7fffffffffffffff / 4 / height {
        B.log_error("Unsupported bitmap channels or dimensions: '%'", bitmap_filename);
        return "";
    }

    count := cast(s64) width * height * 4;
    converted := cast(*u8) B.alloc(count);

    if !converted return "";

    defer B.free(converted);

    for row: 0..height - 1 for column : 0..width - 1 {
        input := pixels + (cast(s64) row * width + column) * 4;
        output := converted + ((cast(s64) height - 1 - row) * width + column) * 4;
        output[0] = input[2];
        output[1] = input[1];
        output[2] = input[0];
        output[3] = input[3];
    }

    return Encode.create_ico_file(width, height, 4, converted);
}

#scope_file
B :: #import "Basic";
Files :: #import "File";
Stb :: #import "stb_image";
Encode :: #import "internal/resource-wire/ico-encode";
