// test: skip_on(.WASM)
// PNG against the C stb libraries: ours decodes what stb_image_write encodes, stb_image decodes
// what ours encodes, and both decoders agree on every fixture. Native and interpreted only.

#import "Basic";
#import "Extensions/Image";
Load :: #import "stb_image";
Write :: #import "stb_image_write";
#load "helpers/common.jai";
#load "fixtures/png_cases.jai";

Sink :: struct {
    bytes: [..] u8;
}

collect :: (user: *void, data: *void, size: s32) #c_call {
    sink := cast(*Sink) user;
    base_context: #Context;

    push_context base_context {
        for 0..size - 1 array_add(*sink.bytes, (cast(*u8) data)[it]);
    }
}

channels_of :: (format: Pixel_Format) -> s32 {
    return cast(s32) bytes_per_pixel(format);
}

// stb_image's pixels, copied into an array we own.
stb_decode :: (data: [] u8, wanted_channels: s32) -> [] u8, width: s32, height: s32, channels: s32 {
    w, h, found: s32;
    loaded := Load.stbi_load_from_memory(data.data, cast(s32) data.count, *w, *h, *found, wanted_channels);

    if !loaded {
        empty: [] u8;
        return empty, 0, 0, 0;
    }

    n := ifx wanted_channels != 0 then wanted_channels else found;
    result := NewArray(cast(s64) w * h * n, u8);

    memcpy(result.data, loaded, result.count);
    Load.stbi_image_free(loaded);

    return result, w, h, n;
}

main :: () {
    for format: ALL_FORMATS {
        if format == .BGRA8 continue;

        for size: s32.[1, 3, 17, 64] {
            source := random_image(size, size + 5, format, cast(u64) size);
            n := channels_of(format);

            // stb writes, we read.
            sink: Sink;

            check(Write.stbi_write_png_to_func(collect, *sink, size, size + 5, n, source.pixels.data, size * n) != 0, "stb encodes");

            decoded, ok, error := decode_png(sink.bytes);

            check(ok, error);
            check(decoded.format == format && same_bytes(decoded.pixels, source.pixels), tprint("we read stb's % %", format, size));
            array_free(sink.bytes);
            deinit(*decoded);

            // we write, stb reads.
            for level: s32.[0, 1, 6, 9] {
                ours := encode_png(source, level);
                pixels, w, h, channels := stb_decode(ours, 0);

                check(w == size && h == size + 5 && channels == n, tprint("stb reads our header (% %)", format, size));
                check(same_bytes(pixels, source.pixels), tprint("stb reads our % % level %", format, size, level));
                array_free(pixels);
                array_free(ours);
            }

            deinit(*source);
        }
    }

    // Both decoders agree on the fixtures.
    for PNG_CASES {
        data := bytes_of(it.png);
        theirs, w, h, channels := stb_decode(data, 0);
        ours, ok, error := decode_png(data);

        if !ok {
            check(false, tprint("% failed: %", it.name, error));
            continue;
        }

        check(w == ours.width && h == ours.height && channels == channels_of(ours.format), tprint("% shape differs", it.name));
        check(same_bytes(theirs, ours.pixels), tprint("% pixels differ from stb_image", it.name));
        array_free(theirs);
        deinit(*ours);
    }

    finish();
}
