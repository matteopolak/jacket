#!/usr/bin/env bash
# Fuzz one target for a fixed time with the settings CI uses (see docs/tools/fuzzing.md).
#
#   fuzz/run.sh <target> [seconds=600] [workers=1] [extra libFuzzer flags...]
#
# Crashes, timeouts and OOMs are collected in fuzz/artifacts/<target>/ without stopping the run
# (fork mode); the exit status is 1 when any were found.
set -euo pipefail
cd "$(dirname "$0")"
target=$1
seconds=${2:-600}
workers=${3:-1}
shift $(($# < 3 ? $# : 3))

case $target in
  lexer | parser) max_len=16384 timeout=5 ;;
  lsp | lsp_json | lsp_edits) max_len=4096 timeout=20 ;;
  jaifmt) max_len=8192 timeout=20 ;;
  check | interp | generated) max_len=8192 timeout=10 ;;
  *) echo "unknown target $target" >&2; exit 2 ;;
esac

mkdir -p "corpus/$target" "artifacts/$target"
if [ -z "$(ls -A "corpus/$target")" ] && [ "$target" != generated ]; then
  python3 seed_corpus.py "$target"
fi

# A unit is slow only when it reaches the target's timeout (libFuzzer's default, 10 s, is below
# the LSP targets' 20 s, and run.sh fails on any file in artifacts/).
# No sanitizer: the compiler is safe Rust except the interpreter's program memory, and ASan
# would also flag the interpreted program's own (intended) raw memory use. Debug assertions
# turn arithmetic overflow into panics.
cargo fuzz run --sanitizer none --debug-assertions "$target" "corpus/$target" -- \
  -dict=jai.dict \
  -artifact_prefix="artifacts/$target/" \
  -max_total_time="$seconds" \
  -timeout="$timeout" \
  -report_slow_units="$timeout" \
  -rss_limit_mb=4096 \
  -max_len="$max_len" \
  -fork="$workers" -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 \
  -print_final_stats=1 \
  "$@"

if [ -n "$(ls -A "artifacts/$target")" ]; then
  echo "findings in fuzz/artifacts/$target:" >&2
  ls -l "artifacts/$target" >&2
  exit 1
fi
