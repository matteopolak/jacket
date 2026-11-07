// The JPEG fixtures of make_fixtures.sh, read at compile time so that every way of running a
// test (interpreter, native, wasm) has them without file access. `reference` is the raw
// PPM/PGM reference (djpeg, or exact constant-DC header cases), or empty for refused inputs.

Jpeg_Fixture :: struct {
    name: string;
    jpeg: string;
    reference: string;
}

JPEG_FIXTURE_NAMES :: string.[
    "base444", "base422", "base420", "base440", "base411", "base410", "wide_chroma", "odd_sampling",
    "gray", "gray_progressive", "progressive420", "progressive444", "progressive_restart", "restart420", "restart_each", "progressive_restart_each", "optimized", "progressive_optimized", "rgb", "wide_quant", "tiny", "odd17x9",
    "block8", "cmyk", "ycck", "arithmetic", "twelve_bit", "lossless",
    "header_ycc", "header_jfif_adobe", "header_jfif_rgbids", "header_adobe_ycc_rgbids",
    "header_short_jfif_adobe", "header_rgb", "header_rgb_ids", "header_cmyk_plain", "header_cmyk_adobe",
];

jpeg_fixtures :: #run jpeg_load_fixtures();

jpeg_load_fixtures :: () -> [] Jpeg_Fixture {
    fixtures: [..] Jpeg_Fixture;

    for JPEG_FIXTURE_NAMES {
        fixture: Jpeg_Fixture;
        fixture.name = it;
        fixture.jpeg = jpeg_read_fixture(tprint("%%.jpg", #filepath, it));
        fixture.reference = jpeg_read_fixture(tprint("%%.ppm", #filepath, it));
        if !fixture.reference.count fixture.reference = jpeg_read_fixture(tprint("%%.pgm", #filepath, it));

        array_add(*fixtures, fixture);
    }

    return fixtures;
}

jpeg_read_fixture :: (path: string) -> string {
    text, ok := read_entire_file(path, log_errors = false);
    if !ok return "";
    return text;
}

// The pixels of a P5/P6 file: width, height, channels and the bytes after the header.
jpeg_parse_pnm :: (text: string) -> width: s32, height: s32, channels: s32, pixels: string {
    channels: s32 = cast(s32) ifx text[1] == #char "6" then 3 else 1;
    at := 3;
    numbers: [3] s32;

    for 0..2 {
        while text[at] == #char " " || text[at] == #char "\n" at += 1;

        while text[at] >= #char "0" && text[at] <= #char "9" {
            numbers[it] = numbers[it] * 10 + (text[at] - #char "0");
            at += 1;
        }
    }

    at += 1;

    return numbers[0], numbers[1], channels, slice(text, at, text.count - at);
}

jpeg_bytes :: (text: string) -> [] u8 {
    bytes: [] u8;
    bytes.data = text.data;
    bytes.count = text.count;
    return bytes;
}

// Finds the next marker after entropy data, respecting byte stuffing and restart markers.
jpeg_entropy_end :: (bytes: [] u8, at: int) -> int {
    while at + 1 < bytes.count {
        if bytes[at] == 0xFF && bytes[at + 1] != 0 && !(bytes[at + 1] >= 0xD0 && bytes[at + 1] <= 0xD7) return at;
        at += 1;
    }

    return bytes.count;
}

// Offset of the nth marker. Segment lengths skip marker-shaped bytes inside header payloads.
jpeg_find_marker :: (bytes: [] u8, code: u8, nth := 0) -> int {
    seen := 0;
    at := 2;

    while at + 1 < bytes.count {
        if bytes[at] != 0xFF {
            at += 1;
            continue;
        }

        marker := bytes[at + 1];
        if marker == 0 || marker == 0xFF {
            at += 1;
            continue;
        }

        if marker == code {
            if seen == nth return at;
            seen += 1;
        }

        if marker == 0xD9 return -1;

        if marker == 1 || (marker >= 0xD0 && marker <= 0xD8) {
            at += 2;
            continue;
        }

        if at + 4 > bytes.count return -1;

        length := ((cast(int) bytes[at + 2]) << 8) | bytes[at + 3];
        if length < 2 || at + 2 + length > bytes.count return -1;

        at += 2 + length;
        if marker == 0xDA at = jpeg_entropy_end(bytes, at);
    }

    return -1;
}
