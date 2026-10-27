#!/usr/bin/env bash
# The modules under `[test] paths` in jacket.toml, once per platform: native, interpreted and native -O2,
# and with WASM=true, wasm interpreted and built. JACKET_TEST_PARTITION, when it is `hash:K/N` (jacket test
# --partition), runs only that share of the tests, so the platform's legs can split them; `all` or unset runs them all.
# Tests that open a window or use the clipboard (`needs(.DISPLAY)`) run: the platform has a display, which on Linux
# is a virtual one (xvfb-run). The data files some tests read (ZIP, archive, MPEG and Meow fixtures) are written first.
# Every run happens; the exit status is non-zero if any failed.
set -e
ulimit -c 0
jacket=target/release/jacket
status=0
display=(env)
if [[ "$(uname -s)" == Linux ]] && command -v xvfb-run > /dev/null; then
  display=(xvfb-run -a -s "-screen 0 1280x1024x24")
fi
PYTHONPATH=tools python3 -B tools/portable/build_portable.py data
partition=()
case "${JACKET_TEST_PARTITION:-all}" in
  all) ;;
  *) partition=(--partition "$JACKET_TEST_PARTITION") ;;
esac
"${display[@]}" "$jacket" test -q --needs display "${partition[@]}" || status=1
"${display[@]}" "$jacket" test -q --needs display "${partition[@]}" --interp || status=1
"${display[@]}" "$jacket" test -q --needs display "${partition[@]}" -- -O2 || status=1
if [[ "${WASM:-}" == true ]]; then
  "${display[@]}" "$jacket" test -q --needs display "${partition[@]}" --interp -os wasm || status=1
  "${display[@]}" "$jacket" test -q --needs display "${partition[@]}" -os wasm --runner "node --no-warnings tools/wasi_run.mjs" || status=1
fi
exit $status
