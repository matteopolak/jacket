// The image decoders on arbitrary bytes, the encoders against the decoders, and the pixel
// transforms against their inverses. A decoder must return an error or a consistent image for any
// input, never crash, read outside the data or loop; an encoder's output must decode to the image
// that went in.

// Bytes per gradient step, and where the pixels start in the BMP headers built below.
GRADIENT_STEP :: 5;
BMP_PIXELS_AT :: 54;

image_valid :: (image: Target.Image) -> bool {
    if image.width <= 0 || image.height <= 0 return false;
    return image.pixels.count == cast(s64) image.width * image.height * Target.bytes_per_pixel(image.format);
}

image_same :: (a: Target.Image, b: Target.Image) -> bool {
    if a.width != b.width || a.height != b.height || a.format != b.format || a.pixels.count != b.pixels.count return false;
    for 0..a.pixels.count - 1 if a.pixels[it] != b.pixels[it] return false;
    return true;
}

// A small image with some structure: flat runs, gradients and noise.
image_random :: (r: *Rng) -> Target.Image {
    width := cast(s32) below(r, 24) + 1;
    height := cast(s32) below(r, 24) + 1;
    format := cast(Target.Pixel_Format) below(r, 5);
    image := Target.make_image(width, height, format);
    mode := below(r, 3);

    for 0..image.pixels.count - 1 {
        if mode == 0 {
            image.pixels[it] = cast,trunc(u8) next(r);
        } else if mode == 1 {
            image.pixels[it] = cast,trunc(u8) (it / GRADIENT_STEP);
        } else {
            image.pixels[it] = cast,trunc(u8) (128 + below(r, 3));
        }
    }

    return image;
}

// Runs every decoder; a success must hold a consistent image.
image_decode_all :: (data: [] u8, max_pixels: s64) {
    image, ok, message := Target.decode_png(data, max_pixels);

    image_check(image, ok, message, max_pixels);

    image, ok, message = Target.decode_qoi(data, max_pixels);
    image_check(image, ok, message, max_pixels);

    image, ok, message = Target.decode_bmp(data, max_pixels);
    image_check(image, ok, message, max_pixels);

    image, ok, message = Target.decode_pnm(data, max_pixels);
    image_check(image, ok, message, max_pixels);

    image, ok, message = Target.decode_image(data, max_pixels);
    image_check(image, ok, message, max_pixels);
}

image_check :: (image: Target.Image, ok: bool, message: string, max_pixels: s64) {
    if ok {
        assert(image_valid(image), "a decoded image is consistent");
        assert(cast(s64) image.width * image.height <= max_pixels, "a decoded image is within max_pixels");
        copy := image;
        Target.deinit(*copy);
    } else {
        assert(image.pixels.count == 0 && message.count > 0, "a failure gives an empty image and a message");
    }
}

// Any bytes, and bytes that begin like each format so the parsers get past their first check.
fuzz_image_decode_random :: (r: *Rng) {
    body := gen(r, string);
    prefixes := string.["", "\x89PNG\r\n\x1a\n", "qoif", "BM", "P6\n", "P5 ", "P3\n", "P2\n"];
    prefix := prefixes[below(r, prefixes.count)];

    data := NewArray(prefix.count + body.count, u8);

    defer array_free(data);
    memcpy(data.data, prefix.data, prefix.count);
    memcpy(data.data + prefix.count, body.data, body.count);

    image_decode_all(data, 1 << 14);
}

// Headers of the right shape (small dimensions, valid fields) followed by arbitrary bytes.
fuzz_image_decode_headers :: (r: *Rng) {
    body := gen(r, string);
    width := cast(u8) below(r, 12) + 1;
    height := cast(u8) below(r, 12) + 1;
    kind := below(r, 3);

    header: [..] u8;

    defer array_free(header);

    if kind == 0 {
        for "qoif" array_add(*header, it);
        for u8.[0, 0, 0, width, 0, 0, 0, height, cast(u8) (3 + below(r, 2)), cast(u8) below(r, 2)] array_add(*header, it);
    } else if kind == 1 {
        for "BM" array_add(*header, it);
        for 0..11 array_add(*header, 0);

        header[10] = cast(u8) (BMP_PIXELS_AT + 4 * below(r, 2));
        for u8.[40, 0, 0, 0, width, 0, 0, 0, height, 0, 0, 0, 1, 0, cast(u8) (1 << below(r, 6)), 0, cast(u8) below(r, 4), 0, 0, 0] array_add(*header, it);
        for 0..19 array_add(*header, 0);
    } else {
        for "\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR" array_add(*header, it);
        for u8.[0, 0, 0, width, 0, 0, 0, height, cast(u8) (1 << below(r, 5)), cast(u8) below(r, 7), 0, 0, cast(u8) below(r, 2)] array_add(*header, it);
        for 0..3 array_add(*header, cast,trunc(u8) next(r));
    }

    data := NewArray(header.count + body.count, u8);

    defer array_free(data);
    memcpy(data.data, header.data, header.count);
    memcpy(data.data + header.count, body.data, body.count);

    image_decode_all(data, 1 << 14);
}

// A real file with bytes changed, inserted, dropped or cut off.
fuzz_image_decode_mutated :: (r: *Rng) {
    source := image_random(r);

    defer Target.deinit(*source);

    which := below(r, 4);
    file: [] u8;

    if which == 0 {
        file = Target.encode_png(source, cast(s32) below(r, 10));
    } else if which == 1 {
        file = Target.encode_qoi(source);
    } else if which == 2 {
        file = Target.encode_bmp(source);
    } else {
        file = Target.encode_ppm(source);
    }

    defer array_free(file);

    data: [..] u8;

    defer array_free(data);
    for file array_add(*data, it);

    changes := below(r, 6) + 1;
    for 0..changes - 1 {
        if data.count == 0 break;

        at := below(r, data.count);
        action := below(r, 5);

        if action == 0 {
            data[at] ^= cast(u8) (1 << below(r, 8));
        } else if action == 1 {
            data[at] = cast,trunc(u8) next(r);
        } else if action == 2 {
            array_ordered_remove_by_index(*data, at);
        } else if action == 3 {
            array_insert_at(*data, cast,trunc(u8) next(r), at);
        } else {
            data.count = at;
        }
    }

    image_decode_all(data, 1 << 14);
}

// Encoding then decoding returns the image (colour formats that a file cannot name come back as
// the nearest one).
fuzz_image_round_trip :: (r: *Rng) {
    source := image_random(r);

    defer Target.deinit(*source);

    exact := source;
    converted_exact := false;

    if source.format == .BGRA8 {
        exact = Target.convert(source, .RGBA8);
        converted_exact = true;
    }

    defer if converted_exact Target.deinit(*exact);

    png := Target.encode_png(source, cast(s32) below(r, 10));

    defer array_free(png);
    image, ok, message := Target.decode_png(png);

    assert(ok, "png round trip decodes");
    assert(image_same(image, exact), "png round trip");
    Target.deinit(*image);

    qoi := Target.encode_qoi(source);

    defer array_free(qoi);
    image, ok, message = Target.decode_qoi(qoi);
    assert(ok, "qoi round trip decodes");
    wanted := ifx Target.has_alpha(source.format) then Target.Pixel_Format.RGBA8 else .RGB8;
    reference := Target.convert(source, wanted);

    assert(image_same(image, reference), "qoi round trip");
    Target.deinit(*image);

    bmp := Target.encode_bmp(source);

    defer array_free(bmp);
    image, ok, message = Target.decode_bmp(bmp);
    assert(ok, "bmp round trip decodes");
    assert(image_same(image, reference), "bmp round trip");
    Target.deinit(*image);
    Target.deinit(*reference);

    ppm := Target.encode_ppm(source);

    defer array_free(ppm);
    image, ok, message = Target.decode_pnm(ppm);
    assert(ok, "ppm round trip decodes");
    assert(image.pixels.count == cast(s64) source.width * source.height * (ifx image.format == .GRAY8 then 1 else 3), "ppm size");
    Target.deinit(*image);
}

// Conversions that lose nothing return the original.
fuzz_image_convert_round_trip :: (r: *Rng) {
    source := image_random(r);

    defer Target.deinit(*source);

    for via: Target.Pixel_Format.[.GRAY8, .GRAY_ALPHA8, .RGB8, .RGBA8, .BGRA8] {
        // Gray survives any format; colour survives formats that keep colour; alpha survives
        // formats that keep alpha.
        gray := source.format == .GRAY8 || source.format == .GRAY_ALPHA8;
        keeps_colour := via == .RGB8 || via == .RGBA8 || via == .BGRA8;
        keeps_alpha := via == .GRAY_ALPHA8 || via == .RGBA8 || via == .BGRA8;

        if !gray && !keeps_colour continue;
        if Target.has_alpha(source.format) && !keeps_alpha continue;

        there := Target.convert(source, via);
        back := Target.convert(there, source.format);

        assert(image_same(back, source), "lossless conversion round trip");
        Target.deinit(*there);
        Target.deinit(*back);
    }
}

// Transforms against their inverses and their shapes.
fuzz_image_transforms :: (r: *Rng) {
    source := image_random(r);

    defer Target.deinit(*source);

    flipped := Target.convert(source, source.format);

    Target.flip_horizontal(*flipped);
    Target.flip_horizontal(*flipped);
    Target.flip_vertical(*flipped);
    Target.flip_vertical(*flipped);
    assert(image_same(flipped, source), "flips are involutions");
    Target.deinit(*flipped);

    turned := Target.rotate_90(source, true);

    assert(turned.width == source.height && turned.height == source.width, "rotation swaps the sides");
    back := Target.rotate_90(turned, false);

    assert(image_same(back, source), "a quarter turn each way");
    Target.deinit(*turned);
    Target.deinit(*back);

    x := cast(s32) below(r, source.width);
    y := cast(s32) below(r, source.height);
    w := cast(s32) below(r, source.width - x) + 1;
    h := cast(s32) below(r, source.height - y) + 1;
    part, ok := Target.crop(source, x, y, w, h);

    assert(ok && part.width == w && part.height == h && image_valid(part), "crop inside");
    Target.deinit(*part);
    _, ok = Target.crop(source, x, y, source.width - x + 1, h);
    assert(!ok, "crop outside");

    width := cast(s32) below(r, 40) + 1;
    height := cast(s32) below(r, 40) + 1;

    for filter: Target.Resize_Filter.[.NEAREST, .BILINEAR, .BOX] {
        scaled := Target.resize(source, width, height, filter);
        assert(scaled.width == width && scaled.height == height && scaled.format == source.format && image_valid(scaled), "resize shape");
        Target.deinit(*scaled);
    }

    copy := Target.convert(source, source.format);

    Target.premultiply_alpha(*copy);
    assert(image_valid(copy), "premultiply");
    if !Target.has_alpha(source.format) assert(image_same(copy, source), "premultiply without alpha");

    Target.deinit(*copy);
}
