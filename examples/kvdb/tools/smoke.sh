#!/bin/sh
# Local smoke test of kvdb: builds the program, drives every CLI command, damages the log the way a
# crash would, and talks to a running server with the `remote` command. Not part of CI; the unit
# tests (`jacket test` in examples/kvdb) cover the same ground without a built binary.
#
#     examples/kvdb/tools/smoke.sh        # needs jaic (or $JAIC)

set -eu

here=$(cd "$(dirname "$0")" && pwd)
app=$(dirname "$here")
jaic=${JAIC:-jaic}

work=$(mktemp -d "${TMPDIR:-/tmp}/kvdb-smoke.XXXXXX")
server_pid=""

cleanup() {
    if [ -n "$server_pid" ]; then kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi
    rm -rf "$work"
}
trap cleanup EXIT INT TERM

fail() {
    echo "smoke: FAILED: $*" >&2
    exit 1
}

expect() {
    # expect WHAT WANT GOT
    [ "$2" = "$3" ] || fail "$1: wanted '$2', got '$3'"
}

echo "== build"
"$jaic" "$app/main.jai" -exe "$work/kvdb" -quiet || fail "kvdb does not build"

kvdb="$work/kvdb"
export KVDB_FILE="$work/data.log"

echo "== commands"
"$kvdb" set greeting hello
"$kvdb" set greeting "hello, world"
"$kvdb" set gone x
"$kvdb" del gone
expect "get" "hello, world" "$("$kvdb" get greeting)"
"$kvdb" get gone && fail "a deleted key was found"
expect "scan" "greeting	hello, world" "$("$kvdb" scan gree)"
expect "export" "greeting	aGVsbG8sIHdvcmxk" "$("$kvdb" export)"

echo "== compaction"
before=$(wc -c <"$KVDB_FILE")
"$kvdb" compact >/dev/null
after=$(wc -c <"$KVDB_FILE")
[ "$after" -lt "$before" ] || fail "compaction did not shrink the log ($before -> $after)"
expect "get after compact" "hello, world" "$("$kvdb" get greeting)"

echo "== crash recovery"
valid=$(wc -c <"$KVDB_FILE" | tr -d ' ')
"$kvdb" set second 2
size=$(wc -c <"$KVDB_FILE" | tr -d ' ')
head -c $((size - 4)) "$KVDB_FILE" >"$work/torn.log"
cp "$work/torn.log" "$KVDB_FILE"
"$kvdb" get second && fail "an unfinished record was applied"
expect "log after recovery" "$valid" "$(wc -c <"$KVDB_FILE" | tr -d ' ')"
expect "older data" "hello, world" "$("$kvdb" get greeting)"

echo "== corruption"
"$kvdb" set third 3
printf 'X' | dd of="$KVDB_FILE" bs=1 seek=14 conv=notrunc 2>/dev/null
"$kvdb" set more 3 && fail "a log damaged before its end was opened"

echo "== server"
rm -f "$KVDB_FILE"
"$kvdb" serve --listen 127.0.0.1:0 >"$work/server.log" 2>&1 &
server_pid=$!
for _ in 1 2 3 4 5 6 7 8 9 10; do
    grep -q "serving" "$work/server.log" 2>/dev/null && break
    sleep 0.2
done
address=$(sed -n 's/.* on \(127\.0\.0\.1:[0-9]*\)$/\1/p' "$work/server.log")
[ -n "$address" ] || fail "the server did not start"
remote() { "$kvdb" remote --address "$address" "$@"; }
expect "remote set" "OK" "$(remote SET name Ada Lovelace)"
expect "remote get" "VALUE Ada Lovelace" "$(remote GET name)"
expect "remote del" "MISSING" "$(remote DEL nobody)"
expect "remote shutdown" "OK" "$(remote SHUTDOWN)"
wait "$server_pid" || true
server_pid=""
expect "the write reached the log" "Ada Lovelace" "$("$kvdb" get name)"

echo "smoke: ok"
