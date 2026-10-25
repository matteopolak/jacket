// JPEG decoding against djpeg: every fixture of fixtures/jpeg/make_fixtures.sh is decoded and
// compared with libjpeg-turbo's output for it (`djpeg -dct int`, fancy upsampling). The decoder
// matches it exactly today; the check allows a mean error of 0.5 and a worst error of 2 so that a
// different but equally accurate transform would not fail. Also: the files that must be refused,
// the pixel limit, and the pixel format of each kind of file.
//
//     jaic stdlib/Extensions/Image/tests/jpeg_decode.jai -interp

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

REFUSED :: string.[
    "arithmetic", "arithmetic-coded JPEG is not supported",
    "twelve_bit", "only 8-bit JPEG is supported",
    "lossless", "lossless JPEG is not supported",
];

compare_all :: () {
    worst := 0;
    decoded := 0;

    for fixture: jpeg_fixtures {
        if !fixture.reference.count continue;

        image, ok, error := decode_image(jpeg_bytes(fixture.jpeg));

        check(ok, tprint("% decodes (%)", fixture.name, error));
        if !ok continue;

        decoded += 1;

        width, height, channels, expected := jpeg_parse_pnm(fixture.reference);

        check(image.width == width && image.height == height, tprint("% has the size of its reference", fixture.name));
        check(image.format == (ifx channels == 1 then Pixel_Format.GRAY8 else Pixel_Format.RGB8), tprint("% has the right pixel format", fixture.name));
        check(image.pixels.count == expected.count, tprint("% has the right byte count", fixture.name));

        if image.pixels.count == expected.count {
            sum := 0;
            largest := 0;

            for 0..expected.count - 1 {
                difference := cast(int) image.pixels[it] - cast(int) expected[it];
                if difference < 0 difference = -difference;
                sum += difference;
                largest = max(largest, difference);
            }

            check(largest <= 2, tprint("% is within 2 of djpeg (worst %)", fixture.name, largest));
            check(sum * 2 <= expected.count, tprint("% has a mean error under 0.5 (sum %)", fixture.name, sum));
            worst = max(worst, largest);
        }

        deinit(*image);
    }

    check(decoded >= 23, "most fixtures were compared");
    print("compared % fixtures, worst error %\n", decoded, worst);
}

refusals :: () {
    for 0..REFUSED.count / 2 - 1 {
        name := REFUSED[it * 2];

        for fixture: jpeg_fixtures {
            if fixture.name != name continue;

            image, ok, error := decode_image(jpeg_bytes(fixture.jpeg));

            check(!ok, tprint("% is refused", name));
            check(error == REFUSED[it * 2 + 1], tprint("% says why (%)", name, error));
            check(image.pixels.count == 0, tprint("% returns no pixels", name));
        }
    }
}

pixel_limit :: () {
    for fixture: jpeg_fixtures {
        if fixture.name != "base444" continue;

        bytes := jpeg_bytes(fixture.jpeg);

        _, ok, error := decode_image(bytes, 37 * 23 - 1);

        check(!ok && error == "image is larger than the pixel limit", "one pixel over the limit is refused");

        image, fits, _ := decode_image(bytes, 37 * 23);

        check(fits, "exactly the limit is accepted");
        if fits deinit(*image);

        _, ok2, _ := decode_image(bytes, 0);

        check(!ok2, "a zero limit refuses everything");
    }
}

not_jpeg :: () {
    empty: [] u8;
    _, ok, _ := decode_image(empty);

    check(!ok, "empty input is refused");

    soi := u8.[0xFF, 0xD8, 0xFF, 0xD9];
    _, ok2, error2 := decode_image(soi);

    check(!ok2 && error2 == "JPEG has no frame header", "SOI then EOI has no frame");

    junk := u8.[0xFF, 0xD8, 0xFF, 0x00, 0x12, 0x34];
    _, ok3, _ := decode_image(junk);

    check(!ok3, "junk after SOI is refused");
}

main :: () {
    compare_all();
    refusals();
    pixel_limit();
    not_jpeg();

    if failures {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
