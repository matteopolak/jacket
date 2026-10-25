// Shared by the Image tests: checks, a deterministic generator and byte builders.

failures := 0;

check :: (condition: bool, what: string, loc := #caller_location) {
    if condition return;
    print("FAIL (line %): %\n", loc.line_number, what);
    failures += 1;
}

finish :: () {
    if failures != 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}

same_bytes :: (a: [] u8, b: [] u8) -> bool {
    if a.count != b.count return false;
    for 0..a.count - 1 if a[it] != b[it] return false;
    return true;
}

bytes_of :: (text: string) -> [] u8 {
    result: [] u8;
    result.data = text.data;
    result.count = text.count;
    return result;
}

Lcg :: struct {
    state: u64;
}

next_random :: (r: *Lcg) -> u32 {
    r.state = r.state * 6364136223846793005 + 1442695040888963407;
    return cast,trunc(u32) (r.state >> 33);
}

// Pixels with flat stretches, gradients and noise, so every QOI op and PNG filter has work.
random_image :: (width: s32, height: s32, format: Pixel_Format, seed: u64) -> Image {
    r := Lcg.{ seed * 2654435761 + 1 };
    image := make_image(width, height, format);
    bpp := bytes_per_pixel(format);

    for *byte, i: image.pixels {
        pixel := i / bpp;
        channel := i % bpp;
        kind := (pixel / 7) % 4;

        if kind == 0 {
            byte.* = cast,trunc(u8) (channel * 40 + 10);
        } else if kind == 1 {
            byte.* = cast,trunc(u8) (pixel * 3 + channel);
        } else if kind == 2 {
            byte.* = cast,trunc(u8) next_random(*r);
        } else {
            byte.* = cast,trunc(u8) (128 + (next_random(*r) & 3));
        }
    }

    return image;
}

ALL_FORMATS :: Pixel_Format.[.GRAY8, .GRAY_ALPHA8, .RGB8, .RGBA8, .BGRA8];

Builder :: struct {
    bytes: [..] u8;
}

put8 :: (b: *Builder, v: s64) {
    array_add(*b.bytes, cast,trunc(u8) v);
}

put16 :: (b: *Builder, v: s64) {
    put8(b, v);
    put8(b, v >> 8);
}

put32 :: (b: *Builder, v: s64) {
    put16(b, v);
    put16(b, v >> 16);
}

put_all :: (b: *Builder, values: ..s64) {
    for values put8(b, it);
}
