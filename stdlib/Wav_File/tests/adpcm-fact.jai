// DVI ADPCM header built in the test: the fmt chunk's extension and the fact chunk, with an unknown chunk before them.
#import "Basic";
W :: #import "Wav_File";

put_u16 :: (out: *[..] u8, value: u16) {
    array_add(out, cast(u8) (value & 0xff));
    array_add(out, cast(u8) (value >> 8));
}

put_u32 :: (out: *[..] u8, value: u32) {
    for 0..3 array_add(out, cast(u8) ((value >> (8 * it)) & 0xff));
}

put_tag :: (out: *[..] u8, tag: string) {
    for tag array_add(out, it);
}

main :: () {
    out: [..] u8;

    put_tag(*out, "RIFF");
    // The RIFF body: WAVE (4), junk with pad (12), fmt (28), fact (12), data header and bytes (16).
    put_u32(*out, 72);
    put_tag(*out, "WAVE");

    // An unknown chunk of 3 bytes, then the pad byte that makes it even.
    put_tag(*out, "junk");
    put_u32(*out, 3);
    put_tag(*out, "xyz");
    array_add(*out, 0);

    put_tag(*out, "fmt ");
    put_u32(*out, 20);
    put_u16(*out, 0x11);
    put_u16(*out, 1);
    put_u32(*out, 8000);
    put_u32(*out, 4055);
    put_u16(*out, 256);
    put_u16(*out, 4);
    put_u16(*out, 2);
    put_u16(*out, 505);

    put_tag(*out, "fact");
    put_u32(*out, 4);
    put_u32(*out, 1010);

    put_tag(*out, "data");
    put_u32(*out, 8);
    for "ADPCMDAT" array_add(*out, it);

    file := string.{ out.count, out.data };
    format, samples, ok, extra := W.get_wav_header(file);

    print("ok=% tag=% channels=% rate=% align=% bits=% cbSize=%\n", ok, format.wFormatTag, format.nChannels, format.nSamplesPerSec, format.nBlockAlign, format.wBitsPerSample, format.cbSize);
    print("per_block=% fact=% samples=%\n", extra.wSamplesPerBlock, extra.wSamplesAccordingToFactChunk, samples);
    assert(ok && format.wFormatTag == W.WAVE_FORMAT_DVI_ADPCM && format.cbSize == 2);
    assert(format.wBitsPerSample == 4 && format.nBlockAlign == 256 && format.nAvgBytesPerSec == 4055);
    assert(extra.wSamplesPerBlock == 505 && extra.wSamplesAccordingToFactChunk == 1010);
    assert(samples == "ADPCMDAT");

    // Without a fact chunk the fact count is zero.
    short: [..] u8;

    put_tag(*short, "RIFF");
    put_u32(*short, 4 + 8 + 20 + 8 + 2);
    put_tag(*short, "WAVE");
    put_tag(*short, "fmt ");
    put_u32(*short, 20);
    put_u16(*short, 0x11);
    put_u16(*short, 1);
    put_u32(*short, 8000);
    put_u32(*short, 4055);
    put_u16(*short, 256);
    put_u16(*short, 4);
    put_u16(*short, 2);
    put_u16(*short, 505);
    put_tag(*short, "data");
    put_u32(*short, 2);
    array_add(*short, 1);
    array_add(*short, 2);
    _, samples, ok, extra = W.get_wav_header(string.{ short.count, short.data });
    print("no fact ok=% per_block=% fact=% samples=%\n", ok, extra.wSamplesPerBlock, extra.wSamplesAccordingToFactChunk, samples.count);
    assert(ok && extra.wSamplesAccordingToFactChunk == 0 && samples.count == 2);
    print("ok\n");
}
