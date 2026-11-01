#!/usr/bin/env bash
# Every stdlib module's tests (`[test] paths` in jacket.toml finds them), once per platform: native, interpreted and native -O2,
# and with WASM=true, wasm interpreted and built. JACKET_TEST_PARTITION, when it is `hash:K/N` (jacket test
# --partition), runs only that share of the tests, so the platform's legs can split them; `all` or unset runs them all.
# Tests that open a window or use the clipboard (`needs(.DISPLAY)`) run: the platform has a display, which on Linux
# is a virtual one (xvfb-run). The data files some tests read (ZIP, archive, MPEG and Meow fixtures) are written first.
# JACKET_TEST_CHANGED, a JSON list of changed files (ci_changes.py sets it only for a change that cannot
# reach beyond the stdlib's own modules), narrows every run to the tests those files can affect, before the
# partition: tools/ci/select_tests.py asks jacket which. If it cannot, every test runs.
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
only=()
if [[ -n "${JACKET_TEST_CHANGED:-}" ]]; then
  names="$(mktemp)"
  selection=0
  python3 -B tools/ci/select_tests.py --jacket "$jacket" --out "$names" || selection=$?
  if [[ $selection == 0 ]]; then
    if [[ ! -s "$names" ]]; then
      echo "stdlib_jacket_test: no stdlib test is affected by the change"
      exit 0
    fi
    while IFS= read -r name; do only+=("$name"); done < "$names"
    only+=(--exact)
  fi
fi
run() {
  "${display[@]}" "$jacket" test -q --needs display "${partition[@]}" "${only[@]}" "$@" || status=1
}
run
run --interp
run -- -O2
if [[ "${WASM:-}" == true ]]; then
  run --interp -os wasm
  run -os wasm --runner "node --no-warnings tools/wasi_run.mjs"
fi
exit $status
