#!/bin/sh
# Local smoke test of sitegen: builds the program, generates the demo site, and checks the output
# the way a browser would meet it. Not part of CI; the unit tests (`jacket test` in
# examples/sitegen) cover the same ground without a built binary.
#
#     examples/sitegen/tools/smoke.sh        # needs jaic (or $JAIC)

set -eu

here=$(cd "$(dirname "$0")" && pwd)
app=$(dirname "$here")
jaic=${JAIC:-jaic}

work=$(mktemp -d "${TMPDIR:-/tmp}/sitegen-smoke.XXXXXX")
trap 'rm -rf "$work"' EXIT INT TERM

fail() {
    echo "smoke: FAILED: $*" >&2
    exit 1
}

"$jaic" "$app/main.jai" -exe "$work/sitegen" >/dev/null || fail "the program does not build"

"$work/sitegen" "$app/demo" "$work/public" >"$work/first.log" 2>&1 || fail "the demo site does not build"
grep -q "wrote 3 pages and 3 assets" "$work/first.log" || fail "unexpected summary: $(cat "$work/first.log")"

[ -f "$work/public/index.html" ] || fail "no index.html"
[ -f "$work/public/guide/intro.html" ] || fail "no guide/intro.html"

css=$(cd "$work/public" && find . -maxdepth 1 -name 'style.*.css' | head -n 1 | sed 's|^\./||')
[ -n "$css" ] || fail "the style sheet was not published under a hashed name"
grep -q "href=\"$css\"" "$work/public/index.html" || fail "index.html does not link to $css"
grep -q "href=\"../$css\"" "$work/public/guide/intro.html" || fail "guide/intro.html does not link to ../$css"
grep -q 'src="data:image/png;base64,' "$work/public/index.html" || fail "the logo was not inlined"
grep -q 'guide/intro.html' "$work/public/index.html" || fail "the link to the guide was not rewritten"

# A second build changes nothing: the same names come out.
(cd "$work/public" && find . -type f | sort) >"$work/before.txt"
"$work/sitegen" "$app/demo" "$work/public" >/dev/null 2>&1 || fail "the second build failed"
(cd "$work/public" && find . -type f | sort) >"$work/after.txt"
cmp -s "$work/before.txt" "$work/after.txt" || fail "a second build wrote different file names"

# Changing an asset changes its name.
cp -R "$app/demo" "$work/changed"
echo "p { color: red }" >>"$work/changed/style.css"
"$work/sitegen" "$work/changed" "$work/public2" >/dev/null 2>&1 || fail "the changed site did not build"
[ ! -f "$work/public2/$css" ] || fail "the changed style sheet kept its old name"

# A broken link is a warning, not a failure.
echo "[nowhere](missing.md)" >"$work/changed/broken.md"
"$work/sitegen" "$work/changed" "$work/public3" >"$work/warn.log" 2>&1 || fail "a broken link stopped the build"
grep -q "links to a page that does not exist" "$work/warn.log" || fail "no warning for a broken link"

# A bad template stops the build before anything is written.
echo "{{ nonsense }}" >"$work/changed/_template.html"
if "$work/sitegen" "$work/changed" "$work/public4" >"$work/bad.log" 2>&1; then fail "a bad template did not fail"; fi
[ ! -e "$work/public4" ] || fail "a failed build wrote output"

echo "smoke: ok"
