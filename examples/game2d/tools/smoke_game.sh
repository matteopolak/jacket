#!/bin/sh
# Local smoke test of the game: builds the asset tool and the game, packs the assets into a zip and
# runs the scripted session (`game2d --smoke`) with a real window's OpenGL context drawing one frame
# offscreen. Not part of CI.
#
#     examples/game2d/tools/smoke_game.sh
#     SERVER_URL=http://localhost:8080 examples/game2d/tools/smoke_game.sh   # also calls a running backend
#
# Environment: JAIC (compiler, default jaic), SERVER_URL and SERVER_ROOTS (certificate to trust for
# https), FRAME (where to keep the offscreen frame as a PNG).

set -eu

here=$(cd "$(dirname "$0")" && pwd)
app=$(dirname "$here")
jaic=${JAIC:-jaic}

work=$(mktemp -d "${TMPDIR:-/tmp}/game2d-game.XXXXXX")
trap 'rm -rf "$work"' EXIT INT TERM

echo "== build"
"$jaic" "$app/tools/assetgen/main.jai" -exe "$work/assetgen" -quiet
"$jaic" "$app/game/main.jai" -exe "$work/game2d" -quiet

echo "== pack"
# Regenerate the art and sounds into a scratch folder (the Ogg music is the committed one) and
# check that what the program draws today is what the repository carries.
mkdir "$work/assets"
cp "$app/game/assets/music.ogg" "$work/assets/"
"$work/assetgen" "$work/assets" --pack "$work/assets.zip" >/dev/null
for name in sprites.png font.png shoot.wav hit.wav explode.wav pickup.wav; do
    cmp -s "$work/assets/$name" "$app/game/assets/$name" || { echo "smoke_game: $name is not what tools/assetgen makes; run it again and commit" >&2; exit 1; }
done

echo "== smoke"
set -- --smoke --pack "$work/assets.zip"
if [ -n "${FRAME:-}" ]; then set -- "$@" --smoke-frame "$FRAME"; fi
if [ -n "${SERVER_URL:-}" ]; then set -- "$@" --server "$SERVER_URL"; fi
if [ -n "${SERVER_ROOTS:-}" ]; then set -- "$@" --roots "$SERVER_ROOTS"; fi
"$work/game2d" "$@"

echo "== autoplay"
# The whole game loop with a scripted player: title, a match, a death, a retry, the save file, and
# the score on its way to the backend when there is one.
name="auto$$"
set -- --autoplay 240 --no-audio --no-hot-reload --name "$name" --save "$work/play.sav" --pack "$work/assets.zip"
if [ -n "${SERVER_URL:-}" ]; then set -- "$@" --server "$SERVER_URL"; fi
if [ -n "${SERVER_ROOTS:-}" ]; then set -- "$@" --roots "$SERVER_ROOTS"; fi
"$work/game2d" "$@"
[ -s "$work/play.sav" ] || { echo "smoke_game: the autoplay run left no save file" >&2; exit 1; }
if [ -n "${SERVER_URL:-}" ]; then
    curl -sf "$SERVER_URL/api/leaderboard?limit=100" | grep -q "\"$name\"" || { echo "smoke_game: the autoplay score did not reach the leaderboard" >&2; exit 1; }
    echo "the autoplay score is on the leaderboard"
fi
echo "smoke_game: ok"
