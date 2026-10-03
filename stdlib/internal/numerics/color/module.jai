Interpolation :: struct {
    color_space: enum u8 {
        RGB :: 0;
        HSV :: 1;
        OKLAB :: 2;
    }
    geometry: enum u8 {
        LINEAR :: 0;
        QUADRIC :: 1;
        HERMITE :: 2;
    }
    wrapping := false;
}

Color_Keyframe :: struct {
    color_rgb: Vector4;
    t: float;
}

GAMMA :: 2.2;

pow :: inline (a: Vector3, power: float) -> Vector3 {
    return .{ Math.pow(a.x, power), Math.pow(a.y, power), Math.pow(a.z, power) };
}

hermite :: (a: float, b: float, t: float) -> float {
    return lerp(a, b, t * t * (3 - 2 * t));
}

hermite :: (a: Vector3, b: Vector3, t: float) -> Vector3 {
    return lerp(a, b, t * t * (3 - 2 * t));
}

easeInOutQuad :: inline (t: float) -> float {
    if t < 0.5 return 2 * t * t;
    return 1 - square(-2 * t + 2) / 2;
}

hsv_to_rgb :: (hsv: Vector3) -> Vector3 {
    r, g, b := hsv_to_rgb(hsv.x, hsv.y, hsv.z);
    return .{ r, g, b };
}

hsv_to_rgb :: (cx: float, cy: float, cz: float) -> (r: float, g: float, b: float) {
    h := fmod_cycling(cx, 360.0) / 60;
    c := cz * cy;
    x := c * (1 - abs(fmod_cycling(h, 2.0) - 1));
    m := cz - c;

    if h < 1 return c + m, x + m, m;
    if h < 2 return x + m, c + m, m;
    if h < 3 return m, c + m, x + m;
    if h < 4 return m, x + m, c + m;
    if h < 5 return x + m, m, c + m;

    return c + m, m, x + m;
}

rgb_to_hsv :: (rgb: Vector3) -> Vector3 {
    h, s, v := rgb_to_hsv(rgb.x, rgb.y, rgb.z);
    return .{ h, s, v };
}

rgb_to_hsv :: (r: float, g: float, b: float) -> (h: float, s: float, v: float) {
    v = max(r, max(g, b));
    low := min(r, min(g, b));
    d := v - low;

    h = 0;
    s = 0;
    if v != 0 s = d / v;
    if d == 0 return h, s, v;
    if v == r h = 60 * fmod_cycling((g - b) / d, 6.0);
    else if v == g h = 60 * ((b - r) / d + 2);
    else h = 60 * ((r - g) / d + 4);

    return h, s, v;
}

linear_srgb_to_oklab :: (c: Vector3) -> Vector3 {
    r := cast(float64) c.x;
    g := cast(float64) c.y;
    b := cast(float64) c.z;
    l := cube_root(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
    m := cube_root(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
    s := cube_root(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);

    // Retain coefficient precision through both transforms, rounding only the
    // public Vector3 components to their canonical binary32 storage.
    return .{ cast(float) (0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s), cast(float) (1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s), cast(float) (0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s) };
}

oklab_to_linear_srgb :: (c: Vector3) -> Vector3 {
    x := cast(float64) c.x;
    y := cast(float64) c.y;
    z := cast(float64) c.z;
    l := x + 0.3963377774 * y + 0.2158037573 * z;
    l = l * l * l;
    m := x - 0.1055613458 * y - 0.0638541728 * z;
    m = m * m * m;
    s := x - 0.0894841775 * y - 1.291485548 * z;
    s = s * s * s;

    return saturate(xyz(cast(float) (4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s), cast(float) (-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s), cast(float) (-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s)));
}

make_colors :: (sorted: [] *$T/Color_Keyframe, space: type_of(Interpolation.color_space)) -> [] Vector3 {
    r: [] Vector3;
    r.count = sorted.count;
    r.data = cast(*Vector3) alloc(r.count * size_of(Vector3));

    for key, i: sorted {
        c := key.color_rgb.xyz;
        if space == .HSV c = rgb_to_hsv(c);
        else {
            c = pow(c, GAMMA);
            if space == .OKLAB c = linear_srgb_to_oklab(c);
        }

        r[i] = c;
    }

    return r;
}

interpolate_one_sample :: inline (i0: int, t: float, interp: Interpolation, sorted: [] *$T/Color_Keyframe, temp_colors: [] Vector3) -> (rgba: Vector4) {
    assert(sorted.count > 0 && sorted.count == temp_colors.count);

    if sorted.count == 1 {
        c := temp_colors[0];
        if interp.color_space == .HSV c = hsv_to_rgb(c);
        else {
            if interp.color_space == .OKLAB c = oklab_to_linear_srgb(c);
            c = pow(c, 1 / GAMMA);
        }

        return xyzw(c, sorted[0].color_rgb.w);
    }

    n := sorted.count;
    i1 := i0 + 1;
    first := i0;
    last := i1;

    if first < 0 first = n - 1;
    if last >= n last = 0;

    t0 := sorted[first].t;
    t1 := sorted[last].t;

    if i0 < 0 t0 -= 1;
    if i1 >= n t1 += 1;

    denominator := t1 - t0;
    if denominator == 0 denominator = 1;

    u := (t - t0) / denominator;
    if interp.geometry == .QUADRIC u = easeInOutQuad(u);
    else if interp.geometry == .HERMITE u = u * u * (3 - 2 * u);

    a := temp_colors[first];
    b := temp_colors[last];

    if interp.color_space == .HSV {
        delta := fmod_cycling(b.x - a.x + 180, 360.0) - 180;
        b.x = a.x + delta;
    }

    c := lerp(a, b, u);
    if interp.color_space == .HSV c = hsv_to_rgb(c);
    else {
        if interp.color_space == .OKLAB c = oklab_to_linear_srgb(c);
        c = pow(c, 1 / GAMMA);
    }

    return xyzw(c, lerp(sorted[first].color_rgb.w, sorted[last].color_rgb.w, u));
}

generate_color_samples :: (sorted: [] *$T/Color_Keyframe, interp: Interpolation, output_samples: [] Vector4) {
    if output_samples.count == 0 || sorted.count == 0 return;

    if output_samples.count == 1 {
        output_samples[0] = sorted[0].color_rgb;
        return;
    }

    if interp.color_space != .RGB && interp.color_space != .HSV && interp.color_space != .OKLAB {
        print("unsupported color space\n", to_standard_error = true);
        return;
    }

    colors := make_colors(sorted, interp.color_space);

    defer free(colors.data);

    for *sample, i: output_samples {
        t := cast(float) i / cast(float) (output_samples.count - 1);
        index := 0;

        if interp.wrapping {
            if t < sorted[0].t index = -1;
            else {
                while index + 1 < sorted.count && sorted[index + 1].t <= t index += 1;
            }
        } else {
            while index + 1 < sorted.count - 1 && sorted[index + 1].t <= t index += 1;
        }

        sample.* = interpolate_one_sample(index, t, interp, sorted, colors);
    }
}

#scope_file
#import "Basic";
#import "Math";
Math :: #import "Math";

cube_root :: (x: float64) -> float64 {
    if x == 0 return x;
    if x < 0 return -Math.pow(-x, cast(float64) (1.0 / 3.0));
    return Math.pow(x, cast(float64) (1.0 / 3.0));
}
