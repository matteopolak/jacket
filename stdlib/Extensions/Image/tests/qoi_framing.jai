// QOI's end marker and declared pixel count delimit one complete file.
#import "Basic";
#import "Extensions/Image";

failures: int;

check :: (ok: bool, name: string) {
    if !ok {
        print("FAIL: %\n", name);
        failures += 1;
    }
}

rejected :: (data: [] u8, name: string) {
    image, ok := decode_qoi(data);

    defer deinit(*image);
    detected, detected_ok := decode_image(data);

    defer deinit(*detected);
    check(!ok && !detected_ok && image.pixels.count == 0 && detected.pixels.count == 0, name);
}

main :: () {
    valid := u8.[#char "q", #char "o", #char "i", #char "f", 0, 0, 0, 1, 0, 0, 0, 1, 4, 0, 0xFF, 10, 20, 30, 40, 0, 0, 0, 0, 0, 0, 0, 1];
    image, ok := decode_qoi(valid);

    defer deinit(*image);
    check(ok && image.width == 1 && image.height == 1 && image.pixels.count == 4, "independent complete QOI");

    for byte: 0..7 {
        copy := valid;
        copy[19 + byte] ^= 0x40;
        rejected(copy, "exact end marker");
    }

    overrun := u8.[#char "q", #char "o", #char "i", #char "f", 0, 0, 0, 1, 0, 0, 0, 1, 4, 0, 0xC1, 0, 0, 0, 0, 0, 0, 0, 1];

    rejected(overrun, "run exceeds declared pixel count");
    extra: [28] u8;

    memcpy(extra.data, valid.data, 19);
    extra[19] = 0;
    memcpy(extra.data + 20, valid.data + 19, 8);
    rejected(extra, "unused opcode before end marker");

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
