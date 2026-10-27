#!/usr/bin/env bash
# Runs the packaged compiler (PACKAGE_DIR) outside the checkout: formatter, interpreter, linter, bindings
# generator, WebGPU, SDL3, the shipped native libraries and, on a static-LLVM build, built executables.
# Needs NAME (the matrix leg), LLVM (the matrix's LLVM archive; empty for none), PACKAGE_DIR and PYTHONPATH=tools.
set -eo pipefail
exe=""
if [ "$RUNNER_OS" = "Windows" ]; then exe=".exe"; fi
work="$RUNNER_TEMP/smoke"
mkdir -p "$work"
cp examples/compile-time-record/main.jai "$work/main.jai"
jaic="$PWD/$PACKAGE_DIR/jaic$exe"
jailint="$PWD/$PACKAGE_DIR/jailint$exe"
jaifmt="$PWD/$PACKAGE_DIR/jaifmt$exe"
cd "$work"
# Input without a final newline, as Homebrew's formula test sends it; the exit status
# is checked too (a crash prints nothing, which an output check alone could miss).
set +e
printf 'main::(){x:=1;}' | "$jaifmt" --stdin > fmt.out
status=$?
set -e
formatted="$(tr -d '\r' < fmt.out)"
echo "$formatted"
test "$status" = 0
test "$formatted" = "$(printf 'main :: () {\n    x := 1;\n}')"
# Built for the baseline x86-64 CPU, not this runner's: the archive runs on machines
# without AVX (the 0.4.1 jaifmt had AVX-512 and died with SIGILL on other runners).
# The `*_wide` byte searches are the exception: they are AVX2 and only reached through
# a table that picks them when the CPU has AVX2 (Extensions/Byte_Search).
if [ "$NAME" = "linux-x64" ]; then
  if objdump -d "$jaifmt" | awk '/>:$/ { symbol = $0; next }
      symbol !~ /_wide/ && /%[yz]mm|\{%k[0-7]\}/ { print symbol; print; found = 1; exit }
      END { exit !found }'; then
    echo "jaifmt uses instructions beyond baseline x86-64"; exit 1
  fi
fi
set +e
"$jaic" main.jai -interp
status=$?
set -e
test "$status" = 42
# jailint finds the packaged stdlib next to itself.
"$jailint" -D warnings main.jai
# Bindings_Generator runs from the archive alone: its helper library and libclang
# (the Xcode toolchain's on macOS) are found without a checkout (#26, #27).
printf 'typedef struct Pair { int a; float b; } Pair;\n' > bindings.h
printf '%s\n' '#import "Basic";' '#import "Bindings_Generator";' '#import "Compiler";' \
  '#run {' '    set_build_options_dc(.{do_output = false});' '    options: Generate_Bindings_Options;' \
  '    array_add(*options.source_files, "bindings.h");' '    options.generate_library_declarations = false;' \
  '    assert(generate_bindings(options, "bindings.jai"));' '}' > generate.jai
"$jaic" generate.jai
grep -q 'Pair :: struct' bindings.jai
if [ -n "$LLVM" ]; then
  "$jaic" main.jai -exe main
  set +e
  ./main$exe
  status=$?
  set -e
  test "$status" = 42
fi
# WebGPU programs need nothing beyond the archive: the triangle example links the
# packaged static wgpu-native, and `jaic -interp` loads the packaged shared one (exit 77:
# the runner has no GPU adapter, which still proves the library loaded).
cp "$GITHUB_WORKSPACE/examples/webgpu-triangle/main.jai" triangle.jai
cp "$GITHUB_WORKSPACE/stdlib/Extensions/WebGPU/tests/driven/headless.jai" .
if [ -n "$LLVM" ]; then
  "$jaic" triangle.jai -exe triangle
  ls -l triangle$exe
fi
set +e
"$jaic" headless.jai -interp
status=$?
set -e
test "$status" = 0 || test "$status" = 77
# SDL3 is source-built and packaged on all six current release targets. Audit every
# declared runtime export and artifact hash, then compile/run ABI + headless tests
# with the packaged static archive, outside the checkout and without system SDL3.
python3 "$GITHUB_WORKSPACE/tools/bindings/run_sdl3.py" --jaic "$jaic" \
  --native "$GITHUB_WORKSPACE/$PACKAGE_DIR/native-libs/$NAME" \
  --platform "$NAME" --work "$work/sdl3"
# Simp and the stb libraries need nothing beyond the archive either: `jaic -interp` loads
# the packaged shared libraries, and the built executable links the static ones.
cp "$GITHUB_WORKSPACE/stdlib/Simp/tests/image.jai" simp-image.jai
"$jaic" simp-image.jai -interp
if [ -n "$LLVM" ]; then
  "$jaic" simp-image.jai -exe simp-image
  ./simp-image$exe
fi
# The other shipped libraries, with the stdlib's own runtime tests: FreeType (a BDF font
# in memory), Dear ImGui (frames with no window), MojoShader (bytecode to GLSL), and on
# Windows x64 SDL2 (its import library and DLL from the official release).
# Sound_Player's own device test also runs without a device (jai_audio_output).
libs=(stdlib/freetype/tests/bdf-glyph.jai stdlib/ImGui/tests/headless-frames.jai stdlib/MojoShader/tests/translate.jai stdlib/Sound_Player/tests/device-lifecycle.jai)
if [ "$NAME" = "windows-x64" ]; then libs+=(stdlib/SDL/tests/geometry-timer.jai); fi
for lib_path in "${libs[@]}"; do
  lib_test=$(basename "$lib_path" .jai)
  cp "$GITHUB_WORKSPACE/$lib_path" .
  "$jaic" "$lib_test.jai" -interp
  if [ -n "$LLVM" ]; then
    "$jaic" "$lib_test.jai" -exe "$lib_test"
    # Executables load SDL2.dll from their own directory.
    if [ -f "$GITHUB_WORKSPACE/$PACKAGE_DIR/native-libs/$NAME/SDL2.dll" ]; then
      cp "$GITHUB_WORKSPACE/$PACKAGE_DIR/native-libs/$NAME/SDL2.dll" .
    fi
    "./$lib_test$exe"
  fi
done
# The allocator (rpmalloc and the callback context it binds; macOS and Linux) and, on
# Linux, the X11 clipboard library, which needs a display to run, so it is only linked.
if [ "$RUNNER_OS" != "Windows" ]; then
  cp "$GITHUB_WORKSPACE/tools/ci/native-smoke/rpmalloc.jai" rpmalloc-smoke.jai
  "$jaic" rpmalloc-smoke.jai -interp
  if [ -n "$LLVM" ]; then
    "$jaic" rpmalloc-smoke.jai -exe rpmalloc-smoke
    ./rpmalloc-smoke
  fi
fi
if [ "$RUNNER_OS" = "Linux" ] && [ -n "$LLVM" ]; then
  cp "$GITHUB_WORKSPACE/tools/ci/native-smoke/clipboard.jai" clipboard-link.jai
  "$jaic" clipboard-link.jai -exe clipboard-link
fi
# Windows without Clang on PATH, where Git's coreutils `link` is in /usr/bin: jaic
# skips it and links with Visual Studio's link.exe (0.4.2: `link: extra operand`).
if [ "$RUNNER_OS" = "Windows" ]; then
  (
    PATH="$(echo "$PATH" | tr ':' '\n' | grep -vi llvm | paste -sd:)"
    export PATH
    which -a link
    "$jaic" simp-image.jai -exe simp-image-msvc
    ./simp-image-msvc.exe
  )
fi
# (`! cmd` would not stop the script: `set -e` ignores a negated command.)
if [ "$RUNNER_OS" = "macOS" ]; then
  otool -L "$jaic"
  if otool -L "$jaic" | grep -E '/opt/homebrew|/usr/local|@rpath'; then
    echo "jaic links a library outside the system"; exit 1
  fi
fi
if [ "$RUNNER_OS" = "Linux" ]; then
  ldd "$jaic"
  if ldd "$jaic" | grep -i llvm; then echo "jaic links a shared LLVM"; exit 1; fi
fi
# Installed the usual way: unpacked elsewhere, linked from a bin folder, with no
# checkout to fall back on (the build's own stdlib path is baked in as the fallback).
if [ "$RUNNER_OS" != "Windows" ]; then
  mv "$GITHUB_WORKSPACE/stdlib" "$GITHUB_WORKSPACE/stdlib.hidden"
  cp -R "$GITHUB_WORKSPACE/$PACKAGE_DIR" "$RUNNER_TEMP/opt-jaic"
  mkdir -p "$RUNNER_TEMP/bin"
  ln -s "$RUNNER_TEMP/opt-jaic/jaic" "$RUNNER_TEMP/bin/jaic"
  set +e
  "$RUNNER_TEMP/bin/jaic" main.jai -interp
  status=$?
  # The native libraries are found beside the real stdlib behind the symlink.
  "$RUNNER_TEMP/bin/jaic" simp-image.jai -interp
  simp_status=$?
  # jaifmt is self-contained: it needs neither the stdlib nor a checkout.
  ln -s "$RUNNER_TEMP/opt-jaic/jaifmt" "$RUNNER_TEMP/bin/jaifmt"
  linked_fmt="$(printf 'main::(){x:=1;}' | "$RUNNER_TEMP/bin/jaifmt" --stdin)"
  fmt_status=$?
  set -e
  mv "$GITHUB_WORKSPACE/stdlib.hidden" "$GITHUB_WORKSPACE/stdlib"
  test "$status" = 42
  test "$simp_status" = 0
  test "$fmt_status" = 0
  test "$linked_fmt" = "$formatted"
fi
