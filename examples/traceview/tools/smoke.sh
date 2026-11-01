#!/bin/sh
# Real recorder -> binary trace -> real viewer -> offscreen PNG, without a display.
set -eu
cd "$(dirname "$0")/../../.."
: "${T:?set T to an external build directory}"
JAIC=${JAIC:-jaic}
mkdir -p "$T/smoke"
"$JAIC" examples/traceview/tools/record/main.jai -exe "$T/record"
"$JAIC" examples/traceview/main.jai -exe "$T/traceview"
"$T/record" "$T/smoke/work.jipt" > "$T/smoke/record.txt"
grep -q 'recorded frames=4 zones=3' "$T/smoke/record.txt"
"$T/traceview" "$T/smoke/work.jipt" --smoke --output "$T/smoke/frame.png" > "$T/smoke/summary.txt"
grep -q 'traceview frames=4 zones=3 spans=24 selected=3 baseline=0' "$T/smoke/summary.txt"
grep -q 'zone=frame calls=1 ' "$T/smoke/summary.txt"
grep -q 'zone=work calls=4 ' "$T/smoke/summary.txt"
grep -q 'zone=leaf calls=4 ' "$T/smoke/summary.txt"
grep -q 'smoke: timeline selection, sorting, search, comparison ok' "$T/smoke/summary.txt"
if grep -q 'offscreen PNG ok' "$T/smoke/summary.txt"; then
    test -s "$T/smoke/frame.png"
else
    grep -q 'skip offscreen PNG: no WebGPU adapter' "$T/smoke/summary.txt"
    test "${REQUIRE_GPU:-0}" != 1
fi
printf 'broken' > "$T/smoke/bad.jipt"
expect_bad() {
    set +e
    "$T/traceview" "$@" > "$T/smoke/error.txt" 2>&1
    status=$?
    set -e
    test "$status" -eq 2
}
expect_bad "$T/smoke/bad.jipt" --smoke
expect_bad "$T/smoke/absent.jipt" --smoke
expect_bad "$T/smoke/work.jipt" --selected 99 --smoke
cat "$T/smoke/summary.txt"
echo 'traceview smoke ok (headless controls checked; PNG requires an adapter)'
