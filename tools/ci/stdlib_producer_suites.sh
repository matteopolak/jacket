#!/usr/bin/env bash
# Runs the producer suites named in SUITES (space separated) against the production build, two groups at a
# time (check_stdlib_suites.py). TOOLING_PART, when set, shards the tooling suite (K/N).
# Needs PYTHONPATH=tools, as every CI job has.
set -e
suites=(--jobs 2)
# shellcheck disable=SC2153
for suite in $SUITES; do suites+=(--suite "$suite"); done
[[ -n "${TOOLING_PART:-}" ]] && suites+=(--tooling-part "$TOOLING_PART")
run=()
if [[ "$RUNNER_OS" == Linux ]]; then
  run=(xvfb-run -a -s "-screen 0 1280x1024x24")
  font=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf
else
  font="/System/Library/Fonts/Supplemental/Arial.ttf"
fi
display=()
[[ "$RUNNER_OS" == Linux ]] && display=(--private-x11-display)
native="$(python3 -c 'from native import build_native_libs;print(build_native_libs.output_dir())')"
"${run[@]}" python3 -u tools/stdlib/check_stdlib_suites.py --jaic target/release/jaic --stdlib stdlib \
  --jailint target/debug/jailint --jaifmt target/jaifmt --native-libs "$native" --bindings-native-libs target/production-bindings/native --font "$font" "${display[@]}" "${suites[@]}"
