#!/bin/sh
# Runs a WASI module under wasmtime the way tools/wasi_run.ts does under node: the root is
# pre-opened, the guest's PWD is the working directory, and the environment is passed on.
#
# Usage: sh tools/wasmtime_run.sh module.wasm [args...]
# `jacket test --runner "sh tools/wasmtime_run.sh" -- -target wasm32-wasi`
if ! command -v wasmtime >/dev/null 2>&1; then
    echo "wasmtime is not installed: curl https://wasmtime.dev/install.sh -sSf | bash, or brew install wasmtime" >&2
    exit 127
fi
module=$1
shift
flags=""
for name in $(env | sed -n 's/^\([A-Za-z_][A-Za-z0-9_]*\)=.*/\1/p'); do
    [ "$name" = PWD ] || flags="$flags --env=$name"
done
# shellcheck disable=SC2086
exec wasmtime run --dir=/ --dir=/tmp::/tmp --env="PWD=$PWD" $flags "$module" "$@"
