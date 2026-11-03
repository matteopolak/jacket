#import "Basic";
Icons :: #import "Ico_File";
Image :: #import "Extensions/Image";

// The 16-pixel entry of an ICO built from `source`, as RGBA.
smallest :: (width: s32, height: s32, comp: s32, source: *u8) -> Image.Image {
    bytes := Icons.create_ico_file(width, height, comp, source);

    defer free(bytes);
    image, ok := Image.decode_ico(cast([] u8) bytes, 4);

    assert(ok && image.width == 16 && image.height == 16);

    return image;
}

main :: () {
    // One-pixel black and white checkerboard: averaging in linear light gives sRGB 188, not 128.
    checker: [32 * 32 * 3] u8;

    for y: 0..31 for x : 0..31 {
        value: u8 = ifx (x + y) % 2 == 0 then 255 else 0;
        for c: 0..2 checker[(y * 32 + x) * 3 + c] = value;
    }

    image := smallest(32, 32, 3, checker.data);

    defer Image.deinit(*image);

    for y: 4..11 for x : 4..11 {
        value := cast(s64) image.pixels[(y * 16 + x) * 4];
        assert(value >= 186 && value <= 190);
        assert(image.pixels[(y * 16 + x) * 4 + 3] == 255);
    }

    // Opaque red on the left half, fully transparent green on the right: no green bleeds into the red.
    halves: [32 * 8 * 4] u8;

    for y: 0..7 for x : 0..31 {
        at := (y * 32 + x) * 4;

        if x < 16 {
            halves[at + 2] = 255;
            halves[at + 3] = 255;
        } else {
            halves[at + 1] = 255;
        }
    }

    wide := smallest(32, 8, 4, halves.data);

    defer Image.deinit(*wide);

    for y: 0..15 {
        left := (y * 16 + 1) * 4;
        right := (y * 16 + 14) * 4;
        assert(wide.pixels[left] == 255 && wide.pixels[left + 1] == 0 && wide.pixels[left + 2] == 0 && wide.pixels[left + 3] == 255);
        assert(wide.pixels[right + 3] == 0);
    }

    // A flat picture stays flat at every size, including when it grows.
    flat := u8.[40, 120, 200, 255];
    image2 := smallest(1, 1, 4, flat.data);

    defer Image.deinit(*image2);

    for 0..255 {
        assert(image2.pixels[it * 4] == 200 && image2.pixels[it * 4 + 1] == 120 && image2.pixels[it * 4 + 2] == 40);
    }

    print("ok\n");
}
