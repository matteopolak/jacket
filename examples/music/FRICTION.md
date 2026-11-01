# Friction log

## Fixed in the tools

- `Sound_Player.load_audio_file` reads the complete source file and its Vorbis decoder opens memory, so neither provided bounded file input for a music library. Added `Extensions/Audio_Stream` in "feat(audio-stream): decode and seek file audio with bounded buffers". It uses the stb file API, `Wav_File` headers and `Adpcm` blocks, with caller-owned sample output and exact-frame seeks. Tests are `stdlib/Extensions/Audio_Stream/tests/reader.jai`.

## Open

- `Sound_Player` has no public producer callback or queued-buffer API. Music feeds three finite PCM streams, scheduling their start with `silent_samples_at_start_of_play`. Its synchronous producer runs from the UI tick; a slow disk or UI frame can cause an underrun. A callback/ring-buffer extension that integrates with the mixer's private lock would be a larger change; no exports were added to the mirrored module.
- `Sound_Player` has no locked pause/seek accessor. Music disables its async producer and owns all stream reads/writes on the UI thread. Pausing/seeking cannot retract frames already handed to the device (roughly 40 ms).
- The waveform worker has no cancellation; selecting a new track waits for the previous bounded-memory scan to finish. Large or slow recordings can delay selection/shutdown. Metadata scanning is also synchronous.
- The stdlib has no Vorbis encoder. Test-time synthesis produces WAV; the Vorbis decoder regression uses the existing authored `Sound_Player` fixture. When FFmpeg is installed, smoke additionally creates tagged Vorbis from the generated sweep and checks tag extraction/sorting.
- WAV LIST/INFO tags are not exposed by `Wav_File`. WAV entries use filenames; Vorbis comments provide ARTIST/ALBUM/TITLE. Float WAV and more than two channels are explicitly rejected rather than silently misinterpreted.
- This checkout has no Cargo package named `jaifmt`; the formatter is the Jai program `jaifmt/main.jai`, built with the freshly compiled `jaic`.
- The sandbox denies writes to this worktree's shared git metadata. Commits are created using separate metadata under the authorized external target directory and delivered in `music.bundle`; the original branch ref is unchanged.
