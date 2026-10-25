// Hand-damaged JPEG headers: each edit of a valid file is refused with the right reason or, where
// the format allows it, decoded in a defined way (a frame taller than its data keeps mid-grey
// below the data).
//
//     jaic stdlib/Extensions/Image/tests/jpeg_malformed.jai -interp

#import "Basic";
#import "String";
#import "File";
#import "Extensions/Image";

#load "fixtures/jpeg/fixtures.jai";

failures := 0;

check :: (condition: bool, what: string) {
    if condition return;
    print("FAIL: %\n", what);
    failures += 1;
}

find_fixture :: (name: string) -> [] u8 {
    for jpeg_fixtures if it.name == name return jpeg_bytes(it.jpeg);
    assert(false, "no such fixture");
    return .[];
}

// Offset of the nth (from 0) marker segment with this code, or -1.
find_marker :: (bytes: [] u8, code: u8, nth := 0) -> int {
    seen := 0;
    at := 2;

    while at + 4 <= bytes.count {
        if bytes[at] != 0xFF {
            at += 1;
            continue;
        }

        marker := bytes[at + 1];
        if marker == 0 || marker == 0xFF || (marker >= 0xD0 && marker <= 0xD8) {
            at += 1;
            continue;
        }

        if marker == code {
            if seen == nth return at;
            seen += 1;
        }

        length := (cast(int) bytes[at + 2] << 8) | bytes[at + 3];

        at += 2 + length;

        // Entropy-coded data follows a scan header: skip to the next marker.
        if marker == 0xDA {
            while at + 1 < bytes.count && !(bytes[at] == 0xFF && bytes[at + 1] != 0 && !(bytes[at + 1] >= 0xD0 && bytes[at + 1] <= 0xD7)) at += 1;
        }
    }

    return -1;
}

copy_of :: (bytes: [] u8) -> [] u8 {
    copy := NewArray(bytes.count, u8);
    memcpy(copy.data, bytes.data, bytes.count);
    return copy;
}

// The file with bytes[at] = value must be refused with `message`.
expect_error :: (bytes: [] u8, at: int, value: u8, message: string, label: string) {
    copy := copy_of(bytes);

    defer array_free(copy);
    copy[at] = value;

    image, ok, error := decode_image(copy);

    check(!ok, tprint("% is refused", label));
    check(error == message, tprint("% says \"%\" (got \"%\")", label, message, error));
    check(image.pixels.count == 0, tprint("% returns no pixels", label));
    if ok deinit(*image);
}

same_pixels :: (a: Image, b: Image, count: int) -> bool {
    for 0..count - 1 if a.pixels[it] != b.pixels[it] return false;
    return true;
}

// restart_each is 4:4:4, 5 x 3 MCUs of 8 x 8 pixels with a restart marker after every MCU.
// Damage inside one interval must stay inside that MCU.
restart_resynchronisation :: () {
    original := find_fixture("restart_each");
    reference, ok, _ := decode_image(original);

    check(ok, "restart_each decodes");
    if !ok return;

    defer deinit(*reference);

    markers: [..] int;

    defer array_free(markers);

    for 0..original.count - 2 {
        if original[it] == 0xFF && original[it + 1] >= 0xD0 && original[it + 1] <= 0xD7 array_add(*markers, it);
    }

    check(markers.count >= 14, "restart_each has a marker after every MCU");

    // Interval 3 (between markers 2 and 3) filled with another bit pattern.
    {
        copy := copy_of(original);

        defer array_free(copy);
        for markers[2] + 2..markers[3] - 1 copy[it] = 0x55;

        image, decoded, _ := decode_image(copy);

        check(decoded, "a damaged interval still decodes");

        if decoded {
            check(only_differs_in_mcu(image, reference, 3), "damage stays inside its MCU");
            deinit(*image);
        }
    }

    // Marker 4 removed: interval 5 is swallowed, the rest resynchronises on marker 5.
    {
        copy: [..] u8;

        defer array_free(copy);

        for original {
            if it_index == markers[4] || it_index == markers[4] + 1 continue;
            array_add(*copy, it);
        }

        image, decoded, _ := decode_image(copy);

        check(decoded, "a missing restart marker still decodes");

        if decoded {
            check(only_differs_in_mcu(image, reference, 5), "a missing marker costs one MCU");
            grey := true;

            for y: 8..15 for x : 0..7 {
                texel := image.pixels.data + (y * image.width + x) * 3;
                if texel[0] != 128 || texel[1] != 128 || texel[2] != 128 grey = false;
            }

            check(grey, "the lost MCU is mid-grey");
            deinit(*image);
        }
    }
}

// True when every pixel outside MCU number `mcu` (5 per row, 8 x 8 pixels) matches.
only_differs_in_mcu :: (image: Image, reference: Image, mcu: int) -> bool {
    if image.width != reference.width || image.height != reference.height return false;

    for y: 0..image.height - 1 for x : 0..image.width - 1 {
        if (y / 8) * 5 + x / 8 == mcu continue;

        for c: 0..2 {
            at := (y * image.width + x) * 3 + c;
            if image.pixels[at] != reference.pixels[at] return false;
        }
    }

    return true;
}

main :: () {
    base := find_fixture("base444");
    sof := find_marker(base, 0xC0);

    check(sof > 0, "base444 has a SOF0");

    // Layout from `sof`: marker (2), length (2), precision (1), height (2), width (2), count (1),
    // then 3 bytes per component (id, sampling, quantization table).
    expect_error(base, sof + 4, 12, "only 8-bit JPEG is supported", "12-bit precision");

    {
        copy := copy_of(base);

        defer array_free(copy);
        copy[sof + 7] = 0;
        copy[sof + 8] = 0;
        _, ok, error := decode_image(copy);

        check(!ok && error == "JPEG has a zero dimension", "zero width");

        copy[sof + 8] = 37;
        copy[sof + 5] = 0;
        copy[sof + 6] = 0;
        _, ok2, error2 := decode_image(copy);

        check(!ok2 && error2 == "JPEG has a zero dimension", "zero height");

        for 5..8 copy[sof + it] = 0xFF;

        _, ok3, error3 := decode_image(copy);

        check(!ok3 && error3 == "image is larger than the pixel limit", "65535 x 65535 against the default limit");
    }

    expect_error(base, sof + 9, 2, "unsupported number of JPEG components", "two components");
    expect_error(base, sof + 9, 5, "unsupported number of JPEG components", "five components");
    expect_error(base, sof + 11, 0x00, "bad frame header", "sampling 0x0");
    expect_error(base, sof + 11, 0x51, "bad frame header", "horizontal sampling 5");
    expect_error(base, sof + 11, 0x15, "bad frame header", "vertical sampling 5");
    expect_error(base, sof + 12, 4, "bad frame header", "quantization table 4");
    expect_error(base, sof + 1, 0xC3, "lossless JPEG is not supported", "SOF3");
    expect_error(base, sof + 1, 0xC5, "hierarchical JPEG is not supported", "SOF5");
    expect_error(base, sof + 1, 0xC9, "arithmetic-coded JPEG is not supported", "SOF9");
    expect_error(base, sof + 1, 0xCC, "arithmetic-coded JPEG is not supported", "DAC");
    expect_error(base, sof + 1, 0xC8, "unsupported JPEG process", "JPG");
    expect_error(base, sof + 1, 0xC2, "bad progressive scan header", "a progressive frame with a baseline scan");

    // The extended sequential type decodes like baseline.
    {
        copy := copy_of(base);

        defer array_free(copy);
        copy[sof + 1] = 0xC1;
        image, ok, _ := decode_image(copy);
        reference, ok2, _ := decode_image(base);

        check(ok && ok2 && image.pixels.count == reference.pixels.count, "SOF1 decodes");
        if ok && ok2 check(same_pixels(image, reference, image.pixels.count), "SOF1 gives the baseline pixels");
        if ok deinit(*image);
        if ok2 deinit(*reference);
    }

    // Scan header.
    sos := find_marker(base, 0xDA);

    check(sos > 0, "base444 has a SOS");
    expect_error(base, sos + 4, 0, "bad scan header", "scan with no components");
    expect_error(base, sos + 4, 5, "bad scan header", "scan with five components");
    expect_error(base, sos + 5, 9, "JPEG scan names a component the frame does not have", "unknown component");
    expect_error(base, sos + 7, 1, "bad scan header", "component named twice");
    expect_error(base, sos + 6, 0x40, "bad scan header", "DC table 4");

    // Missing tables.
    {
        copy := copy_of(base);

        defer array_free(copy);

        while true {
            at := find_marker(copy, 0xDB);
            if at < 0 break;
            copy[at + 1] = 0xFE;
        }

        _, ok, error := decode_image(copy);

        check(!ok && error == "JPEG scan uses an undefined quantization table", "no quantization tables");
    }

    {
        copy := copy_of(base);

        defer array_free(copy);

        while true {
            at := find_marker(copy, 0xC4);
            if at < 0 break;
            copy[at + 1] = 0xFE;
        }

        _, ok, error := decode_image(copy);

        check(!ok && error == "JPEG scan uses an undefined Huffman table", "no Huffman tables");
    }

    // Tables.
    dht := find_marker(base, 0xC4);

    expect_error(base, dht + 4, 0x20, "bad Huffman table", "Huffman class 2");
    expect_error(base, dht + 4, 0x04, "bad Huffman table", "Huffman table id 4");
    expect_error(base, dht + 5, 3, "bad Huffman table", "three codes of length one");

    {
        copy := copy_of(base);

        defer array_free(copy);
        for 5..20 copy[dht + it] = 255;

        _, ok, error := decode_image(copy);

        check(!ok && error == "bad Huffman table", "more than 256 symbols");
    }

    dqt := find_marker(base, 0xDB);

    expect_error(base, dqt + 4, 0x20, "bad quantization table", "quantization precision 2");
    expect_error(base, dqt + 4, 0x05, "bad quantization table", "quantization table id 5");

    // Cut-off headers.
    for length: 0..sof + 10 {
        cut := base;
        cut.count = length;
        image, ok, _ := decode_image(cut);

        check(!ok, tprint("the first % bytes are not an image", length));
        if ok deinit(*image);
    }

    // A frame taller than its data: the missing rows stay mid-grey.
    {
        copy := copy_of(base);

        defer array_free(copy);
        copy[sof + 5] = 0x07;
        copy[sof + 6] = 0xD0; // 2000 rows
        image, ok, _ := decode_image(copy);
        reference, ok2, _ := decode_image(base);

        check(ok && ok2 && image.width == 37 && image.height == 2000, "a taller frame decodes");

        if ok && ok2 {
            check(same_pixels(image, reference, 37 * 23 * 3), "the rows that have data are unchanged");
            grey := true;
            for 1999 * 37 * 3..2000 * 37 * 3 - 1 if image.pixels[it] != 128 grey = false;
            check(grey, "the last row is mid-grey");
        }

        if ok deinit(*image);
        if ok2 deinit(*reference);
    }

    // Many blocks per MCU.
    {
        copy := copy_of(base);

        defer array_free(copy);
        for 0..2 copy[sof + 11 + it * 3] = 0x44;

        _, ok, error := decode_image(copy);

        check(!ok && error == "too many blocks in a JPEG MCU", "three 4x4 components");
    }

    // Progressive scan parameters.
    progressive := find_fixture("progressive420");
    first := find_marker(progressive, 0xDA);

    check(first > 0, "progressive420 has scans");
    count := cast(int) progressive[first + 4];

    expect_error(progressive, first + 5 + count * 2 + 1, 1, "bad progressive scan header", "a DC scan with a spectral end");
    expect_error(progressive, first + 5 + count * 2, 70, "bad progressive scan header", "a spectral start above 63");
    second := find_marker(progressive, 0xDA, 1);

    check(second > 0, "progressive420 has a second scan");

    restart_resynchronisation();

    if failures {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
