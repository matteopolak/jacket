// jailint: allow-file(magic_number) // the QOI header and op encodings
// QOI ("Quite OK Image"): 14-byte header, a stream of one-to-five byte ops, 8-byte end marker.

// Decodes a QOI file to RGB8 (3 channels) or RGBA8 (4 channels).
decode_qoi :: (data: [] u8, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> image: Image, ok: bool, error: string {
    if data.count < 14 + 8 || data[0] != #char "q" || data[1] != #char "o" || data[2] != #char "i" || data[3] != #char "f" {
        return .{}, false, "not a QOI image";
    }

    width := qoi_read_u32(data, 4);
    height := qoi_read_u32(data, 8);
    channels := data[12];
    colorspace := data[13];

    if channels != 3 && channels != 4 return .{}, false, "invalid QOI channel count";
    if colorspace > 1 return .{}, false, "invalid QOI colour space";
    if !image_size_ok(width, height, max_pixels) return .{}, false, "invalid or too large QOI image size";

    // Every pixel costs at least a sixty-second of a byte (a run of 62), so the payload bounds the size.
    if width * height > (cast(s64) data.count - 22) * 62 return .{}, false, "truncated QOI data";

    image := make_image(cast(s32) width, cast(s32) height, ifx channels == 3 then Pixel_Format.RGB8 else .RGBA8);
    out := image.pixels.data;
    total := width * height;
    end := cast(s64) data.count - 8;
    pos := 14;

    index: [64] u32;
    r, g, b, a: u8 = 0, 0, 0, 255;
    run := 0;
    written := 0;

    while written < total {
        if run > 0 {
            run -= 1;
        } else {
            if pos >= end {
                deinit(*image);
                return .{}, false, "truncated QOI data";
            }

            op := data[pos];

            pos += 1;

            if op == 0xfe {
                if pos + 3 > end {
                    deinit(*image);
                    return .{}, false, "truncated QOI data";
                }

                r = data[pos];
                g = data[pos + 1];
                b = data[pos + 2];
                pos += 3;
            } else if op == 0xff {
                if pos + 4 > end {
                    deinit(*image);
                    return .{}, false, "truncated QOI data";
                }

                r = data[pos];
                g = data[pos + 1];
                b = data[pos + 2];
                a = data[pos + 3];
                pos += 4;
            } else if (op >> 6) == 0 {
                v := index[op];

                r = cast,trunc(u8) v;
                g = cast,trunc(u8) (v >> 8);
                b = cast,trunc(u8) (v >> 16);
                a = cast,trunc(u8) (v >> 24);
            } else if (op >> 6) == 1 {
                r += cast,trunc(u8) (((op >> 4) & 3) - 2);
                g += cast,trunc(u8) (((op >> 2) & 3) - 2);
                b += cast,trunc(u8) ((op & 3) - 2);
            } else if (op >> 6) == 2 {
                if pos >= end {
                    deinit(*image);
                    return .{}, false, "truncated QOI data";
                }

                next := data[pos];

                pos += 1;
                dg := cast(s32) (op & 63) - 32;

                r += cast,trunc(u8) (dg - 8 + (next >> 4));
                g += cast,trunc(u8) dg;
                b += cast,trunc(u8) (dg - 8 + (next & 15));
            } else {
                run = op & 63;
            }

            index[qoi_hash(r, g, b, a)] = cast(u32) r | ((cast(u32) g) << 8) | ((cast(u32) b) << 16) | ((cast(u32) a) << 24);
        }

        if channels == 3 {
            p := out + written * 3;
            p[0] = r;
            p[1] = g;
            p[2] = b;
        } else {
            p := out + written * 4;
            p[0] = r;
            p[1] = g;
            p[2] = b;
            p[3] = a;
        }

        written += 1;
    }

    return image, true, "";
}

// Encodes as QOI with 3 channels, or 4 when the image has alpha. Gray is written as colour.
encode_qoi :: (image: Image) -> [] u8 {
    if !image_is_valid(image) return empty_bytes();

    source := image;
    converted := false;

    if image.format != .RGB8 && image.format != .RGBA8 {
        source = convert(image, ifx has_alpha(image.format) then Pixel_Format.RGBA8 else .RGB8);
        converted = true;
    }

    defer if converted deinit(*source);

    channels := bytes_per_pixel(source.format);
    total := cast(s64) source.width * source.height;
    out := NewArray(14 + total * (channels + 1) + 8, u8);

    memcpy(out.data, "qoif".data, 4);
    qoi_write_u32(out, 4, cast(u32) source.width);
    qoi_write_u32(out, 8, cast(u32) source.height);
    out[12] = cast,trunc(u8) channels;
    out[13] = 0;

    pos := 14;
    index: [64] u32;
    pr, pg, pb, pa: u8 = 0, 0, 0, 255;
    run := 0;
    in := source.pixels.data;

    for i: 0..total - 1 {
        r, g, b, a: u8;
        a = 255;

        if channels == 3 {
            p := in + i * 3;
            r = p[0];
            g = p[1];
            b = p[2];
        } else {
            p := in + i * 4;

            r = p[0];
            g = p[1];
            b = p[2];
            a = p[3];
        }

        if r == pr && g == pg && b == pb && a == pa {
            run += 1;
            if run == 62 {
                out[pos] = cast,trunc(u8) (0xc0 | (run - 1));
                pos += 1;
                run = 0;
            }

            continue;
        }

        if run > 0 {
            out[pos] = cast,trunc(u8) (0xc0 | (run - 1));
            pos += 1;
            run = 0;
        }

        key := qoi_hash(r, g, b, a);
        packed := cast(u32) r | ((cast(u32) g) << 8) | ((cast(u32) b) << 16) | ((cast(u32) a) << 24);

        if index[key] == packed {
            out[pos] = cast,trunc(u8) key;
            pos += 1;
        } else {
            index[key] = packed;

            if a == pa {
                dr := cast(s32) r - pr;
                dg := cast(s32) g - pg;
                db := cast(s32) b - pb;

                // Differences wrap in 8 bits, so compare them as signed bytes.
                dr = (dr << 24) >> 24;
                dg = (dg << 24) >> 24;
                db = (db << 24) >> 24;
                drg := dr - dg;
                dbg := db - dg;

                if dr >= -2 && dr <= 1 && dg >= -2 && dg <= 1 && db >= -2 && db <= 1 {
                    out[pos] = cast,trunc(u8) (0x40 | ((dr + 2) << 4) | ((dg + 2) << 2) | (db + 2));
                    pos += 1;
                } else if dg >= -32 && dg <= 31 && drg >= -8 && drg <= 7 && dbg >= -8 && dbg <= 7 {
                    out[pos] = cast,trunc(u8) (0x80 | (dg + 32));
                    out[pos + 1] = cast,trunc(u8) (((drg + 8) << 4) | (dbg + 8));
                    pos += 2;
                } else {
                    out[pos] = 0xfe;
                    out[pos + 1] = r;
                    out[pos + 2] = g;
                    out[pos + 3] = b;
                    pos += 4;
                }
            } else {
                out[pos] = 0xff;
                out[pos + 1] = r;
                out[pos + 2] = g;
                out[pos + 3] = b;
                out[pos + 4] = a;
                pos += 5;
            }
        }

        pr = r;
        pg = g;
        pb = b;
        pa = a;
    }

    if run > 0 {
        out[pos] = cast,trunc(u8) (0xc0 | (run - 1));
        pos += 1;
    }

    for 0..6 out[pos + it] = 0;

    out[pos + 7] = 1;
    pos += 8;

    out.count = pos;

    return out;
}

#scope_file

qoi_hash :: inline (r: u8, g: u8, b: u8, a: u8) -> u32 {
    return (cast(u32) r * 3 + cast(u32) g * 5 + cast(u32) b * 7 + cast(u32) a * 11) & 63;
}

qoi_read_u32 :: (data: [] u8, at: s64) -> s64 {
    return ((cast(s64) data[at]) << 24) | ((cast(s64) data[at + 1]) << 16) | ((cast(s64) data[at + 2]) << 8) | data[at + 3];
}

qoi_write_u32 :: (data: [] u8, at: s64, value: u32) {
    data[at] = cast,trunc(u8) (value >> 24);
    data[at + 1] = cast,trunc(u8) (value >> 16);
    data[at + 2] = cast,trunc(u8) (value >> 8);
    data[at + 3] = cast,trunc(u8) value;
}
