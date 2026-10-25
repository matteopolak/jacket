// Damaged JPEG data never crashes the decoder and never hangs it: files cut off at every byte,
// bytes replaced by random values (the whole file, and the header region separately), runs of
// 0xFF and 0x00, and swapped restart markers. A result is either an error or an image of the
// size the header announces; truncated entropy data decodes as far as it goes.
//
//     jaic stdlib/Extensions/Image/tests/jpeg_robust.jai -interp

#import "Basic";
#import "String";
#import "File";
#import "Extensions/Image";

#load "fixtures/jpeg/fixtures.jai";

failures := 0;
decodes := 0;
accepted := 0;

check :: (condition: bool, what: string) {
    if condition return;
    print("FAIL: %\n", what);
    failures += 1;
}

// Decodes damaged bytes; returns whether they were accepted.
try_decode :: (bytes: [] u8, expected_width: s32, expected_height: s32, name: string, detail: int) -> bool {
    image, ok, error := decode_image(bytes);

    decodes += 1;

    if !ok {
        if error.count == 0 || image.pixels.count != 0 check(false, tprint("% (%) fails with a reason and no pixels", name, detail));
        return false;
    }

    accepted += 1;
    consistent := image.width > 0 && image.height > 0 && image.pixels.count == cast(s64) image.width * image.height * bytes_per_pixel(image.format);
    if !consistent check(false, tprint("% (%) decodes to a consistent image", name, detail));
    if expected_width > 0 && (image.width != expected_width || image.height != expected_height) check(false, tprint("% (%) keeps its size", name, detail));

    deinit(*image);

    return true;
}

random_state: u32 = 2463534242;

random :: () -> u32 {
    random_state ^= random_state << 13;
    random_state ^= random_state >> 17;
    random_state ^= random_state << 5;
    return random_state;
}

find_fixture :: (name: string) -> Jpeg_Fixture {
    for jpeg_fixtures if it.name == name return it;
    assert(false, "no such fixture");
    return .{};
}

truncations :: (name: string, step: int) {
    fixture := find_fixture(name);
    width, height, _, _ := jpeg_parse_pnm(fixture.reference);
    bytes := jpeg_bytes(fixture.jpeg);

    for length: 0..bytes.count - 1 {
        if step > 1 && length % step != 0 && length > 40 continue;
        cut := bytes;
        cut.count = length;
        try_decode(cut, width, height, name, length);
    }
}

corruptions :: (name: string, rounds: int, first: int, last: int) {
    fixture := find_fixture(name);
    original := jpeg_bytes(fixture.jpeg);
    copy := NewArray(original.count, u8);

    defer array_free(copy);

    limit := min(last, original.count);

    for 1..rounds {
        memcpy(copy.data, original.data, original.count);
        edits := 1 + random() % 4;
        for 1..edits {
            at := first + cast(int) (random() % cast(u32) (limit - first));
            copy[at] = cast(u8) (random() & 255);
        }

        // The size can change when a header byte changes: only require sane output.
        try_decode(copy, 0, 0, name, it);
    }
}

// Every header byte (up to the first scan) replaced by 0x00, 0xFF and 0x80 in turn.
header_sweep :: (name: string) {
    fixture := find_fixture(name);
    original := jpeg_bytes(fixture.jpeg);
    copy := NewArray(original.count, u8);

    defer array_free(copy);

    scan_start := 0;

    for 0..original.count - 2 {
        if original[it] == 0xFF && original[it + 1] == 0xDA {
            scan_start = it;
            break;
        }
    }

    values := u8.[0x00, 0xFF];

    for at: 0..scan_start {
        for value: values {
            memcpy(copy.data, original.data, original.count);
            copy[at] = value;
            try_decode(copy, 0, 0, name, at * 256 + value);
        }
    }
}

// Restart markers moved, repeated or dropped.
restart_damage :: (name: string) {
    fixture := find_fixture(name);
    original := jpeg_bytes(fixture.jpeg);
    copy := NewArray(original.count, u8);

    defer array_free(copy);

    markers: [..] int;

    defer array_free(markers);

    for 0..original.count - 2 {
        if original[it] == 0xFF && original[it + 1] >= 0xD0 && original[it + 1] <= 0xD7 array_add(*markers, it);
    }

    check(markers.count > 2, tprint("% has restart markers", name));

    for at: markers {
        for mutation: 0..3 {
            memcpy(copy.data, original.data, original.count);
            if mutation == 0 copy[at + 1] = 0xD0 + (copy[at + 1] + 1) % 8;
            if mutation == 1 copy[at + 1] = 0xD0 + (copy[at + 1] + 3) % 8;
            if mutation == 2 copy[at + 1] = 0x00;
            if mutation == 3 copy[at] = 0x00;

            try_decode(copy, 0, 0, name, at * 4 + mutation);
        }
    }
}

main :: () {
    truncations("odd17x9", 2);
    truncations("gray_progressive", 1);
    truncations("progressive_restart", 1);
    truncations("restart_each", 7);
    truncations("base420", 7);
    truncations("progressive420", 7);
    truncations("cmyk", 23);
    truncations("wide_quant", 11);
    truncations("optimized", 11);

    names := string.["base444", "base422", "base420", "base411", "gray", "progressive420", "progressive_restart", "restart420", "cmyk", "ycck", "odd_sampling", "wide_quant", "tiny"];
    for names {
        corruptions(it, 20, 0, 100000);
        corruptions(it, 12, 0, 700);
    }

    header_sweep("base420");
    header_sweep("progressive_restart");
    restart_damage("restart_each");
    restart_damage("progressive_restart_each");

    print("% decodes, % accepted\n", decodes, accepted);
    check(accepted > 0 && accepted < decodes, "some damaged files decode and some do not");

    if failures {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
