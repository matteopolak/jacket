#!/usr/bin/env bash
# Packages the PGO build in target/pgo/dist into jacket-$NAME (archive: .zip on Windows, .tar.gz elsewhere),
# builds the native libraries and jaifmt into it, and exports ARCHIVE and PACKAGE_DIR through GITHUB_ENV.
# Needs NAME (the matrix leg, e.g. linux-x64) and PYTHONPATH=tools; run from the checkout.
set -eo pipefail
exe=""
if [ "$RUNNER_OS" = "Windows" ]; then exe=".exe"; fi
dir="jacket-$NAME"
mkdir -p "$dir"
out="target/pgo/dist"
# Symbol tables are about a tenth of the macOS binary (docs/tools/releases.md,
# "Binary size"). Stripped after PGO and BOLT, which need them. macOS: local symbols
# only, then the ad-hoc signature arm64 requires is made again. Windows: the
# symbols are in the .pdb, which the archive does not ship.
case "$RUNNER_OS" in
  macOS) for t in jaic jailsp jailint jacket; do strip -x "$out/$t"; codesign --force -s - "$out/$t"; done ;;
  Linux) for t in jaic jailsp jailint jacket; do strip --strip-unneeded "$out/$t"; done ;;
esac
ls -l "$out"
cp "$out/jaic$exe" "$out/jailsp$exe" "$out/jailint$exe" "$out/jacket$exe" "$dir/"
# Bootstrap definitions are packaged with the standard library.
cp -R stdlib "$dir/stdlib"
# Windows: the crash runtime the standard library binds (crash-native/platform_io_windows_crash)
# is C built here, not stored in the repository. The DLL is what `jaic -interp` and a
# metaprogram load; the static archive is what built programs link, so they need no DLL.
if [ "$RUNNER_OS" = "Windows" ]; then
  cpu=x64
  if [ "$NAME" = windows-arm64 ]; then cpu=arm64; fi
  python3 tools/stdlib/build_platform_io_runtime.py --stdlib "$dir/stdlib" --cpu "$cpu" --static-library
  ls -l "$dir/stdlib/internal/runtime/crash-native"
fi
cp README.md CHANGELOG.md "$dir/"
# wgpu-native for the WebGPU module (pinned and checksum-verified, tools/bindings/webgpu.json),
# where jaic looks for native libraries: <stdlib>/../native-libs/<platform>.
# The static library is what `jaic` links, the shared one what `jaic -interp` loads.
python3 tools/bindings/fetch_wgpu_native.py --platform "$NAME" --out "$dir/native-libs/$NAME"
# The C libraries the stdlib binds (stb_image, stb_image_write, stb_image_resize,
# stb_vorbis, lz4, meshoptimizer, pl_mpeg, FreeType, Dear ImGui and MojoShader; rpmalloc
# on macOS and Linux; SDL2 on Windows x64), built from the pinned, checksum-verified
# sources in tools/native/native-libs.json, static and shared like wgpu-native, each with its
# licence under licenses/. Simp, GetRect, Sound_Player and the other modules need them.
# The script stops when a file it should have left is missing. macOS: for the oldest
# macOS the jaic binary runs on, not the runner's.
if [ "$RUNNER_OS" = "macOS" ]; then
  MACOSX_DEPLOYMENT_TARGET=11.0
  if [ "$NAME" = macos-x64 ]; then MACOSX_DEPLOYMENT_TARGET=10.12; fi
  export MACOSX_DEPLOYMENT_TARGET
fi
python3 tools/native/build_native_libs.py --platform "$NAME" --out "$dir/native-libs/$NAME"
# What Bindings_Generator's generate_bindings needs at run time: its helper library on every
# platform, and libclang from the pinned LLVM on Linux and Windows (macOS uses the Xcode
# toolchain's; docs/tools/releases.md, "Bindings_Generator"). Printed with its size.
python3 tools/ci/package_bindings_helpers.py --platform "$NAME" \
  --llvm-prefix "${LLVM_SYS_231_PREFIX:-$RUNNER_TEMP/llvm}" --out "$dir/native-libs/$NAME"
unset MACOSX_DEPLOYMENT_TARGET
ls -l "$dir/native-libs/$NAME"
# Every library the packaged stdlib binds is either in the archive, built into jaic, or
# the system's on purpose (the audio output, clipboard and allocator libraries once
# went missing from all six archives).
python3 tools/ci/check_native_libs.py --stdlib "$dir/stdlib" --platform "$NAME" \
  --native "$dir/native-libs/$NAME"
# jaifmt is a Jai program: built by the packaged jaic against the packaged stdlib, -O2
# with the default checks (not jaifmt/build.jai's unchecked build), no debug files.
"$dir/jaic$exe" jaifmt/main.jai -O2 -no_debug_info -exe "$dir/jaifmt"
# SDL3's generated source must satisfy the real repository formatter and linter.
# No missing dependency or syntax-only lint fallback is accepted by these checks.
"$dir/jaifmt$exe" --check stdlib/Extensions/SDL3 stdlib/Extensions/SDL3/tests
python3 tools/bindings/sdl3_gen.py --source target/sdl3/SDL3-3.4.18 --jaifmt "$PWD/$dir/jaifmt$exe" --check
python3 tools/bindings/sdl3_lint.py --jailint "$PWD/$dir/jailint$exe" --stdlib "$PWD/$dir/stdlib"
ls "$dir"
if [ "$RUNNER_OS" = "Windows" ]; then
  7z a -tzip "$dir.zip" "$dir" > /dev/null
  echo "ARCHIVE=$dir.zip" >> "$GITHUB_ENV"
else
  tar -czf "$dir.tar.gz" "$dir"
  echo "ARCHIVE=$dir.tar.gz" >> "$GITHUB_ENV"
fi
echo "PACKAGE_DIR=$dir" >> "$GITHUB_ENV"
