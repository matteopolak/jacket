// jailint: allow-file(magic_number) // offsets and field widths of the BMP headers
// BMP: file header, a DIB header (OS/2 core, BITMAPINFOHEADER, V4, V5), an optional palette or
// channel masks, then rows padded to four bytes, bottom row first unless the height is negative.

// Decodes BMP files of 1, 4, 8 (palette, optionally RLE), 16, 24 and 32 bits per pixel, with
// channel masks for 16 and 32. Palettised and 24-bit images become RGB8. 16 and 32-bit images
// become RGBA8 when they have an alpha mask (or, for 32-bit without masks, any nonzero alpha
// byte) and RGB8 otherwise.
decode_bmp :: (data: [] u8, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> image: Image, ok: bool, error: string {
    info: Bmp_Info;
    problem := bmp_read_info(data, max_pixels, *info);

    if problem.count > 0 return .{}, false, problem;

    pixels := byte_range(data, info.pixel_offset, data.count - info.pixel_offset);
    width := info.width;
    height := info.height;

    if info.compression == 1 || info.compression == 2 {
        // A run of escapes can skip a long way for few bytes, but not without bound.
        if width * height > (cast(s64) pixels.count + 1) * 16384 return .{}, false, "truncated BMP pixel data";

        image := make_image(cast(s32) width, cast(s32) height, .RGB8);

        if !bmp_decode_rle(pixels, *image, *info) {
            deinit(*image);
            return .{}, false, "malformed BMP RLE data";
        }

        return image, true, "";
    }

    stride := (width * info.bits + 31) / 32 * 4;
    if stride * height > pixels.count return .{}, false, "truncated BMP pixel data";

    if info.bits <= 8 return bmp_decode_palettized(pixels, *info, stride), true, "";
    if info.bits == 24 return bmp_decode_bgr(pixels, *info, stride), true, "";

    return bmp_decode_masked(pixels, *info, stride);
}

// Encodes as an uncompressed bottom-up BMP: 24-bit, or 32-bit with an alpha mask (a V4 header)
// when the image has alpha. Gray is written as colour.
encode_bmp :: (image: Image) -> [] u8 {
    if !image_is_valid(image) return empty_bytes();

    alpha := has_alpha(image.format);
    source := image;
    converted := false;
    wanted := ifx alpha then Pixel_Format.BGRA8 else .RGB8;

    if image.format != wanted {
        source = convert(image, wanted);
        converted = true;
    }

    defer if converted deinit(*source);

    width := cast(s64) image.width;
    height := cast(s64) image.height;
    bits := ifx alpha then 32 else 24;
    stride := (width * bits + 31) / 32 * 4;
    header := ifx alpha then 14 + 108 else 14 + 40;
    total := header + stride * height;

    if total > 0xffffffff return empty_bytes();

    out := NewArray(total, u8);

    memset(out.data, 0, total);
    out[0] = #char "B";
    out[1] = #char "M";
    bmp_put32(out, 2, total);
    bmp_put32(out, 10, header);
    bmp_put32(out, 14, header - 14);
    bmp_put32(out, 18, width);
    bmp_put32(out, 22, height);
    out[26] = 1;
    out[28] = cast,trunc(u8) bits;
    bmp_put32(out, 30, ifx alpha then 3 else 0);
    bmp_put32(out, 34, stride * height);
    bmp_put32(out, 38, 2835);
    bmp_put32(out, 42, 2835);

    if alpha {
        // V4: red, green, blue and alpha masks, then "sRGB" colour space.
        bmp_put32(out, 54, 0x00ff0000);
        bmp_put32(out, 58, 0x0000ff00);
        bmp_put32(out, 62, 0x000000ff);
        bmp_put32(out, 66, 0xff000000);
        bmp_put32(out, 70, 0x73524742);
    }

    bpp := bits / 8;

    for y: 0..height - 1 {
        src := source.pixels.data + (height - 1 - y) * width * bpp;
        dst := out.data + header + y * stride;

        if alpha {
            memcpy(dst, src, width * 4);
        } else {
            for x: 0..width - 1 {
                dst[x * 3] = src[x * 3 + 2];
                dst[x * 3 + 1] = src[x * 3 + 1];
                dst[x * 3 + 2] = src[x * 3];
            }
        }
    }

    return out;
}

#scope_file

Bmp_Info :: struct {
    header_size: s64;
    pixel_offset: s64;
    width: s64;
    height: s64;
    bits: s64;
    compression: s64;
    colors_used: s64;
    palette_entry: s64;
    extra: s64; // bytes of masks between the header and the palette
    top_down: bool;
    masks: [4] u32; // red, green, blue, alpha of 16 and 32-bit pixels
    palette: [256] u32; // r | g << 8 | b << 16
    palette_count: s64;
}

Bmp_Field :: struct {
    shift: s32;
    bits: s32;
}

// Reads and checks the file and DIB headers, the channel masks and the palette.
bmp_read_info :: (data: [] u8, max_pixels: s64, info: *Bmp_Info) -> string {
    if data.count < 26 || data[0] != #char "B" || data[1] != #char "M" return "not a BMP image";

    header_size := bmp_u32(data, 14);

    if header_size != 12 && header_size < 40 return "unsupported BMP header";
    if 14 + header_size > data.count return "truncated BMP header";

    info.header_size = header_size;
    info.pixel_offset = bmp_u32(data, 10);

    if header_size == 12 {
        info.width = bmp_u16(data, 18);
        info.height = bmp_u16(data, 20);
        info.bits = bmp_u16(data, 24);
        info.palette_entry = 3;
    } else {
        info.width = bmp_s32(data, 18);
        info.height = bmp_s32(data, 22);
        info.bits = bmp_u16(data, 28);
        info.compression = bmp_u32(data, 30);
        info.colors_used = bmp_u32(data, 46);
        info.palette_entry = 4;
    }

    if info.height < 0 {
        info.top_down = true;
        info.height = -info.height;
    }

    bits := info.bits;
    compression := info.compression;

    if bits != 1 && bits != 4 && bits != 8 && bits != 16 && bits != 24 && bits != 32 return "unsupported BMP bit depth";
    if compression > 6 || compression == 4 || compression == 5 return "unsupported BMP compression";
    if (compression == 1 && bits != 8) || (compression == 2 && bits != 4) return "invalid BMP RLE depth";
    if (compression == 3 || compression == 6) && bits != 16 && bits != 32 return "invalid BMP bit field depth";
    if !image_size_ok(info.width, info.height, max_pixels) return "invalid or too large BMP image size";

    problem := bmp_read_masks(data, info);
    if problem.count > 0 return problem;

    if bits <= 8 bmp_read_palette(data, info);
    if info.pixel_offset < 14 + header_size || info.pixel_offset > data.count return "invalid BMP pixel data offset";

    return "";
}

bmp_read_masks :: (data: [] u8, info: *Bmp_Info) -> string {
    if info.compression == 3 || info.compression == 6 {
        // The masks follow a 40-byte header, or are part of a longer one.
        at := 14 + 40;
        count := ifx info.compression == 6 || info.header_size >= 56 then 4 else 3;

        if at + count * 4 > data.count return "truncated BMP masks";
        for 0..count - 1 info.masks[it] = cast(u32) bmp_u32(data, at + it * 4);
        if info.header_size == 40 info.extra = count * 4;
    } else if info.bits == 16 {
        info.masks[0] = 0x7c00;
        info.masks[1] = 0x03e0;
        info.masks[2] = 0x001f;
    } else if info.bits == 32 {
        info.masks[0] = 0x00ff0000;
        info.masks[1] = 0x0000ff00;
        info.masks[2] = 0x000000ff;
    }

    return "";
}

bmp_read_palette :: (data: [] u8, info: *Bmp_Info) {
    wanted := ifx info.colors_used != 0 then info.colors_used else ((cast(s64) 1) << info.bits);
    if wanted > 256 wanted = 256;

    at := 14 + info.header_size + info.extra;
    available := (cast(s64) data.count - at) / info.palette_entry;

    if available < 0 available = 0;

    info.palette_count = min(wanted, available);

    for i: 0..info.palette_count - 1 {
        q := at + i * info.palette_entry;
        info.palette[i] = cast(u32) data[q + 2] | ((cast(u32) data[q + 1]) << 8) | ((cast(u32) data[q]) << 16);
    }
}

// 1, 4 and 8 bits per pixel: indices into the palette (entries past it are black).
bmp_decode_palettized :: (pixels: [] u8, info: *Bmp_Info, stride: s64) -> Image {
    width := info.width;
    bits := info.bits;
    image := make_image(cast(s32) width, cast(s32) info.height, .RGB8);
    mask := (1 << bits) - 1;

    for y: 0..info.height - 1 {
        source_row := ifx info.top_down then y else info.height - 1 - y;
        row := pixels.data + source_row * stride;
        dst := image.pixels.data + y * width * 3;

        for x: 0..width - 1 {
            bit := x * bits;
            v := ((cast(s32) row[bit >> 3]) >> (8 - bits - (bit & 7))) & mask;
            c := info.palette[v];

            dst[x * 3] = cast,trunc(u8) c;
            dst[x * 3 + 1] = cast,trunc(u8) (c >> 8);
            dst[x * 3 + 2] = cast,trunc(u8) (c >> 16);
        }
    }

    return image;
}

bmp_decode_bgr :: (pixels: [] u8, info: *Bmp_Info, stride: s64) -> Image {
    width := info.width;
    image := make_image(cast(s32) width, cast(s32) info.height, .RGB8);

    for y: 0..info.height - 1 {
        source_row := ifx info.top_down then y else info.height - 1 - y;
        row := pixels.data + source_row * stride;
        dst := image.pixels.data + y * width * 3;

        for x: 0..width - 1 {
            dst[x * 3] = row[x * 3 + 2];
            dst[x * 3 + 1] = row[x * 3 + 1];
            dst[x * 3 + 2] = row[x * 3];
        }
    }

    return image;
}

// 16 and 32 bits per pixel through channel masks.
bmp_decode_masked :: (pixels: [] u8, info: *Bmp_Info, stride: s64) -> image: Image, ok: bool, error: string {
    width := info.width;
    height := info.height;
    bits := info.bits;
    fields: [4] Bmp_Field;

    for 0..3 {
        valid, field := bmp_make_field(info.masks[it], bits);
        if !valid return .{}, false, "invalid BMP channel mask";
        fields[it] = field;
    }

    has_a := fields[3].bits > 0;
    guess_alpha := bits == 32 && info.compression == 0 && !has_a;

    if guess_alpha {
        // Without an alpha mask the fourth byte is often unused; keep it only when it is used.
        for y: 0..height - 1 {
            row := pixels.data + y * stride;

            for x: 0..width - 1 {
                if row[x * 4 + 3] != 0 {
                    has_a = true;
                    break;
                }
            }

            if has_a break;
        }
    }

    format := ifx has_a then Pixel_Format.RGBA8 else .RGB8;
    image := make_image(cast(s32) width, cast(s32) height, format);
    out_bpp := bytes_per_pixel(format);

    for y: 0..height - 1 {
        source_row := ifx info.top_down then y else height - 1 - y;
        row := pixels.data + source_row * stride;
        dst := image.pixels.data + y * width * out_bpp;

        for x: 0..width - 1 {
            v: u32;

            if bits == 16 {
                v = cast(u32) row[x * 2] | ((cast(u32) row[x * 2 + 1]) << 8);
            } else {
                v = cast(u32) row[x * 4] | ((cast(u32) row[x * 4 + 1]) << 8) | ((cast(u32) row[x * 4 + 2]) << 16) | ((cast(u32) row[x * 4 + 3]) << 24);
            }

            d := dst + x * out_bpp;
            d[0] = bmp_extract(v, fields[0]);
            d[1] = bmp_extract(v, fields[1]);
            d[2] = bmp_extract(v, fields[2]);

            if has_a {
                d[3] = ifx guess_alpha then cast,trunc(u8) (v >> 24) else bmp_extract(v, fields[3]);
            }
        }
    }

    return image, true, "";
}

bmp_u16 :: (data: [] u8, at: s64) -> s64 {
    return cast(s64) data[at] | ((cast(s64) data[at + 1]) << 8);
}

bmp_u32 :: (data: [] u8, at: s64) -> s64 {
    return cast(s64) data[at] | ((cast(s64) data[at + 1]) << 8) | ((cast(s64) data[at + 2]) << 16) | ((cast(s64) data[at + 3]) << 24);
}

bmp_s32 :: (data: [] u8, at: s64) -> s64 {
    v := bmp_u32(data, at);
    if v >= 0x80000000 v -= 0x100000000;
    return v;
}

bmp_put32 :: (data: [] u8, at: s64, value: s64) {
    for 0..3 data[at + it] = cast,trunc(u8) (value >> (it * 8));
}

// Position and width of a contiguous mask; no mask at all is a valid, empty field.
bmp_make_field :: (mask: u32, depth: s64) -> bool, Bmp_Field {
    field: Bmp_Field;
    if mask == 0 return true, field;
    if depth == 16 && (mask >> 16) != 0 return false, field;

    shift: s32;
    while ((mask >> shift) & 1) == 0 shift += 1;

    run := mask >> shift;
    if (run & (run + 1)) != 0 return false, field;

    count: s32;

    while run != 0 {
        count += 1;
        run >>= 1;
    }

    field.shift = shift;
    field.bits = count;

    return true, field;
}

// A channel of `v` scaled to 0..255.
bmp_extract :: (v: u32, field: Bmp_Field) -> u8 {
    if field.bits == 0 return 0;
    if field.bits == 8 return cast,trunc(u8) (v >> field.shift);
    if field.bits > 8 return cast,trunc(u8) (v >> (field.shift + field.bits - 8));

    raw := (v >> field.shift) & (((cast(u32) 1) << field.bits) - 1);
    top := ((cast(u32) 1) << field.bits) - 1;

    return cast,trunc(u8) ((raw * 255 + top / 2) / top);
}

// RLE8 and RLE4: pairs of bytes, (count, value) for runs and (0, n) escapes for end of line (0),
// end of bitmap (1), delta (2) and literal runs (n >= 3). Pixels never written stay black.
bmp_decode_rle :: (pixels: [] u8, image: *Image, info: *Bmp_Info) -> bool {
    width := cast(s64) image.width;
    height := cast(s64) image.height;
    four := info.bits == 4;

    // Palette indices first, then the colours.
    indices := NewArray(width * height, u8);

    defer array_free(indices);
    memset(indices.data, 0, indices.count);

    set :: (indices: [] u8, width: s64, height: s64, x: s64, y: s64, v: u8, top_down: bool) {
        if x < 0 || x >= width || y < 0 || y >= height return;
        row := ifx top_down then y else height - 1 - y;
        indices[row * width + x] = v;
    }

    x := 0;
    y := 0;
    pos := 0;

    while pos + 1 < pixels.count {
        n := cast(s64) pixels[pos];
        v := pixels[pos + 1];

        pos += 2;

        if n > 0 {
            for i: 0..n - 1 {
                value := v;
                if four value = ifx (i & 1) == 0 then v >> 4 else v & 15;
                set(indices, width, height, x, y, value, info.top_down);
                x += 1;
            }

            continue;
        }

        if v == 0 {
            x = 0;
            y += 1;
        } else if v == 1 {
            break;
        } else if v == 2 {
            if pos + 1 >= pixels.count return false;
            x += pixels[pos];
            y += pixels[pos + 1];
            pos += 2;
        } else {
            count := cast(s64) v;
            bytes := ifx four then (count + 1) / 2 else count;

            if pos + bytes > pixels.count return false;

            for i: 0..count - 1 {
                value: u8;

                if four {
                    b := pixels[pos + i / 2];
                    value = ifx (i & 1) == 0 then b >> 4 else b & 15;
                } else {
                    value = pixels[pos + i];
                }

                set(indices, width, height, x, y, value, info.top_down);
                x += 1;
            }

            pos += bytes + (bytes & 1); // runs end on a 16-bit boundary
        }
    }

    for index, i: indices {
        c := info.palette[index];
        out := image.pixels.data + i * 3;
        out[0] = cast,trunc(u8) c;
        out[1] = cast,trunc(u8) (c >> 8);
        out[2] = cast,trunc(u8) (c >> 16);
    }

    return true;
}
