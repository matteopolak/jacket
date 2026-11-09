#scope_file
// Run with jacket bench --path stdlib/Extensions/Image.
#import "Extensions/Testing";
#import "Basic";
#import "File";
#import "Math";
#import "Extensions/Environment";
#import "Process";
#import "Extensions/Image";
Load :: #import "stb_image";
Write :: #import "stb_image_write";

append_bytes :: (bytes: *[..] u8, data: *u8, count: s64) {
    array_reserve(bytes, bytes.count + count);
    memcpy(bytes.data + bytes.count, data, count);
    bytes.count += count;
}

Sink :: struct {
    bytes: [..] u8;
}

collect :: (user: *void, data: *void, size: s32) #c_call {
    sink := cast(*Sink) user;
    base_context: #Context;

    push_context base_context {
        append_bytes(*sink.bytes, cast(*u8) data, size);
    }
}

// Smooth gradients, a few soft shapes, hard edges and sensor-like noise.
make_picture :: (width: s32, height: s32) -> [] u8 {
    pixels := NewArray(cast(s64) width * height * 3, u8);
    state: u32 = 12345;

    for y: 0..height - 1 {
        for x: 0..width - 1 {
            fx := cast(float) x / width;
            fy := cast(float) y / height;
            wave := sin(fx * 23.0 + sin(fy * 9.0) * 2.0) * cos(fy * 17.0 - fx * 5.0);

            r := 90.0 + 110.0 * fx + 30.0 * wave;
            g := 200.0 - 120.0 * fy + 25.0 * sin(fx * 41.0);
            b := 120.0 + 60.0 * wave + 40.0 * sin(fy * 29.0 + fx * 13.0);

            dx := fx - 0.62;
            dy := fy - 0.4;

            if dx * dx + dy * dy < 0.04 {
                r = 240.0 - r * 0.5;
                b = 40.0 + g * 0.3;
            }

            if (x / 61 + y / 47) % 7 == 0 {
                g = 255.0 - g;
            }

            // Fine texture: two high-frequency waves and per-pixel noise.
            fine := 14.0 * sin(fx * 611.0 + fy * 97.0) * cos(fy * 487.0 - fx * 131.0);

            state = state * 1664525 + 1013904223;
            noise := cast(float) ((state >> 24) % 31) - 15.0 + fine;
            texel := pixels.data + (cast(s64) y * width + x) * 3;
            texel[0] = clamp8(r + noise);
            texel[1] = clamp8(g + noise * 0.8);
            texel[2] = clamp8(b + noise * 1.2);
        }
    }

    return pixels;
}

clamp8 :: (v: float) -> u8 {
    if v < 0 return 0;
    if v > 255 return 255;
    return cast(u8) v;
}

to_gray :: (rgb: [] u8) -> [] u8 {
    gray := NewArray(rgb.count / 3, u8);
    for 0..gray.count - 1 {
        gray[it] = cast(u8) ((cast(s32) rgb[it * 3] * 77 + cast(s32) rgb[it * 3 + 1] * 150 + cast(s32) rgb[it * 3 + 2] * 29) >> 8);
    }

    return gray;
}

encode_stb :: (rgb: [] u8, width: s32, height: s32, quality: s32) -> [] u8 {
    sink: Sink;
    Write.stbi_write_jpg_to_func(collect, *sink, width, height, 3, rgb.data, quality);
    return sink.bytes;
}

// Encodes with cjpeg from a PPM or PGM written under the temporary directory.
encode_cjpeg :: (pixels: [] u8, width: s32, height: s32, channels: s32, directory: string, flags: ..string) -> [] u8 {
    input := tprint("%/in.%", directory, ifx channels == 3 then "ppm" else "pgm");
    output := tprint("%/out.jpg", directory);

    header := tprint("%\n% %\n255\n", ifx channels == 3 then "P6" else "P5", width, height);
    file: [..] u8;

    append_bytes(*file, header.data, header.count);
    append_bytes(*file, pixels.data, pixels.count);
    write_entire_file(input, cast(string) file);
    array_free(file);

    arguments: [..] string;

    array_add(*arguments, "cjpeg");
    for flags array_add(*arguments, it);

    array_add(*arguments, "-outfile");
    array_add(*arguments, output);
    array_add(*arguments, input);
    result := run_command(..arguments);
    if result.exit_code != 0 {
        print("cjpeg failed: is it installed?\n");
        exit(1);
    }

    text, ok := read_entire_file(output);

    assert(ok);
    bytes := NewArray(text.count, u8);

    memcpy(bytes.data, text.data, text.count);

    free(text);

    return bytes;
}

jpeg_case :: (b: *Bench, width: s32, height: s32, variant: int, stb: bool) {
    base := environment_variable("JACKET_OUTPUT_DIR");
    if !base.count base = tprint("%../../../../target", #filepath);

    make_directory_if_it_does_not_exist(base);
    directory := tprint("%/jpeg-bench", base);

    make_directory_if_it_does_not_exist(directory);
    rgb := make_picture(width, height);

    defer array_free(rgb);
    gray := to_gray(rgb);

    defer array_free(gray);
    data: [] u8;

    if variant == {
        case 0; data = encode_stb(rgb, width, height, 75);
        case 1; data = encode_stb(rgb, width, height, 95);
        case 2; data = encode_cjpeg(rgb, width, height, 3, directory, "-quality", "85", "-sample", "2x1");
        case 3; data = encode_cjpeg(rgb, width, height, 3, directory, "-quality", "75", "-optimize");
        case 4; data = encode_cjpeg(rgb, width, height, 3, directory, "-quality", "75", "-restart", "32B");
        case 5; data = encode_cjpeg(rgb, width, height, 3, directory, "-quality", "75", "-progressive");
        case 6; data = encode_cjpeg(rgb, width, height, 3, directory, "-quality", "95", "-progressive", "-sample", "1x1");
        case 7; data = encode_cjpeg(gray, width, height, 1, directory, "-quality", "85", "-grayscale");
    }

    defer free(data.data);
    set_bytes(b, data.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        input := black_box(data);

        if stb {
            w, h, found: s32;
            pixels := Load.stbi_load_from_memory(input.data, cast(s32) input.count, *w, *h, *found, ifx variant == 7 then cast(s32) 1 else 3);

            assert(pixels != null && w == width && h == height);
            black_box(pixels);
            Load.stbi_image_free(pixels);
        } else {
            image, ok := decode_image(input);
            assert(ok && image.width == width && image.height == height);
            black_box(image);
            deinit(*image);
        }
    }
}

#scope_export
jpeg_baseline_q75_our_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 0, false);
} @bench

jpeg_baseline_q75_stb_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 0, true);
} @bench

jpeg_baseline_q95_our_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 1, false);
} @bench

jpeg_baseline_q95_stb_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 1, true);
} @bench

jpeg_baseline_422_our_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 2, false);
} @bench

jpeg_baseline_422_stb_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 2, true);
} @bench

jpeg_optimized_our_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 3, false);
} @bench

jpeg_optimized_stb_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 3, true);
} @bench

jpeg_restarts_our_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 4, false);
} @bench

jpeg_restarts_stb_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 4, true);
} @bench

jpeg_progressive_420_our_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 5, false);
} @bench

jpeg_progressive_420_stb_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 5, true);
} @bench

jpeg_progressive_444_our_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 6, false);
} @bench

jpeg_progressive_444_stb_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 6, true);
} @bench

jpeg_gray_our_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 7, false);
} @bench

jpeg_gray_stb_1mp :: (b: *Bench) {
    jpeg_case(b, 1280, 800, 7, true);
} @bench

jpeg_baseline_q75_our_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 0, false);
} @bench

jpeg_baseline_q75_stb_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 0, true);
} @bench

jpeg_baseline_q95_our_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 1, false);
} @bench

jpeg_baseline_q95_stb_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 1, true);
} @bench

jpeg_baseline_422_our_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 2, false);
} @bench

jpeg_baseline_422_stb_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 2, true);
} @bench

jpeg_optimized_our_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 3, false);
} @bench

jpeg_optimized_stb_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 3, true);
} @bench

jpeg_restarts_our_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 4, false);
} @bench

jpeg_restarts_stb_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 4, true);
} @bench

jpeg_progressive_420_our_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 5, false);
} @bench

jpeg_progressive_420_stb_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 5, true);
} @bench

jpeg_progressive_444_our_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 6, false);
} @bench

jpeg_progressive_444_stb_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 6, true);
} @bench

jpeg_gray_our_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 7, false);
} @bench

jpeg_gray_stb_12mp :: (b: *Bench) {
    jpeg_case(b, 4000, 3000, 7, true);
} @bench
