#!/bin/sh
# Builds only into T (or the caller's scratch directory), assembles our original ROMs, checks hashes.
set -eu
here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
app=$(dirname "$here")
jaic=${JAIC:-jaic}
base=${T:-${TMPDIR:-/tmp}}
work=$(mktemp -d "$base/chip8-smoke.XXXXXX")
trap 'rm -rf "$work"' EXIT INT TERM
fail() { echo "chip8 smoke: FAILED: $*" >&2; exit 1; }
"$jaic" "$app/main.jai" -threads 2 -exe "$work/chip8" -quiet
"$jaic" "$app/tools/asm/main.jai" -threads 2 -exe "$work/asm" -quiet
for name in opcodes quirks super paddle; do
    "$work/asm" "$app/programs/$name.asm" "$work/$name.ch8"
done
check() {
    name=$1 preset=$2 want=$3
    got=$("$work/chip8" "$work/$name.ch8" --smoke --preset "$preset" --cycles 2400 --seed 42)
    [ "$got" = "$want" ] || fail "$name/$preset: got $got, expected $want"
}
for preset in modern chip8 schip; do
    check opcodes "$preset" '6006420880825620157 18050090054056575314'
    check super "$preset" '11634445405610108877 11625899729685590258'
    check paddle "$preset" '15933688504617090203 16673716456674905627'
done
check quirks modern '14209766163855909693 10390197774341902634'
check quirks schip '14209766163855909693 10390197774341902634'
check quirks chip8 '1880364998207761871 8579071839672045405'
# A replay in the playable part, before the game-over glyph, also exercises the random seed.
a=$("$work/chip8" "$work/paddle.ch8" --smoke --cycles 120 --seed 42)
b=$("$work/chip8" "$work/paddle.ch8" --smoke --cycles 120 --seed 42)
c=$("$work/chip8" "$work/paddle.ch8" --smoke --cycles 120 --seed 43)
if [ "$a" != "$b" ] || [ "$a" = "$c" ]; then
    fail 'seed/replay mismatch'
fi
# Bad inputs must report the documented exit code and diagnostic.
reject() {
    expected=$1 message=$2
    shift 2
    status=0
    "$@" >"$work/error" 2>&1 || status=$?
    [ "$status" = "$expected" ] || fail "bad input exited $status, expected $expected"
    grep -q "$message" "$work/error" || fail "missing diagnostic: $message"
}
: > "$work/empty.ch8"
reject 1 'ROM size' "$work/chip8" "$work/empty.ch8" --smoke
printf 'jp missing\n' > "$work/bad.asm"
reject 1 'unknown label' "$work/asm" "$work/bad.asm" "$work/bad.ch8"
reject 2 'unknown preset' "$work/chip8" "$work/opcodes.ch8" --preset unknown --smoke
printf '\377\377' > "$work/bad.ch8"
reject 1 'unsupported F opcode' "$work/chip8" "$work/bad.ch8" --smoke
window=1
case $(uname -s) in
    Linux) [ -n "${DISPLAY:-}" ] || window=0 ;;
    Darwin) [ "$(launchctl managername 2>/dev/null || true)" = Aqua ] || window=0 ;;
    *) window=0 ;;
esac
if [ "${SKIP_WINDOW:-0}" = 1 ] || [ "$window" = 0 ]; then
    echo 'chip8 smoke: SKIP window/audio (display unavailable or SKIP_WINDOW=1)'
else
    status=0
    "$work/chip8" "$work/paddle.ch8" --window-frames 12 >"$work/window.log" 2>&1 || status=$?
    if [ "$status" != 0 ]; then
        if grep -Eq 'AppKit activation policy failed|no display/window' "$work/window.log"; then
            echo 'chip8 smoke: SKIP window/audio (native display access unavailable)'
        else
            cat "$work/window.log" >&2
            fail "window run exited $status"
        fi
    else
        cat "$work/window.log"
        echo 'chip8 smoke: window loop passed (12 frames)'
    fi
fi
echo 'chip8 smoke: ok (12 golden hashes, seeded replay, invalid inputs)'
