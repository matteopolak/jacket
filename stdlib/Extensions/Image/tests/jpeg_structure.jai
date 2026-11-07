// Structural JPEG errors are distinct from the documented recovery of damaged entropy data.
#import "Basic";
#import "String";
#import "File";
#import "Extensions/Image";
#load "fixtures/jpeg/fixtures.jai";

failures: int;

check :: (ok: bool, label: string) {
    if !ok {
        print("FAIL: %\n", label);
        failures += 1;
    }
}

fixture :: (name: string) -> [] u8 {
    for jpeg_fixtures if it.name == name return jpeg_bytes(it.jpeg);
    assert(false);
    return .[];
}

copy_of :: (source: [] u8) -> [] u8 {
    copy := NewArray(source.count, u8);
    memcpy(copy.data, source.data, source.count);
    return copy;
}

without_span :: (source: [] u8, first: int, end: int) -> [] u8 {
    copy := NewArray(source.count - (end - first), u8);
    memcpy(copy.data, source.data, first);
    memcpy(copy.data + first, source.data + end, source.count - end);
    return copy;
}

with_marker :: (source: [] u8, marker: [] u8) -> [] u8 {
    copy := NewArray(source.count + marker.count, u8);

    memcpy(copy.data, source.data, source.count - 2);
    memcpy(copy.data + source.count - 2, marker.data, marker.count);
    memcpy(copy.data + source.count - 2 + marker.count, source.data + source.count - 2, 2);

    return copy;
}

rejected :: (source: [] u8, label: string) {
    image, ok, error := decode_image(source);
    defer deinit(*image);
    check(!ok && error.count > 0 && image.pixels.count == 0, label);
}

main :: () {
    base := fixture("header_ycc");
    sof := jpeg_find_marker(base, 0xC0);
    sos := jpeg_find_marker(base, 0xDA);

    assert(sof >= 0 && sos >= 0 && base[sos + 4] == 3);

    for value, index: u8.[1, 62, 0x10, 1] {
        copy := copy_of(base);

        defer array_free(copy);
        field := sos + 11 + min(index, 2);
        copy[field] = value;
        rejected(copy, tprint("sequential scan field %", index));
    }

    {
        copy := copy_of(base);

        defer array_free(copy);
        copy[sof + 13] = 3;
        copy[sof + 16] = 2;
        rejected(copy, "scan components follow frame order");
    }

    {
        copy := copy_of(base);

        defer array_free(copy);
        identifiers := u8.[200, 5, 100];
        for identifiers {
            copy[sof + 10 + it_index * 3] = it;
            copy[sos + 5 + it_index * 2] = it;
        }

        image, ok := decode_image(copy);

        defer deinit(*image);
        check(ok && image.pixels.count == 3, "component identifiers need not be numerically ordered");
        if ok for image.pixels check(it == 64, "arbitrary component identifiers preserve pixels");
    }

    {
        copy := without_span(base, sos + 7, sos + 9);

        defer array_free(copy);
        copy[sof + 13] = copy[sof + 10];
        copy[sos + 3] = 10;
        copy[sos + 4] = 2;
        rejected(copy, "frame component identifiers are unique");
    }

    for code: u8.[0xD8, 0xF7, 0xDE, 0xDF] {
        marker := u8.[0xFF, code, 0, 2];
        addition := cast([] u8) marker;

        if code == 0xD8 addition.count = 2;

        copy := with_marker(base, addition);

        defer array_free(copy);
        rejected(copy, tprint("structural marker %", code));
    }

    {
        copy := NewArray(sos + 4, u8);

        defer array_free(copy);
        memcpy(copy.data, base.data, sos);
        copy[sos] = 0xFF;
        copy[sos + 1] = 0xDA;
        copy[sos + 2] = 0;
        copy[sos + 3] = 2;
        rejected(copy, "empty scan segment ends at the buffer boundary");
    }

    progressive := fixture("progressive420");

    assert(jpeg_find_marker(progressive, 0xC2) >= 0);
    first := jpeg_find_marker(progressive, 0xDA);
    count := progressive[first + 4];
    approximation := first + 7 + count * 2;

    assert(progressive[first + 5 + count * 2] == 0 && progressive[first + 6 + count * 2] == 0);

    for value: u8.[0x12, 0x10] {
        copy := copy_of(progressive);
        defer array_free(copy);
        copy[approximation] = value;
        rejected(copy, tprint("invalid initial refinement %", value));
    }

    {
        start := first + 2 + ((cast(int) progressive[first + 2]) << 8) + progressive[first + 3];
        end := jpeg_entropy_end(progressive, start);
        copy := without_span(progressive, first, end);

        defer array_free(copy);
        rejected(copy, "AC data requires the component's DC scan");
    }

    {
        start := first + 2 + ((cast(int) progressive[first + 2]) << 8) + progressive[first + 3];
        end := jpeg_entropy_end(progressive, start);
        copy := NewArray(progressive.count + end - first, u8);

        defer array_free(copy);
        memcpy(copy.data, progressive.data, end);
        memcpy(copy.data + end, progressive.data + first, end - first);
        memcpy(copy.data + end + end - first, progressive.data + end, progressive.count - end);
        rejected(copy, "progressive first transmission is not repeated");
    }

    found_refinement := false;

    for nth: 1..30 {
        at := jpeg_find_marker(progressive, 0xDA, nth);
        if at < 0 break;

        components := progressive[at + 4];
        field := at + 7 + components * 2;
        old := progressive[field] >> 4;

        if old == 0 continue;

        copy := copy_of(progressive);

        defer array_free(copy);
        high := ifx old == 1 then 2 else 1;

        copy[field] = cast(u8) ((high << 4) | (high - 1));
        rejected(copy, "refinement follows the previous bit level");
        found_refinement = true;
        break;
    }

    assert(found_refinement);

    if failures != 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
