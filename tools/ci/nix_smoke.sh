#!/usr/bin/env bash
# Runs the Nix-built compiler outside the checkout: interpret a program through `nix run`, build and run it
# with result/bin/jaic, format through result-1/bin/jaifmt, and check the dev shell's tools.
set -e
work="$RUNNER_TEMP/smoke"
mkdir -p "$work"
printf '#import "Basic";\nmain :: () { print("hello from nix\\n"); }\n' > "$work/hello.jai"
nix run .# -- "$work/hello.jai" -interp
jaic="$PWD/result/bin/jaic"
cd "$work"
"$jaic" hello.jai -exe hello
./hello
printf 'x:=1;\n' | "$GITHUB_WORKSPACE/result-1/bin/jaifmt" --stdin
# The inner shell expands $LLVM_PREFIX, not this one.
# shellcheck disable=SC2016
nix develop "$GITHUB_WORKSPACE" --command sh -c 'cargo --version && llvm-config --version && python3 --version && node --version && echo "$LLVM_PREFIX"'
