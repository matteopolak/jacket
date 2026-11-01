#!/bin/sh
# Local smoke test of seek: builds the program, searches a small tree with every flag, and checks
# the output and the exit codes. Not part of CI; the unit tests (`jacket test` in examples/seek)
# cover the same ground without a built binary.
#
#     examples/seek/tools/smoke.sh        # needs jaic (or $JAIC)

set -eu

here=$(cd "$(dirname "$0")" && pwd)
app=$(dirname "$here")
jaic=${JAIC:-jaic}

work=$(mktemp -d "${TMPDIR:-/tmp}/seek-smoke.XXXXXX")
trap 'rm -rf "$work"' EXIT INT TERM

fail() {
    echo "smoke: FAILED: $*" >&2
    exit 1
}

"$jaic" "$app/main.jai" -exe "$work/seek" >/dev/null || fail "the program does not build"
seek="$work/seek"

tree="$work/tree"
mkdir -p "$tree/sub" "$tree/.hidden"
printf 'one\nTwo needle\nthree\n' >"$tree/a.txt"
printf 'needle first\nNeedle again\n' >"$tree/b.md"
printf 'deep needle\n' >"$tree/sub/c.txt"
printf '\000needle\n' >"$tree/sub/d.bin"
printf 'needle in hiding\n' >"$tree/.hidden/e.txt"

# Plain matches: path:line:col: text, in path order, without the binary file or the hidden folder.
out=$("$seek" needle "$tree") || fail "a search with matches did not exit 0"
want="$tree/a.txt:2:5: Two needle
$tree/b.md:1:1: needle first
$tree/sub/c.txt:1:6: deep needle"
[ "$out" = "$want" ] || fail "unexpected matches: $out"

# -i, -F, -l, -c, -n, --glob.
n=$("$seek" -i needle "$tree" | wc -l | tr -d ' ')
[ "$n" = 4 ] || fail "-i found $n lines, wanted 4"
printf 'a.b\naxb\n' >"$work/dots.txt"
n=$("$seek" -F 'a.b' "$work/dots.txt" | wc -l | tr -d ' ')
[ "$n" = 1 ] || fail "-F found $n lines, wanted 1"
n=$("$seek" 'a.b' "$work/dots.txt" | wc -l | tr -d ' ')
[ "$n" = 2 ] || fail "the pattern a.b found $n lines, wanted 2"
[ "$("$seek" -l needle "$tree" | wc -l | tr -d ' ')" = 3 ] || fail "-l did not name three files"
[ "$("$seek" -c -i needle "$tree/b.md")" = "$tree/b.md:2" ] || fail "-c gave the wrong count"
[ "$("$seek" -n deep "$tree")" = "$tree/sub/c.txt: deep needle" ] || fail "-n kept the line number"
[ "$("$seek" --glob '*.md' needle "$tree")" = "$tree/b.md:1:1: needle first" ] || fail "--glob did not filter"

# The thread count changes nothing.
mkdir -p "$work/many"
i=0
while [ "$i" -lt 80 ]; do
    printf 'x\nneedle %s\n' "$i" >"$work/many/f$i.txt"
    i=$((i + 1))
done
"$seek" -j 1 needle "$work/many" >"$work/j1.txt" || fail "-j 1 failed"
"$seek" -j 8 needle "$work/many" >"$work/j8.txt" || fail "-j 8 failed"
cmp -s "$work/j1.txt" "$work/j8.txt" || fail "-j 1 and -j 8 printed different output"
[ "$(wc -l <"$work/j8.txt" | tr -d ' ')" = 80 ] || fail "expected 80 matches"

# Exit codes: 1 for no match, 2 for a missing path or a bad pattern.
status=0
"$seek" zzzz "$tree" >/dev/null || status=$?
[ "$status" = 1 ] || fail "no match exited $status, wanted 1"
status=0
"$seek" needle "$tree/nope" >/dev/null 2>&1 || status=$?
[ "$status" = 2 ] || fail "a missing path exited $status, wanted 2"
status=0
"$seek" '(' "$tree" >/dev/null 2>&1 || status=$?
[ "$status" = 2 ] || fail "a bad pattern exited $status, wanted 2"

echo "smoke: ok"
