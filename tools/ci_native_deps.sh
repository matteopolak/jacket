#!/usr/bin/env bash
# Native helper libraries the CI jobs build before running Jai programs. Usage:
#   ci_native_deps.sh developer   the Debug, bindings and generator helpers
#   ci_native_deps.sh clipboard   Linux: the stdlib's native libraries plus the clipboard provider
#   ci_native_deps.sh full        everything the stdlib runtime and sanitizer jobs link
# Under GitHub Actions it appends JAIC_NATIVE_LIBS (and JAIC_CLIPBOARD_GUARD) to $GITHUB_ENV.
set -euo pipefail

env_file=${GITHUB_ENV:-/dev/null}
os=${RUNNER_OS:-$(uname -s)}

native_dir() {
  python3 -c 'import sys;sys.path.insert(0,"tools");import build_native_libs;print(build_native_libs.output_dir())'
}

developer() {
  TMPDIR="${RUNNER_TEMP:-${TMPDIR:-/tmp}}" PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p test_conda_sdk_notices.py -v
  python3 tools/build_developer_bindings.py
  python3 tools/build_developer_debug.py
  python3 tools/build_generator_platform.py --output-dir stdlib/internal/developer-tools
}

# $1: the JAIC_NATIVE_LIBS value to extend; empty builds the stdlib's native libraries first.
clipboard() {
  local libs=${1:-}
  if [ -z "$libs" ]; then
    python3 tools/build_native_libs.py
    libs=$(native_dir)
  fi
  python3 tools/build_portable_clipboard.py --output target/clipboard-native
  echo "JAIC_NATIVE_LIBS=$libs:$PWD/target/clipboard-native" >> "$env_file"
}

full() {
  python3 tools/build_portable_runtime.py
  python3 tools/window_render_native.py
  python3 tools/build_native_libs.py
  developer
  local audio=(--test-runtime)
  [[ "$os" == Linux ]] && audio+=(--require-device)
  python3 tools/audio_build_native.py "${audio[@]}"
  python3 tools/build_ci_native_bindings.py --output target/production-bindings/native
  local libs="$PWD/target/audio-native:$PWD/target/production-bindings/native:$PWD/target/window-native:$(native_dir)"
  echo "JAIC_NATIVE_LIBS=$libs" >> "$env_file"
  if [[ "$os" == macOS || "$os" == Darwin ]]; then
    clang -Wall -Wextra -Werror tests/window-render/clipboard-guard.m -framework AppKit -o target/clipboard-guard
    echo "JAIC_CLIPBOARD_GUARD=$PWD/target/clipboard-guard" >> "$env_file"
  fi
  if [[ "$os" == Linux ]]; then
    clipboard "$libs"
  fi
}

case ${1:-} in
  developer) developer ;;
  clipboard) clipboard ;;
  full) full ;;
  *) echo "usage: ci_native_deps.sh developer|clipboard|full" >&2; exit 2 ;;
esac
