// JPEG decoder: baseline, extended sequential and progressive Huffman-coded images with 8-bit
// samples. Written from ITU-T T.81 and the JFIF and Adobe APP14 conventions.
//
// decode_jpeg parses the marker segments, decodes every scan into per-component sample planes
// (jpeg_scan.jai, jpeg_huffman.jai), then upsamples and colour-converts row by row
// (jpeg_output.jai). Arithmetic coding, 12-bit samples, lossless and hierarchical streams are
// refused.
//
// Damaged input never reads or writes out of bounds and always terminates. A stream that ends in
// the middle of the entropy-coded data still decodes: blocks that were not reached keep a flat
// mid-grey, and a restart interval whose data is cut short is skipped. Headers that are cut off,
// or a file without a single scan, are errors.

#load "jpeg_huffman.jai";
#load "jpeg_scan.jai";
#load "jpeg_output.jai";

decode_jpeg :: (data: [] u8, max_pixels: s64) -> image: Image, ok: bool, error: string {
    if data.count < 4 || data[0] != 0xFF || data[1] != 0xD8 return .{}, false, "not a JPEG image";

    d: *Jpeg_Decoder = alloc(size_of(Jpeg_Decoder));

    memset(d, 0, size_of(Jpeg_Decoder));
    defer jpeg_release(d);

    d.data = data.data;
    d.end = data.count;
    d.pos = 2;
    d.adobe_transform = -1;

    error := jpeg_read_segments(d, max_pixels);
    if error.count return .{}, false, error;

    image := make_image(d.width, d.height, ifx d.ncomp == 1 then Pixel_Format.GRAY8 else Pixel_Format.RGB8);

    jpeg_write_pixels(d, image.pixels.data);

    return image, true, "";
}

JPEG_MAX_COMPONENTS :: 4;

Jpeg_Component :: struct {
    id: s32;
    h: s32;
    v: s32;
    quant_id: s32;
    // Size in samples, and in blocks for a scan that holds this component alone.
    width: s32;
    height: s32;
    blocks_w: s32;
    blocks_h: s32;
    // Blocks allocated per row and per column, rounded up to whole MCUs.
    alloc_w: s32;
    alloc_h: s32;
    stride: s64;
    plane: *u8;
    coefs: *s16;
    pred: s32;
    quant_taken: bool;
    quant: [64] s32; // dequantizers in zigzag order
    quant_natural: [64] s32;
}

Jpeg_Decoder :: struct {
    data: *u8;
    end: s64;
    pos: s64;

    // Entropy-coded segment reader.
    bits: u64;
    nbits: s64;
    pad: s64;
    marker: u8;
    eobrun: s32;

    width: s32;
    height: s32;
    ncomp: s32;
    hmax: s32;
    vmax: s32;
    mcus_x: s32;
    mcus_y: s32;
    progressive: bool;
    frame_seen: bool;
    scans: s32;
    restart_interval: s32;
    adobe_transform: s32;
    adobe_seen: bool;
    jfif_seen: bool;

    comps: [4] Jpeg_Component;
    quant_defined: [4] bool;
    quant_tables: [4][64] s32;
    dc_tables: [4] Jpeg_Huffman;
    ac_tables: [4] Jpeg_Huffman;

    block: [64] s32;
}

jpeg_release :: (d: *Jpeg_Decoder) {
    for 0..JPEG_MAX_COMPONENTS - 1 {
        if d.comps[it].plane free(d.comps[it].plane);
        if d.comps[it].coefs free(d.comps[it].coefs);
    }

    free(d);
}

jpeg_be16 :: inline (p: *u8) -> s32 {
    return ((cast(s32) p[0]) << 8) | p[1];
}

// Walks the marker segments after SOI. Returns an empty string on success.
jpeg_read_segments :: (d: *Jpeg_Decoder, max_pixels: s64) -> string {
    while true {
        while d.pos < d.end && d.data[d.pos] != 0xFF d.pos += 1;
        while d.pos < d.end && d.data[d.pos] == 0xFF d.pos += 1;
        if d.pos >= d.end break;

        code := d.data[d.pos];

        d.pos += 1;
        if code == 0 || code == 1 || (code >= 0xD0 && code <= 0xD8) continue;
        if code == 0xD9 break;

        if d.pos + 2 > d.end return "truncated JPEG";

        length := cast(s64) jpeg_be16(d.data + d.pos);
        if length < 2 || d.pos + length > d.end return "truncated JPEG";

        segment := d.data + d.pos + 2;
        size := length - 2;
        next := d.pos + length;

        error := "";

        if code == 0xC0 || code == 0xC1 || code == 0xC2 {
            error = jpeg_read_frame(d, segment, size, code == 0xC2, max_pixels);
        } else if code == 0xC3 {
            return "lossless JPEG is not supported";
        } else if code == 0xC5 || code == 0xC6 || code == 0xC7 || code == 0xCD || code == 0xCE || code == 0xCF {
            return "hierarchical JPEG is not supported";
        } else if code == 0xC9 || code == 0xCA || code == 0xCB || code == 0xCC {
            return "arithmetic-coded JPEG is not supported";
        } else if code == 0xC8 {
            return "unsupported JPEG process";
        } else if code == 0xC4 {
            error = jpeg_read_huffman_tables(d, segment, size);
        } else if code == 0xDB {
            error = jpeg_read_quant_tables(d, segment, size);
        } else if code == 0xDD {
            if size < 2 return "bad restart interval segment";
            d.restart_interval = jpeg_be16(segment);
        } else if code == 0xE0 {
            if size >= 5 && segment[0] == #char "J" && segment[1] == #char "F" && segment[2] == #char "I" && segment[3] == #char "F" && segment[4] == 0 d.jfif_seen = true;
        } else if code == 0xEE {
            if size >= 12 && segment[0] == #char "A" && segment[1] == #char "d" && segment[2] == #char "o" && segment[3] == #char "b" && segment[4] == #char "e" {
                d.adobe_seen = true;
                d.adobe_transform = segment[11];
            }
        } else if code == 0xDA {
            d.pos = next;
            error = jpeg_read_scan(d, segment, size);
            if error.count return error;
            continue;
        }

        if error.count return error;

        d.pos = next;
    }

    if !d.frame_seen return "JPEG has no frame header";
    if d.scans == 0 return "JPEG has no image data";
    if d.progressive jpeg_finish_progressive(d);

    return "";
}

jpeg_read_quant_tables :: (d: *Jpeg_Decoder, segment: *u8, size: s64) -> string {
    at: s64 = 0;
    while at < size {
        precision := segment[at] >> 4;
        id := segment[at] & 15;

        at += 1;
        if id > 3 || precision > 1 return "bad quantization table";

        entry_size: s64 = ifx precision == 0 then 1 else 2;
        if at + 64 * entry_size > size return "bad quantization table";

        for 0..63 {
            if precision == 0 {
                d.quant_tables[id][it] = segment[at + it];
            } else {
                d.quant_tables[id][it] = jpeg_be16(segment + at + it * 2);
            }
        }

        d.quant_defined[id] = true;
        at += 64 * entry_size;
    }

    return "";
}

jpeg_read_huffman_tables :: (d: *Jpeg_Decoder, segment: *u8, size: s64) -> string {
    at: s64 = 0;
    while at < size {
        if at + 17 > size return "bad Huffman table";

        is_ac := (segment[at] >> 4) == 1;
        id := segment[at] & 15;

        if (segment[at] >> 4) > 1 || id > 3 return "bad Huffman table";

        counts := segment + at + 1;
        total: s64 = 0;

        for 0..15 total += counts[it];
        if total > 256 || at + 17 + total > size return "bad Huffman table";

        table: *Jpeg_Huffman = ifx is_ac then *d.ac_tables[id] else *d.dc_tables[id];
        if !jpeg_build_huffman(table, counts, segment + at + 17, is_ac) return "bad Huffman table";

        at += 17 + total;
    }

    return "";
}

jpeg_read_frame :: (d: *Jpeg_Decoder, segment: *u8, size: s64, progressive: bool, max_pixels: s64) -> string {
    if d.frame_seen return "JPEG has more than one frame";
    if size < 6 return "bad frame header";
    if segment[0] != 8 return "only 8-bit JPEG is supported";

    height := jpeg_be16(segment + 1);
    width := jpeg_be16(segment + 3);
    ncomp := cast(s32) segment[5];

    if width == 0 || height == 0 return "JPEG has a zero dimension";
    if ncomp != 1 && ncomp != 3 && ncomp != 4 return "unsupported number of JPEG components";
    if size != 6 + 3 * ncomp return "bad frame header";
    if cast(s64) width * height > max_pixels return "image is larger than the pixel limit";

    hmax: s32 = 1;
    vmax: s32 = 1;

    for 0..ncomp - 1 {
        c := *d.comps[it];
        c.id = segment[6 + it * 3];
        c.h = segment[7 + it * 3] >> 4;
        c.v = segment[7 + it * 3] & 15;
        c.quant_id = segment[8 + it * 3];
        if c.h < 1 || c.h > 4 || c.v < 1 || c.v > 4 || c.quant_id > 3 return "bad frame header";

        if ncomp == 1 {
            c.h = 1;
            c.v = 1;
        }

        hmax = max(hmax, c.h);
        vmax = max(vmax, c.v);
    }

    d.width = width;
    d.height = height;
    d.ncomp = ncomp;
    d.hmax = hmax;
    d.vmax = vmax;
    d.progressive = progressive;
    d.mcus_x = (width + 8 * hmax - 1) / (8 * hmax);
    d.mcus_y = (height + 8 * vmax - 1) / (8 * vmax);

    for 0..ncomp - 1 {
        c := *d.comps[it];
        c.width = (width * c.h + hmax - 1) / hmax;
        c.height = (height * c.v + vmax - 1) / vmax;
        c.blocks_w = (c.width + 7) / 8;
        c.blocks_h = (c.height + 7) / 8;
        c.alloc_w = d.mcus_x * c.h;
        c.alloc_h = d.mcus_y * c.v;
        c.stride = cast(s64) c.alloc_w * 8;

        samples := c.stride * c.alloc_h * 8;

        c.plane = alloc(samples);
        memset(c.plane, 128, samples);

        if progressive {
            coef_bytes := cast(s64) c.alloc_w * c.alloc_h * 128;
            c.coefs = alloc(coef_bytes);
            memset(c.coefs, 0, coef_bytes);
        }
    }

    d.frame_seen = true;

    return "";
}
