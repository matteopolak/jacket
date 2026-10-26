#!/usr/bin/env bash
# The modules under `[test] paths` in jacket.toml, once per platform: native, interpreted and native -O2,
# and with WASM=true, wasm interpreted and built. Every run happens; the exit status is non-zero if any failed.
set -e
ulimit -c 0
jacket=target/release/jacket
status=0
"$jacket" test -q || status=1
"$jacket" test -q --interp || status=1
"$jacket" test -q -- -O2 || status=1
if [[ "${WASM:-}" == true ]]; then
  "$jacket" test -q --interp -os wasm || status=1
  "$jacket" test -q -os wasm --runner "node tools/wasi_run.mjs" || status=1
fi
exit $status
