# Authored audio fixture

`tone.ogg` is generated test input, not reference/compiler output. It was created
locally with the installed FFmpeg native Vorbis encoder:

```sh
ffmpeg -v error -f lavfi -i 'aevalsrc=0.25*sin(2*PI*440*t)|0.5*sin(2*PI*880*t):s=44100:d=0.1' -c:a vorbis -strict -2 -y tone.ogg
```

The encoder emits a granule duration of 4,416 frames, verified independently by
ffprobe (stereo, 44,100 Hz). An initial 4,410-frame assumption was corrected after
this measurement; decoded signal-energy assertions were retained. Tests check
metadata, approximate independent channel energies, page boundaries and seeking.
No corpus vendor input or supplied reference executable was used.
