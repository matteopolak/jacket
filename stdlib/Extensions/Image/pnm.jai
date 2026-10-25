// jailint: allow-file(magic_number) // decimal digit arithmetic of the PNM header
// PNM: P2/P3 (text) and P5/P6 (binary) graymaps and pixmaps.

// Decodes P2/P5 to GRAY8 and P3/P6 to RGB8. Samples are scaled from maxval (1 to 65535) to 8 bits
// with rounding; binary samples are 16-bit big-endian when maxval exceeds 255.
decode_pnm :: (data: [] u8, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> image: Image, ok: bool, error: string {
    if data.count < 2 || data[0] != #char "P" return .{}, false, "not a PNM image";

    kind := cast(s64) data[1] - #char "0";
    if kind != 2 && kind != 3 && kind != 5 && kind != 6 return .{}, false, "unsupported PNM type";

    pos := 2;
    width := pnm_number(data, *pos);
    height := pnm_number(data, *pos);
    maxval := pnm_number(data, *pos);

    if width < 0 || height < 0 || maxval < 0 return .{}, false, "malformed PNM header";
    if maxval < 1 || maxval > 65535 return .{}, false, "invalid PNM maximum value";
    if !image_size_ok(width, height, max_pixels) return .{}, false, "invalid or too large PNM image size";

    channels: s64 = ifx kind == 3 || kind == 6 then 3 else 1;
    binary := kind >= 5;
    samples := width * height * channels;
    wide := maxval > 255;

    if binary {
        // One whitespace byte ends the header.
        if pos >= data.count || !pnm_is_space(data[pos]) return .{}, false, "malformed PNM header";
        pos += 1;
        if samples * (ifx wide then 2 else 1) > data.count - pos return .{}, false, "truncated PNM data";
    } else {
        // A text sample takes at least a digit and a separator.
        if samples * 2 - 1 > data.count - pos return .{}, false, "truncated PNM data";
    }

    image := make_image(cast(s32) width, cast(s32) height, ifx channels == 3 then Pixel_Format.RGB8 else .GRAY8);
    out := image.pixels.data;

    for i: 0..samples - 1 {
        v: s64;

        if binary {
            if wide {
                v = ((cast(s64) data[pos]) << 8) | data[pos + 1];
                pos += 2;
            } else {
                v = data[pos];
                pos += 1;
            }
        } else {
            v = pnm_number(data, *pos);
            if v < 0 || v > maxval {
                deinit(*image);
                return .{}, false, "invalid PNM sample";
            }
        }

        if binary && v > maxval {
            deinit(*image);
            return .{}, false, "invalid PNM sample";
        }

        out[i] = cast,trunc(u8) (ifx maxval == 255 then v else (v * 255 + maxval / 2) / maxval);
    }

    return image, true, "";
}

// Encodes as binary PPM (P6), or PGM (P5) for gray; alpha is dropped.
encode_ppm :: (image: Image) -> [] u8 {
    if !image_is_valid(image) return empty_bytes();

    gray := image.format == .GRAY8 || image.format == .GRAY_ALPHA8;
    channels: s64 = ifx gray then 1 else 3;

    header := tprint("P%\n% %\n255\n", ifx gray then 5 else 6, image.width, image.height);
    count := cast(s64) image.width * image.height;
    out := NewArray(header.count + count * channels, u8);

    memcpy(out.data, header.data, header.count);

    dst := out.data + header.count;
    src := image.pixels.data;

    if image.format == .GRAY8 {
        memcpy(dst, src, count);
    } else if image.format == .RGB8 {
        memcpy(dst, src, count * 3);
    } else if gray {
        for i: 0..count - 1 dst[i] = src[i * 2];
    } else if image.format == .RGBA8 {
        for i: 0..count - 1 {
            dst[i * 3] = src[i * 4];
            dst[i * 3 + 1] = src[i * 4 + 1];
            dst[i * 3 + 2] = src[i * 4 + 2];
        }
    } else {
        for i: 0..count - 1 {
            dst[i * 3] = src[i * 4 + 2];
            dst[i * 3 + 1] = src[i * 4 + 1];
            dst[i * 3 + 2] = src[i * 4];
        }
    }

    return out;
}

#scope_file

pnm_is_space :: inline (c: u8) -> bool {
    return c == 32 || (c >= 9 && c <= 13);
}

// The next unsigned decimal number after whitespace and # comments; -1 when there is none or it
// is longer than nine digits. Leaves `pos` after the number.
pnm_number :: (data: [] u8, pos: *s64) -> s64 {
    p := pos.*;

    while p < data.count {
        if pnm_is_space(data[p]) {
            p += 1;
        } else if data[p] == #char "#" {
            while p < data.count && data[p] != 10 && data[p] != 13 p += 1;
        } else {
            break;
        }
    }

    value: s64;
    digits := 0;

    while p < data.count && data[p] >= #char "0" && data[p] <= #char "9" {
        value = value * 10 + (data[p] - #char "0");
        digits += 1;
        if digits > 9 {
            pos.* = p;
            return -1;
        }

        p += 1;
    }

    pos.* = p;
    if digits == 0 return -1;

    return value;
}
