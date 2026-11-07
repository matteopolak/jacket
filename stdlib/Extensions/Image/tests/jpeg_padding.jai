// T.81 byte alignment uses one bits after complete blocks, including restart boundaries.
#import "Basic";
#import "File";
#import "String";
#import "Extensions/Image";
#load "fixtures/jpeg/fixtures.jai";

Kind :: enum {
    FINAL;
    ALIGNED;
    MULTISCAN;
    RESTART;
    PROGRESSIVE;
}

Stream :: struct {
    bytes: [..] u8;
    offsets: [2] int;
    padding: [2] int;
    count: int;
}

failures: int;

check :: (ok: bool, label: string) {
    if !ok {
        print("FAIL: %\n", label);
        failures += 1;
    }
}

segment :: (bytes: *[..] u8, marker: u8, payload: [] u8) {
    array_add(bytes, 0xFF);
    array_add(bytes, marker);
    length := payload.count + 2;

    array_add(bytes, cast(u8) (length >> 8));
    array_add(bytes, cast(u8) length);
    for payload array_add(bytes, it);
}

entropy :: (stream: *Stream, byte: u8, padding: int) {
    stream.offsets[stream.count] = stream.bytes.count;
    stream.padding[stream.count] = padding;
    stream.count += 1;
    array_add(*stream.bytes, byte);
}

// Each block has DC difference zero and AC EOB, both using the single zero Huffman code.
// Sequential blocks consume two bits; each progressive DC/AC block consumes one bit.
make_stream :: (kind: Kind, fill: bool) -> Stream {
    stream: Stream;

    array_add(*stream.bytes, 0xFF);
    array_add(*stream.bytes, 0xD8);
    quant: [65] u8;
    for 1..64 quant[it] = 1;

    segment(*stream.bytes, 0xDB, quant);
    frame := u8.[8, 0, 8, 0, 8, 1, 1, 17, 0];
    if kind == .RESTART frame[2] = 16;
    if kind == .ALIGNED frame[4] = 32;

    if kind == .MULTISCAN {
        segment(*stream.bytes, 0xC0, u8.[8, 0, 8, 0, 8, 3, 1, 17, 0, 2, 17, 0, 3, 17, 0]);
    } else {
        segment(*stream.bytes, ifx kind == .PROGRESSIVE then 0xC2 else 0xC0, frame);
    }

    tables: [36] u8;
    tables[1] = 1;
    tables[18] = 16;
    tables[19] = 1;
    segment(*stream.bytes, 0xC4, tables);
    if kind == .RESTART segment(*stream.bytes, 0xDD, u8.[0, 1]);

    scan := u8.[1, 1, 0, 0, 63, 0];
    if kind == .PROGRESSIVE scan[4] = 0;

    segment(*stream.bytes, 0xDA, scan);

    if kind == .ALIGNED {
        entropy(*stream, 0, 0);
    } else {
        entropy(*stream, ifx kind == .PROGRESSIVE then 0x7F else 0x3F, ifx kind == .PROGRESSIVE then 7 else 6);
    }

    if fill array_add(*stream.bytes, 0xFF);

    if kind == .MULTISCAN {
        segment(*stream.bytes, 0xDA, u8.[2, 2, 0, 3, 0, 0, 63, 0]);
        entropy(*stream, 0x0F, 4);
    } else if kind == .RESTART {
        array_add(*stream.bytes, 0xFF);
        array_add(*stream.bytes, 0xD0);
        entropy(*stream, 0x3F, 6);
    } else if kind == .PROGRESSIVE {
        segment(*stream.bytes, 0xDA, u8.[1, 1, 0, 1, 63, 0]);
        entropy(*stream, 0x7F, 7);
    }

    if fill array_add(*stream.bytes, 0xFF);

    array_add(*stream.bytes, 0xFF);
    array_add(*stream.bytes, 0xD9);

    return stream;
}

accepted :: (bytes: [] u8, label: string, constant: bool) {
    image, ok := decode_jpeg(bytes, DEFAULT_MAX_PIXELS);

    defer deinit(*image);
    detected, detected_ok := decode_image(bytes);

    defer deinit(*detected);
    check(ok && detected_ok && image.pixels.count > 0 && image.pixels.count == detected.pixels.count, label);
    if constant && ok for image.pixels check(it == 128, "constant zero-coefficient pixels");
}

rejected :: (bytes: [] u8, label: string) {
    image, ok, error := decode_jpeg(bytes, DEFAULT_MAX_PIXELS);

    defer deinit(*image);
    detected, detected_ok := decode_image(bytes);

    defer deinit(*detected);
    check(!ok && !detected_ok && error.count > 0 && image.pixels.count == 0 && detected.pixels.count == 0, label);
}

main :: () {
    for kind: Kind.[.FINAL, .ALIGNED, .MULTISCAN, .RESTART, .PROGRESSIVE] {
        stream := make_stream(kind, false);

        defer array_free(stream.bytes);
        accepted(stream.bytes, "one-bit alignment padding", true);
        filled := make_stream(kind, true);

        defer array_free(filled.bytes);
        accepted(filled.bytes, "marker fill bytes", true);

        for boundary: 0..stream.count - 1 {
            for bit: 0..stream.padding[boundary] - 1 {
                at := stream.offsets[boundary];
                stream.bytes[at] ^= cast(u8) (1 << bit);
                rejected(stream.bytes, tprint("zero pad bit kind % boundary % bit %", kind, boundary, bit));
                stream.bytes[at] ^= cast(u8) (1 << bit);
            }
        }
    }

    for fixture: jpeg_fixtures {
        if fixture.name != "restart_each" && fixture.name != "progressive_restart_each" continue;

        bytes := jpeg_bytes(fixture.jpeg);
        stuffed := false;

        for at: 0..bytes.count - 2 if bytes[at] == 0xFF && bytes[at + 1] == 0 stuffed = true;

        check(stuffed, "fixture contains stuffed FF entropy bytes");
        accepted(bytes, "stuffed FF and restarts", false);
    }

    if failures > 0 {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
