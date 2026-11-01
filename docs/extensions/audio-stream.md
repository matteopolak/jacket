# Audio_Stream

Seekable decoding of audio files without loading them whole: WAV (8/16/24/32-bit integer PCM, IMA ADPCM; mono or stereo) and Ogg Vorbis, into interleaved `s16` frames. `@experimental`. The [music player](../tools/dogfood.md) streams through it.

```jai
Audio :: #import "Extensions/Audio_Stream";

r: Audio.Reader;
if Audio.open(*r, "song.ogg") {
    defer Audio.close(*r);
    samples: [4096 * 2] s16;                  // whole frames only
    while true {
        frames := Audio.read(*r, samples);    // 0 at the end, -1 on error
        if frames <= 0 break;
        ...
    }
    Audio.seek(*r, r.rate * 30);              // jump to 0:30
}
```

## How it works

- `open` picks the codec by extension and copies Vorbis `ARTIST`/`ALBUM`/`TITLE` comments into the reader. `rate`, `channels` and `frames` describe the stream; `position` counts frames.
- Vorbis uses `stb_vorbis`' file API. WAV headers go through `Wav_File`; PCM reads through a 4096-frame cache, IMA keeps one compressed block and its decoded samples. Memory stays bounded by those buffers whatever the file size.
- `seek` is exact and clamps to the ends. For IMA it decodes from the start of the containing block.

## How to change it

- A new codec: a case in `open`, `read` and `seek`, plus fields on `Reader`. Keep the per-reader memory bounded.
- A `Reader` owns its file and buffers: use it on one thread, don't copy it, and `close` it before reopening (`close` is safe after a failed open).
- Float WAV and more than two channels are rejected; adding them means a conversion step in `read`.

Tests: `jacket test --path stdlib/Extensions/Audio_Stream` (generated fixtures, no audio device). Depends on `File`, `Wav_File`, `Adpcm` and `stb_vorbis`.
