#import "Basic";
Icons :: #import "Ico_File";
Files :: #import "File";

u32_at :: (data: string, offset: int) -> u32 {
    return (cast(u32) data[offset]) | ((cast(u32) data[offset + 1]) << 8) | ((cast(u32) data[offset + 2]) << 16) | ((cast(u32) data[offset + 3]) << 24);
}

check_frames :: (encoded: string, channels: s32, expected: [] u8) {
    sizes: [5] s32 = .[256, 128, 48, 32, 16];

    assert(encoded[0] == 0 && encoded[2] == 1 && encoded[4] == 5);
    offset: s64 = 86;

    for size, index: sizes {
        entry := 6 + 16 * index;
        bytes := 40 + 2 * size * size * channels + 2 * ((size * size) / 4);

        assert(encoded[entry] == cast,truncate(u8) size && encoded[entry + 1] == cast,truncate(u8) size);
        assert(encoded[entry + 2] == 0 && encoded[entry + 3] == 0);
        assert(encoded[entry + 4] == 1 && encoded[entry + 6] == channels * 8);
        assert(u32_at(encoded, entry + 8) == cast(u32) bytes && u32_at(encoded, entry + 12) == offset);
        assert(u32_at(encoded, offset) == 40 && u32_at(encoded, offset + 4) == cast(u32) size && u32_at(encoded, offset + 8) == cast(u32) (size * 2));
        assert(u32_at(encoded, offset + 20) == 0);
        for pixel: 0..size * size - 1 for channel : 0..channels - 1 assert(encoded[offset + 40 + pixel * channels + channel] == expected[channel]);
        for padding: offset + 40 + size * size * channels..offset + bytes - 1 assert(encoded[padding] == 0);

        offset += bytes;
    }

    assert(offset == encoded.count);
}

main :: () {
    image_bytes, mask_bytes, total_bytes := Icons.get_bytes_for_image(4, 16);

    assert(image_bytes == 2088 && mask_bytes == 64 && total_bytes == 2216);
    assert(Icons.get_total_bytes_for_size(4, 16) == total_bytes);
    arguments := get_command_line_arguments();

    assert(arguments.count == 2);
    rgba_path := sprint("%/icon-rgba.ico", arguments[1]);
    rgb_path := sprint("%/icon-rgb.ico", arguments[1]);

    defer free(rgba_path);
    defer free(rgb_path);
    pixels: [4] u8 = .[10, 20, 30, 255];
    encoded := Icons.create_ico_file(1, 1, 4, pixels.data);

    defer free(encoded);
    assert(encoded.data != pixels.data);
    check_frames(encoded, 4, pixels);
    assert(Files.write_entire_file(rgba_path, encoded));
    rgb: [3] u8 = .[10, 20, 30];
    small := Icons.create_ico_file(1, 1, 3, rgb.data);

    defer free(small);
    check_frames(small, 3, rgb);
    assert(Files.write_entire_file(rgb_path, small));
    print("ok\n");
}
