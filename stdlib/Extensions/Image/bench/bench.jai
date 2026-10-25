// jailint: allow-file(magic_number) // image sizes and synthetic content constants are the workload
// Throughput of the Image module in megapixels per second (MP/s), against stb_image,
// stb_image_write and stb_image_resize where they have a counterpart. Native only (stb is C).
//
//     jaic -O2 stdlib/Extensions/Image/bench/bench.jai -exe /tmp/image-bench
//     /tmp/image-bench            # 1 MP and 8 MP images
//     /tmp/image-bench quick      # 1 MP only
//
// Every table row is the best of several runs. "x stb" is our speed over stb's (above 1.00 is
// faster); sizes are compressed bytes, ours over stb's. PNG decoding reads the same file in both
// libraries (the one stb_image_write produced). stb's resizer filters with Catmull-Rom/Mitchell
// kernels by default, so it is asked for its box and triangle filters here, in linear space.

#import "Basic";
#import "Math";
#import "Extensions/Image";
Load :: #import "stb_image";
Write :: #import "stb_image_write";
Resize :: #import "stb_image_resize";

libc :: #system_library "libc";
c_free :: (memory: *void) #foreign libc "free";

seconds :: () -> float64 {
    return to_float64_seconds(current_time_monotonic());
}

// The best time of a run, over at least three runs and about 0.4 s.
measure :: (work: () -> void) -> float64 {
    best: float64 = 1.0e30;
    started := seconds();
    runs := 0;

    while runs < 3 || (seconds() - started < 0.4 && runs < 100) {
        begin := seconds();

        work();
        elapsed := seconds() - begin;
        if elapsed < best best = elapsed;

        runs += 1;
    }

    return best;
}

fixed :: (value: float64, digits: int) -> string {
    return tprint("%", FormatFloat.{ value = value, trailing_width = digits });
}

// ---- Content -------------------------------------------------------------------------------

noise_state: u64 = 12345;

noise :: () -> s32 {
    noise_state = noise_state * 6364136223846793005 + 1442695040888963407;
    return cast(s32) (noise_state >> 40) & 0xffff;
}

// Smooth gradients and waves with a little sensor noise: what a photograph looks like to a filter.
photo :: (width: s32, height: s32, format: Pixel_Format) -> Image {
    image := make_image(width, height, format);
    bpp := bytes_per_pixel(format);

    for y: 0..height - 1 {
        for x: 0..width - 1 {
            p := image.pixels.data + (cast(s64) y * width + x) * bpp;
            fx := cast(float) x / width;
            fy := cast(float) y / height;
            base := 128 + 90 * sin(fx * 9 + sin(fy * 5) * 2) * cos(fy * 7);

            for c: 0..bpp - 1 {
                value := base + c * 17 * sin(fx * 3 + c) + cast(float) (noise() % 9) - 4;
                if has_alpha(format) && c == bpp - 1 value = 255 - 60 * fy;
                p[c] = cast(u8) clamp(value, 0, 255);
            }
        }
    }

    return image;
}

// Flat panels, thin lines and a few colours, like a screenshot.
ui :: (width: s32, height: s32, format: Pixel_Format) -> Image {
    image := make_image(width, height, format);
    bpp := bytes_per_pixel(format);

    palette := u8.[250, 245, 240, 40, 90, 200, 230, 60, 60, 20, 20, 30];

    for y: 0..height - 1 {
        for x: 0..width - 1 {
            p := image.pixels.data + (cast(s64) y * width + x) * bpp;
            block := ((x / 97) + (y / 61) * 3) % 4;
            line := (x % 97 == 0) || (y % 61 == 0);
            glyph := ((x / 5) * 7 + (y / 9) * 3) % 11 == 0 && (x % 5) < 3;
            pick := ifx line then 3 else (ifx glyph then 2 else block % 2);

            for c: 0..bpp - 1 {
                p[c] = palette[pick * 3 + c % 3];
                if has_alpha(format) && c == bpp - 1 p[c] = 255;
                if format == .GRAY8 p[c] = palette[pick * 3];
            }
        }
    }

    return image;
}

// ---- State shared with the measured procedures -----------------------------------------------

source: Image;
file: [] u8; // a PNG of `source` from stb_image_write
interlaced_file: [] u8;
channels: s32;
qoi_file: [] u8;
resize_to_w: s32;
resize_to_h: s32;
resize_filter: Resize_Filter;
convert_to: Pixel_Format;

sink_bytes: [..] u8;

collect :: (user: *void, data: *void, size: s32) #c_call {
    base_context: #Context;

    push_context base_context {
        for 0..size - 1 array_add(*sink_bytes, (cast(*u8) data)[it]);
    }
}

stb_encode_png :: () {
    sink_bytes.count = 0;
    Write.stbi_write_png_to_func(collect, null, source.width, source.height, channels, source.pixels.data, source.width * channels);
}

our_level: s32 = 6;

our_encode_png :: () {
    data := encode_png(source, our_level);
    array_free(data);
}

stb_decode_png :: () {
    w, h, n: s32;
    pixels := Load.stbi_load_from_memory(file.data, cast(s32) file.count, *w, *h, *n, 0);
    Load.stbi_image_free(pixels);
}

our_decode_png :: () {
    image, _, _ := decode_png(file);
    deinit(*image);
}

stb_decode_interlaced :: () {
    w, h, n: s32;
    pixels := Load.stbi_load_from_memory(interlaced_file.data, cast(s32) interlaced_file.count, *w, *h, *n, 0);
    Load.stbi_image_free(pixels);
}

our_decode_interlaced :: () {
    image, _, _ := decode_png(interlaced_file);
    deinit(*image);
}

our_encode_qoi :: () {
    data := encode_qoi(source);
    array_free(data);
}

our_decode_qoi :: () {
    image, _, _ := decode_qoi(qoi_file);
    deinit(*image);
}

our_convert :: () {
    image := convert(source, convert_to);
    deinit(*image);
}

our_resize :: () {
    image := resize(source, resize_to_w, resize_to_h, resize_filter);
    deinit(*image);
}

stb_resize_filter: Resize.stbir_filter;

stb_resize :: () {
    out := NewArray(cast(s64) resize_to_w * resize_to_h * channels, u8);
    Resize.stbir_resize_uint8_generic(source.pixels.data, source.width, source.height, source.width * channels, out.data, resize_to_w, resize_to_h, resize_to_w * channels, channels, Resize.STBIR_ALPHA_CHANNEL_NONE, 0, .CLAMP, stb_resize_filter, .COLORSPACE_LINEAR, null);
    array_free(out);
}

// ---- An Adam7 PNG for the interlaced rows (Sub filter, stored through our own deflate) ---------

Pass :: struct {
    x0, y0, dx, dy: s64;
}

PASSES :: Pass.[.{ 0, 0, 8, 8 }, .{ 4, 0, 8, 8 }, .{ 0, 4, 4, 8 }, .{ 2, 0, 4, 4 }, .{ 0, 2, 2, 4 }, .{ 1, 0, 2, 2 }, .{ 0, 1, 1, 2 }];

make_interlaced :: (image: Image) -> [] u8 {
    bpp := bytes_per_pixel(image.format);
    raw: [..] u8;

    defer array_free(raw);

    for pass: PASSES {
        row: [..] u8;

        defer array_free(row);
        y := pass.y0;
        while y < image.height {
            row.count = 0;
            x := pass.x0;
            while x < image.width {
                for c: 0..bpp - 1 array_add(*row, image.pixels[(y * image.width + x) * bpp + c]);
                x += pass.dx;
            }

            if row.count > 0 {
                array_add(*raw, 1);
                for i: 0..row.count - 1 array_add(*raw, row[i] - ifx i >= bpp then row[i - bpp] else cast(u8) 0);
            }

            y += pass.dy;
        }
    }

    packed := deflate_for_bench(raw);

    defer array_free(packed);

    out: [..] u8;
    for 0..7 array_add(*out, "\x89PNG\r\n\x1a\n"[it]);

    color_type: u8 = ifx bpp == 1 then cast(u8) 0 else (ifx bpp == 3 then cast(u8) 2 else 6);
    header := u8.[0, 0, 0, 0, 0, 0, 0, 0, 8, color_type, 0, 0, 1];

    put_be(header, 0, image.width);
    put_be(header, 4, image.height);
    append_chunk(*out, "IHDR", header);
    append_chunk(*out, "IDAT", packed);
    append_chunk(*out, "IEND", .[]);

    return out;
}

put_be :: (data: [] u8, at: s64, value: s64) {
    for 0..3 data[at + it] = cast,trunc(u8) (value >> ((3 - it) * 8));
}

append_chunk :: (out: *[..] u8, kind: string, body: [] u8) {
    start := out.count;

    for 0..3 array_add(out, cast(u8) 0);

    put_be(out.*, start, body.count);
    for 0..3 array_add(out, kind[it]);
    for body array_add(out, it);

    view: [] u8;
    view.data = out.data + start + 4;
    view.count = body.count + 4;
    crc := crc32(view);
    for 0..3 array_add(out, cast,trunc(u8) (crc >> ((3 - it) * 8)));
}

#import "Extensions/Deflate";

deflate_for_bench :: (data: [] u8) -> [] u8 {
    return deflate(data, .ZLIB, 6);
}

// ---- Reports ---------------------------------------------------------------------------------

row :: (label: string, ours_seconds: float64, theirs_seconds: float64, megapixels: float64, extra: string) {
    mine := megapixels / ours_seconds;

    if theirs_seconds > 0 {
        print("  % % MP/s   stb % MP/s   x stb %%\n", label, fixed(mine, 1), fixed(megapixels / theirs_seconds, 1), fixed(theirs_seconds / ours_seconds, 2), extra);
    } else {
        print("  % % MP/s%\n", label, fixed(mine, 1), extra);
    }
}

run_size :: (width: s32, height: s32) {
    megapixels := cast(float64) width * height / 1.0e6;

    print("\n== % x % (% MP) ==\n", width, height, fixed(megapixels, 1));

    for content: 0..1 for format : Pixel_Format.[.GRAY8, .RGB8, .RGBA8] {
        name := tprint("% %", ifx content == 0 then "photo" else "ui   ", format);

        source = ifx content == 0 then photo(width, height, format) else ui(width, height, format);
        channels = cast(s32) bytes_per_pixel(format);

        stb_time := measure(stb_encode_png);
        stb_size := sink_bytes.count;
        our_time := measure(our_encode_png);
        ours := encode_png(source, 6);

        row(tprint("png encode %", name), our_time, stb_time, megapixels, tprint("   size x%", fixed(cast(float64) ours.count / stb_size, 3)));
        array_free(ours);

        our_level = 1;
        our_time = measure(our_encode_png);
        ours = encode_png(source, 1);
        row(tprint("png encode %  (level 1)", name), our_time, stb_time, megapixels, tprint("   size x%", fixed(cast(float64) ours.count / stb_size, 3)));
        array_free(ours);
        our_level = 6;

        array_free(file);
        file = NewArray(sink_bytes.count, u8);
        memcpy(file.data, sink_bytes.data, file.count);
        stb_time = measure(stb_decode_png);
        our_time = measure(our_decode_png);
        row(tprint("png decode %", name), our_time, stb_time, megapixels, "");

        array_free(interlaced_file);
        interlaced_file = make_interlaced(source);
        stb_time = measure(stb_decode_interlaced);
        our_time = measure(our_decode_interlaced);
        row(tprint("png adam7  %", name), our_time, stb_time, megapixels, "");

        if format != .GRAY8 {
            our_time = measure(our_encode_qoi);
            array_free(qoi_file);
            qoi_file = encode_qoi(source);
            row(tprint("qoi encode %", name), our_time, 0, megapixels, tprint("   size x%", fixed(cast(float64) qoi_file.count / stb_size, 3)));
            our_time = measure(our_decode_qoi);
            row(tprint("qoi decode %", name), our_time, 0, megapixels, "");
        }

        deinit(*source);
    }

    // Conversions.
    source = photo(width, height, .RGBA8);

    for to: Pixel_Format.[.RGB8, .GRAY8, .BGRA8, .GRAY_ALPHA8] {
        convert_to = to;
        row(tprint("convert RGBA8 to %", to), measure(our_convert), 0, megapixels, "");
    }

    deinit(*source);
    source = photo(width, height, .GRAY8);
    convert_to = .RGBA8;
    row("convert GRAY8 to RGBA8", measure(our_convert), 0, megapixels, "");
    deinit(*source);

    // Resizing, in input megapixels per second.
    for format: Pixel_Format.[.RGB8, .RGBA8] {
        source = photo(width, height, format);
        channels = cast(s32) bytes_per_pixel(format);

        for filter: Resize_Filter.[.NEAREST, .BILINEAR, .BOX] {
            resize_filter = filter;
            stb_resize_filter = ifx filter == .BOX then Resize.stbir_filter.BOX else .TRIANGLE;

            resize_to_w = width / 4;
            resize_to_h = height / 4;
            our_time := measure(our_resize);
            stb_time := ifx filter != .NEAREST then measure(stb_resize) else 0.0;

            row(tprint("resize % % down 4x", format, filter), our_time, stb_time, megapixels, "");

            resize_to_w = width * 2;
            resize_to_h = height * 2;
            our_time = measure(our_resize);
            stb_time = ifx filter != .NEAREST then measure(stb_resize) else 0.0;
            row(tprint("resize % % up 2x  ", format, filter), our_time, stb_time, megapixels, "");
        }

        deinit(*source);
    }
}

main :: () {
    quick := false;
    args := get_command_line_arguments();

    for args if it == "quick" quick = true;

    run_size(1024, 1024);
    if !quick run_size(2896, 2896);
}
