#import "Basic";

transform :: (dst: *int, src: [] int, other: [] int, count: int) {
    // The value changes on the way.
    for i: 0..count - 1 {
        dst[i] = src[i] * 2;
    }

    // A shifted index.
    for i: 0..count - 2 {
        dst[i] = src[i + 1];
    }

    // The range does not start at the front.
    for i: 1..count - 1 {
        dst[i] = src[i];
    }

    // More than a copy.
    for i: 0..count - 1 {
        dst[i] = src[i];
        other[i] = src[i];
    }

    // An array against itself.
    for i: 0..count - 1 {
        dst[i] = dst[i];
    }
}

mixed :: (dst: [] float, src: [] int) {
    // Different element types: a conversion, not a copy.
    for i: 0..src.count - 1 {
        dst[i] = cast(float) src[i];
    }
}

main :: () {
    values: [4] int;
    out: [4] int;
    transform(out.data, values, values, 4);
    print("%\n", out);
}
