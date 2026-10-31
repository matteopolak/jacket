// jailint: allow-file(magic_number) // chunk offsets, Adam7 tables and filter arithmetic
// PNG: signature, then chunks of length, type, data and CRC-32. The image data is the zlib stream
// formed by the IDAT chunks, holding filtered scanlines (each a filter-type byte and the row).

// Decodes a PNG to the nearest pixel format: GRAY8, GRAY_ALPHA8, RGB8 or RGBA8. Palettes are
// expanded to RGB8 (RGBA8 with a tRNS chunk), a tRNS colour key on gray or RGB adds alpha, and
// 1, 2 and 4-bit gray is scaled to 8 bits. 16-bit samples keep their high byte (truncation).
// Adam7 interlacing is supported. Gamma, colour profiles and other ancillary chunks are ignored,
// and of an animated PNG only the default image is decoded. Chunk CRCs are checked for the
// critical chunks and tRNS.
decode_png :: (data: [] u8, max_pixels: s64 = DEFAULT_MAX_PIXELS) -> image: Image, ok: bool, error: string {
    if data.count < 8 || !png_has_signature(data) return .{}, false, "not a PNG image";

    file: Png_File;

    defer array_free(file.ranges);

    scanned, problem := png_scan(data, max_pixels, *file);
    if !scanned return .{}, false, problem;

    // Deflate cannot shrink data by more than about a thousandfold.
    header := file.header;
    raw_size := png_raw_size(header);

    if raw_size / 1032 > file.idat_total + 64 return .{}, false, "PNG image data is too short";

    // Gather the zlib stream; a single IDAT chunk is used where it lies.
    stream: [] u8;
    gathered := file.ranges.count > 1;

    if gathered {
        stream = NewArray(file.idat_total, u8);
        at := 0;

        for range: file.ranges {
            memcpy(stream.data + at, data.data + range.start, range.length);
            at += range.length;
        }
    } else {
        stream = byte_range(data, file.ranges[0].start, file.ranges[0].length);
    }

    defer if gathered array_free(stream);

    raw := NewArray(raw_size, u8, false); // inflate fills it
    written, inflated, message := inflate_into(stream, raw, .ZLIB);

    if !inflated || written != raw_size {
        array_free(raw);
        return .{}, false, ifx inflated then "PNG image data is too short" else message;
    }

    image, ok, error, kept := png_unpack(*file, raw);
    if !kept array_free(raw);

    return image, ok, error;
}

// Encodes as an 8-bit PNG: gray, gray with alpha, RGB or RGBA (BGRA is written as RGBA). Each
// row gets the filter whose output has the smallest sum of absolute values (read as signed
// bytes): estimated from a sample of the row for levels 1 to 8, over the whole row at level 9, which is slower.
// `level` is the deflate level, 0 to 9; 0 stores the rows unfiltered and uncompressed.
encode_png :: (image: Image, level: s32 = 6) -> [] u8 {
    if !image_is_valid(image) return empty_bytes();

    source := image;
    converted := false;

    if image.format == .BGRA8 {
        source = convert(image, .RGBA8);
        converted = true;
    }

    defer if converted deinit(*source);

    bpp := bytes_per_pixel(source.format);
    width := cast(s64) source.width;
    height := cast(s64) source.height;
    stride := width * bpp;

    color_type: u8 = 6;

    if source.format == .GRAY8 {
        color_type = 0;
    } else if source.format == .GRAY_ALPHA8 {
        color_type = 4;
    } else if source.format == .RGB8 {
        color_type = 2;
    }

    filtered := NewArray((stride + 1) * height, u8);

    defer array_free(filtered);

    if level <= 0 {
        for y: 0..height - 1 {
            filtered[y * (stride + 1)] = 0;
            memcpy(filtered.data + y * (stride + 1) + 1, source.pixels.data + y * stride, stride);
        }
    } else {
        png_filter_image(source, filtered, bpp, level >= PNG_THOROUGH_LEVEL);
    }

    packed := deflate(filtered, .ZLIB, level);

    defer array_free(packed);
    if packed.count > 0x7fffffff return empty_bytes();

    out := NewArray(8 + 25 + 12 + packed.count + 12, u8);

    memcpy(out.data, PNG_SIGNATURE.data, 8);

    at := 8;

    png_put32(out, at, 13);
    memcpy(out.data + at + 4, "IHDR".data, 4);
    png_put32(out, at + 8, width);
    png_put32(out, at + 12, height);
    out[at + 16] = 8;
    out[at + 17] = color_type;
    out[at + 18] = 0;
    out[at + 19] = 0;
    out[at + 20] = 0;
    png_put32(out, at + 21, crc32(byte_range(out, at + 4, 17)));
    at += 25;

    png_put32(out, at, packed.count);
    memcpy(out.data + at + 4, "IDAT".data, 4);
    memcpy(out.data + at + 8, packed.data, packed.count);
    png_put32(out, at + 8 + packed.count, crc32(byte_range(out, at + 4, packed.count + 4)));
    at += packed.count + 12;

    png_put32(out, at, 0);
    memcpy(out.data + at + 4, "IEND".data, 4);
    png_put32(out, at + 8, crc32(byte_range(out, at + 4, 4)));

    return out;
}

#scope_file

PNG_SIGNATURE :: "\x89PNG\r\n\x1a\n";

PNG_IHDR :: 0x49484452;
PNG_PLTE :: 0x504c5445;
PNG_IDAT :: 0x49444154;
PNG_IEND :: 0x49454e44;
PNG_TRNS :: 0x74524e53;

PNG_PASS_X :: s64.[0, 4, 0, 2, 0, 1, 0];
PNG_PASS_Y :: s64.[0, 0, 4, 0, 2, 0, 1];
PNG_PASS_DX :: s64.[8, 8, 4, 4, 2, 2, 1];
PNG_PASS_DY :: s64.[8, 8, 8, 4, 4, 2, 2];

Png_Header :: struct {
    width: s32;
    height: s32;
    depth: s64;
    color_type: s64;
    interlace: s64;
    bits_per_pixel: s64;
}

Png_Range :: struct {
    start: s64;
    length: s64;
}

// What turning an unfiltered row into output pixels needs.
Png_Decode :: struct {
    header: Png_Header;
    format: Pixel_Format;
    out_bytes: s64;
    have_trns: bool;
    key: [3] s32;
    palette: *[256] u32;
}

// Everything the chunks before the image data say.
Png_File :: struct {
    header: Png_Header;
    palette: [256] u32; // r | g << 8 | b << 16 | a << 24
    palette_count: s64;
    have_trns: bool;
    key: [3] s32; // tRNS colour key for gray (key[0]) and RGB
    ranges: [..] Png_Range;
    idat_total: s64;
}

// Walks the chunks, checking their checksums and order, and fills in `file`.
png_scan :: (data: [] u8, max_pixels: s64, file: *Png_File) -> bool, string {
    have_header := false;
    seen_end := false;
    pos := 8;

    while pos < data.count {
        if pos + 12 > data.count return false, "truncated PNG chunk";

        length := png_be32(data, pos);
        kind := png_be32(data, pos + 4);

        if length > 0x7fffffff || pos + 12 + length > data.count return false, "truncated PNG chunk";

        body := pos + 8;
        critical := (data[pos + 4] & 0x20) == 0;

        if critical || kind == PNG_TRNS {
            if crc32(byte_range(data, pos + 4, length + 4)) != cast,trunc(u32) png_be32(data, body + length) return false, "PNG chunk checksum mismatch";
        }

        if !have_header && kind != PNG_IHDR return false, "PNG does not start with a header";

        if kind == PNG_IHDR {
            if have_header || length != 13 return false, "invalid PNG header";
            valid, problem := png_parse_header(data, body, max_pixels, *file.header);
            if !valid return false, problem;
            have_header = true;
        } else if kind == PNG_PLTE {
            problem := png_read_palette(data, body, length, file);
            if problem.count > 0 return false, problem;
        } else if kind == PNG_TRNS {
            problem := png_read_transparency(data, body, length, file);
            if problem.count > 0 return false, problem;
        } else if kind == PNG_IDAT {
            if file.header.color_type == 3 && file.palette_count == 0 return false, "PNG palette is missing";
            array_add(*file.ranges, .{ body, length });
            file.idat_total += length;
        } else if kind == PNG_IEND {
            seen_end = true;
            break;
        } else if critical {
            return false, "unsupported critical PNG chunk";
        }

        pos = body + length + 4;
    }

    if !have_header return false, "PNG does not start with a header";
    if file.ranges.count == 0 return false, "PNG has no image data";
    if !seen_end return false, "PNG has no end chunk";

    return true, "";
}

// PLTE matters only for palette images (other colour types may carry it as a hint).
png_read_palette :: (data: [] u8, body: s64, length: s64, file: *Png_File) -> string {
    if file.header.color_type != 3 return "";

    count := length / 3;
    if file.palette_count != 0 || file.ranges.count != 0 || length == 0 || length % 3 != 0 || count > 256 || count > (1 << file.header.depth) return "invalid PNG palette";

    file.palette_count = count;

    for i: 0..count - 1 {
        file.palette[i] = cast(u32) data[body + i * 3] | ((cast(u32) data[body + i * 3 + 1]) << 8) | ((cast(u32) data[body + i * 3 + 2]) << 16) | 0xff000000;
    }

    return "";
}

png_read_transparency :: (data: [] u8, body: s64, length: s64, file: *Png_File) -> string {
    color_type := file.header.color_type;

    if color_type == 0 {
        if length != 2 return "invalid PNG transparency";
        file.key[0] = cast(s32) png_be16(data, body);
        file.have_trns = true;
    } else if color_type == 2 {
        if length != 6 return "invalid PNG transparency";
        for 0..2 file.key[it] = cast(s32) png_be16(data, body + it * 2);
        file.have_trns = true;
    } else if color_type == 3 {
        if length > file.palette_count return "invalid PNG transparency";

        for i: 0..length - 1 {
            file.palette[i] = (file.palette[i] & 0x00ffffff) | ((cast(u32) data[body + i]) << 24);
        }

        file.have_trns = length > 0;
    }

    return "";
}

// Unfilters `raw` and turns it into pixels. When the rows can stay as they are, the buffer itself
// becomes the pixels and `kept` says so.
png_unpack :: (file: *Png_File, raw: [] u8) -> image: Image, ok: bool, error: string, kept: bool {
    header := file.header;
    have_trns := file.have_trns;

    // Output format, and the table that turns palette indices (or low-depth gray) into pixels.
    format: Pixel_Format;

    if header.color_type == 0 {
        format = ifx have_trns then Pixel_Format.GRAY_ALPHA8 else .GRAY8;
    } else if header.color_type == 2 || header.color_type == 3 {
        format = ifx have_trns then Pixel_Format.RGBA8 else .RGB8;
    } else if header.color_type == 4 {
        format = .GRAY_ALPHA8;
    } else {
        format = .RGBA8;
    }

    if header.color_type == 0 && header.depth <= 8 {
        levels := (1 << header.depth) - 1;

        for i: 0..levels {
            g := cast(u32) (i * 255 / levels);
            alpha: u32 = ifx have_trns && i == file.key[0] then 0 else 255;
            file.palette[i] = g | (alpha << 24);
        }
    }

    ctx: Png_Decode;
    ctx.header = header;
    ctx.format = format;
    ctx.out_bytes = bytes_per_pixel(format);
    ctx.have_trns = have_trns;
    ctx.key = file.key;
    ctx.palette = *file.palette;

    width := cast(s64) header.width;
    height := cast(s64) header.height;
    bits := header.bits_per_pixel;
    bpp := max(bits / 8, 1);

    zero := NewArray((width * bits + 7) / 8 + 8, u8);

    defer array_free(zero);

    // 8-bit samples that need no change: unfilter each row straight into its final place and
    // keep the buffer as the pixels.
    if header.interlace == 0 && header.depth == 8 && !have_trns && header.color_type != 3 {
        stride := width * bpp;
        previous := zero.data;

        for y: 0..height - 1 {
            src := raw.data + y * (stride + 1);
            dst := raw.data + y * stride;
            if !png_unfilter(src[0], bpp, src + 1, dst, previous, stride) return .{}, false, "invalid PNG filter", false;
            previous = dst;
        }

        image: Image;
        image.width = header.width;
        image.height = header.height;
        image.format = format;
        image.pixels.data = raw.data;
        image.pixels.count = stride * height;

        return image, true, "", true;
    }

    image := make_image(header.width, header.height, format);
    ok, message := png_unpack_rows(*ctx, raw, *image, zero.data);

    if !ok {
        deinit(*image);
        return .{}, false, message, false;
    }

    return image, true, "", false;
}

// Unfilters each row in place and expands it into `image`, pass by pass when interlaced.
png_unpack_rows :: (ctx: *Png_Decode, raw: [] u8, image: *Image, zero: *u8) -> bool, string {
    header := ctx.header;
    width := cast(s64) header.width;
    height := cast(s64) header.height;
    bits := header.bits_per_pixel;
    bpp := max(bits / 8, 1);
    out_bytes := ctx.out_bytes;

    if header.interlace == 0 {
        row_bytes := (width * bits + 7) / 8;
        previous := zero;

        for y: 0..height - 1 {
            src := raw.data + y * (row_bytes + 1);
            if !png_unfilter(src[0], bpp, src + 1, src + 1, previous, row_bytes) return false, "invalid PNG filter";
            png_expand_row(ctx, src + 1, width, image.pixels.data + y * width * out_bytes);
            previous = src + 1;
        }

        return true, "";
    }

    // Adam7: seven reduced images, each filtered on its own, scattered into the full image.
    line := NewArray(width * out_bytes, u8);

    defer array_free(line);
    at := 0;

    for pass: 0..6 {
        pw := (width - PNG_PASS_X[pass] + PNG_PASS_DX[pass] - 1) / PNG_PASS_DX[pass];
        ph := (height - PNG_PASS_Y[pass] + PNG_PASS_DY[pass] - 1) / PNG_PASS_DY[pass];

        if pw <= 0 || ph <= 0 continue;

        row_bytes := (pw * bits + 7) / 8;
        previous := zero;

        for y: 0..ph - 1 {
            src := raw.data + at;

            at += row_bytes + 1;

            if !png_unfilter(src[0], bpp, src + 1, src + 1, previous, row_bytes) return false, "invalid PNG filter";

            previous = src + 1;

            png_expand_row(ctx, src + 1, pw, line.data);
            row := image.pixels.data + (PNG_PASS_Y[pass] + y * PNG_PASS_DY[pass]) * width * out_bytes;

            for x: 0..pw - 1 {
                memcpy(row + (PNG_PASS_X[pass] + x * PNG_PASS_DX[pass]) * out_bytes, line.data + x * out_bytes, out_bytes);
            }
        }
    }

    return true, "";
}

png_has_signature :: (data: [] u8) -> bool {
    for 0..7 if data[it] != PNG_SIGNATURE[it] return false;
    return true;
}

png_be16 :: (data: [] u8, at: s64) -> s64 {
    return ((cast(s64) data[at]) << 8) | data[at + 1];
}

png_be32 :: (data: [] u8, at: s64) -> s64 {
    return ((cast(s64) data[at]) << 24) | ((cast(s64) data[at + 1]) << 16) | ((cast(s64) data[at + 2]) << 8) | data[at + 3];
}

png_put32 :: (data: [] u8, at: s64, value: s64) {
    data[at] = cast,trunc(u8) (value >> 24);
    data[at + 1] = cast,trunc(u8) (value >> 16);
    data[at + 2] = cast,trunc(u8) (value >> 8);
    data[at + 3] = cast,trunc(u8) value;
}

png_parse_header :: (data: [] u8, at: s64, max_pixels: s64, header: *Png_Header) -> bool, string {
    width := png_be32(data, at);
    height := png_be32(data, at + 4);
    depth := cast(s64) data[at + 8];
    color_type := cast(s64) data[at + 9];

    if !image_size_ok(width, height, max_pixels) return false, "invalid or too large PNG image size";

    valid := false;

    if color_type == 0 {
        valid = depth == 1 || depth == 2 || depth == 4 || depth == 8 || depth == 16;
    } else if color_type == 3 {
        valid = depth == 1 || depth == 2 || depth == 4 || depth == 8;
    } else if color_type == 2 || color_type == 4 || color_type == 6 {
        valid = depth == 8 || depth == 16;
    }

    if !valid return false, "invalid PNG colour type or bit depth";
    if data[at + 10] != 0 || data[at + 11] != 0 return false, "unsupported PNG compression or filter method";
    if data[at + 12] > 1 return false, "unsupported PNG interlace method";

    channels: s64 = 1;

    if color_type == 2 {
        channels = 3;
    } else if color_type == 4 {
        channels = 2;
    } else if color_type == 6 {
        channels = 4;
    }

    header.width = cast(s32) width;
    header.height = cast(s32) height;
    header.depth = depth;
    header.color_type = color_type;
    header.interlace = data[at + 12];
    header.bits_per_pixel = depth * channels;

    return true, "";
}

// Bytes of filtered scanlines: one filter byte per row, rows padded to whole bytes.
png_raw_size :: (header: Png_Header) -> s64 {
    width := cast(s64) header.width;
    height := cast(s64) header.height;
    bits := header.bits_per_pixel;

    if header.interlace == 0 return height * (1 + (width * bits + 7) / 8);

    total: s64;

    for pass: 0..6 {
        pw := (width - PNG_PASS_X[pass] + PNG_PASS_DX[pass] - 1) / PNG_PASS_DX[pass];
        ph := (height - PNG_PASS_Y[pass] + PNG_PASS_DY[pass] - 1) / PNG_PASS_DY[pass];
        if pw > 0 && ph > 0 total += ph * (1 + (pw * bits + 7) / 8);
    }

    return total;
}

// Undoes `filter` on one row. `src` and `dst` may be the same row, or `dst` may lie before `src`
// in the same buffer; `previous` is the unfiltered row above (zeros for the first row).
png_unfilter :: (filter: u8, bpp: s64, src: *u8, dst: *u8, previous: *u8, count: s64) -> bool {
    if filter > 4 return false;

    if bpp == 1 {
        png_unfilter_bpp(1, filter, src, dst, previous, count);
    } else if bpp == 2 {
        png_unfilter_bpp(2, filter, src, dst, previous, count);
    } else if bpp == 3 {
        png_unfilter_bpp(3, filter, src, dst, previous, count);
    } else if bpp == 4 {
        png_unfilter_bpp(4, filter, src, dst, previous, count);
    } else if bpp == 6 {
        png_unfilter_bpp(6, filter, src, dst, previous, count);
    } else {
        png_unfilter_bpp(8, filter, src, dst, previous, count);
    }

    return true;
}

png_unfilter_bpp :: ($BPP: s64, filter: u8, src: *u8, dst: *u8, previous: *u8, count: s64) {
    // The row is a whole number of pixels; the left neighbours of a pixel stay in registers.
    if filter == {
        case 0;
            if src != dst memcpy(dst, src, count); // copes with the overlap of a row moving up
        case 1;
            left: [BPP] u8;
            i := 0;

            while i + BPP <= count {
                for k: 0..BPP - 1 {
                    left[k] += src[i + k];
                    dst[i + k] = left[k];
                }

                i += BPP;
            }
        case 2;
            i := 0;
            while i + 16 <= count {
                store(load(u8x16, src + i) + load(u8x16, previous + i), dst + i);
                i += 16;
            }

            while i < count {
                dst[i] = src[i] + previous[i];
                i += 1;
            }
        case 3;
            left: [BPP] u32;
            i := 0;

            while i + BPP <= count {
                for k: 0..BPP - 1 {
                    left[k] = (src[i + k] + ((left[k] + previous[i + k]) >> 1)) & 255;
                    dst[i + k] = cast,trunc(u8) left[k];
                }

                i += BPP;
            }
        case 4;
            left: [BPP] u8;
            corner: [BPP] u8;
            i := 0;

            while i + BPP <= count {
                for k: 0..BPP - 1 {
                    above := previous[i + k];
                    left[k] = src[i + k] + png_paeth(left[k], above, corner[k]);
                    corner[k] = above;
                    dst[i + k] = left[k];
                }

                i += BPP;
            }
    }
}

// The Paeth predictor: whichever of left, above and upper-left is nearest to left + above -
// upper-left, ties going to left, then above. Written with selects, not branches.
png_paeth :: inline (a: u8, b: u8, c: u8) -> u8 {
    ia := cast(s32) a;
    ib := cast(s32) b;
    ic := cast(s32) c;
    pa := ib - ic;
    pb := ia - ic;
    pc := pa + pb;

    pa = ifx pa < 0 then -pa else pa;
    pb = ifx pb < 0 then -pb else pb;
    pc = ifx pc < 0 then -pc else pc;
    near := ifx pa <= pb then a else b;
    distance := ifx pa <= pb then pa else pb;

    return ifx distance <= pc then near else c;
}

// Converts `count` unfiltered pixels of a row to the output format, contiguously at `dst`.
png_expand_row :: (ctx: *Png_Decode, src: *u8, count: s64, dst: *u8) {
    header := ctx.header;

    if header.depth == 16 {
        png_expand_16(ctx, src, count, dst);
        return;
    }

    if header.color_type == 0 || header.color_type == 3 {
        if ctx.out_bytes == 1 {
            png_expand_table(1, header.depth, ctx.palette, src, count, dst);
        } else if ctx.out_bytes == 2 {
            png_expand_table(2, header.depth, ctx.palette, src, count, dst);
        } else if ctx.out_bytes == 3 {
            png_expand_table(3, header.depth, ctx.palette, src, count, dst);
        } else {
            png_expand_table(4, header.depth, ctx.palette, src, count, dst);
        }

        return;
    }

    if header.color_type == 2 && ctx.have_trns {
        // 8-bit RGB with a colour key.
        for x: 0..count - 1 {
            r := src[x * 3];
            g := src[x * 3 + 1];
            b := src[x * 3 + 2];
            d := dst + x * 4;
            d[0] = r;
            d[1] = g;
            d[2] = b;
            d[3] = ifx r == ctx.key[0] && g == ctx.key[1] && b == ctx.key[2] then cast(u8) 0 else 255;
        }

        return;
    }

    memcpy(dst, src, count * ctx.out_bytes);
}

// Gray and palette rows of at most 8 bits through a table of ready pixels (see the tables built
// in decode_png): the low byte, or the three low bytes, with alpha in the top byte.
png_expand_table :: ($OUT: s64, depth: s64, table: *[256] u32, src: *u8, count: s64, dst: *u8) {
    if depth == 8 {
        for x: 0..count - 1 {
            png_put_table_pixel(OUT, dst + x * OUT, table.*[src[x]]);
        }

        return;
    }

    mask := ((cast(s32) 1) << depth) - 1;

    for x: 0..count - 1 {
        bit := x * depth;
        index := ((cast(s32) src[bit >> 3]) >> (8 - depth - (bit & 7))) & mask;
        png_put_table_pixel(OUT, dst + x * OUT, table.*[index]);
    }
}

png_put_table_pixel :: inline ($OUT: s64, d: *u8, t: u32) {
    #if OUT == 1 {
        d[0] = cast,trunc(u8) t;
    } else #if OUT == 2 {
        d[0] = cast,trunc(u8) t;
        d[1] = cast,trunc(u8) (t >> 24);
    } else #if OUT == 3 {
        d[0] = cast,trunc(u8) t;
        d[1] = cast,trunc(u8) (t >> 8);
        d[2] = cast,trunc(u8) (t >> 16);
    } else {
        d[0] = cast,trunc(u8) t;
        d[1] = cast,trunc(u8) (t >> 8);
        d[2] = cast,trunc(u8) (t >> 16);
        d[3] = cast,trunc(u8) (t >> 24);
    }
}

// 16-bit samples: the high byte of each, with a colour key compared on the full sample.
png_expand_16 :: (ctx: *Png_Decode, src: *u8, count: s64, dst: *u8) {
    color_type := ctx.header.color_type;

    if color_type == 0 {
        if ctx.have_trns {
            for x: 0..count - 1 {
                v := ((cast(s32) src[x * 2]) << 8) | src[x * 2 + 1];
                dst[x * 2] = src[x * 2];
                dst[x * 2 + 1] = ifx v == ctx.key[0] then cast(u8) 0 else 255;
            }
        } else {
            for x: 0..count - 1 dst[x] = src[x * 2];
        }
    } else if color_type == 2 {
        if ctx.have_trns {
            for x: 0..count - 1 {
                s := src + x * 6;
                d := dst + x * 4;
                d[0] = s[0];
                d[1] = s[2];
                d[2] = s[4];
                match := (((cast(s32) s[0]) << 8) | s[1]) == ctx.key[0] && (((cast(s32) s[2]) << 8) | s[3]) == ctx.key[1] && (((cast(s32) s[4]) << 8) | s[5]) == ctx.key[2];

                d[3] = ifx match then cast(u8) 0 else 255;
            }
        } else {
            for x: 0..count - 1 {
                s := src + x * 6;
                d := dst + x * 3;
                d[0] = s[0];
                d[1] = s[2];
                d[2] = s[4];
            }
        }
    } else if color_type == 4 {
        for x: 0..count - 1 {
            dst[x * 2] = src[x * 4];
            dst[x * 2 + 1] = src[x * 4 + 2];
        }
    } else {
        for x: 0..count - 1 {
            s := src + x * 8;
            d := dst + x * 4;
            d[0] = s[0];
            d[1] = s[2];
            d[2] = s[4];
            d[3] = s[6];
        }
    }
}

// The level from which every filter is tried on each row; below it only Sub and Up are.
PNG_THOROUGH_LEVEL :: 9;

// Filters every row, keeping the filter with the smallest sum of absolute differences from zero.
png_filter_image :: (source: Image, filtered: [] u8, bpp: s64, thorough: bool) {
    if thorough {
        if bpp == 1 {
            png_filter_rows(1, source, filtered);
        } else if bpp == 2 {
            png_filter_rows(2, source, filtered);
        } else if bpp == 3 {
            png_filter_rows(3, source, filtered);
        } else {
            png_filter_rows(4, source, filtered);
        }
    } else {
        if bpp == 1 {
            png_filter_rows_fast(1, source, filtered);
        } else if bpp == 2 {
            png_filter_rows_fast(2, source, filtered);
        } else if bpp == 3 {
            png_filter_rows_fast(3, source, filtered);
        } else {
            png_filter_rows_fast(4, source, filtered);
        }
    }
}

// How many bytes of a row are compared to pick between Sub and Up.
PNG_FILTER_SAMPLES :: 256;

// Sub, Up, Average or Paeth for each row. The choice compares the filtered bytes of all four at a
// spread of positions along the row (the sum of their distances from zero, as signed bytes). Sub
// and Up then filter the row eight bytes at a time; Average and Paeth go byte by byte, and only
// rows where they won pay for it. On the first line Up is the row itself.
png_filter_rows_fast :: ($BPP: s64, source: Image, filtered: [] u8) {
    width := cast(s64) source.width;
    height := cast(s64) source.height;
    stride := width * BPP;

    // How far a filtered byte is from zero, reading it as a signed byte.
    cost: [256] u8;

    for 0..255 cost[it] = cast,trunc(u8) (ifx it < 128 then it else 256 - it);

    cost_at := cost.data;
    zero := NewArray(stride, u8);

    defer array_free(zero);

    // A step that is a multiple of the pixel size would only ever look at one channel.
    step := max(stride / PNG_FILTER_SAMPLES, 1);
    if BPP > 1 && step % BPP == 0 step += 1;

    for y: 0..height - 1 {
        row := source.pixels.data + y * stride;
        previous := ifx y == 0 then zero.data else row - stride;
        out := filtered.data + y * (stride + 1);

        sum_sub, sum_up, sum_average, sum_paeth: s64;
        i := BPP;

        while i < stride {
            x := row[i];
            a := row[i - BPP];
            b := previous[i];
            c := previous[i - BPP];

            sum_sub += cost_at[x - a];
            sum_up += cost_at[x - b];
            sum_average += cost_at[x - cast,trunc(u8) ((cast(u32) a + b) >> 1)];
            sum_paeth += cost_at[x - png_paeth(a, b, c)];
            i += step;
        }

        best := sum_sub;
        kind: u8 = 1;

        if sum_up < best {
            best = sum_up;
            kind = 2;
        }

        if sum_average < best {
            best = sum_average;
            kind = 3;
        }

        if sum_paeth < best {
            best = sum_paeth;
            kind = 4;
        }

        out[0] = kind;
        dst := out + 1;

        if kind == 1 {
            memcpy(dst, row, BPP);
            png_subtract_bytes(row + BPP, row, dst + BPP, stride - BPP);
        } else if kind == 2 {
            png_subtract_bytes(row, previous, dst, stride);
        } else if kind == 3 {
            for k: 0..BPP - 1 dst[k] = row[k] - (previous[k] >> 1);

            for k: BPP..stride - 1 dst[k] = row[k] - cast,trunc(u8) ((cast(u32) row[k - BPP] + previous[k]) >> 1);
        } else {
            for k: 0..BPP - 1 dst[k] = row[k] - previous[k];

            for k: BPP..stride - 1 {
                a := cast(s32) row[k - BPP];
                b := cast(s32) previous[k];
                c := cast(s32) previous[k - BPP];
                pa := b - c;
                pb := a - c;
                pc := pa + pb;

                pa = ifx pa < 0 then -pa else pa;
                pb = ifx pb < 0 then -pb else pb;
                pc = ifx pc < 0 then -pc else pc;

                near := ifx pa <= pb then a else b;
                distance := ifx pa <= pb then pa else pb;

                dst[k] = row[k] - cast,trunc(u8) (ifx distance <= pc then near else c);
            }
        }
    }
}

// dst[i] = a[i] - b[i] for `count` bytes, eight at a time (each byte subtracted on its own, with
// no borrow into its neighbour).
png_subtract_bytes :: (a: *u8, b: *u8, dst: *u8, count: s64) {
    high: u64 = 0x8080808080808080;
    i: s64;

    while i + 8 <= count {
        x := (cast(*u64) (a + i)).*;
        y := (cast(*u64) (b + i)).*;

        (cast(*u64) (dst + i)).* = ((x | high) - (y & ~high)) ^ ((x ^ ~y) & high);
        i += 8;
    }

    while i < count {
        dst[i] = a[i] - b[i];
        i += 1;
    }
}

png_filter_rows :: ($BPP: s64, source: Image, filtered: [] u8) {
    width := cast(s64) source.width;
    height := cast(s64) source.height;
    stride := width * BPP;

    // How far a filtered byte is from zero, reading it as a signed byte.
    cost: [256] u8;

    for 0..255 cost[it] = cast,trunc(u8) (ifx it < 128 then it else 256 - it);

    zero := NewArray(stride, u8);
    candidates := NewArray(stride * 4, u8);

    defer array_free(zero);
    defer array_free(candidates);

    for y: 0..height - 1 {
        row := source.pixels.data + y * stride;
        previous := ifx y == 0 then zero.data else row - stride;
        out := filtered.data + y * (stride + 1);

        sub := candidates.data;
        up := sub + stride;
        average := up + stride;
        paeth := average + stride;

        sum_none, sum_sub, sum_up, sum_average, sum_paeth: s64;

        for i: 0..BPP - 1 {
            x := row[i];
            b := previous[i];

            sub[i] = x;
            up[i] = x - b;
            average[i] = x - (b >> 1);
            paeth[i] = x - b;
            sum_none += cost[x];
            sum_sub += cost[sub[i]];
            sum_up += cost[up[i]];
            sum_average += cost[average[i]];
            sum_paeth += cost[paeth[i]];
        }

        for i: BPP..stride - 1 {
            x := row[i];
            a := row[i - BPP];
            b := previous[i];
            c := previous[i - BPP];

            s := x - a;
            u := x - b;
            v := x - cast,trunc(u8) ((cast(u32) a + b) >> 1);
            p := x - png_paeth(a, b, c);

            sub[i] = s;
            up[i] = u;
            average[i] = v;
            paeth[i] = p;
            sum_none += cost[x];
            sum_sub += cost[s];
            sum_up += cost[u];
            sum_average += cost[v];
            sum_paeth += cost[p];
        }

        best: u8 = 0;
        best_sum := sum_none;
        best_row := row;

        if sum_sub < best_sum {
            best = 1;
            best_sum = sum_sub;
            best_row = sub;
        }

        if sum_up < best_sum {
            best = 2;
            best_sum = sum_up;
            best_row = up;
        }

        if sum_average < best_sum {
            best = 3;
            best_sum = sum_average;
            best_row = average;
        }

        if sum_paeth < best_sum {
            best = 4;
            best_sum = sum_paeth;
            best_row = paeth;
        }

        out[0] = best;
        memcpy(out + 1, best_row, stride);
    }
}
