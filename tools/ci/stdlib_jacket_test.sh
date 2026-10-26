#!/usr/bin/env bash
# The modules under `[test] paths` in jacket.toml, once per platform: native, interpreted and native -O2,
# and with WASM=true, wasm interpreted and built. JACKET_TEST_PARTITION, when it is `hash:K/N` (jacket test
# --partition), runs only that share of the tests, so the platform's legs can split them; `all` or unset runs them all.
# Every run happens; the exit status is non-zero if any failed.
set -e
ulimit -c 0
jacket=target/release/jacket
status=0
partition=()
case "${JACKET_TEST_PARTITION:-all}" in
  all) ;;
  *) partition=(--partition "$JACKET_TEST_PARTITION") ;;
esac
"$jacket" test -q "${partition[@]}" || status=1
"$jacket" test -q "${partition[@]}" --interp || status=1
"$jacket" test -q "${partition[@]}" -- -O2 || status=1
if [[ "${WASM:-}" == true ]]; then
  "$jacket" test -q "${partition[@]}" --interp -os wasm || status=1
  "$jacket" test -q "${partition[@]}" -os wasm --runner "node --no-warnings tools/wasi_run.mjs" || status=1
fi
exit $status
