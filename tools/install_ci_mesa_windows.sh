#!/usr/bin/env bash
# Unpack Mesa's software OpenGL (llvmpipe) for x64 Windows into a directory, for the window tests
# on CI runners that only have GDI's OpenGL 1.1. Usage: install_ci_mesa_windows.sh DIR
# MESA_VERSION and MESA_SHA256 pin the mesa-dist-win release (.github/workflows/windows-native.yml).
# Copy DIR/*.dll next to a program (Windows loads opengl32.dll from there before System32) and set
# GALLIUM_DRIVER=llvmpipe. See docs/tools/continuous-integration.md.
set -euo pipefail

out=${1:?usage: install_ci_mesa_windows.sh DIR}
: "${MESA_VERSION:?}" "${MESA_SHA256:?}"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
curl --fail --show-error --silent --location --proto '=https' --tlsv1.2 \
  "https://github.com/pal1000/mesa-dist-win/releases/download/$MESA_VERSION/mesa3d-$MESA_VERSION-release-msvc.7z" \
  -o "$work/mesa.7z"
echo "$MESA_SHA256  $work/mesa.7z" | sha256sum --check --strict -
# opengl32.dll is a thin front for the Gallium driver library next to it.
7z e -y -bd "-o$(cygpath -w "$work/x64")" "$(cygpath -w "$work/mesa.7z")" x64/opengl32.dll x64/libgallium_wgl.dll > /dev/null
mkdir -p "$out"
cp "$work/x64/opengl32.dll" "$work/x64/libgallium_wgl.dll" "$out/"
ls -l "$out"
