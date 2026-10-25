// The JPEG fixtures of make_fixtures.sh, read at compile time so that every way of running a
// test (interpreter, native, wasm) has them without file access. `reference` is the raw
// PPM/PGM that djpeg writes for the file, or empty when the file must be refused.

Jpeg_Fixture :: struct {
    name: string;
    jpeg: string;
    reference: string;
}

JPEG_FIXTURE_NAMES :: string.[
    "base444", "base422", "base420", "base440", "base411", "base410", "wide_chroma", "odd_sampling",
    "gray", "gray_progressive", "progressive420", "progressive444", "progressive_restart", "restart420", "restart_each", "progressive_restart_each", "optimized", "progressive_optimized", "rgb", "wide_quant", "tiny", "odd17x9",
    "block8", "cmyk", "ycck", "arithmetic", "twelve_bit", "lossless",
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
    channels: s32 = ifx text[1] == #char "6" then 3 else 1;
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
