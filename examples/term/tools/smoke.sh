#!/bin/sh
set -eu
cd "$(dirname "$0")/../../.."
: "${T:?set T to an external build directory}"
mkdir -p "$T/tmp"
export TMPDIR="$T/tmp"
case $(uname -s) in
    Darwin|Linux) ;;
    *) echo 'skip term: PTY requires macOS or Linux'; exit 0 ;;
esac
# Detect an unavailable PTY independently so a program regression still fails the smoke.
if command -v python3 >/dev/null 2>&1; then
    if ! python3 -c 'import os; a,b=os.openpty(); os.close(a); os.close(b)' ; then
        echo 'skip term: no pseudoterminal device is available'
        exit 0
    fi
fi
"${JAIC:-jaic}" examples/term/main.jai -exe "$T/term"
"$T/term" --smoke --frame "${FRAME:-$T/term-frame.png}"
