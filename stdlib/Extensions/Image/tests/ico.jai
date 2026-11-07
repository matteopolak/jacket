#import "Basic";
Icons :: #import "Ico_File";
Image :: #import "Extensions/Image";

u32_at :: (bytes: string, at: int) -> u32 {
    value: u32;
    for byte: 0..3 value |= (cast(u32) bytes[at + byte]) << (byte * 8);
    return value;
}

// Windows still consumes the AND mask when every alpha byte is zero, and for DI_MASK.
check_alpha_masks :: (bytes: string) {
    for size, index: s32.[256, 128, 48, 32, 16] {
        color := cast(int) u32_at(bytes, 18 + index * 16) + 40;
        mask := color + size * size * 4;
        stride := ((size + 31) / 32) * 4;

        for row: 0..size - 1 for column : 0..stride * 8 - 1 {
            bit := (bytes[mask + row * stride + column / 8] & (128 >> (column % 8))) != 0;
            transparent := column < size && bytes[color + (row * size + column) * 4 + 3] == 0;
            assert(bit == transparent);
        }
    }
}

main :: () {
    pixel := u8.[10, 20, 30, 255];
    bytes := Icons.create_ico_file(1, 1, 4, pixel.data);

    defer free(bytes);
    sizes := s32.[256, 128, 48, 32, 16];
    for sizes {
        image, ok := Image.decode_ico(cast([] u8) bytes, it_index);
        assert(ok && image.width == it && image.height == it);
        assert(image.pixels[0] == 30 && image.pixels[2] == 10 && image.pixels[3] == 255);
        Image.deinit(*image);
    }

    _, ok := Image.decode_ico(cast([] u8) "junk");

    assert(!ok);
    _, ok = Image.decode_ico(cast([] u8) bytes, 5);
    assert(!ok);

    for n: 0..85 {
        _, ok = Image.decode_ico([] u8.{ n, bytes.data });
        assert(!ok);
    }

    // AND-mask rows are bottom-up and take precedence over 32-bit alpha.
    bytes[86 + 40 + 256 * 256 * 4 + 255 * 32] = 128;
    masked, mask_ok := Image.decode_ico(cast([] u8) bytes, 0);

    assert(mask_ok && masked.pixels[3] == 0 && masked.pixels[7] == 255);
    Image.deinit(*masked);
    rgb := u8.[10, 20, 30];
    rgb_bytes := Icons.create_ico_file(1, 1, 3, rgb.data);

    defer free(rgb_bytes);
    rgb_image, rgb_ok := Image.decode_ico(cast([] u8) rgb_bytes, 4);

    assert(rgb_ok && rgb_image.width == 16 && rgb_image.pixels[0] == 30 && rgb_image.pixels[3] == 255);
    Image.deinit(*rgb_image);
    // An all-zero alpha plane is legacy 0RGB on Windows: the AND mask must retain transparency.
    clear_pixel := u8.[10, 20, 30, 0];
    clear_bytes := Icons.create_ico_file(1, 1, 4, clear_pixel.data);

    defer free(clear_bytes);

    for sizes {
        clear_image, clear_ok := Image.decode_ico(cast([] u8) clear_bytes, it_index);
        defer Image.deinit(*clear_image);
        assert(clear_ok && clear_image.width == it && clear_image.height == it);
        for pixel: 0..it * it - 1 assert(clear_image.pixels[pixel * 4 + 3] == 0);
    }

    check_alpha_masks(clear_bytes);
    mixed := u8.[10, 20, 30, 0, 40, 50, 60, 128, 70, 80, 90, 255, 100, 110, 120, 0];
    mixed_bytes := Icons.create_ico_file(2, 2, 4, mixed.data);

    defer free(mixed_bytes);
    check_alpha_masks(mixed_bytes);
    // Shrinking sparse alpha=1 can quantize the whole small icon to zero alpha.
    // Its hidden color must also be black for Windows' legacy AND/XOR path.
    faint := NewArray(256 * 256 * 4, u8);

    defer array_free(faint);
    faint[0] = 120;
    faint[1] = 80;
    faint[2] = 40;
    faint[3] = 1;
    faint_bytes := Icons.create_ico_file(256, 256, 4, faint.data);

    defer free(faint_bytes);

    for size, index: sizes {
        color := cast(int) u32_at(faint_bytes, 18 + index * 16) + 40;

        for pixel: 0..size * size - 1 {
            at := color + pixel * 4;
            if faint_bytes[at + 3] == 0 for channel: 0..2 assert(faint_bytes[at + channel] == 0);
        }
    }

    check_alpha_masks(faint_bytes);

    // An ICO may hold a PNG instead of a DIB.
    tiny := Image.make_image(2, 1, .RGBA8);

    defer Image.deinit(*tiny);
    tiny.pixels[0] = 220;
    tiny.pixels[3] = 128;
    png := Image.encode_png(tiny);

    defer array_free(png);
    container := NewArray(22 + png.count, u8);

    defer array_free(container);
    container[2] = 1;
    container[4] = 1;
    container[6] = 2;
    container[7] = 1;
    for i: 0..3 container[14 + i] = cast,trunc(u8) (png.count >> (i * 8));

    container[18] = 22;
    memcpy(container.data + 22, png.data, png.count);
    png_image, png_ok := Image.decode_ico(container);

    assert(png_ok && png_image.width == 2 && png_image.pixels[0] == 220 && png_image.pixels[3] == 128);
    Image.deinit(*png_image);
    bytes[18] = 255;
    bytes[19] = 255;
    bytes[20] = 255;
    bytes[21] = 255;
    _, ok = Image.decode_ico(cast([] u8) bytes, 0);
    assert(!ok);
    print("ok\n");
}
