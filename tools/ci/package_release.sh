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
cp README.md CHANGELOG.md "$dir/"
# The native libraries the packaged stdlib binds, as tools/native/native-libs.json declares them
# (docs/tools/native-libs.md), in the archive's native-libs/<platform>, where jaic looks:
# the Windows crash runtime built into stdlib/ (the DLL is what `jaic -interp` and a metaprogram
# load, the static archive what built programs link), wgpu-native (pinned and checksum-verified),
# the C libraries built from pinned sources (static and shared, each with its licence under
# licenses/), and what Bindings_Generator's generate_bindings needs at run time (docs/tools/releases.md,
# "Bindings_Generator"). It ends by checking that every library the packaged stdlib binds is in the
# archive, built into jaic, or the system's on purpose. macOS: the libraries are built for the
# oldest macOS the jaic binary runs on, not the runner's.
if [ "$RUNNER_OS" = "macOS" ]; then
  MACOSX_DEPLOYMENT_TARGET=11.0
  if [ "$NAME" = macos-x64 ]; then MACOSX_DEPLOYMENT_TARGET=10.12; fi
  export MACOSX_DEPLOYMENT_TARGET
fi
python3 tools/native/package_native_libs.py --platform "$NAME" --package "$dir" \
  --llvm-prefix "${LLVM_PREFIX:-$RUNNER_TEMP/llvm}"
unset MACOSX_DEPLOYMENT_TARGET
ls -l "$dir/native-libs/$NAME"
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
