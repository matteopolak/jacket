// Ico_File and Wav_File at run time on real files: a BMP turned into a five-size ICO (24-bit
// bottom-up and 32-bit top-down sources, alpha masks), raw pixels straight to ICO, and WAV
// headers with PCM and IMA ADPCM formats, odd-sized chunks, fact chunks and truncation.
#import "Basic";
#import "File";
#import "Ico_File";
#import "Wav_File";

ROOT :: ".scratch/ico-wav-files";

u16_at :: (s: string, at: int) -> int {
    return s[at] | ((cast(int) s[at + 1]) << 8);
}

u32_at :: (s: string, at: int) -> int {
    return u16_at(s, at) | (u16_at(s, at + 2) << 16);
}

put16 :: (b: *String_Builder, v: int) {
    append(b, cast,trunc(u8) v);
    append(b, cast,trunc(u8) (v >> 8));
}

put32 :: (b: *String_Builder, v: int) {
    put16(b, v & 0xffff);
    put16(b, v >> 16);
}

// A 2x2 BMP. Pixel (x, y) counted from the top has blue x*100+10, green y*100+20, red 30, and
// alpha 0 only at the top-left when 32-bit.
make_bmp :: (bits: int, top_down: bool) -> string {
    comp := bits / 8;
    row := (2 * comp + 3) & ~3;
    b: String_Builder;
    append(*b, "BM");
    put32(*b, 54 + row * 2);
    put32(*b, 0);
    put32(*b, 54);
    put32(*b, 40);
    put32(*b, 2);
    put32(*b, ifx top_down then -2 else 2);
    put16(*b, 1);
    put16(*b, bits);
    put32(*b, 0);
    put32(*b, row * 2);
    for 1..4 put32(*b, 0);
    for stored: 0..1 {
        y := ifx top_down then stored else 1 - stored;
        for x: 0..1 {
            append(*b, cast(u8) (x * 100 + 10));
            append(*b, cast(u8) (y * 100 + 20));
            append(*b, 30);
            if comp == 4 append(*b, cast(u8) (ifx x == 0 && y == 0 then 0 else 255));
        }
        for comp * 2..row - 1 append(*b, 0);
    }
    return builder_to_string(*b);
}

SIZES :: int.[256, 128, 48, 32, 16];

check_ico :: (ico: string, comp: int, expect_mask: bool, loc := #caller_location) {
    assert(ico.count > 6 + 16 * 5, "ico of % bytes", ico.count, loc = loc);
    assert(u16_at(ico, 0) == 0 && u16_at(ico, 2) == 1 && u16_at(ico, 4) == 5, "ico header", loc = loc);
    expected_offset := 6 + 16 * 5;
    for size: SIZES {
        entry := 6 + it_index * 16;
        assert(ico[entry] == cast,trunc(u8) size && ico[entry + 1] == cast,trunc(u8) size, "entry % dimensions", it_index, loc = loc);
        assert(u16_at(ico, entry + 4) == 1 && u16_at(ico, entry + 6) == comp * 8, "entry % planes and bits", it_index, loc = loc);
        bytes := u32_at(ico, entry + 8);
        offset := u32_at(ico, entry + 12);
        assert(bytes == get_total_bytes_for_size(xx comp, xx size) && offset == expected_offset, "entry %: % bytes at %", it_index, bytes, offset, loc = loc);
        assert(offset + bytes <= ico.count, "entry % in bounds", it_index, loc = loc);
        // The DIB header: twice the height for the image plus mask.
        assert(u32_at(ico, offset) == 40 && u32_at(ico, offset + 4) == size && u32_at(ico, offset + 8) == size * 2, "dib % header", it_index, loc = loc);
        // Each image row maps to source row (row * 2 / size); the first row of the icon is
        // the first stored row of the source, and pixel (0, 0) samples source (0, 0).
        stride := (size * comp + 3) & ~3;
        image := offset + 40;
        first := image;
        last := image + (size - 1) * stride + (size - 1) * comp;
        assert(ico[first] == 10 && ico[first + 2] == 30, "first pixel of size %", size, loc = loc);
        assert(ico[last] == 110 && ico[last + 2] == 30, "last pixel of size %", size, loc = loc);
        mask := image + stride * size;
        mask_bit := ico[mask] & 0x80;
        if expect_mask assert(mask_bit != 0, "transparent pixel masked at size %", size, loc = loc);
        else assert(mask_bit == 0, "opaque pixel unmasked at size %", size, loc = loc);
        expected_offset += bytes;
    }
    assert(expected_offset <= ico.count && ico.count - expected_offset <= 5 * 40, "ico ends after its images: % vs %", expected_offset, ico.count, loc = loc);
}

main :: () {
    delete_directory(ROOT);
    assert(make_directory_if_it_does_not_exist(ROOT, recursive = true));

    // Bottom-up 24-bit: stored rows are y = 1 then y = 0, so source row 0 (first in memory) has
    // green 120. Every pixel is opaque.
    bmp24 := tprint("%/rgb.bmp", ROOT);
    assert(write_entire_file(bmp24, make_bmp(24, top_down = false)));
    ico := create_ico_file_from_bitmap_filename(bmp24);
    check_ico(ico, 4, expect_mask = false);
    assert(ico[6 + 16 * 5 + 40 + 1] == 120, "bottom-up rows keep their stored order");

    // Top-down 32-bit with a transparent top-left pixel: flipped to bottom-up, so source row 0
    // is the bottom row (green 120), and the transparent pixel lands in the last image row.
    bmp32 := tprint("%/rgba.bmp", ROOT);
    assert(write_entire_file(bmp32, make_bmp(32, top_down = true)));
    ico = create_ico_file_from_bitmap_filename(bmp32);
    check_ico(ico, 4, expect_mask = false);
    assert(ico[6 + 16 * 5 + 40 + 1] == 120, "top-down rows are flipped");
    // The top-left pixel (transparent) is in the last row of the 16x16 image.
    offset16 := u32_at(ico, 6 + 4 * 16 + 12);
    stride16 := 16 * 4;
    mask16 := offset16 + 40 + stride16 * 16;
    assert(ico[mask16 + 15 * 4] & 0x80 != 0, "the transparent pixel is masked");

    // Raw BGR pixels straight to an icon.
    pixels := u8.[10, 20, 30, 110, 20, 30, 10, 120, 30, 110, 120, 30];
    raw := create_ico_file(2, 2, 3, pixels.data);
    check_ico(raw, 3, expect_mask = false);
    // Raw BGRA with alpha 0 first: masked.
    alpha := u8.[10, 20, 30, 0, 110, 20, 30, 255, 10, 120, 30, 255, 110, 120, 30, 255];
    check_ico(create_ico_file(2, 2, 4, alpha.data), 4, expect_mask = true);

    // Files that are not usable bitmaps.
    assert(create_ico_file_from_bitmap_filename(tprint("%/missing.bmp", ROOT)) == "");
    assert(write_entire_file(tprint("%/text.bmp", ROOT), "not a bitmap at all, but long enough to have a header......"));
    assert(create_ico_file_from_bitmap_filename(tprint("%/text.bmp", ROOT)) == "");
    truncated := make_bmp(24, false);
    truncated.count -= 8;
    assert(write_entire_file(tprint("%/short.bmp", ROOT), truncated));
    assert(create_ico_file_from_bitmap_filename(tprint("%/short.bmp", ROOT)) == "", "pixel rows past the end");

    wav_files();

    assert(delete_directory(ROOT));
    print("ok\n");
}

wav_files :: () {
    // PCM stereo 16-bit at 44100 Hz, an odd-length chunk before fmt, a fact chunk, then data.
    b: String_Builder;
    append(*b, "RIFF");
    put32(*b, 0); // patched below
    append(*b, "WAVE");
    append(*b, "LIST");
    put32(*b, 3);
    append(*b, "abc");
    append(*b, 0); // pad byte of the odd chunk
    append(*b, "fmt ");
    put32(*b, 16);
    put16(*b, 1);
    put16(*b, 2);
    put32(*b, 44100);
    put32(*b, 44100 * 4);
    put16(*b, 4);
    put16(*b, 16);
    append(*b, "fact");
    put32(*b, 4);
    put32(*b, 3);
    append(*b, "data");
    put32(*b, 12);
    for 0..11 append(*b, cast(u8) (it * 3));
    bytes := builder_to_string(*b);
    riff := bytes.count - 8;
    for 0..3 bytes[4 + it] = cast,trunc(u8) (riff >> (it * 8));

    name := tprint("%/pcm.wav", ROOT);
    assert(write_entire_file(name, bytes));
    data, ok := read_entire_file(name);
    assert(ok);
    format, samples, success, extra := get_waveformatex_header(data);
    assert(success, "pcm header");
    assert(format.wFormatTag == WAVE_FORMAT_PCM && format.nChannels == 2 && format.nSamplesPerSec == 44100, "pcm format %", format);
    assert(format.nAvgBytesPerSec == 176400 && format.nBlockAlign == 4 && format.wBitsPerSample == 16);
    assert(samples.count == 12 && samples[11] == 33 && samples.data == data.data + data.count - 12, "pcm samples");
    assert(extra.wSamplesAccordingToFactChunk == 3);
    wav := load_wav_file(data);
    assert(wav.success && wav.format.nChannels == 2 && wav.format.nBlockAlign == 4 && wav.samples.count == 12);
    simple, simple_samples, simple_ok := get_wav_header(data);
    assert(simple_ok && simple.nSamplesPerSec == 44100 && simple_samples.count == 12);

    // A RIFF size beyond the file, or a data chunk running past it, is refused.
    cut := string.{ data.count - 4, data.data };
    assert(!load_wav_file(cut).success, "truncated data chunk");
    assert(!load_wav_file(string.{ 11, data.data }).success);
    assert(!load_wav_file("").success);
    assert(!load_wav_file("RIFF\x04\x00\x00\x00WAVX").success, "not WAVE");

    // IMA ADPCM mono with its samples-per-block extension.
    a: String_Builder;
    append(*a, "RIFF");
    put32(*a, 4 + 8 + 20 + 8 + 4);
    append(*a, "WAVE");
    append(*a, "fmt ");
    put32(*a, 20);
    put16(*a, WAVE_FORMAT_DVI_ADPCM);
    put16(*a, 1);
    put32(*a, 22050);
    put32(*a, 11100);
    put16(*a, 512);
    put16(*a, 4);
    put16(*a, 2);
    put16(*a, 1017);
    append(*a, "data");
    put32(*a, 4);
    append(*a, "\x01\x02\x03\x04");
    adpcm := builder_to_string(*a);
    adpcm_name := tprint("%/adpcm.wav", ROOT);
    assert(write_entire_file(adpcm_name, adpcm));
    adpcm_data := read_entire_file(adpcm_name);
    loaded := load_wav_file(adpcm_data);
    assert(loaded.success && loaded.format.wFormatTag == WAVE_FORMAT_DVI_ADPCM && loaded.format.nBlockAlign == 512, "adpcm %", loaded.format);
    assert(loaded.extra.wSamplesPerBlock == 1017 && loaded.samples == "\x01\x02\x03\x04");

    // Other compressed formats are not understood.
    adpcm_data[20] = 3; // IEEE float
    assert(!load_wav_file(adpcm_data).success);
}
