#!/bin/sh
# Non-interactive editor session; jailsp is required when present on PATH.
set -eu
cd "$(dirname "$0")/../../.."
: "${T:?set T to an external build directory}"
JAIC=${JAIC:-jaic}
mkdir -p "$T/tmp"
export TMPDIR="$T/tmp"
if command -v jailsp >/dev/null 2>&1; then
    export JAI_EDITOR_SERVER="$(command -v jailsp)" EDITOR_REQUIRE_LSP=1
else
    echo 'skip live LSP checks: jailsp not found on PATH'
    export JAI_EDITOR_SERVER=off
fi
"$JAIC" examples/editor/main.jai -exe "$T/editor"
"$T/editor" --smoke
