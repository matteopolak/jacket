#!/usr/bin/env bash
# tests/compiler/native/simp-game: first.jai builds src/main.jai in a workspace and runs build/game.exe, first
# with the runner's Clang off PATH, then with every Clang moved away so jaic must use Visual Studio's link.exe.
# Needs JAIC (relative to the checkout), MESA_DIR and RUNNER_TEMP; Windows, Git Bash.
set -eo pipefail
# shellcheck disable=SC2153
jaic="$PWD/$JAIC"
game="$RUNNER_TEMP/simp-game"
cp -R tests/compiler/native/simp-game "$game"
mkdir -p "$game/build"
cp "$MESA_DIR"/*.dll "$game/build/"
cd "$game"
PATH="$(echo "$PATH" | tr ':' '\n' | grep -vi llvm | paste -sd:)"
export PATH
if command -v clang; then echo "clang is still on PATH"; exit 1; fi
which -a link
# First with the runner's older Clang in Program Files\LLVM (found off PATH), then
# with that and Visual Studio's bundled Clang moved away, so link.exe links.
"$jaic" first.jai -interp
vs="$(cygpath -u "$("/c/Program Files (x86)/Microsoft Visual Studio/Installer/vswhere.exe" -latest -property installationPath)")"
# A rename fails while a process of the run before still holds a file inside open (Windows
# denies it), which ends in a moment, so it is retried.
rename() {
  for _ in 1 2 3 4 5 6 7 8 9 10; do mv "$1" "$2" && return 0; sleep 3; done
  mv "$1" "$2"
}
hidden=()
for dir in "/c/Program Files/LLVM" "$vs/VC/Tools/Llvm"; do
  if [ -d "$dir" ]; then rename "$dir" "$dir.hidden"; hidden+=("$dir"); fi
done
rm -f build/game.exe
set +e
"$jaic" first.jai -interp
status=$?
set -e
for dir in "${hidden[@]}"; do rename "$dir.hidden" "$dir"; done
test "$status" = 0
ls -l build
# The same program under the interpreter (Mesa's opengl32.dll is beside jaic.exe).
"$jaic" src/main.jai -interp
