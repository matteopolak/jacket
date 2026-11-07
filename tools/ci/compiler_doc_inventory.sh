#!/usr/bin/env bash
# Opt-in proof for existing check-only doc commands, with full and assigned identities.
# Usage: tools/ci/compiler_doc_inventory.sh target/debug/jacket REPORT_DIR K/N
set -eo pipefail
if [[ $# != 3 ]]; then
  echo 'usage: compiler_doc_inventory.sh JACKET REPORT_DIR K/N' >&2
  exit 2
fi
jacket="$1"
reports="$2"
share="$3"
mkdir -p "$reports"
status=0
diagnostics=()
if [[ "${GITHUB_ACTIONS:-}" == true ]]; then diagnostics=(--diagnostics); fi
for os in linux macos windows wasm; do
  command=("$jacket" test -q --doc --check-only --os "$os")
  "${command[@]}" --list --format json \
    | python3 tools/ci/jacket_inventory.py list > "$reports/doc-$os-all.jsonl" || status=1
  "${command[@]}" --partition "count:$share" --list --format json \
    | python3 tools/ci/jacket_inventory.py list > "$reports/doc-$os-part.jsonl" || status=1
  "${command[@]}" --partition "count:$share" --format json \
    | python3 tools/ci/jacket_inventory.py outcomes "${diagnostics[@]}" \
    | tee "$reports/doc-$os-outcomes.jsonl" || status=1
done
exit "$status"
