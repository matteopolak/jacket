#import "Basic";
Icons :: #import "Ico_File";
Image :: #import "Extensions/Image";

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
