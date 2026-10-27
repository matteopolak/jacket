#!/usr/bin/env bash
# The modules under `[test] paths` in jacket.toml, built with AddressSanitizer and UBSan.
#
#   tools/ci/stdlib_jacket_sanitize.sh O0 K/N     every module's tests, the K-th of N shares
#   tools/ci/stdlib_jacket_sanitize.sh O2 K/N     only the modules in tests/compiler/optimized-modules.txt
#
# -O0 keeps every value in memory, so the sanitizers see each access; -O2 is where LLVM acts on
# undefined behaviour. Tests that open a window or use the clipboard run: Linux gets a virtual
# display (xvfb-run). The data files some tests read are written first. The exit status is non-zero
# if any test failed.
set -e
ulimit -c 0
level=${1:?usage: stdlib_jacket_sanitize.sh O0|O2 [K/N]}
share=${2:-1/1}
jacket=${JACKET:-target/debug/jacket}
display=(env)
if [[ "$(uname -s)" == Linux ]] && command -v xvfb-run > /dev/null; then
  display=(xvfb-run -a -s "-screen 0 1280x1024x24")
fi
paths=()
if [[ "$level" == O2 ]]; then
  while read -r module; do
    [[ -z "$module" || "$module" == \#* ]] && continue
    paths+=(--path "stdlib/$module")
  done < tests/compiler/optimized-modules.txt
fi
PYTHONPATH=tools python3 -B tools/portable/build_portable.py data
"${display[@]}" "$jacket" test -q --needs display --partition "hash:$share" -j 4 "${paths[@]}" -- "-$level" -sanitize address,undefined
