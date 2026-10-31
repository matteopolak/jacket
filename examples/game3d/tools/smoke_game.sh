#!/bin/sh
# Local smoke test of game3d: builds the game (with the compiler's performance report) and runs the
# scripted session `game3d --smoke`, which needs a GPU adapter but no window. Not part of CI.
#
#     examples/game3d/tools/smoke_game.sh
#     FRAME=frame.png examples/game3d/tools/smoke_game.sh     # also keep the last offscreen frame
#     REQUIRE_GPU=1 examples/game3d/tools/smoke_game.sh       # fail instead of skipping without a GPU
#
# Environment: JAIC (compiler, default jaic), FRAME (PNG to write), REQUIRE_GPU, THREADS (worker
# threads, default 3). Needs the native libraries (wgpu-native, meshoptimizer, pl_mpeg); in a
# checkout without target/native-libs set JAIC_NATIVE_LIBS to a directory that has them.

set -eu

here=$(cd "$(dirname "$0")" && pwd)
app=$(dirname "$here")
jaic=${JAIC:-jaic}

work=$(mktemp -d "${TMPDIR:-/tmp}/game3d-smoke.XXXXXX")
trap 'rm -rf "$work"' EXIT INT TERM

echo "== build"
"$jaic" "$app/game/main.jai" -exe "$work/game3d" -quiet +Performance_Report > "$work/build.txt"
sed -n '1,7p' "$work/build.txt"

echo "== smoke"
set -- --smoke --threads "${THREADS:-3}"
if [ -n "${FRAME:-}" ]; then set -- "$@" --smoke-frame "$FRAME"; fi
if [ -n "${REQUIRE_GPU:-}" ]; then set -- "$@" --require-gpu; fi
"$work/game3d" "$@"

echo "smoke_game: ok"
