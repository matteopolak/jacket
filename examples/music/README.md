# Music

A small native music player on `Extensions/UI`. It scans a folder recursively, sorts Vorbis tags by artist/album/title (filename when untagged), and plays a queue through `Sound_Player`. WAV PCM and IMA ADPCM and Ogg Vorbis are supported. Mono is duplicated to stereo; stereo stays stereo.

```sh
# From the repository root, with jaic/jacket on PATH and JAIC_STDLIB set; T is any build directory.
jaic examples/music/main.jai -exe "$T/music"
"$T/music" ~/Music
"$T/music" --generate "$T/demo"     # synthesize a one-second sweep and lyrics
"$T/music" --list "$T/demo"         # sorted metadata without a window/device
(cd examples/music && jacket test -j 3)
sh examples/music/tools/smoke.sh
```

Click a library entry to play it and append it to the queue; `+` only appends. Click a queued entry to play it. Remove deletes an entry other than the current one. Play/Pause, Next, ±10 seconds, volume, shuffle and repeat are buttons; buttons also support the UI library's Tab/Enter keyboard navigation. Click the waveform or a lyric to seek. Repeat cycles off/all/one; an explicit Next overrides repeat-one. Shuffle uses a seeded generator and avoids immediately repeating the same queue entry. It picks from the upcoming queue suffix without replacement, then stops unless repeat-all is enabled.

`--smoke FOLDER` runs offline mixing and headless UI checks on a folder made with `--generate`. `--device-smoke FOLDER` attempts brief muted playback, pause, seek and resume, or prints a skip when the device cannot open. `--screenshot FILE FOLDER` renders an offscreen PNG using a GPU. `--frames N` closes the interactive window after N frames. Invalid flags exit nonzero. `--list` reports invalid audio and unreadable folders on stderr and exits 1; the UI displays those errors while keeping valid entries.

## How it works

`player/` owns library scanning, queue transitions, the LRC parser, a linear resampler and test fixture synthesis. `Extensions/Audio_Stream` owns bounded decoding: Vorbis uses `stb_vorbis`' file API, WAV headers go through `Wav_File`, PCM uses a 4096-frame cache and IMA uses one compressed block plus its decoded samples. Playback never reads or decodes an entire recording into memory. WAV PCM 8/16/24/32-bit integer samples convert to signed 16-bit; IMA/DVI is the ADPCM format `Wav_File` supports. Channels beyond stereo and float WAV are rejected.

`playback.jai` converts to the device rate with a continuous fractional source cursor and schedules three stereo chunks of at most 8192 frames each. Each chunk is a `Sound_Player` stream delayed by the previous chunk's remaining output frames. `Sound_Player` runs its producer on the UI thread so stream cursors and flags can be accessed there without concurrent rendering. Its release callback clears the chunk pointer before reuse. An idle UI wakes every 10 ms to fill the device. Disk stalls can underrun; device-buffered audio can remain audible briefly after pause or seek.

`player/waveform.jai` opens a separate decoder on a worker and accumulates 128 peak buckets across the complete recording. The UI reads the peaks only after joining the finished worker. Selection and shutdown join before freeing its path. `interface.jai` builds the same widget tree for a window, headless smoke and screenshots. The current lyric is selected according to the playback frame cursor.

A sidecar replaces the track's extension with `.lrc`. The parser accepts multiple `[mm:ss]`, `[mm:ss.f]`, `[mm:ss.ff]` or `[mm:ss.fff]` timestamps per line, CRLF, metadata and a signed `[offset:milliseconds]` applying to all timestamps. Malformed timestamps are skipped, negative adjusted times clamp to zero, equal timestamps preserve input order, and the last due lyric is highlighted.

## How to change it

Add queue policies in `player/queue.jai` and tests in `tests/core.jai`. Extend timestamp syntax in `player/lyrics.jai`. Keep resampling state across output chunks in `player/mix.jai`; resetting fractional position causes discontinuities. Decoder changes belong in `stdlib/Extensions/Audio_Stream` with module tests. The reader must remain on one thread and be closed before reuse. A playback `Sound_Data` must remain at a stable address until its release callback; the app is heap allocated for this reason.

## Configuration and dependencies

The CLI accepts the folder and flags above; it has no settings file or network services. `T` selects external smoke/build output; `JAIC` overrides the compiler used by the smoke script. `JAIC_STDLIB` selects this checkout's standard library. Native dependencies are those already used by `Extensions/UI` (windowing, WebGPU, FreeType) and `Sound_Player`/`stb_vorbis`. The headless path needs no display, GPU or audio device. Python 3 independently checks the smoke's mix checksum. FFmpeg is optional for an extra tagged Vorbis smoke fixture; the standard library has no Vorbis encoder, so the required generated fixture is WAV only.

The eight unit tests cover generated WAV decoding, sample-rate conversion, clipping, mono/stereo math, irregular chunk boundaries, seeking, recursive sorting/errors, lyrics, queue rules and worker waveform results. Module tests cover PCM cache boundaries, IMA blocks, malformed files and Vorbis stereo/seek using the existing authored stdlib fixture. The smoke runs the built binary, checks its checksum against a separate Python implementation, clicks UI controls and attempts muted device playback.
