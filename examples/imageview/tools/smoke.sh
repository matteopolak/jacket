#!/bin/sh
set -eu
cd "$(dirname "$0")/../../.."
T=${T:-${TMPDIR:-/tmp}/imageview}
JAIC=${JAIC:-jaic}
if ! command -v "$JAIC" >/dev/null 2>&1; then
    echo 'skip: imageview smoke needs jaic in PATH'
    exit 0
fi
mkdir -p "$T"
work=$(mktemp -d "$T/smoke.XXXXXX")
trap 'rm -rf "$work"' EXIT HUP INT TERM
"$JAIC" examples/imageview/main.jai -threads 2 -exe "$work/imageview"
"$work/imageview" --help > "$work/help.txt"
if "$work/imageview" --cache-mb 0 > "$work/bad.txt" 2>&1; then
    echo 'invalid cache cap unexpectedly succeeded'
    exit 1
fi
if "$work/imageview" "$work/missing" > "$work/missing.txt" 2>&1; then
    echo 'missing folder unexpectedly succeeded'
    exit 1
fi
"$work/imageview" --smoke "$work/images"
test -s "$work/images/export.png"
test -s "$work/images/export.jpg"
echo 'imageview CLI smoke passed'
