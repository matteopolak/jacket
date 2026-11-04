// Scales an 8-bit picture to a square without any native library. Colour is averaged in linear
// light (the channels are sRGB), weighted by alpha so transparent pixels do not tint their neighbours,
// and the result is re-encoded to sRGB. The kernel is a tent (bilinear) that widens when shrinking,
// so every source pixel contributes. Edges clamp.
//
// `comp` is 3 or 4 channels per pixel; with 4, channel 3 is alpha and stays linear.
// Returns false if memory ran out.
resize_square :: (source: *u8, width: s64, height: s64, comp: s64, target: *u8, size: s64) -> bool {
    decode: [256] float;
    for value: 0..255 decode[value] = srgb_to_linear(value / 255.0);

    encode: [ENCODE_STEPS + 1] u8;
    level := 0;

    for step: 0..ENCODE_STEPS {
        // Pick the byte whose linear value is nearest: advance while the next byte is closer.
        linear := step / cast(float) ENCODE_STEPS;

        while level < 255 && linear >= (decode[level] + decode[level + 1]) * 0.5 level += 1;
        encode[step] = cast(u8) level;
    }

    columns := make_taps(width, size);
    rows := make_taps(height, size);
    // Rows of the source, already shrunk across and holding colour * alpha.
    middle := cast(*float) B.alloc(height * size * comp * size_of(float));

    defer B.free(middle);

    if !columns.starts || !rows.starts || !middle {
        B.free(columns.starts);
        B.free(columns.weights);
        B.free(rows.starts);
        B.free(rows.weights);

        return false;
    }

    defer {
        B.free(columns.starts);
        B.free(columns.weights);
        B.free(rows.starts);
        B.free(rows.weights);
    }

    has_alpha := comp == 4;

    for y: 0..height - 1 {
        line := source + y * width * comp;

        for x: 0..size - 1 {
            sums: [4] float;
            tap := columns.starts[x];
            weights := columns.weights + x * columns.stride;

            for k: 0..columns.counts[x] - 1 {
                pixel := line + (tap + k) * comp;
                weight := weights[k];
                alpha := 1.0;

                if has_alpha {
                    alpha = pixel[3] / 255.0;
                    sums[3] += weight * alpha;
                }

                weight *= alpha;
                for c: 0..2 sums[c] += weight * decode[pixel[c]];
            }

            out := middle + (y * size + x) * comp;

            memcpy(out, sums.data, comp * size_of(float));
        }
    }

    for y: 0..size - 1 {
        weights := rows.weights + y * rows.stride;
        first := rows.starts[y];

        for x: 0..size - 1 {
            sums: [4] float;

            for k: 0..rows.counts[y] - 1 {
                pixel := middle + ((first + k) * size + x) * comp;
                for c: 0..comp - 1 sums[c] += weights[k] * pixel[c];
            }

            out := target + (y * size + x) * comp;
            alpha := 1.0;

            if has_alpha {
                alpha = sums[3];
                out[3] = to_byte(alpha);
            }

            for c: 0..2 {
                linear := ifx alpha > 0.0 then sums[c] / alpha else 0.0;
                if linear < 0.0 linear = 0.0;
                if linear > 1.0 linear = 1.0;
                out[c] = encode[cast(s64) (linear * ENCODE_STEPS + 0.5)];
            }
        }
    }

    return true;
}

#scope_file
B :: #import "Basic";
M :: #import "Math";

ENCODE_STEPS :: 8192;

// For each output position, which source pixels feed it and how much each counts.
Taps :: struct {
    starts: *s64;
    counts: *s64;
    weights: *float;
    stride: s64;
}

srgb_to_linear :: (value: float) -> float {
    if value <= 0.04045 return value / 12.92;
    return cast(float) M.pow(cast(float64) ((value + 0.055) / 1.055), 2.4);
}

to_byte :: (value: float) -> u8 {
    if value <= 0.0 return 0;
    if value >= 1.0 return 255;
    return cast(u8) (value * 255.0 + 0.5);
}

make_taps :: (from: s64, to: s64) -> Taps {
    scale := from / cast(float) to;
    radius := ifx scale > 1.0 then scale else 1.0;
    stride := cast(s64) M.ceil(radius * 2.0) + 2;
    result: Taps;
    result.stride = stride;
    block := cast(*s64) B.alloc(to * 2 * size_of(s64));

    result.weights = cast(*float) B.alloc(to * stride * size_of(float));

    if !block || !result.weights {
        B.free(block);
        B.free(result.weights);
        return .{};
    }

    result.starts = block;
    result.counts = block + to;

    for i: 0..to - 1 {
        centre := (i + 0.5) * scale - 0.5;
        first := cast(s64) M.floor(centre - radius) + 1;
        last := cast(s64) M.ceil(centre + radius) - 1;
        total := 0.0;
        weights := result.weights + i * stride;
        count := 0;

        // Taps outside the picture fold onto the edge pixel, which keeps the weights summing to one.
        start := (ifx first < 0 then 0 else first);

        result.starts[i] = start;

        for j: first..last {
            distance := (j - centre) / radius;
            if distance < 0.0 distance = -distance;

            weight := 1.0 - distance;
            if weight <= 0.0 continue;

            at := clamp_index(j, from) - start;
            while count <= at {
                weights[count] = 0.0;
                count += 1;
            }

            weights[at] += weight;
            total += weight;
        }

        if count == 0 {
            // A degenerate window: fall back to the nearest source pixel.
            weights[0] = 1.0;
            result.starts[i] = clamp_index(cast(s64) M.floor(centre + 0.5), from);
            count = 1;
            total = 1.0;
        }

        for k: 0..count - 1 weights[k] /= total;

        result.counts[i] = count;
    }

    return result;
}

clamp_index :: (value: s64, count: s64) -> s64 {
    if value < 0 return 0;
    if value >= count return count - 1;
    return value;
}
