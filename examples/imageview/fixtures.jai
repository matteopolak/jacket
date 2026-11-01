// Tiny assets are generated, never checked in as binaries.
make_fixture_picture :: () -> Image.Image {
    image := Image.make_image(64, 32, .RGBA8);

    for y: 0..31 for x : 0..63 {
        at := (y * 64 + x) * 4;
        image.pixels[at] = cast(u8) (x * 4);
        image.pixels[at + 1] = cast(u8) (y * 8);
        image.pixels[at + 2] = ifx (x / 8 + y / 8) % 2 == 0 then 32 else 224;
        image.pixels[at + 3] = ifx x >= 48 then 128 else 255;

        if x < 8 && y < 8 {
            image.pixels[at] = 255;
            image.pixels[at + 1] = 0;
            image.pixels[at + 2] = 0;
        }
    }

    return image;
}

generate_fixtures :: (root: string) {
    assert(make_directory_if_it_does_not_exist(root));
    image := make_fixture_picture();

    defer Image.deinit(*image);
    assert(export_picture(image, tprint("%/gradient.png", root)));
    assert(export_picture(image, tprint("%/gradient.jpg", root)));
    assert(Write.stbi_write_bmp(temp_c_string(tprint("%/checker.bmp", root)), 64, 32, 4, image.pixels.data) != 0);
    assert(Write.stbi_write_tga(temp_c_string(tprint("%/alpha.tga", root)), 64, 32, 4, image.pixels.data) != 0);
    qoi := Image.encode_qoi(image);

    defer array_free(qoi);
    assert(write_entire_file(tprint("%/gradient.qoi", root), string.{ qoi.count, qoi.data }));
    ppm := Image.encode_ppm(image);

    defer array_free(ppm);
    assert(write_entire_file(tprint("%/gradient.ppm", root), string.{ ppm.count, ppm.data }));
    // Ico_File takes BGRA, not RGBA. A flat asymmetric colour exposes channel swaps.
    bgra := u8.[10, 20, 230, 255];
    icon := Icons.create_ico_file(1, 1, 4, bgra.data);

    defer free(icon);
    assert(write_entire_file(tprint("%/sizes.ico", root), icon));
    // Two 2x1 frames, global red/blue palette, clear/index/index/end LZW codes.
    gif := "GIF89a\x02\x00\x01\x00\x80\x00\x00\xff\x00\x00\x00\x00\xff";
    control := "\x21\xf9\x04\x00\x0a\x00\x00\x00";
    descriptor := "\x2c\x00\x00\x00\x00\x02\x00\x01\x00\x00\x02\x02";
    bytes := tprint("%%%%%%%\x3b", gif, control, descriptor, "\x44\x0a\x00", control, descriptor, "\x0c\x0a\x00");

    assert(write_entire_file(tprint("%/animated.gif", root), bytes));
    assert(write_entire_file(tprint("%/broken.png", root), "not a picture"));
}

#scope_file
Icons :: #import "Ico_File";
