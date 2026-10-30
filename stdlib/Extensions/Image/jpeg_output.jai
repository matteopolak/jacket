// Inverse DCT, chroma upsampling and colour conversion of the JPEG decoder.

// The 8x8 inverse DCT is a separable integer transform with 13-bit constants: a column pass
// that keeps two extra fractional bits, then a row pass that rounds, adds the level shift of 128
// and clamps. Columns and rows that hold only a DC term take a shortcut. `last` is the highest
// zigzag position that may be non-zero; a block with only a DC term becomes a flat tile.
jpeg_idct_block :: (block: *s32, last: s32, out: *u8, stride: s64) #no_abc #no_aoc {
    if last == 0 {
        flat := jpeg_clamp(((block[0] + 4) >> 3) + 128);
        for 0..7 memset(out + cast(s64) it * stride, flat, 8);
        return;
    }

    work: [64] s32 = ---;
    result: [8] s32 = ---;

    for c: 0..7 {
        p := block + c;
        if (p[8] | p[16] | p[24] | p[32] | p[40] | p[48] | p[56]) == 0 {
            dc := p[0] << 2;
            for r: 0..7 work[c + r * 8] = dc;
        } else {
            jpeg_idct_1d(p[0], p[8], p[16], p[24], p[32], p[40], p[48], p[56], *result);
            for r: 0..7 work[c + r * 8] = (result[r] + 1024) >> 11;
        }
    }

    for r: 0..7 {
        w := work.data + r * 8;
        row := out + cast(s64) r * stride;

        if (w[1] | w[2] | w[3] | w[4] | w[5] | w[6] | w[7]) == 0 {
            memset(row, jpeg_clamp(((w[0] + 16) >> 5) + 128), 8);
        } else {
            jpeg_idct_1d(w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7], *result);
            for i: 0..7 row[i] = jpeg_clamp(((result[i] + 131072) >> 18) + 128);
        }
    }
}

jpeg_clamp :: inline (v: s32) -> u8 {
    if v < 0 return 0;
    if v > 255 return 255;
    return cast,trunc(u8) v;
}

// One 8-point transform scaled by 2^13, a rotation-based factorisation: an even half over
// inputs 0, 2, 4, 6 and an odd half over 1, 3, 5, 7.
jpeg_idct_1d :: inline (i0: s32, i1: s32, i2: s32, i3: s32, i4: s32, i5: s32, i6: s32, i7: s32, o: *[8] s32) #no_abc #no_aoc {
    z := (i2 + i6) * 4433;
    r2 := z - i6 * 15137;
    r3 := z + i2 * 6270;
    sum := (i0 + i4) << 13;
    dif := (i0 - i4) << 13;
    e0 := sum + r3;
    e3 := sum - r3;
    e1 := dif + r2;
    e2 := dif - r2;

    a := i7;
    b := i5;
    c := i3;
    e := i1;
    s1 := a + e;
    s2 := b + c;
    s3 := a + c;
    s4 := b + e;
    s5 := (s3 + s4) * 9633;

    a *= 2446;
    b *= 16819;
    c *= 25172;
    e *= 12299;
    s1 *= -7373;
    s2 *= -20995;
    s3 = s3 * -16069 + s5;
    s4 = s4 * -3196 + s5;
    a += s1 + s3;
    b += s2 + s4;
    c += s2 + s3;
    e += s1 + s4;

    o.*[0] = e0 + e;
    o.*[7] = e0 - e;
    o.*[1] = e1 + c;
    o.*[6] = e1 - c;
    o.*[2] = e2 + b;
    o.*[5] = e2 - b;
    o.*[3] = e3 + a;
    o.*[4] = e3 - a;
}

Jpeg_Color :: struct {
    cr_r: [256] s32;
    cb_b: [256] s32;
    cr_g: [256] s32;
    cb_g: [256] s32;
    range: [768] u8; // clamps -256..511 to 0..255 (index + 256)
}

jpeg_make_color_tables :: (t: *Jpeg_Color) {
    for 0..255 {
        x := cast(s32) it - 128;

        t.cr_r[it] = (91881 * x + 32768) >> 16;
        t.cb_b[it] = (116130 * x + 32768) >> 16;
        t.cr_g[it] = -46802 * x;
        t.cb_g[it] = -22554 * x + 32768;
    }

    for 0..767 {
        v := it - 256;
        t.range[it] = cast,trunc(u8) clamp(v, 0, 255);
    }
}

Jpeg_Rows :: struct {
    color: Jpeg_Color;
    rows: [4] *u8;
    buffers: [4] *u8;
    sums: *s32;
}

jpeg_row_ycc :: (yy: *u8, cb: *u8, cr: *u8, out: *u8, n: s64, t: *Jpeg_Color) #no_abc {
    range := t.range.data + 256;

    for i: 0..n - 1 {
        y := cast(s32) yy[i];
        u := cb[i];
        v := cr[i];

        out[i * 3] = range[y + t.cr_r[v]];
        out[i * 3 + 1] = range[y + ((t.cb_g[u] + t.cr_g[v]) >> 16)];
        out[i * 3 + 2] = range[y + t.cb_b[u]];
    }
}

jpeg_row_rgb :: (r: *u8, g: *u8, b: *u8, out: *u8, n: s64) #no_abc {
    for i: 0..n - 1 {
        out[i * 3] = r[i];
        out[i * 3 + 1] = g[i];
        out[i * 3 + 2] = b[i];
    }
}

jpeg_scale_255 :: inline (a: s32, b: s32) -> u8 {
    t := a * b + 128;
    return cast,trunc(u8) ((t + (t >> 8)) >> 8);
}

// Four-component rows. Adobe files store inverted ink values, so the colour is the product of
// the stored value and the stored black.
jpeg_row_cmyk :: (c: *u8, m: *u8, y: *u8, k: *u8, out: *u8, n: s64) #no_abc {
    for i: 0..n - 1 {
        out[i * 3] = jpeg_scale_255(c[i], k[i]);
        out[i * 3 + 1] = jpeg_scale_255(m[i], k[i]);
        out[i * 3 + 2] = jpeg_scale_255(y[i], k[i]);
    }
}

jpeg_row_ycck :: (yy: *u8, cb: *u8, cr: *u8, k: *u8, out: *u8, n: s64, t: *Jpeg_Color) #no_abc {
    range := t.range.data + 256;

    for i: 0..n - 1 {
        y := cast(s32) yy[i];
        u := cb[i];
        v := cr[i];

        out[i * 3] = jpeg_scale_255(255 - range[y + t.cr_r[v]], k[i]);
        out[i * 3 + 1] = jpeg_scale_255(255 - range[y + ((t.cb_g[u] + t.cr_g[v]) >> 16)], k[i]);
        out[i * 3 + 2] = jpeg_scale_255(255 - range[y + t.cb_b[u]], k[i]);
    }
}

// A full-resolution row of component `c`: the plane itself, or the upsampled neighbours. Chroma
// that is halved horizontally (and vertically) gets the triangle filter, other ratios repeat
// samples.
jpeg_component_row :: (d: *Jpeg_Decoder, rows: *Jpeg_Rows, c: s32, y: s32) -> *u8 #no_abc {
    comp := *d.comps[c];
    if comp.h == d.hmax && comp.v == d.vmax return comp.plane + cast(s64) y * comp.stride;

    out := rows.buffers[c];
    n := cast(s64) comp.width;

    if comp.h * 2 == d.hmax && comp.v == d.vmax && n > 2 {
        src := comp.plane + cast(s64) y * comp.stride;

        out[0] = src[0];
        out[1] = cast,trunc(u8) ((cast(s32) src[0] * 3 + src[1] + 2) >> 2);

        for i: 1..n - 2 {
            s := cast(s32) src[i] * 3;
            out[i * 2] = cast,trunc(u8) ((s + src[i - 1] + 1) >> 2);
            out[i * 2 + 1] = cast,trunc(u8) ((s + src[i + 1] + 2) >> 2);
        }

        out[n * 2 - 2] = cast,trunc(u8) ((cast(s32) src[n - 1] * 3 + src[n - 2] + 1) >> 2);
        out[n * 2 - 1] = src[n - 1];

        return out;
    }

    sy := y >> 1;
    far := ifx (y & 1) == 0 then sy - 1 else sy + 1;

    if far < 0 far = 0;
    if far >= comp.height far = comp.height - 1;

    if comp.h * 2 == d.hmax && comp.v * 2 == d.vmax && n > 2 {
        near := comp.plane + cast(s64) sy * comp.stride;
        other := comp.plane + cast(s64) far * comp.stride;
        sums := rows.sums;

        for i: 0..n - 1 sums[i] = cast(s32) near[i] * 3 + other[i];

        out[0] = cast,trunc(u8) ((sums[0] * 4 + 8) >> 4);
        out[1] = cast,trunc(u8) ((sums[0] * 3 + sums[1] + 7) >> 4);

        for i: 1..n - 2 {
            s := sums[i] * 3;
            out[i * 2] = cast,trunc(u8) ((s + sums[i - 1] + 8) >> 4);
            out[i * 2 + 1] = cast,trunc(u8) ((s + sums[i + 1] + 7) >> 4);
        }

        out[n * 2 - 2] = cast,trunc(u8) ((sums[n - 1] * 3 + sums[n - 2] + 8) >> 4);
        out[n * 2 - 1] = cast,trunc(u8) ((sums[n - 1] * 4 + 7) >> 4);

        return out;
    }

    if comp.h == d.hmax && comp.v * 2 == d.vmax {
        near := comp.plane + cast(s64) sy * comp.stride;
        other := comp.plane + cast(s64) far * comp.stride;
        bias: s32 = cast(s32) ifx (y & 1) == 0 then 1 else 2;

        for i: 0..n - 1 out[i] = cast,trunc(u8) ((cast(s32) near[i] * 3 + other[i] + bias) >> 2);

        return out;
    }

    // Repeat samples. Ratios that do not divide evenly pick the nearest source sample.
    source_row := cast(s64) y * comp.v / d.vmax;
    if source_row >= comp.height source_row = comp.height - 1;

    src := comp.plane + source_row * comp.stride;

    if d.hmax % comp.h == 0 {
        ratio := cast(s64) (d.hmax / comp.h);

        for i: 0..n - 1 {
            for j: 0..ratio - 1 out[i * ratio + j] = src[i];
        }
    } else {
        for i: 0..cast(s64) d.width - 1 out[i] = src[i * comp.h / d.hmax];
    }

    return out;
}

// Writes the finished image: 1 byte per pixel for one component, else RGB.
jpeg_write_pixels :: (d: *Jpeg_Decoder, out: *u8) {
    width := cast(s64) d.width;

    if d.ncomp == 1 {
        comp := *d.comps[0];
        for y: 0..d.height - 1 memcpy(out + cast(s64) y * width, comp.plane + cast(s64) y * comp.stride, width);
        return;
    }

    rows: *Jpeg_Rows = alloc(size_of(Jpeg_Rows));

    memset(rows, 0, size_of(Jpeg_Rows));
    defer free(rows);
    jpeg_make_color_tables(*rows.color);
    // Upsampled rows can overshoot the width by a few samples.
    for 0..d.ncomp - 1 rows.buffers[it] = alloc(width + 64);

    defer for 0..d.ncomp - 1 free(rows.buffers[it]);
    rows.sums = alloc((width + 64) * 4);
    defer free(rows.sums);

    ycc := true;

    if d.ncomp == 3 {
        if d.adobe_seen {
            ycc = d.adobe_transform != 0;
        } else if !d.jfif_seen && d.comps[0].id == #char "R" && d.comps[1].id == #char "G" && d.comps[2].id == #char "B" {
            ycc = false;
        }
    } else {
        ycc = d.adobe_seen && d.adobe_transform == 2;
    }

    for y: 0..d.height - 1 {
        for 0..d.ncomp - 1 rows.rows[it] = jpeg_component_row(d, rows, cast(s32) it, y);

        dst := out + cast(s64) y * width * 3;
        r := rows.rows;

        if d.ncomp == 3 {
            if ycc jpeg_row_ycc(r[0], r[1], r[2], dst, width, *rows.color);
            else jpeg_row_rgb(r[0], r[1], r[2], dst, width);
        } else {
            if ycc jpeg_row_ycck(r[0], r[1], r[2], r[3], dst, width, *rows.color);
            else jpeg_row_cmyk(r[0], r[1], r[2], r[3], dst, width);
        }
    }
}
