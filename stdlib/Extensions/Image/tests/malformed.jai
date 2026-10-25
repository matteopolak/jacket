// Damaged input fails cleanly and leaks nothing: every decoder, on files cut off at every byte,
// with every byte changed in turn, and with sizes far beyond the data. A tracking allocator counts
// the blocks left over and the largest block asked for.

#import "Basic";
#import "Extensions/Image";
#import "Extensions/Deflate";
#load "helpers/common.jai";
#load "fixtures/png_cases.jai";

Tally :: struct {
    outstanding: s64;
    largest: s64;
}

tally_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    tally := cast(*Tally) data;
    backing := context.default_allocator;
    result := backing.proc(mode, size, old_size, old_memory, backing.data);

    if mode == .ALLOCATE && result {
        tally.outstanding += 1;
        if size > tally.largest tally.largest = size;
    } else if mode == .RESIZE && result {
        if !old_memory tally.outstanding += 1;
        if size > tally.largest tally.largest = size;
    } else if mode == .FREE && old_memory {
        tally.outstanding -= 1;
    }

    return result;
}

Expectation :: enum {
    SUCCEEDS;
    FAILS;
    EITHER;
}

// Decodes under the tracking allocator and frees the result. Returns whether it decoded.
attempt :: (data: [] u8, max_pixels: s64, tally: *Tally) -> bool {
    ok: bool;

    {
        push_allocator(tally_proc, tally);
        image, decoded := decode_image(data, max_pixels);

        ok = decoded;

        if decoded {
            deinit(*image);
        } else if image.pixels.count != 0 || image.width != 0 {
            ok = true; // reported below: a failure must return an empty image
            tally.outstanding += 1000000;
        }
    }

    return ok;
}

probe :: (data: [] u8, label: string, expectation: Expectation, max_pixels: s64 = DEFAULT_MAX_PIXELS) {
    tally: Tally;
    ok := attempt(data, max_pixels, *tally);

    if expectation == .SUCCEEDS check(ok, tprint("% decodes", label));
    if expectation == .FAILS check(!ok, tprint("% is refused", label));

    check(tally.outstanding == 0, tprint("% left % blocks allocated", label, tally.outstanding));
}

sweep :: (data: [] u8, name: string, cut_must_fail: bool) {
    probe(data, name, .SUCCEEDS);

    for length: 0..data.count - 1 {
        cut: [] u8;
        cut.data = data.data;
        cut.count = length;
        probe(cut, tprint("% cut to %", name, length), ifx cut_must_fail then Expectation.FAILS else .EITHER);
    }

    copy := NewArray(data.count, u8);

    defer array_free(copy);

    for offset: 0..data.count - 1 {
        for mask: u8.[0xff, 0x01, 0x80] {
            memcpy(copy.data, data.data, data.count);
            copy[offset] ^= mask;
            probe(copy, tprint("% byte % changed", name, offset), .EITHER);
        }
    }
}

main :: () {
    // PNG fixtures covering each decoding path.
    wanted := string.["rgb8_split_idat", "rgb8_ancillary_and_apng", "gray4_9x7_adam7", "gray16_5x3_key", "palette4_5x3_alpha", "palette8_9x7_adam7", "rgba16_5x3_adam7", "graya8_5x3", "rgb16_9x7_key", "gray1_5x3_plain"];
    for entry: PNG_CASES for wanted if entry.name == it {
        sweep(bytes_of(entry.png), entry.name, false);
    }

    // Our own output in each format.
    for format: ALL_FORMATS {
        source := random_image(11, 6, format, 8);
        png := encode_png(source, 6);
        qoi := encode_qoi(source);
        bmp := encode_bmp(source);
        ppm := encode_ppm(source);

        sweep(png, tprint("png %", format), true);
        sweep(qoi, tprint("qoi %", format), true);
        sweep(bmp, tprint("bmp %", format), false);
        sweep(ppm, tprint("ppm %", format), true);

        array_free(png);
        array_free(qoi);
        array_free(bmp);
        array_free(ppm);
        deinit(*source);
    }

    // Text PNM, which has no fixed length.
    text := "P3\n# c\n3 2\n255\n1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18\n";

    sweep(bytes_of(text), "P3", false);

    // max_pixels is checked before anything is allocated.
    source := random_image(10, 10, .RGB8, 1);
    png := encode_png(source);
    qoi := encode_qoi(source);
    bmp := encode_bmp(source);
    ppm := encode_ppm(source);
    files: [..][] u8;

    array_add(*files, png);
    array_add(*files, qoi);
    array_add(*files, bmp);
    array_add(*files, ppm);

    for data: files {
        tally: Tally;
        check(!attempt(data, 99, *tally), "refused over the limit");
        check(tally.largest == 0 && tally.outstanding == 0, "nothing allocated over the limit");
        check(attempt(data, 100, *tally), "accepted at the limit");
    }

    // Headers that claim far more than the data holds: refused without a big allocation.
    be32 :: (b: *Builder, v: s64) {
        for shift: 0..3 put8(b, v >> ((3 - shift) * 8));
    }

    claims: [..][] u8;

    {
        b: Builder;
        for "qoif" put8(*b, it);

        be32(*b, 8000);
        be32(*b, 8000);
        put_all(*b, 4, 0, 0, 0xc2, 0, 0, 0, 0, 0, 0, 0, 1);
        array_add(*claims, b.bytes);
    }

    {
        b: Builder;
        for "P5\n8000 8000\n255\n" put8(*b, it);
        for 0..9 put8(*b, 1);
        array_add(*claims, b.bytes);
    }

    {
        b: Builder;
        for "P2\n8000 8000\n255\n1 2 3\n" put8(*b, it);
        array_add(*claims, b.bytes);
    }

    {
        // PNG: a valid IHDR for 8000 x 8000 and a short stream.
        b: Builder;
        for 0..7 put8(*b, "\x89PNG\r\n\x1a\n"[it]);

        header: Builder;

        be32(*header, 8000);
        be32(*header, 8000);
        put_all(*header, 8, 0, 0, 0, 0);
        chunk(*b, "IHDR", header.bytes);
        stream := deflate(bytes_of("not much data"), .ZLIB, 6);

        chunk(*b, "IDAT", stream);
        chunk(*b, "IEND", .[]);
        array_add(*claims, b.bytes);
    }

    {
        // BMP RLE8: 8000 x 8000 with a few bytes of data.
        b: Builder;

        put_all(*b, #char "B", #char "M");
        put32(*b, 54 + 1024 + 4);
        put32(*b, 0);
        put32(*b, 54 + 1024);
        put32(*b, 40);
        put32(*b, 8000);
        put32(*b, 8000);
        put16(*b, 1);
        put16(*b, 8);
        put32(*b, 1);
        put32(*b, 4);
        put32(*b, 0);
        put32(*b, 0);
        put32(*b, 256);
        put32(*b, 0);
        for 0..1023 put8(*b, 0);

        put_all(*b, 0, 1, 0, 1);
        array_add(*claims, b.bytes);
    }

    for claims {
        tally: Tally;
        ok := attempt(it, DEFAULT_MAX_PIXELS, *tally);
        check(!ok, tprint("claim % is refused", it_index));
        check(tally.largest < 1000000, tprint("claim % allocated % bytes", it_index, tally.largest));
    }

    // Absurd dimensions are refused outright.
    huge: Builder;
    for "qoif" put8(*huge, it);

    be32(*huge, 0xffffffff);
    be32(*huge, 0xffffffff);
    put_all(*huge, 4, 0, 0, 0, 0, 0, 0, 0, 0, 1);
    probe(huge.bytes, "huge qoi", .FAILS);

    // Empty and unknown input.
    probe(.[], "empty", .FAILS);
    probe(bytes_of("hello world, this is not an image"), "text", .FAILS);

    finish();
}

chunk :: (b: *Builder, kind: string, body: [] u8) {
    start := b.bytes.count;

    for shift: 0..3 put8(b, body.count >> ((3 - shift) * 8));
    for 0..3 put8(b, kind[it]);
    for body put8(b, it);

    crc: u32;

    {
        view: [] u8;
        view.data = b.bytes.data + start + 4;
        view.count = body.count + 4;
        crc = crc32(view);
    }

    for shift: 0..3 put8(b, crc >> ((3 - shift) * 8));
}
