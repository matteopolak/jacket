#!/bin/sh
# Source after setting smoke_name. External T is retained; an owned temp T is removed on exit.
jaic=${JAIC:-jaic}
: "${smoke_name:?set smoke_name before sourcing smoke-common.sh}"
smoke_owned=0
if [ -n "${T:-}" ]; then
    mkdir -p "$T"
else
    T=$(mktemp -d "${TMPDIR:-/tmp}/${smoke_name}-smoke.XXXXXX")
    smoke_owned=1
fi
# Resolve before callers change directories. Compiler and app scratch stay together.
T=$(cd "$T" && pwd)
mkdir -p "$T/tmp"
export T TMPDIR="$T/tmp"
# shellcheck disable=SC2034
work=$(mktemp -d "$TMPDIR/${smoke_name}-work.XXXXXX")

smoke_cleanup() {
    # kvdb registers its running server here so every exit path still reaps it.
    if [ -n "${server_pid:-}" ]; then
        kill "$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
    rm -rf "$work"
    if [ "$smoke_owned" = 1 ]; then rm -rf "$T"; fi
}
trap smoke_cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

fail() {
    echo "smoke: FAILED: $*" >&2
    exit 1
}

# Keep local smoke compilation bounded while sharing the compiler selection.
smoke_build() {
    "$jaic" "$@" -j 2
}
