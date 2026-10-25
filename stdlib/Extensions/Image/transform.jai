// Geometric transforms. All treat the channels of a pixel alike; alpha is never premultiplied
// unless premultiply_alpha says so.

// A new image of width x height. NEAREST picks the closest source pixel. BILINEAR blends the
// four pixels around each sample point (it does not widen its footprint when shrinking, so use
// BOX to shrink by much). BOX averages the exact source area under each output pixel. Channels
// are filtered independently in 14-bit fixed point, so alpha is not premultiplied: colours of
// transparent pixels bleed into their neighbours. An invalid image or size gives an empty image.
resize :: (image: Image, width: s32, height: s32, filter := Resize_Filter.BILINEAR) -> Image {
    if !image_is_valid(image) || width <= 0 || height <= 0 return .{};
    if cast(s64) width * height * bytes_per_pixel(image.format) > 0x7fffffff * 8 return .{};

    if filter == .NEAREST return resize_nearest(image, width, height);

    // Same size: nothing to filter.
    if width == image.width && height == image.height return convert(image, image.format);

    taps_x := make_taps(image.width, width, filter);
    taps_y := make_taps(image.height, height, filter);

    defer free_taps(*taps_x);
    defer free_taps(*taps_y);

    channels := bytes_per_pixel(image.format);
    result := make_image(width, height, image.format);

    // Horizontal pass into 16-bit samples (8 fractional bits), then the vertical pass.
    middle := NewArray(cast(s64) width * image.height * channels, u16);

    defer array_free(middle);

    if channels == 1 {
        resize_horizontal(1, image, middle, taps_x);
        resize_vertical(1, middle, result, taps_y);
    } else if channels == 2 {
        resize_horizontal(2, image, middle, taps_x);
        resize_vertical(2, middle, result, taps_y);
    } else if channels == 3 {
        resize_horizontal(3, image, middle, taps_x);
        resize_vertical(3, middle, result, taps_y);
    } else {
        resize_horizontal(4, image, middle, taps_x);
        resize_vertical(4, middle, result, taps_y);
    }

    return result;
}

// The width x height rectangle with its top left corner at (x, y). Not ok when any part of it is
// outside the image or it is empty.
crop :: (image: Image, x: s32, y: s32, width: s32, height: s32) -> Image, ok: bool {
    if !image_is_valid(image) || width <= 0 || height <= 0 || x < 0 || y < 0 return .{}, false;
    if cast(s64) x + width > image.width || cast(s64) y + height > image.height return .{}, false;

    result := make_image(width, height, image.format);
    bpp := bytes_per_pixel(image.format);
    row_bytes := cast(s64) width * bpp;

    for row: 0..height - 1 {
        from := (cast(s64) (y + row) * image.width + x) * bpp;
        memcpy(result.pixels.data + cast(s64) row * row_bytes, image.pixels.data + from, row_bytes);
    }

    return result, true;
}

// Mirrors the image top to bottom, in place.
flip_vertical :: (image: *Image) {
    if !image_is_valid(image.*) return;

    row_bytes := cast(s64) image.width * bytes_per_pixel(image.format);
    scratch := NewArray(row_bytes, u8);

    defer array_free(scratch);

    top := 0;
    bottom := cast(s64) image.height - 1;

    while top < bottom {
        a := image.pixels.data + top * row_bytes;
        b := image.pixels.data + bottom * row_bytes;

        memcpy(scratch.data, a, row_bytes);
        memcpy(a, b, row_bytes);
        memcpy(b, scratch.data, row_bytes);
        top += 1;
        bottom -= 1;
    }
}

// Mirrors the image left to right, in place.
flip_horizontal :: (image: *Image) {
    if !image_is_valid(image.*) return;

    bpp := bytes_per_pixel(image.format);
    width := cast(s64) image.width;

    for row: 0..image.height - 1 {
        line := image.pixels.data + cast(s64) row * width * bpp;
        left := 0;
        right := width - 1;

        while left < right {
            a := line + left * bpp;
            b := line + right * bpp;

            for 0..bpp - 1 {
                t := a[it];
                a[it] = b[it];
                b[it] = t;
            }

            left += 1;
            right -= 1;
        }
    }
}

// A new image turned a quarter turn, clockwise or counter-clockwise; its width is the old height.
rotate_90 :: (image: Image, clockwise := true) -> Image {
    if !image_is_valid(image) return .{};

    result := make_image(image.height, image.width, image.format);
    bpp := bytes_per_pixel(image.format);
    w := cast(s64) image.width;
    h := cast(s64) image.height;

    for y: 0..h - 1 {
        for x: 0..w - 1 {
            // Clockwise: (x, y) lands at (h - 1 - y, x); counter-clockwise at (y, w - 1 - x).
            nx, ny: s64;

            if clockwise {
                nx = h - 1 - y;
                ny = x;
            } else {
                nx = y;
                ny = w - 1 - x;
            }

            memcpy(result.pixels.data + (ny * h + nx) * bpp, image.pixels.data + (y * w + x) * bpp, bpp);
        }
    }

    return result;
}

// Multiplies the colour channels by alpha (rounded), in place. A no-op for formats without alpha.
premultiply_alpha :: (image: *Image) {
    if !image_is_valid(image.*) || !has_alpha(image.format) return;

    count := cast(s64) image.width * image.height;
    p := image.pixels.data;

    if image.format == .GRAY_ALPHA8 {
        for i: 0..count - 1 {
            p[i * 2] = premultiply_channel(p[i * 2], p[i * 2 + 1]);
        }
    } else {
        for i: 0..count - 1 {
            q := p + i * 4;
            a := q[3];

            q[0] = premultiply_channel(q[0], a);
            q[1] = premultiply_channel(q[1], a);
            q[2] = premultiply_channel(q[2], a);
        }
    }
}

#scope_file

// round(c * a / 255) without a division.
premultiply_channel :: inline (c: u8, a: u8) -> u8 {
    t := cast(u32) c * a + 128;
    return cast,trunc(u8) ((t + (t >> 8)) >> 8);
}

resize_nearest :: (image: Image, width: s32, height: s32) -> Image {
    result := make_image(width, height, image.format);
    bpp := bytes_per_pixel(image.format);

    columns := NewArray(width, s64);

    defer array_free(columns);

    for x: 0..width - 1 {
        columns[x] = min((cast(s64) x * 2 + 1) * image.width / (cast(s64) width * 2), cast(s64) image.width - 1) * bpp;
    }

    for y: 0..height - 1 {
        sy := min((cast(s64) y * 2 + 1) * image.height / (cast(s64) height * 2), cast(s64) image.height - 1);
        from := image.pixels.data + sy * image.width * bpp;
        to := result.pixels.data + cast(s64) y * width * bpp;

        if bpp == 4 {
            for x: 0..width - 1 {
                (cast(*u32) (to + cast(s64) x * 4)).* = (cast(*u32) (from + columns[x])).*;
            }
        } else {
            for x: 0..width - 1 {
                memcpy(to + cast(s64) x * bpp, from + columns[x], bpp);
            }
        }
    }

    return result;
}

// Where each output sample reads from: `count[o]` source positions starting at `first[o]`, with
// 14-bit weights (summing to 16384) at `weights[start[o]...]`.
Taps :: struct {
    first: [] s32;
    count: [] s32;
    start: [] s32;
    weights: [] s32;
}

WEIGHT_ONE :: 16384;

free_taps :: (taps: *Taps) {
    array_free(taps.first);
    array_free(taps.count);
    array_free(taps.start);
    array_free(taps.weights);
}

make_taps :: (source: s32, target: s32, filter: Resize_Filter) -> Taps {
    taps: Taps;
    taps.first = NewArray(target, s32);
    taps.count = NewArray(target, s32);
    taps.start = NewArray(target, s32);

    scale := cast(float64) source / target;
    list: [..] s32;

    defer array_free(list);

    for o: 0..target - 1 {
        taps.start[o] = cast(s32) list.count;

        if filter == .BILINEAR {
            // Sample point at the output pixel's centre, between two source centres.
            centre := (o + 0.5) * scale - 0.5;
            low := cast(s64) floor_to_int(centre);
            frac := centre - low;
            a := clamp(low, 0, cast(s64) source - 1);
            b := clamp(low + 1, 0, cast(s64) source - 1);

            wb := cast(s32) (frac * WEIGHT_ONE + 0.5);

            if wb < 0 wb = 0;
            if wb > WEIGHT_ONE wb = WEIGHT_ONE;

            if a == b {
                taps.first[o] = cast(s32) a;
                taps.count[o] = 1;
                array_add(*list, WEIGHT_ONE);
            } else {
                taps.first[o] = cast(s32) a;
                taps.count[o] = 2;
                array_add(*list, WEIGHT_ONE - wb);
                array_add(*list, wb);
            }
        } else {
            // BOX: the source interval [o * scale, (o + 1) * scale), weighted by overlap.
            lo := o * scale;
            hi := (o + 1) * scale;
            i0 := cast(s64) floor_to_int(lo);
            i1 := cast(s64) floor_to_int(hi);

            if i1 >= source i1 = source - 1;
            if i0 > i1 i0 = i1;

            taps.first[o] = cast(s32) i0;
            taps.count[o] = cast(s32) (i1 - i0 + 1);

            total := 0;
            heaviest := 0;
            begin := list.count;

            for i: i0..i1 {
                left := max(cast(float64) i, lo);
                right := min(cast(float64) i + 1, hi);
                w := cast(s32) (max(right - left, 0.0) / scale * WEIGHT_ONE + 0.5);

                array_add(*list, w);
                total += w;
                if w > list[begin + heaviest] heaviest = list.count - 1 - begin;
            }

            // Make the weights sum to exactly one.
            list[begin + heaviest] += cast(s32) (WEIGHT_ONE - total);
        }
    }

    taps.weights = NewArray(list.count, s32);
    memcpy(taps.weights.data, list.data, list.count * size_of(s32));

    return taps;
}

floor_to_int :: (v: float64) -> s64 {
    i := cast(s64) v;
    if cast(float64) i > v i -= 1;
    return i;
}

// One output sample of the horizontal pass, a channel at a time.
resize_sample :: inline ($C: s64, row: *u8, first: s64, n: s64, w: *s32, out: *u16) {
    acc0, acc1, acc2, acc3: s32;

    for k: 0..n - 1 {
        p := row + (first + k) * C;
        weight := w[k];

        acc0 += weight * p[0];
        #if C > 1 acc1 += weight * p[1];
        #if C > 2 acc2 += weight * p[2];
        #if C > 3 acc3 += weight * p[3];
    }

    out[0] = cast,trunc(u16) ((acc0 + 32) >> 6);
    #if C > 1 out[1] = cast,trunc(u16) ((acc1 + 32) >> 6);
    #if C > 2 out[2] = cast,trunc(u16) ((acc2 + 32) >> 6);
    #if C > 3 out[3] = cast,trunc(u16) ((acc3 + 32) >> 6);
}

resize_horizontal :: ($C: s64, image: Image, middle: [] u16, taps: Taps) {
    out_width := cast(s64) taps.first.count;
    in_width := cast(s64) image.width;
    firsts := taps.first.data;
    counts := taps.count.data;
    starts := taps.start.data;
    weights := taps.weights.data;

    for y: 0..image.height - 1 {
        row := image.pixels.data + cast(s64) y * in_width * C;
        out := middle.data + cast(s64) y * out_width * C;

        for x: 0..out_width - 1 {
            first := cast(s64) firsts[x];
            n := cast(s64) counts[x];
            w := weights + starts[x];

            #if C >= 3 {
                // Three and four channels at a time; a 3-channel load reads one byte past its
                // pixel, so the last pixels of the row take the scalar path.
                if first + n < in_width || (C == 4 && first + n <= in_width) {
                    acc := splat(u32x4, 32);

                    for k: 0..n - 1 {
                        pixel := convert(u32x4, load(Vector(u8, 4), row + (first + k) * C));
                        acc = acc + pixel * splat(u32x4, cast(u32) w[k]);
                    }

                    #if C == 4 {
                        store(convert(Vector(u16, 4), acc >> 6), out + x * C);
                    } else {
                        if x < out_width - 1 {
                            store(convert(Vector(u16, 4), acc >> 6), out + x * C);
                        } else {
                            resize_sample(C, row, first, n, w, out + x * C);
                        }
                    }

                    continue;
                }
            }

            if n == 2 {
                a := w[0];
                b := w[1];
                p := row + first * C;
                for c: 0..C - 1 out[x * C + c] = cast,trunc(u16) ((a * p[c] + b * p[C + c] + 32) >> 6);
            } else {
                resize_sample(C, row, first, n, w, out + x * C);
            }
        }
    }
}

// The vertical pass, eight samples at a time: each output row is a weighted sum of a few rows of
// 16-bit samples with 8 fractional bits, and 14 bits of weight.
resize_vertical :: ($C: s64, middle: [] u16, result: Image, taps: Taps) {
    row_len := cast(s64) result.width * C;

    for y: 0..result.height - 1 {
        first := cast(s64) taps.first[y];
        n := cast(s64) taps.count[y];
        w := taps.weights.data + taps.start[y];
        out := result.pixels.data + cast(s64) y * row_len;
        rows := middle.data + first * row_len;

        i := 0;

        while i + 8 <= row_len {
            acc := splat(u32x8, 1 << 21);

            for k: 0..n - 1 {
                acc = acc + convert(u32x8, load(u16x8, rows + k * row_len + i)) * splat(u32x8, cast(u32) w[k]);
            }

            store(convert(Vector(u8, 8), min(acc >> 22, splat(u32x8, 255))), out + i);
            i += 8;
        }

        while i < row_len {
            acc: s32 = 1 << 21;
            for k: 0..n - 1 acc += w[k] * rows[k * row_len + i];

            v := acc >> 22;

            out[i] = cast,trunc(u8) (ifx v > 255 then 255 else v);
            i += 1;
        }
    }
}
