#import "Basic";
W :: #import "Wav_File";

#assert size_of(W.Waveformatex) == 18;

main :: () {
    pcm := "RIFF\x28\0\0\0WAVEfmt \x10\0\0\0\x01\0\x01\0\x40\x1f\0\0\x80\x3e\0\0\x02\0\x10\0data\x04\0\0\0\x11\x22\x33\x44";
    format, samples, ok, extra := W.get_wav_header(pcm);

    assert(ok && format.wFormatTag == W.WAVE_FORMAT_PCM && format.nSamplesPerSec == 8000);
    assert(format.nAvgBytesPerSec == 16000 && format.nBlockAlign == 2 && format.wBitsPerSample == 16 && format.cbSize == 0);
    assert(samples.count == 4 && samples.data == pcm.data + 44 && samples[2] == 0x33);
    adpcm := "RIFF\xff\xff\xff\x7fWAVEJUNK\x01\0\0\0x\0fmt \x14\0\0\0\x11\0\x02\0\x40\x1f\0\0\x10\x27\0\0\x10\0\x04\0\x02\0\x09\0fact\x04\0\0\0\x12\0\0\0data\xff\xff\xff\x7fABCD";

    format, samples, ok, extra = W.get_wav_header(adpcm);
    assert(ok && format.wFormatTag == W.WAVE_FORMAT_DVI_ADPCM && format.nChannels == 2 && format.cbSize == 2);
    assert(extra.wSamplesPerBlock == 9 && extra.wSamplesAccordingToFactChunk == 18 && samples == "ABCD");

    for truncated: 0..35 {
        format, samples, ok, extra = W.get_wav_header(string.{ truncated, pcm.data });
        assert(!ok);
    }

    bad := copy_string(pcm);

    defer free(bad);
    bad[0] = 0;
    format, samples, ok, extra = W.get_wav_header(bad);
    assert(!ok);
    bad[0] = 82;
    bad[20] = 2;
    format, samples, ok, extra = W.get_wav_header(bad);
    assert(!ok);
    bad[20] = 1;
    bad[4] = 4;
    format, samples, ok, extra = W.get_wav_header(bad);
    assert(!ok);
    print("ok\n");
}
