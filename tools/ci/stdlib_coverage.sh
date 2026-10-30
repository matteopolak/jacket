#!/usr/bin/env bash
# The coverage of the standard library's tests (jacket test --coverage), natively, against the recorded
# tools/ci/coverage-baseline.json: the run fails when a module, or all of them, cover more than 0.5 points fewer
# lines. The per-module table goes to the job summary. The coverage this run measured is written to
# target/coverage-baseline.json, to be uploaded or copied over the baseline.
# `--record` measures and writes tools/ci/coverage-baseline.json instead of comparing with it.
set -e
ulimit -c 0
jacket=target/release/jacket
baseline=tools/ci/coverage-baseline.json
display=(env)
if [[ "$(uname -s)" == Linux ]] && command -v xvfb-run > /dev/null; then
  display=(xvfb-run -a -s "-screen 0 1280x1024x24")
fi
PYTHONPATH=tools python3 -B tools/portable/build_portable.py data
args=(--coverage --coverage-include stdlib --coverage-by module --coverage-out target/stdlib-coverage.lcov)
if [[ "${1:-}" == --record ]]; then
  args+=(--coverage-record "$baseline")
elif ! grep -q '"lines"' "$baseline"; then
  # No baseline recorded for this leg yet: measure only. Commit the uploaded artifact as the baseline.
  echo "no $baseline: measuring without comparing"
  args+=(--coverage-record target/coverage-baseline.json)
else
  args+=(--coverage-baseline "$baseline" --coverage-record target/coverage-baseline.json)
  if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
    args+=(--coverage-summary "$GITHUB_STEP_SUMMARY")
  fi
fi
"${display[@]}" "$jacket" test -q --needs display "${args[@]}"
