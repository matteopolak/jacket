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

# The compression libraries LLVM was built with. llvm-config names them (zstd by its
# build-machine path), the archive ships neither, and LLVM's MC and Object libraries reference
# both, so empty stand-ins would not link. Built from pinned upstream sources.
ZLIB_VERSION=1.3.1
ZLIB_SHA256=9a93b2b7dfdac77ceba5a558a580e74667dd6fede4585b91eefb60f03b72df23
ZSTD_VERSION=1.5.7
ZSTD_SHA256=eb33e51f49a15e023950cd7825ca74a4a2b43db8354825ac24fc1b7ee09e6fa3

case "$("$llvm/bin/llvm-config" --host-target)" in
  x86_64-*) triple=x86_64-pc-windows-msvc ;;
  aarch64-*) triple=aarch64-pc-windows-msvc ;;
  *) echo "unexpected LLVM host target" >&2; exit 1 ;;
esac

# fetch <url> <sha256> <tar member to extract>: unpacks into $work.
fetch() {
  curl -fsSL -o "$work/source.tar.gz" "$1"
  echo "$2  $work/source.tar.gz" | sha256sum -c -
  tar -xzf "$work/source.tar.gz" -C "$work" "$3"
  rm "$work/source.tar.gz"
}

# archive <lib name> <extra clang flags> <C files...>: compiles against the static C runtime
# (/MT, like the release itself) into the release's lib/, already on llvm-sys's search path.
archive() {
  local name="$1" flags="$2" objs=() c obj
  shift 2
  mkdir -p "$work/obj-$name"
  for c in "$@"; do
    obj="$work/obj-$name/$(basename "$c" .c).obj"
    # shellcheck disable=SC2086 # $flags is a word list.
    "$clang" --target="$triple" -fms-runtime-lib=static -O2 $flags -c "$c" -o "$obj"
    objs+=("$(win "$obj")")
  done
  "$lib_tool" "/out:$(win "$llvm/lib/$name")" "${objs[@]}"
}

# 1. libxml2 (`xml2s.lib`): only LLVMWindowsManifest (lld, llvm-mt) uses it and jaic links
#    nothing from there, so an empty library satisfies the linker.
if [ ! -f "$llvm/lib/xml2s.lib" ]; then
  : > "$work/empty.c"
  archive xml2s.lib "" "$work/empty.c"
fi

# 2. zlib (`zs.lib`).
if [ ! -f "$llvm/lib/zs.lib" ]; then
  fetch "https://github.com/madler/zlib/releases/download/v$ZLIB_VERSION/zlib-$ZLIB_VERSION.tar.gz" \
    "$ZLIB_SHA256" "zlib-$ZLIB_VERSION"
  archive zs.lib "-D_CRT_SECURE_NO_WARNINGS -D_CRT_NONSTDC_NO_DEPRECATE" "$work/zlib-$ZLIB_VERSION"/*.c
fi

# 3. zstd (`zstd_static.lib`). Only lib/: the test tree has symlinks Git Bash's tar cannot
#    create. ZSTD_DISABLE_ASM: the x64 Huffman decoder in .S is GNU assembler syntax.
if [ ! -f "$llvm/lib/zstd_static.lib" ]; then
  fetch "https://github.com/facebook/zstd/releases/download/v$ZSTD_VERSION/zstd-$ZSTD_VERSION.tar.gz" \
    "$ZSTD_SHA256" "zstd-$ZSTD_VERSION/lib"
  src="$work/zstd-$ZSTD_VERSION/lib"
  archive zstd_static.lib -DZSTD_DISABLE_ASM "$src"/common/*.c "$src"/compress/*.c "$src"/decompress/*.c
fi

# 4. The llvm-config front that llvm-sys picks first (see llvm-config-shim.c).
"$clang" --target="$triple" -fms-runtime-lib=static -O2 "$here/llvm-config-shim.c" \
  -o "$llvm/bin/llvm-config-23.exe"

# 5. Every system library must now be a bare name the linker can find: one of Windows' import
#    libraries, or a file in the release's lib/.
libs="$("$llvm/bin/llvm-config-23.exe" --link-static --system-libs | tr -d '\r')"
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
