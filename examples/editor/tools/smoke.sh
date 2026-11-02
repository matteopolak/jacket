#!/bin/sh
# Non-interactive editor session; jailsp is required when present on PATH.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
smoke_name=editor
# shellcheck source=examples/tools/smoke-common.sh
. "$here/../../tools/smoke-common.sh"
cd "$here/../../.."
if command -v jailsp >/dev/null 2>&1; then
    JAI_EDITOR_SERVER=$(command -v jailsp)
    export JAI_EDITOR_SERVER EDITOR_REQUIRE_LSP=1
else
    echo 'skip live LSP checks: jailsp not found on PATH'
    export JAI_EDITOR_SERVER=off
fi
smoke_build examples/editor/main.jai -exe "$T/editor"
"$T/editor" --smoke
