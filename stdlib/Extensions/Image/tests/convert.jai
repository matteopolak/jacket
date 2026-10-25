// convert between every pair of pixel formats against a per-pixel reference.

#import "Basic";
#import "Extensions/Image";
#load "helpers/common.jai";

Rgba :: struct {
    r, g, b, a: s32;
}

read_pixel :: (image: Image, index: s64) -> Rgba {
    p := image.pixels.data + index * bytes_per_pixel(image.format);

    if image.format == {
        case .GRAY8; return .{ p[0], p[0], p[0], 255 };
        case .GRAY_ALPHA8; return .{ p[0], p[0], p[0], p[1] };
        case .RGB8; return .{ p[0], p[1], p[2], 255 };
        case .RGBA8; return .{ p[0], p[1], p[2], p[3] };
        case .BGRA8; return .{ p[2], p[1], p[0], p[3] };
    }

    return .{};
}

expected_bytes :: (c: Rgba, format: Pixel_Format) -> [4] u8, count: s64 {
    luma := cast(u8) ((13933 * cast(s64) c.r + 46871 * cast(s64) c.g + 4732 * cast(s64) c.b + 32768) / 65536);
    gray_source := c.r == c.g && c.g == c.b;
    gray := ifx gray_source then cast(u8) c.r else luma;
    out: [4] u8;

    if format == {
        case .GRAY8;
            out[0] = gray;
            return out, 1;
        case .GRAY_ALPHA8;
            out[0] = gray;
            out[1] = cast(u8) c.a;
            return out, 2;
        case .RGB8;
            out[0] = cast(u8) c.r;
            out[1] = cast(u8) c.g;
            out[2] = cast(u8) c.b;

            return out, 3;
        case .RGBA8;
            out[0] = cast(u8) c.r;
            out[1] = cast(u8) c.g;
            out[2] = cast(u8) c.b;
            out[3] = cast(u8) c.a;

            return out, 4;
        case .BGRA8;
            out[0] = cast(u8) c.b;
            out[1] = cast(u8) c.g;
            out[2] = cast(u8) c.r;
            out[3] = cast(u8) c.a;

            return out, 4;
    }

    return out, 0;
}

main :: () {
    for from: ALL_FORMATS for to : ALL_FORMATS {
        source := random_image(13, 9, from, 5);
        result := convert(source, to);

        check(result.width == 13 && result.height == 9 && result.format == to, "shape");
        check(result.pixels.count == 13 * 9 * bytes_per_pixel(to), "size");

        bad := 0;

        for i: 0..13 * 9 - 1 {
            c := read_pixel(source, i);
            want, n := expected_bytes(c, to);

            // Colour to gray uses luma, gray to colour repeats; for a gray source pixel the
            // luma of equal channels is the channel itself, which the reference relies on.
            for k: 0..n - 1 if result.pixels[i * n + k] != want[k] bad += 1;
        }

        check(bad == 0, tprint("% to %: % wrong bytes", from, to, bad));

        deinit(*source);
        deinit(*result);
    }

    // Anchors: white stays white, primaries get their Rec. 709 weights.
    white := make_image(1, 1, .RGB8);
    white.pixels[0] = 255;
    white.pixels[1] = 255;
    white.pixels[2] = 255;
    gray := convert(white, .GRAY8);

    check(gray.pixels[0] == 255, "white luma");

    red := make_image(1, 1, .RGB8);
    red.pixels[0] = 255;
    check(convert(red, .GRAY8).pixels[0] == 54, "red luma");
    green := make_image(1, 1, .RGB8);
    green.pixels[1] = 255;
    check(convert(green, .GRAY8).pixels[0] == 182, "green luma");
    blue := make_image(1, 1, .RGB8);
    blue.pixels[2] = 255;
    check(convert(blue, .GRAY8).pixels[0] == 18, "blue luma");

    // Invalid images come back empty.
    empty := convert(Image.{}, .RGB8);

    check(empty.pixels.count == 0 && empty.width == 0, "empty in, empty out");
    bad_image := make_image(2, 2, .RGB8);
    bad_image.pixels.count = 5;
    check(convert(bad_image, .RGBA8).pixels.count == 0, "inconsistent size");

    // The same format copies.
    same := random_image(3, 3, .RGBA8, 1);
    copy := convert(same, .RGBA8);

    check(same_bytes(same.pixels, copy.pixels) && copy.pixels.data != same.pixels.data, "same format is a copy");

    finish();
}
