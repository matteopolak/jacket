// JPEG decode throughput of Extensions/Image against stb_image (native only: stb_image is a C
// library). Photo-like pictures are generated at run time and encoded with stb_image_write
// (baseline) or the `cjpeg` command line tool (progressive, grayscale, restart intervals, ...);
// nothing is written outside the temporary directory.
//
//     JAIC_NATIVE_LIBS=target/native-libs/macos-arm64 \
//         jaic -O2 stdlib/Extensions/Image/bench/jpeg_bench.jai -exe /tmp/jpeg-bench
//     /tmp/jpeg-bench                      # 1 MP and 12 MP pictures, every case
//     /tmp/jpeg-bench 4                    # one 4 MP picture
//     /tmp/jpeg-bench photo.jpg other.jpg  # your own files instead
//
// MB/s counts compressed bytes, MP/s decoded pixels. "ratio" is our time divided by stb_image's:
// below 1.0 is faster.

#import "Basic";
#import "String";
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

Case :: struct {
    name: string;
    jpeg: [] u8;
    width: s32;
    height: s32;
    channels: s32;
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

    return bytes;
}

best_time :: (what: (data: [] u8) -> bool, data: [] u8) -> float64 {
    best: float64 = 1.0e30;
    total: float64 = 0;
    runs := 0;

    while runs < 3 || (total < 1.5 && runs < 40) {
        start := seconds_since_init();
        ok := what(data);
        elapsed := seconds_since_init() - start;

        assert(ok);
        total += elapsed;
        runs += 1;
        if elapsed < best best = elapsed;
    }

    return best;
}

decode_ours :: (data: [] u8) -> bool {
    image, ok := decode_image(data);
    if ok deinit(*image);
    return ok;
}

stb_channels: s32;

decode_stb :: (data: [] u8) -> bool {
    w, h, found: s32;
    pixels := Load.stbi_load_from_memory(data.data, cast(s32) data.count, *w, *h, *found, stb_channels);

    if !pixels return false;

    Load.stbi_image_free(pixels);

    return true;
}

fixed :: (value: float64) -> string {
    scaled := cast(s64) (value * 10.0 + 0.5);
    return tprint("%.%", scaled / 10, scaled % 10);
}

fixed2 :: (value: float64) -> string {
    scaled := cast(s64) (value * 100.0 + 0.5);
    return tprint("%.%%", scaled / 100, (scaled / 10) % 10, scaled % 10);
}

report :: (c: Case) {
    megapixels := cast(float64) c.width * c.height / 1.0e6;
    megabytes := cast(float64) c.jpeg.count / 1.0e6;

    stb_channels = c.channels;
    ours := best_time(decode_ours, c.jpeg);
    theirs := best_time(decode_stb, c.jpeg);

    // Both decoders must agree on what the picture is.
    image, ok := decode_image(c.jpeg);

    assert(ok);
    assert(image.width == c.width && image.height == c.height);
    deinit(*image);

    print("%  % KB  % bpp  |  ours % MB/s % MP/s  |  stb % MB/s % MP/s  |  ratio %\n",
        pad(c.name, 28), c.jpeg.count / 1000, fixed(megabytes * 8 / megapixels), fixed(megabytes / ours), fixed(megapixels / ours),
        fixed(megabytes / theirs), fixed(megapixels / theirs), fixed2(ours / theirs));
}

pad :: (s: string, width: s64) -> string {
    if s.count >= width return s;
    return tprint("%%", s, repeat(" ", width - s.count));
}

repeat :: (s: string, count: s64) -> string {
    builder: String_Builder;
    for 1..count append(*builder, s);
    return builder_to_string(*builder);
}

run_size :: (width: s32, height: s32, directory: string) {
    print("\n%x% (% MP)\n", width, height, fixed(cast(float64) width * height / 1.0e6));
    rgb := make_picture(width, height);
    gray := to_gray(rgb);

    cases: [..] Case;

    add :: (cases: *[..] Case, name: string, jpeg: [] u8, width: s32, height: s32, channels: s32) {
        array_add(cases, .{ name, jpeg, width, height, channels });
    }

    add(*cases, "baseline 4:2:0 q75 (stb)", encode_stb(rgb, width, height, 75), width, height, 3);
    add(*cases, "baseline 4:4:4 q95 (stb)", encode_stb(rgb, width, height, 95), width, height, 3);
    add(*cases, "baseline 4:2:2 q85", encode_cjpeg(rgb, width, height, 3, directory, "-quality", "85", "-sample", "2x1"), width, height, 3);
    add(*cases, "baseline 4:2:0 q75 optimized", encode_cjpeg(rgb, width, height, 3, directory, "-quality", "75", "-optimize"), width, height, 3);
    add(*cases, "baseline 4:2:0 q75 restarts", encode_cjpeg(rgb, width, height, 3, directory, "-quality", "75", "-restart", "32B"), width, height, 3);
    add(*cases, "progressive 4:2:0 q75", encode_cjpeg(rgb, width, height, 3, directory, "-quality", "75", "-progressive"), width, height, 3);
    add(*cases, "progressive 4:4:4 q95", encode_cjpeg(rgb, width, height, 3, directory, "-quality", "95", "-progressive", "-sample", "1x1"), width, height, 3);
    add(*cases, "grayscale q85", encode_cjpeg(gray, width, height, 1, directory, "-quality", "85", "-grayscale"), width, height, 1);

    for cases report(it);
}

main :: () {
    arguments := get_command_line_arguments();
    temp := environment_variable("TMPDIR");

    if !temp.count temp = "/tmp";

    directory := tprint("%/jpeg-bench", trim_right(temp, "/"));

    make_directory_if_it_does_not_exist(directory);

    if arguments.count > 1 && !(arguments[1][0] >= #char "0" && arguments[1][0] <= #char "9") {
        for 1..arguments.count - 1 {
            text, ok := read_entire_file(arguments[it]);
            if !ok continue;

            bytes := NewArray(text.count, u8);

            memcpy(bytes.data, text.data, text.count);
            image, decoded, error := decode_image(bytes);
            if !decoded {
                print("%: %\n", arguments[it], error);
                continue;
            }

            report(.{ arguments[it], bytes, image.width, image.height, ifx image.format == .GRAY8 then cast(s32) 1 else 3 });
            deinit(*image);
        }

        return;
    }

    if arguments.count > 1 {
        megapixels := string_to_int(arguments[1]);
        side := cast(s32) sqrt(cast(float) megapixels * 1.0e6 * 4.0 / 3.0);
        run_size(side, side * 3 / 4, directory);
    } else {
        run_size(1280, 800, directory);
        run_size(4000, 3000, directory);
    }
}
