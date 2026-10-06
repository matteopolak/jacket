#!/usr/bin/env bash
# Make an unpacked official Windows LLVM release (clang+llvm-*-{x86_64,aarch64}-pc-windows-msvc)
# linkable by llvm-sys. Used by .github/workflows/windows-native.yml and release.yml; see
# docs/tools/llvm-setup.md ("Windows MSVC builds").
#
#   bash tools/windows-llvm/prepare.sh <llvm dir, POSIX spelling>
#
# Runs in Git Bash on a Windows runner (needs cygpath, curl, tar and the Visual Studio libraries
# that the release's clang finds by itself).
set -euo pipefail

llvm="$1"
here="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
clang="$llvm/bin/clang"
lib_tool="$llvm/bin/llvm-lib"
win() { cygpath -w "$1"; }

# Static zstd: llvm-config names the one LLVM was built with by its build-machine path, and the
# archive does not ship it. Built from the pinned upstream source with the static C runtime (/MT),
# like the release itself.
ZSTD_VERSION=1.5.7
ZSTD_SHA256=eb33e51f49a15e023950cd7825ca74a4a2b43db8354825ac24fc1b7ee09e6fa3

case "$("$llvm/bin/llvm-config" --host-target)" in
  x86_64-*) triple=x86_64-pc-windows-msvc ;;
  aarch64-*) triple=aarch64-pc-windows-msvc ;;
  *) echo "unexpected LLVM host target" >&2; exit 1 ;;
esac

# 1. libxml2: llvm-config names `xml2s.lib`, which the archive does not ship. Only
#    LLVMWindowsManifest (lld, llvm-mt) uses it and jaic links nothing from there, so an empty
#    library satisfies the linker.
if [ ! -f "$llvm/lib/xml2s.lib" ]; then
  : > "$work/empty.c"
  "$clang" --target="$triple" -c "$work/empty.c" -o "$work/empty.obj"
  "$lib_tool" "/out:$(win "$llvm/lib/xml2s.lib")" "$(win "$work/empty.obj")"
fi

# 2. zstd_static.lib, into the release's lib/ (already on llvm-sys's link search path).
if [ ! -f "$llvm/lib/zstd_static.lib" ]; then
  curl -fsSL -o "$work/zstd.tar.gz" \
    "https://github.com/facebook/zstd/releases/download/v$ZSTD_VERSION/zstd-$ZSTD_VERSION.tar.gz"
  echo "$ZSTD_SHA256  $work/zstd.tar.gz" | sha256sum -c -
  tar -xzf "$work/zstd.tar.gz" -C "$work"
  src="$work/zstd-$ZSTD_VERSION/lib"
  mkdir -p "$work/zstd-obj"
  objs=()
  for c in "$src"/common/*.c "$src"/compress/*.c "$src"/decompress/*.c; do
    obj="$work/zstd-obj/$(basename "$c" .c).obj"
    # ZSTD_DISABLE_ASM: the x64 Huffman decoder in .S is GNU assembler syntax.
    "$clang" --target="$triple" -fms-runtime-lib=static -O2 -DZSTD_DISABLE_ASM -c "$c" -o "$obj"
    objs+=("$(win "$obj")")
  done
  "$lib_tool" "/out:$(win "$llvm/lib/zstd_static.lib")" "${objs[@]}"
fi

# 3. The llvm-config front that llvm-sys picks first (see llvm-config-shim.c).
"$clang" --target="$triple" -fms-runtime-lib=static -O2 "$here/llvm-config-shim.c" \
  -o "$llvm/bin/llvm-config-231.exe"

# 4. Every system library must now be a bare name the linker can find: one of Windows' import
#    libraries, or a file in the release's lib/.
libs="$("$llvm/bin/llvm-config-231.exe" --link-static --system-libs | tr -d '\r')"
echo "system libraries (via shim): $libs"
status=0
for name in $libs; do
  case "$name" in
    */* | *\\* | *:*) echo "still a path: $name" >&2; status=1 ;;
    psapi.lib | shell32.lib | ole32.lib | uuid.lib | advapi32.lib | ws2_32.lib | ntdll.lib) ;;
    *) [ -f "$llvm/lib/$name" ] || { echo "missing from $llvm/lib: $name" >&2; status=1; } ;;
  esac
done
exit $status
