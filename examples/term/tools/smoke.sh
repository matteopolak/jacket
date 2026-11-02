#!/bin/sh
set -eu
here=$(cd "$(dirname "$0")" && pwd)
smoke_name=term
# shellcheck source=tools/dev/smoke-common.sh
. "$here/../../../tools/dev/smoke-common.sh"
cd "$here/../../.."
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
smoke_build examples/term/main.jai -exe "$T/term"
"$T/term" --smoke --frame "${FRAME:-$T/term-frame.png}"
