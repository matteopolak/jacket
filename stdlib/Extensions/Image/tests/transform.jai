// resize, crop, flips, rotation and alpha premultiplication on small images, exactly.

#import "Basic";
#import "Extensions/Image";
#load "helpers/common.jai";

// A w x h gray image whose pixel i holds i.
counting :: (w: s32, h: s32) -> Image {
    image := make_image(w, h, .GRAY8);
    for *byte, i: image.pixels byte.* = cast,trunc(u8) i;
    return image;
}

main :: () {
    // crop
    image := counting(4, 3);
    cropped, ok := crop(image, 1, 1, 2, 2);

    check(ok && cropped.width == 2 && cropped.height == 2 && same_bytes(cropped.pixels, u8.[5, 6, 9, 10]), "crop");
    _, ok = crop(image, 3, 0, 2, 1);
    check(!ok, "crop past the right edge");
    _, ok = crop(image, 0, 0, 5, 1);
    check(!ok, "crop too wide");
    _, ok = crop(image, -1, 0, 1, 1);
    check(!ok, "crop at a negative origin");
    _, ok = crop(image, 0, 0, 0, 1);
    check(!ok, "crop of zero width");
    whole: Image;
    whole, ok = crop(image, 0, 0, 4, 3);
    check(ok && same_bytes(whole.pixels, image.pixels), "crop of everything");
    _, ok = crop(image, 2147483647, 0, 2147483647, 1);
    check(!ok, "crop does not overflow");

    // flips
    flipped := counting(3, 2);

    flip_horizontal(*flipped);
    check(same_bytes(flipped.pixels, u8.[2, 1, 0, 5, 4, 3]), "flip_horizontal");
    flip_vertical(*flipped);
    check(same_bytes(flipped.pixels, u8.[5, 4, 3, 2, 1, 0]), "flip_vertical");

    rgb := make_image(2, 3, .RGB8);
    for 0..17 rgb.pixels[it] = cast,trunc(u8) it;

    flip_horizontal(*rgb);
    check(same_bytes(rgb.pixels, u8.[3, 4, 5, 0, 1, 2, 9, 10, 11, 6, 7, 8, 15, 16, 17, 12, 13, 14]), "flip_horizontal keeps pixels whole");
    flip_vertical(*rgb);
    check(same_bytes(rgb.pixels, u8.[15, 16, 17, 12, 13, 14, 9, 10, 11, 6, 7, 8, 3, 4, 5, 0, 1, 2]), "flip_vertical on rgb");

    // rotation: 3 wide, 2 tall, [0 1 2 / 3 4 5]
    rotated := rotate_90(counting(3, 2), true);

    check(rotated.width == 2 && rotated.height == 3 && same_bytes(rotated.pixels, u8.[3, 0, 4, 1, 5, 2]), "rotate clockwise");
    rotated = rotate_90(counting(3, 2), false);
    check(rotated.width == 2 && rotated.height == 3 && same_bytes(rotated.pixels, u8.[2, 5, 1, 4, 0, 3]), "rotate counter-clockwise");

    for format: ALL_FORMATS {
        original := random_image(5, 3, format, 9);
        a := rotate_90(original, true);
        b := rotate_90(a, true);
        c := rotate_90(b, true);
        d := rotate_90(c, true);

        check(same_bytes(d.pixels, original.pixels) && d.width == 5 && d.height == 3, "four turns");
        e := rotate_90(a, false);

        check(same_bytes(e.pixels, original.pixels), "a turn and back");
    }

    // premultiply_alpha
    p := make_image(4, 1, .RGBA8);

    set_pixel :: (image: *Image, i: s64, r: u8, g: u8, b: u8, a: u8) {
        image.pixels[i * 4] = r;
        image.pixels[i * 4 + 1] = g;
        image.pixels[i * 4 + 2] = b;
        image.pixels[i * 4 + 3] = a;
    }

    set_pixel(*p, 0, 200, 100, 50, 128);
    set_pixel(*p, 1, 200, 100, 50, 0);
    set_pixel(*p, 2, 200, 100, 50, 255);
    set_pixel(*p, 3, 255, 255, 255, 1);
    premultiply_alpha(*p);
    check(same_bytes(p.pixels, u8.[100, 50, 25, 128, 0, 0, 0, 0, 200, 100, 50, 255, 1, 1, 1, 1]), "premultiply RGBA");

    ga := make_image(2, 1, .GRAY_ALPHA8);
    ga.pixels[0] = 255;
    ga.pixels[1] = 51;
    ga.pixels[2] = 10;
    ga.pixels[3] = 255;
    premultiply_alpha(*ga);
    check(same_bytes(ga.pixels, u8.[51, 51, 10, 255]), "premultiply gray alpha");

    opaque := random_image(3, 3, .RGB8, 2);
    copy := convert(opaque, .RGB8);

    premultiply_alpha(*opaque);
    check(same_bytes(copy.pixels, opaque.pixels), "premultiply without alpha does nothing");

    // Every value times every alpha: round(c * a / 255).
    all := make_image(256, 1, .GRAY_ALPHA8);

    for alpha: 0..255 {
        for c: 0..255 {
            all.pixels[c * 2] = cast(u8) c;
            all.pixels[c * 2 + 1] = cast(u8) alpha;
        }

        premultiply_alpha(*all);
        bad := 0;
        for c: 0..255 if all.pixels[c * 2] != cast(u8) ((c * alpha * 2 + 255) / 510) bad += 1;

        check(bad == 0, tprint("premultiply by %", alpha));
    }

    // resize: nearest
    up := resize(counting(2, 2), 4, 4, .NEAREST);

    check(same_bytes(up.pixels, u8.[0, 0, 1, 1, 0, 0, 1, 1, 2, 2, 3, 3, 2, 2, 3, 3]), "nearest up");
    down := resize(counting(4, 4), 2, 2, .NEAREST);

    check(same_bytes(down.pixels, u8.[5, 7, 13, 15]), "nearest down");

    // resize: box averages exact blocks
    blocks := make_image(4, 2, .GRAY8);
    values := u8.[10, 20, 100, 110, 30, 40, 120, 130];

    for values blocks.pixels[it_index] = it;

    box := resize(blocks, 2, 1, .BOX);

    check(same_bytes(box.pixels, u8.[25, 115]), "box 2x2 blocks");
    third := make_image(3, 1, .GRAY8);
    third.pixels[0] = 0;
    third.pixels[1] = 100;
    third.pixels[2] = 200;
    check(resize(third, 1, 1, .BOX).pixels[0] == 100, "box mean of three");
    uneven := resize(third, 2, 1, .BOX); // [0, 1.5) -> (0 + 50) / 1.5, [1.5, 3) -> (50 + 200) / 1.5

    check(uneven.pixels[0] == 33 && uneven.pixels[1] == 167, "box with fractional coverage");

    // resize: bilinear
    pair := make_image(2, 1, .GRAY8);
    pair.pixels[1] = 255;
    check(same_bytes(resize(pair, 4, 1, .BILINEAR).pixels, u8.[0, 64, 191, 255]), "bilinear up");
    check(same_bytes(resize(pair, 2, 1, .BILINEAR).pixels, pair.pixels), "same size is unchanged");

    // Flat images stay flat in every format, filter and direction, and sizes are right.
    for format: ALL_FORMATS for filter : Resize_Filter.[.NEAREST, .BILINEAR, .BOX] for size : s32.[1, 3, 7, 40] {
        flat := make_image(9, 5, format);
        for *byte, i: flat.pixels byte.* = cast,trunc(u8) (37 + (i % bytes_per_pixel(format)) * 50);

        result := resize(flat, size, size + 2, filter);

        check(result.width == size && result.height == size + 2 && result.format == format, "resize shape");
        bad := 0;
        bpp := bytes_per_pixel(format);

        for value, i: result.pixels if value != cast,trunc(u8) (37 + (i % bpp) * 50) bad += 1;

        check(bad == 0, tprint("flat image stays flat: % % %", format, filter, size));
        deinit(*result);
        deinit(*flat);
    }

    // Invalid requests.
    check(resize(image, 0, 4).pixels.count == 0, "resize to zero width");
    check(resize(image, 4, -1).pixels.count == 0, "resize to negative height");
    check(resize(Image.{}, 4, 4).pixels.count == 0, "resize of an empty image");
    check(rotate_90(Image.{}).pixels.count == 0, "rotate of an empty image");
    empty := Image.{};

    flip_vertical(*empty);
    flip_horizontal(*empty);
    premultiply_alpha(*empty);

    finish();
}
