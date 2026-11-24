// WAV headers built in the test: a LIST chunk before fmt, data sizes that are too large, and a RIFF size larger than the file.
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

// A PCM file: stereo, 44100 Hz, 16-bit, with the given RIFF and data sizes written as claimed.
make_pcm :: (riff_size: u32, data_size: u32, samples: [] u8) -> string {
    out: [..] u8;

    put_tag(*out, "RIFF");
    put_u32(*out, riff_size);
    put_tag(*out, "WAVE");

    put_tag(*out, "LIST");
    put_u32(*out, 4);
    put_tag(*out, "abcd");

    put_tag(*out, "fmt ");
    put_u32(*out, 16);
    put_u16(*out, 1);
    put_u16(*out, 2);
    put_u32(*out, 44100);
    put_u32(*out, 176400);
    put_u16(*out, 4);
    put_u16(*out, 16);

    put_tag(*out, "data");
    put_u32(*out, data_size);
    for samples array_add(*out, it);

    return .{ out.count, out.data };
}

show :: (label: string, file: string) {
    format, samples, ok := W.get_wav_header(file);
    if !ok {
        print("% ok=false\n", label);
        return;
    }

    print("% ok=% tag=% channels=% rate=% avg=% align=% bits=% cbSize=% samples=% first=% last=%\n", label, ok, format.wFormatTag, format.nChannels, format.nSamplesPerSec, format.nAvgBytesPerSec, format.nBlockAlign, format.wBitsPerSample, format.cbSize, samples.count, samples[0], samples[samples.count - 1]);
}

main :: () {
    audio: [8] u8 = .[1, 2, 3, 4, 5, 6, 250, 251];

    // Sizes are exact: RIFF size is the file size minus 8.
    // The RIFF body after its size field: WAVE (4) + LIST (12) + fmt (24) + data header and samples (16).
    exact := make_pcm(56, 8, audio);

    show("exact", exact);
    format, samples, ok, extra := W.get_wav_header(exact);

    assert(ok && format.wFormatTag == W.WAVE_FORMAT_PCM && format.nChannels == 2);
    assert(samples.count == 8 && samples == "\x01\x02\x03\x04\x05\x06\xfa\xfb");
    assert(samples.data == exact.data + 56);

    // The data chunk claims 1000 bytes; only 8 are in the file, so the view is clamped to those.
    overclaimed := make_pcm(0x7fffffff, 1000, audio);

    show("overclaimed", overclaimed);
    format, samples, ok, extra = W.get_wav_header(overclaimed);
    assert(ok && samples.count == 8);

    // A RIFF size larger than the file is tolerated as well.
    big_riff := make_pcm(0x7fffffff, 8, audio);

    show("big riff", big_riff);
    format, samples, ok, extra = W.get_wav_header(big_riff);
    assert(ok && samples.count == 8 && extra.wSamplesPerBlock == 0);

    // Not a WAVE file: the form type is wrong.
    not_wave := copy_string(exact);

    defer free(not_wave);
    not_wave[8] = #char "X";
    show("not wave", not_wave);
    format, samples, ok, extra = W.get_wav_header(not_wave);
    assert(!ok);
    print("ok\n");
}
