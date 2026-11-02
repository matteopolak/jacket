#!/bin/sh
# Generated assets and all build products stay in the caller's external target directory.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
smoke_name=music
# shellcheck source=examples/tools/smoke-common.sh
. "$here/../../tools/smoke-common.sh"
cd "$here/../../.."
smoke_build examples/music/main.jai -exe "$T/music"
"$T/music" --generate "$work"
"$T/music" --smoke "$work" > "$work/result"
cat "$work/result"
# An independent reference mixer reads the generated WAV with Python's stdlib. This checks
# every sample against source PCM, including the final clamped interpolation sample.
python3 - "$work" <<'PY'
import pathlib, struct, sys, wave
root = pathlib.Path(sys.argv[1])
with wave.open(str(root / 'sweep.wav'), 'rb') as source:
    assert source.getframerate() == 8000 and source.getnchannels() == 1
    pcm = struct.unpack('<8000h', source.readframes(8000))
checksum = 2166136261
for i, sample in enumerate(pcm):
    for value in (int(sample * .5), int((sample + pcm[min(i + 1, 7999)]) * .25)):
        for channel in range(2):
            for byte in struct.pack('<h', value):
                checksum = ((checksum ^ byte) * 16777619) & 0xffffffff
assert f'offline frames=16000 checksum={checksum}' in (root / 'result').read_text()
assert 'headless-ui waveform lyrics queue: ok' in (root / 'result').read_text()
print('independent PCM checksum: ok')
PY
if command -v ffmpeg >/dev/null 2>&1; then
    ffmpeg -v error -i "$work/sweep.wav" -ac 2 -c:a vorbis -strict -2 \
        -metadata artist=Artist -metadata album=Album -metadata title=Tagged "$work/tagged.ogg"
    "$T/music" --list "$work" > "$work/library"
    python3 - "$work/library" <<'PYTAGS'
import pathlib, sys
lines = pathlib.Path(sys.argv[1]).read_text().splitlines()
assert len(lines) == 2 and lines[0].startswith('Artist | Album | Tagged | 8000 Hz | '), lines
print('generated Vorbis tags and sorting: ok')
PYTAGS
else
    echo 'Vorbis generation: skipped (ffmpeg absent; stdlib has no encoder)'
fi
"$T/music" --device-smoke "$work"
status=0
"$T/music" --not-a-flag > "$work/invalid" 2>&1 || status=$?
[ "$status" -ne 0 ] || { echo 'bad flags unexpectedly succeeded' >&2; exit 1; }
echo 'music smoke: ok'
