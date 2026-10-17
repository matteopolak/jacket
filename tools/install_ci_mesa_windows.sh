#!/usr/bin/env bash
# Unpack Mesa's software OpenGL (llvmpipe) for Windows into a directory, for the window tests on CI
# runners that only have GDI's OpenGL 1.1. Usage: install_ci_mesa_windows.sh CPU DIR (x64 or arm64)
# MESA_VERSION and the per-CPU MESA_SHA256_X64 / MESA_SHA256_ARM64 pin the mmozeiko/build-mesa
# release (.github/workflows/ci.yml). The archive holds one self-contained opengl32.dll with
# llvmpipe linked in; copy it next to a program (Windows loads opengl32.dll from there before
# System32) and set GALLIUM_DRIVER=llvmpipe. See docs/tools/development.md.
set -euo pipefail

cpu=${1:?usage: install_ci_mesa_windows.sh CPU DIR}
out=${2:?usage: install_ci_mesa_windows.sh CPU DIR}
: "${MESA_VERSION:?}"
case "$cpu" in
  x64) : "${MESA_SHA256_X64:?}"; sha=$MESA_SHA256_X64 ;;
  arm64) : "${MESA_SHA256_ARM64:?}"; sha=$MESA_SHA256_ARM64 ;;
  *) echo "unknown cpu: $cpu" >&2; exit 2 ;;
esac
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
curl --fail --show-error --silent --location --proto '=https' --tlsv1.2 \
  "https://github.com/mmozeiko/build-mesa/releases/download/$MESA_VERSION/mesa-llvmpipe-$cpu-$MESA_VERSION.7z" \
  -o "$work/mesa.7z"
echo "$sha  $work/mesa.7z" | sha256sum --check --strict -
mkdir -p "$out"
if command -v 7z > /dev/null; then
  7z e -y -bd "-o$(cygpath -w "$out")" "$(cygpath -w "$work/mesa.7z")" opengl32.dll > /dev/null
else
  # Windows' own bsdtar reads 7z archives; Git's GNU tar does not.
  "$(cygpath -u "$SYSTEMROOT")/System32/tar.exe" -xf "$(cygpath -w "$work/mesa.7z")" -C "$(cygpath -w "$out")" opengl32.dll
fi
ls -l "$out"
